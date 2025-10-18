#!/usr/bin/env python3
"""Generate shields.io compatible badges for BazBOM scan results.

This module generates badge data that can be used with shields.io's
dynamic JSON endpoint to create security status badges for README files.
"""

import argparse
import json
import sys
from typing import Any, Dict, Optional


def calculate_badge_data(
    findings_data: Dict[str, Any],
    license_data: Optional[Dict[str, Any]] = None
) -> Dict[str, Any]:
    """Calculate badge data from scan results.
    
    Args:
        findings_data: Vulnerability findings JSON
        license_data: Optional license report JSON
        
    Returns:
        Dictionary containing badge configuration data
        
    Raises:
        ValueError: If findings data is invalid
    """
    if not isinstance(findings_data.get('vulnerabilities'), list):
        raise ValueError("Findings data missing 'vulnerabilities' array")
    
    vulnerabilities = findings_data['vulnerabilities']
    
    # Count vulnerabilities by severity
    critical_count = 0
    high_count = 0
    medium_count = 0
    low_count = 0
    
    for vuln in vulnerabilities:
        severity = 'UNKNOWN'
        for severity_entry in vuln.get('database_specific', {}).get('severity', []):
            if severity_entry.get('type') == 'CVSS_V3':
                severity = severity_entry.get('severity', 'UNKNOWN')
                break
        
        if severity == 'CRITICAL':
            critical_count += 1
        elif severity == 'HIGH':
            high_count += 1
        elif severity == 'MEDIUM':
            medium_count += 1
        elif severity == 'LOW':
            low_count += 1
    
    total_vulns = len(vulnerabilities)
    
    # Determine overall security status and color
    if critical_count > 0:
        status = f"{critical_count} critical"
        color = "critical"  # red
    elif high_count > 0:
        status = f"{high_count} high"
        color = "important"  # orange
    elif medium_count > 0:
        status = f"{medium_count} medium"
        color = "yellow"
    elif low_count > 0:
        status = f"{low_count} low"
        color = "informational"  # blue
    else:
        status = "no known vulnerabilities"
        color = "success"  # green
    
    badge_data = {
        "schemaVersion": 1,
        "label": "security",
        "message": status,
        "color": color,
        "namedLogo": "github",
    }
    
    # Add license information if available
    if license_data:
        packages = license_data.get('packages', [])
        copyleft_count = sum(1 for pkg in packages if pkg.get('is_copyleft', False))
        
        if copyleft_count > 0:
            badge_data['labelColor'] = "orange"
    
    return badge_data


def generate_shields_json(
    findings_data: Dict[str, Any],
    license_data: Optional[Dict[str, Any]] = None
) -> str:
    """Generate shields.io endpoint JSON.
    
    Args:
        findings_data: Vulnerability findings JSON
        license_data: Optional license report JSON
        
    Returns:
        JSON string for shields.io dynamic badge endpoint
    """
    badge_data = calculate_badge_data(findings_data, license_data)
    return json.dumps(badge_data, indent=2)


def generate_markdown_badge(
    badge_url: str,
    alt_text: str = "Security Status"
) -> str:
    """Generate Markdown badge snippet.
    
    Args:
        badge_url: URL to the badge JSON endpoint
        alt_text: Alt text for the badge image
        
    Returns:
        Markdown badge snippet
    """
    # Use shields.io dynamic JSON endpoint
    shields_url = f"https://img.shields.io/endpoint?url={badge_url}"
    return f"![{alt_text}]({shields_url})"


def generate_html_badge(
    badge_url: str,
    alt_text: str = "Security Status"
) -> str:
    """Generate HTML badge snippet.
    
    Args:
        badge_url: URL to the badge JSON endpoint
        alt_text: Alt text for the badge image
        
    Returns:
        HTML badge snippet
    """
    shields_url = f"https://img.shields.io/endpoint?url={badge_url}"
    return f'<img src="{shields_url}" alt="{alt_text}" />'


def main():
    """Main entry point for badge generator."""
    parser = argparse.ArgumentParser(
        description='Generate security badges for BazBOM scan results',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate badge JSON for shields.io endpoint
  %(prog)s --findings sca_findings.json --output badge.json
  
  # Generate badge with license information
  %(prog)s --findings sca_findings.json --license-report licenses.json --output badge.json
  
  # Generate Markdown badge snippet
  %(prog)s --findings sca_findings.json --output badge.json --format markdown --badge-url https://example.com/badge.json
        """
    )
    
    parser.add_argument(
        '--findings',
        type=str,
        required=True,
        help='Path to vulnerability findings JSON file'
    )
    
    parser.add_argument(
        '--license-report',
        type=str,
        help='Path to license report JSON file (optional)'
    )
    
    parser.add_argument(
        '--output',
        type=str,
        required=True,
        help='Path to output file'
    )
    
    parser.add_argument(
        '--format',
        type=str,
        choices=['json', 'markdown', 'html'],
        default='json',
        help='Output format (default: json)'
    )
    
    parser.add_argument(
        '--badge-url',
        type=str,
        help='Badge JSON endpoint URL (required for markdown/html formats)'
    )
    
    parser.add_argument(
        '--alt-text',
        type=str,
        default='Security Status',
        help='Alt text for badge image (for markdown/html formats)'
    )
    
    args = parser.parse_args()
    
    # Validate markdown/html format requirements
    if args.format in ['markdown', 'html'] and not args.badge_url:
        print("ERROR: --badge-url is required for markdown and html formats",
              file=sys.stderr)
        sys.exit(1)
    
    try:
        # Load findings data
        with open(args.findings, 'r', encoding='utf-8') as f:
            findings_data = json.load(f)
        
        # Load license data if provided
        license_data = None
        if args.license_report:
            with open(args.license_report, 'r', encoding='utf-8') as f:
                license_data = json.load(f)
        
        # Generate output based on format
        if args.format == 'json':
            output = generate_shields_json(findings_data, license_data)
        elif args.format == 'markdown':
            output = generate_markdown_badge(args.badge_url, args.alt_text)
        elif args.format == 'html':
            output = generate_html_badge(args.badge_url, args.alt_text)
        
        # Write output
        with open(args.output, 'w', encoding='utf-8') as f:
            f.write(output)
            if args.format == 'json':
                f.write('\n')  # Add newline for JSON files
        
        print(f"Badge generated: {args.output}")
        
        # Print preview for markdown/html
        if args.format in ['markdown', 'html']:
            print(f"\nPreview:\n{output}")
        
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
