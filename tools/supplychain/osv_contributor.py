#!/usr/bin/env python3
"""OSV Contributor - Generate OSV-format vulnerability reports.

This module helps teams contribute vulnerability findings back to the
Open Source Vulnerabilities (OSV) database by auto-generating properly
formatted OSV YAML files.
"""

import argparse
import json
import sys
import yaml
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional


class OSVContributor:
    """Generate OSV-format vulnerability reports for contribution."""
    
    # OSV schema version
    OSV_SCHEMA_VERSION = "1.6.0"
    
    def __init__(self):
        """Initialize OSV contributor."""
        pass
    
    def generate_osv_entry(
        self,
        vulnerability_id: str,
        package_name: str,
        package_ecosystem: str,
        affected_versions: List[str],
        fixed_version: Optional[str] = None,
        summary: str = "",
        details: str = "",
        severity: Optional[str] = None,
        cvss_score: Optional[float] = None,
        references: Optional[List[str]] = None,
        aliases: Optional[List[str]] = None,
        database_specific: Optional[Dict[str, Any]] = None
    ) -> Dict[str, Any]:
        """Generate an OSV entry.
        
        Args:
            vulnerability_id: Vulnerability ID (e.g., GHSA-xxxx-xxxx-xxxx or CVE-2023-1234)
            package_name: Package name (e.g., com.example:mylib)
            package_ecosystem: Ecosystem (Maven, npm, PyPI, etc.)
            affected_versions: List of affected version ranges
            fixed_version: Version where vulnerability was fixed (optional)
            summary: Brief vulnerability summary
            details: Detailed vulnerability description
            severity: Severity level (CRITICAL, HIGH, MEDIUM, LOW)
            cvss_score: CVSS score (0.0-10.0)
            references: List of reference URLs
            aliases: List of alternate IDs (CVEs, GHSAs)
            database_specific: Additional database-specific metadata
            
        Returns:
            OSV entry as dictionary
            
        Raises:
            ValueError: If required fields are missing or invalid
        """
        if not vulnerability_id:
            raise ValueError("vulnerability_id is required")
        if not package_name:
            raise ValueError("package_name is required")
        if not package_ecosystem:
            raise ValueError("package_ecosystem is required")
        if not affected_versions:
            raise ValueError("At least one affected version is required")
        
        # Build OSV entry
        osv_entry = {
            "id": vulnerability_id,
            "schema_version": self.OSV_SCHEMA_VERSION,
            "modified": datetime.now().isoformat() + "Z",
        }
        
        # Add summary and details
        if summary:
            osv_entry["summary"] = summary
        if details:
            osv_entry["details"] = details
        
        # Add aliases (CVEs, GHSAs, etc.)
        if aliases:
            osv_entry["aliases"] = sorted(set(aliases))
        
        # Add affected packages
        osv_entry["affected"] = self._build_affected_section(
            package_name,
            package_ecosystem,
            affected_versions,
            fixed_version
        )
        
        # Add severity
        if severity or cvss_score:
            osv_entry["severity"] = self._build_severity_section(severity, cvss_score)
        
        # Add references
        if references:
            osv_entry["references"] = self._build_references_section(references)
        
        # Add database-specific metadata
        if database_specific:
            osv_entry["database_specific"] = database_specific
        
        return osv_entry
    
    def _build_affected_section(
        self,
        package_name: str,
        ecosystem: str,
        affected_versions: List[str],
        fixed_version: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Build the 'affected' section of OSV entry.
        
        Args:
            package_name: Package name
            ecosystem: Package ecosystem
            affected_versions: List of affected version ranges
            fixed_version: Fixed version (optional)
            
        Returns:
            List of affected package entries
        """
        affected = [{
            "package": {
                "name": package_name,
                "ecosystem": ecosystem,
            },
            "ranges": [
                {
                    "type": "ECOSYSTEM",
                    "events": self._build_version_events(affected_versions, fixed_version)
                }
            ]
        }]
        
        # Add versions list if specific versions are affected
        if affected_versions and not any('*' in v or '>' in v or '<' in v for v in affected_versions):
            affected[0]["versions"] = sorted(affected_versions)
        
        return affected
    
    def _build_version_events(
        self,
        affected_versions: List[str],
        fixed_version: Optional[str] = None
    ) -> List[Dict[str, str]]:
        """Build version events for OSV ranges.
        
        Args:
            affected_versions: List of affected version ranges
            fixed_version: Fixed version (optional)
            
        Returns:
            List of version events
        """
        events = []
        
        # Parse affected version ranges
        for version_range in affected_versions:
            if version_range.startswith('>='):
                # Introduced in this version
                version = version_range[2:].strip()
                events.append({"introduced": version})
            elif version_range.startswith('<='):
                # Last affected version
                version = version_range[2:].strip()
                # OSV uses "fixed" for the version after the last affected
                # So we can't directly map <=X without knowing X+1
                pass
            elif version_range == '*':
                # All versions affected
                events.append({"introduced": "0"})
            else:
                # Specific version
                events.append({"introduced": version_range})
        
        # Add fixed version
        if fixed_version:
            events.append({"fixed": fixed_version})
        
        # If no introduced event, add "0" (all versions from beginning)
        if not any('introduced' in e for e in events):
            events.insert(0, {"introduced": "0"})
        
        return events
    
    def _build_severity_section(
        self,
        severity: Optional[str] = None,
        cvss_score: Optional[float] = None
    ) -> List[Dict[str, Any]]:
        """Build severity section.
        
        Args:
            severity: Severity level (CRITICAL, HIGH, MEDIUM, LOW)
            cvss_score: CVSS score (0.0-10.0)
            
        Returns:
            List of severity entries
        """
        severity_entries = []
        
        if severity:
            severity_entries.append({
                "type": "CVSS_V3",
                "score": severity
            })
        
        if cvss_score is not None:
            # Map score to CVSS vector (simplified)
            if cvss_score >= 9.0:
                vector = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:C/C:H/I:H/A:H"
            elif cvss_score >= 7.0:
                vector = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"
            elif cvss_score >= 4.0:
                vector = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:L/I:L/A:L"
            else:
                vector = "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:N/I:N/A:L"
            
            severity_entries.append({
                "type": "CVSS_V3",
                "score": vector
            })
        
        return severity_entries
    
    def _build_references_section(self, references: List[str]) -> List[Dict[str, str]]:
        """Build references section.
        
        Args:
            references: List of reference URLs
            
        Returns:
            List of reference entries
        """
        ref_entries = []
        
        for ref_url in references:
            ref_type = "WEB"  # Default
            
            # Classify reference type based on URL
            if "github.com" in ref_url and "/advisories/" in ref_url:
                ref_type = "ADVISORY"
            elif "nvd.nist.gov" in ref_url or "cve.org" in ref_url:
                ref_type = "ADVISORY"
            elif "github.com" in ref_url and "/issues/" in ref_url:
                ref_type = "REPORT"
            elif "github.com" in ref_url and "/commit/" in ref_url:
                ref_type = "FIX"
            
            ref_entries.append({
                "type": ref_type,
                "url": ref_url
            })
        
        return ref_entries
    
    def to_yaml(self, osv_entry: Dict[str, Any]) -> str:
        """Convert OSV entry to YAML format.
        
        Args:
            osv_entry: OSV entry dictionary
            
        Returns:
            YAML-formatted string
        """
        return yaml.dump(
            osv_entry,
            default_flow_style=False,
            sort_keys=False,
            allow_unicode=True
        )
    
    def save_to_file(self, osv_entry: Dict[str, Any], output_path: str) -> None:
        """Save OSV entry to YAML file.
        
        Args:
            osv_entry: OSV entry dictionary
            output_path: Path to output file
            
        Raises:
            IOError: If file cannot be written
        """
        try:
            with open(output_path, 'w', encoding='utf-8') as f:
                yaml.dump(
                    osv_entry,
                    f,
                    default_flow_style=False,
                    sort_keys=False,
                    allow_unicode=True
                )
        except IOError as e:
            raise IOError(f"Failed to write OSV file to {output_path}: {e}")
    
    def from_finding(self, finding: Dict[str, Any]) -> Dict[str, Any]:
        """Convert a vulnerability finding to OSV format.
        
        Args:
            finding: Vulnerability finding (from osv_query.py or similar)
            
        Returns:
            OSV entry dictionary
            
        Raises:
            ValueError: If finding is missing required fields
        """
        # Extract required fields
        vuln_id = finding.get('id') or finding.get('cve') or finding.get('vulnerability_id')
        if not vuln_id:
            raise ValueError("Finding missing vulnerability ID (id, cve, or vulnerability_id)")
        
        package_info = finding.get('package', {})
        package_name = package_info.get('name') or finding.get('package_name')
        if not package_name:
            raise ValueError("Finding missing package name")
        
        ecosystem = package_info.get('ecosystem') or finding.get('ecosystem', 'Maven')
        
        # Extract affected versions
        affected_versions = finding.get('affected_versions', [])
        if not affected_versions and 'version' in finding:
            affected_versions = [finding['version']]
        
        # Extract fixed version
        fixed_version = finding.get('fixed_version') or finding.get('patched_version')
        
        # Extract other fields
        summary = finding.get('summary', '')
        details = finding.get('details', '')
        severity = finding.get('severity')
        cvss_score = finding.get('cvss_score')
        
        # Extract references
        references = finding.get('references', [])
        if 'url' in finding and finding['url']:
            references.append(finding['url'])
        
        # Extract aliases
        aliases = finding.get('aliases', [])
        
        # Database-specific metadata
        database_specific = {}
        if 'source' in finding:
            database_specific['source'] = finding['source']
        if 'published' in finding:
            database_specific['published'] = finding['published']
        
        return self.generate_osv_entry(
            vulnerability_id=vuln_id,
            package_name=package_name,
            package_ecosystem=ecosystem,
            affected_versions=affected_versions,
            fixed_version=fixed_version,
            summary=summary,
            details=details,
            severity=severity,
            cvss_score=cvss_score,
            references=references,
            aliases=aliases,
            database_specific=database_specific if database_specific else None
        )


def main() -> int:
    """Main entry point for CLI.
    
    Returns:
        Exit code (0 for success, non-zero for errors)
    """
    parser = argparse.ArgumentParser(
        description='Generate OSV-format vulnerability reports',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Generate OSV entry from scratch
  %(prog)s --id CVE-2023-1234 \\
    --package com.example:mylib \\
    --ecosystem Maven \\
    --affected ">=1.0.0" \\
    --fixed 1.5.0 \\
    --summary "SQL injection vulnerability" \\
    -o osv-entry.yaml
  
  # Convert finding from JSON
  %(prog)s --from-finding finding.json -o osv-entry.yaml
  
  # Generate multiple entries from findings file
  %(prog)s --from-findings findings.json --output-dir osv_entries/
"""
    )
    
    parser.add_argument(
        '--id',
        help='Vulnerability ID (CVE, GHSA, etc.)'
    )
    
    parser.add_argument(
        '--package',
        help='Package name (e.g., com.example:mylib)'
    )
    
    parser.add_argument(
        '--ecosystem',
        help='Package ecosystem (Maven, npm, PyPI, etc.)'
    )
    
    parser.add_argument(
        '--affected',
        action='append',
        help='Affected version range (can be specified multiple times)'
    )
    
    parser.add_argument(
        '--fixed',
        help='Version where vulnerability was fixed'
    )
    
    parser.add_argument(
        '--summary',
        help='Brief vulnerability summary'
    )
    
    parser.add_argument(
        '--details',
        help='Detailed vulnerability description'
    )
    
    parser.add_argument(
        '--severity',
        choices=['CRITICAL', 'HIGH', 'MEDIUM', 'LOW'],
        help='Severity level'
    )
    
    parser.add_argument(
        '--cvss',
        type=float,
        help='CVSS score (0.0-10.0)'
    )
    
    parser.add_argument(
        '--reference',
        action='append',
        help='Reference URL (can be specified multiple times)'
    )
    
    parser.add_argument(
        '--alias',
        action='append',
        help='Alternate ID (can be specified multiple times)'
    )
    
    parser.add_argument(
        '--from-finding',
        help='Path to JSON file with vulnerability finding'
    )
    
    parser.add_argument(
        '--from-findings',
        help='Path to JSON file with multiple findings'
    )
    
    parser.add_argument(
        '-o', '--output',
        help='Output file path (default: stdout)'
    )
    
    parser.add_argument(
        '--output-dir',
        help='Output directory for multiple entries'
    )
    
    args = parser.parse_args()
    
    contributor = OSVContributor()
    
    try:
        # Mode 1: Convert from finding file
        if args.from_finding:
            print(f"Loading finding from: {args.from_finding}", file=sys.stderr)
            with open(args.from_finding, 'r', encoding='utf-8') as f:
                finding = json.load(f)
            
            osv_entry = contributor.from_finding(finding)
            
            if args.output:
                contributor.save_to_file(osv_entry, args.output)
                print(f"Saved OSV entry to: {args.output}", file=sys.stderr)
            else:
                print(contributor.to_yaml(osv_entry))
        
        # Mode 2: Convert from findings file (multiple)
        elif args.from_findings:
            if not args.output_dir:
                print("ERROR: --output-dir required when using --from-findings", file=sys.stderr)
                return 1
            
            Path(args.output_dir).mkdir(parents=True, exist_ok=True)
            
            print(f"Loading findings from: {args.from_findings}", file=sys.stderr)
            with open(args.from_findings, 'r', encoding='utf-8') as f:
                findings = json.load(f)
            
            if not isinstance(findings, list):
                findings = [findings]
            
            print(f"Processing {len(findings)} findings...", file=sys.stderr)
            for i, finding in enumerate(findings):
                try:
                    osv_entry = contributor.from_finding(finding)
                    vuln_id = osv_entry['id']
                    output_file = Path(args.output_dir) / f"{vuln_id}.yaml"
                    contributor.save_to_file(osv_entry, str(output_file))
                    print(f"  [{i+1}/{len(findings)}] Saved {vuln_id} to {output_file}", file=sys.stderr)
                except Exception as e:
                    print(f"  [{i+1}/{len(findings)}] Error processing finding: {e}", file=sys.stderr)
            
            print(f"Done! Generated {len(findings)} OSV entries in {args.output_dir}", file=sys.stderr)
        
        # Mode 3: Generate from command-line args
        else:
            if not all([args.id, args.package, args.ecosystem, args.affected]):
                print("ERROR: --id, --package, --ecosystem, and --affected are required", file=sys.stderr)
                print("       Or use --from-finding / --from-findings", file=sys.stderr)
                return 1
            
            osv_entry = contributor.generate_osv_entry(
                vulnerability_id=args.id,
                package_name=args.package,
                package_ecosystem=args.ecosystem,
                affected_versions=args.affected,
                fixed_version=args.fixed,
                summary=args.summary or "",
                details=args.details or "",
                severity=args.severity,
                cvss_score=args.cvss,
                references=args.reference,
                aliases=args.alias
            )
            
            if args.output:
                contributor.save_to_file(osv_entry, args.output)
                print(f"Saved OSV entry to: {args.output}", file=sys.stderr)
            else:
                print(contributor.to_yaml(osv_entry))
        
        return 0
        
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 1
    except json.JSONDecodeError as e:
        print(f"ERROR: Invalid JSON: {e}", file=sys.stderr)
        return 2
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 3


if __name__ == '__main__':
    sys.exit(main())
