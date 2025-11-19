# Scanner Trait Architecture

**Status:** Production (v6.5.0)
**Last Updated:** 2025-11-19
**Introduced:** November 2025 refactor

## Overview

The `Scanner` trait provides a unified interface for all ecosystem scanners in BazBOM. This trait-based architecture replaced the previous ad-hoc implementation with a clean, maintainable pattern that enables parallel scanning, thread-safe license caching, and consistent error handling.

## Motivation

**Before the refactor (v6.4):**
- 8 different scanner implementations with inconsistent interfaces
- Duplicate license fetching logic
- No standardized error handling
- Difficult to add new ecosystems
- Sequential scanning only

**After the refactor (v6.5):**
- Single `Scanner` trait implemented by all scanners
- Automatic license caching with thread-safety
- Consistent error handling via `anyhow::Result`
- Easy to add new ecosystems (just implement the trait)
- Parallel scanning via `ParallelOrchestrator`

## Architecture

### Scanner Trait Definition

**Location:** `crates/bazbom-scanner/src/scanner.rs`

```rust
use async_trait::async_trait;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

#[async_trait]
pub trait Scanner: Send + Sync {
    /// Scanner name (e.g., "npm", "python", "rust")
    fn name(&self) -> &str;

    /// Detect if this scanner applies to the given directory
    fn detect(&self, root: &Path) -> bool;

    /// Scan the project and return dependency information
    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult>;

    /// Fetch license for a package (with automatic caching)
    fn fetch_license(&self, ctx: &LicenseContext) -> License {
        let key = format!("{}:{}:{}", self.name(), ctx.package, ctx.version);
        ctx.cache.get_or_insert_with(key, || {
            self.fetch_license_uncached(ctx)
        })
    }

    /// Override to implement custom license fetching
    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        License::Unknown
    }
}
```

### Key Components

#### 1. ScanContext

Provides context for scanning operations:

```rust
pub struct ScanContext {
    /// Project root directory
    pub root: PathBuf,

    /// Optional lockfile path (for exact versions)
    pub lockfile: Option<PathBuf>,

    /// Optional manifest path (fallback)
    pub manifest: Option<PathBuf>,

    /// Shared license cache (thread-safe)
    pub cache: Arc<LicenseCache>,
}
```

**Builder pattern for configuration:**

```rust
let cache = Arc::new(LicenseCache::new());
let ctx = ScanContext::new(project_root, cache)
    .with_lockfile("Cargo.lock")
    .with_manifest("Cargo.toml");
```

#### 2. LicenseCache

Thread-safe cache using `RwLock<HashMap>`:

```rust
pub struct LicenseCache {
    cache: RwLock<HashMap<String, License>>,
}

impl LicenseCache {
    pub fn get_or_insert_with<F>(&self, key: String, f: F) -> License
    where
        F: FnOnce() -> License,
    {
        // Try read lock first (fast path)
        if let Some(license) = self.get(&key) {
            return license;
        }

        // Compute and cache (slow path)
        let license = f();
        self.insert(key, license.clone());
        license
    }
}
```

**Performance:**
- O(1) lookups for cached licenses
- Read-biased locking (multiple readers, single writer)
- Automatic deduplication of license requests

#### 3. ScannerRegistry

Central registry for all scanners:

```rust
pub struct ScannerRegistry {
    scanners: Vec<Box<dyn Scanner>>,
}

impl ScannerRegistry {
    pub fn new() -> Self {
        Self {
            scanners: vec![
                Box::new(NpmScanner::new()),
                Box::new(PythonScanner::new()),
                Box::new(GoScanner::new()),
                Box::new(RustScanner::new()),
                Box::new(RubyScanner::new()),
                Box::new(PhpScanner::new()),
                Box::new(MavenScanner::new()),
                Box::new(GradleScanner::new()),
            ],
        }
    }

    pub fn detect(&self, root: &Path) -> Vec<&dyn Scanner> {
        self.scanners
            .iter()
            .filter(|s| s.detect(root))
            .map(|s| s.as_ref())
            .collect()
    }
}
```

## Implementation Guide

### Adding a New Scanner

**1. Create scanner module:**

```rust
// crates/bazbom-scanner/src/ecosystems/elixir/mod.rs

use crate::scanner::{Scanner, ScanContext, License, LicenseContext};
use crate::types::EcosystemScanResult;
use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;

pub struct ElixirScanner;

impl ElixirScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for ElixirScanner {
    fn name(&self) -> &str {
        "elixir"
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("mix.exs").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result = EcosystemScanResult::new(
            "Elixir".to_string(),
            ctx.root.display().to_string(),
        );

        // 1. Parse lockfile (preferred)
        if let Some(ref lockfile) = ctx.lockfile {
            if lockfile.file_name().unwrap() == "mix.lock" {
                parse_mix_lock(lockfile, &mut result)?;
            }
        }

        // 2. Fallback to manifest
        if result.packages.is_empty() {
            if let Some(ref manifest) = ctx.manifest {
                parse_mix_exs(manifest, &mut result)?;
            }
        }

        Ok(result)
    }

    fn fetch_license_uncached(&self, ctx: &LicenseContext) -> License {
        // Query Hex.pm API for license information
        query_hex_license(&ctx.package, &ctx.version)
            .unwrap_or(License::Unknown)
    }
}
```

**2. Register scanner:**

```rust
// crates/bazbom-scanner/src/registry.rs

impl ScannerRegistry {
    pub fn new() -> Self {
        Self {
            scanners: vec![
                // ... existing scanners ...
                Box::new(ElixirScanner::new()),  // Add here
            ],
        }
    }
}
```

**3. Add tests:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[tokio::test]
    async fn test_elixir_scanner_detect() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("mix.exs"), "defmodule MyApp.MixProject").unwrap();

        let scanner = ElixirScanner::new();
        assert!(scanner.detect(temp.path()));
    }

    #[tokio::test]
    async fn test_elixir_scanner_scan() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("mix.lock"), r#"
  "phoenix": {:hex, :phoenix, "1.7.0", "..."},
  "plug": {:hex, :plug, "1.14.0", "..."}
"#).unwrap();

        let scanner = ElixirScanner::new();
        let cache = Arc::new(LicenseCache::new());
        let ctx = ScanContext::new(temp.path().to_path_buf(), cache)
            .with_lockfile("mix.lock");

        let result = scanner.scan(&ctx).await.unwrap();
        assert_eq!(result.ecosystem, "Elixir");
        assert!(result.packages.iter().any(|p| p.name == "phoenix"));
    }
}
```

### Best Practices

#### 1. Lockfile First, Manifest Fallback

**Always prefer lockfiles** for exact versions:

```rust
async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(...);

    // Lockfile path (exact versions)
    if let Some(ref lockfile) = ctx.lockfile {
        parse_lockfile(lockfile, &mut result)?;
    }

    // Fallback to manifest (version ranges)
    if result.packages.is_empty() {
        if let Some(ref manifest) = ctx.manifest {
            parse_manifest(manifest, &mut result)?;
        }
    }

    Ok(result)
}
```

#### 2. Use License Cache

**Don't implement license caching yourself** - the trait does it:

```rust
// ✅ Good: Use default implementation (cached)
impl Scanner for MyScanner {
    // Don't override fetch_license(), just implement fetch_license_uncached()
    fn fetch_license_uncached(&self, ctx: &LicenseContext) -> License {
        // Expensive operation (only called once per package)
        query_registry_api(&ctx.package, &ctx.version)
    }
}

// ❌ Bad: Re-implementing caching
impl Scanner for MyScanner {
    fn fetch_license(&self, ctx: &LicenseContext) -> License {
        // Don't do this! Trait already implements caching
        if let Some(cached) = self.my_cache.get(...) {
            return cached;
        }
        // ...
    }
}
```

#### 3. Error Handling with Context

**Use `anyhow::Context` for clear error messages:**

```rust
use anyhow::{Context, Result};

async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
    let lockfile = ctx.lockfile
        .as_ref()
        .context("No lockfile found for Python project")?;

    let content = fs::read_to_string(lockfile)
        .context(format!("Failed to read lockfile: {}", lockfile.display()))?;

    let parsed = parse_poetry_lock(&content)
        .context("Failed to parse poetry.lock format")?;

    Ok(parsed)
}
```

#### 4. Async for I/O Operations

**Use async for file I/O and network requests:**

```rust
async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
    // ✅ Good: Use tokio::fs for async file I/O
    let content = tokio::fs::read_to_string(&lockfile_path).await?;

    // ✅ Good: Use reqwest for async HTTP requests
    let response = reqwest::get(&registry_url).await?;
    let metadata = response.json::<PackageMetadata>().await?;

    Ok(result)
}
```

## Existing Scanners

| Scanner | Lockfile | Manifest | License Source | LOC |
|---------|----------|----------|----------------|-----|
| **NpmScanner** | package-lock.json, yarn.lock, pnpm-lock.yaml | package.json | npm registry | 280 |
| **PythonScanner** | poetry.lock, Pipfile.lock | requirements.txt, pyproject.toml | PyPI | 290 |
| **GoScanner** | go.sum | go.mod | go.dev | 270 |
| **RustScanner** | Cargo.lock | Cargo.toml | crates.io | 240 |
| **RubyScanner** | Gemfile.lock | Gemfile | RubyGems | 290 |
| **PhpScanner** | composer.lock | composer.json | Packagist | 300 |
| **MavenScanner** | - | pom.xml | Maven Central | 320 |
| **GradleScanner** | - | build.gradle[.kts] | Maven Central | 310 |

All scanners average **~280 lines of code** thanks to the unified trait architecture.

## Performance Characteristics

### Serial Scanning (Old)

```
Time = T(npm) + T(python) + T(rust) + T(go) + ...
     = 0.8s + 1.2s + 0.5s + 0.4s + ...
     = ~3-4 seconds (sequential)
```

### Parallel Scanning (New)

```
Time = max(T(npm), T(python), T(rust), T(go), ...)
     = max(0.8s, 1.2s, 0.5s, 0.4s, ...)
     = ~1.2 seconds (parallel)
```

**Speedup:** 3-4× for multi-ecosystem projects

### License Cache Hit Rate

For a monorepo with duplicate dependencies:

```
Total license requests: 500
Unique packages: 80
Cache hit rate: (500 - 80) / 500 = 84%
```

**Performance:** ~5× faster license fetching in large monorepos

## Integration with ParallelOrchestrator

See [orchestrator.md](orchestrator.md) for how scanners are executed in parallel.

**Quick example:**

```rust
use bazbom_orchestrator::{ParallelOrchestrator, OrchestratorConfig};

let config = OrchestratorConfig {
    max_concurrent: num_cpus::get(),
    show_progress: true,
    enable_reachability: true,
    enable_vulnerabilities: true,
};

let orchestrator = ParallelOrchestrator::with_config(config);
let results = orchestrator.scan_directory("./monorepo").await?;

// Results contain data from all detected scanners
for result in results {
    println!("{}: {} packages", result.ecosystem, result.total_packages);
}
```

## Migration from Old Architecture

### For Scanner Developers

**Old pattern (v6.4):**

```rust
pub fn scan_npm_project(root: &Path) -> Result<Vec<Package>> {
    let lockfile = root.join("package-lock.json");
    let packages = parse_lockfile(&lockfile)?;
    Ok(packages)
}
```

**New pattern (v6.5):**

```rust
pub struct NpmScanner;

#[async_trait]
impl Scanner for NpmScanner {
    fn name(&self) -> &str { "npm" }

    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        // Implementation...
    }
}
```

### For CLI Integration

**Old pattern (v6.4):**

```rust
// Sequential scanning
let npm_result = scan_npm_project(root)?;
let python_result = scan_python_project(root)?;
let rust_result = scan_rust_project(root)?;
```

**New pattern (v6.5):**

```rust
// Parallel scanning via orchestrator
use bazbom_orchestrator::ParallelOrchestrator;

let orchestrator = ParallelOrchestrator::new();
let results = orchestrator.scan_directory(root).await?;
```

## See Also

- [orchestrator.md](orchestrator.md) - Parallel orchestration architecture
- [../polyglot/README.md](../polyglot/README.md) - Polyglot scanning guide
- [../archive/refactor/DAY1_COMPLETE.md](../archive/refactor/DAY1_COMPLETE.md) - Refactor history

---

**Next Steps:**
- Read [orchestrator.md](orchestrator.md) to understand parallel execution
- See [../polyglot/README.md](../polyglot/README.md) for ecosystem-specific details
- Check [../reference/capabilities-reference.md](../reference/capabilities-reference.md) for supported features
