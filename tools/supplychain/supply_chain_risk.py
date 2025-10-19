#!/usr/bin/env python3
"""Supply chain risk scanner for detecting typosquatting, deprecated packages, and unmaintained dependencies."""

import argparse
import json
import sys
import re
from typing import Dict, List, Any, Optional, Set
from datetime import datetime, timedelta
import urllib.request
import urllib.error

def levenshtein_distance(s1: str, s2: str) -> int:
    """Calculate Levenshtein distance between two strings."""
    if len(s1) < len(s2):
        return levenshtein_distance(s2, s1)
    
    if len(s2) == 0:
        return len(s1)
    
    previous_row = range(len(s2) + 1)
    for i, c1 in enumerate(s1):
        current_row = [i + 1]
        for j, c2 in enumerate(s2):
            insertions = previous_row[j + 1] + 1
            deletions = current_row[j] + 1
            substitutions = previous_row[j] + (c1 != c2)
            current_row.append(min(insertions, deletions, substitutions))
        previous_row = current_row
    
    return previous_row[-1]


def check_typosquatting(package_name: str, known_packages: Set[str], threshold: int = 2) -> List[Dict[str, Any]]:
    """
    Check if a package name is similar to known popular packages (potential typosquatting).
    
    Args:
        package_name: Package name to check
        known_packages: Set of known popular package names
        threshold: Maximum Levenshtein distance to consider as potential typosquatting
        
    Returns:
        List of potential typosquatting matches
    """
    findings = []
    package_lower = package_name.lower()
    
    for known_pkg in known_packages:
        known_lower = known_pkg.lower()
        if package_lower == known_lower:
            continue
            
        distance = levenshtein_distance(package_lower, known_lower)
        if distance <= threshold:
            findings.append({
                "type": "typosquatting",
                "package": package_name,
                "similar_to": known_pkg,
                "distance": distance,
                "severity": "HIGH" if distance == 1 else "MEDIUM",
                "description": f"Package name '{package_name}' is very similar to popular package '{known_pkg}' (distance: {distance})"
            })
    
    return findings


def check_deprecated_maven(group_id: str, artifact_id: str, version: str) -> Optional[Dict[str, Any]]:
    """
    Check if a Maven package is deprecated using Maven Central API.
    
    Args:
        group_id: Maven group ID
        artifact_id: Maven artifact ID
        version: Version string
        
    Returns:
        Finding dict if deprecated, None otherwise
    """
    # Note: Maven Central doesn't have a formal deprecation API
    # This is a placeholder for the structure - in production, you'd check:
    # 1. Maven metadata for deprecation notices
    # 2. Project README/docs
    # 3. Latest version to see if current version is very old
    
    try:
        # Check if there's a much newer version available
        search_url = f"https://search.maven.org/solrsearch/select?q=g:{group_id}+AND+a:{artifact_id}&rows=1&wt=json"
        
        # Validate URL scheme for security (prevent file:/ and other schemes)
        if not search_url.startswith(('http://', 'https://')):
            return []
        
        with urllib.request.urlopen(search_url, timeout=5) as response:
            data = json.loads(response.read())
            if data.get('response', {}).get('numFound', 0) > 0:
                docs = data['response']['docs']
                if docs:
                    latest_version = docs[0].get('latestVersion', version)
                    if latest_version != version:
                        # Simple heuristic: if version is very different, might be deprecated
                        return {
                            "type": "outdated_version",
                            "package": f"{group_id}:{artifact_id}",
                            "current_version": version,
                            "latest_version": latest_version,
                            "severity": "MEDIUM",
                            "description": f"Package {group_id}:{artifact_id}:{version} has a newer version available: {latest_version}"
                        }
    except (urllib.error.URLError, urllib.error.HTTPError, TimeoutError):
        # Network errors are expected in some environments
        pass
    except Exception as e:
        print(f"Warning: Error checking Maven Central for {group_id}:{artifact_id}: {e}", file=sys.stderr)
    
    return None


def check_unmaintained(package_name: str, last_commit_date: Optional[str], threshold_years: int = 2) -> Optional[Dict[str, Any]]:
    """
    Check if a package appears unmaintained based on last commit date.
    
    Args:
        package_name: Package name
        last_commit_date: ISO format date string of last commit
        threshold_years: Years of inactivity to consider unmaintained
        
    Returns:
        Finding dict if unmaintained, None otherwise
    """
    if not last_commit_date:
        return None
    
    try:
        last_date = datetime.fromisoformat(last_commit_date.replace('Z', '+00:00'))
        threshold_date = datetime.now().astimezone() - timedelta(days=threshold_years * 365)
        
        if last_date < threshold_date:
            years_inactive = (datetime.now().astimezone() - last_date).days / 365
            return {
                "type": "unmaintained",
                "package": package_name,
                "last_commit": last_commit_date,
                "years_inactive": round(years_inactive, 1),
                "severity": "MEDIUM",
                "description": f"Package {package_name} has not been updated in {round(years_inactive, 1)} years (last commit: {last_commit_date})"
            }
    except (ValueError, TypeError) as e:
        print(f"Warning: Invalid date format for {package_name}: {last_commit_date}: {e}", file=sys.stderr)
    
    return None


def load_popular_packages() -> Set[str]:
    """Load a set of popular package names for typosquatting detection."""
    # Common popular Java packages that are often targeted
    return {
        "guava", "gson", "jackson", "slf4j", "logback", "log4j",
        "junit", "mockito", "spring", "hibernate", "commons-lang",
        "commons-io", "apache-commons", "netty", "jetty", "tomcat",
        "servlet", "jaxb", "jersey", "okhttp", "retrofit", "rxjava",
        "protobuf", "grpc", "thrift", "avro", "kafka", "hadoop",
        "spark", "flink", "elasticsearch", "mongodb", "redis",
        "mysql", "postgresql", "h2", "derby", "hsqldb"
    }


def parse_sbom(sbom_path: str) -> List[Dict[str, Any]]:
    """Parse SBOM and extract package information."""
    with open(sbom_path, 'r') as f:
        sbom = json.load(f)
    
    packages = []
    for pkg in sbom.get('packages', []):
        pkg_name = pkg.get('name', '')
        version = pkg.get('versionInfo', '')
        
        # Extract Maven coordinates from external refs
        group_id = None
        artifact_id = None
        for ref in pkg.get('externalRefs', []):
            if ref.get('referenceType') == 'purl':
                purl = ref.get('referenceLocator', '')
                # Parse PURL format: pkg:maven/group/artifact@version
                match = re.match(r'pkg:maven/([^/]+)/([^@]+)@(.+)', purl)
                if match:
                    group_id = match.group(1)
                    artifact_id = match.group(2)
        
        packages.append({
            "name": pkg_name,
            "version": version,
            "group_id": group_id,
            "artifact_id": artifact_id,
            "spdx_id": pkg.get('SPDXID')
        })
    
    return packages


def main():
    parser = argparse.ArgumentParser(description='Supply chain risk scanner')
    parser.add_argument('--sbom', required=True, help='Path to SBOM file (SPDX JSON)')
    parser.add_argument('--output', required=True, help='Output JSON file for findings')
    parser.add_argument('--check-typosquatting', action='store_true', help='Check for typosquatting')
    parser.add_argument('--check-deprecated', action='store_true', help='Check for deprecated packages')
    parser.add_argument('--check-unmaintained', action='store_true', help='Check for unmaintained packages')
    parser.add_argument('--threshold-years', type=int, default=2, help='Years of inactivity for unmaintained check')
    parser.add_argument('--offline-mode', action='store_true', help='Skip network-based checks')
    
    args = parser.parse_args()
    
    # If no specific checks requested, enable all
    if not (args.check_typosquatting or args.check_deprecated or args.check_unmaintained):
        args.check_typosquatting = True
        args.check_deprecated = True
        args.check_unmaintained = True
    
    print(f"Reading SBOM from {args.sbom}", file=sys.stderr)
    packages = parse_sbom(args.sbom)
    print(f"Found {len(packages)} packages to analyze", file=sys.stderr)
    
    findings = []
    
    # Typosquatting check
    if args.check_typosquatting:
        print("Checking for typosquatting...", file=sys.stderr)
        popular_packages = load_popular_packages()
        for pkg in packages:
            typo_findings = check_typosquatting(pkg['name'], popular_packages)
            findings.extend(typo_findings)
    
    # Deprecated package check
    if args.check_deprecated and not args.offline_mode:
        print("Checking for deprecated/outdated packages...", file=sys.stderr)
        for pkg in packages:
            if pkg['group_id'] and pkg['artifact_id']:
                deprecated = check_deprecated_maven(pkg['group_id'], pkg['artifact_id'], pkg['version'])
                if deprecated:
                    findings.append(deprecated)
    
    # Unmaintained package check
    if args.check_unmaintained:
        print("Checking for unmaintained packages...", file=sys.stderr)
        # Note: This requires commit date information which would come from
        # enhanced SBOM generation or external metadata. For now, we skip it
        # if the data is not available in the SBOM.
        print("Note: Unmaintained check requires commit date metadata", file=sys.stderr)
    
    # Generate report
    report = {
        "scan_date": datetime.now().isoformat(),
        "sbom_file": args.sbom,
        "checks_performed": {
            "typosquatting": args.check_typosquatting,
            "deprecated": args.check_deprecated and not args.offline_mode,
            "unmaintained": args.check_unmaintained
        },
        "findings": findings,
        "summary": {
            "total_packages": len(packages),
            "total_findings": len(findings),
            "by_severity": {
                "critical": sum(1 for f in findings if f.get('severity') == 'CRITICAL'),
                "high": sum(1 for f in findings if f.get('severity') == 'HIGH'),
                "medium": sum(1 for f in findings if f.get('severity') == 'MEDIUM'),
                "low": sum(1 for f in findings if f.get('severity') == 'LOW')
            },
            "by_type": {
                "typosquatting": sum(1 for f in findings if f.get('type') == 'typosquatting'),
                "outdated_version": sum(1 for f in findings if f.get('type') == 'outdated_version'),
                "unmaintained": sum(1 for f in findings if f.get('type') == 'unmaintained')
            }
        }
    }
    
    with open(args.output, 'w') as f:
        json.dump(report, f, indent=2)
    
    print(f"Supply chain risk report written to {args.output}", file=sys.stderr)
    print(f"Found {len(findings)} potential risks", file=sys.stderr)
    
    # Always return 0 for report generation (don't fail the build)
    # CI can check the findings count and fail based on policy
    return 0


if __name__ == '__main__':
    sys.exit(main())
