# BazBOM Full Command - Complete Execution Order

This document describes the **exact order of operations** when running `bazbom full`, tracing through every step from command invocation to completion.

## Command Invocation

```bash
bazbom full [--limit N] [-o OUTPUT_DIR]
```

---

## Phase 0: Initialization (main.rs:122-137)

### Step 0.1: Logger Initialization
**File**: `crates/bazbom/src/main.rs:127-136`

```rust
tracing_subscriber::fmt()
    .with_env_filter(...)
    .init();
```

**What it does**:
- Initializes structured logging
- Reads `RUST_LOG` environment variable (defaults to "info")
- Configures log output format (no targets, no thread IDs, no file/line numbers)

**Debug logging**: Use `RUST_LOG=debug bazbom full` to see detailed logs

---

### Step 0.2: CLI Argument Parsing
**File**: `crates/bazbom/src/main.rs:138`

Parses command-line arguments using clap and matches the `Full` command variant.

---

### Step 0.3: Command Dispatch
**File**: `crates/bazbom/src/main.rs:366-402`

**Actions**:
1. Prints header: `"💪 Running FULL scan with ALL features enabled..."`
2. If `--limit` is specified, prints: `"ℹ️  Limiting scan to {N} packages/targets"`
3. Calls `handle_scan()` with these parameters:
   ```rust
   reachability: true       // Enable OPAL reachability analysis
   fast: false              // Full analysis, no shortcuts
   cyclonedx: true          // Generate CycloneDX SBOM
   benchmark: true          // Enable performance monitoring
   ml_risk: true            // Enable ML-enhanced risk scoring
   ```

---

## Phase 1: Scan Handler (commands/scan.rs:11-224)

### Step 1.1: Debug Logging Start
**File**: `crates/bazbom/src/commands/scan.rs:45-47`

```
DEBUG Starting scan with path: .
DEBUG Scan options - reachability: true, fast: false, format: spdx, incremental: false
```

---

### Step 1.2: Limit Processing
**File**: `crates/bazbom/src/commands/scan.rs:49-53`

If `--limit N` was specified:
```
INFO Scan limit enabled: will process maximum N packages/targets
```
Sets environment variable: `BAZBOM_SCAN_LIMIT=N`

---

### Step 1.3: Smart Defaults Detection
**File**: `crates/bazbom/src/commands/scan.rs:55-93`

**Actions**:
1. Detects if running in CI environment
2. Detects if in PR context
3. Checks repository size for reachability enablement
4. Auto-enables features based on environment

**Debug logs**:
```
DEBUG Detecting smart defaults for environment
DEBUG Smart defaults detected - is_ci: false, is_pr: false, enable_reachability: false
```

---

### Step 1.4: Profile Loading (Skipped for Full)
**File**: `crates/bazbom/src/commands/scan.rs:101-110`

For `bazbom full`, profile is `None`, so this is skipped.

---

### Step 1.5: Orchestrator Decision
**File**: `crates/bazbom/src/commands/scan.rs:132-168`

**Check**: Is `cyclonedx=true` OR `with_semgrep=true` OR other orchestration flags?

For `bazbom full`: **YES** (cyclonedx=true)

**Actions**:
```
INFO Using orchestrated scan mode
DEBUG Orchestrator options - cyclonedx: true, with_semgrep: false, ...
```

Prints: `"[bazbom] using orchestrated scan mode"`

**Creates** `ScanOrchestrator` and calls `run()`

---

## Phase 2: Scan Orchestrator (scan_orchestrator.rs:78-408)

### Step 2.1: Performance Monitor Setup
**File**: `crates/bazbom/src/scan_orchestrator.rs:82-86`

Since `benchmark=true`:
- Creates `PerformanceMonitor` to track timing metrics

---

### Step 2.2: Configuration Display
**File**: `crates/bazbom/src/scan_orchestrator.rs:88-94`

Prints:
```
   ℹ️ CycloneDX output enabled
```

---

### Step 2.3: Incremental Scan Check (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:96-107`

For `bazbom full`, `incremental=false`, so this is skipped.

---

### Step 2.4: Cache Check
**File**: `crates/bazbom/src/scan_orchestrator.rs:109-127`

**Actions**:
1. Checks if `BAZBOM_DISABLE_CACHE` is set
2. If not, tries to use cached results from `.bazbom/cache/`
3. Generates cache key based on:
   - Workspace path
   - Build files (pom.xml, build.gradle, etc.)
   - Scan parameters (reachability, format, etc.)

**Possible outcomes**:
- **Cache hit**: Returns early with cached results
- **Cache miss**: Continues with full scan

---

### Step 2.5: Progress Phase Setup
**File**: `crates/bazbom/src/scan_orchestrator.rs:129-142`

Builds phase list:
```
phases = ["SBOM Generation", "SCA Analysis"]
```

Creates progress tracker for visual feedback.

---

## Phase 3: SBOM Generation (scan_orchestrator.rs:144-155)

### Step 3.1: Phase Start
```
Analyzing dependencies...
```

Performance monitor starts tracking: `"sbom_generation"`

---

### Step 3.2: Build System Detection
**File**: `crates/bazbom/src/scan_orchestrator.rs:1082-1084`

Detects JVM build system:
- **Maven**: Looks for `pom.xml`
- **Gradle**: Looks for `build.gradle` or `build.gradle.kts`
- **Bazel**: Looks for `BUILD.bazel` or `BUILD`

Prints: `"[bazbom] detected JVM build system: Maven"` (or Gradle/Bazel)

---

### Step 3.3: Polyglot Ecosystem Scanning
**File**: `crates/bazbom/src/scan_orchestrator.rs:1087-1164`

**Actions**:
1. Scans workspace for non-JVM ecosystems:
   - Node.js/npm (package.json)
   - Python (requirements.txt, pyproject.toml)
   - Go (go.mod)
   - Rust (Cargo.toml)
   - Ruby (Gemfile)
   - PHP (composer.json)

2. For each detected ecosystem:
   - Parses dependency files
   - Fetches vulnerability data
   - Reports findings

**Output example**:
```
📦 Detected 2 polyglot ecosystems:
  🐍 Python - 15 packages, 2 vulnerabilities
    🚨 0 CRITICAL, 1 HIGH, 1 MEDIUM, 0 LOW
  📦 Node.js/npm - 42 packages, 0 vulnerabilities
```

---

### Step 3.4: JVM SPDX SBOM Generation
**File**: `crates/bazbom/src/scan_orchestrator.rs:1177-1179`

**Actions**:
1. Generates SPDX 2.3 SBOM for JVM dependencies
2. Writes to: `{out_dir}/sbom/spdx.json`

Prints: `"[bazbom] wrote JVM SPDX SBOM to .../sbom/spdx.json"`

**THIS IS WHERE `--limit` SHOULD BE APPLIED** (currently not implemented in downstream SBOM generation)

---

### Step 3.5: CycloneDX SBOM Generation
**File**: `crates/bazbom/src/scan_orchestrator.rs:1182-1190`

Since `cyclonedx=true` for full scan:

**Actions**:
1. Converts SPDX to CycloneDX format
2. Writes to: `{out_dir}/sbom/cyclonedx.json`

Prints: `"[bazbom] wrote CycloneDX SBOM to .../sbom/cyclonedx.json"`

---

### Step 3.6: Polyglot SBOM Generation
**File**: `crates/bazbom/src/scan_orchestrator.rs:1167-1175`

If polyglot ecosystems were detected:

**Actions**:
1. Generates unified SBOM combining all ecosystems
2. Writes to: `{out_dir}/sbom/polyglot-sbom.json`

Prints: `"[bazbom] wrote polyglot SBOM to .../sbom/polyglot-sbom.json"`

---

### Step 3.7: Phase Complete
```
Parsing build files... (30% progress)
Complete ✓
```

Performance monitor ends tracking: `"sbom_generation"`

---

## Phase 4: SCA (Software Composition Analysis) (scan_orchestrator.rs:160-181)

### Step 4.1: Phase Start
```
Fetching vulnerability data...
```

Performance monitor starts tracking: `"vulnerability_scan"`

---

### Step 4.2: SCA Analyzer Execution
**File**: `crates/bazbom/src/scan_orchestrator.rs:165-177`

**Actions**:
1. Creates `ScaAnalyzer`
2. Syncs advisory database (OSV, NVD, GitHub Security Advisories)
3. Matches dependencies against vulnerability database
4. Filters based on:
   - Severity
   - Reachability (if enabled)
   - Exploitability

**Progress updates**:
```
Analyzing vulnerabilities... (50% progress)
```

---

### Step 4.3: Reachability Analysis (If Enabled)
**For `bazbom full`, reachability=true**

**What happens**:
1. Loads SBOM and identifies vulnerable packages
2. For each vulnerability:
   - Runs language-specific static analysis (OPAL for JVM)
   - Traces call graphs from application entry points
   - Determines if vulnerable code is actually reachable
3. Tags vulnerabilities as "reachable" or "unreachable"

**Impact**: Typically reduces noise by 70-90%

---

### Step 4.4: Report Generation
**File**: `crates/bazbom/src/scan_orchestrator.rs:169-172`

**Actions**:
1. Generates SARIF report with vulnerability findings
2. Adds to reports list for merging

Prints: `"Complete ✓"`

Performance monitor ends tracking: `"vulnerability_scan"`

---

## Phase 5: Optional Analyzers (scan_orchestrator.rs:183-246)

### Step 5.1: Semgrep (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:184-200`

For `bazbom full`, `with_semgrep=false`, so **skipped**.

---

### Step 5.2: CodeQL (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:202-219`

For `bazbom full`, `with_codeql=None`, so **skipped**.

---

### Step 5.3: Threat Intelligence
**File**: `crates/bazbom/src/scan_orchestrator.rs:221-246`

**Default**: `ThreatDetectionLevel::Standard`

**Actions**:
1. Enriches vulnerability data with threat intelligence:
   - CISA KEV (Known Exploited Vulnerabilities)
   - EPSS scores (Exploit Prediction Scoring System)
   - Active exploitation indicators
2. Prioritizes vulnerabilities based on real-world risk

Performance monitor tracks: `"threat_intelligence"`

---

## Phase 6: Enrichment (scan_orchestrator.rs:248-251)

### Step 6.1: deps.dev Enrichment (If Enabled)
**File**: `crates/bazbom/src/scan_orchestrator.rs:410-524`

**If** `config.enrich.depsdev = true` in `bazbom.toml`:

**Actions**:
1. Extracts PURLs from SBOM
2. Queries Google's deps.dev API for each package:
   - Latest version available
   - Scorecard metrics
   - License information
   - Dependency insights
3. Rate-limits requests (200ms delay between calls)
4. Writes enrichment data to: `{out_dir}/enrich/depsdev.json`

**Output**:
```
[bazbom] found 37 components with PURLs
[bazbom]   enriched: commons-io (latest: 2.15.1)
[bazbom]   enriched: log4j-core (latest: 2.22.1)
...
[bazbom] enriched 35/37 components
```

---

## Phase 7: Container Scanning (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:253-256`

For `bazbom full`, `containers=None`, so **skipped**.

---

## Phase 8: Report Merging (scan_orchestrator.rs:258-273)

### Step 8.1: SARIF Merging
**File**: `crates/bazbom/src/scan_orchestrator.rs:262-273`

**Actions**:
1. Merges all analyzer reports into single SARIF file
2. Combines:
   - SCA findings (vulnerabilities)
   - Threat intelligence enrichment
   - Any optional analyzer results
3. Writes to: `{out_dir}/findings/merged.sarif`

**Output**:
```
[bazbom] wrote merged SARIF to .../findings/merged.sarif
[bazbom] total runs in merged report: 1
```

---

## Phase 9: Autofix (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:275-278`

For `bazbom full`, `autofix=None`, so **skipped**.

---

## Phase 10: GitHub Upload (scan_orchestrator.rs:280-298)

### Step 10.1: Upload Check
**File**: `crates/bazbom/src/scan_orchestrator.rs:281-295`

For `bazbom full`, `no_upload=false`, so **attempts upload**.

**Actions**:
1. Checks if GitHub token is configured (`GITHUB_TOKEN` env var)
2. If configured, uploads `merged.sarif` to GitHub Code Scanning
3. If not configured, prints help message

**Possible outputs**:
- **Configured**: `"[bazbom] GitHub Code Scanning upload configured"`
- **Not configured**: `"[bazbom] GitHub upload not configured (use github/codeql-action/upload-sarif@v3)"`

---

## Phase 11: Caching & Finalization (scan_orchestrator.rs:300-394)

### Step 11.1: Save Scan Commit (Skipped for Full)
**File**: `crates/bazbom/src/scan_orchestrator.rs:300-305`

For `bazbom full`, `incremental=false`, so **skipped**.

---

### Step 11.2: Cache Storage
**File**: `crates/bazbom/src/scan_orchestrator.rs:307-313`

Unless `BAZBOM_DISABLE_CACHE=1`:

**Actions**:
1. Stores scan results in `.bazbom/cache/`
2. Cache key based on:
   - Build file hashes
   - Scan parameters
   - Workspace path
3. Stores:
   - SBOM JSON
   - Findings JSON
   - Scan parameters

**Output**:
```
[bazbom] cached scan results (key: a3f2d8e1b4c7...)
```

---

### Step 11.3: Progress Summary
**File**: `crates/bazbom/src/scan_orchestrator.rs:315-317`

```
✨ Scan complete! Generated 1 reports
```

---

### Step 11.4: Performance Metrics Display
**File**: `crates/bazbom/src/scan_orchestrator.rs:319-394`

**Since `benchmark=true` for full scan**:

Displays performance breakdown:

```
[bazbom]
[bazbom] ╔══════════════════════════════════════════════════════════╗
[bazbom] ║            Performance Metrics                           ║
[bazbom] ╠══════════════════════════════════════════════════════════╣
[bazbom] ║  Total Duration: 47.2s                                   ║
[bazbom] ║    SBOM Generation       12.3s                   (26.1%) ║
[bazbom] ║    Vulnerability Scan    28.7s                   (60.8%) ║
[bazbom] ║    Threat Detection       6.2s                   (13.1%) ║
[bazbom] ╠══════════════════════════════════════════════════════════╣
[bazbom] ║  Dependencies: 127                                       ║
[bazbom] ║  Vulnerabilities: 8                                      ║
[bazbom] ║  Build System: Maven                                     ║
[bazbom] ╚══════════════════════════════════════════════════════════╝
[bazbom] Performance metrics saved to: .../performance.json
```

---

### Step 11.5: Next Steps Guidance
**File**: `crates/bazbom/src/scan_orchestrator.rs:396-405`

```
[bazbom] orchestrated scan complete
[bazbom] outputs in: "."
[bazbom]
[bazbom] Next steps:
[bazbom]   - Review findings in: "./findings"
[bazbom]   - Upload SARIF: github/codeql-action/upload-sarif@v3
[bazbom]   - Archive artifacts: actions/upload-artifact@v4
```

---

## Output Artifacts

After `bazbom full` completes, the following files are created:

### SBOMs
- `{out_dir}/sbom/spdx.json` - JVM SPDX 2.3 SBOM
- `{out_dir}/sbom/cyclonedx.json` - CycloneDX SBOM
- `{out_dir}/sbom/polyglot-sbom.json` - Combined multi-language SBOM (if polyglot)

### Findings
- `{out_dir}/findings/merged.sarif` - SARIF vulnerability report
- `{out_dir}/findings/sca_findings.json` - Detailed SCA findings (JSON)

### Enrichment (if enabled)
- `{out_dir}/enrich/depsdev.json` - deps.dev enrichment data

### Performance (benchmark=true)
- `{out_dir}/performance.json` - Timing metrics and statistics

### Cache
- `.bazbom/cache/{cache_key}/` - Cached scan results for future runs

---

## Summary: Key Differences of `bazbom full`

| Feature | `bazbom scan` | `bazbom full` |
|---------|---------------|---------------|
| Reachability Analysis | ❌ Optional | ✅ Enabled |
| CycloneDX SBOM | ❌ Optional | ✅ Generated |
| ML Risk Scoring | ❌ Optional | ✅ Enabled |
| Performance Metrics | ❌ Optional | ✅ Displayed |
| Threat Intelligence | Standard | Standard |
| Speed | Faster | Slower (comprehensive) |
| Noise Reduction | None | 70-90% via reachability |

---

## Critical Note About `--limit`

**Current Implementation**: The `--limit` parameter is **set as an environment variable** (`BAZBOM_SCAN_LIMIT`) but **not yet enforced** in the SBOM generation step.

**To Fully Implement**:
Modify `crates/bazbom/src/scan_orchestrator.rs:1079-1193` (generate_sbom function) to:
1. Check for `BAZBOM_SCAN_LIMIT` environment variable
2. Truncate dependency lists after SBOM generation
3. Log which packages were included/excluded

**Recommendation**:
```rust
// In generate_sbom(), after detecting dependencies:
if let Ok(limit_str) = std::env::var("BAZBOM_SCAN_LIMIT") {
    if let Ok(limit) = limit_str.parse::<usize>() {
        dependencies.truncate(limit);
        println!("[bazbom] limited to {} dependencies", limit);
    }
}
```

---

## Debug Logging Locations

With `RUST_LOG=debug bazbom full`, you'll see debug logs at these key points:

| Line | File | Message |
|------|------|---------|
| scan.rs:45 | commands/scan.rs | Starting scan with path |
| scan.rs:46 | commands/scan.rs | Scan options |
| scan.rs:50 | commands/scan.rs | Scan limit enabled |
| scan.rs:55-58 | commands/scan.rs | Smart defaults detection |
| scan.rs:70 | commands/scan.rs | Auto-enabled JSON output |
| scan.rs:79 | commands/scan.rs | Auto-enabled reachability |
| scan.rs:85 | commands/scan.rs | Auto-enabled incremental mode |
| scan.rs:91 | commands/scan.rs | Auto-enabled diff mode |
| scan.rs:102-109 | commands/scan.rs | Profile loading |
| scan.rs:115 | commands/scan.rs | Running diff mode |
| scan.rs:127 | commands/scan.rs | Enabling JSON output mode |
| scan.rs:141-148 | commands/scan.rs | Orchestrator options |
| scan.rs:172 | commands/scan.rs | Using legacy scan mode |
| scan.rs:189-192 | commands/scan.rs | Scan completion status |
| scan.rs:199-201 | commands/scan.rs | Auto-remediation config |

---

## Questions to Verify Implementation

1. ✅ **Is logging initialized?** Yes, at main.rs:127-136
2. ✅ **Is limit parameter parsed?** Yes, in cli.rs:137-138, 249-250
3. ✅ **Is limit stored as env var?** Yes, in scan.rs:52
4. ❌ **Is limit enforced in SBOM generation?** **NO - needs implementation**
5. ✅ **Is orchestrator used for full?** Yes, because cyclonedx=true
6. ✅ **Is reachability enabled?** Yes, set to true in main.rs:374
7. ✅ **Are benchmarks displayed?** Yes, in scan_orchestrator.rs:319-394
8. ✅ **Is CycloneDX generated?** Yes, in scan_orchestrator.rs:1182-1190

