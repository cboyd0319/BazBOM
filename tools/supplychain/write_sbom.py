#!/usr/bin/env python3
"""Generate SPDX and CycloneDX SBOM documents from dependency data.

This script converts dependency information collected by Bazel aspects
into SPDX 2.3 or CycloneDX 1.5 compliant SBOM documents.
"""

import argparse
import json
import sys
from datetime import datetime, timezone
from typing import Any, Dict, List
from uuid import uuid4


def generate_spdx_document(packages: List[Dict[str, Any]], name: str) -> Dict[str, Any]:
    """Generate an SPDX 2.3 document.
    
    Args:
        packages: List of package information dictionaries
        name: Name for the SBOM document
        
    Returns:
        SPDX document as a dictionary
    """
    document_namespace = f"https://example.com/sboms/{name}-{uuid4()}"
    timestamp = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    
    # Create SPDX document structure
    doc = {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": name,
        "documentNamespace": document_namespace,
        "creationInfo": {
            "created": timestamp,
            "creators": ["Tool: BazBOM"],
            "licenseListVersion": "3.21"
        },
        "packages": [],
        "relationships": []
    }
    
    # Add root package
    root_package = {
        "SPDXID": "SPDXRef-Package-root",
        "name": name,
        "versionInfo": "1.0.0",
        "filesAnalyzed": False,
        "licenseConcluded": "NOASSERTION",
        "licenseDeclared": "NOASSERTION",
        "downloadLocation": "NOASSERTION",
    }
    doc["packages"].append(root_package)
    
    # Document describes root package
    doc["relationships"].append({
        "spdxElementId": "SPDXRef-DOCUMENT",
        "relationshipType": "DESCRIBES",
        "relatedSpdxElement": "SPDXRef-Package-root"
    })
    
    # Build map of package IDs for dependency resolution
    package_id_map = {}
    
    # Add dependency packages
    for pkg in packages:
        pkg_id = sanitize_spdx_id(f"SPDXRef-Package-{pkg['name']}")
        package_id_map[f"{pkg.get('group', '')}:{pkg.get('name', '')}"] = pkg_id
        
        pkg_entry = {
            "SPDXID": pkg_id,
            "name": pkg.get("name", "unknown"),
            "versionInfo": pkg.get("version", "unknown"),
            "filesAnalyzed": False,
            "licenseConcluded": pkg.get("license", "NOASSERTION"),
            "licenseDeclared": pkg.get("license", "NOASSERTION"),
            "downloadLocation": pkg.get("url", "NOASSERTION"),
        }
        
        # Add package URL if available
        if "purl" in pkg:
            pkg_entry["externalRefs"] = [{
                "referenceCategory": "PACKAGE-MANAGER",
                "referenceType": "purl",
                "referenceLocator": pkg["purl"]
            }]
        
        # Add checksums if available (SHA256)
        if pkg.get("sha256"):
            pkg_entry["checksums"] = [{
                "algorithm": "SHA256",
                "checksumValue": pkg["sha256"]
            }]
        
        doc["packages"].append(pkg_entry)
        
        # Add direct dependency relationship from root
        if pkg.get("is_direct", False):
            doc["relationships"].append({
                "spdxElementId": "SPDXRef-Package-root",
                "relationshipType": "DEPENDS_ON",
                "relatedSpdxElement": pkg_id
            })
    
    # Add transitive dependency relationships
    for pkg in packages:
        pkg_id = sanitize_spdx_id(f"SPDXRef-Package-{pkg['name']}")
        dependencies = pkg.get("dependencies", [])
        
        for dep_coord in dependencies:
            # Try to find the dependency in our package map
            if dep_coord in package_id_map:
                dep_id = package_id_map[dep_coord]
                doc["relationships"].append({
                    "spdxElementId": pkg_id,
                    "relationshipType": "DEPENDS_ON",
                    "relatedSpdxElement": dep_id
                })
    
    return doc


def sanitize_spdx_id(spdx_id: str) -> str:
    """Sanitize a string to be a valid SPDX ID.
    
    Args:
        spdx_id: Input string
        
    Returns:
        Valid SPDX ID (only alphanumeric, dots, and hyphens)
    """
    import re
    return re.sub(r'[^A-Za-z0-9.-]', '-', spdx_id)


def generate_cyclonedx_document(packages: List[Dict[str, Any]], name: str) -> Dict[str, Any]:
    """Generate a CycloneDX 1.5 document.
    
    Args:
        packages: List of package information dictionaries
        name: Name for the SBOM document
        
    Returns:
        CycloneDX document as a dictionary
    """
    timestamp = datetime.now(timezone.utc).isoformat()
    document_serial_number = f"urn:uuid:{uuid4()}"
    
    # Create CycloneDX document structure
    doc = {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": document_serial_number,
        "version": 1,
        "metadata": {
            "timestamp": timestamp,
            "tools": [
                {
                    "vendor": "BazBOM",
                    "name": "BazBOM",
                    "version": "1.0.0"
                }
            ],
            "component": {
                "type": "application",
                "name": name,
                "version": "1.0.0",
                "bom-ref": "pkg:generic/application@1.0.0"
            }
        },
        "components": [],
        "dependencies": []
    }
    
    # Add root component to dependencies
    root_dependency = {
        "ref": "pkg:generic/application@1.0.0",
        "dependsOn": []
    }
    
    # Add dependency packages
    for pkg in packages:
        purl = pkg.get("purl", f"pkg:maven/{pkg.get('group', 'unknown')}/{pkg.get('name', 'unknown')}@{pkg.get('version', 'unknown')}")
        
        component = {
            "type": "library",
            "bom-ref": purl,
            "purl": purl,
            "name": pkg.get("name", "unknown"),
            "version": pkg.get("version", "unknown"),
        }
        
        # Add group/publisher if available
        if pkg.get("group"):
            component["group"] = pkg["group"]
        
        # Add licenses if available
        if pkg.get("license") and pkg["license"] != "NOASSERTION":
            component["licenses"] = [
                {
                    "license": {
                        "id": pkg["license"]
                    }
                }
            ]
        
        # Add checksums if available
        if pkg.get("sha256"):
            component["hashes"] = [
                {
                    "alg": "SHA-256",
                    "content": pkg["sha256"]
                }
            ]
        
        # Add external references (download location)
        if pkg.get("url") and pkg["url"] != "NOASSERTION":
            component["externalReferences"] = [
                {
                    "type": "distribution",
                    "url": pkg["url"]
                }
            ]
        
        doc["components"].append(component)
        
        # Add to root dependencies if direct
        if pkg.get("is_direct", False):
            root_dependency["dependsOn"].append(purl)
        
        # Add component's own dependencies
        dependencies = pkg.get("dependencies", [])
        if dependencies:
            comp_dependency = {
                "ref": purl,
                "dependsOn": []
            }
            
            for dep_coord in dependencies:
                # Try to construct purl from coordinate
                parts = dep_coord.split(":")
                if len(parts) >= 2:
                    dep_purl = f"pkg:maven/{parts[0]}/{parts[1]}"
                    if len(parts) >= 3:
                        dep_purl += f"@{parts[2]}"
                    comp_dependency["dependsOn"].append(dep_purl)
            
            if comp_dependency["dependsOn"]:
                doc["dependencies"].append(comp_dependency)
    
    # Add root dependency
    doc["dependencies"].insert(0, root_dependency)
    
    return doc


def main():
    parser = argparse.ArgumentParser(
        description="Generate SPDX or CycloneDX SBOM from dependency data"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with dependency data"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output SBOM JSON file"
    )
    parser.add_argument(
        "--name",
        default="application",
        help="Name for the SBOM document"
    )
    parser.add_argument(
        "--format",
        choices=["spdx", "cyclonedx"],
        default="spdx",
        help="SBOM format (default: spdx)"
    )
    
    args = parser.parse_args()
    
    # Read input dependency data
    try:
        with open(args.input, "r") as f:
            data = json.load(f)
            packages = data.get("packages", [])
    except FileNotFoundError:
        print(f"Error: Input file not found: {args.input}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in input file: {e}", file=sys.stderr)
        return 1
    
    # Generate SBOM document based on format
    if args.format == "cyclonedx":
        sbom_doc = generate_cyclonedx_document(packages, args.name)
        print(f"CycloneDX SBOM written to {args.output}")
    else:
        sbom_doc = generate_spdx_document(packages, args.name)
        print(f"SPDX SBOM written to {args.output}")
    
    # Write output
    try:
        with open(args.output, "w") as f:
            json.dump(sbom_doc, f, indent=2)
    except IOError as e:
        print(f"Error writing output file: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
