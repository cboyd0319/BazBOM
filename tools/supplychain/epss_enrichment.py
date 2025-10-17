#!/usr/bin/env python3
"""EPSS (Exploit Prediction Scoring System) enrichment module.

This module enriches vulnerability findings with EPSS scores from FIRST.org
to predict the likelihood of exploitation.
"""

import json
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class EPSSEnricher:
    """Enrich vulnerabilities with EPSS scores."""

    EPSS_API_URL = "https://api.first.org/data/v1/epss"
    BATCH_SIZE = 100  # API supports batch queries
    CACHE_TTL_HOURS = 24

    def __init__(self, cache_dir: str = ".bazel-cache/epss"):
        """Initialize EPSS enricher.
        
        Args:
            cache_dir: Directory to cache EPSS scores
        """
        self.cache_dir = cache_dir
        self._cache = {}
        self._cache_loaded = False

    def _load_cache(self) -> Dict[str, Dict]:
        """Load EPSS scores from cache.
        
        Returns:
            Dictionary mapping CVE IDs to EPSS data
        """
        if self._cache_loaded:
            return self._cache

        cache_file = Path(self.cache_dir) / "epss_cache.json"
        
        if cache_file.exists():
            try:
                cache_age = datetime.now() - datetime.fromtimestamp(cache_file.stat().st_mtime)
                if cache_age < timedelta(hours=self.CACHE_TTL_HOURS):
                    with open(cache_file, 'r', encoding='utf-8') as f:
                        self._cache = json.load(f)
                        self._cache_loaded = True
            except (json.JSONDecodeError, IOError) as e:
                print(f"Warning: Failed to load EPSS cache: {e}", file=sys.stderr)

        return self._cache

    def _save_cache(self) -> None:
        """Save EPSS scores to cache."""
        cache_file = Path(self.cache_dir) / "epss_cache.json"
        
        try:
            cache_file.parent.mkdir(parents=True, exist_ok=True)
            with open(cache_file, 'w', encoding='utf-8') as f:
                json.dump(self._cache, f, indent=2)
        except IOError as e:
            print(f"Warning: Failed to save EPSS cache: {e}", file=sys.stderr)

    def fetch_epss_scores(self, cve_list: List[str]) -> Dict[str, Dict]:
        """Fetch EPSS scores for multiple CVEs (batched).
        
        Args:
            cve_list: List of CVE identifiers
            
        Returns:
            Dictionary mapping CVE IDs to EPSS data
            
        Raises:
            ValueError: If cve_list is empty or contains invalid CVEs
            requests.RequestException: If API request fails
        """
        if not cve_list:
            return {}
        
        if not isinstance(cve_list, list):
            raise TypeError(f"cve_list must be list, got {type(cve_list)}")
        
        # Validate CVE format
        for cve in cve_list:
            if not isinstance(cve, str):
                raise TypeError(f"CVE ID must be string, got {type(cve)}")
            if not cve.startswith("CVE-"):
                raise ValueError(f"Invalid CVE format: {cve}")

        # Load cache
        self._load_cache()
        
        scores = {}
        cves_to_fetch = []

        # Check cache first
        for cve in cve_list:
            if cve in self._cache:
                scores[cve] = self._cache[cve]
            else:
                cves_to_fetch.append(cve)

        # Fetch uncached CVEs in batches
        for i in range(0, len(cves_to_fetch), self.BATCH_SIZE):
            batch = cves_to_fetch[i:i + self.BATCH_SIZE]

            try:
                # Query API
                params = {"cve": ",".join(batch)}
                response = requests.get(self.EPSS_API_URL, params=params, timeout=30)
                response.raise_for_status()

                data = response.json()
                
                # Validate response structure
                if not isinstance(data, dict):
                    raise ValueError(f"Invalid EPSS API response: expected dict, got {type(data)}")
                
                for entry in data.get("data", []):
                    cve = entry.get("cve")
                    if not cve:
                        continue
                    
                    epss_score = entry.get("epss")
                    percentile = entry.get("percentile")
                    
                    # Validate and convert scores
                    try:
                        epss_float = float(epss_score) if epss_score else 0.0
                        percentile_float = float(percentile) if percentile else 0.0
                    except (ValueError, TypeError) as e:
                        print(f"Warning: Invalid EPSS score for {cve}: {e}", file=sys.stderr)
                        continue
                    
                    score_data = {
                        "epss_score": epss_float,
                        "epss_percentile": percentile_float,
                        "date": entry.get("date", "")
                    }
                    
                    scores[cve] = score_data
                    self._cache[cve] = score_data

            except requests.RequestException as e:
                print(f"Warning: EPSS API request failed for batch {i//self.BATCH_SIZE + 1}: {e}", file=sys.stderr)
                # Add empty entries for failed CVEs
                for cve in batch:
                    if cve not in scores:
                        scores[cve] = {
                            "epss_score": 0.0,
                            "epss_percentile": 0.0,
                            "date": "",
                            "error": str(e)
                        }

        # Save updated cache
        if cves_to_fetch:
            self._save_cache()

        return scores

    def get_priority_level(self, epss_score: float) -> str:
        """Map EPSS score to priority level.
        
        Args:
            epss_score: EPSS score (0.0-1.0)
            
        Returns:
            Priority level string
            
        Raises:
            TypeError: If epss_score is not numeric
            ValueError: If epss_score is out of range
        """
        if not isinstance(epss_score, (int, float)):
            raise TypeError(f"EPSS score must be numeric, got {type(epss_score)}")
        
        if not 0.0 <= epss_score <= 1.0:
            raise ValueError(f"EPSS score must be between 0.0 and 1.0, got {epss_score}")
        
        if epss_score >= 0.75:
            return "CRITICAL"  # Top 25% most likely
        elif epss_score >= 0.50:
            return "HIGH"
        elif epss_score >= 0.25:
            return "MEDIUM"
        else:
            return "LOW"

    def enrich_findings(self, findings: List[Dict]) -> List[Dict]:
        """Add EPSS scores to all findings.
        
        Args:
            findings: List of vulnerability finding dictionaries
            
        Returns:
            Enhanced findings with EPSS data added
            
        Raises:
            TypeError: If findings is not a list
        """
        if not isinstance(findings, list):
            raise TypeError(f"Findings must be list, got {type(findings)}")
        
        if not findings:
            return findings

        # Extract CVE IDs from findings
        cve_list = []
        for finding in findings:
            if not isinstance(finding, dict):
                print(f"Warning: Skipping non-dict finding: {type(finding)}", file=sys.stderr)
                continue
            
            cve = finding.get("cve") or finding.get("id") or finding.get("vulnerability", {}).get("id")
            if cve and cve.startswith("CVE-"):
                cve_list.append(cve)

        if not cve_list:
            return findings

        # Fetch EPSS scores for all CVEs
        try:
            epss_scores = self.fetch_epss_scores(cve_list)
        except Exception as e:
            print(f"Error: Failed to fetch EPSS scores: {e}", file=sys.stderr)
            # Continue with empty scores rather than failing completely
            epss_scores = {}

        # Enrich findings
        for finding in findings:
            if not isinstance(finding, dict):
                continue
                
            cve = finding.get("cve") or finding.get("id") or finding.get("vulnerability", {}).get("id")
            if cve and cve in epss_scores:
                epss_data = epss_scores[cve]
                finding["epss"] = epss_data
                
                epss_score = epss_data["epss_score"]
                finding["exploitation_probability"] = f"{epss_score * 100:.1f}%"
                
                try:
                    finding["epss_priority"] = self.get_priority_level(epss_score)
                except (TypeError, ValueError) as e:
                    print(f"Warning: Failed to calculate priority for {cve}: {e}", file=sys.stderr)
                    finding["epss_priority"] = "UNKNOWN"

        return findings

    def enrich_finding(self, finding: Dict) -> Dict:
        """Add EPSS score to a single finding.
        
        Args:
            finding: Vulnerability finding dictionary
            
        Returns:
            Enhanced finding with EPSS data
        """
        return self.enrich_findings([finding])[0] if finding else finding


def main():
    """CLI for testing EPSS enrichment."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Query EPSS scores for CVEs"
    )
    parser.add_argument(
        "cve_ids",
        nargs="+",
        help="CVE identifiers to check (e.g., CVE-2021-44228)"
    )
    parser.add_argument(
        "--cache-dir",
        default=".bazel-cache/epss",
        help="Directory for caching EPSS scores"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        enricher = EPSSEnricher(cache_dir=args.cache_dir)
        results = enricher.fetch_epss_scores(args.cve_ids)
        
        if args.json:
            print(json.dumps(results, indent=2))
        else:
            for cve_id, data in results.items():
                score = data.get("epss_score", 0)
                percentile = data.get("epss_percentile", 0)
                priority = enricher.get_priority_level(score)
                
                print(f"\n{cve_id}:")
                print(f"  EPSS Score: {score:.5f} ({score*100:.2f}%)")
                print(f"  Percentile: {percentile:.5f} (top {(1-percentile)*100:.2f}%)")
                print(f"  Priority: {priority}")
                print(f"  Date: {data.get('date', 'N/A')}")
        
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
