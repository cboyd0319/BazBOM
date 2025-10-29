#!/usr/bin/env python3
"""VulnCheck KEV enrichment module (optional).

This module enriches vulnerability findings with VulnCheck exploit intelligence
data for detailed exploit maturity and attack vector information.

Note: This is an optional enrichment source that requires a VulnCheck API key.
Free tier available at https://vulncheck.com/
"""

import json
import os
import sys
from typing import Dict, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class VulnCheckEnricher:
    """Enrich vulnerabilities with VulnCheck exploit intelligence."""

    API_URL = "https://api.vulncheck.com/v3/index/vulncheck-kev"

    def __init__(self, api_key: Optional[str] = None):
        """Initialize VulnCheck enricher.
        
        Args:
            api_key: VulnCheck API key (optional). If not provided, will try to
                    read from VULNCHECK_API_KEY environment variable.
        """
        self.api_key = api_key or os.getenv("VULNCHECK_API_KEY")
        self._cache = {}

    def get_exploit_status(self, cve_id: str) -> Dict:
        """Get detailed exploit status from VulnCheck.
        
        Args:
            cve_id: CVE identifier (e.g., "CVE-2021-44228")
            
        Returns:
            Dictionary with exploit intelligence data
            
        Raises:
            ValueError: If cve_id is invalid
            RuntimeError: If API key is not configured
        """
        if not cve_id:
            raise ValueError("CVE ID cannot be empty")
        
        if not isinstance(cve_id, str):
            raise TypeError(f"CVE ID must be string, got {type(cve_id)}")
        
        if not cve_id.startswith("CVE-"):
            raise ValueError(f"Invalid CVE format: {cve_id}")

        # Check cache
        if cve_id in self._cache:
            return self._cache[cve_id]

        # Return basic result if no API key configured
        if not self.api_key:
            result = {
                "exploit_available": False,
                "exploit_maturity": "unknown",
                "attack_vector": "unknown",
                "weaponized": False,
                "note": "VulnCheck API key not configured"
            }
            self._cache[cve_id] = result
            return result

        headers = {
            "Authorization": f"Bearer {self.api_key}",
            "Accept": "application/json"
        }

        try:
            response = requests.get(
                f"{self.API_URL}",
                params={"cve": cve_id},
                headers=headers,
                timeout=30
            )

            # Handle various response codes
            if response.status_code == 401:
                raise RuntimeError("VulnCheck API authentication failed - check API key")
            elif response.status_code == 403:
                raise RuntimeError("VulnCheck API access forbidden - check API permissions")
            elif response.status_code == 429:
                print(f"Warning: VulnCheck rate limit exceeded for {cve_id}", file=sys.stderr)
                result = {
                    "exploit_available": False,
                    "exploit_maturity": "unknown",
                    "error": "Rate limit exceeded"
                }
                self._cache[cve_id] = result
                return result
            
            response.raise_for_status()
            
            data = response.json()
            
            # Validate response structure
            if not isinstance(data, dict):
                raise ValueError(f"Invalid VulnCheck response: expected dict, got {type(data)}")
            
            # Parse VulnCheck response format
            # Note: Actual field names may vary based on VulnCheck API version
            vulns = data.get("data", [])
            
            if vulns:
                vuln_data = vulns[0] if isinstance(vulns, list) else vulns
                
                result = {
                    "exploit_available": vuln_data.get("exploit_available", False),
                    "exploit_maturity": vuln_data.get("exploit_maturity", "unknown"),
                    "attack_vector": vuln_data.get("attack_vector", "unknown"),
                    "weaponized": vuln_data.get("weaponized", False),
                    "exploit_type": vuln_data.get("exploit_type", ""),
                    "added_date": vuln_data.get("date_added", ""),
                    "due_date": vuln_data.get("due_date", ""),
                    "ransomware_use": vuln_data.get("ransomware_campaign_use", False)
                }
            else:
                result = {
                    "exploit_available": False,
                    "exploit_maturity": "none",
                    "attack_vector": "unknown",
                    "weaponized": False
                }
            
            self._cache[cve_id] = result
            return result

        except requests.RequestException as e:
            print(f"Warning: VulnCheck query failed for {cve_id}: {e}", file=sys.stderr)
            result = {
                "exploit_available": False,
                "exploit_maturity": "unknown",
                "attack_vector": "unknown",
                "weaponized": False,
                "error": str(e)
            }
            self._cache[cve_id] = result
            return result

    def enrich_finding(self, finding: Dict) -> Dict:
        """Add VulnCheck exploit intelligence to vulnerability finding.
        
        Args:
            finding: Vulnerability finding dictionary
            
        Returns:
            Enhanced finding with exploit data added
            
        Raises:
            TypeError: If finding is not a dictionary
        """
        if not isinstance(finding, dict):
            raise TypeError(f"Finding must be dict, got {type(finding)}")

        # Extract CVE ID
        cve_id = finding.get("cve") or finding.get("id") or finding.get("vulnerability", {}).get("id")
        
        if not cve_id or not cve_id.startswith("CVE-"):
            finding["exploit"] = {
                "exploit_available": False,
                "exploit_maturity": "unknown",
                "weaponized": False
            }
            return finding

        try:
            exploit_data = self.get_exploit_status(cve_id)
            finding["exploit"] = exploit_data
            
            # Boost priority if weaponized exploit exists
            if exploit_data.get("weaponized"):
                if "priority" not in finding or finding["priority"] not in ["P0-IMMEDIATE", "IMMEDIATE"]:
                    finding["priority"] = "P1-CRITICAL"
                finding["exploit_context"] = "[WARNING] WEAPONIZED EXPLOIT AVAILABLE"
        except Exception as e:
            print(f"Warning: Failed to enrich {cve_id} with VulnCheck data: {e}", file=sys.stderr)
            finding["exploit"] = {
                "exploit_available": False,
                "exploit_maturity": "unknown",
                "weaponized": False
            }

        return finding


def main():
    """CLI for testing VulnCheck enrichment."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Query VulnCheck for exploit intelligence"
    )
    parser.add_argument(
        "cve_id",
        help="CVE identifier to check (e.g., CVE-2021-44228)"
    )
    parser.add_argument(
        "--api-key",
        help="VulnCheck API key (or set VULNCHECK_API_KEY env var)"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        enricher = VulnCheckEnricher(api_key=args.api_key)
        result = enricher.get_exploit_status(args.cve_id)
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            if result.get("exploit_available"):
                print(f"[OK] Exploit intelligence found for {args.cve_id}")
                print(f"   Exploit Available: Yes")
                print(f"   Exploit Maturity: {result['exploit_maturity']}")
                print(f"   Attack Vector: {result['attack_vector']}")
                print(f"   Weaponized: {'Yes' if result.get('weaponized') else 'No'}")
                if result.get("ransomware_use"):
                    print(f"   [WARNING] Used in Ransomware Campaigns")
            else:
                print(f"ℹ  No exploit intelligence found for {args.cve_id}")
                if result.get("note"):
                    print(f"   Note: {result['note']}")
        
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
