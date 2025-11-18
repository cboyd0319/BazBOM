---
name: performance-profiler
description: Analyzes BazBOM scan performance, identifies bottlenecks, and provides optimization recommendations. Use when scans are slow, investigating performance regressions, or optimizing for large repositories.
---

# Performance Profiler Skill

Analyzes execution time, memory usage, and identifies performance bottlenecks in BazBOM scans.

## When to Use

Activate when you hear:
- "Why is this scan so slow?"
- "Performance test"
- "How fast should this be?"
- "Memory usage too high"
- "Optimize the scan"

## Performance Metrics

### Expected Performance Baselines

| Repository Size | Packages | Expected Time | Expected Memory |
|----------------|----------|---------------|-----------------|
| Small (<10MB) | <100 | <1s | ~50MB |
| Medium (10-50MB) | 100-1K | 1-3s | ~100MB |
| Large (50-100MB) | 1K-5K | 3-10s | ~150MB |
| Huge (>100MB) | 5K+ | 10-30s | ~200MB |

*Baselines for Apple Silicon M1/M2*

### Profiling Commands

#### Basic Timing
```bash
# macOS time with memory stats
/usr/bin/time -l bazbom scan . 2>&1

# Key metrics:
#   - real time (wall clock)
#   - user time (CPU)
#   - maximum resident set size (memory)
```

#### Detailed Profiling
```bash
# With debug logging
RUST_LOG=debug /usr/bin/time -l bazbom scan . 2>&1 | tee scan-profile.log

# Trace-level logging (very verbose)
RUST_LOG=trace /usr/bin/time -l bazbom scan . 2>&1 | tee scan-trace.log

# Specific module profiling
RUST_LOG=bazbom::bazel=debug /usr/bin/time -l bazbom scan .
```

#### Comparative Profiling
```bash
# Compare different scan modes
for mode in quick check scan full; do
    echo "=== Testing $mode mode ==="
    /usr/bin/time -l bazbom $mode . 2>&1 | grep -E "real|maximum"
done

# Progressive limits
for limit in 10 50 100 500 1000; do
    echo "=== Limit: $limit ==="
    /usr/bin/time -l bazbom full --limit $limit . 2>&1 | grep -E "real|maximum"
done
```

## Bottleneck Detection

### Common Bottlenecks

#### 1. maven_install.json Parsing
**Symptom:** Slow startup, high CPU during initial phase
**Detection:**
```bash
RUST_LOG=bazbom::bazel=debug bazbom scan . 2>&1 | grep -E "parsing|extracting"
```
**Optimization:** Consider caching parsed results

#### 2. SBOM Generation
**Symptom:** Slow after dependency extraction
**Detection:**
```bash
RUST_LOG=bazbom_formats=debug bazbom scan . 2>&1 | grep -E "spdx|cyclonedx"
```
**Optimization:** Stream output instead of building in memory

#### 3. Polyglot Scanning
**Symptom:** Delays when scanning non-Java ecosystems
**Detection:**
```bash
RUST_LOG=bazbom_polyglot=debug bazbom scan . 2>&1
```
**Optimization:** Parallel ecosystem scanning

#### 4. Reachability Analysis
**Symptom:** Very slow, high memory usage
**Detection:**
```bash
# Check if auto-enabled
bazbom scan . 2>&1 | grep "reachability"
```
**Optimization:**
```bash
# Disable if not needed
export BAZBOM_NO_SMART_DEFAULTS=1
bazbom scan .
```

## Performance Analysis Workflow

### 1. Baseline Measurement
```bash
# Clean run
cd /path/to/repo
/usr/bin/time -l bazbom scan . 2>&1 | tee baseline.txt

# Extract metrics
grep "real" baseline.txt
grep "maximum resident set size" baseline.txt
```

### 2. Identify Slow Phases
```bash
# Enable debug logging
RUST_LOG=debug bazbom scan . 2>&1 | tee debug.log

# Analyze log timestamps
grep -E "\[bazbom\]" debug.log | while read line; do
    echo "$line"
done
```

### 3. Isolate Bottleneck
```bash
# Test without polyglot
# (would need feature flag)

# Test with smaller dataset
bazbom scan --limit 10 .

# Test parsing only
# (could add --parse-only flag)
```

### 4. Measure Improvement
```bash
# Before optimization
/usr/bin/time -l bazbom scan . 2>&1 | tee before.txt

# Apply fix
# ... make changes ...

# After optimization
/usr/bin/time -l bazbom scan . 2>&1 | tee after.txt

# Compare
diff <(grep "real\|maximum" before.txt) <(grep "real\|maximum" after.txt)
```

## Performance Regression Detection

### Test Suite
```bash
#!/bin/bash
# performance-regression-test.sh

REPOS=(
    "~/Documents/BazBOM_Testing/real-repos/bazel-examples"
    "~/Documents/BazBOM_Testing/generated"
)

for repo in "${REPOS[@]}"; do
    echo "Testing: $repo"
    cd "$repo"

    # Run 3 times, take median
    for i in 1 2 3; do
        /usr/bin/time -l bazbom scan . 2>&1 | grep "real"
    done
done
```

### Acceptable Thresholds
- **Time regression:** >20% slower than baseline
- **Memory regression:** >30% more memory
- **Throughput regression:** <50% packages/second

## Optimization Recommendations

### For Large Repositories (>1000 packages)
```bash
# Use incremental mode
bazbom scan --incremental .

# Limit scope
bazbom scan --ecosystems java .

# Disable reachability
export BAZBOM_NO_SMART_DEFAULTS=1
```

### For Repeated Scans
```bash
# Use caching (if implemented)
bazbom scan --cache .

# Scan only changed files
bazbom scan --bazel-affected-by-files $(git diff --name-only HEAD~1)
```

### For CI/CD
```bash
# Quick mode for PRs
bazbom quick .

# Check mode for fast validation
bazbom check .

# Full scan only on main branch
if [ "$BRANCH" = "main" ]; then
    bazbom full .
fi
```

## Performance Report Format

```
Performance Profile Report
=========================

Repository: bazel-examples
Size: 8.2MB
Packages: 59

Timing:
  Real time: 0.92s
  User time: 0.45s
  System time: 0.28s

Memory:
  Maximum RSS: 52MB
  Peak memory: 48MB
  Average memory: 35MB

Phases:
  Detection: 0.02s (2%)
  Parsing: 0.15s (16%)
  Extraction: 0.35s (38%)
  SBOM gen: 0.25s (27%)
  Polyglot: 0.15s (16%)

Bottlenecks:
  ⚠️ Extraction phase taking 38% of total time
  ✅ Memory usage within normal range
  ✅ No apparent bottlenecks

Recommendations:
  - Performance is within expected range
  - Consider caching parsed maven_install.json for repeated runs
```

## Profiling Tools

### Built-in Rust Tools
```bash
# Cargo flamegraph (requires cargo-flamegraph)
cargo flamegraph --bin bazbom -- scan .

# Valgrind (if available)
valgrind --tool=massif target/release/bazbom scan .
```

### macOS Tools
```bash
# Instruments (if available)
instruments -t "Time Profiler" -D trace.trace target/release/bazbom scan .

# Activity Monitor
# (manual observation during execution)
```

## Success Criteria

Performance is acceptable when:
- ✅ Meets baseline expectations for repository size
- ✅ No single phase >50% of total time
- ✅ Memory usage linear with package count
- ✅ No memory leaks (RSS stays stable)
- ✅ Throughput >100 packages/second

Remember: Optimize for the common case, not edge cases. Most repos are small to medium size.
