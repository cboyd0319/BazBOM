#!/usr/bin/env python3
"""Policy enforcement for supply chain security.

This tool enforces security policies on vulnerability findings, license compliance,
and supply chain risks. It's designed to be used in CI/CD pipelines to block
builds that violate security policies.
"""

import argparse
import json
import sys
from typing import Dict, List, Any, Optional
from pathlib import Path


class PolicyViolation:
    """Represents a policy violation."""
    
    def __init__(self, severity: str, rule: str, message: str, details: Optional[Dict[str, Any]] = None):
        self.severity = severity
        self.rule = rule
        self.message = message
        self.details = details or {}
    
    def __str__(self) -> str:
        return f"[{self.severity}] {self.rule}: {self.message}"


class PolicyChecker:
    """Enforces supply chain security policies."""
    
    def __init__(self, config: Dict[str, Any]):
        self.config = config
        self.violations: List[PolicyViolation] = []
    
    def check_vulnerability_thresholds(self, sca_findings: Dict[str, Any]) -> None:
        """Check if vulnerability counts exceed policy thresholds.
        
        Args:
            sca_findings: SCA findings JSON data
        """
        max_critical = self.config.get('max_critical', 0)
        max_high = self.config.get('max_high', float('inf'))
        max_medium = self.config.get('max_medium', float('inf'))
        max_low = self.config.get('max_low', float('inf'))
        
        # Get vulnerability counts by severity
        vulnerabilities = sca_findings.get('vulnerabilities', [])
        counts = {
            'critical': 0,
            'high': 0,
            'medium': 0,
            'low': 0
        }
        
        for vuln in vulnerabilities:
            severity = vuln.get('severity', 'UNKNOWN').lower()
            if severity in counts:
                counts[severity] += 1
        
        # Check thresholds
        if counts['critical'] > max_critical:
            self.violations.append(PolicyViolation(
                severity='CRITICAL',
                rule='max_critical_vulnerabilities',
                message=f"Found {counts['critical']} critical vulnerabilities (max allowed: {max_critical})",
                details={'count': counts['critical'], 'threshold': max_critical}
            ))
        
        if counts['high'] > max_high:
            self.violations.append(PolicyViolation(
                severity='HIGH',
                rule='max_high_vulnerabilities',
                message=f"Found {counts['high']} high severity vulnerabilities (max allowed: {max_high})",
                details={'count': counts['high'], 'threshold': max_high}
            ))
        
        if counts['medium'] > max_medium:
            self.violations.append(PolicyViolation(
                severity='MEDIUM',
                rule='max_medium_vulnerabilities',
                message=f"Found {counts['medium']} medium severity vulnerabilities (max allowed: {max_medium})",
                details={'count': counts['medium'], 'threshold': max_medium}
            ))
        
        if counts['low'] > max_low:
            self.violations.append(PolicyViolation(
                severity='LOW',
                rule='max_low_vulnerabilities',
                message=f"Found {counts['low']} low severity vulnerabilities (max allowed: {max_low})",
                details={'count': counts['low'], 'threshold': max_low}
            ))
    
    def check_license_compliance(self, license_report: Dict[str, Any]) -> None:
        """Check for license policy violations.
        
        Args:
            license_report: License report JSON data
        """
        # Check for blocked licenses
        blocked_licenses = self.config.get('blocked_licenses', [])
        
        packages = license_report.get('packages', [])
        for pkg in packages:
            licenses = pkg.get('licenses', [])
            for license_id in licenses:
                if license_id in blocked_licenses:
                    self.violations.append(PolicyViolation(
                        severity='HIGH',
                        rule='blocked_license',
                        message=f"Package {pkg.get('name', 'unknown')} uses blocked license: {license_id}",
                        details={
                            'package': pkg.get('name'),
                            'version': pkg.get('version'),
                            'license': license_id
                        }
                    ))
        
        # Check for license conflicts
        conflicts = license_report.get('conflicts', [])
        if conflicts and self.config.get('block_license_conflicts', False):
            for conflict in conflicts:
                self.violations.append(PolicyViolation(
                    severity='MEDIUM',
                    rule='license_conflict',
                    message=f"License conflict detected: {conflict.get('description', 'unknown')}",
                    details=conflict
                ))
        
        # Check for copyleft licenses if flagged
        if self.config.get('flag_copyleft', False):
            copyleft_count = license_report.get('summary', {}).get('copyleft', 0)
            if copyleft_count > 0:
                self.violations.append(PolicyViolation(
                    severity='MEDIUM',
                    rule='copyleft_license',
                    message=f"Found {copyleft_count} dependencies with copyleft licenses",
                    details={'count': copyleft_count}
                ))
    
    def check_vex_requirements(self, sca_findings: Dict[str, Any]) -> None:
        """Check if accepted risks have VEX statements.
        
        Args:
            sca_findings: SCA findings JSON data
        """
        if not self.config.get('require_vex_for_accepted', False):
            return
        
        # Check for vulnerabilities without VEX statements
        vulnerabilities = sca_findings.get('vulnerabilities', [])
        for vuln in vulnerabilities:
            # If a vulnerability is present and not suppressed, it might need a VEX statement
            # This is a simplified check - real implementation would check against VEX database
            if vuln.get('severity', '').upper() in ['CRITICAL', 'HIGH']:
                # Check if there's a justification or VEX reference
                if not vuln.get('vex_statement') and not vuln.get('justification'):
                    self.violations.append(PolicyViolation(
                        severity='MEDIUM',
                        rule='missing_vex_statement',
                        message=f"High/critical vulnerability {vuln.get('id', 'unknown')} lacks VEX statement",
                        details={
                            'vulnerability_id': vuln.get('id'),
                            'package': vuln.get('package', {}).get('name')
                        }
                    ))
    
    def check_dependency_conflicts(self, conflicts_report: Dict[str, Any]) -> None:
        """Check for unresolved dependency conflicts.
        
        Args:
            conflicts_report: Dependency conflicts report
        """
        if not self.config.get('block_dependency_conflicts', False):
            return
        
        conflicts = conflicts_report.get('conflicts', [])
        if conflicts:
            for conflict in conflicts:
                self.violations.append(PolicyViolation(
                    severity='MEDIUM',
                    rule='dependency_conflict',
                    message=f"Dependency version conflict: {conflict.get('package', 'unknown')}",
                    details=conflict
                ))
    
    def check_supply_chain_risks(self, risk_report: Dict[str, Any]) -> None:
        """Check for supply chain security risks.
        
        Args:
            risk_report: Supply chain risk analysis report
        """
        # Check for typosquatting
        typosquatting = risk_report.get('typosquatting_risks', [])
        if typosquatting and self.config.get('block_typosquatting', True):
            for risk in typosquatting:
                self.violations.append(PolicyViolation(
                    severity='HIGH',
                    rule='typosquatting_risk',
                    message=f"Potential typosquatting detected: {risk.get('package', 'unknown')}",
                    details=risk
                ))
        
        # Check for unmaintained packages
        unmaintained = risk_report.get('unmaintained_packages', [])
        unmaintained_threshold = self.config.get('unmaintained_threshold', 0)
        if len(unmaintained) > unmaintained_threshold:
            self.violations.append(PolicyViolation(
                severity='LOW',
                rule='unmaintained_dependencies',
                message=f"Found {len(unmaintained)} unmaintained dependencies (threshold: {unmaintained_threshold})",
                details={'count': len(unmaintained), 'packages': unmaintained}
            ))
    
    def get_exit_code(self) -> int:
        """Get exit code based on violations.
        
        Returns:
            0 if no violations, 1 if violations found
        """
        # Any violation results in non-zero exit code
        if self.violations:
            # Exit code 1 for policy violations
            # Could be enhanced to use different codes for different severities
            return 1
        return 0
    
    def print_report(self) -> None:
        """Print policy check report."""
        if not self.violations:
            print(" All policy checks passed")
            return
        
        print(f"\n Policy violations found: {len(self.violations)}")
        print("=" * 80)
        
        # Group by severity
        by_severity = {'CRITICAL': [], 'HIGH': [], 'MEDIUM': [], 'LOW': []}
        for violation in self.violations:
            by_severity[violation.severity].append(violation)
        
        # Print violations by severity
        for severity in ['CRITICAL', 'HIGH', 'MEDIUM', 'LOW']:
            violations = by_severity[severity]
            if violations:
                print(f"\n{severity} ({len(violations)}):")
                for violation in violations:
                    print(f"  • {violation}")
        
        print("\n" + "=" * 80)
        print(f"Total violations: {len(self.violations)}")


def load_json_file(filepath: str) -> Optional[Dict[str, Any]]:
    """Load JSON file, return None if file doesn't exist or is invalid.
    
    Args:
        filepath: Path to JSON file
        
    Returns:
        Parsed JSON data or None
    """
    try:
        with open(filepath, 'r') as f:
            return json.load(f)
    except FileNotFoundError:
        print(f"Warning: File not found: {filepath}", file=sys.stderr)
        return None
    except json.JSONDecodeError as e:
        print(f"Warning: Invalid JSON in {filepath}: {e}", file=sys.stderr)
        return None


def main() -> int:
    parser = argparse.ArgumentParser(
        description='Enforce supply chain security policies',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Check with default thresholds (0 critical, unlimited others)
  %(prog)s --findings sca_findings.json
  
  # Strict policy: no critical/high vulnerabilities
  %(prog)s --findings sca_findings.json --max-critical 0 --max-high 0
  
  # Check all reports
  %(prog)s \\
    --findings sca_findings.json \\
    --license-report license_report.json \\
    --conflicts conflicts.json \\
    --risk-report supply_chain_risks.json \\
    --max-critical 0 --max-high 5
        """
    )
    
    # Input files
    parser.add_argument('--findings', help='SCA findings JSON file (filtered with VEX)')
    parser.add_argument('--license-report', help='License compliance report JSON')
    parser.add_argument('--conflicts', help='Dependency conflicts report JSON')
    parser.add_argument('--risk-report', help='Supply chain risk report JSON')
    
    # Vulnerability thresholds
    parser.add_argument('--max-critical', type=int, default=0,
                        help='Maximum critical vulnerabilities allowed (default: 0)')
    parser.add_argument('--max-high', type=int, default=None,
                        help='Maximum high vulnerabilities allowed (default: unlimited)')
    parser.add_argument('--max-medium', type=int, default=None,
                        help='Maximum medium vulnerabilities allowed (default: unlimited)')
    parser.add_argument('--max-low', type=int, default=None,
                        help='Maximum low vulnerabilities allowed (default: unlimited)')
    
    # License policies
    parser.add_argument('--blocked-licenses', nargs='+', default=[],
                        help='Space-separated list of blocked SPDX license IDs')
    parser.add_argument('--block-license-conflicts', action='store_true',
                        help='Block builds with license conflicts')
    parser.add_argument('--flag-copyleft', action='store_true',
                        help='Flag copyleft licenses as policy violation')
    
    # Other policies
    parser.add_argument('--require-vex-for-accepted', action='store_true',
                        help='Require VEX statements for accepted risks')
    parser.add_argument('--block-dependency-conflicts', action='store_true',
                        help='Block builds with dependency version conflicts')
    parser.add_argument('--block-typosquatting', action='store_true', default=True,
                        help='Block builds with typosquatting risks (default: true)')
    parser.add_argument('--unmaintained-threshold', type=int, default=0,
                        help='Maximum unmaintained dependencies allowed (default: 0)')
    
    # Output
    parser.add_argument('--output', help='Write policy report to JSON file')
    parser.add_argument('--quiet', action='store_true', help='Suppress output')
    
    args = parser.parse_args()
    
    # Build policy configuration
    config = {
        'max_critical': args.max_critical,
        'max_high': args.max_high if args.max_high is not None else float('inf'),
        'max_medium': args.max_medium if args.max_medium is not None else float('inf'),
        'max_low': args.max_low if args.max_low is not None else float('inf'),
        'blocked_licenses': args.blocked_licenses,
        'block_license_conflicts': args.block_license_conflicts,
        'flag_copyleft': args.flag_copyleft,
        'require_vex_for_accepted': args.require_vex_for_accepted,
        'block_dependency_conflicts': args.block_dependency_conflicts,
        'block_typosquatting': args.block_typosquatting,
        'unmaintained_threshold': args.unmaintained_threshold,
    }
    
    # Create policy checker
    checker = PolicyChecker(config)
    
    # Load and check findings
    if args.findings:
        findings = load_json_file(args.findings)
        if findings:
            checker.check_vulnerability_thresholds(findings)
            checker.check_vex_requirements(findings)
    
    if args.license_report:
        license_report = load_json_file(args.license_report)
        if license_report:
            checker.check_license_compliance(license_report)
    
    if args.conflicts:
        conflicts = load_json_file(args.conflicts)
        if conflicts:
            checker.check_dependency_conflicts(conflicts)
    
    if args.risk_report:
        risk_report = load_json_file(args.risk_report)
        if risk_report:
            checker.check_supply_chain_risks(risk_report)
    
    # Print report
    if not args.quiet:
        checker.print_report()
    
    # Write JSON output if requested
    if args.output:
        report = {
            'total_violations': len(checker.violations),
            'violations': [
                {
                    'severity': v.severity,
                    'rule': v.rule,
                    'message': v.message,
                    'details': v.details
                }
                for v in checker.violations
            ],
            'policy_config': config
        }
        
        with open(args.output, 'w') as f:
            json.dump(report, f, indent=2)
        
        if not args.quiet:
            print(f"\nPolicy report written to {args.output}")
    
    return checker.get_exit_code()


if __name__ == '__main__':
    sys.exit(main())
