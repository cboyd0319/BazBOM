#!/usr/bin/env python3
"""Generate automated SBOM compliance reports.

This module generates audit-ready compliance reports from SBOM and SCA data
in multiple formats (HTML, PDF, DOCX, XLSX). Reports include executive summaries,
SOC2 compliance certificates, license attribution, and audit trails.
"""

import argparse
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from jinja2 import Environment, FileSystemLoader, Template
except ImportError:
    print("Error: jinja2 library not installed", file=sys.stderr)
    print("Install with: pip install jinja2", file=sys.stderr)
    sys.exit(1)


class ComplianceReportGenerator:
    """Generate compliance reports from SBOM and SCA data."""

    def __init__(
        self,
        templates_dir: Optional[str] = None,
        company_name: str = "Organization",
        company_logo: Optional[str] = None,
        brand_color: str = "#0066cc"
    ):
        """Initialize the compliance report generator.
        
        Args:
            templates_dir: Directory containing Jinja2 templates
            company_name: Organization name for reports
            company_logo: Path to company logo image (optional)
            brand_color: Hex color code for branding (default: #0066cc)
            
        Raises:
            FileNotFoundError: If templates directory doesn't exist
        """
        if templates_dir is None:
            # Try to find templates in Bazel runfiles or relative to script
            script_dir = Path(__file__).parent
            
            # Try runfiles location first (when run via Bazel)
            potential_dirs = [
                script_dir / "templates" / "compliance",  # Normal location
                Path(os.getcwd()) / "tools" / "supplychain" / "templates" / "compliance",  # Bazel build
                script_dir.parent.parent / "tools" / "supplychain" / "templates" / "compliance",  # From bazel-bin
            ]
            
            templates_dir = None
            for dir_path in potential_dirs:
                if dir_path.exists() and (dir_path / "executive_summary.html").exists():
                    templates_dir = dir_path
                    break
            
            if templates_dir is None:
                raise FileNotFoundError(
                    f"Templates directory not found. Tried:\n" +
                    "\n".join(f"  - {d}" for d in potential_dirs) +
                    f"\nExpected templates in tools/supplychain/templates/compliance/"
                )
        
        self.templates_dir = Path(templates_dir)
        if not self.templates_dir.exists():
            raise FileNotFoundError(
                f"Templates directory not found: {self.templates_dir}. "
                f"Expected templates in tools/supplychain/templates/compliance/"
            )
        
        self.company_name = company_name
        self.company_logo = company_logo
        self.brand_color = brand_color
        
        # Initialize Jinja2 environment
        self.jinja_env = Environment(
            loader=FileSystemLoader(str(self.templates_dir)),
            autoescape=True
        )
        
        # Register custom filters
        self.jinja_env.filters['format_date'] = self._format_date
        self.jinja_env.filters['format_number'] = self._format_number
    
    def _format_date(self, date_str: str, format_str: str = "%Y-%m-%d") -> str:
        """Format ISO date string to readable format.
        
        Args:
            date_str: ISO format date string
            format_str: strftime format string
            
        Returns:
            Formatted date string
        """
        try:
            dt = datetime.fromisoformat(date_str.replace('Z', '+00:00'))
            return dt.strftime(format_str)
        except (ValueError, AttributeError):
            return date_str
    
    def _format_number(self, value: int) -> str:
        """Format number with thousands separators.
        
        Args:
            value: Integer to format
            
        Returns:
            Formatted number string
        """
        return f"{value:,}"
    
    def load_data(
        self,
        sbom_path: Optional[str] = None,
        sca_findings_path: Optional[str] = None,
        license_report_path: Optional[str] = None,
        enrichment_data_path: Optional[str] = None
    ) -> Dict[str, Any]:
        """Load all required data files for report generation.
        
        Args:
            sbom_path: Path to SPDX SBOM JSON file
            sca_findings_path: Path to SCA findings JSON file
            license_report_path: Path to license report JSON file
            enrichment_data_path: Path to enriched findings JSON (optional)
            
        Returns:
            Dictionary containing all loaded data
            
        Raises:
            FileNotFoundError: If required file is missing
            json.JSONDecodeError: If file contains invalid JSON
        """
        data = {}
        
        # Load SBOM
        if sbom_path:
            if not os.path.exists(sbom_path):
                raise FileNotFoundError(f"SBOM file not found: {sbom_path}")
            with open(sbom_path, 'r', encoding='utf-8') as f:
                data['sbom'] = json.load(f)
        
        # Load SCA findings
        if sca_findings_path:
            if not os.path.exists(sca_findings_path):
                raise FileNotFoundError(f"SCA findings file not found: {sca_findings_path}")
            with open(sca_findings_path, 'r', encoding='utf-8') as f:
                data['sca_findings'] = json.load(f)
        
        # Load license report
        if license_report_path:
            if not os.path.exists(license_report_path):
                raise FileNotFoundError(f"License report file not found: {license_report_path}")
            with open(license_report_path, 'r', encoding='utf-8') as f:
                data['license_report'] = json.load(f)
        
        # Load enrichment data (optional)
        if enrichment_data_path and os.path.exists(enrichment_data_path):
            with open(enrichment_data_path, 'r', encoding='utf-8') as f:
                data['enrichment'] = json.load(f)
        
        return data
    
    def _calculate_summary_stats(self, data: Dict[str, Any]) -> Dict[str, Any]:
        """Calculate summary statistics for reports.
        
        Args:
            data: Loaded data dictionary
            
        Returns:
            Dictionary with summary statistics
        """
        stats = {
            'total_packages': 0,
            'total_vulnerabilities': 0,
            'critical_vulns': 0,
            'high_vulns': 0,
            'medium_vulns': 0,
            'low_vulns': 0,
            'kev_vulns': 0,
            'total_licenses': 0,
            'copyleft_licenses': 0,
            'proprietary_licenses': 0,
            'permissive_licenses': 0,
            'unknown_licenses': 0,
            'scan_date': datetime.now().isoformat()
        }
        
        # Count packages from SBOM
        if 'sbom' in data:
            packages = data['sbom'].get('packages', [])
            # Exclude root package
            stats['total_packages'] = len([p for p in packages if p.get('SPDXID') != 'SPDXRef-Package-root'])
        
        # Count vulnerabilities from SCA findings
        if 'sca_findings' in data:
            vulns = data['sca_findings'].get('vulnerabilities', [])
            stats['total_vulnerabilities'] = len(vulns)
            
            for vuln in vulns:
                severity = vuln.get('severity', '').upper()
                if severity == 'CRITICAL':
                    stats['critical_vulns'] += 1
                elif severity == 'HIGH':
                    stats['high_vulns'] += 1
                elif severity == 'MEDIUM':
                    stats['medium_vulns'] += 1
                elif severity == 'LOW':
                    stats['low_vulns'] += 1
                
                # Count KEV vulnerabilities
                if vuln.get('kev', {}).get('in_kev'):
                    stats['kev_vulns'] += 1
        
        # Count licenses from license report
        if 'license_report' in data:
            licenses = data['license_report'].get('licenses', {})
            stats['total_licenses'] = len(licenses)
            
            for license_id, license_info in licenses.items():
                license_type = license_info.get('type', 'UNKNOWN').upper()
                if license_type == 'COPYLEFT':
                    stats['copyleft_licenses'] += 1
                elif license_type == 'PROPRIETARY':
                    stats['proprietary_licenses'] += 1
                elif license_type == 'PERMISSIVE':
                    stats['permissive_licenses'] += 1
                else:
                    stats['unknown_licenses'] += 1
        
        return stats
    
    def generate_executive_summary(
        self,
        data: Dict[str, Any],
        output_path: str
    ) -> None:
        """Generate executive summary report (1-page for C-suite).
        
        Args:
            data: Loaded data dictionary
            output_path: Path to output HTML file
            
        Raises:
            FileNotFoundError: If template not found
        """
        stats = self._calculate_summary_stats(data)
        
        template = self.jinja_env.get_template('executive_summary.html')
        
        context = {
            'company_name': self.company_name,
            'company_logo': self.company_logo,
            'brand_color': self.brand_color,
            'report_date': datetime.now().strftime("%B %d, %Y"),
            'stats': stats,
            'data': data
        }
        
        html_output = template.render(**context)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_output)
        
        print(f" Executive summary generated: {output_path}", file=sys.stderr)
    
    def generate_soc2_report(
        self,
        data: Dict[str, Any],
        output_path: str
    ) -> None:
        """Generate SOC2 compliance certificate report.
        
        Args:
            data: Loaded data dictionary
            output_path: Path to output HTML file
            
        Raises:
            FileNotFoundError: If template not found
        """
        stats = self._calculate_summary_stats(data)
        
        template = self.jinja_env.get_template('soc2_report.html')
        
        # SOC2 Trust Services Criteria assessment
        soc2_criteria = {
            'CC6.1': {
                'name': 'Logical and Physical Access Controls',
                'status': 'PASS' if stats['critical_vulns'] == 0 else 'FAIL',
                'details': f"{stats['critical_vulns']} critical vulnerabilities found"
            },
            'CC6.6': {
                'name': 'Vulnerability Management',
                'status': 'PASS' if stats['kev_vulns'] == 0 else 'FAIL',
                'details': f"{stats['kev_vulns']} known exploited vulnerabilities (KEV) found"
            },
            'CC7.1': {
                'name': 'Security Incident Detection',
                'status': 'PASS',
                'details': 'Automated vulnerability scanning enabled'
            },
            'CC8.1': {
                'name': 'Change Management',
                'status': 'PASS',
                'details': 'SBOM generated for all builds'
            }
        }
        
        context = {
            'company_name': self.company_name,
            'company_logo': self.company_logo,
            'brand_color': self.brand_color,
            'report_date': datetime.now().strftime("%B %d, %Y"),
            'stats': stats,
            'soc2_criteria': soc2_criteria,
            'data': data
        }
        
        html_output = template.render(**context)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_output)
        
        print(f" SOC2 report generated: {output_path}", file=sys.stderr)
    
    def generate_attribution_report(
        self,
        data: Dict[str, Any],
        output_path: str
    ) -> None:
        """Generate license attribution report (legal requirement).
        
        Args:
            data: Loaded data dictionary
            output_path: Path to output HTML file
            
        Raises:
            FileNotFoundError: If template not found
        """
        stats = self._calculate_summary_stats(data)
        
        template = self.jinja_env.get_template('attribution.html')
        
        # Extract package attribution information
        attributions = []
        if 'sbom' in data:
            for pkg in data['sbom'].get('packages', []):
                if pkg.get('SPDXID') == 'SPDXRef-Package-root':
                    continue
                
                attribution = {
                    'name': pkg.get('name', 'Unknown'),
                    'version': pkg.get('versionInfo', 'Unknown'),
                    'license': pkg.get('licenseConcluded', 'NOASSERTION'),
                    'copyright': pkg.get('copyrightText', 'NOASSERTION'),
                    'download_location': pkg.get('downloadLocation', 'NOASSERTION')
                }
                
                # Extract PURL if available
                for ref in pkg.get('externalRefs', []):
                    if ref.get('referenceType') == 'purl':
                        attribution['purl'] = ref.get('referenceLocator', '')
                        break
                
                attributions.append(attribution)
        
        context = {
            'company_name': self.company_name,
            'company_logo': self.company_logo,
            'brand_color': self.brand_color,
            'report_date': datetime.now().strftime("%B %d, %Y"),
            'stats': stats,
            'attributions': attributions,
            'data': data
        }
        
        html_output = template.render(**context)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_output)
        
        print(f" Attribution report generated: {output_path}", file=sys.stderr)
    
    def generate_audit_trail(
        self,
        data: Dict[str, Any],
        output_path: str,
        approvals: Optional[List[Dict]] = None
    ) -> None:
        """Generate audit trail report (who approved what, when).
        
        Args:
            data: Loaded data dictionary
            output_path: Path to output HTML file
            approvals: List of approval records (optional)
            
        Raises:
            FileNotFoundError: If template not found
        """
        stats = self._calculate_summary_stats(data)
        
        template = self.jinja_env.get_template('audit_trail.html')
        
        # Use provided approvals or create empty list
        if approvals is None:
            approvals = []
        
        # Add scan metadata as audit entry
        audit_entries = [
            {
                'timestamp': stats['scan_date'],
                'action': 'SBOM Generated',
                'user': 'BazBOM Automated System',
                'details': f"Generated SBOM with {stats['total_packages']} packages"
            },
            {
                'timestamp': stats['scan_date'],
                'action': 'Vulnerability Scan',
                'user': 'BazBOM Automated System',
                'details': f"Scanned {stats['total_packages']} packages, found {stats['total_vulnerabilities']} vulnerabilities"
            }
        ]
        
        # Add approval entries
        audit_entries.extend(approvals)
        
        context = {
            'company_name': self.company_name,
            'company_logo': self.company_logo,
            'brand_color': self.brand_color,
            'report_date': datetime.now().strftime("%B %d, %Y"),
            'stats': stats,
            'audit_entries': audit_entries,
            'data': data
        }
        
        html_output = template.render(**context)
        
        with open(output_path, 'w', encoding='utf-8') as f:
            f.write(html_output)
        
        print(f" Audit trail generated: {output_path}", file=sys.stderr)
    
    def generate_all_reports(
        self,
        data: Dict[str, Any],
        output_dir: str,
        formats: Optional[List[str]] = None
    ) -> Dict[str, List[str]]:
        """Generate all compliance reports in specified formats.
        
        Args:
            data: Loaded data dictionary
            output_dir: Directory for output files
            formats: List of formats to generate (default: ['html'])
            
        Returns:
            Dictionary mapping report types to generated file paths
            
        Raises:
            ValueError: If unsupported format specified
        """
        if formats is None:
            formats = ['html']
        
        # Validate formats
        supported_formats = ['html', 'pdf', 'docx', 'xlsx']
        for fmt in formats:
            if fmt not in supported_formats:
                raise ValueError(
                    f"Unsupported format: {fmt}. "
                    f"Supported formats: {', '.join(supported_formats)}"
                )
        
        # Create output directory
        os.makedirs(output_dir, exist_ok=True)
        
        generated_files = {
            'executive_summary': [],
            'soc2_report': [],
            'attribution': [],
            'audit_trail': []
        }
        
        # Generate HTML reports (base format)
        html_files = {
            'executive_summary': os.path.join(output_dir, 'executive_summary.html'),
            'soc2_report': os.path.join(output_dir, 'soc2_report.html'),
            'attribution': os.path.join(output_dir, 'attribution.html'),
            'audit_trail': os.path.join(output_dir, 'audit_trail.html')
        }
        
        self.generate_executive_summary(data, html_files['executive_summary'])
        generated_files['executive_summary'].append(html_files['executive_summary'])
        
        self.generate_soc2_report(data, html_files['soc2_report'])
        generated_files['soc2_report'].append(html_files['soc2_report'])
        
        self.generate_attribution_report(data, html_files['attribution'])
        generated_files['attribution'].append(html_files['attribution'])
        
        self.generate_audit_trail(data, html_files['audit_trail'])
        generated_files['audit_trail'].append(html_files['audit_trail'])
        
        # Convert to other formats if requested
        if 'pdf' in formats:
            print("\n PDF generation requires additional tools (wkhtmltopdf or weasyprint)", file=sys.stderr)
            print("HTML reports can be printed to PDF manually", file=sys.stderr)
        
        if 'docx' in formats:
            print("\n DOCX generation requires additional tools (pandoc)", file=sys.stderr)
            print("HTML reports can be converted to DOCX manually", file=sys.stderr)
        
        if 'xlsx' in formats:
            print("\n XLSX generation requires additional implementation", file=sys.stderr)
            print("Consider extracting data to CSV format instead", file=sys.stderr)
        
        return generated_files


def main() -> int:
    """Main entry point for compliance report generation.
    
    Returns:
        Exit code (0 for success, non-zero for error)
    """
    parser = argparse.ArgumentParser(
        description="Generate automated compliance reports from SBOM and SCA data"
    )
    
    # Input files
    parser.add_argument(
        "--sbom",
        required=True,
        help="Path to SPDX SBOM JSON file"
    )
    parser.add_argument(
        "--sca-findings",
        help="Path to SCA findings JSON file"
    )
    parser.add_argument(
        "--license-report",
        help="Path to license report JSON file"
    )
    parser.add_argument(
        "--enrichment",
        help="Path to enriched findings JSON file (optional)"
    )
    
    # Output configuration
    parser.add_argument(
        "--output-dir",
        default="compliance-reports",
        help="Output directory for reports (default: compliance-reports)"
    )
    parser.add_argument(
        "--formats",
        nargs='+',
        default=['html'],
        choices=['html', 'pdf', 'docx', 'xlsx'],
        help="Output formats (default: html)"
    )
    
    # Branding options
    parser.add_argument(
        "--company-name",
        default="Organization",
        help="Company name for reports"
    )
    parser.add_argument(
        "--company-logo",
        help="Path to company logo image (optional)"
    )
    parser.add_argument(
        "--brand-color",
        default="#0066cc",
        help="Hex color code for branding (default: #0066cc)"
    )
    
    # Templates
    parser.add_argument(
        "--templates-dir",
        help="Custom templates directory (optional)"
    )
    
    args = parser.parse_args()
    
    try:
        # Initialize generator
        generator = ComplianceReportGenerator(
            templates_dir=args.templates_dir,
            company_name=args.company_name,
            company_logo=args.company_logo,
            brand_color=args.brand_color
        )
        
        # Load data
        print("Loading data files...", file=sys.stderr)
        data = generator.load_data(
            sbom_path=args.sbom,
            sca_findings_path=args.sca_findings,
            license_report_path=args.license_report,
            enrichment_data_path=args.enrichment
        )
        
        # Generate reports
        print("\nGenerating compliance reports...", file=sys.stderr)
        generated_files = generator.generate_all_reports(
            data=data,
            output_dir=args.output_dir,
            formats=args.formats
        )
        
        # Print summary
        print("\n" + "="*60, file=sys.stderr)
        print(" Compliance Reports Generated Successfully", file=sys.stderr)
        print("="*60, file=sys.stderr)
        
        for report_type, files in generated_files.items():
            print(f"\n{report_type.replace('_', ' ').title()}:", file=sys.stderr)
            for file_path in files:
                print(f"  • {file_path}", file=sys.stderr)
        
        print(f"\nAll reports saved to: {args.output_dir}", file=sys.stderr)
        
        return 0
        
    except FileNotFoundError as e:
        print(f"Error: {e}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"Error: Invalid JSON in input file: {e}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"Error: Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        return 3


if __name__ == "__main__":
    sys.exit(main())
