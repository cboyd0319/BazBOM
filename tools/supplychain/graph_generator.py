#!/usr/bin/env python3
"""Generate dependency graphs from SBOM data.

This script converts SBOM data into dependency graph formats including
JSON and GraphML for visualization tools like Gephi or yEd.
"""

import argparse
import json
import sys
from typing import Any, Dict, List, Set


def calculate_depths(packages: List[Dict[str, Any]]) -> Dict[str, int]:
    """Calculate depth of each package in the dependency tree.
    
    Uses BFS to determine the shortest distance from root to each package.
    
    Args:
        packages: List of package dictionaries
        
    Returns:
        Dictionary mapping package coordinates to depth level
    """
    depths = {}
    queue = []
    
    # Build adjacency map
    dependency_map = {}
    
    for pkg in packages:
        coord = f"{pkg.get('group', '')}:{pkg.get('name', '')}"
        dependency_map[coord] = pkg.get("dependencies", [])
        
        # Direct dependencies (from WORKSPACE/maven_install) start at depth 1
        if pkg.get("is_direct", False):
            depths[coord] = 1
            queue.append((coord, 1))
    
    # BFS to calculate remaining depths
    visited = set()
    while queue:
        current_coord, current_depth = queue.pop(0)
        
        if current_coord in visited:
            continue
        visited.add(current_coord)
        
        # Process dependencies
        for dep_coord in dependency_map.get(current_coord, []):
            # Update depth if not set or found shorter path
            new_depth = current_depth + 1
            if dep_coord not in depths or depths[dep_coord] > new_depth:
                depths[dep_coord] = new_depth
                queue.append((dep_coord, new_depth))
    
    return depths


def generate_graph_json(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate a JSON dependency graph with depth tracking.
    
    Args:
        packages: List of package dictionaries
        
    Returns:
        Graph in JSON format with nodes (including depth) and edges
    """
    nodes = []
    edges = []
    seen_packages = set()
    
    # Calculate depths for all packages
    depths = calculate_depths(packages)
    
    # Build coordinate to package map
    coord_to_pkg = {}
    for pkg in packages:
        coord = f"{pkg.get('group', '')}:{pkg.get('name', '')}"
        coord_to_pkg[coord] = pkg
    
    # Create root node
    root_node = {
        "id": "root",
        "name": "root",
        "type": "root",
        "depth": 0
    }
    nodes.append(root_node)
    
    # Create nodes for each package
    for pkg in packages:
        pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
        coord = f"{pkg.get('group', '')}:{pkg.get('name', '')}"
        
        if pkg_id not in seen_packages:
            node = {
                "id": pkg_id,
                "name": pkg.get("name", "unknown"),
                "version": pkg.get("version", "unknown"),
                "type": pkg.get("type", "unknown"),
                "depth": depths.get(coord, 1),
                "is_direct": pkg.get("is_direct", False),
            }
            
            if "group" in pkg:
                node["group"] = pkg["group"]
            
            if "sha256" in pkg and pkg["sha256"]:
                node["sha256"] = pkg["sha256"]
            
            nodes.append(node)
            seen_packages.add(pkg_id)
    
    # Create edges from root to direct dependencies
    for pkg in packages:
        if pkg.get("is_direct", False):
            pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
            edges.append({
                "from": "root",
                "to": pkg_id,
                "type": "depends_on"
            })
    
    # Create edges for transitive dependencies
    for pkg in packages:
        pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
        dependencies = pkg.get("dependencies", [])
        
        for dep_coord in dependencies:
            # Find the dependency package
            if dep_coord in coord_to_pkg:
                dep_pkg = coord_to_pkg[dep_coord]
                dep_id = dep_pkg.get("purl", dep_pkg.get("name", "unknown"))
                
                edges.append({
                    "from": pkg_id,
                    "to": dep_id,
                    "type": "depends_on"
                })
    
    return {
        "version": "1.0",
        "graph": {
            "nodes": nodes,
            "edges": edges
        },
        "statistics": {
            "total_packages": len(nodes) - 1,  # Exclude root
            "max_depth": max(depths.values()) if depths else 0,
            "direct_dependencies": sum(1 for p in packages if p.get("is_direct", False)),
            "transitive_dependencies": len(packages) - sum(1 for p in packages if p.get("is_direct", False))
        }
    }


def generate_graphml(packages: List[Dict[str, Any]]) -> str:
    """Generate a GraphML dependency graph with depth information.
    
    Args:
        packages: List of package dictionaries
        
    Returns:
        GraphML XML string
    """
    graphml = ['<?xml version="1.0" encoding="UTF-8"?>']
    graphml.append('<graphml xmlns="http://graphml.graphdrawing.org/xmlns"')
    graphml.append('         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"')
    graphml.append('         xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns')
    graphml.append('         http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">')
    
    # Define attribute keys
    graphml.append('  <key id="name" for="node" attr.name="name" attr.type="string"/>')
    graphml.append('  <key id="version" for="node" attr.name="version" attr.type="string"/>')
    graphml.append('  <key id="type" for="node" attr.name="type" attr.type="string"/>')
    graphml.append('  <key id="purl" for="node" attr.name="purl" attr.type="string"/>')
    graphml.append('  <key id="depth" for="node" attr.name="depth" attr.type="int"/>')
    graphml.append('  <key id="is_direct" for="node" attr.name="is_direct" attr.type="boolean"/>')
    graphml.append('  <key id="edgetype" for="edge" attr.name="type" attr.type="string"/>')
    
    graphml.append('  <graph id="G" edgedefault="directed">')
    
    # Calculate depths
    depths = calculate_depths(packages)
    coord_to_pkg = {}
    for pkg in packages:
        coord = f"{pkg.get('group', '')}:{pkg.get('name', '')}"
        coord_to_pkg[coord] = pkg
    
    # Add root node
    graphml.append('    <node id="root">')
    graphml.append('      <data key="name">root</data>')
    graphml.append('      <data key="type">root</data>')
    graphml.append('      <data key="depth">0</data>')
    graphml.append('    </node>')
    
    # Add package nodes
    seen_packages = set()
    for pkg in packages:
        pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
        coord = f"{pkg.get('group', '')}:{pkg.get('name', '')}"
        
        if pkg_id not in seen_packages:
            # Escape XML special characters
            pkg_id_safe = pkg_id.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            name = pkg.get("name", "unknown").replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            version = pkg.get("version", "unknown").replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            pkg_type = pkg.get("type", "unknown")
            depth = depths.get(coord, 1)
            is_direct = str(pkg.get("is_direct", False)).lower()
            
            graphml.append(f'    <node id="{pkg_id_safe}">')
            graphml.append(f'      <data key="name">{name}</data>')
            graphml.append(f'      <data key="version">{version}</data>')
            graphml.append(f'      <data key="type">{pkg_type}</data>')
            graphml.append(f'      <data key="purl">{pkg_id_safe}</data>')
            graphml.append(f'      <data key="depth">{depth}</data>')
            graphml.append(f'      <data key="is_direct">{is_direct}</data>')
            graphml.append('    </node>')
            
            seen_packages.add(pkg_id)
    
    # Add edges from root to direct dependencies
    for pkg in packages:
        if pkg.get("is_direct", False):
            pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
            pkg_id_safe = pkg_id.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            graphml.append(f'    <edge source="root" target="{pkg_id_safe}">')
            graphml.append('      <data key="edgetype">depends_on</data>')
            graphml.append('    </edge>')
    
    # Add edges for transitive dependencies
    for pkg in packages:
        pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
        pkg_id_safe = pkg_id.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
        dependencies = pkg.get("dependencies", [])
        
        for dep_coord in dependencies:
            if dep_coord in coord_to_pkg:
                dep_pkg = coord_to_pkg[dep_coord]
                dep_id = dep_pkg.get("purl", dep_pkg.get("name", "unknown"))
                dep_id_safe = dep_id.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
                
                graphml.append(f'    <edge source="{pkg_id_safe}" target="{dep_id_safe}">')
                graphml.append('      <data key="edgetype">depends_on</data>')
                graphml.append('    </edge>')
    
    graphml.append('  </graph>')
    graphml.append('</graphml>')
    
    return '\n'.join(graphml)


def main():
    parser = argparse.ArgumentParser(
        description="Generate dependency graphs from SBOM or dependency data"
    )
    parser.add_argument(
        "--sbom",
        help="Path to SPDX SBOM file"
    )
    parser.add_argument(
        "--deps",
        help="Path to workspace_deps.json file (richer data with transitive deps)"
    )
    parser.add_argument(
        "--output-json",
        help="Output path for JSON graph"
    )
    parser.add_argument(
        "--output-graphml",
        help="Output path for GraphML graph"
    )
    
    args = parser.parse_args()
    
    if not args.sbom and not args.deps:
        print("Error: Either --sbom or --deps must be specified", file=sys.stderr)
        return 1
    
    if not args.output_json and not args.output_graphml:
        print("Error: At least one output format must be specified", file=sys.stderr)
        return 1
    
    packages = []
    
    # Read from workspace_deps.json (preferred - has full dependency info)
    if args.deps:
        try:
            with open(args.deps, "r") as f:
                deps_data = json.load(f)
            packages = deps_data.get("packages", [])
            print(f"Processed {len(packages)} packages")
        except FileNotFoundError:
            print(f"Error: Deps file not found: {args.deps}", file=sys.stderr)
            return 1
        except json.JSONDecodeError as e:
            print(f"Error: Invalid JSON in deps file: {e}", file=sys.stderr)
            return 1
    
    # Read from SBOM (fallback - limited info)
    elif args.sbom:
        try:
            with open(args.sbom, "r") as f:
                sbom = json.load(f)
        except FileNotFoundError:
            print(f"Error: SBOM file not found: {args.sbom}", file=sys.stderr)
            return 1
        except json.JSONDecodeError as e:
            print(f"Error: Invalid JSON in SBOM file: {e}", file=sys.stderr)
            return 1
        
        # Extract packages
        for pkg in sbom.get("packages", []):
            if pkg.get("SPDXID") == "SPDXRef-Package-root":
                continue
            
            pkg_data = {
                "name": pkg.get("name", "unknown"),
                "version": pkg.get("versionInfo", "unknown"),
            }
            
            # Extract PURL
            for ref in pkg.get("externalRefs", []):
                if ref.get("referenceType") == "purl":
                    pkg_data["purl"] = ref.get("referenceLocator", "")
                    # Determine type from PURL
                    purl = pkg_data["purl"]
                    if purl.startswith("pkg:maven/"):
                        pkg_data["type"] = "maven"
                        # Extract group from PURL
                        parts = purl.split("/")
                        if len(parts) >= 3:
                            pkg_data["group"] = parts[1].replace("pkg:", "")
                    elif purl.startswith("pkg:npm/"):
                        pkg_data["type"] = "npm"
                    elif purl.startswith("pkg:pypi/"):
                        pkg_data["type"] = "pypi"
                    break
            
            packages.append(pkg_data)
        
        print(f"Processed {len(packages)} packages")
    
    # Generate JSON graph
    if args.output_json:
        graph_json = generate_graph_json(packages)
        with open(args.output_json, "w") as f:
            json.dump(graph_json, f, indent=2)
        print(f"JSON graph written to {args.output_json}")
    
    # Generate GraphML graph
    if args.output_graphml:
        graphml = generate_graphml(packages)
        with open(args.output_graphml, "w") as f:
            f.write(graphml)
        print(f"GraphML graph written to {args.output_graphml}")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
