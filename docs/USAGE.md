# Usage Guide

This guide covers day-to-day commands and workflows for BazBOM.

## SBOM Generation

### Generate SBOM for a Single Target

```bash
bazel build //path/to:target.sbom
```

### Generate SBOMs for All Targets

```bash
bazel build //:sbom_all
```

### Generate SBOM with Custom Options

```bash
bazel build //path/to:target.sbom \
  --@bazbom//config:sbom_format=spdx-2.3 \
  --@bazbom//config:include_transitive=true
```

## Vulnerability Scanning

### Scan Single SBOM

```bash
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/path/to/package.spdx.json \
  --output bazel-bin/path/to/vulnerabilities.sarif.json
```

### Scan All Generated SBOMs

```bash
bazel run //:sca_from_sbom
```

This automatically discovers all `.spdx.json` files and queries OSV.

### Custom OSV Query

```bash
python tools/supplychain/osv_query.py \
  --sbom path/to/sbom.json \
  --output vulnerabilities.sarif.json \
  --severity high,critical
```

## Validation

### Validate SPDX Documents

```bash
# Using the built-in validator
bazel run //tools/supplychain:validate_spdx -- \
  --input bazel-bin/path/to/package.spdx.json

# Using external SPDX validator
spdx-sbom-validator bazel-bin/path/to/package.spdx.json
```

### Validate SARIF Reports

```bash
# Using the built-in validator
bazel run //tools/supplychain:validate_sarif -- \
  --input bazel-bin/path/to/vulnerabilities.sarif.json
```

## Working with Aspects

### List Dependencies

```bash
# Show all dependencies for a target
bazel query 'deps(//path/to:target)' --output package

# Show only direct dependencies
bazel query 'deps(//path/to:target, 1)' --output package
```

### Inspect Build Graph

```bash
# Generate dependency graph
bazel query 'deps(//path/to:target)' --output graph > graph.dot
dot -Tpng graph.dot -o graph.png
```

## Development Workflows

### Build Everything

```bash
bazel build //...
```

### Run All Tests

```bash
bazel test //...
```

### Clean Build Outputs

```bash
bazel clean
```

### Clean Everything (including downloaded dependencies)

```bash
bazel clean --expunge
```

## CI/CD Usage

### GitHub Actions

The repository includes pre-configured workflows:

```bash
# CI workflow runs on every push
# Builds, tests, and lints the code

# Supply chain workflow runs on main branch
# Generates SBOMs, runs SCA, uploads SARIF to GitHub Code Scanning
```

### Local CI Simulation

Run the same checks as CI locally:

```bash
# Lint
bazel run //tools/dev:lint

# Build
bazel build //...

# Test
bazel test //...

# Generate SBOMs
bazel build //:sbom_all

# Run SCA
bazel run //:sca_from_sbom
```

## Advanced Usage

### Custom SBOM Metadata

Edit `tools/supplychain/write_sbom.py` to customize:

- Document namespace
- Creator information
- Package supplier/originator
- License declarations

### Custom Vulnerability Policies

Create a custom policy file:

```json
{
  "ignore": ["CVE-2021-12345"],
  "fail_on": ["critical", "high"],
  "exceptions": {
    "package-name": ["CVE-2021-67890"]
  }
}
```

Apply it:

```bash
bazel run //:sca_from_sbom -- --policy policy.json
```

### Integration with Other Tools

Export SBOMs for use with other tools:

```bash
# Export for dependency-track
cp bazel-bin/path/to/package.spdx.json /path/to/dependency-track/import/

# Export for Grype
grype sbom:bazel-bin/path/to/package.spdx.json

# Export for Syft
syft convert bazel-bin/path/to/package.spdx.json -o cyclonedx-json
```

## Troubleshooting

For common issues and solutions, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
