#!/usr/bin/env python3
"""GHSA (GitHub Security Advisory) enrichment module.

This module enriches vulnerability findings with GitHub Security Advisory data
to provide ecosystem-specific remediation guidance.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


class GHSAEnricher:
    """Enrich vulnerabilities with GitHub Security Advisory data."""

    GRAPHQL_URL = "https://api.github.com/graphql"

    def __init__(self, github_token: Optional[str] = None):
        """Initialize GHSA enricher.
        
        Args:
            github_token: GitHub personal access token (optional, but recommended
                         for higher rate limits). If not provided, will try to read
                         from GITHUB_TOKEN environment variable.
        """
        self.token = github_token or os.getenv("GITHUB_TOKEN")
        self._cache = {}

    def query_advisory(self, cve_id: str) -> Dict:
        """Fetch GHSA data via GraphQL.
        
        Args:
            cve_id: CVE identifier (e.g., "CVE-2021-44228")
            
        Returns:
            Dictionary with GHSA data, or empty dict if not found
            
        Raises:
            ValueError: If cve_id is invalid
            requests.RequestException: If API request fails
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

        query = """
        query($cve: String!) {
          securityAdvisories(first: 1, identifier: {type: CVE, value: $cve}) {
            nodes {
              ghsaId
              summary
              description
              severity
              publishedAt
              updatedAt
              withdrawnAt
              permalink
              vulnerabilities(first: 10) {
                nodes {
                  package {
                    name
                    ecosystem
                  }
                  vulnerableVersionRange
                  firstPatchedVersion {
                    identifier
                  }
                }
              }
              references {
                url
              }
            }
          }
        }
        """

        headers = {}
        if self.token:
            headers["Authorization"] = f"bearer {self.token}"

        try:
            response = requests.post(
                self.GRAPHQL_URL,
                json={"query": query, "variables": {"cve": cve_id}},
                headers=headers,
                timeout=30
            )
            response.raise_for_status()

            data = response.json()
            
            # Check for GraphQL errors
            if "errors" in data:
                error_messages = [e.get("message", "Unknown error") for e in data["errors"]]
                raise RuntimeError(f"GraphQL errors: {'; '.join(error_messages)}")
            
            advisories = data.get("data", {}).get("securityAdvisories", {}).get("nodes", [])
            
            if advisories:
                advisory = advisories[0]
                
                # Extract vulnerability details
                vulnerabilities = []
                for vuln_node in advisory.get("vulnerabilities", {}).get("nodes", []):
                    package_info = vuln_node.get("package", {})
                    first_patched = vuln_node.get("firstPatchedVersion")
                    
                    vuln_detail = {
                        "package_name": package_info.get("name", ""),
                        "ecosystem": package_info.get("ecosystem", ""),
                        "vulnerable_version_range": vuln_node.get("vulnerableVersionRange", ""),
                        "first_patched_version": first_patched.get("identifier", "") if first_patched else ""
                    }
                    vulnerabilities.append(vuln_detail)
                
                # Extract references
                references = [ref.get("url", "") for ref in advisory.get("references", [])]
                
                result = {
                    "ghsa_id": advisory.get("ghsaId", ""),
                    "summary": advisory.get("summary", ""),
                    "description": advisory.get("description", ""),
                    "severity": advisory.get("severity", ""),
                    "published_at": advisory.get("publishedAt", ""),
                    "updated_at": advisory.get("updatedAt", ""),
                    "withdrawn_at": advisory.get("withdrawnAt"),
                    "permalink": advisory.get("permalink", ""),
                    "vulnerabilities": vulnerabilities,
                    "references": references
                }
                
                self._cache[cve_id] = result
                return result
            
            # No advisory found
            empty_result = {"ghsa_id": "", "summary": "", "vulnerabilities": []}
            self._cache[cve_id] = empty_result
            return empty_result

        except requests.RequestException as e:
            print(f"Warning: GHSA query failed for {cve_id}: {e}", file=sys.stderr)
            return {"ghsa_id": "", "summary": "", "vulnerabilities": [], "error": str(e)}

    def enrich_finding(self, finding: Dict) -> Dict:
        """Add GHSA context to vulnerability finding.
        
        Args:
            finding: Vulnerability finding dictionary
            
        Returns:
            Enhanced finding with GHSA data added
            
        Raises:
            TypeError: If finding is not a dictionary
        """
        if not isinstance(finding, dict):
            raise TypeError(f"Finding must be dict, got {type(finding)}")

        # Extract CVE ID
        cve_id = finding.get("cve") or finding.get("id") or finding.get("vulnerability", {}).get("id")
        
        if not cve_id or not cve_id.startswith("CVE-"):
            finding["ghsa"] = {"ghsa_id": "", "summary": "", "vulnerabilities": []}
            return finding

        try:
            ghsa_data = self.query_advisory(cve_id)
            finding["ghsa"] = ghsa_data
            
            # Add remediation info if available
            if ghsa_data.get("vulnerabilities"):
                first_vuln = ghsa_data["vulnerabilities"][0]
                if first_vuln.get("first_patched_version"):
                    if "remediation" not in finding:
                        finding["remediation"] = {}
                    finding["remediation"]["fixed_version"] = first_vuln["first_patched_version"]
                    finding["remediation"]["vulnerable_range"] = first_vuln.get("vulnerable_version_range", "")
        except Exception as e:
            print(f"Warning: Failed to enrich {cve_id} with GHSA data: {e}", file=sys.stderr)
            finding["ghsa"] = {"ghsa_id": "", "summary": "", "vulnerabilities": []}

        return finding

    def enrich_findings(self, findings: List[Dict]) -> List[Dict]:
        """Add GHSA data to multiple findings.
        
        Args:
            findings: List of vulnerability finding dictionaries
            
        Returns:
            Enhanced findings with GHSA data added
        """
        if not isinstance(findings, list):
            raise TypeError(f"Findings must be list, got {type(findings)}")
        
        for finding in findings:
            if isinstance(finding, dict):
                self.enrich_finding(finding)
        
        return findings


def main():
    """CLI for testing GHSA enrichment."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description="Query GitHub Security Advisories for CVE information"
    )
    parser.add_argument(
        "cve_id",
        help="CVE identifier to check (e.g., CVE-2021-44228)"
    )
    parser.add_argument(
        "--token",
        help="GitHub personal access token (or set GITHUB_TOKEN env var)"
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON"
    )
    
    args = parser.parse_args()
    
    try:
        enricher = GHSAEnricher(github_token=args.token)
        result = enricher.query_advisory(args.cve_id)
        
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            if result.get("ghsa_id"):
                print(f"✅ Found GHSA for {args.cve_id}")
                print(f"   GHSA ID: {result['ghsa_id']}")
                print(f"   Summary: {result['summary']}")
                print(f"   Severity: {result['severity']}")
                print(f"   Published: {result['published_at']}")
                print(f"   Permalink: {result['permalink']}")
                print(f"\n   Affected Packages:")
                for vuln in result.get("vulnerabilities", []):
                    print(f"     - {vuln['ecosystem']}: {vuln['package_name']}")
                    print(f"       Vulnerable: {vuln['vulnerable_version_range']}")
                    if vuln['first_patched_version']:
                        print(f"       Fixed in: {vuln['first_patched_version']}")
            else:
                print(f"❌ No GHSA found for {args.cve_id}")
        
        return 0
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
