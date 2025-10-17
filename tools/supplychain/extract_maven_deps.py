#!/usr/bin/env python3
"""Extract Maven dependencies from WORKSPACE file and maven_install.json.

This script extracts Maven artifacts and their transitive dependencies, preferring
the maven_install.json lockfile when available (as per bootstrap recommendations).
Falls back to WORKSPACE parsing for basic extraction.
"""

import argparse
import json
import os
import re
import sys


def extract_from_maven_install_json(lockfile_path):
    """Extract complete dependency graph from maven_install.json.
    
    This is the preferred method as it includes:
    - All transitive dependencies with exact versions
    - SHA256 checksums for verification
    - Complete dependency relationships
    
    Args:
        lockfile_path: Path to maven_install.json
        
    Returns:
        Dictionary with packages and dependency relationships
    """
    try:
        with open(lockfile_path, "r") as f:
            data = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError) as e:
        print(f"Warning: Could not read {lockfile_path}: {e}", file=sys.stderr)
        return None
    
    artifacts = []
    artifacts_dict = data.get("artifacts", {})
    dependencies_dict = data.get("dependencies", {})
    
    # Packages in dependencies_dict are direct dependencies
    direct_dependencies = set(dependencies_dict.keys())
    
    # Process each artifact
    for coord, info in artifacts_dict.items():
        parts = coord.split(":")
        if len(parts) >= 2:
            group = parts[0]
            artifact = parts[1]
            version = info.get("version", "unknown")
            
            # Get SHA256 from shasums
            shasums = info.get("shasums", {})
            sha256 = shasums.get("jar", "")
            
            # Get transitive dependencies
            transitive_deps = dependencies_dict.get(coord, [])
            
            # A package is direct if it appears in the dependencies dict
            # (meaning it has its own list of dependencies, even if empty)
            is_direct = coord in direct_dependencies
            
            artifact_info = {
                "name": artifact,
                "group": group,
                "version": version,
                "purl": f"pkg:maven/{group}/{artifact}@{version}",
                "type": "maven",
                "license": "NOASSERTION",  # License would be fetched from POM
                "url": f"https://repo1.maven.org/maven2/{group.replace('.', '/')}/{artifact}/{version}/{artifact}-{version}.jar",
                "sha256": sha256,
                "dependencies": transitive_deps,
                "is_direct": is_direct
            }
            
            artifacts.append(artifact_info)
    
    return {
        "packages": artifacts,
        "source": "maven_install.json",
        "transitive_included": True
    }


def extract_maven_artifacts(workspace_content):
    """Extract Maven artifacts from WORKSPACE file content.
    
    Fallback method when maven_install.json is not available.
    
    Args:
        workspace_content: String content of WORKSPACE file
        
    Returns:
        Dictionary with packages (without transitive dependencies)
    """
    artifacts = []
    
    # Match maven_install artifacts list
    # Pattern: "group:artifact:version"
    pattern = r'"([^"]+:[^"]+:[^"]+)"'
    
    matches = re.findall(pattern, workspace_content)
    
    for match in matches:
        parts = match.split(":")
        if len(parts) >= 3:
            group = parts[0]
            artifact = parts[1]
            version = parts[2]
            
            artifacts.append({
                "name": artifact,
                "group": group,
                "version": version,
                "purl": f"pkg:maven/{group}/{artifact}@{version}",
                "type": "maven",
                "license": "NOASSERTION",  # Would be fetched from POM in real implementation
                "url": f"https://repo1.maven.org/maven2/{group.replace('.', '/')}/{artifact}/{version}/{artifact}-{version}.jar",
                "sha256": "",
                "dependencies": [],
                "is_direct": True
            })
    
    return {
        "packages": artifacts,
        "source": "WORKSPACE",
        "transitive_included": False
    }


def main():
    parser = argparse.ArgumentParser(
        description="Extract Maven dependencies from maven_install.json or WORKSPACE"
    )
    parser.add_argument(
        "--workspace",
        required=True,
        help="Path to WORKSPACE file"
    )
    parser.add_argument(
        "--maven-install-json",
        default="maven_install.json",
        help="Path to maven_install.json lockfile (default: maven_install.json)"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file with dependencies"
    )
    parser.add_argument(
        "--prefer-lockfile",
        action="store_true",
        default=True,
        help="Prefer maven_install.json over WORKSPACE (default: true)"
    )
    
    args = parser.parse_args()
    
    output_data = None
    
    # Try maven_install.json first (recommended by bootstrap document)
    if args.prefer_lockfile and os.path.exists(args.maven_install_json):
        print(f"Using maven_install.json as source of truth: {args.maven_install_json}")
        output_data = extract_from_maven_install_json(args.maven_install_json)
    
    # Fall back to WORKSPACE if lockfile not available or failed
    if output_data is None:
        print(f"Falling back to WORKSPACE parsing: {args.workspace}")
        try:
            with open(args.workspace, "r") as f:
                content = f.read()
        except FileNotFoundError:
            print(f"Error: WORKSPACE file not found: {args.workspace}", file=sys.stderr)
            return 1
        
        output_data = extract_maven_artifacts(content)
    
    # Write output
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    num_packages = len(output_data.get("packages", []))
    source = output_data.get("source", "unknown")
    has_transitive = output_data.get("transitive_included", False)
    
    print(f"Extracted {num_packages} Maven artifacts from {source}")
    if has_transitive:
        print(f"✓ Included transitive dependencies and checksums")
    else:
        print(f"⚠ Warning: Transitive dependencies not included (use maven_install.json for complete graph)")
    print(f"Output written to {args.output}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
