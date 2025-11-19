# Bazel Feature Audit & Implementation Roadmap

**Date:** 2025-11-18
**Target:** Bazel 7+ Support ONLY
**Goal:** Feature parity with commercial SCA tools for Bazel monorepos

---

## Executive Summary

This document audits BazBOM's Bazel capabilities against commercial SCA tools to ensure we have comprehensive Bazel 7+ support. All competitive references have been removed from public documentation.

**Current Status:**
- ✅ 4 Bazel-specific flags implemented
- ⚠️ 7 missing flags identified
- ✅ Targeted scanning (rdeps) implemented
- ⚠️ Build automation missing
- ⚠️ Workspace handling incomplete

---

## Current Implementation

### ✅ Implemented Flags

| Flag | Status | Description |
|------|--------|-------------|
| `--bazel-targets-query` | ✅ IMPLEMENTED | Bazel query expression to select targets |
| `--bazel-targets` | ✅ IMPLEMENTED | Explicit list of targets (comma-separated) |
| `--bazel-affected-by-files` | ✅ IMPLEMENTED | Scan only targets affected by changed files (rdeps) |
| `--bazel-universe` | ✅ IMPLEMENTED | Universe pattern for rdeps queries (default `//...`) |

**Total:** 4/11 flags (36%)

---

## Monorepo-Specific Capabilities

### ⚠️ CRITICAL: Path-Based Scoping Missing

**Commercial approach:** `--include-path` flag for directory-level filtering
**Use case:** Scan only changed sections of monorepo (e.g., `--include-path=ui/**`)
**Performance impact:** 10-100x speedup for large monorepos with isolated changes

**Example workflow:**
```yaml
# GitHub Actions with path filters
- uses: dorny/paths-filter@v2
  id: changes
  with:
    filters: |
      ui: 'ui/**'
      backend: 'backend/**'

- name: Scan UI (if changed)
  if: steps.changes.outputs.ui == 'true'
  run: bazbom scan --include-path ui/**

- name: Scan Backend (if changed)
  if: steps.changes.outputs.backend == 'true'
  run: bazbom scan --include-path backend/**
```

**Priority:** CRITICAL
**Effort:** 2-3 hours
**Status:** ❌ NOT IMPLEMENTED

### ⚠️ CRITICAL: Language Filtering Missing

**Commercial approach:** `--languages` flag for language-based isolation
**Use case:** Scan only specific languages in polyglot monorepos
**Performance impact:** Reduces scan time by 50-80% in polyglot repos

**Examples:**
- `--languages=java` - Java only
- `--languages=javascript,typescript` - JS/TS only
- `--languages=python,go,rust` - Multiple languages

**Priority:** HIGH
**Effort:** 1-2 hours
**Status:** ❌ NOT IMPLEMENTED

### ⚠️ Result Aggregation Missing

**Commercial approach:** Automatic merging of parallel scan results
**Use case:** Run parallel scans per directory/language, merge SBOMs
**Performance impact:** Enables true parallelization without manual merging

**Current limitation:** We can scan in parallel but results aren't auto-merged

**Priority:** MEDIUM
**Effort:** 2-3 hours
**Status:** ❌ NOT IMPLEMENTED

---

## Gap Analysis

### ❌ Missing Critical Features

#### 1. **Target Exclusion**
**Flag:** `--bazel-exclude-targets`
**Priority:** HIGH
**Commercial equivalent:** `--bazel-exclude-targets`
**Use case:** Exclude test targets, third-party code, experimental modules
**Implementation:** Filter targets after query execution

#### 2. **Workspace Path Support**
**Flag:** `--bazel-workspace-path`
**Priority:** HIGH
**Commercial equivalent:** `--bazel-workspace-path`
**Use case:** Non-root workspaces (common in large monorepos)
**Implementation:** Set `working_directory` before Bazel query execution

#### 3. **Internal Target Visibility**
**Flag:** `--bazel-show-internal-targets`
**Priority:** MEDIUM
**Commercial equivalent:** `--bazel-show-internal-targets`
**Use case:** Show library dependencies (`py_library`, `java_library`, `go_library`)
**Implementation:** Include non-binary targets in dependency graph

#### 4. **Go Vendored Mode**
**Flag:** `--bazel-vendor-manifest-path`
**Priority:** MEDIUM
**Commercial equivalent:** `--bazel-vendor-manifest-path`
**Use case:** Bazel projects using Gazelle with vendored Go modules
**Implementation:** Parse go.mod from custom path

#### 5. **Custom Bazel Config**
**Flag:** `--bazel-rc-path`
**Priority:** LOW
**Commercial equivalent:** `--bazel-rc-path`
**Use case:** Custom `.bazelrc` files in non-standard locations
**Implementation:** Pass `--bazelrc=<path>` to Bazel commands

#### 6. **Additional Bazel Flags**
**Flag:** `--bazel-flags`
**Priority:** LOW
**Commercial equivalent:** `--bazel-flags`
**Use case:** Pass arbitrary flags to Bazel (e.g., `--config=production`)
**Implementation:** Append flags to all Bazel command invocations

#### 7. **Quick Scan Mode**
**Flag:** `--quick-scan` (or just use existing `--fast`)
**Priority:** MEDIUM
**Commercial equivalent:** `--quick-scan`
**Use case:** Fast dependency visibility without reachability
**Implementation:** Skip reachability analysis (we already have `--fast`)

#### 8. **Path-Based Scoping** ⚡ MONOREPO CRITICAL
**Flag:** `--include-path <PATTERN>`
**Priority:** CRITICAL
**Commercial equivalent:** `--include-path`
**Use case:** Scan only specific directories (e.g., `ui/**`, `backend/**`)
**Impact:** 10-100x speedup for large monorepos with isolated changes
**Implementation:** Filter file tree before scanning, support glob patterns

#### 9. **Language Filtering** ⚡ POLYGLOT CRITICAL
**Flag:** `--languages <LANG1,LANG2,...>`
**Priority:** HIGH
**Commercial equivalent:** `--languages`
**Use case:** Scan only specific languages in polyglot monorepos
**Impact:** 50-80% scan time reduction
**Implementation:** Skip ecosystem scanners not in language list

#### 10. **Result Aggregation** ⚡ PARALLELIZATION ENABLER
**Flag:** `--aggregate-results <DIR>` or automatic
**Priority:** MEDIUM
**Commercial equivalent:** Automatic
**Use case:** Merge multiple parallel scan results into single SBOM
**Impact:** Enables true CI/CD parallelization
**Implementation:** Merge SPDX/CycloneDX files, deduplicate packages

---

## Missing Infrastructure

### 1. **Automatic Build Handling**
**Current:** Assumes targets are pre-built
**Commercial approach:** Automatically runs `bazel build <target>` for unbuilt targets
**Priority:** HIGH
**Impact:** Users must manually build before scanning

**Implementation:**
```rust
// Check if target is built
fn is_target_built(target: &str) -> Result<bool> {
    // Query bazel for build artifacts
    Command::new("bazel")
        .args(&["query", &format!("outputs({})", target)])
        .output()
        .map(|o| !o.stdout.is_empty())
}

// Auto-build if needed
fn ensure_target_built(target: &str) -> Result<()> {
    if !is_target_built(target)? {
        Command::new("bazel")
            .args(&["build", target])
            .status()?;
    }
    Ok(())
}
```

### 2. **Parallel Target Scanning**
**Current:** Sequential target processing
**Commercial approach:** Parallel analysis of independent targets
**Priority:** MEDIUM
**Impact:** Slower scans for large monorepos

**Implementation:**
```rust
use rayon::prelude::*;

fn scan_targets_parallel(targets: Vec<String>) -> Result<Vec<Report>> {
    targets
        .par_iter()
        .map(|target| scan_single_target(target))
        .collect()
}
```

### 3. **Progressive Build Support**
**Current:** All-or-nothing builds
**Commercial approach:** Continue scanning built targets even if some fail to build
**Priority:** LOW
**Impact:** Single build failure stops entire scan

---

## Bazel 7+ Specific Features

### ✅ Already Supported (No Changes Needed)

1. **Module System (Bzlmod)**
   - Bazel 7 uses `MODULE.bazel` instead of `WORKSPACE`
   - Our query-based approach works with both
   - No changes needed

2. **New Query Patterns**
   - `rdeps()` still works in Bazel 7
   - `deps()` unchanged
   - No migration needed

3. **Aspect System**
   - Our aspect-based dependency extraction still works
   - Bazel 7 aspects are backward compatible

### ⚠️ Deprecated in Bazel 7

1. **WORKSPACE Files**
   - Still supported but deprecated
   - We should add `MODULE.bazel` detection
   - Priority: LOW (both work for now)

---

## Implementation Roadmap

### Phase 1: Critical Gaps (4-6 hours)

**Goal:** Enable core Bazel functionality for monorepos

| # | Feature | Flag | Time | Priority |
|---|---------|------|------|----------|
| 1 | Target exclusion | `--bazel-exclude-targets` | 1h | HIGH |
| 2 | Workspace path | `--bazel-workspace-path` | 1h | HIGH |
| 3 | Auto-build | N/A (infrastructure) | 2-3h | HIGH |
| 4 | Internal targets | `--bazel-show-internal-targets` | 1h | MEDIUM |

**Deliverables:**
- ✅ Target exclusion working
- ✅ Non-root workspaces supported
- ✅ Automatic `bazel build` for unbuilt targets
- ✅ Library dependencies visible

### Phase 2: Performance & Scale (3-4 hours)

**Goal:** Optimize for large monorepos (1000+ targets)

| # | Feature | Time | Priority |
|---|---------|------|----------|
| 5 | Parallel scanning | 2h | MEDIUM |
| 6 | Progressive builds | 1-2h | LOW |

**Deliverables:**
- ✅ Targets scanned in parallel (rayon)
- ✅ Scan continues despite build failures

### Phase 3: Advanced Features (2-3 hours)

**Goal:** Support edge cases and custom configurations

| # | Feature | Flag | Time | Priority |
|---|---------|------|------|----------|
| 7 | Go vendored mode | `--bazel-vendor-manifest-path` | 1h | MEDIUM |
| 8 | Custom config | `--bazel-rc-path` | 30min | LOW |
| 9 | Extra flags | `--bazel-flags` | 30min | LOW |
| 10 | MODULE.bazel detection | N/A | 1h | LOW |

**Deliverables:**
- ✅ Gazelle vendored mode supported
- ✅ Custom `.bazelrc` paths
- ✅ Arbitrary Bazel flag pass-through
- ✅ Bzlmod detection

---

## Validation Plan

### Test Matrix

| Test Case | Description | Expected Result |
|-----------|-------------|-----------------|
| **Basic Scanning** |
| Full workspace | `bazbom scan --bazel-targets-query '//...'` | All targets scanned |
| Single target | `bazbom scan --bazel-targets //app:main` | Only //app:main scanned |
| Multiple targets | `bazbom scan --bazel-targets //app:main //lib:util` | Both targets scanned |
| **Query-Based** |
| Binary targets | `bazbom scan --bazel-targets-query 'kind(java_binary, //...)'` | Only binaries |
| Test targets | `bazbom scan --bazel-targets-query 'kind(".*_test", //...)'` | Only tests |
| **Exclusion** |
| Exclude tests | `bazbom scan --bazel-exclude-targets '//tests/...'` | Tests excluded |
| Exclude vendor | `bazbom scan --bazel-exclude-targets '//vendor/...'` | Vendor excluded |
| **Incremental** |
| Changed files | `bazbom scan --bazel-affected-by-files src/main.cc` | Only affected targets |
| Multiple files | `bazbom scan --bazel-affected-by-files src/a.cc src/b.cc` | Union of affected |
| **Workspace** |
| Non-root | `bazbom scan --bazel-workspace-path services/api` | Scans services/api workspace |
| **Build Handling** |
| Unbuilt targets | Scan with unbuilt targets | Auto-builds then scans |
| Build failures | Scan with some targets that fail to build | Scans successful targets, warns about failures |

### Test Repositories Needed

1. **Basic Monorepo** (10 targets, all built)
   - Validates: Basic scanning, queries, exclusion

2. **Incremental Test Repo** (100 targets, 5 changed)
   - Validates: `--bazel-affected-by-files`, rdeps correctness

3. **Non-Root Workspace** (workspace in subdirectory)
   - Validates: `--bazel-workspace-path`

4. **Unbuilt Targets** (targets not yet compiled)
   - Validates: Automatic build handling

5. **Bazel 7 + Bzlmod** (MODULE.bazel instead of WORKSPACE)
   - Validates: Modern Bazel compatibility

---

## Feature Parity Checklist

### Core Scanning

| Feature | Commercial SCA | BazBOM | Status |
|---------|----------------|--------|--------|
| Build graph analysis | ✅ | ✅ | COMPLETE |
| Entrypoint detection | ✅ | ✅ | COMPLETE |
| Targeted scanning (rdeps) | ✅ | ✅ | COMPLETE |
| Target queries | ✅ | ✅ | COMPLETE |
| Target exclusion | ✅ | ❌ | **MISSING** |
| Non-root workspaces | ✅ | ❌ | **MISSING** |
| Auto-build | ✅ | ❌ | **MISSING** |

### Performance

| Feature | Commercial SCA | BazBOM | Status |
|---------|----------------|--------|--------|
| Parallel scanning | ✅ | ❌ | **MISSING** |
| Progressive builds | ✅ | ❌ | **MISSING** |
| Query caching | ⚠️ | ❌ | Optional |

### Advanced

| Feature | Commercial SCA | BazBOM | Status |
|---------|----------------|--------|--------|
| Internal targets | ✅ | ❌ | **MISSING** |
| Go vendored mode | ✅ | ❌ | **MISSING** |
| Custom config | ✅ | ❌ | **MISSING** |
| Extra flags | ✅ | ❌ | **MISSING** |

### Bazel 7+

| Feature | Commercial SCA | BazBOM | Status |
|---------|----------------|--------|--------|
| Bzlmod (MODULE.bazel) | ✅ | ⚠️ | Works but not detected |
| WORKSPACE | ✅ | ✅ | COMPLETE |
| Aspects | ✅ | ✅ | COMPLETE |

**Feature Parity:** 4/21 features (19%)
**Critical Gaps:** 6
1. Path-based scoping (`--include-path`) ⚡ CRITICAL
2. Language filtering (`--languages`) ⚡ CRITICAL
3. Result aggregation ⚡ CRITICAL
4. Target exclusion (`--bazel-exclude-targets`)
5. Workspace path (`--bazel-workspace-path`)
6. Auto-build infrastructure

---

## CLI Design

### Proposed Final CLI

```bash
# Basic scanning
bazbom scan                                    # Full workspace
bazbom scan --bazel-targets //app:main         # Single target
bazbom scan --bazel-targets-query '//...'      # Query-based

# Monorepo optimization (NEW) ⚡ CRITICAL
bazbom scan --include-path 'ui/**'             # Scan only UI directory
bazbom scan --include-path 'backend/**' --include-path 'shared/**'  # Multiple paths
bazbom scan --languages java,kotlin            # Java/Kotlin only
bazbom scan --languages javascript,typescript  # JS/TS only

# Exclusion (NEW)
bazbom scan --bazel-exclude-targets '//tests/...' '//vendor/...'

# Non-root workspace (NEW)
bazbom scan --bazel-workspace-path services/api

# Incremental (existing)
bazbom scan --bazel-affected-by-files src/main.cc src/util.cc

# Advanced (NEW)
bazbom scan --bazel-show-internal-targets      # Include libraries
bazbom scan --bazel-vendor-manifest-path go.mod
bazbom scan --bazel-rc-path .bazelrc.custom
bazbom scan --bazel-flags '--config=ci --define=debug=false'

# Parallel monorepo scanning (NEW) ⚡
# Terminal 1: Scan UI
bazbom scan --include-path 'ui/**' -o results/ui

# Terminal 2: Scan backend
bazbom scan --include-path 'backend/**' -o results/backend

# Terminal 3: Merge results
bazbom aggregate results/ui results/backend -o final-sbom.json

# Combined monorepo workflow
bazbom scan \
  --include-path 'services/api/**' \
  --languages java,kotlin \
  --bazel-targets-query 'kind(java_binary, //...)' \
  --bazel-exclude-targets '//third_party/...' \
  --reachability \
  -o results/api
```

---

## Implementation Priority

**Total Effort:** ~16-20 hours (updated after monorepo analysis)
**Timeline:** 3-4 work sessions

### Session 1: Critical Monorepo Features (6-8h) ⚡ HIGHEST PRIORITY
1. ✅ **Path-based scoping** (`--include-path`) - CRITICAL for monorepos (2-3h)
2. ✅ **Language filtering** (`--languages`) - CRITICAL for polyglot (1-2h)
3. ✅ Target exclusion (`--bazel-exclude-targets`) - Important (1h)
4. ✅ Workspace path (`--bazel-workspace-path`) - Important (1h)
5. ✅ Auto-build infrastructure - Important (2-3h)

**Rationale:** Path scoping and language filtering provide 10-100x speedups for monorepos

### Session 2: Performance & Aggregation (4-5h)
6. ✅ Result aggregation (merge parallel scans) - Enables parallelization (2-3h)
7. ✅ Parallel scanning (rayon) - Speed up (1-2h)
8. ✅ Progressive builds - Resilience (1h)

### Session 3: Polish & Advanced (3-4h)
9. ✅ Internal targets (`--bazel-show-internal-targets`) - 1h
10. ✅ Go vendored mode (`--bazel-vendor-manifest-path`) - 1h
11. ✅ Custom config (`--bazel-rc-path`) - 30min
12. ✅ Extra flags (`--bazel-flags`) - 30min
13. ✅ Bzlmod detection - 1h
14. ✅ Comprehensive testing - 1h

### Session 4: Documentation (1h)
12. ✅ Update BAZEL.md
13. ✅ Update examples
14. ✅ Update CLI help
15. ✅ Add troubleshooting guide

---

## Success Criteria

**Minimum Viable (MVP):**
- ✅ Target exclusion works
- ✅ Non-root workspaces work
- ✅ Auto-build functional
- ✅ No regressions in existing functionality

**Full Feature Parity:**
- ✅ All 11 flags implemented
- ✅ Parallel scanning working
- ✅ Progressive builds functional
- ✅ Bzlmod detected
- ✅ Comprehensive test coverage (15+ test cases)
- ✅ Documentation updated

**Performance Targets:**
- Full workspace (100 targets): <30s
- Incremental scan (10 affected): <5s
- Parallel speedup: 3-5x on 8-core machine

---

## Maintenance Plan

### Bazel Version Support

**Supported:** Bazel 7.x ONLY
**Reason:** Bazel 6 is deprecated, Bazel 7 is LTS

**If user has Bazel 6:**
- Detect version: `bazel version`
- Show error: "BazBOM requires Bazel 7+. Please upgrade: https://bazel.build/"
- Exit with code 1

### Breaking Changes Policy

**If Bazel 8+ introduces breaking changes:**
1. Test on Bazel 8 pre-release
2. Add version-specific code paths if needed
3. Update minimum version requirement
4. Document migration guide

### Query Compatibility

**Monitor Bazel query language changes:**
- Subscribe to Bazel release notes
- Test query patterns on new versions
- Update query strings if syntax changes

---

## Next Steps

1. **Immediate:** Implement Phase 1 (critical gaps) - 4-6 hours
2. **This week:** Implement Phase 2 (performance) - 3-4 hours
3. **Next week:** Implement Phase 3 (advanced) - 3-4 hours
4. **After validation:** Update all documentation

**Total time to feature parity:** ~16-20 hours

**CRITICAL INSIGHT from monorepo best practices:**
The most important features are NOT Bazel-specific! They are:
1. **Path-based scoping** (`--include-path`) - 10-100x speedup
2. **Language filtering** (`--languages`) - 50-80% reduction
3. **Result aggregation** - Enables parallelization

These work for ALL build systems, not just Bazel. Implementing these first provides maximum ROI.

---

*Last Updated: 2025-11-18*
*Status: Ready for implementation*
*Target Completion: 2025-11-25 (1 week)*
