# Debug Logging Implementation Quick Start Guide

This document provides a quick reference for implementing debug logging in BazBOM. For detailed analysis, see `EXPLORATION_SUMMARY.md` and `FLOW_DIAGRAMS.md`.

## Current State

**Logging Framework**: `tracing = "0.1"` (partially integrated, not fully initialized)

**Current Approach**: 1358 println!/eprintln! statements + progress bars via indicatif

**Key Issue**: No centralized logging setup, making it hard to enable/disable debug output

## Where to Find Things

| What | File | Lines |
|------|------|-------|
| CLI Command Parsing | `/crates/bazbom/src/cli.rs` | 1-100 (Cli struct) |
| 'full' Command Definition | `/crates/bazbom/src/cli.rs` | 220-245 |
| 'full' Command Handler | `/crates/bazbom/src/main.rs` | 365-401 |
| Main Entry Point | `/crates/bazbom/src/main.rs` | 122-157 |
| Scan Orchestration | `/crates/bazbom/src/scan_orchestrator.rs` | 78-300+ |
| Smart Defaults | `/crates/bazbom/src/commands/scan.rs` | 43-80 |
| SBOM Generation | `/crates/bazbom/src/scan_orchestrator.rs` | (method) |
| SCA Analysis | `/crates/bazbom/src/analyzers/sca.rs` | 1-100+ |
| Advisory DB Sync | `/crates/bazbom/src/analyzers/sca.rs` | 29-54 |
| Performance Monitoring | `/crates/bazbom/src/performance.rs` | 1-100+ |
| Progress Display | `/crates/bazbom/src/progress.rs` | 1-100+ |
| Caching Logic | `/crates/bazbom/src/scan_cache.rs` | (entire file) |
| Incremental Analysis | `/crates/bazbom/src/incremental.rs` | (entire file) |
| Configuration | `/crates/bazbom/src/config.rs` | 1-100+ |

## Top 10 Operations for Debug Logging

### 1. Smart Defaults Detection
**File**: `/crates/bazbom/src/commands/scan.rs` lines 43-80
**What to Log**: 
- Environment detected (CI, PR, dev)
- Features auto-enabled
- Reasoning for each decision

### 2. Build System Detection  
**File**: `/crates/bazbom/src/bazel.rs` and bazbom-core
**What to Log**:
- Which files checked and found
- Detected build system type
- Confidence level (if multiple detected)

### 3. Profile Loading
**File**: `/crates/bazbom/src/commands/scan.rs` lines 82-88
**What to Log**:
- Profile file path
- Profile name loaded
- Settings from profile
- Settings from CLI (overrides)

### 4. Cache Hit/Miss Decision
**File**: `/crates/bazbom/src/scan_orchestrator.rs` lines 109-127
**What to Log**:
- Cache key used
- Cache directory path
- Hit or miss reason
- File sizes of cached files

### 5. SBOM Generation
**File**: `/crates/bazbom/src/scan_orchestrator.rs` lines 144-155
**What to Log**:
- Build system detected
- Files being parsed
- Dependency count
- Generation time

### 6. Advisory Database Operations
**File**: `/crates/bazbom/src/analyzers/sca.rs` lines 29-54
**What to Log**:
- DB sync decision (age check)
- Download start/end
- Data size downloaded
- Records processed
- Time taken

### 7. Vulnerability Matching
**File**: `/crates/bazbom/src/analyzers/sca.rs` lines 56-150+
**What to Log**:
- Components loaded count
- Vulnerabilities queried
- Matches found per component
- EPSS/KEV enrichment progress

### 8. Semgrep SAST Execution
**File**: `/crates/bazbom/src/analyzers/semgrep.rs`
**What to Log**:
- Rules count loaded
- Patterns evaluated
- Findings count
- Execution time

### 9. Remediation Suggestion
**File**: `/crates/bazbom/src/remediation/suggestions.rs`
**What to Log**:
- Vulnerability analyzed
- Version ranges checked
- Candidate versions tested
- Suggestion reasoning

### 10. Report Merging & Output
**File**: `/crates/bazbom/src/scan_orchestrator.rs` lines 200+
**What to Log**:
- Reports being merged
- Output files created
- Total findings
- File paths written

## Environment Variables to Check/Support

| Variable | Current Use | Proposed Use |
|----------|-------------|--------------|
| `BAZBOM_JSON_MODE` | Enables JSON output | Keep as-is |
| `BAZBOM_NO_SMART_DEFAULTS` | Disables smart defaults | Keep as-is |
| `BAZBOM_DISABLE_CACHE` | Disables caching | Add logging about this |
| `RUST_LOG` | Not yet used | **ADD**: Tracing logging level |

## Implementation Checklist

### Phase 1: Setup (Prerequisite)
- [ ] Add `tracing-subscriber` with `env_filter` feature to Cargo.toml
- [ ] Initialize tracing in `main.rs` before parsing CLI
- [ ] Test with `RUST_LOG=debug bazbom scan .`

### Phase 2: Critical Paths (High Priority)
- [ ] Add logs to smart defaults detection (commands/scan.rs:43-80)
- [ ] Add logs to cache check (scan_orchestrator.rs:109-127)
- [ ] Add logs to SBOM generation (scan_orchestrator.rs:144-155)
- [ ] Add logs to advisory DB sync (analyzers/sca.rs:29-54)
- [ ] Add logs to vulnerability matching (analyzers/sca.rs:56-102)

### Phase 3: Detailed Flows (Medium Priority)
- [ ] Add logs to profile loading (commands/scan.rs:82-88)
- [ ] Add logs to incremental check (scan_orchestrator.rs:96-107)
- [ ] Add logs to Semgrep execution
- [ ] Add logs to CodeQL execution
- [ ] Add logs to remediation suggestions

### Phase 4: Implementation Details (Low Priority)
- [ ] Add logs to file I/O operations
- [ ] Add logs to configuration parsing
- [ ] Add logs to error recovery paths
- [ ] Add logs to parallel operations

## Code Patterns to Follow

### For Existing println! Statements
Convert these:
```rust
println!("[bazbom] scanning advisory database...");
```

To:
```rust
tracing::debug!("Starting advisory database scan");
```

### For Decisions/Branches
```rust
tracing::debug!(
    "Cache decision: hit={}, size={}, age_seconds={}", 
    cache_hit, 
    cache_size, 
    cache_age.as_secs()
);
```

### For Phase Transitions
```rust
tracing::info!("Starting SBOM generation");
// ... work ...
tracing::info!("Completed SBOM generation in {:?}", elapsed);
```

### For Complex Operations
```rust
#[tracing::instrument(skip(sbom), fields(components_count = sbom.packages.len()))]
fn analyze_vulnerabilities(sbom: &SpdxDocument) -> Result<Vec<Finding>> {
    tracing::debug!("Querying advisory database");
    // ...
}
```

## Testing Debug Logging

Once implemented, test with:

```bash
# Enable all debug logs
RUST_LOG=debug bazbom scan .

# Enable specific module
RUST_LOG=bazbom::scan_orchestrator=debug bazbom scan .

# Enable full command
RUST_LOG=bazbom::commands::scan=debug bazbom scan .

# Multiple modules
RUST_LOG=bazbom::scan_orchestrator=debug,bazbom::analyzers::sca=debug bazbom full .

# With color output
RUST_LOG=debug cargo run -- scan .

# JSON output for parsing
RUST_LOG=debug bazbom scan . 2>&1 | jq .
```

## Useful Log Levels

- **ERROR**: Actual failures, file not found, API errors, validation failures
- **WARN**: Operations slower than expected, cache misses, fallback paths taken
- **INFO**: Phase start/end, major milestones (already in use)
- **DEBUG**: Detailed decisions, parameter values, branch selections
- **TRACE**: Loop iterations, recursion details, low-level operations

## Key Code References

### Smart Defaults Detection (commands/scan.rs:43-50)
```rust
let defaults = SmartDefaults::detect();
// TODO: Add debug log showing what was detected
```

### Cache Check (scan_orchestrator.rs:110-120)
```rust
if !cache_disabled {
    if let Ok(cached_result) = self.try_use_cache() {
        if cached_result {
            // TODO: Log cache hit
        }
    }
}
```

### SBOM Generation (scan_orchestrator.rs:150)
```rust
self.generate_sbom()?;
// TODO: Log build system, file count, generation time
```

### Advisory DB Sync (analyzers/sca.rs:45-48)
```rust
if needs_sync {
    println!("[bazbom] syncing advisory database...");
    // TODO: Replace with tracing::info! and add progress logging
}
```

### Vulnerability Processing (analyzers/sca.rs:120-150)
```rust
for pkg in packages {
    // TODO: Add debug log for each component being processed
    // TODO: Add debug log for matches found
}
```

## Related Documentation

- **EXPLORATION_SUMMARY.md** - Detailed analysis of logging, CLI, and commands
- **FLOW_DIAGRAMS.md** - Visual diagrams of scan flows and entry points
- **Cargo.toml** - Line 31 shows `tracing = "0.1"`
- **Cargo.toml** - Line 33+ show dependencies available

## References

- [tracing crate docs](https://docs.rs/tracing/)
- [tracing-subscriber docs](https://docs.rs/tracing-subscriber/)
- [clap derive docs](https://docs.rs/clap/latest/clap/_derive/index.html)

---

**Next Steps**: Start with Phase 1 setup, then implement Phase 2 (critical paths) for maximum value with minimum changes.
