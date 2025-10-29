#!/usr/bin/env python3
"""BazBOM Benchmark Suite Runner.

This module provides performance benchmarking for BazBOM against other SBOM tools.
Measures SBOM generation time, memory usage, and accuracy across different repository sizes.
"""

import argparse
import json
import os
import subprocess
import sys
import time
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

try:
    import psutil
    HAS_PSUTIL = True
except ImportError:
    HAS_PSUTIL = False
    print("Warning: psutil not installed. Memory tracking will be limited.", file=sys.stderr)
    print("Install with: pip install psutil", file=sys.stderr)


@dataclass
class BenchmarkResult:
    """Results from a single benchmark run."""
    
    tool: str
    repo_size: str
    dependency_count: int
    execution_time_seconds: float
    memory_peak_mb: float
    sbom_size_bytes: int
    packages_detected: int
    success: bool
    error_message: Optional[str] = None
    timestamp: str = ""
    
    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()


class BenchmarkRunner:
    """Run performance benchmarks for BazBOM and comparison tools."""
    
    SUPPORTED_TOOLS = ["bazbom", "syft", "trivy", "cdxgen"]
    REPO_SIZES = ["small_100_deps", "medium_500_deps", "large_2000_deps", "massive_10000_deps"]
    
    def __init__(self, output_dir: str = "benchmarks/results"):
        """Initialize benchmark runner.
        
        Args:
            output_dir: Directory to store benchmark results
            
        Raises:
            OSError: If output directory cannot be created
        """
        self.output_dir = Path(output_dir)
        try:
            self.output_dir.mkdir(parents=True, exist_ok=True)
        except OSError as e:
            raise OSError(f"Failed to create output directory {output_dir}: {e}")
        
        self.results: List[BenchmarkResult] = []
    
    def _get_process_memory(self, process=None) -> float:
        """Get peak memory usage of a process in MB.
        
        Args:
            process: psutil.Process object (optional, requires psutil)
            
        Returns:
            Peak memory usage in megabytes
        """
        if not HAS_PSUTIL or process is None:
            return 0.0
        
        try:
            mem_info = process.memory_info()
            return mem_info.rss / (1024 * 1024)  # Convert bytes to MB
        except Exception:
            return 0.0
    
    def _run_bazbom(self, repo_path: str) -> BenchmarkResult:
        """Run BazBOM on a test repository.
        
        Args:
            repo_path: Path to test repository
            
        Returns:
            BenchmarkResult with timing and metrics
            
        Raises:
            FileNotFoundError: If repository path doesn't exist
        """
        if not os.path.exists(repo_path):
            raise FileNotFoundError(f"Repository path not found: {repo_path}")
        
        repo_name = os.path.basename(repo_path)
        start_time = time.time()
        peak_memory = 0.0
        
        try:
            # Start memory monitoring
            process = psutil.Process() if HAS_PSUTIL else None
            
            # Run bazel build to generate SBOM
            cmd = ["bazel", "build", "//:sbom_all"]
            result = subprocess.run(
                cmd,
                cwd=repo_path,
                capture_output=True,
                text=True,
                timeout=600  # 10 minute timeout
            )
            
            # Track peak memory
            peak_memory = self._get_process_memory(process)
            
            execution_time = time.time() - start_time
            
            if result.returncode != 0:
                return BenchmarkResult(
                    tool="bazbom",
                    repo_size=repo_name,
                    dependency_count=0,
                    execution_time_seconds=execution_time,
                    memory_peak_mb=peak_memory,
                    sbom_size_bytes=0,
                    packages_detected=0,
                    success=False,
                    error_message=result.stderr[:500]
                )
            
            # Find generated SBOM
            sbom_path = self._find_sbom(repo_path)
            if not sbom_path:
                return BenchmarkResult(
                    tool="bazbom",
                    repo_size=repo_name,
                    dependency_count=0,
                    execution_time_seconds=execution_time,
                    memory_peak_mb=peak_memory,
                    sbom_size_bytes=0,
                    packages_detected=0,
                    success=False,
                    error_message="SBOM file not found after build"
                )
            
            # Parse SBOM to count packages
            with open(sbom_path, 'r', encoding='utf-8') as f:
                sbom_data = json.load(f)
            
            sbom_size = os.path.getsize(sbom_path)
            packages_detected = len(sbom_data.get("packages", []))
            
            return BenchmarkResult(
                tool="bazbom",
                repo_size=repo_name,
                dependency_count=packages_detected,
                execution_time_seconds=execution_time,
                memory_peak_mb=peak_memory,
                sbom_size_bytes=sbom_size,
                packages_detected=packages_detected,
                success=True
            )
            
        except subprocess.TimeoutExpired:
            return BenchmarkResult(
                tool="bazbom",
                repo_size=repo_name,
                dependency_count=0,
                execution_time_seconds=600.0,
                memory_peak_mb=peak_memory,
                sbom_size_bytes=0,
                packages_detected=0,
                success=False,
                error_message="Timeout after 600 seconds"
            )
        except Exception as e:
            return BenchmarkResult(
                tool="bazbom",
                repo_size=repo_name,
                dependency_count=0,
                execution_time_seconds=time.time() - start_time,
                memory_peak_mb=peak_memory,
                sbom_size_bytes=0,
                packages_detected=0,
                success=False,
                error_message=str(e)[:500]
            )
    
    def _find_sbom(self, repo_path: str) -> Optional[str]:
        """Find generated SBOM file in bazel-bin directory.
        
        Args:
            repo_path: Path to repository
            
        Returns:
            Path to SBOM file or None if not found
        """
        bazel_bin = os.path.join(repo_path, "bazel-bin")
        if not os.path.exists(bazel_bin):
            return None
        
        # Search for .spdx.json files
        for root, dirs, files in os.walk(bazel_bin):
            for file in files:
                if file.endswith(".spdx.json"):
                    return os.path.join(root, file)
        
        return None
    
    def _run_syft(self, repo_path: str) -> BenchmarkResult:
        """Run Syft on a test repository (if available).
        
        Args:
            repo_path: Path to test repository
            
        Returns:
            BenchmarkResult with timing and metrics
        """
        repo_name = os.path.basename(repo_path)
        
        # Check if syft is installed
        try:
            subprocess.run(["syft", "--version"], capture_output=True, check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            return BenchmarkResult(
                tool="syft",
                repo_size=repo_name,
                dependency_count=0,
                execution_time_seconds=0,
                memory_peak_mb=0,
                sbom_size_bytes=0,
                packages_detected=0,
                success=False,
                error_message="Syft not installed or not in PATH"
            )
        
        start_time = time.time()
        output_path = self.output_dir / f"syft_{repo_name}.json"
        
        try:
            process = psutil.Process() if HAS_PSUTIL else None
            
            cmd = ["syft", repo_path, "-o", "spdx-json", f"--file={output_path}"]
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=600
            )
            
            peak_memory = self._get_process_memory(process)
            execution_time = time.time() - start_time
            
            if result.returncode != 0:
                return BenchmarkResult(
                    tool="syft",
                    repo_size=repo_name,
                    dependency_count=0,
                    execution_time_seconds=execution_time,
                    memory_peak_mb=peak_memory,
                    sbom_size_bytes=0,
                    packages_detected=0,
                    success=False,
                    error_message=result.stderr[:500]
                )
            
            # Parse output
            with open(output_path, 'r', encoding='utf-8') as f:
                sbom_data = json.load(f)
            
            sbom_size = os.path.getsize(output_path)
            packages_detected = len(sbom_data.get("packages", []))
            
            return BenchmarkResult(
                tool="syft",
                repo_size=repo_name,
                dependency_count=packages_detected,
                execution_time_seconds=execution_time,
                memory_peak_mb=peak_memory,
                sbom_size_bytes=sbom_size,
                packages_detected=packages_detected,
                success=True
            )
            
        except Exception as e:
            return BenchmarkResult(
                tool="syft",
                repo_size=repo_name,
                dependency_count=0,
                execution_time_seconds=time.time() - start_time,
                memory_peak_mb=0,
                sbom_size_bytes=0,
                packages_detected=0,
                success=False,
                error_message=str(e)[:500]
            )
    
    def run_benchmarks(self, tools: List[str], repo_sizes: List[str]) -> List[BenchmarkResult]:
        """Run benchmarks for specified tools and repository sizes.
        
        Args:
            tools: List of tool names to benchmark
            repo_sizes: List of repository size categories
            
        Returns:
            List of BenchmarkResult objects
            
        Raises:
            ValueError: If invalid tool or repo_size specified
        """
        for tool in tools:
            if tool not in self.SUPPORTED_TOOLS:
                raise ValueError(f"Unsupported tool: {tool}. Supported: {self.SUPPORTED_TOOLS}")
        
        for size in repo_sizes:
            if size not in self.REPO_SIZES:
                raise ValueError(f"Unsupported repo size: {size}. Supported: {self.REPO_SIZES}")
        
        results = []
        
        for repo_size in repo_sizes:
            repo_path = Path(f"benchmarks/repos/{repo_size}")
            
            if not repo_path.exists():
                print(f"Warning: Repository {repo_size} not found, skipping...")
                continue
            
            print(f"\nBenchmarking {repo_size}...")
            
            for tool in tools:
                print(f"  Running {tool}...", end=" ", flush=True)
                
                if tool == "bazbom":
                    result = self._run_bazbom(str(repo_path))
                elif tool == "syft":
                    result = self._run_syft(str(repo_path))
                else:
                    # Other tools not implemented yet
                    result = BenchmarkResult(
                        tool=tool,
                        repo_size=repo_size,
                        dependency_count=0,
                        execution_time_seconds=0,
                        memory_peak_mb=0,
                        sbom_size_bytes=0,
                        packages_detected=0,
                        success=False,
                        error_message=f"{tool} runner not implemented"
                    )
                
                results.append(result)
                self.results.append(result)
                
                if result.success:
                    print(f"✅ {result.execution_time_seconds:.2f}s")
                else:
                    print(f"❌ {result.error_message[:50]}")
        
        return results
    
    def save_results(self, filename: str = "benchmark_results.json") -> None:
        """Save benchmark results to JSON file.
        
        Args:
            filename: Output filename
            
        Raises:
            IOError: If file cannot be written
        """
        output_path = self.output_dir / filename
        
        try:
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(
                    [asdict(r) for r in self.results],
                    f,
                    indent=2
                )
            print(f"\n💾 Results saved to {output_path}")
        except IOError as e:
            raise IOError(f"Failed to write results to {output_path}: {e}")
    
    def generate_leaderboard(self, filename: str = "leaderboard.md") -> None:
        """Generate markdown leaderboard from results.
        
        Args:
            filename: Output filename
            
        Raises:
            IOError: If file cannot be written
        """
        output_path = self.output_dir / filename
        
        # Group results by repo size
        by_size: Dict[str, List[BenchmarkResult]] = {}
        for result in self.results:
            if result.success:
                by_size.setdefault(result.repo_size, []).append(result)
        
        try:
            with open(output_path, 'w', encoding='utf-8') as f:
                f.write("# BazBOM Performance Leaderboard\n\n")
                f.write(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
                f.write("## Summary\n\n")
                
                for repo_size in self.REPO_SIZES:
                    if repo_size not in by_size:
                        continue
                    
                    f.write(f"\n### {repo_size.replace('_', ' ').title()}\n\n")
                    f.write("| Tool | Time (s) | Memory (MB) | Packages | SBOM Size (KB) |\n")
                    f.write("|------|----------|-------------|----------|----------------|\n")
                    
                    # Sort by execution time
                    sorted_results = sorted(by_size[repo_size], key=lambda r: r.execution_time_seconds)
                    
                    for result in sorted_results:
                        sbom_kb = result.sbom_size_bytes / 1024
                        f.write(
                            f"| {result.tool} | {result.execution_time_seconds:.2f} | "
                            f"{result.memory_peak_mb:.1f} | {result.packages_detected} | "
                            f"{sbom_kb:.1f} |\n"
                        )
                
                # Add performance comparison
                f.write("\n## Performance Comparison\n\n")
                
                for repo_size in self.REPO_SIZES:
                    if repo_size not in by_size or len(by_size[repo_size]) < 2:
                        continue
                    
                    sorted_results = sorted(by_size[repo_size], key=lambda r: r.execution_time_seconds)
                    fastest = sorted_results[0]
                    
                    f.write(f"\n**{repo_size}:**\n")
                    f.write(f"- **Fastest:** {fastest.tool} ({fastest.execution_time_seconds:.2f}s)\n")
                    
                    # Compare others to fastest
                    for result in sorted_results[1:]:
                        speedup = result.execution_time_seconds / fastest.execution_time_seconds
                        f.write(f"- {result.tool}: {speedup:.1f}x slower\n")
            
            print(f"Leaderboard generated at {output_path}")
            
        except IOError as e:
            raise IOError(f"Failed to write leaderboard to {output_path}: {e}")


def main():
    """Main entry point for benchmark runner."""
    parser = argparse.ArgumentParser(
        description="BazBOM Performance Benchmark Suite",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Run all benchmarks
  python runner.py --tools bazbom syft --sizes all

  # Run BazBOM only on small repo
  python runner.py --tools bazbom --sizes small_100_deps

  # Run and generate leaderboard
  python runner.py --tools bazbom syft --sizes all --leaderboard
        """
    )
    
    parser.add_argument(
        "--tools",
        nargs="+",
        default=["bazbom"],
        choices=BenchmarkRunner.SUPPORTED_TOOLS,
        help="Tools to benchmark (default: bazbom)"
    )
    
    parser.add_argument(
        "--sizes",
        nargs="+",
        default=["all"],
        help="Repository sizes to test (default: all)"
    )
    
    parser.add_argument(
        "--output-dir",
        default="benchmarks/results",
        help="Output directory for results (default: benchmarks/results)"
    )
    
    parser.add_argument(
        "--leaderboard",
        action="store_true",
        help="Generate leaderboard markdown"
    )
    
    args = parser.parse_args()
    
    # Handle "all" sizes
    if "all" in args.sizes:
        sizes = BenchmarkRunner.REPO_SIZES
    else:
        sizes = args.sizes
    
    try:
        runner = BenchmarkRunner(args.output_dir)
        
        print("Starting BazBOM Benchmark Suite")
        print(f"Tools: {', '.join(args.tools)}")
        print(f"Sizes: {', '.join(sizes)}")
        
        runner.run_benchmarks(args.tools, sizes)
        runner.save_results()
        
        if args.leaderboard:
            runner.generate_leaderboard()
        
        print("\n✅ Benchmark suite complete!")
        
        return 0
        
    except Exception as e:
        print(f"\n❌ Error: {e}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
