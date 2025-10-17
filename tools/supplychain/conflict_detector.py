#!/usr/bin/env python3
"""Detect and report dependency version conflicts.

This script analyzes dependency graphs to identify version conflicts
and provides recommendations for resolution.
"""

import argparse
import json
import sys
from collections import defaultdict
from typing import Any, Dict, List, Set


def parse_purl(purl: str) -> Dict[str, str]:
    """Parse a Package URL into components.
    
    Args:
        purl: Package URL string
        
    Returns:
        Dictionary with package, version, and other components
    """
    # Simple PURL parser for Maven packages
    if not purl.startswith("pkg:maven/"):
        return {}
    
    purl = purl[len("pkg:maven/"):]
    
    # Split on @ to separate package from version
    if "@" in purl:
        package, version_part = purl.split("@", 1)
        version = version_part.split("?")[0]  # Remove qualifiers
    else:
        package = purl.split("?")[0]
        version = "unknown"
    
    # Split package into namespace and name
    parts = package.split("/")
    if len(parts) == 2:
        namespace, name = parts
    else:
        namespace = ""
        name = package
    
    return {
        "namespace": namespace,
        "name": name,
        "package": package,
        "version": version,
    }


def detect_conflicts(dependencies: List[Dict[str, Any]]) -> Dict[str, List[Dict[str, Any]]]:
    """Detect version conflicts in dependencies.
    
    Args:
        dependencies: List of dependency dictionaries with PURLs
        
    Returns:
        Dictionary mapping package names to conflicting versions
    """
    # Group dependencies by package
    packages: Dict[str, List[Dict[str, Any]]] = defaultdict(list)
    
    for dep in dependencies:
        purl = dep.get("purl", "")
        if not purl:
            # Try to construct from other fields
            if "coordinates" in dep:
                coords = dep["coordinates"].split(":")
                if len(coords) >= 3:
                    package = f"{coords[0]}/{coords[1]}"
                    packages[package].append(dep)
        else:
            parsed = parse_purl(purl)
            if parsed.get("package"):
                packages[parsed["package"]].append(dep)
    
    # Find packages with multiple versions
    conflicts = {}
    for package, deps in packages.items():
        versions = set()
        for dep in deps:
            if "purl" in dep:
                parsed = parse_purl(dep["purl"])
                versions.add(parsed.get("version", "unknown"))
            elif "version" in dep:
                versions.add(dep["version"])
        
        if len(versions) > 1:
            conflicts[package] = deps
    
    return conflicts


def generate_resolution_suggestions(
    conflicts: Dict[str, List[Dict[str, Any]]]
) -> List[Dict[str, Any]]:
    """Generate suggestions for resolving conflicts.
    
    Args:
        conflicts: Dictionary of conflicting packages
        
    Returns:
        List of resolution suggestions
    """
    suggestions = []
    
    for package, deps in conflicts.items():
        versions = []
        for dep in deps:
            if "purl" in dep:
                parsed = parse_purl(dep["purl"])
                version = parsed.get("version", "unknown")
            elif "version" in dep:
                version = dep["version"]
            else:
                version = "unknown"
            
            if version not in versions:
                versions.append(version)
        
        # Sort versions (simple string sort, not semantic versioning)
        versions.sort()
        latest = versions[-1] if versions else "unknown"
        
        suggestion = {
            "package": package,
            "conflicting_versions": versions,
            "recommended_version": latest,
            "reason": "Latest version in conflict set",
            "affected_targets": [
                dep.get("target", dep.get("source", "unknown")) for dep in deps
            ],
        }
        suggestions.append(suggestion)
    
    return suggestions


def generate_report(
    conflicts: Dict[str, List[Dict[str, Any]]],
    output_file: str,
) -> None:
    """Generate a conflict report.
    
    Args:
        conflicts: Dictionary of conflicting packages
        output_file: Path to output JSON file
    """
    suggestions = generate_resolution_suggestions(conflicts)
    
    report = {
        "version": "1.0",
        "conflicts_found": len(conflicts),
        "conflicts": [],
    }
    
    for suggestion in suggestions:
        conflict_detail = {
            "package": suggestion["package"],
            "conflicting_versions": suggestion["conflicting_versions"],
            "recommended_version": suggestion["recommended_version"],
            "resolution_strategy": suggestion["reason"],
            "affected_targets": suggestion["affected_targets"],
        }
        report["conflicts"].append(conflict_detail)
    
    with open(output_file, "w") as f:
        json.dump(report, f, indent=2)
    
    print(f"Found {len(conflicts)} version conflicts", file=sys.stderr)
    print(f"Conflict report written to {output_file}", file=sys.stderr)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Detect and report dependency version conflicts"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with dependencies",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file for conflict report",
    )
    parser.add_argument(
        "--show-all",
        action="store_true",
        help="Show all dependencies, not just conflicts",
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
        
        # Detect conflicts
        conflicts = detect_conflicts(dependencies)
        
        if not conflicts:
            print("No version conflicts detected", file=sys.stderr)
            # Write empty report
            report = {
                "version": "1.0",
                "conflicts_found": 0,
                "conflicts": [],
            }
            with open(args.output, "w") as f:
                json.dump(report, f, indent=2)
        else:
            generate_report(conflicts, args.output)
        
    except Exception as e:
        print(f"Error detecting conflicts: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)


if __name__ == "__main__":
    main()
