#!/usr/bin/env python3
"""Tests for license_scanner.py RipGrep integration."""

import csv
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from license_scanner import (
    check_ripgrep_available,
    scan_license_headers,
    find_unlicensed_files,
    check_copyleft_licenses,
    generate_license_report,
)


class TestLicenseScanner(unittest.TestCase):
    """Test suite for license scanner."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = tempfile.mkdtemp()
        self.test_path = Path(self.test_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)

    @patch('subprocess.run')
    def test_check_ripgrep_available(self, mock_run):
        """Test RipGrep availability check."""
        mock_run.return_value = MagicMock(returncode=0)
        self.assertTrue(check_ripgrep_available())

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_happy_path(self, mock_check, mock_run):
        """Test license header scanning with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output for Apache license
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout='src/main/java/Foo.java\nsrc/main/java/Bar.java\n'
        )
        
        licenses = scan_license_headers(str(self.test_path))
        
        # Should find licenses
        self.assertIsInstance(licenses, dict)

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_ripgrep_not_available(self, mock_check, mock_run):
        """Test license scanning when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            scan_license_headers(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_invalid_workspace(self, mock_check):
        """Test license scanning with invalid workspace."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            scan_license_headers('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_find_unlicensed_files_happy_path(self, mock_check, mock_run):
        """Test finding unlicensed files."""
        mock_check.return_value = True
        
        # Mock responses for different calls
        def side_effect(*args, **kwargs):
            cmd = args[0]
            if '--files' in cmd:
                # All files
                return MagicMock(
                    returncode=0,
                    stdout='src/main/java/Foo.java\nsrc/main/java/Bar.java\n'
                )
            else:
                # Licensed files
                return MagicMock(
                    returncode=0,
                    stdout='src/main/java/Foo.java\n'
                )
        
        mock_run.side_effect = side_effect
        
        unlicensed = find_unlicensed_files(str(self.test_path))
        
        # Should find one unlicensed file
        self.assertEqual(len(unlicensed), 1)
        self.assertIn('src/main/java/Bar.java', unlicensed)

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_check_copyleft_licenses(self, mock_check, mock_run):
        """Test checking for copyleft licenses."""
        mock_check.return_value = True
        
        # Mock different responses for each license pattern
        def side_effect(*args, **kwargs):
            cmd = args[0]
            pattern = None
            for i, arg in enumerate(cmd):
                if arg in ['--type', '--glob', '--files-with-matches']:
                    continue
                if i > 0 and cmd[i-1] not in ['rg', '--type', '--ignore-case', '--max-count']:
                    pattern = arg
                    break
            
            if pattern and 'GPL' in pattern:
                return MagicMock(returncode=0, stdout='src/main/java/GplFile.java\n')
            else:
                return MagicMock(returncode=0, stdout='')
        
        mock_run.side_effect = side_effect
        
        copyleft = check_copyleft_licenses(str(self.test_path))
        
        # Should find GPL files
        self.assertIsInstance(copyleft, dict)

    @patch('license_scanner.find_unlicensed_files')
    @patch('license_scanner.scan_license_headers')
    def test_generate_license_report(self, mock_scan, mock_unlicensed):
        """Test license report generation."""
        # Mock data
        mock_scan.return_value = {
            'Apache-2.0': ['src/File1.java'],
            'GPL-3.0': ['src/File2.java']
        }
        mock_unlicensed.return_value = ['src/File3.java']
        
        output_file = self.test_path / 'license_report.csv'
        generate_license_report(str(self.test_path), str(output_file))
        
        # Verify CSV was created
        self.assertTrue(output_file.exists())
        
        # Read and verify CSV content
        with open(output_file, 'r') as f:
            reader = csv.reader(f)
            rows = list(reader)
        
        # Should have header + 3 rows
        self.assertEqual(len(rows), 4)
        self.assertEqual(rows[0], ['File', 'License', 'Compliance Status'])

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_timeout(self, mock_check, mock_run):
        """Test license scanning with timeout."""
        mock_check.return_value = True
        
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 30)
        
        with self.assertRaises(RuntimeError) as ctx:
            scan_license_headers(str(self.test_path))
        self.assertIn('timed out', str(ctx.exception))

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_find_unlicensed_files_all_licensed(self, mock_check, mock_run):
        """Test finding unlicensed files when all are licensed."""
        mock_check.return_value = True
        
        # Mock all files are licensed
        def side_effect(*args, **kwargs):
            return MagicMock(
                returncode=0,
                stdout='src/main/java/Foo.java\nsrc/main/java/Bar.java\n'
            )
        
        mock_run.side_effect = side_effect
        
        unlicensed = find_unlicensed_files(str(self.test_path))
        
        # Should find no unlicensed files
        self.assertEqual(len(unlicensed), 0)

    @patch('subprocess.run')
    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_empty_result(self, mock_check, mock_run):
        """Test license scanning with no results."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(returncode=0, stdout='')
        
        licenses = scan_license_headers(str(self.test_path))
        
        # Should return empty dict
        self.assertEqual(len(licenses), 0)

    @patch('license_scanner.check_ripgrep_available')
    def test_scan_license_headers_workspace_is_file(self, mock_check):
        """Test license scanning when workspace is a file."""
        mock_check.return_value = True
        
        test_file = self.test_path / "test.txt"
        test_file.write_text("test")
        
        with self.assertRaises(ValueError) as ctx:
            scan_license_headers(str(test_file))
        self.assertIn('not a directory', str(ctx.exception))


if __name__ == '__main__':
    unittest.main()
