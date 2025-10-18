#!/usr/bin/env python3
"""Tests for cve_tracker.py RipGrep integration."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from cve_tracker import (
    check_ripgrep_available,
    find_cve_references,
    cross_reference_with_sbom,
    find_vex_statements,
)


class TestCVETracker(unittest.TestCase):
    """Test suite for CVE tracker."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = tempfile.mkdtemp()
        self.test_path = Path(self.test_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)

    @patch('subprocess.run')
    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_happy_path(self, mock_check, mock_run):
        """Test finding CVE references with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output
        mock_output = '''src/main/java/Foo.java:42:CVE-2023-12345
docs/SECURITY.md:10:CVE-2023-67890'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        cves = find_cve_references(str(self.test_path))
        
        self.assertEqual(len(cves), 2)
        self.assertEqual(cves[0]['cve'], 'CVE-2023-12345')
        self.assertEqual(cves[0]['file'], 'src/main/java/Foo.java')
        self.assertEqual(cves[0]['line'], 42)

    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_ripgrep_not_available(self, mock_check):
        """Test CVE finding when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            find_cve_references(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_invalid_workspace(self, mock_check):
        """Test CVE finding with invalid workspace."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            find_cve_references('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))

    def test_cross_reference_with_sbom_happy_path(self):
        """Test cross-referencing CVEs with SBOM findings."""
        # Create sample CVE references
        cves = [
            {'file': 'src/Foo.java', 'line': 42, 'cve': 'CVE-2023-12345'},
            {'file': 'docs/SECURITY.md', 'line': 10, 'cve': 'CVE-2023-67890'}
        ]
        
        # Create sample SBOM findings
        sbom_file = self.test_path / "sbom_findings.json"
        sbom_data = {
            "vulnerabilities": [
                {"cve": "CVE-2023-12345"},
                {"id": "CVE-2023-99999"}
            ]
        }
        with open(sbom_file, 'w') as f:
            json.dump(sbom_data, f)
        
        result = cross_reference_with_sbom(cves, str(sbom_file))
        
        # Should categorize correctly
        self.assertIn('CVE-2023-12345', result['in_both'])
        self.assertIn('CVE-2023-67890', result['documented_only'])
        self.assertIn('CVE-2023-99999', result['sbom_only'])

    def test_cross_reference_with_sbom_file_not_found(self):
        """Test cross-reference with missing SBOM file."""
        cves = []
        
        with self.assertRaises(ValueError) as ctx:
            cross_reference_with_sbom(cves, '/nonexistent/sbom.json')
        self.assertIn('not found', str(ctx.exception))

    def test_cross_reference_with_sbom_invalid_json(self):
        """Test cross-reference with invalid SBOM JSON."""
        cves = []
        
        sbom_file = self.test_path / "invalid.json"
        sbom_file.write_text("{invalid json")
        
        with self.assertRaises(ValueError) as ctx:
            cross_reference_with_sbom(cves, str(sbom_file))
        self.assertIn('Invalid JSON', str(ctx.exception))

    @patch('subprocess.run')
    @patch('cve_tracker.check_ripgrep_available')
    def test_find_vex_statements_happy_path(self, mock_check, mock_run):
        """Test finding VEX statements with CVEs."""
        mock_check.return_value = True
        
        # Mock finding VEX files
        def side_effect(*args, **kwargs):
            cmd = args[0]
            if '--files' in cmd:
                return MagicMock(
                    returncode=0,
                    stdout='vex/statements/vex-001.json\n'
                )
            else:
                # CVE extraction
                return MagicMock(
                    returncode=0,
                    stdout='CVE-2023-12345\nCVE-2023-67890\n'
                )
        
        mock_run.side_effect = side_effect
        
        vex_statements = find_vex_statements(str(self.test_path))
        
        self.assertEqual(len(vex_statements), 1)
        self.assertEqual(vex_statements[0]['file'], 'vex/statements/vex-001.json')
        self.assertEqual(len(vex_statements[0]['cves']), 2)

    @patch('subprocess.run')
    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_timeout(self, mock_check, mock_run):
        """Test CVE finding with timeout."""
        mock_check.return_value = True
        
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 60)
        
        with self.assertRaises(RuntimeError) as ctx:
            find_cve_references(str(self.test_path))
        self.assertIn('timed out', str(ctx.exception))

    @patch('subprocess.run')
    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_empty_result(self, mock_check, mock_run):
        """Test CVE finding with no results."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(returncode=0, stdout='')
        
        cves = find_cve_references(str(self.test_path))
        
        self.assertEqual(len(cves), 0)

    @patch('subprocess.run')
    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_malformed_line(self, mock_check, mock_run):
        """Test CVE finding with malformed output lines."""
        mock_check.return_value = True
        
        # Mock output with some malformed lines
        mock_output = '''src/main/java/Foo.java:42:CVE-2023-12345
malformed line without CVE
docs/SECURITY.md:10:CVE-2023-67890'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        cves = find_cve_references(str(self.test_path))
        
        # Should skip malformed lines
        self.assertEqual(len(cves), 2)

    @patch('cve_tracker.check_ripgrep_available')
    def test_find_cve_references_workspace_is_file(self, mock_check):
        """Test CVE finding when workspace is a file."""
        mock_check.return_value = True
        
        test_file = self.test_path / "test.txt"
        test_file.write_text("test")
        
        with self.assertRaises(ValueError) as ctx:
            find_cve_references(str(test_file))
        self.assertIn('not a directory', str(ctx.exception))

    def test_cross_reference_with_sbom_empty_cves(self):
        """Test cross-reference with no CVEs."""
        sbom_file = self.test_path / "sbom.json"
        sbom_data = {"vulnerabilities": []}
        with open(sbom_file, 'w') as f:
            json.dump(sbom_data, f)
        
        result = cross_reference_with_sbom([], str(sbom_file))
        
        self.assertEqual(len(result['in_both']), 0)
        self.assertEqual(len(result['documented_only']), 0)
        self.assertEqual(len(result['sbom_only']), 0)


if __name__ == '__main__':
    unittest.main()
