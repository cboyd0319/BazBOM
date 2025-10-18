#!/usr/bin/env python3
"""Export BazBOM scan results to CSV format.

This module provides utilities to export various BazBOM outputs (SBOMs,
vulnerability findings, license reports) to CSV format for easier analysis
in spreadsheet applications and data processing tools.
"""

import argparse
import csv
import json
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional


def export_sbom_to_csv(sbom_data: Dict[str, Any], output_path: str) -> None:
    """Export SPDX SBOM to CSV format.
    
    Args:
        sbom_data: Parsed SPDX JSON document
        output_path: Path to write CSV file
        
    Raises:
        ValueError: If SBOM is missing required fields
        IOError: If unable to write to output file
    """
    if not sbom_data.get('packages'):
        raise ValueError("SBOM missing required 'packages' field")
    
    try:
        with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
            fieldnames = [
                'Name',
                'Version',
                'SPDXID',
                'License',
                'Supplier',
                'Download Location',
                'Package URL (PURL)',
                'Checksum',
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for pkg in sbom_data['packages']:
                # Extract PURL from external references
                purl = ''
                for ref in pkg.get('externalRefs', []):
                    if ref.get('referenceType') == 'purl':
                        purl = ref.get('referenceLocator', '')
                        break
                
                # Extract checksum
                checksum = ''
                checksums = pkg.get('checksums', [])
                if checksums:
                    checksum = f"{checksums[0].get('algorithm', '')}:{checksums[0].get('checksumValue', '')}"
                
                writer.writerow({
                    'Name': pkg.get('name', ''),
                    'Version': pkg.get('versionInfo', ''),
                    'SPDXID': pkg.get('SPDXID', ''),
                    'License': pkg.get('licenseConcluded', 'NOASSERTION'),
                    'Supplier': pkg.get('supplier', 'NOASSERTION'),
                    'Download Location': pkg.get('downloadLocation', 'NOASSERTION'),
                    'Package URL (PURL)': purl,
                    'Checksum': checksum,
                })
    except IOError as e:
        raise IOError(f"Failed to write CSV file {output_path}: {str(e)}")


def export_vulnerabilities_to_csv(findings_data: Dict[str, Any], output_path: str) -> None:
    """Export vulnerability findings to CSV format.
    
    Args:
        findings_data: Parsed vulnerability findings JSON
        output_path: Path to write CSV file
        
    Raises:
        ValueError: If findings data is invalid
        IOError: If unable to write to output file
    """
    if not isinstance(findings_data.get('vulnerabilities'), list):
        raise ValueError("Findings data missing 'vulnerabilities' array")
    
    try:
        with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
            fieldnames = [
                'CVE ID',
                'Package Name',
                'Package Version',
                'Severity',
                'CVSS Score',
                'Summary',
                'Fixed Version',
                'Published Date',
                'Modified Date',
                'References',
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for vuln in findings_data['vulnerabilities']:
                # Extract severity and CVSS
                severity = 'UNKNOWN'
                cvss_score = ''
                
                for severity_entry in vuln.get('database_specific', {}).get('severity', []):
                    if severity_entry.get('type') == 'CVSS_V3':
                        cvss_score = str(severity_entry.get('score', ''))
                        severity = severity_entry.get('severity', 'UNKNOWN')
                        break
                
                # Join references
                references = '; '.join(vuln.get('references', []))
                
                writer.writerow({
                    'CVE ID': vuln.get('id', ''),
                    'Package Name': vuln.get('package_name', ''),
                    'Package Version': vuln.get('package_version', ''),
                    'Severity': severity,
                    'CVSS Score': cvss_score,
                    'Summary': vuln.get('summary', ''),
                    'Fixed Version': ', '.join(vuln.get('fixed_versions', [])),
                    'Published Date': vuln.get('published', ''),
                    'Modified Date': vuln.get('modified', ''),
                    'References': references,
                })
    except IOError as e:
        raise IOError(f"Failed to write CSV file {output_path}: {str(e)}")


def export_license_report_to_csv(license_data: Dict[str, Any], output_path: str) -> None:
    """Export license report to CSV format.
    
    Args:
        license_data: Parsed license report JSON
        output_path: Path to write CSV file
        
    Raises:
        ValueError: If license data is invalid
        IOError: If unable to write to output file
    """
    if not isinstance(license_data.get('packages'), list):
        raise ValueError("License data missing 'packages' array")
    
    try:
        with open(output_path, 'w', newline='', encoding='utf-8') as csvfile:
            fieldnames = [
                'Package Name',
                'Version',
                'License',
                'License Type',
                'Is Copyleft',
                'Is Permissive',
                'Conflicts',
            ]
            writer = csv.DictWriter(csvfile, fieldnames=fieldnames)
            writer.writeheader()
            
            for pkg in license_data['packages']:
                writer.writerow({
                    'Package Name': pkg.get('name', ''),
                    'Version': pkg.get('version', ''),
                    'License': pkg.get('license', 'UNKNOWN'),
                    'License Type': pkg.get('license_type', 'UNKNOWN'),
                    'Is Copyleft': 'Yes' if pkg.get('is_copyleft', False) else 'No',
                    'Is Permissive': 'Yes' if pkg.get('is_permissive', False) else 'No',
                    'Conflicts': ', '.join(pkg.get('conflicts', [])),
                })
    except IOError as e:
        raise IOError(f"Failed to write CSV file {output_path}: {str(e)}")


def main():
    """Main entry point for CSV export utility."""
    parser = argparse.ArgumentParser(
        description='Export BazBOM scan results to CSV format',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Export SBOM to CSV
  %(prog)s --sbom sbom.spdx.json --output sbom.csv
  
  # Export vulnerability findings to CSV
  %(prog)s --findings sca_findings.json --output vulnerabilities.csv
  
  # Export license report to CSV
  %(prog)s --license-report license_report.json --output licenses.csv
        """
    )
    
    parser.add_argument(
        '--sbom',
        type=str,
        help='Path to SPDX SBOM JSON file'
    )
    
    parser.add_argument(
        '--findings',
        type=str,
        help='Path to vulnerability findings JSON file'
    )
    
    parser.add_argument(
        '--license-report',
        type=str,
        help='Path to license report JSON file'
    )
    
    parser.add_argument(
        '--output',
        type=str,
        required=True,
        help='Path to output CSV file'
    )
    
    args = parser.parse_args()
    
    # Validate that at least one input is provided
    if not any([args.sbom, args.findings, args.license_report]):
        print("ERROR: At least one of --sbom, --findings, or --license-report must be provided",
              file=sys.stderr)
        sys.exit(1)
    
    # Validate that only one input is provided
    input_count = sum([bool(args.sbom), bool(args.findings), bool(args.license_report)])
    if input_count > 1:
        print("ERROR: Only one input file type can be specified at a time",
              file=sys.stderr)
        sys.exit(1)
    
    try:
        # Export based on input type
        if args.sbom:
            with open(args.sbom, 'r', encoding='utf-8') as f:
                sbom_data = json.load(f)
            export_sbom_to_csv(sbom_data, args.output)
            print(f"SBOM exported to {args.output}")
            
        elif args.findings:
            with open(args.findings, 'r', encoding='utf-8') as f:
                findings_data = json.load(f)
            export_vulnerabilities_to_csv(findings_data, args.output)
            print(f"Vulnerability findings exported to {args.output}")
            
        elif args.license_report:
            with open(args.license_report, 'r', encoding='utf-8') as f:
                license_data = json.load(f)
            export_license_report_to_csv(license_data, args.output)
            print(f"License report exported to {args.output}")
            
    except FileNotFoundError as e:
        print(f"ERROR: Input file not found: {e.filename}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON in input file at line {e.lineno}: {e.msg}",
              file=sys.stderr)
        sys.exit(2)
    except ValueError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(2)
    except IOError as e:
        print(f"ERROR: {str(e)}", file=sys.stderr)
        sys.exit(2)
    except Exception as e:
        print(f"ERROR: Unexpected error: {str(e)}", file=sys.stderr)
        sys.exit(2)


if __name__ == '__main__':
    main()
