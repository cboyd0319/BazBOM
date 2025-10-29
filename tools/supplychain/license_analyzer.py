#!/usr/bin/env python3
"""Analyze licenses for compliance and conflicts.

This script checks for license compatibility issues, copyleft licenses,
and generates compliance reports.
"""

import argparse
import json
import sys
from typing import Any, Dict, List, Set


# License categories
COPYLEFT_LICENSES = {
    "GPL-2.0",
    "GPL-2.0-only",
    "GPL-2.0-or-later",
    "GPL-3.0",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
    "AGPL-3.0",
    "AGPL-3.0-only",
    "AGPL-3.0-or-later",
    "LGPL-2.1",
    "LGPL-2.1-only",
    "LGPL-2.1-or-later",
    "LGPL-3.0",
    "LGPL-3.0-only",
    "LGPL-3.0-or-later",
}

PERMISSIVE_LICENSES = {
    "MIT",
    "Apache-2.0",
    "Apache-1.1",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "0BSD",
}

# Known incompatible license pairs
LICENSE_CONFLICTS = [
    ({"GPL-2.0", "GPL-2.0-only"}, {"Apache-2.0"}),
    ({"GPL-3.0", "GPL-3.0-only"}, {"Apache-1.1"}),
]


def normalize_license(license_str: str) -> str:
    """Normalize license string to SPDX identifier.
    
    Args:
        license_str: License string
        
    Returns:
        Normalized SPDX license identifier
    """
    if not license_str:
        return "NOASSERTION"
    
    # Simple normalization
    license_str = license_str.strip()
    
    # Handle common variations
    if license_str.lower() in ["apache license 2.0", "apache 2", "apache-2"]:
        return "Apache-2.0"
    elif license_str.lower() in ["mit license", "mit"]:
        return "MIT"
    elif license_str.lower() in ["bsd", "bsd license"]:
        return "BSD-3-Clause"
    
    return license_str


def categorize_license(license_id: str) -> str:
    """Categorize a license.
    
    Args:
        license_id: SPDX license identifier
        
    Returns:
        Category: 'copyleft', 'permissive', 'proprietary', 'unknown'
    """
    if license_id in COPYLEFT_LICENSES:
        return "copyleft"
    elif license_id in PERMISSIVE_LICENSES:
        return "permissive"
    elif license_id in ["NOASSERTION", "NONE", ""]:
        return "unknown"
    else:
        return "other"


def detect_license_conflicts(licenses: List[str]) -> List[Dict[str, Any]]:
    """Detect conflicting licenses.
    
    Args:
        licenses: List of license identifiers
        
    Returns:
        List of conflict descriptions
    """
    conflicts = []
    license_set = set(licenses)
    
    for conflict_pair in LICENSE_CONFLICTS:
        group1, group2 = conflict_pair
        if license_set & group1 and license_set & group2:
            conflicts.append({
                "licenses": list(license_set & (group1 | group2)),
                "reason": f"Incompatible licenses: {', '.join(group1)} and {', '.join(group2)}",
                "severity": "high",
            })
    
    return conflicts


def analyze_dependencies(dependencies: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Analyze licenses in dependencies.
    
    Args:
        dependencies: List of dependency dictionaries
        
    Returns:
        Analysis results
    """
    license_counts: Dict[str, int] = {}
    copyleft_deps = []
    unknown_license_deps = []
    all_licenses: Set[str] = set()
    
    for dep in dependencies:
        license_id = dep.get("license", dep.get("licenseDeclared", "NOASSERTION"))
        license_id = normalize_license(license_id)
        
        all_licenses.add(license_id)
        license_counts[license_id] = license_counts.get(license_id, 0) + 1
        
        category = categorize_license(license_id)
        
        if category == "copyleft":
            copyleft_deps.append({
                "name": dep.get("name", "unknown"),
                "version": dep.get("version", "unknown"),
                "license": license_id,
                "purl": dep.get("purl", ""),
            })
        elif category == "unknown":
            unknown_license_deps.append({
                "name": dep.get("name", "unknown"),
                "version": dep.get("version", "unknown"),
                "license": license_id,
                "purl": dep.get("purl", ""),
            })
    
    # Detect conflicts
    conflicts = detect_license_conflicts(list(all_licenses))
    
    # Calculate statistics
    copyleft_count = sum(1 for lic in all_licenses if categorize_license(lic) == "copyleft")
    permissive_count = sum(1 for lic in all_licenses if categorize_license(lic) == "permissive")
    unknown_count = sum(1 for lic in all_licenses if categorize_license(lic) == "unknown")
    
    return {
        "total_dependencies": len(dependencies),
        "unique_licenses": len(all_licenses),
        "license_distribution": license_counts,
        "copyleft_licenses": copyleft_count,
        "permissive_licenses": permissive_count,
        "unknown_licenses": unknown_count,
        "copyleft_dependencies": copyleft_deps,
        "unknown_license_dependencies": unknown_license_deps,
        "conflicts": conflicts,
    }


def generate_report(analysis: Dict[str, Any], output_file: str, check_conflicts: bool, flag_copyleft: bool) -> None:
    """Generate license compliance report.
    
    Args:
        analysis: Analysis results
        output_file: Path to output JSON file
        check_conflicts: Whether to check for conflicts
        flag_copyleft: Whether to flag copyleft licenses
    """
    report = {
        "version": "1.0",
        "summary": {
            "total_dependencies": analysis["total_dependencies"],
            "unique_licenses": analysis["unique_licenses"],
            "copyleft_count": analysis["copyleft_licenses"],
            "permissive_count": analysis["permissive_licenses"],
            "unknown_count": analysis["unknown_licenses"],
        },
        "license_distribution": analysis["license_distribution"],
    }
    
    if flag_copyleft and analysis["copyleft_dependencies"]:
        report["copyleft_dependencies"] = analysis["copyleft_dependencies"]
        print(f"[WARNING]  Found {len(analysis['copyleft_dependencies'])} copyleft dependencies", file=sys.stderr)
    
    if analysis["unknown_license_dependencies"]:
        report["unknown_license_dependencies"] = analysis["unknown_license_dependencies"]
        print(f"[WARNING]  Found {len(analysis['unknown_license_dependencies'])} dependencies with unknown licenses", file=sys.stderr)
    
    if check_conflicts and analysis["conflicts"]:
        report["conflicts"] = analysis["conflicts"]
        print(f" Found {len(analysis['conflicts'])} license conflicts", file=sys.stderr)
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"License analysis report written to {output_file}", file=sys.stderr)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Analyze licenses for compliance and conflicts"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with dependencies",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file for license report",
    )
    parser.add_argument(
        "--check-conflicts",
        action="store_true",
        help="Check for license conflicts",
    )
    parser.add_argument(
        "--flag-copyleft",
        action="store_true",
        help="Flag copyleft licenses",
    )
    
    args = parser.parse_args()
    
    try:
        with open(args.input, "r") as f:
            data = json.load(f)
        
        # Extract dependencies from various formats
        dependencies = []
        if isinstance(data, dict):
            if "dependencies" in data:
                dependencies = data["dependencies"]
            elif "packages" in data:
                dependencies = data["packages"]
        elif isinstance(data, list):
            dependencies = data
        
        # Analyze licenses
        analysis = analyze_dependencies(dependencies)
        
        # Generate report
        generate_report(analysis, args.output, args.check_conflicts, args.flag_copyleft)
        
    except Exception as e:
        print(f"Error analyzing licenses: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
