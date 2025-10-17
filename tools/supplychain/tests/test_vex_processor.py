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
