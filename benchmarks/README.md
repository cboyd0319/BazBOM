# BazBOM Performance Benchmarks

This directory contains performance benchmarks for BazBOM to ensure it meets scale requirements for large monorepos.

## Benchmark Targets

Based on Phase 8 requirements, BazBOM should handle:
- **Small projects:** 100-1,000 dependencies in <10 seconds
- **Medium projects:** 1,000-10,000 dependencies in <30 seconds  
- **Large monorepos:** 10,000-50,000+ targets in <5 minutes

## Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench scan_performance

# Generate flamegraph for profiling
cargo flamegraph --bench scan_performance
```

## Benchmark Categories

### 1. Scan Performance (`scan_performance.rs`)
- Tests SBOM generation speed
- Tests dependency graph construction
- Tests reachability analysis performance
- Measures memory usage

### 2. Cache Performance (`cache_performance.rs`)
- Cache hit/miss ratios
- Cache lookup speed
- Cache eviction performance
- Storage efficiency

### 3. Advisory Database (`advisory_performance.rs`)
- Database query speed
- In-memory lookup performance
- Bulk vulnerability matching

### 4. SBOM Parsing (`sbom_parsing.rs`)
- SPDX parsing speed
- CycloneDX parsing speed
- Large file handling (>100MB)

## Performance Goals

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| 1K deps scan | ~10s | <5s | ⚠️ |
| 10K deps scan | ~60s | <30s | ⚠️ |
| 50K deps scan | N/A | <300s | 📋 |
| Cache lookup | ~10ms | <1ms | ✅ |
| Memory (1K deps) | ~50MB | <100MB | ✅ |
| Memory (50K deps) | N/A | <2GB | 📋 |

## Optimization Strategies

1. **Parallel Processing**
   - Use rayon for parallel dependency analysis
   - Thread pool for batch operations
   
2. **Caching**
   - LRU cache for advisory lookups
   - Build file hashing for incremental scans
   
3. **Memory Optimization**
   - Streaming JSON parsing for large files
   - String interning for repeated values
   
4. **Database Optimization**
   - SQLite with WAL mode
   - Prepared statements
   - Index optimization

## Profiling Tools

- **cargo-flamegraph** - CPU profiling
- **valgrind/massif** - Memory profiling
- **perf** - Linux performance monitoring
- **Instruments** - macOS profiling

## CI Integration

Benchmarks run automatically on:
- Main branch commits
- Pull requests (performance regression check)
- Weekly scheduled runs

Performance regressions >10% block merge.
