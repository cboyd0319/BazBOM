#!/usr/bin/env python3
"""Generate SPDX SBOM documents from dependency data.

This script converts dependency information collected by Bazel aspects
into SPDX 2.3 compliant SBOM documents.
"""

import argparse
import json
import sys
from datetime import datetime
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
    timestamp = datetime.now(datetime.UTC).strftime("%Y-%m-%dT%H:%M:%SZ") if hasattr(datetime, 'UTC') else datetime.utcnow().strftime("%Y-%m-%dT%H:%M:%SZ")
    
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
    
    # Add dependency packages
    for pkg in packages:
        pkg_id = sanitize_spdx_id(f"SPDXRef-Package-{pkg['name']}")
        
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
        
        doc["packages"].append(pkg_entry)
        
        # Add dependency relationship
        doc["relationships"].append({
            "spdxElementId": "SPDXRef-Package-root",
            "relationshipType": "DEPENDS_ON",
            "relatedSpdxElement": pkg_id
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


def main():
    parser = argparse.ArgumentParser(
        description="Generate SPDX SBOM from dependency data"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with dependency data"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output SPDX JSON file"
    )
    parser.add_argument(
        "--name",
        default="application",
        help="Name for the SBOM document"
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
    
    # Generate SPDX document
    spdx_doc = generate_spdx_document(packages, args.name)
    
    # Write output
    try:
        with open(args.output, "w") as f:
            json.dump(spdx_doc, f, indent=2)
        print(f"SBOM written to {args.output}")
    except IOError as e:
        print(f"Error writing output file: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
