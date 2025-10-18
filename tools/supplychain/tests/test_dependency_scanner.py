#!/usr/bin/env python3
"""Tests for dependency_scanner.py RipGrep integration."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from dependency_scanner import (
    check_ripgrep_available,
    find_maven_dependencies,
    find_gradle_dependencies,
    find_bazel_maven_jars,
)


class TestDependencyScanner(unittest.TestCase):
    """Test suite for dependency scanner."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = tempfile.mkdtemp()
        self.test_path = Path(self.test_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)

    @patch('subprocess.run')
    def test_check_ripgrep_available_when_installed(self, mock_run):
        """Test RipGrep availability check when installed."""
        mock_run.return_value = MagicMock(returncode=0)
        self.assertTrue(check_ripgrep_available())

    @patch('subprocess.run')
    def test_check_ripgrep_available_when_not_installed(self, mock_run):
        """Test RipGrep availability check when not installed."""
        mock_run.side_effect = FileNotFoundError()
        self.assertFalse(check_ripgrep_available())

    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_invalid_workspace(self, mock_check):
        """Test Maven dependency discovery with invalid workspace."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            find_maven_dependencies('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_happy_path(self, mock_check, mock_run):
        """Test Maven dependency discovery with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output
        mock_output = '''{"group": "com.google.guava", "artifact": "guava", "version": "31.1-jre"}
{"group": "org.slf4j", "artifact": "slf4j-api", "version": "1.7.36"}'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        deps = find_maven_dependencies(str(self.test_path))
        
        self.assertEqual(len(deps), 2)
        self.assertEqual(deps[0]['group'], 'com.google.guava')
        self.assertEqual(deps[0]['artifact'], 'guava')
        self.assertEqual(deps[0]['version'], '31.1-jre')

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_ripgrep_not_available(self, mock_check, mock_run):
        """Test Maven dependency discovery when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            find_maven_dependencies(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_gradle_dependencies_happy_path(self, mock_check, mock_run):
        """Test Gradle dependency discovery with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output
        mock_output = '''com.google.guava:guava:31.1-jre
org.slf4j:slf4j-api:1.7.36'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        deps = find_gradle_dependencies(str(self.test_path))
        
        self.assertEqual(len(deps), 2)
        self.assertIn('com.google.guava:guava:31.1-jre', deps)

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_bazel_maven_jars_happy_path(self, mock_check, mock_run):
        """Test Bazel Maven JAR discovery with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output with duplicates
        mock_output = '''@maven//:com_google_guava_guava
@maven//:org_slf4j_slf4j_api
@maven//:com_google_guava_guava'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        refs = find_bazel_maven_jars(str(self.test_path))
        
        # Should deduplicate
        self.assertEqual(len(refs), 2)
        self.assertIn('@maven//:com_google_guava_guava', refs)
        self.assertIn('@maven//:org_slf4j_slf4j_api', refs)

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_timeout(self, mock_check, mock_run):
        """Test Maven dependency discovery with timeout."""
        mock_check.return_value = True
        
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 30)
        
        with self.assertRaises(RuntimeError) as ctx:
            find_maven_dependencies(str(self.test_path))
        self.assertIn('timed out', str(ctx.exception))

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_malformed_json(self, mock_check, mock_run):
        """Test Maven dependency discovery with malformed JSON."""
        mock_check.return_value = True
        
        # Mock RipGrep output with malformed JSON
        mock_output = '''{"group": "com.google.guava", "artifact": "guava", "version": "31.1-jre"}
{"invalid json
{"group": "org.slf4j", "artifact": "slf4j-api", "version": "1.7.36"}'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        deps = find_maven_dependencies(str(self.test_path))
        
        # Should skip malformed JSON
        self.assertEqual(len(deps), 2)

    @patch('subprocess.run')
    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_empty_result(self, mock_check, mock_run):
        """Test Maven dependency discovery with no results."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=''
        )
        
        deps = find_maven_dependencies(str(self.test_path))
        
        self.assertEqual(len(deps), 0)

    @patch('dependency_scanner.check_ripgrep_available')
    def test_find_maven_dependencies_workspace_is_file(self, mock_check):
        """Test Maven dependency discovery when workspace is a file."""
        mock_check.return_value = True
        
        test_file = self.test_path / "test.txt"
        test_file.write_text("test")
        
        with self.assertRaises(ValueError) as ctx:
            find_maven_dependencies(str(test_file))
        self.assertIn('not a directory', str(ctx.exception))


if __name__ == '__main__':
    unittest.main()
