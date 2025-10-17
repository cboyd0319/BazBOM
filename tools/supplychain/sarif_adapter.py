#!/usr/bin/env python3
"""Convert OSV vulnerability data to SARIF format.

This script converts vulnerability data from OSV into SARIF 2.1.0 format
for upload to GitHub Code Scanning.
"""

import argparse
import json
import sys
from typing import Any, Dict, List


def priority_to_level(priority: str, severity: str = "MEDIUM") -> str:
    """Convert enriched priority or severity to SARIF level.
    
    Args:
        priority: Enriched priority (P0-IMMEDIATE, P1-CRITICAL, etc.)
        severity: Fallback OSV severity (CRITICAL, HIGH, MEDIUM, LOW)
        
    Returns:
        SARIF level (error, warning, note)
    """
    # Priority-based mapping (if enriched)
    if priority:
        priority_upper = priority.upper()
        if "P0" in priority_upper or "P1" in priority_upper:
            return "error"
        elif "P2" in priority_upper:
            return "warning"
        else:
            return "note"
    
    # Fallback to severity-based mapping
    severity_upper = severity.upper()
    if severity_upper in ("CRITICAL", "HIGH"):
        return "error"
    elif severity_upper == "MEDIUM":
        return "warning"
    else:
        return "note"


def severity_to_level(severity: str) -> str:
    """Convert OSV severity to SARIF level (legacy compatibility).
    
    Args:
        severity: OSV severity (CRITICAL, HIGH, MEDIUM, LOW)
        
    Returns:
        SARIF level (error, warning, note)
    """
    return priority_to_level("", severity)


def format_enriched_message(vuln_data: Dict[str, Any]) -> str:
    """Format SARIF message with enrichment context.
    
    Args:
        vuln_data: Vulnerability data with optional enrichment
        
    Returns:
        Formatted message text with KEV/EPSS/GHSA context
    """
    # Extract basic info
    vuln = vuln_data.get("vulnerability", {})
    package = vuln_data.get("package", "unknown")
    version = vuln_data.get("version", "unknown")
    cve = vuln_data.get("cve") or vuln.get("id", "UNKNOWN")
    summary = vuln_data.get("summary") or vuln.get("summary", "No summary available")
    
    # Start message
    message = f"{summary}\n\nAffected: {package}@{version}"
    
    # Add KEV warning (highest priority)
    kev = vuln_data.get("kev", {})
    if kev.get("in_kev"):
        message += "\n\n⚠️ KNOWN EXPLOITED IN THE WILD (CISA KEV)"
        if kev.get("vulnerability_name"):
            message += f"\nVulnerability Name: {kev['vulnerability_name']}"
        if kev.get("due_date"):
            message += f"\nRemediation Due Date: {kev['due_date']}"
        if kev.get("required_action"):
            message += f"\nRequired Action: {kev['required_action']}"
    
    # Add EPSS context
    epss = vuln_data.get("epss", {})
    if epss.get("epss_score"):
        prob = vuln_data.get("exploitation_probability", "N/A")
        percentile = epss.get("epss_percentile", 0)
        message += f"\n\nExploitation Probability: {prob} (EPSS)"
        message += f"\nEPSS Percentile: Top {(1 - percentile) * 100:.1f}%"
    
    # Add exploit status
    exploit = vuln_data.get("exploit", {})
    if exploit.get("weaponized"):
        message += "\n\n⚠️ WEAPONIZED EXPLOIT AVAILABLE"
        if exploit.get("exploit_maturity"):
            message += f"\nExploit Maturity: {exploit['exploit_maturity']}"
    elif exploit.get("exploit_available"):
        message += f"\n\nPublic exploit available ({exploit.get('exploit_maturity', 'unknown')} maturity)"
    
    # Add remediation
    fixed_version = None
    remediation = vuln_data.get("remediation", {})
    if remediation.get("fixed_version"):
        fixed_version = remediation["fixed_version"]
    else:
        # Fallback to old extraction logic
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
    
    if fixed_version:
        message += f"\n\nFixed in version: {fixed_version}"
    
    # Add GHSA info
    ghsa = vuln_data.get("ghsa", {})
    if ghsa.get("ghsa_id"):
        message += f"\n\nGitHub Security Advisory: {ghsa['ghsa_id']}"
        if ghsa.get("permalink"):
            message += f"\n{ghsa['permalink']}"
    
    # Add priority and risk score
    if vuln_data.get("priority"):
        message += f"\n\nPriority: {vuln_data['priority']}"
    if vuln_data.get("risk_score"):
        message += f"\nRisk Score: {vuln_data['risk_score']}/100"
    
    return message


def create_sarif_document(vulnerabilities: List[Dict[str, Any]]) -> Dict[str, Any]:
    """Create a SARIF 2.1.0 document from vulnerability data.
    
    Supports both legacy (unenriched) and enriched vulnerability data.
    
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
        
        # Get IDs (prefer CVE)
        cve = vuln_data.get("cve")
        vuln_id = cve or vuln_data.get("id") or vuln.get("id", "UNKNOWN")
        
        # Determine severity (use enriched if available)
        severity = vuln_data.get("severity", "MEDIUM")
        priority = vuln_data.get("priority", "")
        
        # Create SARIF result with enriched message
        message_text = format_enriched_message(vuln_data)
        
        result = {
            "ruleId": vuln_id,
            "level": priority_to_level(priority, severity),
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
        
        # Add enriched properties if available
        if vuln_data.get("cvss_score"):
            result["properties"]["cvssScore"] = vuln_data["cvss_score"]
        
        if vuln_data.get("risk_score"):
            result["properties"]["riskScore"] = vuln_data["risk_score"]
        
        if vuln_data.get("priority"):
            result["properties"]["priority"] = vuln_data["priority"]
        
        if vuln_data.get("kev", {}).get("in_kev"):
            result["properties"]["inKEV"] = True
            result["properties"]["kevDueDate"] = vuln_data["kev"].get("due_date", "")
        
        if vuln_data.get("epss", {}).get("epss_score"):
            result["properties"]["epssScore"] = vuln_data["epss"]["epss_score"]
            result["properties"]["exploitationProbability"] = vuln_data.get("exploitation_probability", "")
        
        if vuln_data.get("exploit", {}).get("weaponized"):
            result["properties"]["weaponizedExploit"] = True
        
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
