#!/usr/bin/env python3
"""Tests for vex_processor.py"""

import unittest
import json
import tempfile
import os
from pathlib import Path
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from vex_processor import (
    parse_vex_statement,
    should_suppress_finding,
    filter_findings,
    validate_vex_statement,
)


class TestParseVexStatement(unittest.TestCase):
    def test_parse_simple_format(self):
        statement = {
            "cve": "CVE-2023-12345",
            "package": "pkg:maven/com.example/vulnerable@1.0.0",
            "status": "not_affected",
            "justification": "Code path not used"
        }
        
        parsed = parse_vex_statement(statement)
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed['vulnerability_id'], "CVE-2023-12345")
        self.assertEqual(parsed['status'], "not_affected")
    
    def test_parse_with_vulnerability_id(self):
        statement = {
            "vulnerability_id": "OSV-2023-999",
            "status": "mitigated"
        }
        
        parsed = parse_vex_statement(statement)
        self.assertIsNotNone(parsed)
        self.assertEqual(parsed['vulnerability_id'], "OSV-2023-999")
    
    def test_parse_csaf_format_without_cve(self):
        """Test parsing CSAF VEX format without CVE ID."""
        statement = {
            "document": {},
            "vulnerabilities": [
                {
                    # No cve field
                    "product_status": {
                        "known_not_affected": ["product1"]
                    }
                }
            ]
        }
        
        parsed = parse_vex_statement(statement)
        # Should parse but vulnerability_id will be None
        self.assertIsNotNone(parsed)
        self.assertIsNone(parsed.get('vulnerability_id'))
    
    def test_parse_unknown_format(self):
        """Test parsing unknown VEX format returns None."""
        statement = {
            "unknown_field": "value"
        }
        
        parsed = parse_vex_statement(statement)
        self.assertIsNone(parsed)


class TestShouldSuppressFinding(unittest.TestCase):
    def test_suppress_not_affected(self):
        finding = {
            "id": "OSV-2023-12345",
            "cve": "CVE-2023-12345",
            "package": {
                "purl": "pkg:maven/com.example/vulnerable@1.0.0"
            }
        }
        
        vex_statements = [
            {
                "cve": "CVE-2023-12345",
                "status": "not_affected",
                "justification": "Test justification"
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)
        self.assertIn("not_affected", reason)
    
    def test_skip_vex_statement_without_vulnerability_id(self):
        """Test that VEX statements without vulnerability_id are skipped."""
        finding = {
            "id": "OSV-2023-12345",
            "cve": "CVE-2023-12345",
            "package": {
                "purl": "pkg:maven/com.example/vulnerable@1.0.0"
            }
        }
        
        # CSAF VEX format without CVE ID - parses but has no vulnerability_id
        vex_statements = [
            {
                "document": {},
                "vulnerabilities": [
                    {
                        # No cve field - will result in vulnerability_id being None
                        "product_status": {
                            "known_not_affected": ["product1"]
                        }
                    }
                ]
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        # Should not suppress since VEX statement has no vulnerability_id
        self.assertFalse(should_suppress)
    
    def test_suppress_csaf_vex_format(self):
        """Test suppression with CSAF VEX format."""
        finding = {
            "id": "CVE-2023-12345",
            "cve": "CVE-2023-12345",
            "package": {
                "purl": "pkg:maven/com.example/test@1.0.0"
            }
        }
        
        # CSAF VEX format with known_not_affected
        vex_statements = [
            {
                "document": {},
                "vulnerabilities": [
                    {
                        "cve": "CVE-2023-12345",
                        "product_status": {
                            "known_not_affected": ["product1"]
                        }
                    }
                ]
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        # Should suppress with CSAF VEX
        self.assertTrue(should_suppress)
    
    def test_suppress_csaf_vex_fixed_status(self):
        """Test CSAF VEX format with fixed status."""
        finding = {
            "id": "CVE-2023-888",
            "cve": "CVE-2023-888",
            "package": {
                "purl": "pkg:maven/com.example/test@1.0.0"
            }
        }
        
        # CSAF VEX format with fixed status (no known_not_affected)
        vex_statements = [
            {
                "document": {},
                "vulnerabilities": [
                    {
                        "cve": "CVE-2023-888",
                        "product_status": {
                            # Only fixed, not known_not_affected
                            "fixed": ["product1"]
                        }
                    }
                ]
            }
        ]
        
        # Parse the VEX statement
        from vex_processor import parse_vex_statement
        parsed = parse_vex_statement(vex_statements[0])
        # Verify it parsed to fixed status
        self.assertEqual(parsed['status'], 'fixed')
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        # Fixed is not a suppressible status in the current logic
        self.assertFalse(should_suppress)
    
    def test_suppress_csaf_vex_under_investigation(self):
        """Test CSAF VEX format with under_investigation status."""
        finding = {
            "id": "CVE-2023-999",
            "cve": "CVE-2023-999",
            "package": {
                "purl": "pkg:maven/com.example/test@1.0.0"
            }
        }
        
        # CSAF VEX format with only under_investigation (no known_not_affected or fixed)
        vex_statements = [
            {
                "document": {},
                "vulnerabilities": [
                    {
                        "cve": "CVE-2023-999",
                        "product_status": {
                            # Only under_investigation, not known_not_affected or fixed
                            "under_investigation": ["product1"]
                        }
                    }
                ]
            }
        ]
        
        # Parse the VEX statement to trigger the branch
        from vex_processor import parse_vex_statement
        parsed = parse_vex_statement(vex_statements[0])
        # Verify it parsed to under_investigation status
        self.assertEqual(parsed['status'], 'under_investigation')
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        # Under investigation is not a suppressible status
        self.assertFalse(should_suppress)
    
    def test_dont_suppress_different_cve(self):
        finding = {
            "id": "OSV-2023-12345",
            "cve": "CVE-2023-12345",
            "package": {
                "purl": "pkg:maven/com.example/vulnerable@1.0.0"
            }
        }
        
        vex_statements = [
            {
                "cve": "CVE-2023-99999",  # Different CVE
                "status": "not_affected"
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)
    
    def test_suppress_false_positive(self):
        finding = {
            "id": "OSV-2023-12345",
            "cve": "CVE-2023-12345",
            "package": {
                "purl": "pkg:maven/com.example/test@1.0.0"
            }
        }
        
        vex_statements = [
            {
                "cve": "CVE-2023-12345",
                "status": "false_positive",
                "justification": "Scanner error"
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)


class TestFilterFindings(unittest.TestCase):
    def test_filter_with_vex(self):
        findings = [
            {
                "id": "OSV-1",
                "cve": "CVE-2023-001",
                "severity": "HIGH",
                "package": {"purl": "pkg:maven/test/a@1.0"}
            },
            {
                "id": "OSV-2",
                "cve": "CVE-2023-002",
                "severity": "MEDIUM",
                "package": {"purl": "pkg:maven/test/b@1.0"}
            },
            {
                "id": "OSV-3",
                "cve": "CVE-2023-003",
                "severity": "LOW",
                "package": {"purl": "pkg:maven/test/c@1.0"}
            }
        ]
        
        vex_statements = [
            {
                "cve": "CVE-2023-002",
                "status": "not_affected"
            }
        ]
        
        remaining, suppressed = filter_findings(findings, vex_statements)
        
        self.assertEqual(len(remaining), 2)
        self.assertEqual(len(suppressed), 1)
        self.assertEqual(suppressed[0]['cve'], "CVE-2023-002")
        self.assertTrue(suppressed[0]['suppressed'])
    
    def test_no_vex_statements(self):
        findings = [
            {"id": "OSV-1", "cve": "CVE-2023-001", "package": {"purl": "pkg:maven/test/a@1.0"}}
        ]
        
        remaining, suppressed = filter_findings(findings, [])
        
        self.assertEqual(len(remaining), 1)
        self.assertEqual(len(suppressed), 0)


class TestValidateVexStatement(unittest.TestCase):
    def test_valid_statement(self):
        statement = {
            "cve": "CVE-2023-12345",
            "status": "not_affected",
            "justification": "Test"
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertTrue(is_valid)
        self.assertEqual(len(errors), 0)
    
    def test_missing_cve(self):
        statement = {
            "status": "not_affected"
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertFalse(is_valid)
        self.assertGreater(len(errors), 0)
    
    def test_invalid_status(self):
        statement = {
            "cve": "CVE-2023-12345",
            "status": "invalid_status"
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertFalse(is_valid)
        self.assertGreater(len(errors), 0)
    
    def test_valid_with_vulnerability_id(self):
        statement = {
            "vulnerability_id": "OSV-2023-999",
            "status": "mitigated"
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertTrue(is_valid)


if __name__ == '__main__':
    unittest.main()
