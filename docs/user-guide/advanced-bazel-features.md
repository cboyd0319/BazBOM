# Advanced Bazel Features Guide

> **Quick Start:** See [../BAZEL.md](../BAZEL.md) for essential Bazel concepts, aspects, and common patterns. This document covers advanced optimization and enterprise-scale scenarios.

This guide covers advanced features and optimization techniques for Bazel scanning with BazBOM.

## Table of Contents

- [Query Optimization](#query-optimization)
- [Caching Strategies](#caching-strategies)
- [Performance Tuning](#performance-tuning)
- [JVM Rule Detection](#jvm-rule-detection)
- [Target Filtering](#target-filtering)
- [Custom Workflows](#custom-workflows)

## Query Optimization

BazBOM includes a `BazelQueryOptimizer` that provides intelligent query caching and batching for improved performance when analyzing large Bazel workspaces.

### Using the Query Optimizer

The query optimizer is automatically used in optimized scan workflows. It provides:

- **Query result caching**: Reuses results from identical queries
- **Batch processing**: Groups multiple queries for efficiency
- **Performance metrics**: Tracks cache hit rates and query times

### Example: Programmatic Usage

```rust
use bazbom::bazel::BazelQueryOptimizer;
use std::path::PathBuf;

// Create optimizer for your workspace
let workspace = PathBuf::from("/path/to/workspace");
let mut optimizer = BazelQueryOptimizer::new(workspace);

// Query dependencies with automatic caching
let deps = optimizer.query_deps("//my:target", None)?;

// Query with depth limit
let shallow_deps = optimizer.query_deps("//my:target", Some(2))?;

// Query reverse dependencies
let rdeps = optimizer.query_rdeps("//my:target", "//...")?;

// View performance metrics
optimizer.print_metrics();
```

### Performance Metrics

The optimizer tracks:

- **query_count**: Total number of queries executed
- **cache_hits**: Number of queries served from cache
- **cache_misses**: Number of queries that required execution
- **total_targets**: Total number of targets discovered
- **query_time_ms**: Total time spent executing queries

Access metrics programmatically:

```rust
let metrics = optimizer.metrics();
println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate());
println!("Average query time: {}ms", 
         metrics.query_time_ms / metrics.query_count as u64);
```

### Cache Management

Control cache behavior:

```rust
// Clear cache when switching contexts
optimizer.clear_cache();

// Cache is automatically used for duplicate queries
let first = optimizer.query("deps(//my:target)")?;
let second = optimizer.query("deps(//my:target)")?; // Served from cache
```

## Caching Strategies

### Query Result Caching

BazBOM caches Bazel query results to avoid redundant executions:

1. **Cache Key**: Full query expression string
2. **Cache Value**: List of matching targets
3. **Cache Lifetime**: Per-optimizer instance (in-memory)

### Best Practices

- **Reuse optimizer instances** across multiple operations
- **Clear cache** when workspace state changes
- **Use specific queries** to maximize cache hits
- **Monitor metrics** to tune query patterns

### Cache Hit Optimization

To maximize cache hits:

```rust
// Good: Consistent query format
let query1 = "deps(//foo:bar)";
let query2 = "deps(//foo:bar)"; // Cache hit

// Bad: Inconsistent formatting
let query1 = "deps(//foo:bar)";
let query2 = "deps( //foo:bar )"; // Cache miss (whitespace difference)
```

## Performance Tuning

### Parallel Processing

BazBOM uses parallel processing for large dependency sets:

```rust
use rayon::prelude::*;

// Process components in parallel
let results: Vec<_> = components
    .par_iter()
    .map(|component| analyze_component(component))
    .collect();
```

### Benchmark Your Workflows

Run benchmarks to measure performance:

```bash
# Run all benchmarks
cargo bench --package bazbom

# Run Bazel-specific benchmarks
cargo bench --package bazbom --bench bazel_scan_benchmarks

# Run specific benchmark
cargo bench --package bazbom --bench bazel_scan_benchmarks -- maven_install_parsing
```

### Performance Baselines

Expected performance for common operations:

| Operation | 10 Components | 100 Components | 1000 Components |
|-----------|---------------|----------------|-----------------|
| Parse maven_install.json | ~50µs | ~500µs | ~5ms |
| Generate SPDX | ~100µs | ~1ms | ~10ms |
| Query caching (hit) | ~10µs | ~10µs | ~10µs |
| Serialize components | ~200µs | ~2ms | ~20ms |

*Benchmarks run on modern 8-core CPU. Your mileage may vary.*

### Optimization Tips

1. **Use target filtering** to reduce scope:
   ```bash
   bazbom scan --targets "//specific/package/..."
   ```

2. **Enable parallel processing** (enabled by default):
   ```bash
   # Uses all available CPU cores
   bazbom scan --parallel
   ```

3. **Cache query results** in long-running processes

4. **Batch operations** when analyzing multiple targets

## JVM Rule Detection

BazBOM automatically detects JVM-related Bazel rules.

### Supported JVM Rules

**Java Rules:**
- `java_library`
- `java_binary`
- `java_test`
- `java_plugin`
- `java_import`

**Kotlin Rules (rules_kotlin):**
- `kotlin_library`
- `kotlin_jvm_library`
- `kt_jvm_library`
- `kt_jvm_binary`
- `kt_jvm_test`
- `kt_jvm_import`

**Scala Rules (rules_scala):**
- `scala_library`
- `scala_binary`
- `scala_test`
- `scala_import`
- `scala_macro_library`

### Programmatic Rule Detection

```rust
use bazbom::bazel::{is_jvm_rule, get_jvm_rule_query, query_all_jvm_targets};

// Check if a rule is JVM-related
assert!(is_jvm_rule("java_library"));
assert!(is_jvm_rule("kt_jvm_library"));
assert!(!is_jvm_rule("py_binary"));

// Generate query for all JVM rules
let query = get_jvm_rule_query("//...");
// Returns: "kind(java_library, //...) + kind(java_binary, //...) + ..."

// Query all JVM targets in workspace
let workspace = PathBuf::from("/path/to/workspace");
let jvm_targets = query_all_jvm_targets(&workspace)?;
println!("Found {} JVM targets", jvm_targets.len());
```

### Custom Rule Filtering

For custom JVM rules:

```bash
# Query custom rules
bazel query 'kind(my_custom_java_rule, //...)'

# Use with BazBOM
bazbom scan --query 'kind(my_custom_java_rule, //...)'
```

## Target Filtering

### Query-Based Filtering

Filter targets using Bazel query expressions:

```bash
# Scan specific package
bazbom scan --targets "//my/package/..."

# Scan by rule type
bazbom scan --query "kind(java_library, //...)"

# Scan affected by files
bazbom scan --affected-by src/main/Main.java

# Complex queries
bazbom scan --query "deps(//my:target) except //external/..."
```

### Performance Considerations

Target filtering reduces:
- Query execution time
- Dependency resolution time
- SBOM generation time
- Memory usage

**Example impact:**

```bash
# Full workspace scan: 10,000 targets, 2 minutes
bazbom scan //...

# Filtered scan: 100 targets, 5 seconds
bazbom scan //my/package/...
```

### Best Practices

1. **Use specific packages** instead of `//...` when possible
2. **Filter early** to reduce processing overhead
3. **Exclude test targets** for production scans:
   ```bash
   bazbom scan --query "kind(java_library, //...) except kind(java_test, //...)"
   ```

## Custom Workflows

### Incremental Scanning

Scan only changed targets:

```bash
# Get changed files
CHANGED_FILES=$(git diff --name-only HEAD~1)

# Scan affected targets
bazbom scan --affected-by $CHANGED_FILES
```

### Multi-Stage Pipeline

```bash
#!/bin/bash
# Stage 1: Quick scan of main packages
bazbom scan //src/main/... --out-dir stage1/

# Stage 2: Deep scan with reachability analysis
bazbom scan //src/main/... --reachability --out-dir stage2/

# Stage 3: Generate reports
bazbom report stage2/sbom.spdx.json --format html
```

### Integration with CI/CD

```yaml
# GitHub Actions example
- name: BazBOM Scan
  run: |
    # Install BazBOM
    curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
    
    # Scan with optimization
    bazbom scan \
      --targets "//src/..." \
      --format spdx \
      --format cyclonedx \
      --sarif \
      --out-dir sbom/
    
    # Enforce policy
    bazbom policy apply \
      --policy .github/bazbom-policy.yaml \
      --sbom sbom/sbom.spdx.json \
      --fail-on-violation
```

### Custom Query Patterns

**Find all JVM libraries with specific dependencies:**

```bash
bazbom scan --query "kind(java_library, //...) intersect attr('deps', '//third_party/guava', //...)"
```

**Exclude external dependencies:**

```bash
bazbom scan --query "//... except //external/..."
```

**Target specific Maven artifacts:**

```bash
# Requires maven_install.json
bazbom scan --maven-filter "com.google.guava:guava"
```

## Advanced Configuration

### Configuration File

Create `.bazbom/config.toml` in your workspace:

```toml
[bazel]
# Enable query optimization
optimize_queries = true

# Cache size (number of queries to cache)
query_cache_size = 1000

# Parallel processing threads (0 = auto-detect)
num_threads = 0

# Bazel query timeout (seconds)
query_timeout = 300

[bazel.filters]
# Exclude patterns
exclude = [
    "//external/...",
    "//bazel-out/...",
]

# Include only JVM rules
jvm_only = true

[bazel.performance]
# Enable performance metrics
enable_metrics = true

# Log slow queries (ms threshold)
log_slow_queries = 1000
```

### Environment Variables

Configure BazBOM via environment:

```bash
# Override Bazel binary path
export BAZBOM_BAZEL_PATH=/custom/path/to/bazel

# Enable debug logging
export BAZBOM_LOG_LEVEL=debug

# Set query timeout
export BAZBOM_QUERY_TIMEOUT=600
```

## Troubleshooting

### Slow Query Performance

**Symptoms:** Queries take longer than expected

**Solutions:**
1. Enable query optimization
2. Use more specific target patterns
3. Check Bazel query performance directly:
   ```bash
   time bazel query 'deps(//...)'
   ```
4. Increase query timeout for large workspaces

### Cache Not Working

**Symptoms:** Low cache hit rate despite repeated queries

**Solutions:**
1. Ensure queries are formatted identically
2. Reuse optimizer instances
3. Check metrics to confirm cache behavior
4. Clear cache if workspace state changed

### Memory Issues

**Symptoms:** Out of memory errors during large scans

**Solutions:**
1. Use target filtering to reduce scope
2. Process targets in batches
3. Increase JVM heap size:
   ```bash
   export BAZEL_OPTS="--host_jvm_args=-Xmx8g"
   ```
4. Enable incremental scanning

## Next Steps

- Review [Performance Guide](../../operations/performance.md) for optimization strategies
- See [Bazel Integration Test](../../crates/bazbom/tests/bazel_integration_test.rs) for examples
- Run benchmarks: `cargo bench --package bazbom --bench bazel_scan_benchmarks`
- Check [Capabilities Reference](../reference/capabilities-reference.md) for complete feature list

## Resources

- [Bazel Query Language](https://bazel.build/query/language)
- [rules_jvm_external](https://github.com/bazelbuild/rules_jvm_external)
- [rules_kotlin](https://github.com/bazelbuild/rules_kotlin)
- [rules_scala](https://github.com/bazelbuild/rules_scala)
- [BazBOM Benchmarks](../../crates/bazbom/benches/)

## Feedback

Found an issue or have a suggestion? [Open an issue](https://github.com/cboyd0319/BazBOM/issues) or contribute to the documentation.
