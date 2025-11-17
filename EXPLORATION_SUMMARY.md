# BazBOM Codebase Exploration Summary

## 1. CURRENT LOGGING IMPLEMENTATION

### Logging Framework
- **Crate Used**: `tracing = "0.1"` (in Cargo.toml, line 31)
- **Current State**: Partially integrated but not fully configured
  - Some modules use `use tracing::info` (found in 4 files):
    - `/crates/bazbom/src/remediation/github.rs` - uses `tracing::info`
    - `/crates/bazbom/src/remediation/apply.rs` - uses `tracing::{info, warn}`
    - `/crates/bazbom/src/incremental.rs` - uses `tracing::info`
    - `/crates/bazbom/src/backup.rs` - uses `tracing::info`

### Current Logging Approach
- **Primary Method**: `println!` and `eprintln!` (1358 logging statements found)
- **User-Facing Output**: Uses colored output via `colored = "3.0"` crate
- **Progress Indication**: Uses `indicatif = "0.18"` for progress bars
- **No Centralized Logger**: Tracing is partially used but not initialized via tracing_subscriber

### Environment Variables
- `BAZBOM_JSON_MODE` - Enables JSON output mode (set in scan.rs line 104)
- `BAZBOM_NO_SMART_DEFAULTS` - Disables smart defaults (checked in commands/scan.rs line 47)
- `BAZBOM_DISABLE_CACHE` - Disables scan caching (checked in scan_orchestrator.rs line 110)

---

## 2. MAIN COMMAND STRUCTURE

### CLI Framework
- **Framework**: `clap = { version = "4", features = ["derive"] }` (derives-based CLI parsing)
- **Entry Point**: `/crates/bazbom/src/main.rs` (614 lines)
- **CLI Definition**: `/crates/bazbom/src/cli.rs` (874 lines) - defines all commands via enums

### Command Categories

#### Primary Scan Commands (main.rs lines 160-401)
1. **`scan`** (Lines 160-227) - Main command with full options
   - Path, profile, reachability, fast mode, format, output dir
   - Bazel-specific options (targets, queries, universe)
   - SCA/SAST options (semgrep, codeql, autofix, containers)
   - Incremental & diff modes
   - Benchmark, ML risk, Jira/GitHub integration

2. **`check`** (Lines 230-283) - Quick local scan
   - Auto-detects main module if path is "."
   - Fast mode enabled, no upload, 10 second target

3. **`ci`** (Lines 285-321) - CI-optimized scan
   - JSON + SARIF output, no reachability (too slow for CI)
   - Fast mode enabled

4. **`pr`** (Lines 323-363) - PR-optimized scan
   - Incremental + diff mode
   - Compares with base branch

5. **`full`** (Lines 365-401) - Full scan with all features
   - **KEY IMPLEMENTATION**: Lines 365-401 in main.rs
   - Enables: reachability=true, cyclonedx=true, benchmark=true, ml_risk=true
   - Path parameter (default ".")
   - Output directory parameter (default ".")

6. **`quick`** (Lines 403-439) - Super-fast smoke test
   - < 5 seconds target, fast mode, auto-detected target

#### Specialized Commands
- `container-scan` - Container SBOM + vulnerability scanning
- `policy` - Policy checks
- `fix` - Remediation suggestions
- `license` - License compliance
- `init` - Setup wizard
- `explore` - Interactive graph explorer
- `dashboard` - Web dashboard
- `install-hooks` - Git pre-commit hooks
- `install` - CI/CD templates
- `db` - Advisory database sync
- `team` - Team coordination
- `report` - Security reports
- `explain` - CVE details
- `status` - Security overview
- `compare` - Branch comparison
- `watch` - Continuous monitoring
- `jira` - Jira integration (v6.8)
- `github` - GitHub integration (v6.8)

### Command Handler Pattern
All commands delegate to handler functions in `/crates/bazbom/src/commands/` directory:
- `handle_scan()` - Main entry point for all scan variants
- `handle_policy()`, `handle_fix()`, `handle_license()`, etc.

---

## 3. MAIN SCAN FLOW & KEY OPERATIONS

### Scan Entry Point: `handle_scan()` in `/crates/bazbom/src/commands/scan.rs`

#### Phase 1: Smart Defaults (Lines 43-80)
- Auto-detect CI environment
- Auto-enable JSON for CI
- Auto-enable reachability for small repos
- Auto-enable incremental for PRs
- Auto-enable diff if baseline exists
- **Debug Opportunity**: Log what defaults were detected

#### Phase 2: Profile Loading (Lines 82-88)
- Load `bazbom.toml` profile if specified
- **Debug Opportunity**: Log which profile was loaded and applied settings

#### Phase 3: Diff Mode (Lines 90-99)
- Compare with baseline findings
- **Debug Opportunity**: Log baseline path and findings count

#### Phase 4: JSON Mode Setup (Lines 101-105)
- Set environment variable for JSON output
- **Debug Opportunity**: Log when JSON mode is enabled

#### Phase 5: Orchestrated Scan (Lines 107-138)
**Uses `ScanOrchestrator`** when any of these flags are set:
- `cyclonedx` - Additional output format
- `with_semgrep` - SAST analysis
- `with_codeql` - Security analysis
- `autofix` - Automatic fixes
- `containers` - Container scanning

### ScanOrchestrator Flow: `/crates/bazbom/src/scan_orchestrator.rs` (1259 lines)

#### Step 0: Incremental Scan Check (Lines 96-107)
- Uses `IncrementalAnalyzer` from `bazbom_cache`
- Checks if significant changes detected
- **Debug Opportunity**: Log what files changed, why cache hit/miss

#### Step 0.5: Scan Cache Check (Lines 109-127)
- Checks for previous scan cache
- Uses cached results if available (fast path)
- Can be disabled via `BAZBOM_DISABLE_CACHE=1`
- **Debug Opportunity**: Log cache key, cache size, cache validity

#### Step 1: SBOM Generation (Lines 144-155)
- Calls `generate_sbom()` method
- Detects build system (Maven, Gradle, Bazel, etc.)
- Parses dependency files
- Generates SPDX/CycloneDX SBOM
- **Debug Opportunity**: Log build system detection, number of files parsed, dependency tree structure

#### Step 2: SCA Analysis (Lines 160-181)
- Uses `ScaAnalyzer`
- **Operations**:
  - Syncs advisory database if > 24 hours old (lines 29-51 in analyzers/sca.rs)
  - Loads SBOM components from `sbom/spdx.json`
  - Queries vulnerability database
  - Enriches with EPSS scores, KEV catalog
  - Generates SARIF report
- **Debug Opportunity**: Log advisory DB sync progress, number of vulnerabilities found, enrichment details

#### Step 3: Semgrep SAST (Lines 184-199, if enabled)
- Uses `SemgrepAnalyzer`
- Scans code patterns with curated JVM ruleset
- **Debug Opportunity**: Log rules loaded, patterns matched, findings count

#### Step 4: CodeQL Analysis (if enabled)
- Uses `CodeqlAnalyzer`
- Runs security queries
- **Debug Opportunity**: Log analysis start/end, queries executed, results

#### Step 5: Threat Detection (if enabled)
- Uses `ThreatAnalyzer`
- **Debug Opportunity**: Log threat patterns evaluated, threats found

#### Step 6: Performance Monitoring (Lines 82-86 if enabled)
- Uses `PerformanceMonitor` struct
- Tracks phase durations, dependency count, vulnerability count
- **Debug Opportunity**: Already structured but needs logging integration

### Key Operations For Debug Logging

1. **Build System Detection**
   - File: `/crates/bazbom/src/bazel.rs`, etc.
   - Operations: Detecting Maven, Gradle, Bazel, npm, Python, Go, Rust projects
   - Log: Which files trigger detection, confidence level

2. **Dependency Parsing**
   - Files: Bazel (bazel.rs), language-specific parsers in bazbom-polyglot
   - Operations: Parsing pom.xml, build.gradle, package.json, go.mod, Cargo.toml, etc.
   - Log: Files being parsed, parsing errors, dependency counts per level

3. **Advisory Database Operations**
   - File: `/crates/bazbom/src/analyzers/sca.rs` (lines 29-54)
   - Operations: Checking manifest age, syncing from remote, loading scores
   - Log: Sync start/end, data size downloaded, manifest path, age check results

4. **Vulnerability Matching**
   - File: `/crates/bazbom/src/analyzers/sca.rs` (lines 56-102)
   - Operations: Loading SBOM components, extracting PURL, querying advisories
   - Log: Components loaded, version matching logic, score enrichment

5. **Caching & Incremental**
   - Files: `/crates/bazbom/src/scan_cache.rs`, `/crates/bazbom/src/incremental.rs`
   - Operations: Cache key generation, file change detection, cache validation
   - Log: Cache paths, file checksums, change reasons

6. **Remediation & Autofix**
   - Files: `/crates/bazbom/src/remediation/`, `/crates/bazbom/src/fixes/`
   - Operations: OpenRewrite recipe generation, version resolution, PR creation
   - Log: Recipe selection, version constraints, API calls to GitHub/Jira

7. **Performance Phases**
   - File: `/crates/bazbom/src/performance.rs`
   - Tracks: SBOM generation, vulnerability scan, reachability, threat detection
   - Log: Phase durations, bottleneck identification

---

## 4. CLI ARGUMENT PARSING

### Clap Derive Macro System
**File**: `/crates/bazbom/src/cli.rs` (874 lines)

### Main Entry Point
```rust
#[derive(Parser, Debug)]
#[command(name = "bazbom", version, about = "...", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}
```

### Commands Enum Structure
**Example (Full command definition)**: Lines 220-245 in cli.rs
```rust
#[derive(Subcommand, Debug)]
pub enum Commands {
    Full {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Output directory
        #[arg(long, short = 'o', default_value = ".")]
        out_dir: String,
    },
    // ... other commands
}
```

### Parsing Flow
1. **Line 124** in main.rs: `let cli = Cli::parse();`
   - Clap parses CLI arguments automatically from struct definitions
   
2. **Lines 125-157** in main.rs: Convert to default Scan command if no command specified
   - Default command is `Commands::Scan` with default parameters
   
3. **Lines 159-613** in main.rs: Match statement routes to handler functions
   - Each command variant has its handler
   - Handler extracts parameters and calls functions like `handle_scan()`

### Argument Features Used
- `#[arg(default_value = "...")]` - Default values
- `#[arg(long, short = 'x')]` - Long and short flags
- `#[arg(value_enum)]` - Enumeration values (CodeqlSuite, AutofixMode, ContainerStrategy)
- `#[arg(num_args = 1..)]` - Variable length arguments
- `#[arg(long, value_name = "...")]` - Documentation for values
- `#[command(after_help = "...")]` - Contextual help text

### Important Value Enums
1. **CodeqlSuite** (lines 564-577): `default` or `security-extended`
2. **AutofixMode** (lines 579-594): `off`, `dry-run`, or `pr`
3. **ContainerStrategy** (lines 596-611): `auto`, `syft`, or `bazbom`
4. **ComplianceFrameworkArg** (lines 772-781): PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST CSF

---

## 5. KEY FILE LOCATIONS & SIZES

### Largest/Most Complex Files
| File | Lines | Purpose |
|------|-------|---------|
| `/crates/bazbom/src/commands/container_scan.rs` | 3428 | Container scanning implementation |
| `/crates/bazbom/src/shading.rs` | 1300 | JAR shading/repackaging logic |
| `/crates/bazbom/src/scan_orchestrator.rs` | 1259 | Main scan orchestration |
| `/crates/bazbom/src/bazel.rs` | 896 | Bazel build system support |
| `/crates/bazbom/src/cli.rs` | 874 | CLI command definitions |
| `/crates/bazbom/src/main.rs` | 614 | Entry point, command routing |
| `/crates/bazbom/src/remediation/suggestions.rs` | 606 | Fix recommendations |
| `/crates/bazbom/src/analyzers/sca.rs` | 682 | Vulnerability scanning |

### Core Modules
- `/crates/bazbom-core/` - SBOM generation, build system detection
- `/crates/bazbom-formats/` - SPDX, CycloneDX, SARIF serialization
- `/crates/bazbom-advisories/` - Vulnerability database handling
- `/crates/bazbom-polyglot/` - Multi-language dependency parsing
- `/crates/bazbom-cache/` - Incremental & caching logic
- `/crates/bazbom-graph/` - Dependency graph analysis

---

## 6. RECOMMENDED DEBUG LOGGING POINTS

### High Priority (Core Flow)
1. **Smart Defaults Detection** - What environment was detected
2. **Build System Detection** - Which build system, confidence
3. **Profile Loading** - Which profile, what settings applied
4. **Cache Status** - Cache hit/miss, key, reason
5. **SBOM Generation Start/End** - Build system, file counts
6. **Advisory DB Sync** - Download size, time, version
7. **Vulnerability Matching** - Components processed, matches found
8. **Phase Timing** - Start/end of each major phase

### Medium Priority (Detailed Analysis)
1. **Dependency Resolution** - Version ranges, conflict resolution
2. **PURL Extraction** - Coordinate mapping, ecosystem detection
3. **Semgrep/CodeQL Execution** - Rules loaded, patterns evaluated
4. **Enrichment Operations** - EPSS scores, KEV lookups
5. **Remediation Suggestion** - Version constraints, compatibility checks
6. **Reachability Analysis** - Call graph construction, filtering

### Low Priority (Implementation Details)
1. **File I/O Operations** - Read/write paths, sizes
2. **Configuration Parsing** - TOML structure, errors
3. **Error Recovery** - Fallback paths, retries
4. **Thread Pool Activity** - Task distribution, parallelization

---

## 7. CURRENT STATE SUMMARY

### What's Working
- ✅ Full command structure and routing
- ✅ Smart defaults for CI/PR/dev scenarios
- ✅ Progress indication with colored output
- ✅ Caching system with incremental support
- ✅ Multiple output formats (SPDX, CycloneDX, SARIF, JSON)
- ✅ Performance monitoring infrastructure
- ✅ Configuration profiles via bazbom.toml

### What Needs Debugging Support
- ⚠️ No centralized logging setup
- ⚠️ Tracing crate imported but not initialized
- ⚠️ Missing debug logs for critical operations
- ⚠️ Cache hit/miss reasons not logged
- ⚠️ Build system detection process not visible
- ⚠️ Dependency parsing errors not detailed
- ⚠️ Advisory DB operations not tracked

---

## Implementation Notes for Adding Debug Logging

### Setup Required
1. Add `tracing-subscriber` dependency with `env_filter` feature
2. Initialize logging in main.rs before parsing CLI
3. Check for `RUST_LOG` environment variable
4. Use `#[instrument]` macro for function tracing

### Recommended Debug Log Levels
- **ERROR**: Failures, missing files, API errors
- **WARN**: Slow operations, cache misses, fallback paths
- **INFO**: Phase start/end, key findings (already using some)
- **DEBUG**: Detailed operations, parameter values, decisions
- **TRACE**: Low-level details, loops, recursion

### Environment Configuration
```bash
# Enable all debug logs
RUST_LOG=debug bazbom scan .

# Enable specific module
RUST_LOG=bazbom::scan_orchestrator=debug bazbom scan .

# Combine with existing env vars
BAZBOM_NO_SMART_DEFAULTS=1 RUST_LOG=debug bazbom full .
```

