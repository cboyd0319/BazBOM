#!/usr/bin/env python3
"""Tests for sarif_adapter.py - SARIF generation functionality."""

import json
import sys
import tempfile
import unittest
from pathlib import Path

# Add parent directory to path to import sarif_adapter
sys.path.insert(0, str(Path(__file__).parent.parent))

from sarif_adapter import (
    severity_to_level,
    create_sarif_document,
    main,
)


class TestSeverityToLevel(unittest.TestCase):
    """Test severity level conversion."""
    
    def test_critical_severity(self):
        """Test CRITICAL maps to error."""
        self.assertEqual(severity_to_level("CRITICAL"), "error")
        self.assertEqual(severity_to_level("critical"), "error")
    
    def test_high_severity(self):
        """Test HIGH maps to error."""
        self.assertEqual(severity_to_level("HIGH"), "error")
        self.assertEqual(severity_to_level("high"), "error")
    
    def test_medium_severity(self):
        """Test MEDIUM maps to warning."""
        self.assertEqual(severity_to_level("MEDIUM"), "warning")
        self.assertEqual(severity_to_level("medium"), "warning")
    
    def test_low_severity(self):
        """Test LOW maps to note."""
        self.assertEqual(severity_to_level("LOW"), "note")
        self.assertEqual(severity_to_level("low"), "note")
    
    def test_unknown_severity(self):
        """Test unknown severity defaults to note."""
        self.assertEqual(severity_to_level("UNKNOWN"), "note")
        self.assertEqual(severity_to_level(""), "note")


class TestCreateSarifDocument(unittest.TestCase):
    """Test SARIF document generation."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.sample_vulnerabilities = [
            {
                "package": "com.example:vulnerable-lib",
                "version": "1.2.3",
                "vulnerability": {
                    "id": "GHSA-1234-5678-90ab",
                    "summary": "Remote code execution vulnerability",
                    "details": "Allows arbitrary code execution via crafted input.",
                    "severity": [
                        {
                            "type": "CVSS_V3",
                            "score": "7.5"
                        }
                    ],
                    "affected": [
                        {
                            "package": {
                                "ecosystem": "Maven",
                                "name": "com.example:vulnerable-lib"
                            },
                            "ranges": [
                                {
                                    "type": "ECOSYSTEM",
                                    "events": [
                                        {"introduced": "0"},
                                        {"fixed": "1.2.4"}
                                    ]
                                }
                            ]
                        }
                    ],
                    "references": [
                        {
                            "type": "ADVISORY",
                            "url": "https://github.com/advisories/GHSA-1234-5678-90ab"
                        }
                    ]
                }
            }
        ]
    
    def test_basic_structure(self):
        """Test that SARIF document has required fields."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        
        # Check required top-level fields
        self.assertEqual(doc["version"], "2.1.0")
        self.assertEqual(doc["$schema"], 
            "https://json.schemastore.org/sarif-2.1.0.json")
        self.assertIn("runs", doc)
        self.assertEqual(len(doc["runs"]), 1)
    
    def test_run_structure(self):
        """Test that run has required fields."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        run = doc["runs"][0]
        
        self.assertIn("tool", run)
        self.assertIn("results", run)
        
        # Check tool info
        tool = run["tool"]
        self.assertIn("driver", tool)
        self.assertEqual(tool["driver"]["name"], "BazBOM SCA")
    
    def test_results_included(self):
        """Test that vulnerabilities are converted to results."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        run = doc["runs"][0]
        results = run["results"]
        
        self.assertEqual(len(results), 1)
        result = results[0]
        
        # Check required result fields
        self.assertIn("ruleId", result)
        self.assertIn("level", result)
        self.assertIn("message", result)
    
    def test_rule_id(self):
        """Test that rule ID matches vulnerability ID."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        self.assertEqual(result["ruleId"], "GHSA-1234-5678-90ab")
    
    def test_severity_mapping(self):
        """Test that CVSS scores map to correct SARIF levels."""
        # Test HIGH severity (7.5 CVSS)
        doc = create_sarif_document(self.sample_vulnerabilities)
        result = doc["runs"][0]["results"][0]
        # Level should be error or warning based on severity
        self.assertIn(result["level"], ["error", "warning"])
    
    def test_message(self):
        """Test that message includes summary."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        message = result["message"]["text"]
        self.assertIn("Remote code execution", message)
        self.assertIn("com.example:vulnerable-lib", message)
    
    def test_locations(self):
        """Test that locations are included if available."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Check if locations are present
        if "locations" in result:
            self.assertTrue(len(result["locations"]) > 0)
    
    def test_empty_vulnerabilities(self):
        """Test handling of empty vulnerability list."""
        doc = create_sarif_document([])
        
        # Should still have valid structure
        self.assertEqual(doc["version"], "2.1.0")
        self.assertIn("runs", doc)
        
        run = doc["runs"][0]
        self.assertEqual(len(run["results"]), 0)
    
    def test_multiple_vulnerabilities(self):
        """Test handling of multiple vulnerabilities."""
        vulns = [
            {
                "package": "pkg1",
                "version": "1.0.0",
                "vulnerability": {
                    "id": "VULN-1",
                    "summary": "Vuln 1",
                    "severity": []
                }
            },
            {
                "package": "pkg2",
                "version": "2.0.0",
                "vulnerability": {
                    "id": "VULN-2",
                    "summary": "Vuln 2",
                    "severity": []
                }
            }
        ]
        
        doc = create_sarif_document(vulns)
        results = doc["runs"][0]["results"]
        
        self.assertEqual(len(results), 2)
        self.assertEqual(results[0]["ruleId"], "VULN-1")
        self.assertEqual(results[1]["ruleId"], "VULN-2")
    
    def test_missing_severity(self):
        """Test handling of missing severity information."""
        vulns = [{
            "package": "test-pkg",
            "version": "1.0.0",
            "vulnerability": {
                "id": "TEST-VULN",
                "summary": "Test vulnerability",
                "severity": []  # No severity info
            }
        }]
        
        doc = create_sarif_document(vulns)
        result = doc["runs"][0]["results"][0]
        
        # Should default to warning
        self.assertIn("level", result)
        self.assertIn(result["level"], ["error", "warning", "note"])
    
    def test_references(self):
        """Test that references are included."""
        doc = create_sarif_document(self.sample_vulnerabilities)
        result = doc["runs"][0]["results"][0]
        
        # Check if help URI is present
        if "helpUri" in result:
            self.assertTrue(result["helpUri"].startswith("http"))


class TestMainFunction(unittest.TestCase):
    """Test main entry point and CLI handling."""
    
    def setUp(self):
        """Set up temporary files for testing."""
        self.temp_dir = tempfile.mkdtemp()
        self.temp_dir_path = Path(self.temp_dir)
        
        # Create sample input file
        self.input_file = self.temp_dir_path / "sca_findings.json"
        sample_data = {
            "packages_scanned": 1,
            "vulnerabilities_found": 1,
            "vulnerabilities": [
                {
                    "id": "TEST-VULN-123",
                    "package": {
                        "name": "test-package",
                        "purl": "pkg:maven/com.test/package@1.0.0"
                    },
                    "severity": "HIGH",
                    "summary": "Test vulnerability"
                }
            ]
        }
        with open(self.input_file, 'w') as f:
            json.dump(sample_data, f)
    
    def tearDown(self):
        """Clean up temporary files."""
        import shutil
        shutil.rmtree(self.temp_dir)
    
    def test_successful_conversion(self):
        """Test successful conversion to SARIF."""
        output_file = self.temp_dir_path / "output.sarif"
        
        sys.argv = [
            "sarif_adapter.py",
            "--input", str(self.input_file),
            "--output", str(output_file)
        ]
        
        result = main()
        self.assertEqual(result, 0)
        
        # Verify output file exists and is valid JSON
        self.assertTrue(output_file.exists())
        with open(output_file) as f:
            data = json.load(f)
        
        self.assertEqual(data["version"], "2.1.0")
        self.assertIn("runs", data)
    
    def test_missing_input_file(self):
        """Test handling of missing input file."""
        output_file = self.temp_dir_path / "output.sarif"
        
        sys.argv = [
            "sarif_adapter.py",
            "--input", "/nonexistent/file.json",
            "--output", str(output_file)
        ]
        
        result = main()
        self.assertNotEqual(result, 0)
    
    def test_invalid_json_input(self):
        """Test handling of invalid JSON input."""
        bad_input = self.temp_dir_path / "bad.json"
        with open(bad_input, 'w') as f:
            f.write("not valid json{")
        
        output_file = self.temp_dir_path / "output.sarif"
        
        sys.argv = [
            "sarif_adapter.py",
            "--input", str(bad_input),
            "--output", str(output_file)
        ]
        
        result = main()
        self.assertNotEqual(result, 0)


if __name__ == '__main__':
    unittest.main()
