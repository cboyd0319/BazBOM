#!/usr/bin/env python3
"""Convert OSV vulnerability data to SARIF format.

This script converts vulnerability data from OSV into SARIF 2.1.0 format
for upload to GitHub Code Scanning.
"""

import argparse
import json
import sys
from typing import Any, Dict, List


def severity_to_level(severity: str) -> str:
    """Convert OSV severity to SARIF level.
    
    Args:
        severity: OSV severity (CRITICAL, HIGH, MEDIUM, LOW)
        
    Returns:
        SARIF level (error, warning, note)
    """
    severity_upper = severity.upper()
    if severity_upper in ("CRITICAL", "HIGH"):
        return "error"
    elif severity_upper == "MEDIUM":
        return "warning"
    else:
        return "note"


def create_sarif_document(vulnerabilities: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Create a SARIF 2.1.0 document from vulnerability data.
    
    Args:
        vulnerabilities: List of vulnerability dictionaries from OSV
        
    Returns:
        SARIF document as a dictionary
    """
    results = []
    
    for vuln_data in vulnerabilities:
        vuln = vuln_data.get("vulnerability", {})
        package = vuln_data.get("package", "unknown")
        version = vuln_data.get("version", "unknown")
        
        vuln_id = vuln.get("id", "UNKNOWN")
        summary = vuln.get("summary", "No summary available")
        
        # Determine severity
        severity = "MEDIUM"
        for item in vuln.get("severity", []):
            if item.get("type") == "CVSS_V3":
                score = item.get("score", "")
                # Extract base score
                if ":" in score:
                    base_score = float(score.split(":")[0].split("/")[-1])
                    if base_score >= 9.0:
                        severity = "CRITICAL"
                    elif base_score >= 7.0:
                        severity = "HIGH"
                    elif base_score >= 4.0:
                        severity = "MEDIUM"
                    else:
                        severity = "LOW"
                break
        
        # Get fixed version if available
        fixed_version = None
        for affected in vuln.get("affected", []):
            for ranges in affected.get("ranges", []):
                for event in ranges.get("events", []):
                    if "fixed" in event:
                        fixed_version = event["fixed"]
                        break
                if fixed_version:
                    break
            if fixed_version:
                break
        
        # Create SARIF result
        message_text = f"{summary}\n\nAffected: {package}@{version}"
        if fixed_version:
            message_text += f"\nFixed in: {fixed_version}"
        
        result = {
            "ruleId": vuln_id,
            "level": severity_to_level(severity),
            "message": {
                "text": message_text
            },
            "locations": [
                {
                    "physicalLocation": {
                        "artifactLocation": {
                            "uri": "pom.xml"  # Generic location
                        }
                    }
                }
            ],
            "properties": {
                "severity": severity,
                "package": package,
                "version": version,
            }
        }
        
        if fixed_version:
            result["properties"]["fixedVersion"] = fixed_version
        
        results.append(result)
    
    # Create SARIF document
    sarif = {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [
            {
                "tool": {
                    "driver": {
                        "name": "BazBOM SCA",
                        "version": "1.0.0",
                        "informationUri": "https://github.com/cboyd0319/BazBOM",
                        "rules": []
                    }
                },
                "results": results
            }
        ]
    }
    
    # Add rules for each unique vulnerability
    rules = {}
    for vuln_data in vulnerabilities:
        vuln = vuln_data.get("vulnerability", {})
        vuln_id = vuln.get("id", "UNKNOWN")
        if vuln_id not in rules:
            rules[vuln_id] = {
                "id": vuln_id,
                "shortDescription": {
                    "text": vuln.get("summary", "No summary available")
                },
                "helpUri": f"https://osv.dev/vulnerability/{vuln_id}",
                "properties": {
                    "tags": ["security", "vulnerability"]
                }
            }
    
    sarif["runs"][0]["tool"]["driver"]["rules"] = list(rules.values())
    
    return sarif


def main():
    parser = argparse.ArgumentParser(
        description="Convert OSV data to SARIF format"
    )
    parser.add_argument(
        "--input",
        required=True,
        help="Input JSON file with OSV vulnerability data"
    )
    parser.add_argument(
        "--output",
        required=True,
        help="Output SARIF file"
    )
    
    args = parser.parse_args()
    
    # Read vulnerability data
    try:
        with open(args.input, "r") as f:
            data = json.load(f)
        vulnerabilities = data.get("vulnerabilities", [])
    except FileNotFoundError:
        print(f"Error: Input file not found: {args.input}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in input file: {e}", file=sys.stderr)
        return 1
    
    # Generate SARIF
    sarif = create_sarif_document(vulnerabilities)
    
    # Write output
    try:
        with open(args.output, "w") as f:
            json.dump(sarif, f, indent=2)
        print(f"SARIF written to {args.output}")
        print(f"Generated {len(vulnerabilities)} vulnerability findings")
    except IOError as e:
        print(f"Error writing output file: {e}", file=sys.stderr)
        return 1
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
