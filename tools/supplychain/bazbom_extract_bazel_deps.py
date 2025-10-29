#!/usr/bin/env python3
"""
Extract dependencies from Bazel project and resolve Maven coordinates.

This script:
1. Parses maven_install.json to get Maven coordinates and checksums
2. Uses Bazel query/aspects to extract dependency graph
3. Generates normalized dependency JSON for BazBOM
"""

import argparse
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, List, Optional, Set


def load_maven_install_json(path: Path) -> Dict:
    """Load and parse maven_install.json file.
    
    Args:
        path: Path to maven_install.json
        
    Returns:
        Parsed JSON data with artifacts, dependencies, and packages
    """
    if not path.exists():
        return {
            "artifacts": {},
            "dependencies": {},
            "packages": {},
            "repositories": {},
        }
    
    with open(path, 'r') as f:
        return json.load(f)


def parse_maven_coordinates(coord: str) -> Dict[str, str]:
    """Parse Maven coordinate string into components.
    
    Args:
        coord: Maven coordinate like "group:artifact:version"
        
    Returns:
        Dict with group, artifact, version keys
    """
    parts = coord.split(":")
    if len(parts) < 3:
        return {}
    
    return {
        "group": parts[0],
        "artifact": parts[1],
        "version": parts[2],
        "packaging": parts[3] if len(parts) > 3 else "jar",
        "classifier": parts[4] if len(parts) > 4 else "",
    }


def create_purl(group: str, artifact: str, version: str) -> str:
    """Create Package URL (PURL) for Maven artifact.
    
    Args:
        group: Maven group ID
        artifact: Maven artifact ID
        version: Version string
        
    Returns:
        PURL string
    """
    # PURL format: pkg:maven/group/artifact@version
    # Group ID dots should be replaced with slashes
    namespace = group.replace(".", "/")
    return f"pkg:maven/{namespace}/{artifact}@{version}"


def extract_bazel_targets(workspace_path: Path, target_pattern: str = "//...") -> List[str]:
    """Use Bazel query to find all java_* targets.
    
    Args:
        workspace_path: Path to Bazel workspace
        target_pattern: Bazel target pattern (default: all targets)
        
    Returns:
        List of target labels
    """
    try:
        result = subprocess.run(
            ["bazel", "query", f"kind('java_.*', {target_pattern})", "--output=label"],
            cwd=workspace_path,
            capture_output=True,
            text=True,
            check=True,
        )
        targets = [line.strip() for line in result.stdout.splitlines() if line.strip()]
        return targets
    except subprocess.CalledProcessError as e:
        print(f"Warning: bazel query failed: {e.stderr}", file=sys.stderr)
        return []


def build_dependency_graph(maven_data: Dict, workspace_path: Path) -> Dict:
    """Build dependency graph from maven_install.json.
    
    Args:
        maven_data: Parsed maven_install.json data
        workspace_path: Path to Bazel workspace
        
    Returns:
        Normalized dependency graph with components and edges
    """
    components = []
    edges = []
    seen = set()
    
    artifacts = maven_data.get("artifacts", {})
    dependencies = maven_data.get("dependencies", {})
    repositories = maven_data.get("repositories", {})
    
    # Process each artifact
    # In maven_install.json, artifacts are keyed as "group:artifact"
    for coord, artifact_info in artifacts.items():
        if coord in seen:
            continue
        seen.add(coord)
        
        # Parse coordinate which is in format "group:artifact"
        parts = coord.split(":")
        if len(parts) < 2:
            continue
        
        group = parts[0]
        artifact = parts[1]
        
        # Version is stored separately in artifact_info
        version = artifact_info.get("version", "")
        if not version:
            continue
        
        # Get SHA256 from shasums
        shasums = artifact_info.get("shasums", {})
        sha256 = shasums.get("jar", "")
        
        # Create PURL
        purl = create_purl(group, artifact, version)
        
        # Find repository
        repo_url = ""
        for repo, artifacts_list in repositories.items():
            if coord in artifacts_list:
                repo_url = repo
                break
        
        # Full Maven coordinate for reference
        full_coord = f"{group}:{artifact}:{version}"
        
        component = {
            "name": artifact,
            "group": group,
            "version": version,
            "purl": purl,
            "type": "maven",
            "scope": "compile",  # Default, can be refined
            "sha256": sha256,
            "repository": repo_url,
            "coordinates": full_coord,
        }
        components.append(component)
        
        # Process dependencies
        # Dependencies in maven_install.json are keyed by full coordinate
        # Check if this artifact has dependencies listed
        deps = dependencies.get(full_coord, [])
        if not deps:
            # Also try the short coordinate
            deps = dependencies.get(coord, [])
        
        for dep_coord in deps:
            # dep_coord is in format "group:artifact", need to find the full coordinate
            # Look it up in the artifacts dict
            dep_version = artifacts.get(dep_coord, {}).get("version", "")
            if dep_version:
                full_dep_coord = f"{dep_coord}:{dep_version}"
            else:
                # Fall back to short coordinate
                full_dep_coord = dep_coord
            
            edges.append({
                "from": full_coord,
                "to": full_dep_coord,
                "type": "depends_on",
            })
    
    return {
        "components": components,
        "edges": edges,
        "metadata": {
            "build_system": "bazel",
            "workspace": str(workspace_path),
            "maven_install_version": maven_data.get("version", "unknown"),
        },
    }


def main():
    parser = argparse.ArgumentParser(
        description="Extract Bazel dependencies and generate dependency graph"
    )
    parser.add_argument(
        "--workspace",
        type=Path,
        default=Path.cwd(),
        help="Path to Bazel workspace (default: current directory)",
    )
    parser.add_argument(
        "--maven-install-json",
        type=Path,
        default=Path("maven_install.json"),
        help="Path to maven_install.json (default: ./maven_install.json)",
    )
    parser.add_argument(
        "--output",
        type=Path,
        required=True,
        help="Output JSON file for dependency graph",
    )
    parser.add_argument(
        "--target",
        default="//...",
        help="Bazel target pattern (default: //...)",
    )
    
    args = parser.parse_args()
    
    # Load maven_install.json
    maven_data = load_maven_install_json(args.maven_install_json)
    if not maven_data.get("artifacts"):
        print(
            f"Warning: No artifacts found in {args.maven_install_json}",
            file=sys.stderr,
        )
    
    # Build dependency graph
    graph = build_dependency_graph(maven_data, args.workspace)
    
    # Write output
    args.output.parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, 'w') as f:
        json.dump(graph, f, indent=2)
    
    print(f"Extracted {len(graph['components'])} components", file=sys.stderr)
    print(f"Extracted {len(graph['edges'])} edges", file=sys.stderr)
    print(f"Output written to {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()
