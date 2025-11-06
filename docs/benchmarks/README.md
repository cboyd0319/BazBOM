# BazBOM Benchmark Suite

Performance benchmarking framework for BazBOM against other SBOM generation tools.

## Overview

The benchmark suite measures:
- **Execution time** - How long it takes to generate SBOMs
- **Memory usage** - Peak memory consumption during generation
- **Accuracy** - Number of packages detected
- **SBOM size** - Size of generated SBOM files

## Supported Tools

- **bazbom** - This tool (BazBOM)
- **syft** - Anchore Syft
- **trivy** - Aqua Security Trivy (planned)
- **cdxgen** - CycloneDX Generator (planned)

## Repository Sizes

Synthetic repositories of varying sizes:

| Size | Dependencies | Description |
|------|--------------|-------------|
| `small_100_deps` | ~100 | Small project (typical library) |
| `medium_500_deps` | ~500 | Medium project (typical application) |
| `large_2000_deps` | ~2000 | Large monorepo |
| `massive_10000_deps` | ~10000 | Enterprise-scale monorepo |

## Usage

### Run All Benchmarks

```bash
# Run BazBOM across all repo sizes
bazel run //benchmarks:runner -- --tools bazbom --sizes all

# Compare BazBOM vs Syft
bazel run //benchmarks:runner -- --tools bazbom syft --sizes all

# Generate leaderboard
bazel run //benchmarks:runner -- --tools bazbom syft --sizes all --leaderboard
```

### Run Specific Benchmarks

```bash
# Test only small repos
bazel run //benchmarks:runner -- --tools bazbom --sizes small_100_deps

# Test specific tools
bazel run //benchmarks:runner -- --tools bazbom syft --sizes medium_500_deps
```

### Output

Results are saved to `benchmarks/results/`:
- `benchmark_results.json` - Detailed JSON results
- `leaderboard.md` - Human-readable markdown leaderboard

## Example Output

```
 Starting BazBOM Benchmark Suite
Tools: bazbom, syft
Sizes: small_100_deps, medium_500_deps

 Benchmarking small_100_deps...
  Running bazbom...  5.23s
  Running syft...  7.45s

 Benchmarking medium_500_deps...
  Running bazbom...  12.67s
  Running syft...  18.92s

 Results saved to benchmarks/results/benchmark_results.json
 Leaderboard generated at benchmarks/results/leaderboard.md

 Benchmark suite complete!
```

## Leaderboard Example

```markdown
# BazBOM Performance Leaderboard

## Summary

### Small 100 Deps

| Tool | Time (s) | Memory (MB) | Packages | SBOM Size (KB) |
|------|----------|-------------|----------|----------------|
| bazbom | 5.23 | 256.5 | 100 | 50.1 |
| syft | 7.45 | 512.3 | 98 | 75.2 |

## Performance Comparison

**small_100_deps:**
-  **Fastest:** bazbom (5.23s)
- syft: 1.4x slower

**medium_500_deps:**
-  **Fastest:** bazbom (12.67s)
- syft: 1.5x slower
```

## Creating Synthetic Repositories

To create test repositories:

```bash
# Create directory structure
mkdir -p benchmarks/repos/small_100_deps

# Create minimal Bazel workspace
cd benchmarks/repos/small_100_deps
cat > WORKSPACE <<EOF
workspace(name = "benchmark_small")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Add dependencies here
EOF

# Create BUILD file with dependencies
cat > BUILD.bazel <<EOF
load("@rules_jvm_external//rules:defs.bzl", "maven_install")

maven_install(
    name = "maven",
    artifacts = [
        # Add ~100 Maven dependencies
        "com.google.guava:guava:31.1-jre",
        # ... more dependencies
    ],
    repositories = [
        "https://repo1.maven.org/maven2",
    ],
)
EOF
```

## Running Tests

```bash
# Run benchmark suite tests
bazel test //benchmarks:test_runner

# Run with coverage
bazel coverage //benchmarks:test_runner --combined_report=lcov
```

## Performance Targets

| Repo Size | BazBOM Target | Industry Average |
|-----------|---------------|------------------|
| Small (100 deps) | < 10s | ~15s |
| Medium (500 deps) | < 30s | ~60s |
| Large (2000 deps) | < 120s | ~300s |
| Massive (10000 deps) | < 600s | ~1800s |

## CI Integration

The benchmark suite can be run in CI to detect performance regressions:

```yaml
# .github/workflows/benchmarks.yml
- name: Run Benchmarks
  run: |
    bazel run //benchmarks:runner -- \
      --tools bazbom \
      --sizes all \
      --leaderboard
    
    # Upload results as artifacts
    - uses: actions/upload-artifact@v3
      with:
        name: benchmark-results
        path: benchmarks/results/
```

## Troubleshooting

### Syft Not Found

If you get "Syft not installed" errors:

```bash
# Install Syft (macOS)
brew install syft

# Install Syft (Linux)
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh -s -- -b /usr/local/bin

# Verify installation
syft --version
```

### Bazel Build Timeout

