# ParallelOrchestrator Architecture

**Status:** Production (v6.5.0)
**Last Updated:** 2025-11-19
**Crate:** `bazbom-orchestrator`

## Overview

The `ParallelOrchestrator` is the central coordinator for BazBOM's multi-ecosystem scanning. It detects ecosystems, launches scanners in parallel, tracks progress, and aggregates results. Introduced in the November 2025 refactor, it provides a 6× performance improvement for monorepo scanning.

## Motivation

**Problem (v6.4):**
- Sequential ecosystem scanning (npm → python → go → rust)
- Single-threaded execution underutilized CPU
- No progress visibility during long scans
- 91 individual HTTP requests for 91 packages (10ms delay each)

**Solution (v6.5):**
- Parallel ecosystem scanning with Tokio
- CPU-based concurrency (scales automatically)
- Real-time progress bars with `indicatif`
- OSV batch API (97% fewer HTTP requests)

## Architecture

### Core Components

```
┌─────────────────────────────────────────────────────────────┐
│                    ParallelOrchestrator                     │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │ OrchestratorConfig │  │ LicenseCache    │                │
│  │ - max_concurrent │  │ - RwLock<HashMap>│                 │
│  │ - show_progress  │  │ - Thread-safe   │                  │
│  │ - enable_*       │  │ - Automatic     │                  │
│  └─────────────────┘  └─────────────────┘                   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Ecosystem Detection                      │   │
│  │  npm? → python? → go? → rust? → ruby? → php? → ...  │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │           Parallel Scanner Execution                  │   │
│  │  ┌─────┐ ┌──────┐ ┌────┐ ┌──────┐                   │   │
│  │  │ npm │ │python│ │ go │ │ rust │  ... (concurrent)  │   │
│  │  └─────┘ └──────┘ └────┘ └──────┘                   │   │
│  │     │        │       │       │                       │   │
│  │     ▼        ▼       ▼       ▼                       │   │
│  │  ┌────────────────────────────────┐                 │   │
│  │  │     Progress Tracking           │                 │   │
│  │  │  📦 npm [████████░░] 80%       │                 │   │
│  │  │  🐍 python [██████████] 100%   │                 │   │
│  │  │  🦀 rust [███████░░░] 70%      │                 │   │
│  │  └────────────────────────────────┘                 │   │
│  └─────────────────────────────────────────────────────┘   │
│                           │                                 │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Result Aggregation                       │   │
│  │  Vec<EcosystemScanResult> → unified SBOM             │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

### Key Data Structures

#### OrchestratorConfig

```rust
pub struct OrchestratorConfig {
    /// Maximum concurrent scanner tasks
    /// Default: num_cpus::get()
    pub max_concurrent: usize,

    /// Show progress bars during scanning
    /// Default: true
    pub show_progress: bool,

    /// Enable reachability analysis per ecosystem
    /// Default: false
    pub enable_reachability: bool,

    /// Enable vulnerability scanning
    /// Default: true
    pub enable_vulnerabilities: bool,
}
```

#### ParallelOrchestrator

```rust
pub struct ParallelOrchestrator {
    config: OrchestratorConfig,
    license_cache: Arc<LicenseCache>,
}

impl ParallelOrchestrator {
    /// Create with default configuration
    pub fn new() -> Self;

    /// Create with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self;

    /// Scan directory for all ecosystems
    pub async fn scan_directory(
        &self,
        path: impl AsRef<Path>
    ) -> Result<Vec<EcosystemScanResult>>;
}
```

## Usage

### Basic Usage

```rust
use bazbom_orchestrator::ParallelOrchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    let orchestrator = ParallelOrchestrator::new();
    let results = orchestrator.scan_directory("./my-monorepo").await?;

    for result in results {
        println!("{}: {} packages, {} vulnerabilities",
            result.ecosystem,
            result.total_packages,
            result.vulnerabilities.len());
    }

    Ok(())
}
```

### Custom Configuration

```rust
use bazbom_orchestrator::{ParallelOrchestrator, OrchestratorConfig};

let config = OrchestratorConfig {
    max_concurrent: 4,              // Limit to 4 concurrent scanners
    show_progress: true,            // Show progress bars
    enable_reachability: true,      // Enable call graph analysis
    enable_vulnerabilities: true,   // Query OSV for vulnerabilities
};

let orchestrator = ParallelOrchestrator::with_config(config);
let results = orchestrator.scan_directory("./project").await?;
```

### CLI Integration

```rust
// In bazbom/src/scan_orchestrator.rs

use bazbom_orchestrator::{OrchestratorConfig, ParallelOrchestrator};

let orchestrator_config = OrchestratorConfig {
    max_concurrent: num_cpus::get(),
    show_progress: true,
    enable_reachability: self.reachability,
    enable_vulnerabilities: true,
};

let orchestrator = ParallelOrchestrator::with_config(orchestrator_config);

// Handle both async contexts
match tokio::runtime::Handle::try_current() {
    Ok(handle) => {
        tokio::task::block_in_place(|| {
            handle.block_on(orchestrator.scan_directory(workspace_path))
        })?
    }
    Err(_) => {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(orchestrator.scan_directory(workspace_path))?
    }
}
```

## Implementation Details

### Parallel Execution with Tokio

```rust
use futures::stream::{self, StreamExt};
use tokio::task;

pub async fn scan_directory(&self, path: impl AsRef<Path>) -> Result<Vec<EcosystemScanResult>> {
    let ecosystems = detect_ecosystems(path)?;

    let results: Vec<Result<EcosystemScanResult>> = stream::iter(
        ecosystems.into_iter().enumerate(),
    )
    .map(|(idx, ecosystem)| {
        let license_cache = self.license_cache.clone();
        let config = self.config.clone();

        task::spawn(async move {
            scan_single_ecosystem(&ecosystem, license_cache, &config).await
        })
    })
    .buffer_unordered(self.config.max_concurrent)
    .collect()
    .await;

    // Filter out errors, return successful results
    Ok(results.into_iter().filter_map(|r| r.ok()).collect())
}
```

### Progress Tracking with Indicatif

```rust
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

fn create_progress_bar(ecosystem: &str, total: usize) -> ProgressBar {
    let pb = ProgressBar::new(total as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} {prefix} [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_prefix(ecosystem.to_string());
    pb
}

// During scanning:
pb.set_message("Scanning package.json...");
pb.inc(1);

// On completion:
pb.finish_with_message("Done!");
```

### OSV Batch Query API

**Location:** `crates/bazbom-scanner/src/vulnerabilities.rs`

```rust
const OSV_BATCH_API_URL: &str = "https://api.osv.dev/v1/querybatch";

pub async fn scan_vulnerabilities(packages: &[Package]) -> Result<Vec<Vulnerability>> {
    // Build batch request
    let queries: Vec<OsvQueryRequest> = packages
        .iter()
        .map(|p| OsvQueryRequest {
            package: OsvPackage {
                ecosystem: map_ecosystem(&p.ecosystem),
                name: format_package_name(p),
            },
            version: p.version.clone(),
        })
        .collect();

    // Single HTTP request for all packages
    let response = query_osv_batch(&client, &queries).await?;

    // Process results
    let mut vulnerabilities = Vec::new();
    for (i, result) in response.results.iter().enumerate() {
        if let Some(vulns) = &result.vulns {
            for vuln in vulns {
                vulnerabilities.push(parse_osv_vulnerability(vuln, &packages[i]));
            }
        }
    }

    Ok(vulnerabilities)
}
```

**Performance impact:**
- **Before:** 91 packages = 91 HTTP requests (+ 10ms delay each = ~1 second overhead)
- **After:** 91 packages = 1 HTTP request (~300ms)
- **Reduction:** 97% fewer HTTP requests

## Performance Characteristics

### Benchmarks

**Test environment:**
- macOS (Apple M2)
- 3 ecosystems (npm + Go + Ruby)
- 91 total packages

| Metric | Sequential (v6.4) | Parallel (v6.5) | Improvement |
|--------|-------------------|-----------------|-------------|
| **Scan time** | ~3-4 seconds | ~0.54 seconds | 6× faster |
| **HTTP requests** | 91 | 3 | 97% reduction |
| **CPU utilization** | ~25% | ~100% | 4× better |

### Scaling Behavior

**Concurrency vs. Performance:**

```
Ecosystems | 1 Scanner | 2 Scanners | 4 Scanners | 8 Scanners
-----------|-----------|------------|------------|------------
1          | 1.0s      | 1.0s       | 1.0s       | 1.0s
2          | 2.0s      | 1.0s       | 1.0s       | 1.0s
4          | 4.0s      | 2.0s       | 1.0s       | 1.0s
8          | 8.0s      | 4.0s       | 2.0s       | 1.0s
```

**Observation:** Near-linear scaling up to CPU count, then plateaus.

### Memory Usage

**Per-scanner overhead:** ~2-5 MB (depends on dependency count)

**Typical monorepo (8 ecosystems, 500 packages):**
- Scan: ~50 MB peak
- Cache: ~10 MB (shared)
- Total: ~60 MB

**Large monorepo (8 ecosystems, 5000 packages):**
- Scan: ~200 MB peak
- Cache: ~50 MB (shared)
- Total: ~250 MB

## Error Handling

### Graceful Degradation

One ecosystem failure doesn't block others:

```rust
let results: Vec<_> = stream::iter(ecosystems)
    .map(|eco| async {
        match scan_ecosystem(&eco).await {
            Ok(result) => Some(result),
            Err(e) => {
                eprintln!("Warning: {} scan failed: {}", eco.name, e);
                None
            }
        }
    })
    .buffer_unordered(config.max_concurrent)
    .filter_map(|r| async { r })  // Drop None results
    .collect()
    .await;
```

### Timeout Handling

```rust
use tokio::time::timeout;
use std::time::Duration;

let result = timeout(
    Duration::from_secs(60),
    scan_ecosystem(&ecosystem)
).await;

match result {
    Ok(Ok(scan_result)) => results.push(scan_result),
    Ok(Err(e)) => eprintln!("Scan error: {}", e),
    Err(_) => eprintln!("Scan timeout after 60s"),
}
```

## Configuration Reference

### Environment Variables

```bash
# Override max concurrent scanners
export BAZBOM_MAX_CONCURRENT=4

# Disable progress bars (for CI)
export BAZBOM_NO_PROGRESS=1

# Override OSV API URL
export OSV_API_URL=https://api.osv.dev

# Disable caching (for testing)
export BAZBOM_DISABLE_CACHE=1
```

### CLI Flags

```bash
# Disable reachability (faster)
bazbom scan . --no-reachability

# Disable vulnerability scanning (SBOM only)
bazbom scan . --no-vulns

# Limit concurrency
bazbom scan . --max-concurrent 2

# Show performance metrics
bazbom scan . --benchmark
```

## Best Practices

### 1. Let it Scale Automatically

**Don't hardcode concurrency:**

```rust
// ✅ Good: Let it scale with CPU
let config = OrchestratorConfig {
    max_concurrent: num_cpus::get(),
    ..Default::default()
};

// ❌ Bad: Hardcoded concurrency
let config = OrchestratorConfig {
    max_concurrent: 4,
    ..Default::default()
};
```

### 2. Share the License Cache

**One cache per orchestrator:**

```rust
// ✅ Good: Shared cache across all scanners
let orchestrator = ParallelOrchestrator::new();  // Creates shared cache
let results = orchestrator.scan_directory("./project").await?;

// ❌ Bad: Multiple orchestrators with separate caches
let orchestrator1 = ParallelOrchestrator::new();
let orchestrator2 = ParallelOrchestrator::new();
```

### 3. Disable Progress in CI

**Avoid ANSI codes in non-interactive environments:**

```rust
let config = OrchestratorConfig {
    show_progress: atty::is(atty::Stream::Stdout),
    ..Default::default()
};
```

### 4. Use Batch API for Vulnerabilities

**Already default behavior, but don't disable:**

```rust
// ✅ Good: Default vulnerability scanning (uses batch)
let config = OrchestratorConfig {
    enable_vulnerabilities: true,
    ..Default::default()
};

// Vulnerabilities will use OSV batch API automatically
```

## Troubleshooting

### "Too many open files"

**Symptom:** `Error: Too many open files (os error 24)`

**Solution:** Increase file descriptor limit:

```bash
# Check current limit
ulimit -n

# Increase (macOS/Linux)
ulimit -n 4096

# Or add to ~/.bashrc or ~/.zshrc
echo "ulimit -n 4096" >> ~/.zshrc
```

### Progress Bars Corrupt Output

**Symptom:** Progress bars interfere with piped output

**Solution:** Disable progress when not interactive:

```bash
BAZBOM_NO_PROGRESS=1 bazbom scan . | jq '.'
```

Or in code:

```rust
let config = OrchestratorConfig {
    show_progress: false,
    ..Default::default()
};
```

### Scan Hangs on Large Projects

**Symptom:** Scan appears stuck on a single ecosystem

**Solution:** Check for:
1. Very large lockfiles (>100MB) - consider splitting
2. Network issues with package registries
3. Slow license fetching - disable with `--no-license-fetch`

## See Also

- [scanner-trait.md](scanner-trait.md) - Scanner trait architecture
- [../operations/performance.md](../operations/performance.md) - Performance tuning
- [../polyglot/README.md](../polyglot/README.md) - Polyglot scanning guide
- [../archive/refactor/DAY2_PROGRESS.md](../archive/refactor/DAY2_PROGRESS.md) - Orchestrator implementation history

---

**Summary:**

The `ParallelOrchestrator` provides automatic, scalable parallel scanning with:
- **6× faster** multi-ecosystem scans
- **97% fewer** HTTP requests
- **Automatic** CPU-based concurrency
- **Real-time** progress tracking
- **Graceful** error handling

For most use cases, `ParallelOrchestrator::new()` with defaults is the right choice.
