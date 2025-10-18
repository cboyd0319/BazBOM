#!/usr/bin/env python3
"""Tests for AI query engine."""

import json
import os
import tempfile
import unittest
from pathlib import Path

# Import the module from parent directory
import sys
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
if parent_dir not in sys.path:
    sys.path.insert(0, parent_dir)

import ai_query_engine


class TestAIQueryEngine(unittest.TestCase):
    """Test AI query engine functionality."""
    
    def setUp(self):
        """Set up test fixtures."""
        # Create sample SBOM
        self.temp_dir = tempfile.mkdtemp()
        self.sbom_path = os.path.join(self.temp_dir, "test.spdx.json")
        
        self.sample_sbom = {
            "spdxVersion": "SPDX-2.3",
            "dataLicense": "CC0-1.0",
            "SPDXID": "SPDXRef-DOCUMENT",
            "name": "Test SBOM",
            "packages": [
                {
                    "SPDXID": "SPDXRef-Package-1",
                    "name": "com.google.guava:guava",
                    "versionInfo": "31.1-jre",
                    "licenseConcluded": "Apache-2.0",
                    "supplier": "Organization: Google",
                },
                {
                    "SPDXID": "SPDXRef-Package-2",
                    "name": "org.apache.logging.log4j:log4j-core",
                    "versionInfo": "2.14.1",
                    "licenseConcluded": "Apache-2.0",
                    "externalRefs": [
                        {
                            "referenceCategory": "SECURITY",
                            "referenceType": "cpe23Type",
                            "referenceLocator": "CVE-2021-44228"
                        }
                    ]
                },
                {
                    "SPDXID": "SPDXRef-Package-3",
                    "name": "org.gnu:gpl-library",
                    "versionInfo": "1.0.0",
                    "licenseConcluded": "GPL-3.0",
                },
            ]
        }
        
        with open(self.sbom_path, 'w') as f:
            json.dump(self.sample_sbom, f)
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_initialization_without_sbom(self):
        """Test creating engine without loading SBOM."""
        engine = ai_query_engine.AIQueryEngine()
        self.assertIsNone(engine.sbom_data)
        self.assertEqual(len(engine.packages), 0)
    
    def test_load_sbom_success(self):
        """Test loading valid SBOM file."""
        engine = ai_query_engine.AIQueryEngine()
        engine.load_sbom(self.sbom_path)
        
        self.assertIsNotNone(engine.sbom_data)
        self.assertEqual(len(engine.packages), 3)
        self.assertGreater(len(engine.vulnerabilities), 0)
    
    def test_load_sbom_file_not_found(self):
        """Test loading non-existent SBOM file."""
        engine = ai_query_engine.AIQueryEngine()
        
        with self.assertRaises(FileNotFoundError):
            engine.load_sbom("/nonexistent/file.json")
    
    def test_load_sbom_invalid_json(self):
        """Test loading invalid JSON file."""
        bad_path = os.path.join(self.temp_dir, "bad.json")
        with open(bad_path, 'w') as f:
            f.write("{ invalid json }")
        
        engine = ai_query_engine.AIQueryEngine()
        
        with self.assertRaises(ValueError):
            engine.load_sbom(bad_path)
    
    def test_load_sbom_no_packages(self):
        """Test loading SBOM with no packages."""
        empty_path = os.path.join(self.temp_dir, "empty.json")
        with open(empty_path, 'w') as f:
            json.dump({"spdxVersion": "SPDX-2.3", "packages": []}, f)
        
        engine = ai_query_engine.AIQueryEngine()
        
        with self.assertRaises(ValueError):
            engine.load_sbom(empty_path)
    
    def test_query_without_loaded_sbom(self):
        """Test querying without loading SBOM."""
        engine = ai_query_engine.AIQueryEngine()
        
        with self.assertRaises(ValueError):
            engine.query("What uses log4j?")
    
    def test_query_find_dependencies(self):
        """Test finding dependencies by name."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("What uses log4j?")
        
        self.assertIn("log4j", result.answer.lower())
        self.assertGreater(len(result.data), 0)
        self.assertGreater(result.confidence, 0.5)
        self.assertEqual(result.data[0]["name"], "org.apache.logging.log4j:log4j-core")
    
    def test_query_find_dependencies_no_match(self):
        """Test finding dependencies with no matches."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("What uses zzz-nonexistent-package-zzz?")
        
        self.assertIn("No packages found", result.answer)
        self.assertEqual(len(result.data), 0)
    
    def test_query_find_by_license_gpl(self):
        """Test finding packages by GPL license."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Show GPL dependencies")
        
        self.assertIn("GPL", result.answer)
        self.assertGreater(len(result.data), 0)
        self.assertEqual(result.data[0]["license"], "GPL-3.0")
    
    def test_query_find_by_license_apache(self):
        """Test finding packages by Apache license."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("List Apache license dependencies")
        
        self.assertIn("Apache", result.answer)
        self.assertEqual(len(result.data), 2)  # guava and log4j
    
    def test_query_find_by_license_no_match(self):
        """Test finding packages by license with no matches."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Show BSD dependencies")
        
        self.assertIn("No packages found", result.answer)
        self.assertEqual(len(result.data), 0)
    
    def test_query_find_vulnerabilities(self):
        """Test finding vulnerabilities."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Which packages are vulnerable?")
        
        self.assertIn("vulnerability", result.answer.lower())
        self.assertGreater(len(result.data), 0)
        self.assertIn("CVE-2021-44228", result.data[0]["cve"])
    
    def test_query_find_specific_cve(self):
        """Test finding specific CVE."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Show CVE-2021-44228")
        
        self.assertIn("CVE-2021-44228", result.answer)
        self.assertGreater(len(result.data), 0)
        self.assertEqual(result.data[0]["package"], "org.apache.logging.log4j:log4j-core")
    
    def test_query_find_specific_cve_not_found(self):
        """Test finding CVE that doesn't exist in SBOM."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Show CVE-9999-99999")
        
        self.assertIn("not found", result.answer)
        self.assertEqual(len(result.data), 0)
    
    def test_query_cve_blast_radius(self):
        """Test querying CVE blast radius."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("What's the blast radius of CVE-2021-44228?")
        
        # Should mention the CVE and show affected packages
        self.assertIn("CVE-2021-44228", result.answer)
        self.assertIn("log4j", result.answer.lower())
    
    def test_query_count_dependencies(self):
        """Test counting dependencies."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("How many dependencies?")
        
        self.assertIn("3", result.answer)
        self.assertIn("Total dependencies", result.answer)
        self.assertGreater(result.confidence, 0.5)
    
    def test_query_statistics(self):
        """Test requesting statistics."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Show statistics")
        
        self.assertIn("Statistics", result.answer)
        self.assertIn("Total Packages", result.answer)
        self.assertIn("Apache-2.0", result.answer)  # Most common license
        self.assertGreater(result.confidence, 0.9)
    
    def test_query_unrecognized(self):
        """Test unrecognized query."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Tell me a joke")
        
        self.assertIn("not sure", result.answer.lower())
        self.assertGreater(len(result.suggestions), 0)
        self.assertEqual(result.confidence, 0.0)
    
    def test_query_upgrade_recommendations(self):
        """Test upgrade recommendation query (not yet implemented)."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result = engine.query("Should I upgrade guava?")
        
        self.assertIn("not yet implemented", result.answer.lower())
    
    def test_query_result_dataclass(self):
        """Test QueryResult dataclass."""
        result = ai_query_engine.QueryResult(
            query="test query",
            answer="test answer",
            data=[{"key": "value"}],
            confidence=0.95,
            suggestions=["suggestion 1"]
        )
        
        self.assertEqual(result.query, "test query")
        self.assertEqual(result.answer, "test answer")
        self.assertEqual(len(result.data), 1)
        self.assertAlmostEqual(result.confidence, 0.95)
        self.assertEqual(len(result.suggestions), 1)
    
    def test_query_patterns_coverage(self):
        """Test that all query patterns are valid regex."""
        for pattern, handler in ai_query_engine.AIQueryEngine.QUERY_PATTERNS:
            # Should compile without error
            import re
            compiled = re.compile(pattern, re.IGNORECASE)
            self.assertIsNotNone(compiled)
    
    def test_load_vulnerabilities_from_external_refs(self):
        """Test vulnerability loading from externalRefs."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        # Should have loaded CVE from externalRefs
        self.assertGreater(len(engine.vulnerabilities), 0)
        
        vuln = engine.vulnerabilities[0]
        self.assertEqual(vuln["cve"], "CVE-2021-44228")
        self.assertEqual(vuln["package"], "org.apache.logging.log4j:log4j-core")
    
    def test_initialization_with_sbom_path(self):
        """Test initializing engine with SBOM path."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        self.assertIsNotNone(engine.sbom_data)
        self.assertEqual(len(engine.packages), 3)
    
    def test_multiple_queries_same_engine(self):
        """Test running multiple queries on same engine instance."""
        engine = ai_query_engine.AIQueryEngine(self.sbom_path)
        
        result1 = engine.query("How many dependencies?")
        result2 = engine.query("Show GPL dependencies")
        result3 = engine.query("What uses log4j?")
        
        # All should succeed
        self.assertIsNotNone(result1.answer)
        self.assertIsNotNone(result2.answer)
        self.assertIsNotNone(result3.answer)
        
        # Each should have different results
        self.assertNotEqual(result1.answer, result2.answer)
        self.assertNotEqual(result2.answer, result3.answer)


class TestMainFunction(unittest.TestCase):
    """Test main entry point."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.sbom_path = os.path.join(self.temp_dir, "test.spdx.json")
        
        sample_sbom = {
            "spdxVersion": "SPDX-2.3",
            "packages": [
                {"name": "test-package", "versionInfo": "1.0.0", "licenseConcluded": "MIT"}
            ]
        }
        
        with open(self.sbom_path, 'w') as f:
            json.dump(sample_sbom, f)
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_main_single_query(self):
        """Test main function with single query."""
        import sys
        from unittest.mock import patch
        
        with patch.object(sys, 'argv', [
            'ai_query_engine.py',
            '--sbom', self.sbom_path,
            '--query', 'How many dependencies?'
        ]):
            result = ai_query_engine.main()
        
        self.assertEqual(result, 0)
    
    def test_main_json_output(self):
        """Test main function with JSON output."""
        import sys
        from unittest.mock import patch
        from io import StringIO
        
        with patch.object(sys, 'argv', [
            'ai_query_engine.py',
            '--sbom', self.sbom_path,
            '--query', 'How many dependencies?',
            '--json'
        ]):
            with patch('sys.stdout', new=StringIO()) as fake_out:
                result = ai_query_engine.main()
                output = fake_out.getvalue()
        
        self.assertEqual(result, 0)
        # Should be valid JSON
        json_data = json.loads(output)
        self.assertIn("query", json_data)
        self.assertIn("answer", json_data)
    
    def test_main_file_not_found(self):
        """Test main function with missing file."""
        import sys
        from unittest.mock import patch
        
        with patch.object(sys, 'argv', [
            'ai_query_engine.py',
            '--sbom', '/nonexistent/file.json',
            '--query', 'test'
        ]):
            result = ai_query_engine.main()
        
        self.assertEqual(result, 1)


if __name__ == '__main__':
    unittest.main()
