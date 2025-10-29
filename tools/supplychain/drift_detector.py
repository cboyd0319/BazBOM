#!/usr/bin/env python3
"""SBOM Drift Detector - Monitor unexpected changes in dependencies.

This module detects drift by comparing current SBOMs against a baseline
and identifying unexpected changes that may violate policies or introduce risks.
"""

import argparse
import json
import sys
from datetime import datetime
from pathlib import Path
from typing import Any, Dict, List, Optional

try:
    from sbom_diff import SBOMDiff, load_sbom, Package
except ImportError:
    print("ERROR: sbom_diff module not found", file=sys.stderr)
    print("Ensure sbom_diff.py is in the same directory", file=sys.stderr)
    sys.exit(1)


class DriftRule:
    """Rule for detecting drift."""
    
    def __init__(
        self,
        rule_id: str,
        name: str,
        description: str,
        severity: str = "WARNING"
    ):
        """Initialize drift rule.
        
        Args:
            rule_id: Unique rule identifier
            name: Human-readable rule name
            description: Rule description
            severity: Severity level (INFO, WARNING, ERROR, CRITICAL)
        """
        self.rule_id = rule_id
        self.name = name
        self.description = description
        self.severity = severity.upper()
    
    def check(self, diff: SBOMDiff) -> List[Dict[str, Any]]:
        """Check if this rule is violated.
        
        Args:
            diff: SBOM diff to analyze
            
        Returns:
            List of violation dictionaries
        """
        raise NotImplementedError("Subclasses must implement check()")


class UnexpectedAdditionsRule(DriftRule):
    """Detect unexpected new dependencies."""
    
    def __init__(self, max_additions: int = 5):
        """Initialize rule.
        
        Args:
            max_additions: Maximum allowed new dependencies
        """
        super().__init__(
            rule_id="DRIFT-001",
            name="Unexpected Dependency Additions",
            description=f"More than {max_additions} new dependencies added",
            severity="WARNING"
        )
        self.max_additions = max_additions
    
    def check(self, diff: SBOMDiff) -> List[Dict[str, Any]]:
        """Check for unexpected additions."""
        violations = []
        
        added = diff.get_added_packages()
        if len(added) > self.max_additions:
            violations.append({
                'rule_id': self.rule_id,
                'rule_name': self.name,
                'severity': self.severity,
                'message': f"Found {len(added)} new dependencies (threshold: {self.max_additions})",
                'details': {
                    'count': len(added),
                    'threshold': self.max_additions,
                    'packages': [pkg.to_dict() for pkg in added]
                }
            })
        
        return violations


class UnexpectedRemovalsRule(DriftRule):
    """Detect unexpected dependency removals."""
    
    def __init__(self, max_removals: int = 3):
        """Initialize rule.
        
        Args:
            max_removals: Maximum allowed removed dependencies
        """
        super().__init__(
            rule_id="DRIFT-002",
            name="Unexpected Dependency Removals",
            description=f"More than {max_removals} dependencies removed",
            severity="WARNING"
        )
        self.max_removals = max_removals
    
    def check(self, diff: SBOMDiff) -> List[Dict[str, Any]]:
        """Check for unexpected removals."""
        violations = []
        
        removed = diff.get_removed_packages()
        if len(removed) > self.max_removals:
            violations.append({
                'rule_id': self.rule_id,
                'rule_name': self.name,
                'severity': self.severity,
                'message': f"Found {len(removed)} removed dependencies (threshold: {self.max_removals})",
                'details': {
                    'count': len(removed),
                    'threshold': self.max_removals,
                    'packages': [pkg.to_dict() for pkg in removed]
                }
            })
        
        return violations


class LicenseChangeRule(DriftRule):
    """Detect license changes."""
    
    def __init__(self, forbidden_licenses: Optional[List[str]] = None):
        """Initialize rule.
        
        Args:
            forbidden_licenses: List of forbidden license identifiers (e.g., GPL-3.0)
        """
        super().__init__(
            rule_id="DRIFT-003",
            name="License Changes Detected",
            description="Dependencies changed to potentially problematic licenses",
            severity="ERROR"
        )
        self.forbidden_licenses = set(forbidden_licenses or ['GPL-2.0', 'GPL-3.0', 'AGPL-3.0'])
    
    def check(self, diff: SBOMDiff) -> List[Dict[str, Any]]:
        """Check for problematic license changes."""
        violations = []
        
        # Check license changes in existing packages
        for old, new in diff.get_license_changes():
            if new.license in self.forbidden_licenses:
                violations.append({
                    'rule_id': self.rule_id,
                    'rule_name': self.name,
                    'severity': self.severity,
                    'message': f"{old.name}: License changed to forbidden license {new.license}",
                    'details': {
                        'package': old.name,
                        'version': old.version,
                        'old_license': old.license,
                        'new_license': new.license,
                        'reason': 'Forbidden license'
                    }
                })
        
        # Check licenses in new packages
        for pkg in diff.get_added_packages():
            if pkg.license in self.forbidden_licenses:
                violations.append({
                    'rule_id': self.rule_id,
                    'rule_name': self.name,
                    'severity': self.severity,
                    'message': f"{pkg.name}: New dependency has forbidden license {pkg.license}",
                    'details': {
                        'package': pkg.name,
                        'version': pkg.version,
                        'license': pkg.license,
                        'reason': 'Forbidden license in new dependency'
                    }
                })
        
        return violations


class DowngradeRule(DriftRule):
    """Detect version downgrades (potential security risk)."""
    
    def __init__(self):
        """Initialize rule."""
        super().__init__(
            rule_id="DRIFT-004",
            name="Version Downgrades Detected",
            description="Dependencies downgraded to older versions",
            severity="WARNING"
        )
    
    def check(self, diff: SBOMDiff) -> List[Dict[str, Any]]:
        """Check for downgrades."""
        violations = []
        
        for old, new in diff.get_downgraded_packages():
            violations.append({
                'rule_id': self.rule_id,
                'rule_name': self.name,
                'severity': self.severity,
                'message': f"{old.name}: Downgraded from {old.version} to {new.version}",
                'details': {
                    'package': old.name,
                    'old_version': old.version,
                    'new_version': new.version,
                    'reason': 'Version downgrade may introduce known vulnerabilities'
                }
            })
        
        return violations


class DriftDetector:
    """Detect drift in SBOMs against baseline."""
    
    def __init__(self, rules: Optional[List[DriftRule]] = None):
        """Initialize drift detector.
        
        Args:
            rules: List of drift detection rules (default: all built-in rules)
        """
        if rules is None:
            # Default rules
            self.rules = [
                UnexpectedAdditionsRule(max_additions=5),
                UnexpectedRemovalsRule(max_removals=3),
                LicenseChangeRule(),
                DowngradeRule(),
            ]
        else:
            self.rules = rules
    
    def detect(self, sbom_baseline: Dict[str, Any], sbom_current: Dict[str, Any]) -> Dict[str, Any]:
        """Detect drift between baseline and current SBOM.
        
        Args:
            sbom_baseline: Baseline SBOM
            sbom_current: Current SBOM to check
            
        Returns:
            Dictionary with drift detection results
        """
        # Calculate diff
        diff = SBOMDiff(sbom_baseline, sbom_current)
        
        # Run all rules
        all_violations = []
        for rule in self.rules:
            violations = rule.check(diff)
            all_violations.extend(violations)
        
        # Count by severity
        severity_counts = {
            'CRITICAL': 0,
            'ERROR': 0,
            'WARNING': 0,
            'INFO': 0
        }
        
        for violation in all_violations:
            severity = violation.get('severity', 'WARNING')
            if severity in severity_counts:
                severity_counts[severity] += 1
        
        # Determine overall status
        if severity_counts['CRITICAL'] > 0 or severity_counts['ERROR'] > 0:
            status = 'FAILED'
        elif severity_counts['WARNING'] > 0:
            status = 'WARNING'
        else:
            status = 'PASSED'
        
        return {
            'metadata': {
                'baseline_sbom': sbom_baseline.get('name', 'unknown'),
                'current_sbom': sbom_current.get('name', 'unknown'),
                'detection_date': datetime.now().isoformat(),
                'rules_executed': len(self.rules),
            },
            'status': status,
            'summary': {
                'total_violations': len(all_violations),
                'critical': severity_counts['CRITICAL'],
                'error': severity_counts['ERROR'],
                'warning': severity_counts['WARNING'],
                'info': severity_counts['INFO'],
            },
            'diff_summary': diff.to_dict()['summary'],
            'violations': all_violations,
        }
    
    def to_human_readable(self, results: Dict[str, Any]) -> str:
        """Convert drift detection results to human-readable format.
        
        Args:
            results: Results from detect()
            
        Returns:
            Formatted string
        """
        lines = []
        
        # Header
        lines.append("=" * 80)
        lines.append("DRIFT DETECTION REPORT")
        lines.append("=" * 80)
        lines.append("")
        
        # Metadata
        meta = results['metadata']
        lines.append(f"Baseline: {meta['baseline_sbom']}")
        lines.append(f"Current:  {meta['current_sbom']}")
        lines.append(f"Date:     {meta['detection_date']}")
        lines.append("")
        
        # Overall status
        status = results['status']
        if status == 'PASSED':
            lines.append("[OK] STATUS: PASSED (no critical issues)")
        elif status == 'WARNING':
            lines.append("[WARNING]  STATUS: WARNING (non-critical issues found)")
        else:
            lines.append(" STATUS: FAILED (critical issues found)")
        lines.append("")
        
        # Summary
        summary = results['summary']
        lines.append("Summary:")
        lines.append(f"  Total violations:  {summary['total_violations']}")
        lines.append(f"  Critical:          {summary['critical']}")
        lines.append(f"  Error:             {summary['error']}")
        lines.append(f"  Warning:           {summary['warning']}")
        lines.append(f"  Info:              {summary['info']}")
        lines.append("")
        
        # Diff summary
        diff_summary = results['diff_summary']
        lines.append("Dependency Changes:")
        lines.append(f"  Added:             {diff_summary['added']}")
        lines.append(f"  Removed:           {diff_summary['removed']}")
        lines.append(f"  Upgraded:          {diff_summary['upgraded']}")
        lines.append(f"  Downgraded:        {diff_summary['downgraded']}")
        lines.append(f"  License changed:   {diff_summary['license_changed']}")
        lines.append("")
        
        # Violations
        violations = results['violations']
        if violations:
            lines.append("-" * 80)
            lines.append("DRIFT VIOLATIONS:")
            lines.append("-" * 80)
            
            for violation in violations:
                severity = violation['severity']
                icon = {
                    'CRITICAL': '[CRITICAL]',
                    'ERROR': '',
                    'WARNING': '[WARNING]',
                    'INFO': 'ℹ'
                }.get(severity, '•')
                
                lines.append(f"{icon} [{severity}] {violation['rule_id']}: {violation['rule_name']}")
                lines.append(f"   {violation['message']}")
                lines.append("")
        else:
            lines.append("[OK] No drift violations detected!")
            lines.append("")
        
        lines.append("=" * 80)
        
        return "\n".join(lines)


def main() -> int:
    """Main entry point for CLI.
    
    Returns:
        Exit code (0=passed, 1=warning, 2=failed, 3=error)
    """
    parser = argparse.ArgumentParser(
        description='Detect drift in SBOM against baseline',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Check current SBOM against baseline
  %(prog)s baseline.spdx.json current.spdx.json
  
  # Output JSON for automation
  %(prog)s baseline.json current.json --format json
  
  # Fail on warnings
  %(prog)s baseline.json current.json --strict
"""
    )
    
    parser.add_argument(
        'baseline',
        help='Path to baseline SBOM file'
    )
    
    parser.add_argument(
        'current',
        help='Path to current SBOM file to check'
    )
    
    parser.add_argument(
        '--format',
        choices=['text', 'json'],
        default='text',
        help='Output format (default: text)'
    )
    
    parser.add_argument(
        '-o', '--output',
        help='Output file (default: stdout)'
    )
    
    parser.add_argument(
        '--strict',
        action='store_true',
        help='Treat warnings as failures'
    )
    
    args = parser.parse_args()
    
    try:
        # Load SBOMs
        print(f"Loading baseline SBOM: {args.baseline}", file=sys.stderr)
        sbom_baseline = load_sbom(args.baseline)
        
        print(f"Loading current SBOM: {args.current}", file=sys.stderr)
        sbom_current = load_sbom(args.current)
        
        # Detect drift
        print("Detecting drift...", file=sys.stderr)
        detector = DriftDetector()
        results = detector.detect(sbom_baseline, sbom_current)
        
        # Generate output
        if args.format == 'json':
            output = json.dumps(results, indent=2)
        else:
            output = detector.to_human_readable(results)
        
        # Write output
        if args.output:
            print(f"Writing output to: {args.output}", file=sys.stderr)
            with open(args.output, 'w', encoding='utf-8') as f:
                f.write(output)
        else:
            print(output)
        
        # Determine exit code
        status = results['status']
        if status == 'FAILED':
            return 2
        elif status == 'WARNING':
            return 1 if args.strict else 0
        else:
            return 0
        
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 3
    except json.JSONDecodeError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 3
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 3
    except Exception as e:
        print(f"ERROR: Unexpected error: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc(file=sys.stderr)
        return 3


if __name__ == '__main__':
    sys.exit(main())
