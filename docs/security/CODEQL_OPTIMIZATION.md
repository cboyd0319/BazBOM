# CodeQL Optimization Guide

This document describes strategies for optimizing CodeQL security scans in large repositories to prevent timeouts and improve scan performance.

## Problem Statement

CodeQL security scans can timeout in large repositories due to:
- Large codebase size requiring extensive analysis
- Complex dependency graphs
- Comprehensive query suites (security-extended, security-and-quality)
- Limited GitHub Actions runner resources (6 hours max)

## Optimization Strategies Implemented

### 1. Dynamic Timeout Configuration

The CodeQL workflow uses context-aware timeout values:

- **Pull Request checks**: 120 minutes (lighter queries, focused scope)
- **Scheduled scans**: 360 minutes (comprehensive analysis)
- **Push to main**: 120 minutes (regular monitoring)

```yaml
timeout-minutes: ${{ github.event_name == 'schedule' && 360 || 120 }}
```

### 2. Shallow vs Full Clone

- **Pull Requests**: Shallow clone (`fetch-depth: 1`) - faster checkout
- **Scheduled scans**: Full history (`fetch-depth: 0`) - comprehensive analysis

```yaml
fetch-depth: ${{ github.event_name == 'pull_request' && 1 || 0 }}
```

### 3. Query Suite Selection

Different query suites based on scan context:

- **Pull Requests**: `security-and-quality` (faster, focused on common issues)
- **Scheduled scans**: `security-extended,security-and-quality` (comprehensive)

```yaml
queries: ${{ github.event_name == 'pull_request' && 'security-and-quality' || 'security-extended,security-and-quality' }}
```

### 4. Path Exclusions

Exclude directories that don't require security analysis:

```yaml
paths-ignore:
  - '**/tests/**'        # Test code
  - '**/target/**'       # Build artifacts
  - '**/bazel-*/**'      # Bazel build directories
  - '**/examples/**'     # Example code
  - '**/benches/**'      # Benchmark code
  - '**/benchmarks/**'   # Benchmark code
```

### 5. Database Caching

Cache CodeQL databases to speed up subsequent runs:

```yaml
- name: Cache CodeQL database
  uses: actions/cache@v4
  with:
    path: |
      ~/.codeql
      ~/codeql-home
    key: ${{ runner.os }}-codeql-${{ matrix.language }}-${{ hashFiles('**/Cargo.lock') }}
```

### 6. Build Optimization

Optimize the build process for CodeQL analysis:

```yaml
env:
  CARGO_INCREMENTAL: 0      # Disable incremental compilation
  RUSTFLAGS: "-C opt-level=0"  # Disable optimizations
```

### 7. Resource Tuning

Configure CodeQL resource usage:

```yaml
config: |
  threads: 0        # Use all available CPU threads
  ram: 6144         # 6GB RAM allocation
```

### 8. Snippet Control

Disable code snippets in PR results to reduce processing:

```yaml
add-snippets: ${{ github.event_name != 'pull_request' }}
```

## Performance Impact

Expected improvements:

- **PR scans**: 30-50% faster (lighter queries, shallow clone, no snippets)
- **Scheduled scans**: Same thoroughness, better reliability with extended timeout
- **Build time**: 10-20% faster with build optimizations
- **Cache hit**: 40-60% faster on subsequent runs with same dependencies

## Monitoring and Tuning

### Check Scan Duration

Monitor workflow execution time in GitHub Actions:

1. Go to Actions tab
2. Select "CodeQL Security Scanning" workflow
3. Review run duration for each event type

### Adjust Timeouts

If scans still timeout:

1. Increase timeout for specific event types
2. Further reduce query suite for PRs
3. Consider splitting analysis by crate/module

### Cache Effectiveness

Monitor cache hit rate:

1. Check workflow logs for "Cache restored" vs "Cache not found"
2. Adjust cache key if dependencies change frequently
3. Consider using multiple cache keys for different scenarios

## Advanced Optimization (Future)

For very large repositories, consider:

### Matrix Strategy

Split analysis across multiple jobs:

```yaml
strategy:
  matrix:
    crate: [bazbom, bazbom-core, bazbom-formats, bazbom-advisories]
```

### Incremental Analysis

Analyze only changed files in PRs:

1. Use `paths` filter in workflow triggers
2. Configure CodeQL to analyze specific directories
3. Merge results from multiple runs

### Database Sharding

For extremely large codebases:

1. Create separate CodeQL databases per module
2. Analyze databases in parallel
3. Merge SARIF results

## Troubleshooting

### Timeout Despite Optimizations

If scans still timeout:

1. Check runner resource usage in workflow logs
2. Identify slow queries using `--debug` mode
3. Consider excluding additional paths
4. Use `default` query suite instead of `security-extended`

### Cache Misses

If cache is not being used:

1. Verify `Cargo.lock` is committed
2. Check cache key matches between runs
3. Ensure cache size is within limits (10GB max)

### Build Failures

If autobuild fails:

1. Check build dependencies are available
2. Verify Rust toolchain version
3. Consider manual build commands instead of autobuild

## References

- [CodeQL documentation](https://codeql.github.com/docs/)
- [GitHub Actions caching](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)
- [CodeQL query suites](https://codeql.github.com/docs/codeql-cli/creating-codeql-query-suites/)
- [CodeQL configuration](https://docs.github.com/en/code-security/code-scanning/automatically-scanning-your-code-for-vulnerabilities-and-errors/configuring-code-scanning)

## Related Documentation

- [Workflow Security Policy](WORKFLOW_SECURITY_POLICY.md)
- [Security Review Checklist](SECURITY_REVIEW_CHECKLIST.md)
- [Orchestrated Scan](../integrations/orchestrated-scan.md)
