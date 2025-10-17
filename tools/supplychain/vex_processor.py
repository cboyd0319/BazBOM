#!/usr/bin/env python3
"""VEX (Vulnerability Exploitability eXchange) processor for filtering false positive vulnerabilities."""

import argparse
import json
import sys
from typing import Dict, List, Any, Optional
from datetime import datetime
from pathlib import Path


def load_vex_statements(vex_dir: str) -> List[Dict[str, Any]]:
    """
    Load VEX statements from directory.
    
    Args:
        vex_dir: Directory containing VEX statement JSON files
        
    Returns:
        List of VEX statements
    """
    vex_dir_path = Path(vex_dir)
    if not vex_dir_path.exists():
        print(f"Warning: VEX directory {vex_dir} does not exist", file=sys.stderr)
        return []
    
    statements = []
    for vex_file in vex_dir_path.glob('*.json'):
        try:
            with open(vex_file, 'r') as f:
                statement = json.load(f)
                statements.append(statement)
        except Exception as e:
            print(f"Warning: Error loading VEX statement from {vex_file}: {e}", file=sys.stderr)
    
    return statements


def parse_vex_statement(statement: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """
    Parse VEX statement and extract key information.
    
    Args:
        statement: Raw VEX statement dictionary
        
    Returns:
        Parsed VEX statement or None if invalid
    """
    # Support multiple VEX formats (CSAF VEX, OpenVEX, etc.)
    
    # Try CSAF VEX format
    if 'document' in statement and 'vulnerabilities' in statement:
        return parse_csaf_vex(statement)
    
    # Try simplified format
    if 'cve' in statement or 'vulnerability_id' in statement:
        return {
            'vulnerability_id': statement.get('cve') or statement.get('vulnerability_id'),
            'package': statement.get('package'),
            'status': statement.get('status', 'not_affected'),
            'justification': statement.get('justification', ''),
            'created': statement.get('created', datetime.now().isoformat())
        }
    
    print(f"Warning: Unknown VEX statement format", file=sys.stderr)
    return None


def parse_csaf_vex(statement: Dict[str, Any]) -> Optional[Dict[str, Any]]:
    """Parse CSAF VEX format."""
    vulnerabilities = statement.get('vulnerabilities', [])
    if not vulnerabilities:
        return None
    
    # Take first vulnerability (simplified)
    vuln = vulnerabilities[0]
    
    product_status = vuln.get('product_status', {})
    # Determine status from product_status
    status = 'not_affected'
    if product_status.get('known_not_affected'):
        status = 'not_affected'
    elif product_status.get('fixed'):
        status = 'fixed'
    elif product_status.get('under_investigation'):
        status = 'under_investigation'
    
    return {
        'vulnerability_id': vuln.get('cve'),
        'package': None,  # Extract from product_status if needed
        'status': status,
        'justification': vuln.get('notes', [{}])[0].get('text', '') if vuln.get('notes') else '',
        'created': statement.get('document', {}).get('tracking', {}).get('current_release_date')
    }


def should_suppress_finding(finding: Dict[str, Any], vex_statements: List[Dict[str, Any]]) -> tuple[bool, Optional[str]]:
    """
    Check if a vulnerability finding should be suppressed based on VEX statements.
    
    Args:
        finding: Vulnerability finding
        vex_statements: List of VEX statements
        
    Returns:
        Tuple of (should_suppress, reason)
    """
    vuln_id = finding.get('id') or finding.get('cve')
    finding_cve = finding.get('cve')
    package_purl = finding.get('package', {}).get('purl', '')
    
    if not vuln_id:
        return False, None
    
    for statement in vex_statements:
        parsed = parse_vex_statement(statement)
        if not parsed:
            continue
        
        # Check if VEX statement matches this finding
        # Match against either the id or the cve field
        vex_vuln_id = parsed.get('vulnerability_id')
        if not vex_vuln_id:
            continue
            
        # Match if either the id or CVE matches
        if vex_vuln_id != vuln_id and vex_vuln_id != finding_cve:
            continue
        
        # If package is specified in VEX, check if it matches
        if parsed.get('package') and parsed['package'] not in package_purl:
            continue
        
        # Check status
        status = parsed.get('status', '').lower()
        if status in ['not_affected', 'false_positive', 'mitigated', 'accepted_risk']:
            justification = parsed.get('justification', 'No justification provided')
            return True, f"VEX statement: {status} - {justification}"
    
    return False, None


def filter_findings(findings: List[Dict[str, Any]], vex_statements: List[Dict[str, Any]]) -> tuple[List[Dict[str, Any]], List[Dict[str, Any]]]:
    """
    Filter findings based on VEX statements.
    
    Args:
        findings: List of vulnerability findings
        vex_statements: List of VEX statements
        
    Returns:
        Tuple of (remaining_findings, suppressed_findings)
    """
    remaining = []
    suppressed = []
    
    for finding in findings:
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        
        if should_suppress:
            # Add suppression metadata
            suppressed_finding = finding.copy()
            suppressed_finding['suppressed'] = True
            suppressed_finding['suppression_reason'] = reason
            suppressed.append(suppressed_finding)
        else:
            remaining.append(finding)
    
    return remaining, suppressed


def validate_vex_statement(statement: Dict[str, Any]) -> tuple[bool, List[str]]:
    """
    Validate VEX statement structure.
    
    Args:
        statement: VEX statement dictionary
        
    Returns:
        Tuple of (is_valid, error_messages)
    """
    errors = []
    
    # Check required fields for simplified format
    if 'cve' not in statement and 'vulnerability_id' not in statement:
        if 'document' not in statement:  # Not CSAF format either
            errors.append("Missing required field: 'cve' or 'vulnerability_id'")
    
    if 'status' in statement:
        valid_statuses = ['not_affected', 'false_positive', 'mitigated', 'accepted_risk', 'fixed', 'under_investigation']
        if statement['status'].lower() not in valid_statuses:
            errors.append(f"Invalid status: {statement['status']}. Must be one of {valid_statuses}")
    
    return len(errors) == 0, errors


def main():
    parser = argparse.ArgumentParser(description='VEX processor for filtering vulnerability findings')
    parser.add_argument('--vex-dir', required=True, help='Directory containing VEX statements (JSON files)')
    parser.add_argument('--sca-findings', required=True, help='Input SCA findings JSON file')
    parser.add_argument('--output', required=True, help='Output filtered findings JSON file')
    parser.add_argument('--suppressed-output', help='Optional output file for suppressed findings')
    parser.add_argument('--validate-only', action='store_true', help='Only validate VEX statements without filtering')
    
    args = parser.parse_args()
    
    # Load VEX statements
    print(f"Loading VEX statements from {args.vex_dir}", file=sys.stderr)
    vex_statements = load_vex_statements(args.vex_dir)
    print(f"Loaded {len(vex_statements)} VEX statements", file=sys.stderr)
    
    # Validate VEX statements
    invalid_count = 0
    for i, statement in enumerate(vex_statements):
        is_valid, errors = validate_vex_statement(statement)
        if not is_valid:
            invalid_count += 1
            print(f"Warning: VEX statement {i+1} is invalid:", file=sys.stderr)
            for error in errors:
                print(f"  - {error}", file=sys.stderr)
    
    if invalid_count > 0:
        print(f"Warning: {invalid_count} invalid VEX statements found", file=sys.stderr)
    
    if args.validate_only:
        print("Validation complete", file=sys.stderr)
        return 0 if invalid_count == 0 else 1
    
    # Load SCA findings
    print(f"Loading SCA findings from {args.sca_findings}", file=sys.stderr)
    with open(args.sca_findings, 'r') as f:
        sca_data = json.load(f)
    
    findings = sca_data.get('findings', [])
    print(f"Found {len(findings)} vulnerability findings", file=sys.stderr)
    
    # Apply VEX filters
    print("Applying VEX statements...", file=sys.stderr)
    remaining_findings, suppressed_findings = filter_findings(findings, vex_statements)
    
    print(f"Suppressed {len(suppressed_findings)} findings", file=sys.stderr)
    print(f"Remaining {len(remaining_findings)} findings", file=sys.stderr)
    
    # Update summary
    filtered_data = sca_data.copy()
    filtered_data['findings'] = remaining_findings
    filtered_data['vex_applied'] = True
    filtered_data['vex_statement_count'] = len(vex_statements)
    filtered_data['suppressed_count'] = len(suppressed_findings)
    
    if 'summary' in filtered_data:
        filtered_data['summary']['total_findings'] = len(remaining_findings)
        filtered_data['summary']['suppressed_findings'] = len(suppressed_findings)
        
        # Recalculate severity counts
        filtered_data['summary']['by_severity'] = {
            'critical': sum(1 for f in remaining_findings if f.get('severity') == 'CRITICAL'),
            'high': sum(1 for f in remaining_findings if f.get('severity') == 'HIGH'),
            'medium': sum(1 for f in remaining_findings if f.get('severity') == 'MEDIUM'),
            'low': sum(1 for f in remaining_findings if f.get('severity') == 'LOW')
        }
    
    # Write filtered findings
    with open(args.output, 'w') as f:
        json.dump(filtered_data, f, indent=2)
    print(f"Filtered findings written to {args.output}", file=sys.stderr)
    
    # Write suppressed findings if requested
    if args.suppressed_output and suppressed_findings:
        suppressed_data = {
            'scan_date': sca_data.get('scan_date'),
            'suppressed_findings': suppressed_findings,
            'vex_statement_count': len(vex_statements),
            'summary': {
                'total_suppressed': len(suppressed_findings),
                'by_severity': {
                    'critical': sum(1 for f in suppressed_findings if f.get('severity') == 'CRITICAL'),
                    'high': sum(1 for f in suppressed_findings if f.get('severity') == 'HIGH'),
                    'medium': sum(1 for f in suppressed_findings if f.get('severity') == 'MEDIUM'),
                    'low': sum(1 for f in suppressed_findings if f.get('severity') == 'LOW')
                }
            }
        }
        
        with open(args.suppressed_output, 'w') as f:
            json.dump(suppressed_data, f, indent=2)
        print(f"Suppressed findings written to {args.suppressed_output}", file=sys.stderr)
    
    return 0


if __name__ == '__main__':
    sys.exit(main())
