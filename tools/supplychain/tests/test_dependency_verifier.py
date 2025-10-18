#!/usr/bin/env python3
"""Tests for dependency_verifier.py RipGrep integration."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock, mock_open
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from dependency_verifier import (
    check_ripgrep_available,
    get_declared_dependencies,
    get_referenced_dependencies,
    find_unused_dependencies,
    find_undeclared_dependencies,
    generate_usage_report,
)


class TestDependencyVerifier(unittest.TestCase):
    """Test suite for dependency verifier."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = tempfile.mkdtemp()
        self.test_path = Path(self.test_dir)
        
        # Create a sample maven_install.json
        self.maven_json = self.test_path / "maven_install.json"
        maven_data = {
            "dependency_tree": {
                "dependencies": [
                    {"coord": "com.google.guava:guava:31.1-jre"},
                    {"coord": "org.slf4j:slf4j-api:1.7.36"}
                ]
            }
        }
        with open(self.maven_json, 'w') as f:
            json.dump(maven_data, f)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)

    def test_get_declared_dependencies_happy_path(self):
        """Test getting declared dependencies from maven_install.json."""
        deps = get_declared_dependencies(str(self.maven_json))
        
        self.assertIsInstance(deps, set)
        self.assertIn('com_google_guava_guava', deps)
        self.assertIn('org_slf4j_slf4j_api', deps)

    def test_get_declared_dependencies_file_not_found(self):
        """Test getting declared dependencies with missing file."""
        with self.assertRaises(ValueError) as ctx:
            get_declared_dependencies('/nonexistent/maven_install.json')
        self.assertIn('not found', str(ctx.exception))

    def test_get_declared_dependencies_invalid_json(self):
        """Test getting declared dependencies with invalid JSON."""
        invalid_json = self.test_path / "invalid.json"
        invalid_json.write_text("{invalid json")
        
        with self.assertRaises(ValueError) as ctx:
            get_declared_dependencies(str(invalid_json))
        self.assertIn('Invalid JSON', str(ctx.exception))

    def test_get_declared_dependencies_alternative_format(self):
        """Test getting declared dependencies with alternative JSON format."""
        alt_json = self.test_path / "alt_maven_install.json"
        alt_data = {
            "artifacts": {
                "com.google.guava:guava:31.1-jre": {},
                "org.slf4j:slf4j-api:1.7.36": {}
            }
        }
        with open(alt_json, 'w') as f:
            json.dump(alt_data, f)
        
        deps = get_declared_dependencies(str(alt_json))
        
        self.assertIn('com_google_guava_guava', deps)

    @patch('subprocess.run')
    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_happy_path(self, mock_check, mock_run):
        """Test getting referenced dependencies with RipGrep."""
        mock_check.return_value = True
        
        mock_output = '''@maven//:com_google_guava_guava
@maven//:org_slf4j_slf4j_api
@maven//:com_google_guava_guava'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        refs = get_referenced_dependencies(str(self.test_path))
        
        # Should deduplicate
        self.assertEqual(len(refs), 2)
        self.assertIn('com_google_guava_guava', refs)

    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_ripgrep_not_available(self, mock_check):
        """Test getting referenced dependencies when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            get_referenced_dependencies(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_invalid_workspace(self, mock_check):
        """Test getting referenced dependencies with invalid workspace."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            get_referenced_dependencies('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))

    @patch('dependency_verifier.get_referenced_dependencies')
    def test_find_unused_dependencies(self, mock_refs):
        """Test finding unused dependencies."""
        # Only one dependency is referenced
        mock_refs.return_value = {'com_google_guava_guava'}
        
        unused = find_unused_dependencies(str(self.test_path), str(self.maven_json))
        
        # slf4j should be unused
        self.assertIn('org_slf4j_slf4j_api', unused)
        self.assertNotIn('com_google_guava_guava', unused)

    @patch('dependency_verifier.get_referenced_dependencies')
    def test_find_undeclared_dependencies(self, mock_refs):
        """Test finding undeclared dependencies."""
        # Reference a dependency not in maven_install.json
        mock_refs.return_value = {
            'com_google_guava_guava',
            'undeclared_dependency'
        }
        
        undeclared = find_undeclared_dependencies(str(self.test_path), str(self.maven_json))
        
        # undeclared_dependency should be found
        self.assertIn('undeclared_dependency', undeclared)

    @patch('dependency_verifier.get_referenced_dependencies')
    def test_generate_usage_report(self, mock_refs):
        """Test generating usage report."""
        mock_refs.return_value = {'com_google_guava_guava'}
        
        report = generate_usage_report(str(self.test_path), str(self.maven_json))
        
        self.assertEqual(report['declared_count'], 2)
        self.assertEqual(report['referenced_count'], 1)
        self.assertEqual(report['used_count'], 1)
        self.assertEqual(report['unused_count'], 1)
        self.assertIn('usage_rate', report)

    @patch('subprocess.run')
    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_timeout(self, mock_check, mock_run):
        """Test getting referenced dependencies with timeout."""
        mock_check.return_value = True
        
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 60)
        
        with self.assertRaises(RuntimeError) as ctx:
            get_referenced_dependencies(str(self.test_path))
        self.assertIn('timed out', str(ctx.exception))

    @patch('subprocess.run')
    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_empty_result(self, mock_check, mock_run):
        """Test getting referenced dependencies with no results."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(returncode=0, stdout='')
        
        refs = get_referenced_dependencies(str(self.test_path))
        
        self.assertEqual(len(refs), 0)

    @patch('dependency_verifier.check_ripgrep_available')
    def test_get_referenced_dependencies_workspace_is_file(self, mock_check):
        """Test getting referenced dependencies when workspace is a file."""
        mock_check.return_value = True
        
        test_file = self.test_path / "test.txt"
        test_file.write_text("test")
        
        with self.assertRaises(ValueError) as ctx:
            get_referenced_dependencies(str(test_file))
        self.assertIn('not a directory', str(ctx.exception))

    def test_get_declared_dependencies_empty_dependencies(self):
        """Test getting declared dependencies with no dependencies."""
        empty_json = self.test_path / "empty.json"
        empty_data = {"dependency_tree": {"dependencies": []}}
        with open(empty_json, 'w') as f:
            json.dump(empty_data, f)
        
        deps = get_declared_dependencies(str(empty_json))
        
        self.assertEqual(len(deps), 0)


if __name__ == '__main__':
    unittest.main()
