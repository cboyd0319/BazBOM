#!/usr/bin/env python3
"""Query OSV database for vulnerabilities in packages.

This script queries the Open Source Vulnerabilities (OSV) database
for known security issues in packages listed in an SBOM. It also enriches
vulnerability findings with KEV, EPSS, GHSA, and VulnCheck data.
"""

import argparse
import json
import os
import sys
import time
from datetime import datetime
from typing import Any, Dict, List, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)

# Import enrichment modules
try:
    from vulnerability_enrichment import VulnerabilityEnricher
    ENRICHMENT_AVAILABLE = True
except ImportError:
    ENRICHMENT_AVAILABLE = False
    print("Warning: Enrichment modules not available", file=sys.stderr)


OSV_API_URL = "https://api.osv.dev/v1/query"
OSV_BATCH_URL = "https://api.osv.dev/v1/querybatch"


def extract_packages_from_sbom(sbom_path: str) -> List[Dict[str, str]]:
    """Extract package information from SPDX SBOM.
    
    Args:
        sbom_path: Path to SPDX JSON file
        
    Returns:
        List of package dictionaries with name, version, ecosystem
    """
    with open(sbom_path, "r") as f:
        sbom = json.load(f)
    
    packages = []
    for pkg in sbom.get("packages", []):
        # Skip root package
        if pkg.get("SPDXID") == "SPDXRef-Package-root":
            continue
        
        # Extract ecosystem from purl if available
        ecosystem = "Maven"  # Default to Maven
        purl = None
        
        for ref in pkg.get("externalRefs", []):
            if ref.get("referenceType") == "purl":
                purl = ref.get("referenceLocator", "")
                if purl.startswith("pkg:maven/"):
                    ecosystem = "Maven"
                elif purl.startswith("pkg:npm/"):
                    ecosystem = "npm"
                elif purl.startswith("pkg:pypi/"):
                    ecosystem = "PyPI"
                break
        
        packages.append({
            "name": pkg.get("name", ""),
            "version": pkg.get("versionInfo", ""),
            "ecosystem": ecosystem,
            "purl": purl,
        })
    
    return packages


def query_osv(package: Dict[str, str]) -> Optional[Dict[str, Any]]:
    """Query OSV for a single package.
    
    Args:
        package: Package dict with name, version, ecosystem
        
    Returns:
        OSV response or None on error
    """
    query = {
        "package": {
            "name": package["name"],
            "ecosystem": package["ecosystem"]
        },
        "version": package["version"]
    }
    
    try:
        response = requests.post(
            OSV_API_URL,
            json=query,
            timeout=30,
        )
        response.raise_for_status()
        return response.json()
    except requests.RequestException as e:
        print(f"Warning: OSV query failed for {package['name']}: {e}", file=sys.stderr)
        return None


def query_osv_batch(packages: List[Dict[str, str]]) -> List[Optional[Dict[str, Any]]]:
    """Query OSV for multiple packages in batch.
    
    Args:
        packages: List of package dicts
        
    Returns:
        List of OSV responses
    """
    queries = []
    for pkg in packages:
        queries.append({
            "package": {
                "name": pkg["name"],
                "ecosystem": pkg["ecosystem"]
            },
            "version": pkg["version"]
        })
    
    try:
        response = requests.post(
            OSV_BATCH_URL,
            json={"queries": queries},
            timeout=60,
        )
        response.raise_for_status()
        data = response.json()
        return data.get("results", [])
    except requests.RequestException as e:
        print(f"Warning: OSV batch query failed: {e}", file=sys.stderr)
        return [None] * len(packages)


def extract_cvss_score(vuln: Dict) -> float:
    """Extract CVSS base score from vulnerability data.
    
    Args:
        vuln: Vulnerability dictionary from OSV
        
    Returns:
        CVSS base score (0.0-10.0)
    """
    for severity in vuln.get("severity", []):
        if severity.get("type") == "CVSS_V3":
            score_str = severity.get("score", "")
            # Parse CVSS vector string (e.g., "CVSS:3.1/AV:N/AC:L/...")
            if "/" in score_str:
                try:
                    # Extract base score from vector
                    for part in score_str.split("/"):
                        if part.startswith("CVSS:"):
                            # Base score is after the version
                            continue
                        # Look for numeric score in database_specific
                        break
                except (ValueError, IndexError):
                    pass
        elif severity.get("type") == "CVSS_V2":
            score_str = severity.get("score", "")
    
    # Try database_specific field
    db_specific = vuln.get("database_specific", {})
    cvss_score = db_specific.get("cvss_score")
    if cvss_score:
        try:
            return float(cvss_score)
        except (ValueError, TypeError):
            pass
    
    # Try severity level mapping as fallback
    severity = vuln.get("severity", [{}])[0].get("level", "MEDIUM").upper()
    severity_map = {
        "CRITICAL": 9.5,
        "HIGH": 7.5,
        "MEDIUM": 5.0,
        "LOW": 3.0
    }
    return severity_map.get(severity, 5.0)


def normalize_findings(vulnerabilities: List[Dict]) -> List[Dict]:
    """Normalize vulnerability findings for enrichment.
    
    Args:
        vulnerabilities: Raw vulnerability findings from OSV
        
    Returns:
        Normalized findings with standardized fields
    """
    normalized = []
    for vuln_data in vulnerabilities:
        vuln = vuln_data.get("vulnerability", {})
        vuln_id = vuln.get("id", "UNKNOWN")
        
        # Extract CVE ID if available (prefer CVE over other IDs)
        cve = None
        for alias in vuln.get("aliases", []):
            if alias.startswith("CVE-"):
                cve = alias
                break
        
        if not cve and vuln_id.startswith("CVE-"):
            cve = vuln_id
        
        # Build normalized finding
        finding = {
            "id": vuln_id,
            "cve": cve,
            "package": vuln_data.get("package", ""),
            "version": vuln_data.get("version", ""),
            "purl": vuln_data.get("purl", ""),
            "summary": vuln.get("summary", ""),
            "details": vuln.get("details", ""),
            "severity": "MEDIUM",  # Default
            "cvss_score": extract_cvss_score(vuln),
            "published": vuln.get("published", ""),
            "modified": vuln.get("modified", ""),
            "references": [ref.get("url", "") for ref in vuln.get("references", [])],
            "affected": vuln.get("affected", []),
            "vulnerability": vuln  # Keep original for compatibility
        }
        
        normalized.append(finding)
    
    return normalized


def main():
    parser = argparse.ArgumentParser(
        description="Query OSV database for vulnerabilities with enrichment"
    )
    parser.add_argument(
        "--sbom",
        required=True,
        help="Path to SPDX SBOM file"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output file for vulnerability data (JSON)"
    )
    parser.add_argument(
        "--batch",
        action="store_true",
        help="Use batch API for better performance"
    )
    parser.add_argument(
        "--enrich",
        action="store_true",
        default=True,
        help="Enable vulnerability enrichment (default: True)"
    )
    parser.add_argument(
        "--no-enrich",
        dest="enrich",
        action="store_false",
        help="Disable vulnerability enrichment"
    )
    parser.add_argument(
        "--github-token",
        help="GitHub token for GHSA queries (or set GITHUB_TOKEN env var)"
    )
    parser.add_argument(
        "--vulncheck-api-key",
        help="VulnCheck API key (or set VULNCHECK_API_KEY env var)"
    )
    parser.add_argument(
        "--disable-vulncheck",
        action="store_true",
        help="Disable VulnCheck enrichment"
    )
    parser.add_argument(
        "--disable-ghsa",
        action="store_true",
        help="Disable GHSA enrichment"
    )
    
    args = parser.parse_args()
    
    # Extract packages from SBOM
    print(f"Reading SBOM from {args.sbom}")
    packages = extract_packages_from_sbom(args.sbom)
    print(f"Found {len(packages)} packages")
    
    # Query OSV
    vulnerabilities = []
    
    if args.batch:
        print("Querying OSV (batch mode)...")
        results = query_osv_batch(packages)
        for pkg, result in zip(packages, results):
            if result and result.get("vulns"):
                for vuln in result["vulns"]:
                    vulnerabilities.append({
                        "package": pkg["name"],
                        "version": pkg["version"],
                        "purl": pkg.get("purl", ""),
                        "vulnerability": vuln
                    })
    else:
        print("Querying OSV (individual mode)...")
        for i, pkg in enumerate(packages):
            print(f"  {i+1}/{len(packages)}: {pkg['name']}@{pkg['version']}")
            result = query_osv(pkg)
            if result and result.get("vulns"):
                for vuln in result["vulns"]:
                    vulnerabilities.append({
                        "package": pkg["name"],
                        "version": pkg["version"],
                        "purl": pkg.get("purl", ""),
                        "vulnerability": vuln
                    })
            # Rate limiting
            time.sleep(0.1)
    
    print(f"Found {len(vulnerabilities)} vulnerabilities")
    
    # Normalize findings
    findings = normalize_findings(vulnerabilities)
    
    # Enrich findings if enabled
    if args.enrich and ENRICHMENT_AVAILABLE and findings:
        print("\nEnriching vulnerability findings...")
        try:
            enricher = VulnerabilityEnricher(
                github_token=args.github_token,
                vulncheck_api_key=args.vulncheck_api_key,
                enable_vulncheck=not args.disable_vulncheck,
                enable_ghsa=not args.disable_ghsa
            )
            findings = enricher.enrich_all(findings)
            
            # Print summary
            summary = enricher.get_priority_summary(findings)
            print(f"\nPriority Summary:")
            print(f"  P0 - IMMEDIATE (KEV):     {summary['P0-IMMEDIATE']}")
            print(f"  P1 - CRITICAL:            {summary['P1-CRITICAL']}")
            print(f"  P2 - HIGH:                {summary['P2-HIGH']}")
            print(f"  P3 - MEDIUM:              {summary['P3-MEDIUM']}")
            print(f"  P4 - LOW:                 {summary['P4-LOW']}")
        except Exception as e:
            print(f"Warning: Enrichment failed: {e}", file=sys.stderr)
            print("Continuing with unenriched findings...", file=sys.stderr)
    elif not ENRICHMENT_AVAILABLE:
        print("Warning: Enrichment disabled (modules not available)", file=sys.stderr)
    
    # Write results
    output_data = {
        "scan_date": datetime.now().isoformat(),
        "packages_scanned": len(packages),
        "vulnerabilities_found": len(findings),
        "enrichment_enabled": args.enrich and ENRICHMENT_AVAILABLE,
        "vulnerabilities": findings
    }
    
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    print(f"\nResults written to {args.output}")
    
    # Print top P0/P1 findings if enriched
    if args.enrich and ENRICHMENT_AVAILABLE and findings:
        p0_findings = [f for f in findings if f.get("priority") == "P0-IMMEDIATE"]
        if p0_findings:
            print(f"\n[WARNING]  {len(p0_findings)} P0-IMMEDIATE findings require immediate action!")
            for finding in p0_findings[:3]:  # Show top 3
                cve = finding.get("cve", finding.get("id", "UNKNOWN"))
                pkg = finding.get("package", "unknown")
                print(f"  - {cve} in {pkg} (in CISA KEV)")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
