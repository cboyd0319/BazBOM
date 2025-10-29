#!/usr/bin/env python3
"""Tests for Bazel query support module."""

import json
import subprocess
import unittest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

import sys
sys.path.insert(0, str(Path(__file__).parent.parent))

from bazel_query import (
    run_bazel_query,
    find_targets_by_kind,
    find_rdeps,
    find_affected_by_files,
    scan_targets_incremental,
    validate_query,
)


class TestBazelQuery(unittest.TestCase):
    """Test Bazel query functionality."""
    
    def setUp(self):
        self.workspace = Path("/fake/workspace")
    
    @patch('subprocess.run')
    def test_run_bazel_query_success(self, mock_run):
        """Test successful Bazel query execution."""
        mock_run.return_value = Mock(
            stdout="//src/java:target1\n//src/java:target2\n",
            returncode=0
        )
        
        result = run_bazel_query("kind(java_binary, //...)", self.workspace)
        
        self.assertEqual(result, ["//src/java:target1", "//src/java:target2"])
        mock_run.assert_called_once()
        call_args = mock_run.call_args
        self.assertEqual(call_args[0][0][0], "bazel")
        self.assertEqual(call_args[0][0][1], "query")
        self.assertIn("kind(java_binary, //...)", call_args[0][0])
    
    @patch('subprocess.run')
    def test_run_bazel_query_failure(self, mock_run):
        """Test Bazel query failure handling."""
        mock_run.side_effect = subprocess.CalledProcessError(
            1, ["bazel", "query"], stderr="Query failed"
        )
        
        with self.assertRaises(subprocess.CalledProcessError):
            run_bazel_query("invalid query", self.workspace)
    
    @patch('subprocess.run')
    def test_run_bazel_query_timeout(self, mock_run):
        """Test Bazel query timeout handling."""
        mock_run.side_effect = subprocess.TimeoutExpired(
            ["bazel", "query"], timeout=60
        )
        
        with self.assertRaises(subprocess.TimeoutExpired):
            run_bazel_query("slow query", self.workspace, timeout=60)
    
    @patch('bazel_query.run_bazel_query')
    def test_find_targets_by_kind(self, mock_query):
        """Test finding targets by kind."""
        mock_query.return_value = ["//src/java:app", "//src/java:lib"]
        
        result = find_targets_by_kind("java_binary", "//src/java/...", self.workspace)
        
        self.assertEqual(result, ["//src/java:app", "//src/java:lib"])
        mock_query.assert_called_once_with(
            "kind(java_binary, //src/java/...)",
            self.workspace
        )
    
    @patch('bazel_query.run_bazel_query')
    def test_find_rdeps_without_depth(self, mock_query):
        """Test finding reverse dependencies without depth limit."""
        mock_query.return_value = ["//src/java:app"]
        
        result = find_rdeps("//...", "//src/java:lib", self.workspace)
        
        self.assertEqual(result, ["//src/java:app"])
        mock_query.assert_called_once()
        call_query = mock_query.call_args[0][0]
        self.assertIn("rdeps(//..., //src/java:lib)", call_query)
        # Verify no depth parameter (query should not have a third comma-separated argument)
        self.assertEqual(call_query, "rdeps(//..., //src/java:lib)")
    
    @patch('bazel_query.run_bazel_query')
    def test_find_rdeps_with_depth(self, mock_query):
        """Test finding reverse dependencies with depth limit."""
        mock_query.return_value = ["//src/java:app"]
        
        result = find_rdeps("//...", "//src/java:lib", self.workspace, depth=1)
        
        self.assertEqual(result, ["//src/java:app"])
        call_query = mock_query.call_args[0][0]
        self.assertIn("rdeps(//..., //src/java:lib, 1)", call_query)
    
    @patch('bazel_query.find_rdeps')
    def test_find_affected_by_files_single_file(self, mock_rdeps):
        """Test finding affected targets by single file."""
        mock_rdeps.return_value = ["//src/java:app"]
        
        result = find_affected_by_files(
            ["src/java/lib/Utils.java"],
            "//...",
            self.workspace
        )
        
        self.assertEqual(result, ["//src/java:app"])
        mock_rdeps.assert_called_once()
        # Check that file set was created correctly
        call_args = mock_rdeps.call_args
        self.assertIn("set(", call_args[0][1])
        self.assertIn("src/java/lib/Utils.java", call_args[0][1])
    
    @patch('bazel_query.find_rdeps')
    def test_find_affected_by_files_multiple_files(self, mock_rdeps):
        """Test finding affected targets by multiple files."""
        mock_rdeps.return_value = ["//src/java:app", "//src/java:lib"]
        
        files = [
            "src/java/lib/Utils.java",
            "src/java/lib/Helper.java",
        ]
        result = find_affected_by_files(files, "//...", self.workspace)
        
        self.assertEqual(result, ["//src/java:app", "//src/java:lib"])
        call_args = mock_rdeps.call_args
        target_set = call_args[0][1]
        self.assertIn("Utils.java", target_set)
        self.assertIn("Helper.java", target_set)
    
    @patch('bazel_query.find_rdeps')
    def test_find_affected_by_files_empty_list(self, mock_rdeps):
        """Test finding affected targets with empty file list."""
        result = find_affected_by_files([], "//...", self.workspace)
        
        self.assertEqual(result, [])
        mock_rdeps.assert_not_called()
    
    @patch('bazel_query.run_bazel_query')
    @patch('bazel_query.find_affected_by_files')
    def test_scan_targets_incremental_no_filter(self, mock_affected, mock_query):
        """Test incremental scan without kind filter."""
        mock_affected.return_value = ["//src/java:app", "//src/java:lib"]
        
        files = ["src/java/Utils.java"]
        result = scan_targets_incremental(files, workspace_path=self.workspace)
        
        self.assertEqual(result, ["//src/java:app", "//src/java:lib"])
        mock_affected.assert_called_once_with(files, "//...", self.workspace)
        mock_query.assert_not_called()  # No filter, so no additional query
    
    @patch('bazel_query.run_bazel_query')
    @patch('bazel_query.find_affected_by_files')
    def test_scan_targets_incremental_with_filter(self, mock_affected, mock_query):
        """Test incremental scan with kind filter."""
        mock_affected.return_value = ["//src/java:app", "//src/java:lib"]
        mock_query.return_value = ["//src/java:app"]  # Only app is java_binary
        
        files = ["src/java/Utils.java"]
        result = scan_targets_incremental(
            files,
            target_kind="java_binary",
            workspace_path=self.workspace
        )
        
        self.assertEqual(result, ["//src/java:app"])
        mock_affected.assert_called_once()
        mock_query.assert_called_once()
        # Verify kind filter was applied
        call_query = mock_query.call_args[0][0]
        self.assertIn("kind(java_binary,", call_query)
    
    @patch('bazel_query.run_bazel_query')
    @patch('bazel_query.find_affected_by_files')
    def test_scan_targets_incremental_no_affected(self, mock_affected, mock_query):
        """Test incremental scan when no targets are affected."""
        mock_affected.return_value = []
        
        files = ["README.md"]
        result = scan_targets_incremental(files, workspace_path=self.workspace)
        
        self.assertEqual(result, [])
        mock_affected.assert_called_once()
        mock_query.assert_not_called()  # No targets, so no filter query
    
    @patch('subprocess.run')
    def test_validate_query_valid(self, mock_run):
        """Test query validation with valid query."""
        mock_run.return_value = Mock(returncode=0)
        
        result = validate_query("kind(java_binary, //...)", self.workspace)
        
        self.assertTrue(result)
        mock_run.assert_called_once()
    
    @patch('subprocess.run')
    def test_validate_query_invalid(self, mock_run):
        """Test query validation with invalid query."""
        mock_run.side_effect = subprocess.CalledProcessError(1, ["bazel"])
        
        result = validate_query("invalid syntax", self.workspace)
        
        self.assertFalse(result)
    
    @patch('subprocess.run')
    def test_validate_query_timeout(self, mock_run):
        """Test query validation timeout."""
        mock_run.side_effect = subprocess.TimeoutExpired(["bazel"], 10)
        
        result = validate_query("slow query", self.workspace)
        
        self.assertFalse(result)


class TestBazelQueryCLI(unittest.TestCase):
    """Test Bazel query CLI."""
    
    @patch('bazel_query.run_bazel_query')
    def test_cli_query_mode(self, mock_query):
        """Test CLI with --query flag."""
        mock_query.return_value = ["//src:target"]
        
        with patch('sys.argv', ['bazel_query.py', '--query', 'kind(java_binary, //...)']):
            with patch('builtins.print') as mock_print:
                from bazel_query import main
                result = main()
                
                self.assertEqual(result, 0)
                mock_query.assert_called_once()
    
    @patch('bazel_query.find_targets_by_kind')
    def test_cli_kind_mode(self, mock_find):
        """Test CLI with --kind flag."""
        mock_find.return_value = ["//src:app"]
        
        with patch('sys.argv', ['bazel_query.py', '--kind', 'java_binary']):
            with patch('builtins.print'):
                from bazel_query import main
                result = main()
                
                self.assertEqual(result, 0)
                mock_find.assert_called_once()
    
    @patch('bazel_query.scan_targets_incremental')
    def test_cli_affected_mode(self, mock_scan):
        """Test CLI with --affected-by-files flag."""
        mock_scan.return_value = ["//src:app"]
        
        with patch('sys.argv', [
            'bazel_query.py',
            '--affected-by-files', 'file1.java', 'file2.java',
            '--filter-kind', 'java_binary'
        ]):
            with patch('builtins.print'):
                from bazel_query import main
                result = main()
                
                self.assertEqual(result, 0)
                mock_scan.assert_called_once()
                call_args = mock_scan.call_args
                self.assertEqual(call_args[0][0], ['file1.java', 'file2.java'])
                self.assertEqual(call_args[1]['target_kind'], 'java_binary')


if __name__ == '__main__':
    unittest.main()
