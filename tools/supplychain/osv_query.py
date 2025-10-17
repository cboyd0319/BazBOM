#!/usr/bin/env python3
"""Query OSV database for vulnerabilities in packages.

This script queries the Open Source Vulnerabilities (OSV) database
for known security issues in packages listed in an SBOM.
"""

import argparse
import json
import sys
import time
from typing import Any, Dict, List, Optional

try:
    import requests
except ImportError:
    print("Error: requests library not installed", file=sys.stderr)
    print("Install with: pip install requests", file=sys.stderr)
    sys.exit(1)


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


def main():
    parser = argparse.ArgumentParser(
        description="Query OSV database for vulnerabilities"
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
                        "vulnerability": vuln
                    })
            # Rate limiting
            time.sleep(0.1)
    
    print(f"Found {len(vulnerabilities)} vulnerabilities")
    
    # Write results
    output_data = {
        "packages_scanned": len(packages),
        "vulnerabilities_found": len(vulnerabilities),
        "vulnerabilities": vulnerabilities
    }
    
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    print(f"Results written to {args.output}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
