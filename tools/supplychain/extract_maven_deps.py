#!/usr/bin/env python3
"""Extract Maven dependencies from WORKSPACE file.

This is a helper script to extract Maven artifacts declared in the WORKSPACE
and create a minimal dependency list for SBOM generation when maven_install.json
is not available.
"""

import argparse
import json
import re
import sys


def extract_maven_artifacts(workspace_content):
    """Extract Maven artifacts from WORKSPACE file content.
    
    Args:
        workspace_content: String content of WORKSPACE file
        
    Returns:
        List of artifact dictionaries
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
                "url": f"https://repo1.maven.org/maven2/{group.replace('.', '/')}/{artifact}/{version}/{artifact}-{version}.jar"
            })
    
    return artifacts


def main():
    parser = argparse.ArgumentParser(
        description="Extract Maven dependencies from WORKSPACE"
    )
    parser.add_argument(
        "--workspace",
        required=True,
        help="Path to WORKSPACE file"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file with dependencies"
    )
    
    args = parser.parse_args()
    
    try:
        with open(args.workspace, "r") as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: WORKSPACE file not found: {args.workspace}", file=sys.stderr)
        return 1
    
    artifacts = extract_maven_artifacts(content)
    
    output_data = {
        "packages": artifacts
    }
    
    with open(args.output, "w") as f:
        json.dump(output_data, f, indent=2)
    
    print(f"Extracted {len(artifacts)} Maven artifacts")
    print(f"Output written to {args.output}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
