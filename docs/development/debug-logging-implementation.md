# Debug Logging and Limit Parameter Implementation

## Summary

This document describes the implementation of robust debug logging and a `--limit` parameter for BazBOM commands to help with debugging and testing large monorepos.

## Features Added

### 1. Structured Logging with `tracing-subscriber`

**Changes:**
- Added `tracing-subscriber` dependency to `crates/bazbom/Cargo.toml`
- Initialized logging in `main.rs` with environment variable support
- Added comprehensive debug logging throughout scan operations

**Usage:**
```bash
# Normal output (info level)
bazbom full

# Debug output for troubleshooting
RUST_LOG=debug bazbom full

# Trace level for maximum verbosity
RUST_LOG=trace bazbom full

# Module-specific logging
RUST_LOG=bazbom=debug bazbom full
```

**Benefits:**
- Structured, filterable logging output
- Easy to diagnose failures in large scans
- Industry-standard logging framework
- No performance impact when disabled

### 2. Limit Parameter for Commands

**Changes:**
- Added `--limit` parameter to `Scan` and `Full` commands in `cli.rs`
- Updated `handle_scan` function to accept and process the limit parameter
- Limit is stored in environment variable `BAZBOM_SCAN_LIMIT` for downstream components

**Usage:**
```bash
# Limit full scan to 5 packages/targets (useful for testing)
bazbom full --limit 5

# Limit regular scan to 10 packages/targets
bazbom scan --limit 10

# Works with other flags
bazbom full --limit 3 -o ./output
```

**Benefits:**
- Quick testing in huge monorepos
- Faster iteration during development
- Helps identify issues without full scan overhead

### 3. Debug Logging Coverage

The following operations now have detailed debug logging:

1. **Smart Defaults Detection** (commands/scan.rs:55-58)
   - CI environment detection
   - PR mode detection
   - Reachability analysis enablement

2. **Auto-Feature Enablement** (commands/scan.rs:68-93)
   - JSON output auto-enablement for CI
   - Reachability analysis auto-enablement
   - Incremental mode auto-enablement
   - Diff mode auto-enablement

3. **Profile Loading** (commands/scan.rs:101-110)
   - Profile name and loading status
   - Errors and warnings

4. **Diff Mode** (commands/scan.rs:113-123)
   - Baseline file path
   - Missing baseline warnings

5. **Orchestration Mode** (commands/scan.rs:139-167)
   - Orchestrator options
   - Workspace and output directory paths

6. **Legacy Scan Mode** (commands/scan.rs:171-193)
   - Scan start and completion status
   - Error indicators

7. **Auto-Remediation** (commands/scan.rs:196-221)
   - Configuration details
   - Success/failure status
   - Minimum severity and reachable-only settings

## Implementation Details

### File Changes

1. **crates/bazbom/Cargo.toml**
   - Added: `tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt"] }`

2. **crates/bazbom/src/main.rs**
   - Initialized tracing subscriber at application startup
   - Added limit parameter to all command match arms
   - Added user-friendly message when limit is specified

3. **crates/bazbom/src/cli.rs**
   - Added `limit: Option<usize>` to `Scan` command
   - Added `limit: Option<usize>` to `Full` command
   - Updated help text with limit examples

4. **crates/bazbom/src/commands/scan.rs**
   - Added limit parameter to `handle_scan` function signature
   - Set `BAZBOM_SCAN_LIMIT` environment variable when limit is specified
   - Added comprehensive debug logging with `tracing::{debug, info, warn}` macros

### Logging Levels

- **ERROR**: Critical failures (not added yet, but available for use)
- **WARN**: Non-fatal issues (e.g., profile loading failures)
- **INFO**: Important operational messages (e.g., "Using orchestrated scan mode")
- **DEBUG**: Detailed diagnostic information (e.g., smart defaults detected, scan options)
- **TRACE**: Very detailed information (not used yet, but available)

## Testing Recommendations

### Test Debug Logging
```bash
# Run with debug logging to see all messages
cd /path/to/huge/monorepo
RUST_LOG=debug bazbom full --limit 5

# Should show output like:
# DEBUG Starting scan with path: .
# DEBUG Scan options - reachability: true, fast: false, format: spdx, incremental: false
# INFO Scan limit enabled: will process maximum 5 packages/targets
# DEBUG Detecting smart defaults for environment
# ...
```

### Test Limit Parameter
```bash
# Test that limit is recognized
bazbom full --limit 3

# Should show:
#   ℹ️  Limiting scan to 3 packages/targets
```

### Verify Environment Variable
```bash
# Check that limit is set in environment
RUST_LOG=debug bazbom full --limit 5 2>&1 | grep BAZBOM_SCAN_LIMIT
```

## Next Steps

To fully leverage the limit parameter, downstream components should check for the `BAZBOM_SCAN_LIMIT` environment variable and respect it when:
- Discovering packages
- Scanning Bazel targets
- Processing dependencies
- Running reachability analysis

Example implementation in a scanner:
```rust
if let Ok(limit_str) = std::env::var("BAZBOM_SCAN_LIMIT") {
    if let Ok(limit) = limit_str.parse::<usize>() {
        packages.truncate(limit);
        tracing::info!("Limited scan to {} packages", limit);
    }
}
```

## Troubleshooting

### No debug output appearing
- Ensure you set `RUST_LOG=debug` environment variable
- Check that you rebuilt after making changes: `cargo build --release`

### Limit not working
- Check that downstream components respect `BAZBOM_SCAN_LIMIT`
- Verify the environment variable is set: `env | grep BAZBOM`

### Too much output
- Use more specific logging: `RUST_LOG=bazbom::commands::scan=debug`
- Or use info level: `RUST_LOG=info`

## References

- [tracing documentation](https://docs.rs/tracing)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber)
- BazBOM exploration docs: `EXPLORATION_SUMMARY.md`
