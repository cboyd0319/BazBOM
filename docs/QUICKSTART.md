# Quick Start

Get BazBOM up and running in 5 minutes or less.

## Prerequisites

### For Homebrew Installation (Easiest)
- Homebrew package manager (macOS/Linux)
- Java 11+ (optional, for reachability analysis when enabled)

### For Building from Source
- Rust 1.70+ (stable)
- Java 11+ (optional, for reachability analysis when enabled)

### For Bazel Integration
- [Bazelisk](https://github.com/bazelbuild/bazelisk) or Bazel 7.0.0+
- Java 11+ (for running examples)

## Installation

### Option 1: Homebrew (Recommended)

The easiest way to install BazBOM is via Homebrew:

```bash
# Add the tap
brew tap cboyd0319/bazbom

# Install BazBOM
brew install bazbom

# Verify installation
bazbom --version
bazbom --help
```

See [Homebrew Installation Guide](HOMEBREW_INSTALLATION.md) for more details.

### Option 2: Pre-built Binaries

Download pre-built, signed binaries from [GitHub Releases](https://github.com/cboyd0319/BazBOM/releases):

```bash
# macOS (Intel)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-apple-darwin.tar.gz
tar -xzf bazbom-x86_64-apple-darwin.tar.gz
sudo mv bazbom /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-unknown-linux-gnu.tar.gz
tar -xzf bazbom-x86_64-unknown-linux-gnu.tar.gz
sudo mv bazbom /usr/local/bin/

# Verify installation
bazbom --version
```

### Option 3: Build from Source

The Rust CLI is the primary distribution method aligned with BazBOM's memory-safe, single-binary architecture.

**Build from Source:**

```bash
# Clone and build
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"

# Verify installation
bazbom --version
bazbom --help
```

**Using the Binary:**

```bash
# The compiled binary is at:
./target/release/bazbom

# Or if added to PATH:
bazbom
```

### Option 4: Bazel Integration

For Bazel-specific workflows or development:

```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
bazel version
```

## Basic Usage

### Rust CLI Commands

The Rust CLI provides a unified interface for all build systems (Maven, Gradle, Bazel):

#### 1. Generate SBOM

```bash
# Scan current directory (auto-detects build system)
bazbom scan .

# Scan specific project path
bazbom scan /path/to/project

# Generate SPDX format (default)
bazbom scan . --format spdx

# Generate CycloneDX format
bazbom scan . --format cyclonedx

# Specify output directory
bazbom scan . --out-dir ./reports
```

**Default Outputs:**
- `sbom.spdx.json` or `sbom.cyclonedx.json` - SBOM in requested format
- `sca_findings.json` - Vulnerability findings (machine-readable)
- `sca_findings.sarif` - Vulnerability findings (GitHub Security format)

#### 2. Enable Reachability Analysis

```bash
# Perform bytecode reachability analysis (requires Java 11+)
bazbom scan . --reachability

# Note: Requires BAZBOM_REACHABILITY_JAR environment variable
# when reachability engine is available
```

#### 3. Advisory Database Sync

```bash
# Download/update advisory databases for offline use
bazbom db sync

# Syncs from: OSV, NVD, GHSA, CISA KEV, EPSS
# Creates cache at: .bazbom/cache/

# Use in offline mode
BAZBOM_OFFLINE=1 bazbom db sync
```

#### 4. Policy Checks

```bash
# Run policy checks against findings
bazbom policy check

# Policy configuration from bazbom.yml (if present)
# Outputs: SARIF with policy verdicts
```

#### 5. Remediation

```bash
# Suggest fixes (recommend-only mode)
bazbom fix --suggest

# Apply fixes and open PRs (when implemented)
bazbom fix --apply
```

### Example Workflows

#### Complete Security Scan

```bash
# 1. Sync advisory database
bazbom db sync

# 2. Generate SBOM and find vulnerabilities
bazbom scan . --format spdx

# 3. Check policy compliance
bazbom policy check

# 4. Get remediation suggestions
bazbom fix --suggest
```

#### Offline Operation

```bash
# 1. Sync data when online
bazbom db sync

# 2. Later, scan offline
BAZBOM_OFFLINE=1 bazbom scan .
```

#### CI/CD Integration

```bash
# Typical CI pipeline step
bazbom scan . --format spdx --out-dir ./artifacts
bazbom policy check
```

### Bazel Integration

For Bazel-native workflows:

#### Generate SBOMs

Generate SPDX-compliant SBOMs for all targets:

```bash
bazel build //:sbom_all
```

This creates SPDX JSON files in `bazel-bin/` for all discovered dependencies.

#### Run Software Composition Analysis (SCA)

Scan for known vulnerabilities using the OSV database:

```bash
bazel run //:sca_from_sbom
```

This queries the OSV database and generates SARIF reports.

### View Generated Files

```bash
# List all generated SPDX files
find bazel-bin -name "*.spdx.json"

# View a specific SBOM
cat bazel-bin/path/to/package.spdx.json | jq .

# List SARIF vulnerability reports
find bazel-bin -name "*.sarif.json"
```

## Run the Example

Try the minimal Java example:

```bash
# Build the example
bazel build //examples/minimal_java:all

# Run the example application
bazel run //examples/minimal_java:app

# Generate SBOM for the example
bazel build //examples/minimal_java:app.sbom
```

## Next Steps

- Read the [Usage Guide](USAGE.md) for detailed commands
- Explore the [Architecture](ARCHITECTURE.md) to understand how it works
- Check out [Supply Chain Security](SUPPLY_CHAIN.md) for advanced features
- See [Troubleshooting](TROUBLESHOOTING.md) if you encounter issues

## CI Integration

To integrate with GitHub Actions, see the example workflows in `.github/workflows/`:

- `ci.yml` - Build and test
- `supplychain.yml` - SBOM generation and SCA scanning

## Getting Help

- Check [Troubleshooting](TROUBLESHOOTING.md) for common issues
- Review [Architecture](ARCHITECTURE.md) to understand the system
- Open an issue on GitHub for bugs or feature requests
