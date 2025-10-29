#!/usr/bin/env python3
"""KEV (Known Exploited Vulnerabilities) enrichment module.

This module enriches vulnerability findings with CISA KEV (Known Exploited 
Vulnerabilities) catalog data to identify CVEs that are actively exploited 
in the wild.
"""

import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class KEVEnricher:
    """Enrich vulnerabilities with CISA KEV data."""

    KEV_CATALOG_URL = "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json"
    CACHE_TTL_HOURS = 24

    def __init__(self, cache_dir: str = ".bazel-cache/kev"):
        """Initialize KEV enricher.
        
        Args:
            cache_dir: Directory to cache KEV catalog data
        """
        self.cache_dir = cache_dir
        self._kev_catalog = None
        self._kev_index = None

    def fetch_kev_catalog(self) -> Dict:
        """Download latest KEV catalog with caching.
        
        Returns:
            KEV catalog data as dictionary
            
        Raises:
            requests.RequestException: If download fails
            json.JSONDecodeError: If response is not valid JSON
            FileNotFoundError: If cache directory cannot be created
        """
        cache_file = Path(self.cache_dir) / "kev_catalog.json"

        # Check cache freshness
        if cache_file.exists():
            cache_age = datetime.now() - datetime.fromtimestamp(cache_file.stat().st_mtime)
            if cache_age < timedelta(hours=self.CACHE_TTL_HOURS):
                try:
                    with open(cache_file, 'r', encoding='utf-8') as f:
                        return json.load(f)
                except (json.JSONDecodeError, IOError) as e:
                    print(f"Warning: Failed to read KEV cache: {e}", file=sys.stderr)
                    # Continue to fetch fresh data

        # Fetch fresh data
        try:
            response = requests.get(self.KEV_CATALOG_URL, timeout=30)
            response.raise_for_status()
            data = response.json()
        except requests.RequestException as e:
            # If we have stale cache, use it as fallback
            if cache_file.exists():
                print(f"Warning: KEV fetch failed, using stale cache: {e}", file=sys.stderr)
                with open(cache_file, 'r', encoding='utf-8') as f:
                    return json.load(f)
            raise RuntimeError(f"Failed to fetch KEV catalog and no cache available: {e}")

        # Validate data structure
        if not isinstance(data, dict):
            raise ValueError(f"Invalid KEV catalog format: expected dict, got {type(data)}")
        if "vulnerabilities" not in data:
            raise ValueError("Invalid KEV catalog: missing 'vulnerabilities' field")

        # Cache it
        try:
            cache_file.parent.mkdir(parents=True, exist_ok=True)
            with open(cache_file, 'w', encoding='utf-8') as f:
                json.dump(data, f, indent=2)
        except IOError as e:
            print(f"Warning: Failed to cache KEV data: {e}", file=sys.stderr)

        return data

    def _build_kev_index(self) -> Dict[str, Dict]:
        """Build index of KEV catalog for fast lookups.
        
        Returns:
            Dictionary mapping CVE IDs to KEV entries
        """
        if not self._kev_catalog:
            self._kev_catalog = self.fetch_kev_catalog()

        index = {}
        for vuln in self._kev_catalog.get("vulnerabilities", []):
            cve_id = vuln.get("cveID")
            if cve_id:
                index[cve_id] = vuln

        return index

    def is_known_exploited(self, cve_id: str) -> Optional[Dict]:
        """Check if CVE is in CISA KEV catalog.
        
        Args:
            cve_id: CVE identifier (e.g., "CVE-2021-44228")
            
        Returns:
            Dictionary with KEV data if found, or dict with in_kev=False
            
        Raises:
            ValueError: If cve_id is empty or invalid format
        """
        if not cve_id:
            raise ValueError("CVE ID cannot be empty")
        
        if not isinstance(cve_id, str):
            raise TypeError(f"CVE ID must be string, got {type(cve_id)}")

        # Build index on first use
        if self._kev_index is None:
            self._kev_index = self._build_kev_index()

        # Look up CVE in index
        if cve_id in self._kev_index:
            vuln = self._kev_index[cve_id]
            return {
                "in_kev": True,
                "date_added": vuln.get("dateAdded", ""),
                "due_date": vuln.get("dueDate", ""),
                "required_action": vuln.get("requiredAction", ""),
                "notes": vuln.get("notes", ""),
                "vulnerability_name": vuln.get("vulnerabilityName", ""),
                "vendor_project": vuln.get("vendorProject", ""),
                "product": vuln.get("product", ""),
                "short_description": vuln.get("shortDescription", "")
            }

        return {"in_kev": False}

    def enrich_finding(self, finding: Dict) -> Dict:
        """Add KEV context to vulnerability finding.
        
        Args:
            finding: Vulnerability finding dictionary
            
        Returns:
            Enhanced finding with KEV data added
            
        Raises:
            TypeError: If finding is not a dictionary
        """
        if not isinstance(finding, dict):
            raise TypeError(f"Finding must be dict, got {type(finding)}")

        # Extract CVE ID from various possible locations
        cve_id = finding.get("cve") or finding.get("id") or finding.get("vulnerability", {}).get("id")
        
        if not cve_id or not cve_id.startswith("CVE-"):
            finding["kev"] = {"in_kev": False}
            return finding

        try:
            kev_data = self.is_known_exploited(cve_id)
            finding["kev"] = kev_data

            # Boost severity if in KEV
            if kev_data["in_kev"]:
                finding["effective_severity"] = "CRITICAL"
                finding["priority"] = "IMMEDIATE"
                finding["kev_context"] = f"[WARNING] ACTIVELY EXPLOITED: {kev_data['vulnerability_name']}"
        except (ValueError, TypeError) as e:
            print(f"Warning: Failed to enrich {cve_id} with KEV data: {e}", file=sys.stderr)
            finding["kev"] = {"in_kev": False}

        return finding


def main():
    """CLI for testing KEV enrichment."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Query CISA KEV catalog for CVE status"
    )
    parser.add_argument(
        "cve_id",
        help="CVE identifier to check (e.g., CVE-2021-44228)"
    )
    parser.add_argument(
        "--cache-dir",
        default=".bazel-cache/kev",
        help="Directory for caching KEV catalog"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        enricher = KEVEnricher(cache_dir=args.cache_dir)
        result = enricher.is_known_exploited(args.cve_id)
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            if result["in_kev"]:
                print(f"[OK] {args.cve_id} IS in CISA KEV catalog")
                print(f"   Name: {result['vulnerability_name']}")
                print(f"   Vendor/Product: {result['vendor_project']} / {result['product']}")
                print(f"   Date Added: {result['date_added']}")
                print(f"   Due Date: {result['due_date']}")
                print(f"   Required Action: {result['required_action']}")
            else:
                print(f" {args.cve_id} is NOT in CISA KEV catalog")
        
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
