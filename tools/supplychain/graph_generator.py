#!/usr/bin/env python3
"""Generate dependency graphs from SBOM data.

This script converts SBOM data into dependency graph formats including
JSON and GraphML for visualization tools like Gephi or yEd.
"""

import argparse
import json
import sys
from typing import Any, Dict, List, Set


def generate_graph_json(packages: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Generate a JSON dependency graph.
    
    Args:
        packages: List of package dictionaries
        
    Returns:
        Graph in JSON format with nodes and edges
    """
    nodes = []
    edges = []
    seen_packages = set()
    
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
        
        if pkg_id not in seen_packages:
            node = {
                "id": pkg_id,
                "name": pkg.get("name", "unknown"),
                "version": pkg.get("version", "unknown"),
                "type": pkg.get("type", "unknown"),
                "depth": 1,  # Direct dependency
            }
            
            if "group" in pkg:
                node["group"] = pkg["group"]
            
            nodes.append(node)
            seen_packages.add(pkg_id)
            
            # Create edge from root to this package
            edges.append({
                "from": "root",
                "to": pkg_id,
                "type": "depends_on"
            })
    
    return {
        "version": "1.0",
        "graph": {
            "nodes": nodes,
            "edges": edges
        }
    }


def generate_graphml(packages: List[Dict[str, Any]]) -> str:
    """Generate a GraphML dependency graph.
    
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
    graphml.append('  <key id="edgetype" for="edge" attr.name="type" attr.type="string"/>')
    
    graphml.append('  <graph id="G" edgedefault="directed">')
    
    # Add root node
    graphml.append('    <node id="root">')
    graphml.append('      <data key="name">root</data>')
    graphml.append('      <data key="type">root</data>')
    graphml.append('    </node>')
    
    # Add package nodes
    seen_packages = set()
    for pkg in packages:
        pkg_id = pkg.get("purl", pkg.get("name", "unknown"))
        
        if pkg_id not in seen_packages:
            # Escape XML special characters
            pkg_id_safe = pkg_id.replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            name = pkg.get("name", "unknown").replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            version = pkg.get("version", "unknown").replace('&', '&amp;').replace('<', '&lt;').replace('>', '&gt;')
            pkg_type = pkg.get("type", "unknown")
            
            graphml.append(f'    <node id="{pkg_id_safe}">')
            graphml.append(f'      <data key="name">{name}</data>')
            graphml.append(f'      <data key="version">{version}</data>')
            graphml.append(f'      <data key="type">{pkg_type}</data>')
            graphml.append(f'      <data key="purl">{pkg_id_safe}</data>')
            graphml.append('    </node>')
            
            seen_packages.add(pkg_id)
            
            # Add edge from root to this package
            graphml.append(f'    <edge source="root" target="{pkg_id_safe}">')
            graphml.append('      <data key="edgetype">depends_on</data>')
            graphml.append('    </edge>')
    
    graphml.append('  </graph>')
    graphml.append('</graphml>')
    
    return '\n'.join(graphml)


def main():
    parser = argparse.ArgumentParser(
        description="Generate dependency graphs from SBOM"
    )
    parser.add_argument(
        "--sbom",
        required=True,
        help="Path to SPDX SBOM file"
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
    
    if not args.output_json and not args.output_graphml:
        print("Error: At least one output format must be specified", file=sys.stderr)
        return 1
    
    # Read SBOM
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
    packages = []
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
    
    print(f"Processed {len(packages)} packages")
    return 0


if __name__ == "__main__":
    sys.exit(main())
