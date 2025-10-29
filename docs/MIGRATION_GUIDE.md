# Migration Guide: Python to Rust CLI

This guide helps users transition from the Python-based BazBOM tooling to the new Rust CLI.

## Overview

BazBOM is migrating from Python-based tooling to a memory-safe Rust CLI. The Rust CLI provides:

- **Single binary distribution** - No Python dependencies required
- **Memory safety** - No segfaults or memory leaks
- **Better performance** - Faster scanning and analysis
- **Signed releases** - Cryptographically signed binaries with SLSA provenance
- **Easy installation** - Homebrew support and pre-built binaries

## Key Differences

### Installation

**Python-based (Legacy):**

```bash
curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
```

**Rust CLI (Recommended):**

```bash
# Via Homebrew
brew tap cboyd0319/bazbom
brew install bazbom

# Or download pre-built binary
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-apple-darwin.tar.gz
tar -xzf bazbom-x86_64-apple-darwin.tar.gz
sudo mv bazbom /usr/local/bin/
```

### Command Interface

The Rust CLI maintains compatibility with most common commands:

| Task | Python-based | Rust CLI |
|------|-------------|----------|
| Scan project | `bazbom scan .` | `bazbom scan .` |
| SPDX output | `bazbom scan --format spdx` | `bazbom scan --format spdx` |
| SARIF output | `bazbom scan --format sarif` | `bazbom scan --format sarif` |
| Policy check | `bazbom policy check` | `bazbom policy check` |
| Advisory sync | `bazbom db sync` | `bazbom db sync` |

### Output Formats

Both versions support the same output formats:

- SPDX 2.3 (default)
- CycloneDX 1.5
- SARIF 2.1.0
- CSV

Output file structure and schemas remain compatible.

### Configuration

Configuration files use the same format and location:

- `bazbom.yml` or `.bazbom.yml` in project root
- Policy configuration syntax unchanged
- Environment variables remain the same

## Migration Steps

### Step 1: Backup Configuration

Save your current configuration before migrating:

```bash
# Backup policy configuration
cp bazbom.yml bazbom.yml.backup

# Backup any custom scripts that use bazbom
```

### Step 2: Install Rust CLI

Install the Rust CLI using your preferred method:

```bash
# Recommended: Homebrew
brew tap cboyd0319/bazbom
brew install bazbom

# Verify installation
bazbom --version
```

### Step 3: Test Compatibility

Test the Rust CLI with your existing configuration:

```bash
# Run a test scan
bazbom scan .

# Verify output
ls -la *.spdx.json *.sarif.json

# Test policy check
bazbom policy check
```

### Step 4: Update CI/CD

Update your CI/CD pipelines to use the Rust CLI:

**Before (Python-based):**

```yaml
steps:
  - name: Install BazBOM
    run: |
      curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
      export PATH="$HOME/.bazbom:$PATH"
  
  - name: Run scan
    run: bazbom scan .
```

**After (Rust CLI):**

```yaml
steps:
  - name: Install BazBOM
    run: |
      brew tap cboyd0319/bazbom
      brew install bazbom
  
  - name: Run scan
    run: bazbom scan .
```

Or use pre-built binaries:

```yaml
steps:
  - name: Install BazBOM
    run: |
      curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-unknown-linux-gnu.tar.gz
      tar -xzf bazbom-x86_64-unknown-linux-gnu.tar.gz
      sudo mv bazbom /usr/local/bin/
  
  - name: Run scan
    run: bazbom scan .
```

### Step 5: Remove Python Version

Once you have verified the Rust CLI works for your use case:

```bash
# Remove Python-based installation
rm -rf ~/.bazbom

# Update PATH in shell configuration
# Remove: export PATH="$HOME/.bazbom:$PATH"
```

## Feature Parity Status

The Rust CLI has achieved feature parity with the Python version for core functionality:

### Available in Rust CLI ✅

- Build system detection (Maven, Gradle, Bazel)
- SPDX 2.3 output
- CycloneDX 1.5 output
- SARIF 2.1.0 output
- Policy checking (YAML configuration)
- Advisory database sync
- Offline mode
- Basic reachability analysis (via OPAL)

### Coming Soon 🔄

- Advanced reachability analysis
- VEX auto-generation
- Remediation automation (fix suggestions and apply)
- EPSS enrichment
- KEV enrichment
- CSV export enhancements

### Not Yet Implemented ⏸️

- Watch mode (continuous scanning)
- Container SBOM generation
- Interactive fix mode
- Security badge generation

## Compatibility Notes

### Environment Variables

Both versions use the same environment variables:

- `BAZBOM_CACHE_DIR` - Cache directory location
- `GITHUB_TOKEN` - GitHub token for GHSA queries
- `NO_COLOR` - Disable colored output

### Exit Codes

Exit codes remain consistent:

- `0` - Success
- `1` - General error
- `2` - Policy violation
- `3` - Build system not detected

### File Paths

The Rust CLI uses the same default paths:

- Cache: `~/.bazbom/cache`
- Configuration: `bazbom.yml` or `.bazbom.yml`
- Output: `output.spdx.json`, `output.sarif.json`

## Troubleshooting

### Binary Not Found

If `bazbom` is not found after installation:

```bash
# Check if installed
which bazbom

# Add to PATH
export PATH="/usr/local/bin:$PATH"

# Or for Homebrew
export PATH="$(brew --prefix)/bin:$PATH"
```

### Incompatible Output

If the Rust CLI output differs from Python version:

1. Verify you are using the same version of SPDX/SARIF schemas
2. Check for any custom configuration that might affect output
3. Report differences as issues on GitHub

### Performance Differences

The Rust CLI should be faster than the Python version. If you experience slowness:

1. Ensure you are using a release build, not debug
2. Check for network issues during advisory sync
3. Monitor disk I/O for cache operations

### Missing Features

If you rely on a feature not yet available in the Rust CLI:

1. Continue using the Python version for that specific workflow
2. Check the [Roadmap](ROADMAP_IMPLEMENTATION.md) for planned implementation
3. Open a feature request on GitHub to prioritize your use case

## Getting Help

If you encounter issues during migration:

1. Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
3. Open a new issue with:
   - Python version you were using
   - Rust CLI version you are migrating to
   - Specific error messages or unexpected behavior
   - Minimal reproduction steps

## Rollback Instructions

If you need to rollback to the Python version:

```bash
# Uninstall Rust CLI
brew uninstall bazbom
brew untap cboyd0319/bazbom

# Or remove binary
sudo rm /usr/local/bin/bazbom

# Reinstall Python version
curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
export PATH="$HOME/.bazbom:$PATH"

# Restore backed-up configuration
cp bazbom.yml.backup bazbom.yml
```

## Feedback

We value your feedback during this migration:

- What worked well?
- What was confusing?
- What features are you missing?
- What could we improve?

Please share your migration experience by opening a GitHub issue with the `migration` label.

## References

- [Rust CLI Installation Guide](HOMEBREW_INSTALLATION.md)
- [Release Process](RELEASE_PROCESS.md)
- [Usage Guide](USAGE.md)
- [Troubleshooting](TROUBLESHOOTING.md)
- [Master Plan](copilot/MASTER_PLAN.md)
