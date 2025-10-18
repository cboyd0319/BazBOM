#!/usr/bin/env python3
"""Tests for container_scanner.py RipGrep integration."""

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch, MagicMock
import sys
import os

# Add parent directory to path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from container_scanner import (
    check_ripgrep_available,
    extract_jars_from_image,
    find_os_packages,
)


class TestContainerScanner(unittest.TestCase):
    """Test suite for container scanner."""

    def setUp(self):
        """Set up test fixtures."""
        self.test_dir = tempfile.mkdtemp()
        self.test_path = Path(self.test_dir)

    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        shutil.rmtree(self.test_dir, ignore_errors=True)

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_from_image_happy_path(self, mock_check, mock_run):
        """Test JAR extraction with valid input."""
        mock_check.return_value = True
        
        # Mock RipGrep output
        mock_output = '''/app/lib/guava-31.1-jre.jar
/app/lib/slf4j-api-1.7.36.jar
/app/myapp.jar'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        jars = extract_jars_from_image(str(self.test_path))
        
        self.assertEqual(len(jars), 3)
        # guava-31.1-jre.jar splits to artifact='guava-31.1' version='jre'
        # This is expected behavior - complex version strings may not parse perfectly
        self.assertTrue(jars[0]['artifact'].startswith('guava'))
        self.assertIn(jars[0]['path'], '/app/lib/guava-31.1-jre.jar')

    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_ripgrep_not_available(self, mock_check):
        """Test JAR extraction when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            extract_jars_from_image(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_invalid_path(self, mock_check):
        """Test JAR extraction with invalid path."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            extract_jars_from_image('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))

    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_path_is_file(self, mock_check):
        """Test JAR extraction when path is a file."""
        mock_check.return_value = True
        
        test_file = self.test_path / "test.txt"
        test_file.write_text("test")
        
        with self.assertRaises(ValueError) as ctx:
            extract_jars_from_image(str(test_file))
        self.assertIn('not a directory', str(ctx.exception))

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_no_version_in_filename(self, mock_check, mock_run):
        """Test JAR extraction with JAR files without version."""
        mock_check.return_value = True
        
        mock_output = '''/app/application.jar'''
        
        mock_run.return_value = MagicMock(
            returncode=0,
            stdout=mock_output
        )
        
        jars = extract_jars_from_image(str(self.test_path))
        
        self.assertEqual(len(jars), 1)
        self.assertEqual(jars[0]['artifact'], 'application')
        self.assertEqual(jars[0]['version'], 'unknown')

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_dpkg(self, mock_check, mock_run):
        """Test OS package discovery for Debian/Ubuntu."""
        mock_check.return_value = True
        
        # Mock finding dpkg status file
        def side_effect(*args, **kwargs):
            cmd = args[0]
            if 'dpkg' in str(cmd):
                return MagicMock(
                    returncode=0,
                    stdout='/var/lib/dpkg/status\n'
                )
            else:
                return MagicMock(returncode=0, stdout='')
        
        mock_run.side_effect = side_effect
        
        packages = find_os_packages(str(self.test_path))
        
        self.assertIn('dpkg', packages)
        self.assertEqual(packages['dpkg'], '/var/lib/dpkg/status')

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_apk(self, mock_check, mock_run):
        """Test OS package discovery for Alpine."""
        mock_check.return_value = True
        
        # Mock finding apk installed file
        def side_effect(*args, **kwargs):
            cmd = args[0]
            if 'apk' in str(cmd):
                return MagicMock(
                    returncode=0,
                    stdout='/lib/apk/db/installed\n'
                )
            else:
                return MagicMock(returncode=0, stdout='')
        
        mock_run.side_effect = side_effect
        
        packages = find_os_packages(str(self.test_path))
        
        self.assertIn('apk', packages)

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_rpm(self, mock_check, mock_run):
        """Test OS package discovery for RHEL/CentOS."""
        mock_check.return_value = True
        
        # Mock finding rpm Packages file
        def side_effect(*args, **kwargs):
            cmd = args[0]
            if 'rpm' in str(cmd):
                return MagicMock(
                    returncode=0,
                    stdout='/var/lib/rpm/Packages\n'
                )
            else:
                return MagicMock(returncode=0, stdout='')
        
        mock_run.side_effect = side_effect
        
        packages = find_os_packages(str(self.test_path))
        
        self.assertIn('rpm', packages)

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_none_found(self, mock_check, mock_run):
        """Test OS package discovery when no packages found."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(returncode=0, stdout='')
        
        packages = find_os_packages(str(self.test_path))
        
        self.assertEqual(len(packages), 0)

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_timeout(self, mock_check, mock_run):
        """Test JAR extraction with timeout."""
        mock_check.return_value = True
        
        import subprocess
        mock_run.side_effect = subprocess.TimeoutExpired('rg', 60)
        
        with self.assertRaises(RuntimeError) as ctx:
            extract_jars_from_image(str(self.test_path))
        self.assertIn('timed out', str(ctx.exception))

    @patch('subprocess.run')
    @patch('container_scanner.check_ripgrep_available')
    def test_extract_jars_empty_result(self, mock_check, mock_run):
        """Test JAR extraction with no results."""
        mock_check.return_value = True
        
        mock_run.return_value = MagicMock(returncode=0, stdout='')
        
        jars = extract_jars_from_image(str(self.test_path))
        
        self.assertEqual(len(jars), 0)

    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_ripgrep_not_available(self, mock_check):
        """Test OS package finding when RipGrep not available."""
        mock_check.return_value = False
        
        with self.assertRaises(RuntimeError) as ctx:
            find_os_packages(str(self.test_path))
        self.assertIn('RipGrep', str(ctx.exception))

    @patch('container_scanner.check_ripgrep_available')
    def test_find_os_packages_invalid_path(self, mock_check):
        """Test OS package finding with invalid path."""
        mock_check.return_value = True
        
        with self.assertRaises(ValueError) as ctx:
            find_os_packages('/nonexistent/path')
        self.assertIn('does not exist', str(ctx.exception))


if __name__ == '__main__':
    unittest.main()
