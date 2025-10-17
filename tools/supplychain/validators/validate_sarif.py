#!/usr/bin/env python3
"""Validate SARIF files.

This script validates SARIF files against the SARIF 2.1.0 specification.
"""

import argparse
import json
import sys
from typing import Any, Dict, List, Tuple


def validate_sarif_required_fields(sarif: Dict[str, Any]) -> List[str]:
    """Validate required SARIF fields are present.
    
    Args:
        sarif: SARIF document dictionary
        
    Returns:
        List of validation errors (empty if valid)
    """
    errors = []
    
    # Check top-level required fields
    if "version" not in sarif:
        errors.append("Missing required field: version")
    elif sarif["version"] != "2.1.0":
        errors.append(f"Invalid version: {sarif['version']}, expected 2.1.0")
    
    if "$schema" not in sarif:
        errors.append("Missing recommended field: $schema")
    
    # Check runs array
    if "runs" not in sarif:
        errors.append("Missing required field: runs")
        return errors
    
    if not isinstance(sarif["runs"], list):
        errors.append("Field 'runs' must be an array")
        return errors
    
    if not sarif["runs"]:
        errors.append("Field 'runs' must contain at least one run")
        return errors
    
    # Validate each run
    for i, run in enumerate(sarif["runs"]):
        run_errors = validate_run(run, i)
        errors.extend(run_errors)
    
    return errors


def validate_run(run: Dict[str, Any], index: int) -> List[str]:
    """Validate a run object.
    
    Args:
        run: Run dictionary
        index: Index in the runs array
        
    Returns:
        List of validation errors
    """
    errors = []
    prefix = f"Run {index}"
    
    # Check tool
    if "tool" not in run:
        errors.append(f"{prefix}: Missing required field: tool")
    else:
        tool = run["tool"]
        if "driver" not in tool:
            errors.append(f"{prefix}: Missing required field: tool.driver")
        else:
            driver = tool["driver"]
            if "name" not in driver:
                errors.append(f"{prefix}: Missing required field: tool.driver.name")
    
    # Check results
    if "results" not in run:
        errors.append(f"{prefix}: Missing required field: results")
    elif not isinstance(run["results"], list):
        errors.append(f"{prefix}: Field 'results' must be an array")
    else:
        for j, result in enumerate(run["results"]):
            result_errors = validate_result(result, index, j)
            errors.extend(result_errors)
    
    return errors


def validate_result(result: Dict[str, Any], run_index: int, result_index: int) -> List[str]:
    """Validate a result object.
    
    Args:
        result: Result dictionary
        run_index: Index of the run
        result_index: Index of the result in the run
        
    Returns:
        List of validation errors
    """
    errors = []
    prefix = f"Run {run_index}, Result {result_index}"
    
    # Check ruleId
    if "ruleId" not in result:
        errors.append(f"{prefix}: Missing required field: ruleId")
    
    # Check message
    if "message" not in result:
        errors.append(f"{prefix}: Missing required field: message")
    elif "text" not in result["message"]:
        errors.append(f"{prefix}: Missing required field: message.text")
    
    # Check level (optional but should be valid if present)
    if "level" in result:
        valid_levels = ["none", "note", "warning", "error"]
        if result["level"] not in valid_levels:
            errors.append(f"{prefix}: Invalid level: {result['level']}, must be one of {valid_levels}")
    
    return errors


def validate_sarif_file(filepath: str) -> Tuple[bool, List[str]]:
    """Validate a single SARIF file.
    
    Args:
        filepath: Path to the SARIF file
        
    Returns:
        Tuple of (is_valid, list_of_errors)
    """
    try:
        with open(filepath, "r") as f:
            sarif = json.load(f)
    except FileNotFoundError:
        return False, [f"File not found: {filepath}"]
    except json.JSONDecodeError as e:
        return False, [f"Invalid JSON: {e}"]
    
    errors = validate_sarif_required_fields(sarif)
    
    return len(errors) == 0, errors


def main():
    parser = argparse.ArgumentParser(
        description="Validate SARIF files"
    )
    parser.add_argument(
        "files",
        nargs="+",
        help="SARIF files to validate"
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
        is_valid, errors = validate_sarif_file(filepath)
        
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
