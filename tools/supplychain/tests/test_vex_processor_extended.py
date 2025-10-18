#!/usr/bin/env python3
"""Extended comprehensive tests for vex_processor.py to improve coverage."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
import sys

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from vex_processor import (
    load_vex_statements,
    parse_vex_statement,
    parse_csaf_vex,
    should_suppress_finding,
    filter_findings,
    validate_vex_statement,
    main,
)


class TestLoadVexStatements(unittest.TestCase):
    """Test load_vex_statements function."""
    
    def test_load_from_nonexistent_directory(self):
        """Test loading from non-existent directory returns empty list."""
        result = load_vex_statements("/nonexistent/path")
        self.assertEqual(result, [])
    
    def test_load_from_valid_directory(self):
        """Test loading VEX statements from directory with JSON files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            # Create test VEX files
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            vex2 = {"vulnerability_id": "OSV-2023-0002", "status": "mitigated"}
            
            Path(tmpdir, "vex1.json").write_text(json.dumps(vex1))
            Path(tmpdir, "vex2.json").write_text(json.dumps(vex2))
            
            result = load_vex_statements(tmpdir)
            self.assertEqual(len(result), 2)
    
    def test_load_with_invalid_json(self):
        """Test loading continues even with invalid JSON files."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            Path(tmpdir, "valid.json").write_text(json.dumps(vex1))
            Path(tmpdir, "invalid.json").write_text("{invalid json")
            
            result = load_vex_statements(tmpdir)
            # Should load the valid one, skip the invalid
            self.assertEqual(len(result), 1)
    
    def test_load_ignores_non_json_files(self):
        """Test that non-JSON files are ignored."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            Path(tmpdir, "vex.json").write_text(json.dumps(vex1))
            Path(tmpdir, "readme.txt").write_text("not json")
            
            result = load_vex_statements(tmpdir)
            self.assertEqual(len(result), 1)
    
    def test_load_empty_directory(self):
        """Test loading from empty directory."""
        with tempfile.TemporaryDirectory() as tmpdir:
            result = load_vex_statements(tmpdir)
            self.assertEqual(result, [])


class TestParseCsafVex(unittest.TestCase):
    """Test parse_csaf_vex function."""
    
    def test_parse_csaf_with_not_affected(self):
        """Test parsing CSAF VEX with not_affected status."""
        statement = {
            "document": {
                "tracking": {
                    "current_release_date": "2024-01-01T00:00:00Z"
                }
            },
            "vulnerabilities": [
                {
                    "cve": "CVE-2023-12345",
                    "product_status": {
                        "known_not_affected": ["product1"]
                    },
                    "notes": [{"text": "Not vulnerable"}]
                }
            ]
        }
        
        result = parse_csaf_vex(statement)
        self.assertIsNotNone(result)
        self.assertEqual(result['vulnerability_id'], "CVE-2023-12345")
        self.assertEqual(result['status'], "not_affected")
        self.assertEqual(result['justification'], "Not vulnerable")
    
    def test_parse_csaf_with_fixed(self):
        """Test parsing CSAF VEX with fixed status."""
        statement = {
            "document": {},
            "vulnerabilities": [
                {
                    "cve": "CVE-2023-12345",
                    "product_status": {
                        "fixed": ["product1"]
                    }
                }
            ]
        }
        
        result = parse_csaf_vex(statement)
        self.assertEqual(result['status'], "fixed")
    
    def test_parse_csaf_with_under_investigation(self):
        """Test parsing CSAF VEX with under_investigation status."""
        statement = {
            "document": {},
            "vulnerabilities": [
                {
                    "cve": "CVE-2023-12345",
                    "product_status": {
                        "under_investigation": ["product1"]
                    }
                }
            ]
        }
        
        result = parse_csaf_vex(statement)
        self.assertEqual(result['status'], "under_investigation")
    
    def test_parse_csaf_empty_vulnerabilities(self):
        """Test parsing CSAF VEX with empty vulnerabilities list."""
        statement = {
            "document": {},
            "vulnerabilities": []
        }
        
        result = parse_csaf_vex(statement)
        self.assertIsNone(result)
    
    def test_parse_csaf_without_notes(self):
        """Test parsing CSAF VEX without notes field."""
        statement = {
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
        
        result = parse_csaf_vex(statement)
        self.assertIsNotNone(result)
        self.assertEqual(result['justification'], "")


class TestShouldSuppressFindingExtended(unittest.TestCase):
    """Extended tests for should_suppress_finding function."""
    
    def test_suppress_with_mitigated_status(self):
        """Test suppression with mitigated status."""
        finding = {"id": "CVE-2023-0001"}
        vex_statements = [
            {"cve": "CVE-2023-0001", "status": "mitigated", "justification": "Patched"}
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)
        self.assertIn("mitigated", reason)
    
    def test_suppress_with_accepted_risk_status(self):
        """Test suppression with accepted_risk status."""
        finding = {"id": "CVE-2023-0001"}
        vex_statements = [
            {"cve": "CVE-2023-0001", "status": "accepted_risk"}
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)
    
    def test_no_suppress_with_fixed_status(self):
        """Test no suppression with fixed status (not in suppress list)."""
        finding = {"id": "CVE-2023-0001"}
        vex_statements = [
            {"cve": "CVE-2023-0001", "status": "fixed"}
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)
    
    def test_suppress_matching_by_cve_field(self):
        """Test matching using 'cve' field in finding."""
        finding = {"cve": "CVE-2023-0001"}
        vex_statements = [
            {"cve": "CVE-2023-0001", "status": "not_affected"}
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)
    
    def test_no_suppress_with_package_mismatch(self):
        """Test no suppression when package doesn't match."""
        finding = {
            "id": "CVE-2023-0001",
            "package": {"purl": "pkg:maven/com.example/lib1@1.0"}
        }
        vex_statements = [
            {
                "cve": "CVE-2023-0001",
                "package": "pkg:maven/com.example/lib2@1.0",
                "status": "not_affected"
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)
    
    def test_suppress_with_package_match(self):
        """Test suppression when package matches."""
        finding = {
            "id": "CVE-2023-0001",
            "package": {"purl": "pkg:maven/com.example/lib1@1.0"}
        }
        vex_statements = [
            {
                "cve": "CVE-2023-0001",
                "package": "com.example/lib1",
                "status": "not_affected"
            }
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertTrue(should_suppress)
    
    def test_no_suppress_with_invalid_vex_statement(self):
        """Test no suppression when VEX statement is invalid."""
        finding = {"id": "CVE-2023-0001"}
        vex_statements = [
            {"invalid": "format"}  # No cve or vulnerability_id
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)
    
    def test_no_suppress_finding_without_id(self):
        """Test no suppression when finding has no id."""
        finding = {"severity": "HIGH"}
        vex_statements = [
            {"cve": "CVE-2023-0001", "status": "not_affected"}
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)
        self.assertIsNone(reason)
    
    def test_no_suppress_vex_without_vulnerability_id(self):
        """Test no suppression when VEX statement has no vulnerability ID."""
        finding = {"id": "CVE-2023-0001"}
        vex_statements = [
            {"status": "not_affected"}  # No cve or vulnerability_id
        ]
        
        should_suppress, reason = should_suppress_finding(finding, vex_statements)
        self.assertFalse(should_suppress)


class TestMainFunction(unittest.TestCase):
    """Test main CLI function."""
    
    def test_main_with_validate_only(self):
        """Test main with --validate-only flag."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            (vex_dir / "vex1.json").write_text(json.dumps(vex1))
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', 'dummy.json',
                '--output', 'dummy.json',
                '--validate-only'
            ]):
                result = main()
                self.assertEqual(result, 0)
    
    def test_main_with_invalid_vex_validate_only(self):
        """Test main with invalid VEX in validate-only mode."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            invalid_vex = {"status": "not_affected"}  # Missing cve
            (vex_dir / "invalid.json").write_text(json.dumps(invalid_vex))
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', 'dummy.json',
                '--output', 'dummy.json',
                '--validate-only'
            ]):
                result = main()
                self.assertEqual(result, 1)  # Should return 1 for invalid statements
    
    def test_main_full_processing(self):
        """Test main with full processing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            (vex_dir / "vex1.json").write_text(json.dumps(vex1))
            
            sca_file = Path(tmpdir) / "sca.json"
            sca_data = {
                "findings": [
                    {"id": "CVE-2023-0001", "severity": "HIGH"},
                    {"id": "CVE-2023-0002", "severity": "MEDIUM"}
                ],
                "summary": {
                    "total_findings": 2
                }
            }
            sca_file.write_text(json.dumps(sca_data))
            
            output_file = Path(tmpdir) / "output.json"
            suppressed_file = Path(tmpdir) / "suppressed.json"
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', str(sca_file),
                '--output', str(output_file),
                '--suppressed-output', str(suppressed_file)
            ]):
                result = main()
                self.assertEqual(result, 0)
                
                # Check output files exist
                self.assertTrue(output_file.exists())
                self.assertTrue(suppressed_file.exists())
                
                # Check output content
                with open(output_file) as f:
                    output_data = json.load(f)
                self.assertEqual(len(output_data['findings']), 1)
                self.assertTrue(output_data['vex_applied'])
                self.assertEqual(output_data['suppressed_count'], 1)
                
                # Check suppressed file content
                with open(suppressed_file) as f:
                    suppressed_data = json.load(f)
                self.assertEqual(len(suppressed_data['suppressed_findings']), 1)
    
    def test_main_without_suppressed_output(self):
        """Test main without --suppressed-output flag."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            (vex_dir / "vex1.json").write_text(json.dumps(vex1))
            
            sca_file = Path(tmpdir) / "sca.json"
            sca_data = {
                "findings": [
                    {"id": "CVE-2023-0001", "severity": "HIGH"}
                ]
            }
            sca_file.write_text(json.dumps(sca_data))
            
            output_file = Path(tmpdir) / "output.json"
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', str(sca_file),
                '--output', str(output_file)
            ]):
                result = main()
                self.assertEqual(result, 0)
    
    def test_main_without_summary_in_sca_data(self):
        """Test main with SCA data that has no summary field."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            (vex_dir / "vex1.json").write_text(json.dumps(vex1))
            
            sca_file = Path(tmpdir) / "sca.json"
            sca_data = {
                "findings": [
                    {"id": "CVE-2023-0001", "severity": "HIGH"}
                ]
            }
            sca_file.write_text(json.dumps(sca_data))
            
            output_file = Path(tmpdir) / "output.json"
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', str(sca_file),
                '--output', str(output_file)
            ]):
                result = main()
                self.assertEqual(result, 0)
                
                with open(output_file) as f:
                    output_data = json.load(f)
                # Should not have summary field added
                self.assertNotIn('summary', output_data)
    
    def test_main_recalculates_severity_counts(self):
        """Test that severity counts are recalculated correctly."""
        with tempfile.TemporaryDirectory() as tmpdir:
            vex_dir = Path(tmpdir) / "vex"
            vex_dir.mkdir()
            
            vex1 = {"cve": "CVE-2023-0001", "status": "not_affected"}
            (vex_dir / "vex1.json").write_text(json.dumps(vex1))
            
            sca_file = Path(tmpdir) / "sca.json"
            sca_data = {
                "findings": [
                    {"id": "CVE-2023-0001", "severity": "CRITICAL"},
                    {"id": "CVE-2023-0002", "severity": "HIGH"},
                    {"id": "CVE-2023-0003", "severity": "MEDIUM"},
                    {"id": "CVE-2023-0004", "severity": "LOW"}
                ],
                "summary": {
                    "total_findings": 4,
                    "by_severity": {
                        "critical": 1,
                        "high": 1,
                        "medium": 1,
                        "low": 1
                    }
                }
            }
            sca_file.write_text(json.dumps(sca_data))
            
            output_file = Path(tmpdir) / "output.json"
            
            with patch('sys.argv', [
                'vex_processor.py',
                '--vex-dir', str(vex_dir),
                '--sca-findings', str(sca_file),
                '--output', str(output_file)
            ]):
                result = main()
                self.assertEqual(result, 0)
                
                with open(output_file) as f:
                    output_data = json.load(f)
                
                # Critical should be suppressed, counts should be updated
                self.assertEqual(output_data['summary']['by_severity']['critical'], 0)
                self.assertEqual(output_data['summary']['by_severity']['high'], 1)


class TestParseVexStatementExtended(unittest.TestCase):
    """Extended tests for parse_vex_statement."""
    
    def test_parse_csaf_format(self):
        """Test parsing CSAF format."""
        statement = {
            "document": {"tracking": {}},
            "vulnerabilities": [
                {
                    "cve": "CVE-2023-0001",
                    "product_status": {"known_not_affected": ["prod1"]}
                }
            ]
        }
        
        result = parse_vex_statement(statement)
        self.assertIsNotNone(result)
        self.assertEqual(result['status'], "not_affected")
    
    def test_parse_unknown_format_returns_none(self):
        """Test unknown format returns None."""
        statement = {"unknown_field": "value"}
        
        result = parse_vex_statement(statement)
        self.assertIsNone(result)
    
    def test_parse_with_defaults(self):
        """Test parsing with default values."""
        statement = {
            "cve": "CVE-2023-0001"
            # No status or justification
        }
        
        result = parse_vex_statement(statement)
        self.assertIsNotNone(result)
        self.assertEqual(result['status'], "not_affected")  # Default
        self.assertEqual(result['justification'], "")  # Default


class TestValidateVexStatementExtended(unittest.TestCase):
    """Extended tests for validate_vex_statement."""
    
    def test_validate_csaf_format(self):
        """Test validating CSAF format."""
        statement = {
            "document": {"tracking": {}},
            "vulnerabilities": [{"cve": "CVE-2023-0001"}]
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertTrue(is_valid)
        self.assertEqual(len(errors), 0)
    
    def test_validate_with_no_status_field(self):
        """Test validation with no status field is valid."""
        statement = {
            "cve": "CVE-2023-0001"
        }
        
        is_valid, errors = validate_vex_statement(statement)
        self.assertTrue(is_valid)
    
    def test_validate_all_valid_statuses(self):
        """Test all valid statuses pass validation."""
        valid_statuses = ['not_affected', 'false_positive', 'mitigated', 
                         'accepted_risk', 'fixed', 'under_investigation']
        
        for status in valid_statuses:
            statement = {
                "cve": "CVE-2023-0001",
                "status": status
            }
            is_valid, errors = validate_vex_statement(statement)
            self.assertTrue(is_valid, f"Status {status} should be valid")


if __name__ == '__main__':
    unittest.main()
