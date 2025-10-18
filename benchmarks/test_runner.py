#!/usr/bin/env python3
"""Tests for benchmark runner."""

import json
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

# Import module under test
import sys
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import runner

# Mock psutil if not available
try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
    # Create a mock psutil for tests
    psutil = MagicMock()
    psutil.Process = MagicMock
    psutil.NoSuchProcess = Exception


class TestBenchmarkResult(unittest.TestCase):
    """Test BenchmarkResult dataclass."""
    
    def test_create_result_with_defaults(self):
        """Test creating result with default timestamp."""
        result = runner.BenchmarkResult(
            tool="bazbom",
            repo_size="small_100_deps",
            dependency_count=100,
            execution_time_seconds=5.2,
            memory_peak_mb=256.5,
            sbom_size_bytes=51200,
            packages_detected=100,
            success=True
        )
        
        self.assertEqual(result.tool, "bazbom")
        self.assertEqual(result.repo_size, "small_100_deps")
        self.assertEqual(result.dependency_count, 100)
        self.assertAlmostEqual(result.execution_time_seconds, 5.2)
        self.assertTrue(result.success)
        self.assertIsNotNone(result.timestamp)
    
    def test_create_result_with_error(self):
        """Test creating result with error message."""
        result = runner.BenchmarkResult(
            tool="syft",
            repo_size="large_2000_deps",
            dependency_count=0,
            execution_time_seconds=0,
            memory_peak_mb=0,
            sbom_size_bytes=0,
            packages_detected=0,
            success=False,
            error_message="Tool not installed"
        )
        
        self.assertFalse(result.success)
        self.assertEqual(result.error_message, "Tool not installed")


class TestBenchmarkRunner(unittest.TestCase):
    """Test BenchmarkRunner class."""
    
    def setUp(self):
        """Set up test fixtures."""
        self.temp_dir = tempfile.mkdtemp()
        self.runner = runner.BenchmarkRunner(self.temp_dir)
    
    def tearDown(self):
        """Clean up test fixtures."""
        import shutil
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
    
    def test_initialization(self):
        """Test runner initialization."""
        self.assertTrue(os.path.exists(self.temp_dir))
        self.assertEqual(self.runner.output_dir, Path(self.temp_dir))
        self.assertEqual(len(self.runner.results), 0)
    
    def test_initialization_creates_directory(self):
        """Test that initialization creates output directory."""
        new_dir = os.path.join(self.temp_dir, "nested", "output")
        new_runner = runner.BenchmarkRunner(new_dir)
        self.assertTrue(os.path.exists(new_dir))
    
    def test_initialization_invalid_directory_raises_error(self):
        """Test initialization with invalid directory."""
        # Create a file where directory should be
        bad_path = os.path.join(self.temp_dir, "file.txt")
        with open(bad_path, 'w') as f:
            f.write("test")
        
        # Try to create directory with same name
        with self.assertRaises(OSError):
            runner.BenchmarkRunner(bad_path)
    
    def test_get_process_memory(self):
        """Test memory tracking."""
        if not HAS_PSUTIL:
            self.skipTest("psutil not available")
        
        import psutil
        process = psutil.Process()
        memory = self.runner._get_process_memory(process)
        
        self.assertIsInstance(memory, float)
        self.assertGreater(memory, 0)  # Should have some memory usage
    
    def test_get_process_memory_no_psutil(self):
        """Test memory tracking without psutil."""
        memory = self.runner._get_process_memory(None)
        self.assertEqual(memory, 0.0)
    
    def test_get_process_memory_no_such_process(self):
        """Test memory tracking with invalid process."""
        if not HAS_PSUTIL:
            self.skipTest("psutil not available")
        
        import psutil
        mock_process = Mock()
        mock_process.memory_info.side_effect = psutil.NoSuchProcess(pid=12345)
        
        memory = self.runner._get_process_memory(mock_process)
        self.assertEqual(memory, 0.0)
    
    def test_find_sbom_not_found(self):
        """Test SBOM discovery when file doesn't exist."""
        result = self.runner._find_sbom("/nonexistent/path")
        self.assertIsNone(result)
    
    def test_find_sbom_found(self):
        """Test SBOM discovery when file exists."""
        # Create mock bazel-bin directory with SBOM
        bazel_bin = os.path.join(self.temp_dir, "bazel-bin")
        os.makedirs(bazel_bin)
        
        sbom_path = os.path.join(bazel_bin, "app.spdx.json")
        with open(sbom_path, 'w') as f:
            json.dump({"packages": []}, f)
        
        result = self.runner._find_sbom(self.temp_dir)
        self.assertIsNotNone(result)
        self.assertTrue(result.endswith(".spdx.json"))
    
    @patch('subprocess.run')
    def test_run_bazbom_success(self, mock_run):
        """Test successful BazBOM benchmark run."""
        # Create test repo
        repo_path = os.path.join(self.temp_dir, "test_repo")
        os.makedirs(repo_path)
        
        # Create mock bazel-bin with SBOM
        bazel_bin = os.path.join(repo_path, "bazel-bin")
        os.makedirs(bazel_bin)
        sbom_path = os.path.join(bazel_bin, "app.spdx.json")
        
        with open(sbom_path, 'w') as f:
            json.dump({
                "packages": [
                    {"name": "pkg1"},
                    {"name": "pkg2"},
                    {"name": "pkg3"}
                ]
            }, f)
        
        # Mock successful subprocess
        mock_run.return_value = Mock(returncode=0, stderr="")
        
        result = self.runner._run_bazbom(repo_path)
        
        self.assertTrue(result.success)
        self.assertEqual(result.tool, "bazbom")
        self.assertEqual(result.packages_detected, 3)
        self.assertGreater(result.execution_time_seconds, 0)
    
    @patch('subprocess.run')
    def test_run_bazbom_build_failure(self, mock_run):
        """Test BazBOM benchmark with build failure."""
        repo_path = os.path.join(self.temp_dir, "test_repo")
        os.makedirs(repo_path)
        
        # Mock failed subprocess
        mock_run.return_value = Mock(
            returncode=1,
            stderr="ERROR: Build failed"
        )
        
        result = self.runner._run_bazbom(repo_path)
        
        self.assertFalse(result.success)
        self.assertIn("Build failed", result.error_message)
    
    def test_run_bazbom_nonexistent_repo(self):
        """Test BazBOM benchmark with nonexistent repository."""
        with self.assertRaises(FileNotFoundError):
            self.runner._run_bazbom("/nonexistent/repo")
    
    @patch('subprocess.run')
    def test_run_syft_not_installed(self, mock_run):
        """Test Syft benchmark when tool not installed."""
        mock_run.side_effect = FileNotFoundError()
        
        repo_path = os.path.join(self.temp_dir, "test_repo")
        os.makedirs(repo_path)
        
        result = self.runner._run_syft(repo_path)
        
        self.assertFalse(result.success)
        self.assertIn("not installed", result.error_message)
    
    @patch('subprocess.run')
    def test_run_syft_success(self, mock_run):
        """Test successful Syft benchmark run."""
        repo_path = os.path.join(self.temp_dir, "test_repo")
        os.makedirs(repo_path)
        
        # Mock syft --version check
        mock_run.side_effect = [
            Mock(returncode=0),  # --version check
            Mock(returncode=0, stderr="")  # actual run
        ]
        
        # Create mock output file
        output_path = self.runner.output_dir / f"syft_{os.path.basename(repo_path)}.json"
        with open(output_path, 'w') as f:
            json.dump({"packages": [{"name": "pkg1"}]}, f)
        
        result = self.runner._run_syft(repo_path)
        
        self.assertTrue(result.success)
        self.assertEqual(result.tool, "syft")
        self.assertEqual(result.packages_detected, 1)
    
    def test_run_benchmarks_invalid_tool(self):
        """Test run_benchmarks with invalid tool name."""
        with self.assertRaises(ValueError) as ctx:
            self.runner.run_benchmarks(["invalid_tool"], ["small_100_deps"])
        
        self.assertIn("Unsupported tool", str(ctx.exception))
    
    def test_run_benchmarks_invalid_repo_size(self):
        """Test run_benchmarks with invalid repo size."""
        with self.assertRaises(ValueError) as ctx:
            self.runner.run_benchmarks(["bazbom"], ["invalid_size"])
        
        self.assertIn("Unsupported repo size", str(ctx.exception))
    
    @patch.object(runner.BenchmarkRunner, '_run_bazbom')
    def test_run_benchmarks_success(self, mock_run_bazbom):
        """Test successful benchmark run."""
        # Create test repo
        repo_path = Path(self.temp_dir) / "benchmarks" / "repos" / "small_100_deps"
        repo_path.mkdir(parents=True)
        
        # Mock successful run
        mock_run_bazbom.return_value = runner.BenchmarkResult(
            tool="bazbom",
            repo_size="small_100_deps",
            dependency_count=100,
            execution_time_seconds=5.0,
            memory_peak_mb=256.0,
            sbom_size_bytes=51200,
            packages_detected=100,
            success=True
        )
        
        # Change to temp directory for test
        original_cwd = os.getcwd()
        try:
            os.chdir(self.temp_dir)
            results = self.runner.run_benchmarks(["bazbom"], ["small_100_deps"])
            
            self.assertEqual(len(results), 1)
            self.assertTrue(results[0].success)
            self.assertEqual(results[0].packages_detected, 100)
        finally:
            os.chdir(original_cwd)
    
    def test_save_results(self):
        """Test saving results to JSON file."""
        # Add some test results
        self.runner.results = [
            runner.BenchmarkResult(
                tool="bazbom",
                repo_size="small_100_deps",
                dependency_count=100,
                execution_time_seconds=5.0,
                memory_peak_mb=256.0,
                sbom_size_bytes=51200,
                packages_detected=100,
                success=True
            )
        ]
        
        filename = "test_results.json"
        self.runner.save_results(filename)
        
        # Verify file exists and is valid JSON
        output_path = self.runner.output_dir / filename
        self.assertTrue(output_path.exists())
        
        with open(output_path, 'r') as f:
            data = json.load(f)
        
        self.assertEqual(len(data), 1)
        self.assertEqual(data[0]["tool"], "bazbom")
        self.assertEqual(data[0]["packages_detected"], 100)
    
    def test_save_results_io_error(self):
        """Test save_results with I/O error."""
        # Try to save to a read-only location
        self.runner.output_dir = Path("/root/cannot_write_here")
        self.runner.results = [
            runner.BenchmarkResult(
                tool="bazbom",
                repo_size="small",
                dependency_count=1,
                execution_time_seconds=1.0,
                memory_peak_mb=100.0,
                sbom_size_bytes=1000,
                packages_detected=1,
                success=True
            )
        ]
        
        with self.assertRaises(IOError):
            self.runner.save_results()
    
    def test_generate_leaderboard(self):
        """Test leaderboard generation."""
        # Add test results for multiple tools and sizes
        self.runner.results = [
            runner.BenchmarkResult(
                tool="bazbom",
                repo_size="small_100_deps",
                dependency_count=100,
                execution_time_seconds=5.0,
                memory_peak_mb=256.0,
                sbom_size_bytes=51200,
                packages_detected=100,
                success=True
            ),
            runner.BenchmarkResult(
                tool="syft",
                repo_size="small_100_deps",
                dependency_count=100,
                execution_time_seconds=7.5,
                memory_peak_mb=512.0,
                sbom_size_bytes=102400,
                packages_detected=100,
                success=True
            )
        ]
        
        filename = "test_leaderboard.md"
        self.runner.generate_leaderboard(filename)
        
        # Verify file exists
        output_path = self.runner.output_dir / filename
        self.assertTrue(output_path.exists())
        
        # Verify content
        with open(output_path, 'r') as f:
            content = f.read()
        
        self.assertIn("# BazBOM Performance Leaderboard", content)
        self.assertIn("small_100_deps", content)
        self.assertIn("bazbom", content)
        self.assertIn("syft", content)
        self.assertIn("Fastest:", content)
    
    def test_generate_leaderboard_empty_results(self):
        """Test leaderboard generation with no results."""
        filename = "empty_leaderboard.md"
        self.runner.generate_leaderboard(filename)
        
        # Should still create file
        output_path = self.runner.output_dir / filename
        self.assertTrue(output_path.exists())
    
    def test_generate_leaderboard_io_error(self):
        """Test leaderboard generation with I/O error."""
        self.runner.output_dir = Path("/root/cannot_write_here")
        
        with self.assertRaises(IOError):
            self.runner.generate_leaderboard()


class TestMainFunction(unittest.TestCase):
    """Test main entry point."""
    
    @patch('sys.argv', ['runner.py', '--tools', 'bazbom', '--sizes', 'small_100_deps'])
    @patch.object(runner.BenchmarkRunner, 'run_benchmarks')
    @patch.object(runner.BenchmarkRunner, 'save_results')
    def test_main_basic(self, mock_save, mock_run):
        """Test main function with basic arguments."""
        mock_run.return_value = []
        
        result = runner.main()
        
        self.assertEqual(result, 0)
        mock_run.assert_called_once()
        mock_save.assert_called_once()
    
    @patch('sys.argv', ['runner.py', '--tools', 'bazbom', '--sizes', 'all', '--leaderboard'])
    @patch.object(runner.BenchmarkRunner, 'run_benchmarks')
    @patch.object(runner.BenchmarkRunner, 'save_results')
    @patch.object(runner.BenchmarkRunner, 'generate_leaderboard')
    def test_main_with_leaderboard(self, mock_leaderboard, mock_save, mock_run):
        """Test main function with leaderboard generation."""
        mock_run.return_value = []
        
        result = runner.main()
        
        self.assertEqual(result, 0)
        mock_leaderboard.assert_called_once()
    
    @patch('sys.argv', ['runner.py', '--tools', 'bazbom'])
    @patch.object(runner.BenchmarkRunner, 'run_benchmarks')
    def test_main_with_exception(self, mock_run):
        """Test main function with exception."""
        mock_run.side_effect = RuntimeError("Test error")
        
        result = runner.main()
        
        self.assertEqual(result, 1)


if __name__ == '__main__':
    unittest.main()
