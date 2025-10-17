#!/usr/bin/env python3
"""Validate SPDX SBOM files.

This script validates SPDX SBOM files against the SPDX 2.3 specification.
"""

import argparse
import json
import sys
from typing import Any, Dict, List, Tuple


def validate_spdx_required_fields(sbom: Dict[str, Any]) -> List[str]:
    """Validate required SPDX fields are present.
    
    Args:
        sbom: SPDX document dictionary
        
    Returns:
        List of validation errors (empty if valid)
    """
    errors = []
    
    # Check top-level required fields
    required_fields = [
        "spdxVersion",
        "dataLicense",
        "SPDXID",
        "name",
        "documentNamespace",
        "creationInfo",
    ]
    
    for field in required_fields:
        if field not in sbom:
            errors.append(f"Missing required field: {field}")
    
    # Validate SPDX version
    if sbom.get("spdxVersion") != "SPDX-2.3":
        errors.append(f"Invalid spdxVersion: {sbom.get('spdxVersion')}, expected SPDX-2.3")
    
    # Validate data license
    if sbom.get("dataLicense") != "CC0-1.0":
        errors.append(f"Invalid dataLicense: {sbom.get('dataLicense')}, expected CC0-1.0")
    
    # Validate SPDXID
    if sbom.get("SPDXID") != "SPDXRef-DOCUMENT":
        errors.append(f"Invalid SPDXID: {sbom.get('SPDXID')}, expected SPDXRef-DOCUMENT")
    
    # Check creationInfo
    if "creationInfo" in sbom:
        creation_info = sbom["creationInfo"]
        if "created" not in creation_info:
            errors.append("Missing required field: creationInfo.created")
        if "creators" not in creation_info or not creation_info["creators"]:
            errors.append("Missing or empty field: creationInfo.creators")
    
    # Check packages
    if "packages" not in sbom:
        errors.append("Missing required field: packages")
    elif not isinstance(sbom["packages"], list):
        errors.append("Field 'packages' must be a list")
    else:
        for i, pkg in enumerate(sbom["packages"]):
            pkg_errors = validate_package(pkg, i)
            errors.extend(pkg_errors)
    
    # Check relationships
    if "relationships" not in sbom:
        errors.append("Missing required field: relationships")
    elif not isinstance(sbom["relationships"], list):
        errors.append("Field 'relationships' must be a list")
    
    return errors


def validate_package(pkg: Dict[str, Any], index: int) -> List[str]:
    """Validate a package entry.
    
    Args:
        pkg: Package dictionary
        index: Index in the packages list
        
    Returns:
        List of validation errors
    """
    errors = []
    prefix = f"Package {index}"
    
    required_fields = ["SPDXID", "name", "downloadLocation"]
    for field in required_fields:
        if field not in pkg:
            errors.append(f"{prefix}: Missing required field: {field}")
    
    # Validate SPDXID format
    if "SPDXID" in pkg:
        spdx_id = pkg["SPDXID"]
        if not spdx_id.startswith("SPDXRef-"):
            errors.append(f"{prefix}: Invalid SPDXID format: {spdx_id}, must start with 'SPDXRef-'")
    
    # Validate downloadLocation
    if "downloadLocation" in pkg:
        location = pkg["downloadLocation"]
        if location not in ["NOASSERTION", "NONE"] and not location.startswith(("http://", "https://", "git://", "ftp://")):
            errors.append(f"{prefix}: Invalid downloadLocation: {location}")
    
    return errors


def validate_sbom_file(filepath: str) -> Tuple[bool, List[str]]:
    """Validate a single SBOM file.
    
    Args:
        filepath: Path to the SBOM file
        
    Returns:
        Tuple of (is_valid, list_of_errors)
    """
    try:
        with open(filepath, "r") as f:
            sbom = json.load(f)
    except FileNotFoundError:
        return False, [f"File not found: {filepath}"]
    except json.JSONDecodeError as e:
        return False, [f"Invalid JSON: {e}"]
    
    errors = validate_spdx_required_fields(sbom)
    
    return len(errors) == 0, errors


def main():
    parser = argparse.ArgumentParser(
        description="Validate SPDX SBOM files"
    )
    parser.add_argument(
        "files",
        nargs="+",
        help="SBOM files to validate"
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Print detailed validation results"
    )
    
    args = parser.parse_args()
    
    all_valid = True
    total_files = len(args.files)
    valid_files = 0
    
    for filepath in args.files:
        is_valid, errors = validate_sbom_file(filepath)
        
        if is_valid:
            valid_files += 1
            if args.verbose:
                print(f"✓ {filepath}: Valid")
        else:
            all_valid = False
            print(f"✗ {filepath}: Invalid")
            for error in errors:
                print(f"  - {error}")
    
    print(f"\nValidation complete: {valid_files}/{total_files} files valid")
    
    return 0 if all_valid else 1


if __name__ == "__main__":
    sys.exit(main())
