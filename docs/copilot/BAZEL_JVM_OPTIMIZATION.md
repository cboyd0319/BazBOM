# Bazel JVM Optimization Guide

**Date:** 2025-11-04  
**Status:** Active Development  
**Target:** Phase 8 Completion (90% → 95%)

---

## Overview

BazBOM provides world-class SBOM generation for Bazel JVM projects. This document outlines current optimizations and planned improvements specific to Java, Kotlin, and Scala builds in Bazel.

---

## Current Bazel JVM Support

### Supported Rules

**Java (rules_java):**
- `java_library` - Java library compilation
- `java_binary` - Java executable
- `java_test` - Java unit tests
- `java_import` - Pre-built JAR imports
- `java_proto_library` - Protocol buffer Java bindings

**Kotlin (rules_kotlin):**
- `kt_jvm_library` - Kotlin library for JVM
- `kt_jvm_binary` - Kotlin executable for JVM
- `kt_jvm_test` - Kotlin tests for JVM
- `kt_jvm_import` - Pre-built Kotlin JAR

**Scala (rules_scala):**
- `scala_library` - Scala library
- `scala_binary` - Scala executable
- `scala_test` - ScalaTest tests
- `scala_import` - Pre-built Scala JAR

**Android (rules_android):**
- `android_library` - Android library (AAR)
- `android_binary` - Android application (APK)
- `aar_import` - Pre-built AAR imports

### Dependency Management

**rules_jvm_external (Primary):**
- Maven repository integration via `maven_install`
- Authoritative `maven_install.json` for precise versions
- Transitive dependency resolution
- Version conflict resolution
- Lock file for reproducible builds

**Custom JVM Rules:**
- Direct JAR file dependencies
- Local repository dependencies
- Custom rule integrations

---

## Performance Optimizations

### Query Caching (Implemented)

**BazelQueryOptimizer** provides:
- LRU cache for Bazel query results
- Configurable cache size (default: 1000 entries)
- Cache hit/miss tracking
- Performance metrics collection

**Benefits:**
- 50-90% reduction in repeated queries
- Faster incremental scans
- Reduced Bazel server load

**Usage:**
```rust
let optimizer = BazelQueryOptimizer::new(workspace_path);
let targets = optimizer.query_targets("//...:all")?;
let metrics = optimizer.metrics();
println!("Cache hit rate: {:.1}%", metrics.cache_hit_rate());
```

### Aspect-Based Analysis (Implemented)

**JVM Aspects** (`tools/supplychain/aspects.bzl`):
- Traverses Java, Kotlin, Scala targets
- Extracts Maven coordinates from tags
- Builds dependency graph via `SbomInfo` provider
- Collects checksums and PURLs

**Benefits:**
- Single Bazel invocation for entire workspace
- Accurate transitive dependency tracking
- Native Bazel integration (no external tools)
- Respects visibility and configuration

**Usage:**
```bash
bazel build //... \
  --aspects=//tools/supplychain:aspects.bzl%sbom_aspect \
  --output_groups=sbom
```

---

## Planned Optimizations (Phase 8 Completion)

### 1. Parallel Query Execution

**Problem:** Sequential queries for large JVM monorepos slow
**Solution:** Parallel query execution for independent subtrees

**Implementation:**
```rust
// Divide workspace into independent modules
let modules = detect_jvm_modules(workspace)?;

// Query in parallel using rayon
let results: Vec<_> = modules.par_iter()
    .map(|module| query_module_targets(module))
    .collect();
```

**Benefits:**
- 2-4x faster on multi-module projects
- Scales with CPU cores
- Respects Bazel query independence

**Timeline:** 1 week

---

### 2. Java-Specific Rule Filtering

**Problem:** Bazel query returns all rule types, not just JVM
**Solution:** Client-side filtering for Java/Kotlin/Scala rules

**Implementation:**
```rust
const JVM_RULE_KINDS: &[&str] = &[
    "java_library", "java_binary", "java_test",
    "kt_jvm_library", "kt_jvm_binary",
    "scala_library", "scala_binary",
    "android_library", "android_binary",
];

fn filter_jvm_targets(targets: Vec<String>) -> Vec<String> {
    targets.into_iter()
        .filter(|t| is_jvm_target(t))
        .collect()
}
```

**Benefits:**
- Reduces noise from non-JVM targets
- Faster SBOM generation
- Clearer dependency graphs

**Timeline:** 3 days

---

### 3. Incremental Scanning with Build Events

**Problem:** Full workspace scans on every change
**Solution:** Use Bazel Build Event Protocol (BEP) for incremental analysis

**Implementation:**
```rust
// Subscribe to Bazel Build Events
let events = watch_build_events(workspace)?;

for event in events {
    match event {
        BuildEvent::TargetComplete { label, outputs } => {
            if is_jvm_target(&label) {
                update_sbom_for_target(&label, outputs)?;
            }
        }
        _ => {}
    }
}
```

**Benefits:**
- Real-time SBOM updates during builds
- No full workspace rescans
- Integrates with IDE builds

**Timeline:** 2 weeks

---

### 4. Memory-Mapped maven_install.json

**Problem:** Large `maven_install.json` files (10MB+) loaded into memory
**Solution:** Memory-mapped file access with lazy loading

**Implementation:**
```rust
use memmap2::Mmap;

fn load_maven_install_mmap(path: &Path) -> Result<MavenInstall> {
    let file = File::open(path)?;
    let mmap = unsafe { Mmap::map(&file)? };
    
    // Parse JSON in chunks, only load required dependencies
    parse_maven_install_streaming(&mmap)
}
```

**Benefits:**
- Constant memory usage regardless of file size
- Faster startup time
- Supports 50K+ target monorepos

**Timeline:** 1 week

---

### 5. Kotlin Multiplatform JVM Handling

**Problem:** Kotlin Multiplatform projects have multiple targets
**Solution:** Filter for JVM-specific targets only

**Implementation:**
```rust
fn detect_kotlin_jvm_target(target: &str) -> bool {
    // Detect jvm() platform in Kotlin Multiplatform
    target.contains(":jvm") || 
    target.contains("JvmMain") ||
    is_kt_jvm_library(target)
}
```

**Benefits:**
- Accurate JVM dependency tracking
- Excludes JS, Native, WASM targets
- Proper scoping for mixed projects

**Timeline:** 3 days

---

## Performance Benchmarks

### Current Performance (v0.5.1)

| Project Size | Targets | Dependencies | Scan Time | Memory |
|--------------|---------|--------------|-----------|--------|
| Small        | 100     | 50           | 5s        | 150MB  |
| Medium       | 1,000   | 200          | 30s       | 300MB  |
| Large        | 10,000  | 500          | 3m        | 800MB  |

### Target Performance (Post-Optimization)

| Project Size | Targets | Dependencies | Scan Time | Memory | Improvement |
|--------------|---------|--------------|-----------|--------|-------------|
| Small        | 100     | 50           | 3s        | 100MB  | 40% faster  |
| Medium       | 1,000   | 200          | 15s       | 200MB  | 50% faster  |
| Large        | 10,000  | 500          | 90s       | 400MB  | 50% faster  |
| XL           | 50,000  | 2000         | 6m        | 1GB    | NEW         |

---

## JVM-Specific Best Practices

### 1. Use rules_jvm_external

**Do:**
```starlark
maven_install(
    artifacts = [
        "com.google.guava:guava:31.1-jre",
        "org.slf4j:slf4j-api:2.0.9",
    ],
    repositories = [
        "https://repo1.maven.org/maven2",
    ],
    generate_compat_repositories = True,
    maven_install_json = "@//:maven_install.json",
)
```

**Don't:**
- Use `http_jar` or `http_file` for Maven artifacts
- Mix rules_jvm_external with manual JAR downloads
- Skip lockfile generation

### 2. Tag Maven Coordinates

**Do:**
```starlark
java_library(
    name = "mylib",
    srcs = glob(["*.java"]),
    tags = ["maven_coordinates=com.example:mylib:1.0.0"],
)
```

**Why:** Enables accurate PURL generation and SBOM tracking

### 3. Structure for Modules

**Do:**
```
workspace/
├── MODULE.bazel
├── maven_install.json
├── modules/
│   ├── api/BUILD
│   ├── core/BUILD
│   └── web/BUILD
```

**Why:** Enables parallel analysis and clearer dependency boundaries

---

## Troubleshooting

### Slow Bazel Queries

**Symptom:** `bazel query` takes >1 minute  
**Solution:**
1. Check Bazel server status: `bazel info`
2. Use specific patterns: `//modules/api/...` instead of `//...`
3. Enable query caching in BazBOM (already default)

### Missing Dependencies

**Symptom:** SBOM missing transitive dependencies  
**Solution:**
1. Regenerate lockfile: `bazel run @unpinned_maven//:pin`
2. Verify `maven_install.json` is up-to-date
3. Check aspect is applied to all JVM targets

### High Memory Usage

**Symptom:** BazBOM uses >2GB memory on large projects  
**Solution:**
1. Run with `--max-memory=1g` flag (future)
2. Use incremental mode: `--incremental` (future)
3. Scan specific modules instead of entire workspace

---

## Implementation Roadmap

### Week 1: Parallel Queries + Rule Filtering
- [ ] Implement parallel query execution
- [ ] Add JVM rule kind filtering
- [ ] Benchmark against v0.5.1
- [ ] Update tests

### Week 2: Memory Optimization
- [ ] Memory-mapped `maven_install.json` parsing
- [ ] Streaming JSON parser
- [ ] Memory profiling
- [ ] Performance regression tests

### Week 3: Kotlin & Incremental
- [ ] Kotlin Multiplatform JVM detection
- [ ] Build Event Protocol integration
- [ ] Incremental scan prototype
- [ ] Integration tests

### Week 4: Polish & Release
- [ ] Documentation updates
- [ ] Example Bazel projects
- [ ] Performance comparison blog post
- [ ] Release v0.6.0 with optimizations

---

## Success Metrics

**Performance:**
- [ ] 50% faster scans on 10K+ target projects
- [ ] Memory usage <1GB for 50K target projects
- [ ] Cache hit rate >80% on repeated scans

**Accuracy:**
- [ ] 100% JVM dependency detection
- [ ] Zero false positives for non-JVM targets
- [ ] Correct Kotlin Multiplatform JVM filtering

**Usability:**
- [ ] Single command for any Bazel JVM project
- [ ] Clear progress indicators
- [ ] Actionable error messages

---

## Resources

**Bazel Documentation:**
- [Java Rules](https://bazel.build/reference/be/java)
- [Kotlin Rules](https://github.com/bazelbuild/rules_kotlin)
- [rules_jvm_external](https://github.com/bazelbuild/rules_jvm_external)

**BazBOM Code:**
- `crates/bazbom/src/bazel.rs` - Main Bazel integration
- `tools/supplychain/aspects.bzl` - SBOM extraction aspects
- `docs/ADR/ADR-0003-aspect-scope.md` - Aspect design decisions

---

**Document Owner:** @cboyd0319  
**Last Updated:** 2025-11-04  
**Status:** Active Development - Phase 8 Optimization
