# BazBOM CLI Examples

This directory contains examples of using the new BazBOM CLI features.

## Quick Start

### 1. Scan a Maven Project

```bash
# Navigate to a Maven project
cd /path/to/maven/project

# Scan with BazBOM
bazel run //tools/supplychain:bazbom_cli -- scan .

# Output: dependencies.json with all Maven dependencies
```

### 2. Scan a Gradle Project

```bash
# Navigate to a Gradle project
cd /path/to/gradle/project

# Scan including test dependencies
bazel run //tools/supplychain:bazbom_cli -- scan . --include-test

# Output: dependencies.json with compile + test dependencies
```

### 3. Initialize Configuration

```bash
# Create default bazbom.yml configuration
bazel run //tools/supplychain:bazbom_cli -- init

# Edit bazbom.yml to customize settings
vim bazbom.yml

# Scan with your custom configuration
bazel run //tools/supplychain:bazbom_cli -- scan .
```

## CSV Export Examples

### Export SBOM to CSV

```bash
# Generate SBOM
bazel build //:workspace_sbom

# Export to CSV
bazel build //:sbom_csv

# Open in spreadsheet
open bazel-bin/workspace_sbom.csv
# or
libreoffice bazel-bin/workspace_sbom.csv
```

### Export Vulnerability Findings to CSV

```bash
# Run security scan
bazel build //:sca_findings_with_vex --config=requires-network

# Export to CSV
bazel build //:vulnerabilities_csv

# View in terminal
cat bazel-bin/vulnerabilities.csv | column -t -s,
```

### Export License Report to CSV

```bash
# Generate license report
bazel build //:license_report

# Export to CSV
bazel build //:licenses_csv

# Filter for copyleft licenses
cat bazel-bin/licenses.csv | grep "Yes" | column -t -s,
```

## Badge Generation Examples

### Generate Security Badge for README

```bash
# Run scan and generate badge
bazel build //:sca_findings_with_vex --config=requires-network
bazel build //:security_badge

# View badge JSON
cat bazel-bin/security_badge.json

# Example output:
# {
#   "schemaVersion": 1,
#   "label": "security",
#   "message": "no known vulnerabilities",
#   "color": "success",
#   "namedLogo": "github"
# }
```

### Use Badge in GitHub Workflow

```yaml
# .github/workflows/security.yml
name: Security Scan

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run BazBOM Security Scan
        run: |
          bazel build //:sca_findings_with_vex
          bazel build //:security_badge
      
      - name: Upload Badge
        uses: actions/upload-artifact@v4
        with:
          name: security-badge
          path: bazel-bin/security_badge.json
```

## Configuration File Examples

### Basic Configuration (bazbom.yml)

```yaml
# Automatically detect build system
build_system: auto

# Don't include test dependencies
include_test_deps: false

# Generate both SPDX and CycloneDX formats
output_formats:
  - spdx
  - cyclonedx

# Report MEDIUM severity and above
severity_threshold: MEDIUM
```

### Strict Security Policy Configuration

```yaml
build_system: auto
include_test_deps: true

# Enforce strict security standards
policy:
  block_critical: true
  fail_on_policy_violation: true
  max_critical: 0        # No critical vulnerabilities allowed
  max_high: 0            # No high severity vulnerabilities allowed

# Use multiple vulnerability sources
vulnerability_sources:
  osv:
    enabled: true
  nvd:
    enabled: true
    api_key: ${NVD_API_KEY}
```

### CI/CD Optimized Configuration

```yaml
build_system: auto
include_test_deps: false

# Fast scan with caching
severity_threshold: HIGH

policy:
  block_critical: true
  fail_on_policy_violation: true
  max_critical: 0
  max_high: 5

# Custom output paths
output:
  sbom_path: build/sbom.spdx.json
  findings_path: build/vulnerabilities.json
  sarif_path: build/security.sarif
```

## Advanced Usage

