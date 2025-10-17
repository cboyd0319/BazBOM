# Quick Start

Get BazBOM up and running in 5 minutes or less.

## Prerequisites

- [Bazelisk](https://github.com/bazelbuild/bazelisk) or Bazel 7.0.0+
- Java 11+ (for running examples)
- Python 3.9+ (for supply chain tools)

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/cboyd0319/BazBOM.git
   cd BazBOM
   ```

2. **Verify Bazel setup**:
   ```bash
   bazel version
   ```

## Basic Usage

### Generate SBOMs

Generate SPDX-compliant SBOMs for all targets:

```bash
bazel build //:sbom_all
```

This creates SPDX JSON files in `bazel-bin/` for all discovered dependencies.

### Run Software Composition Analysis (SCA)

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
