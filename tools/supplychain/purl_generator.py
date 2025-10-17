#!/usr/bin/env python3
"""Generate Package URLs (PURLs) from Maven coordinates.

This script converts Maven coordinates to Package URL (PURL) format
as specified in https://github.com/package-url/purl-spec
"""

import argparse
import json
import sys
from typing import Dict, Optional
from urllib.parse import quote


def maven_to_purl(
    group_id: str,
    artifact_id: str,
    version: str,
    classifier: Optional[str] = None,
    packaging: Optional[str] = None,
) -> str:
    """Convert Maven coordinates to PURL format.
    
    Args:
        group_id: Maven group ID (e.g., 'com.google.guava')
        artifact_id: Maven artifact ID (e.g., 'guava')
        version: Version string (e.g., '31.1-jre')
        classifier: Optional classifier (e.g., 'sources', 'javadoc')
        packaging: Optional packaging type (e.g., 'jar', 'war')
        
    Returns:
        PURL string (e.g., 'pkg:maven/com.google.guava/guava@31.1-jre')
    """
    # Encode components
    # For Maven, namespace keeps dots but we replace with / for PURL format
    namespace = group_id.replace(".", "/")
    name = quote(artifact_id, safe="")
    version_encoded = quote(version, safe="")
    
    # Build base PURL
    purl = f"pkg:maven/{namespace}/{name}@{version_encoded}"
    
    # Add qualifiers if present
    qualifiers = []
    if classifier:
        qualifiers.append(f"classifier={quote(classifier, safe='')}")
    if packaging and packaging != "jar":  # jar is default
        qualifiers.append(f"type={quote(packaging, safe='')}")
    
    if qualifiers:
        purl += "?" + "&".join(qualifiers)
    
    return purl


def parse_maven_coordinates(coords: str) -> Dict[str, str]:
    """Parse Maven coordinates string.
    
    Supports formats:
    - group:artifact:version
    - group:artifact:packaging:version
    - group:artifact:packaging:classifier:version
    
    Args:
        coords: Maven coordinates string
        
    Returns:
        Dictionary with parsed components
    """
    parts = coords.split(":")
    
    if len(parts) < 3:
        raise ValueError(f"Invalid Maven coordinates: {coords}")
    
    result = {
        "group_id": parts[0],
        "artifact_id": parts[1],
    }
    
    if len(parts) == 3:
        result["version"] = parts[2]
    elif len(parts) == 4:
        result["packaging"] = parts[2]
        result["version"] = parts[3]
    elif len(parts) == 5:
        result["packaging"] = parts[2]
        result["classifier"] = parts[3]
        result["version"] = parts[4]
    else:
        raise ValueError(f"Unsupported Maven coordinates format: {coords}")
    
    return result


def process_dependencies(input_file: str, output_file: str) -> None:
    """Process dependencies file and add PURLs.
    
    Args:
        input_file: Path to input JSON with Maven coordinates
        output_file: Path to output JSON with PURLs added
    """
    try:
        with open(input_file, "r") as f:
            data = json.load(f)
        
        # Process each dependency
        if isinstance(data, dict) and "dependencies" in data:
            for dep in data["dependencies"]:
                if "coordinates" in dep:
                    coords = parse_maven_coordinates(dep["coordinates"])
                    dep["purl"] = maven_to_purl(
                        coords["group_id"],
                        coords["artifact_id"],
                        coords["version"],
                        coords.get("classifier"),
                        coords.get("packaging"),
                    )
                elif all(k in dep for k in ["group", "artifact", "version"]):
                    dep["purl"] = maven_to_purl(
                        dep["group"],
                        dep["artifact"],
                        dep["version"],
                        dep.get("classifier"),
                        dep.get("packaging"),
                    )
        elif isinstance(data, list):
            for dep in data:
                if "coordinates" in dep:
                    coords = parse_maven_coordinates(dep["coordinates"])
                    dep["purl"] = maven_to_purl(
                        coords["group_id"],
                        coords["artifact_id"],
                        coords["version"],
                        coords.get("classifier"),
                        coords.get("packaging"),
                    )
        
        # Write output
        with open(output_file, "w") as f:
            json.dump(data, f, indent=2)
        
        print(f"Successfully generated PURLs and wrote to {output_file}", file=sys.stderr)
        
    except Exception as e:
        print(f"Error processing dependencies: {e}", file=sys.stderr)
        sys.exit(1)


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Generate Package URLs (PURLs) from Maven coordinates"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with Maven coordinates",
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output JSON file with PURLs added",
    )
    parser.add_argument(
        "--coordinates",
        help="Single Maven coordinates to convert (for testing)",
    )
    
    args = parser.parse_args()
    
    if args.coordinates:
        # Single coordinate conversion (for testing)
        coords = parse_maven_coordinates(args.coordinates)
        purl = maven_to_purl(
            coords["group_id"],
            coords["artifact_id"],
            coords["version"],
            coords.get("classifier"),
            coords.get("packaging"),
        )
        print(purl)
    else:
        # Process full file
        process_dependencies(args.input, args.output)


if __name__ == "__main__":
    main()
