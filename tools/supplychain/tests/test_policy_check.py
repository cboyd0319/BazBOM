#!/usr/bin/env python3
"""Unit tests for policy_check.py."""

import unittest
import json
import sys
from pathlib import Path
import tempfile
import os

# Add parent directory to path to import policy_check
sys.path.insert(0, str(Path(__file__).parent.parent))
from policy_check import PolicyChecker, PolicyViolation


class TestPolicyViolation(unittest.TestCase):
    """Test PolicyViolation class."""
    
    def test_policy_violation_creation(self):
        """Test creating a policy violation."""
        violation = PolicyViolation(
            severity='HIGH',
            rule='test_rule',
            message='Test message',
            details={'foo': 'bar'}
        )
        
        self.assertEqual(violation.severity, 'HIGH')
        self.assertEqual(violation.rule, 'test_rule')
        self.assertEqual(violation.message, 'Test message')
        self.assertEqual(violation.details, {'foo': 'bar'})
    
    def test_policy_violation_str(self):
        """Test string representation of policy violation."""
        violation = PolicyViolation(
            severity='CRITICAL',
            rule='max_critical_vulnerabilities',
            message='Found 2 critical vulnerabilities'
        )
        
        result = str(violation)
        self.assertIn('CRITICAL', result)
        self.assertIn('max_critical_vulnerabilities', result)
        self.assertIn('Found 2 critical vulnerabilities', result)


class TestPolicyChecker(unittest.TestCase):
    """Test PolicyChecker class."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.config = {
            'max_critical': 0,
            'max_high': 5,
            'max_medium': float('inf'),
            'max_low': float('inf'),
            'blocked_licenses': ['GPL-2.0', 'GPL-3.0'],
            'block_license_conflicts': True,
            'flag_copyleft': False,
            'require_vex_for_accepted': False,
            'block_dependency_conflicts': False,
            'block_typosquatting': True,
            'unmaintained_threshold': 0,
        }
        self.checker = PolicyChecker(self.config)
    
    def test_no_violations_returns_zero_exit_code(self):
        """Test that no violations returns exit code 0."""
        self.assertEqual(self.checker.get_exit_code(), 0)
    
    def test_violations_return_nonzero_exit_code(self):
        """Test that violations return exit code 1."""
        self.checker.violations.append(PolicyViolation(
            severity='HIGH',
            rule='test',
            message='test'
        ))
        self.assertEqual(self.checker.get_exit_code(), 1)
    
    def test_check_vulnerability_thresholds_no_violations(self):
        """Test vulnerability check with no violations."""
        findings = {
            'vulnerabilities': [
                {'severity': 'MEDIUM', 'id': 'CVE-2023-0001'},
                {'severity': 'LOW', 'id': 'CVE-2023-0002'},
            ]
        }
        
        self.checker.check_vulnerability_thresholds(findings)
        self.assertEqual(len(self.checker.violations), 0)
    
    def test_check_vulnerability_thresholds_critical_violation(self):
        """Test vulnerability check with critical violation."""
        findings = {
            'vulnerabilities': [
                {'severity': 'CRITICAL', 'id': 'CVE-2023-0001'},
            ]
        }
        
        self.checker.check_vulnerability_thresholds(findings)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].severity, 'CRITICAL')
        self.assertEqual(self.checker.violations[0].rule, 'max_critical_vulnerabilities')
    
    def test_check_vulnerability_thresholds_high_violation(self):
        """Test vulnerability check with high severity violation."""
        findings = {
            'vulnerabilities': [
                {'severity': 'HIGH', 'id': f'CVE-2023-000{i}'}
                for i in range(6)  # 6 high vulns, threshold is 5
            ]
        }
        
        self.checker.check_vulnerability_thresholds(findings)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].severity, 'HIGH')
        self.assertEqual(self.checker.violations[0].rule, 'max_high_vulnerabilities')
    
    def test_check_vulnerability_thresholds_case_insensitive(self):
        """Test that severity comparison is case-insensitive."""
        findings = {
            'vulnerabilities': [
                {'severity': 'critical', 'id': 'CVE-2023-0001'},  # lowercase
                {'severity': 'High', 'id': 'CVE-2023-0002'},      # mixed case
            ]
        }
        
        self.checker.check_vulnerability_thresholds(findings)
        # Should detect 1 critical and 1 high
        self.assertEqual(len(self.checker.violations), 1)  # Only critical violates
    
    def test_check_license_compliance_no_violations(self):
        """Test license check with no violations."""
        report = {
            'packages': [
                {'name': 'foo', 'version': '1.0.0', 'licenses': ['Apache-2.0']},
                {'name': 'bar', 'version': '2.0.0', 'licenses': ['MIT']},
            ],
            'conflicts': []
        }
        
        self.checker.check_license_compliance(report)
        self.assertEqual(len(self.checker.violations), 0)
    
    def test_check_license_compliance_blocked_license(self):
        """Test detection of blocked license."""
        report = {
            'packages': [
                {'name': 'foo', 'version': '1.0.0', 'licenses': ['GPL-2.0']},
            ],
            'conflicts': []
        }
        
        self.checker.check_license_compliance(report)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'blocked_license')
        self.assertIn('GPL-2.0', self.checker.violations[0].message)
    
    def test_check_license_compliance_multiple_blocked_licenses(self):
        """Test detection of multiple blocked licenses."""
        report = {
            'packages': [
                {'name': 'foo', 'version': '1.0.0', 'licenses': ['GPL-2.0']},
                {'name': 'bar', 'version': '2.0.0', 'licenses': ['GPL-3.0']},
            ],
            'conflicts': []
        }
        
        self.checker.check_license_compliance(report)
        self.assertEqual(len(self.checker.violations), 2)
    
    def test_check_license_compliance_conflicts(self):
        """Test detection of license conflicts."""
        report = {
            'packages': [],
            'conflicts': [
                {'description': 'GPL-2.0 incompatible with proprietary'}
            ]
        }
        
        self.checker.check_license_compliance(report)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'license_conflict')
    
    def test_check_license_compliance_copyleft_flag(self):
        """Test flagging copyleft licenses."""
        self.checker.config['flag_copyleft'] = True
        
        report = {
            'packages': [],
            'conflicts': [],
            'summary': {'copyleft': 3}
        }
        
        self.checker.check_license_compliance(report)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'copyleft_license')
    
    def test_check_vex_requirements_disabled(self):
        """Test VEX check when disabled."""
        findings = {
            'vulnerabilities': [
                {'severity': 'CRITICAL', 'id': 'CVE-2023-0001'}
            ]
        }
        
        # VEX requirement disabled by default
        self.checker.check_vex_requirements(findings)
        self.assertEqual(len(self.checker.violations), 0)
    
    def test_check_vex_requirements_enabled(self):
        """Test VEX check when enabled."""
        self.checker.config['require_vex_for_accepted'] = True
        
        findings = {
            'vulnerabilities': [
                {
                    'severity': 'CRITICAL',
                    'id': 'CVE-2023-0001',
                    'package': {'name': 'foo'}
                }
            ]
        }
        
        self.checker.check_vex_requirements(findings)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'missing_vex_statement')
    
    def test_check_vex_requirements_with_vex_statement(self):
        """Test VEX check with VEX statement present."""
        self.checker.config['require_vex_for_accepted'] = True
        
        findings = {
            'vulnerabilities': [
                {
                    'severity': 'CRITICAL',
                    'id': 'CVE-2023-0001',
                    'vex_statement': 'Not affected - code path not used'
                }
            ]
        }
        
        self.checker.check_vex_requirements(findings)
        self.assertEqual(len(self.checker.violations), 0)
    
    def test_check_dependency_conflicts_disabled(self):
        """Test conflict check when disabled."""
        conflicts = {
            'conflicts': [
                {'package': 'foo', 'versions': ['1.0.0', '2.0.0']}
            ]
        }
        
        # Conflict blocking disabled by default
        self.checker.check_dependency_conflicts(conflicts)
        self.assertEqual(len(self.checker.violations), 0)
    
    def test_check_dependency_conflicts_enabled(self):
        """Test conflict check when enabled."""
        self.checker.config['block_dependency_conflicts'] = True
        
        conflicts = {
            'conflicts': [
                {'package': 'foo', 'versions': ['1.0.0', '2.0.0']}
            ]
        }
        
        self.checker.check_dependency_conflicts(conflicts)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'dependency_conflict')
    
    def test_check_supply_chain_risks_typosquatting(self):
        """Test detection of typosquatting risks."""
        risk_report = {
            'typosquatting_risks': [
                {'package': 'requets', 'similar_to': 'requests'}
            ],
            'unmaintained_packages': []
        }
        
        self.checker.check_supply_chain_risks(risk_report)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'typosquatting_risk')
    
    def test_check_supply_chain_risks_unmaintained(self):
        """Test detection of unmaintained packages."""
        risk_report = {
            'typosquatting_risks': [],
            'unmaintained_packages': [
                {'package': 'old-lib', 'last_update': '2020-01-01'}
            ]
        }
        
        self.checker.check_supply_chain_risks(risk_report)
        self.assertEqual(len(self.checker.violations), 1)
        self.assertEqual(self.checker.violations[0].rule, 'unmaintained_dependencies')
    
    def test_check_supply_chain_risks_unmaintained_threshold(self):
        """Test unmaintained threshold."""
        self.checker.config['unmaintained_threshold'] = 5
        
        risk_report = {
            'typosquatting_risks': [],
            'unmaintained_packages': [
                {'package': f'lib-{i}', 'last_update': '2020-01-01'}
                for i in range(3)
            ]
        }
        
        # 3 unmaintained, threshold is 5 - should pass
        self.checker.check_supply_chain_risks(risk_report)
        self.assertEqual(len(self.checker.violations), 0)
        
        # Add 3 more to exceed threshold
        risk_report['unmaintained_packages'].extend([
            {'package': f'lib-{i}', 'last_update': '2020-01-01'}
            for i in range(3, 7)
        ])
        
        self.checker.violations = []  # Reset
        self.checker.check_supply_chain_risks(risk_report)
        self.assertEqual(len(self.checker.violations), 1)
    
    def test_print_report_no_violations(self, capsys=None):
        """Test report printing with no violations."""
        # Just verify it doesn't crash
        self.checker.print_report()
    
    def test_print_report_with_violations(self):
        """Test report printing with violations."""
        self.checker.violations.append(PolicyViolation(
            severity='HIGH',
            rule='test_rule',
            message='Test violation'
        ))
        
        # Just verify it doesn't crash
        self.checker.print_report()


class TestPolicyCheckIntegration(unittest.TestCase):
    """Integration tests for policy check."""
    
    def test_end_to_end_with_files(self):
        """Test end-to-end policy check with temporary files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create test SCA findings
            findings_path = Path(tmpdir) / 'findings.json'
            findings = {
                'vulnerabilities': [
                    {'severity': 'CRITICAL', 'id': 'CVE-2023-0001'},
                    {'severity': 'HIGH', 'id': 'CVE-2023-0002'},
                ]
            }
            with open(findings_path, 'w') as f:
                json.dump(findings, f)
            
            # Create policy checker
            config = {
                'max_critical': 0,
                'max_high': 5,
                'max_medium': float('inf'),
                'max_low': float('inf'),
                'blocked_licenses': [],
                'block_license_conflicts': False,
                'flag_copyleft': False,
                'require_vex_for_accepted': False,
                'block_dependency_conflicts': False,
                'block_typosquatting': False,
                'unmaintained_threshold': 0,
            }
            
            checker = PolicyChecker(config)
            
            # Load and check findings
            with open(findings_path, 'r') as f:
                findings_data = json.load(f)
            
            checker.check_vulnerability_thresholds(findings_data)
            
            # Should have 1 violation (critical)
            self.assertEqual(len(checker.violations), 1)
            self.assertEqual(checker.get_exit_code(), 1)


if __name__ == '__main__':
    unittest.main()
