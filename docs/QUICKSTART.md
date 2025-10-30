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

Reachability analysis uses bytecode analysis to determine which vulnerable methods in your dependencies are actually called by your code. This dramatically reduces false positives by focusing only on vulnerabilities in code paths that are reachable from your application.

**Basic Reachability Scan:**

```bash
# Build the reachability analyzer first
cd tools/bazbom-reachability
mvn clean package
cd ../..

# Set the environment variable to point to the JAR
export BAZBOM_REACHABILITY_JAR=tools/bazbom-reachability/target/bazbom-reachability.jar

# Perform reachability-aware scan
bazbom scan . --reachability
```

**Output Enhancements with Reachability:**

When reachability is enabled:
- SARIF findings are tagged with `[REACHABLE]` or `[NOT REACHABLE]`
- Policy checks can filter based on reachability status
- Findings are prioritized based on actual risk exposure
- Reachability results are cached for faster subsequent scans

**Maven Project Example:**

```bash
# Maven project with reachability
cd /path/to/maven/project
export BAZBOM_REACHABILITY_JAR=/path/to/bazbom-reachability.jar
bazbom scan . --reachability --format spdx --out-dir ./reports

# Output includes:
# - reports/sbom.spdx.json (SBOM)
# - reports/sca_findings.json (vulnerabilities with reachability info)
# - reports/sca_findings.sarif (SARIF with [REACHABLE]/[NOT REACHABLE] tags)
# - reports/reachability.json (detailed reachability analysis)
```

**Gradle Project Example:**

```bash
# Gradle project with reachability
cd /path/to/gradle/project
export BAZBOM_REACHABILITY_JAR=/path/to/bazbom-reachability.jar
bazbom scan . --reachability --format spdx --out-dir ./build/bazbom

# Reachability analysis includes all configurations
```

**Bazel Project Example:**

```bash
# Bazel project with reachability
cd /path/to/bazel/project
export BAZBOM_REACHABILITY_JAR=/path/to/bazbom-reachability.jar

# Scan all targets
bazbom scan . --reachability

# Scan specific targets
bazbom scan . --reachability --bazel-targets //src/main/java/...:all

# Incremental scan (only affected targets)
bazbom scan . --reachability --bazel-affected-by-files src/main/java/com/example/Foo.java
```

**Understanding Reachability Output:**

The `reachability.json` file contains:
```json
{
  "reachableClasses": ["com.example.MyApp", "org.apache.commons.Lang"],
  "reachableMethods": [
    {"className": "org.apache.commons.Lang", "methodName": "isEmpty", "descriptor": "(Ljava/lang/String;)Z"}
  ],
  "reachablePackages": ["com.example", "org.apache.commons"],
  "entryPoints": ["com.example.MyApp.main"]
}
```

**Reachability Cache:**

Results are cached in `.bazbom/reachability-cache/` for faster subsequent runs:
```bash
# First run (slow - analyzes bytecode)
bazbom scan . --reachability  # ~30s

# Subsequent runs with same classpath (fast - uses cache)
bazbom scan . --reachability  # ~2s
```

#### 3. Shaded JAR Detection

BazBOM automatically detects and analyzes shaded (relocated) dependencies in fat JARs, providing accurate attribution even when packages have been relocated.

**Maven Shade Plugin Detection:**

BazBOM automatically detects Maven Shade plugin configuration in `pom.xml`:

```xml
<!-- pom.xml -->
<plugin>
    <artifactId>maven-shade-plugin</artifactId>
    <configuration>
        <relocations>
            <relocation>
                <pattern>com.google.guava</pattern>
                <shadedPattern>myapp.shaded.guava</shadedPattern>
            </relocation>
        </relocations>
    </configuration>
</plugin>
```

**Gradle Shadow Plugin Detection:**

Similarly detects Gradle Shadow plugin configuration:

```kotlin
// build.gradle.kts
plugins {
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

shadowJar {
    relocate("org.apache.commons", "myapp.shaded.commons")
}
```

**Shading Analysis:**

When scanning projects with shaded JARs, BazBOM:
1. Detects relocation mappings from build configuration
2. Extracts nested JARs from fat JARs
3. Fingerprints classes using bytecode hashing
4. Maps shaded classes back to original artifacts
5. Reports accurate vulnerability attribution

**Example Output:**

```bash
# Scan a project with shaded dependencies
bazbom scan . --format spdx

# Output includes relocation information:
# - Original package: com.google.guava.collect.ImmutableList
# - Shaded location: myapp.shaded.guava.collect.ImmutableList
# - Original artifact: com.google.guava:guava:31.1-jre
# - Confidence: 1.0 (exact bytecode match)
```

#### 4. Advisory Database Sync

```bash
# Download/update advisory databases for offline use
bazbom db sync

# Syncs from: OSV, NVD, GHSA, CISA KEV, EPSS
# Creates cache at: .bazbom/cache/

# Use in offline mode
BAZBOM_OFFLINE=1 bazbom db sync
```

#### 5. Policy Checks

```bash
# Run policy checks against findings
bazbom policy check

# Policy configuration from bazbom.yml (if present)
# Outputs: SARIF with policy verdicts
```

#### 6. Remediation

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
