# Usage Guide

This guide covers day-to-day commands and workflows for BazBOM.

## Dependency Extraction

BazBOM uses **maven_install.json** as the source of truth for Maven dependencies, as recommended by the Bazel ecosystem. This lockfile provides:

- Complete transitive dependency graph
- Exact versions for all dependencies
- SHA256 checksums for verification
- Dependency relationship mapping

### Automatic Dependency Extraction

```bash
# Extract all dependencies (prefers maven_install.json if available)
bazel build //:extract_deps

# View extracted dependencies
cat bazel-bin/workspace_deps.json | jq
```

**Output includes:**
- Direct dependencies (declared in WORKSPACE)
- Transitive dependencies (from maven_install.json)
- SHA256 checksums for all artifacts
- Package URLs (PURLs) for each dependency

### Manual Extraction

```bash
# Extract from maven_install.json
python tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --maven-install-json maven_install.json \
  --output deps.json

# Extract from WORKSPACE only (fallback)
python tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --output deps.json \
  --prefer-lockfile=false
```

## SBOM Generation

### Generate SBOM for a Single Target

```bash
bazel build //path/to:target.sbom
```

### Generate SBOMs for All Targets

```bash
# Generate SPDX 2.3 SBOMs (default)
bazel build //:sbom_all

# Generate both SPDX and CycloneDX formats
bazel build //:sbom_all_formats
```

**The SBOM now includes:**
- All transitive dependencies (not just direct deps)
- SHA256 checksums for verification
- Proper dependency relationships (e.g., Guava → failureaccess)
- Package URLs (PURLs) for each artifact

### Generate CycloneDX SBOM

BazBOM supports CycloneDX 1.5 format in addition to SPDX 2.3:

```bash
# Generate CycloneDX SBOM for workspace
bazel build //:workspace_sbom_cyclonedx

# View the CycloneDX SBOM
cat bazel-bin/workspace_sbom.cdx.json | jq

# Generate custom CycloneDX SBOM
python tools/supplychain/write_sbom.py \
  --input workspace_deps.json \
  --output sbom.cdx.json \
  --format cyclonedx \
  --name my-application
```

**CycloneDX format includes:**
- Components with PURLs and hashes
- License information (SPDX IDs)
- External references (download URLs)
- Dependency relationships
- Metadata (tools, timestamp)

### SBOM Format Comparison

| Feature | SPDX 2.3 | CycloneDX 1.5 |
|---------|----------|---------------|
| Format | JSON | JSON |
| Spec Version | 2.3 | 1.5 |
| Relationships | Explicit | Dependency graph |
| License IDs | SPDX IDs | SPDX IDs |
| Tool Support | Wide | Wide |
| Use Case | Compliance, legal | Security, DevSecOps |

**When to use each:**
- **SPDX**: Legal compliance, license analysis, regulatory requirements
- **CycloneDX**: Security scanning, vulnerability management, DevSecOps workflows

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
  --output vulnerabilities.json \
  --batch
```

### Vulnerability Enrichment

BazBOM enriches vulnerability findings with multiple authoritative data sources to provide actionable prioritization:

#### Enrichment Data Sources

1. **CISA KEV (Known Exploited Vulnerabilities)** - CVEs actively exploited in the wild
2. **EPSS (Exploit Prediction Scoring)** - ML-based exploitation probability (0-100%)
3. **GitHub Security Advisories (GHSA)** - Ecosystem-specific remediation guidance
4. **VulnCheck KEV** (optional) - Advanced exploit intelligence with API key

#### Enable Enrichment (Default)

```bash
# Enrichment is enabled by default
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output bazel-bin/vulnerabilities_enriched.json
```

#### Enrichment Output

Enriched findings include:

- **Risk Score (0-100)**: Composite score combining CVSS, EPSS, KEV, and exploit data
- **Priority (P0-P4)**: Actionable priority levels
  - **P0-IMMEDIATE**: In CISA KEV (fix now)
  - **P1-CRITICAL**: Risk score ≥ 80
  - **P2-HIGH**: Risk score ≥ 60
  - **P3-MEDIUM**: Risk score ≥ 40
  - **P4-LOW**: Risk score < 40
- **KEV Status**: Whether CVE is being actively exploited
- **EPSS Score**: Exploitation probability percentage
- **Exploit Intelligence**: Weaponization status, attack vector
- **GHSA Remediation**: Patched versions, vulnerable ranges

#### Priority Summary

```bash
📊 Priority Summary:
  🚨 P0 - IMMEDIATE (KEV):     2  ← FIX NOW
  🔴 P1 - CRITICAL:            5  ← This week
  🟠 P2 - HIGH:                8  ← This sprint
  🟡 P3 - MEDIUM:              6  ← Next quarter
  🟢 P4 - LOW:                 2  ← Backlog
```

#### Advanced Enrichment Options

```bash
# With GitHub token for higher GHSA rate limits
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output bazel-bin/vulnerabilities_enriched.json \
  --github-token "${GITHUB_TOKEN}"

# With VulnCheck API key for exploit intelligence
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output bazel-bin/vulnerabilities_enriched.json \
  --vulncheck-api-key "${VULNCHECK_API_KEY}"

# Disable enrichment (legacy mode)
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output bazel-bin/vulnerabilities.json \
  --no-enrich

# Disable specific enrichment sources
bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output bazel-bin/vulnerabilities_enriched.json \
  --disable-vulncheck \
  --disable-ghsa
```

#### API Keys and Rate Limits

**Free tier limits (no API key required):**

- EPSS: Unlimited (public API)
- KEV: Unlimited (public dataset)
- GHSA: 60 req/hour (unauthenticated)

**With API keys (recommended):**

- GHSA: 5000 req/hour (with GitHub token)
- VulnCheck: 100 req/day (free tier)

**Set environment variables:**

```bash
export GITHUB_TOKEN="ghp_xxxxxxxxxxxxx"
export VULNCHECK_API_KEY="your-api-key"
```

#### Risk Scoring Algorithm

BazBOM calculates a composite risk score (0-100) based on:

```
Risk Score = (CVSS × 0.40) + (EPSS × 0.30) + (KEV × 0.20) + (Exploit × 0.10)
```

**Example:**

```json
{
  "cve": "CVE-2021-44228",
  "package": "log4j-core",
  "version": "2.14.1",
  "risk_score": 97.5,
  "priority": "P0-IMMEDIATE",
  "kev": {
    "in_kev": true,
    "vulnerability_name": "Log4Shell",
    "due_date": "2021-12-24"
  },
  "epss": {
    "epss_score": 0.97538,
    "exploitation_probability": "97.5%"
  },
  "exploit": {
    "weaponized": true
  }
}
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

## Policy Enforcement

BazBOM includes a comprehensive policy enforcement tool to ensure supply chain security standards in CI/CD pipelines.

### Basic Policy Check

```bash
# Run policy check with default thresholds (0 critical vulnerabilities)
bazel build //:policy_check_report

# View policy report
cat bazel-bin/policy_check.json | jq
```

### Custom Policy Thresholds

```bash
# Strict policy: no critical/high vulnerabilities
python tools/supplychain/policy_check.py \
  --findings bazel-bin/sca_findings_filtered.json \
  --max-critical 0 \
  --max-high 0

# Flexible policy: allow some non-critical vulnerabilities
python tools/supplychain/policy_check.py \
  --findings bazel-bin/sca_findings_filtered.json \
  --max-critical 0 \
  --max-high 5 \
  --max-medium 20
```

**Exit codes:**
- `0`: All policies passed
- `1`: Policy violations found (CI should fail)

### Comprehensive Policy Check

```bash
# Check all reports: vulnerabilities, licenses, conflicts, risks
python tools/supplychain/policy_check.py \
  --findings bazel-bin/sca_findings_filtered.json \
  --license-report bazel-bin/license_report.json \
  --conflicts bazel-bin/conflicts.json \
  --risk-report bazel-bin/supply_chain_risks.json \
  --max-critical 0 \
  --max-high 5 \
  --output policy_report.json
```

### License Policy Enforcement

```bash
# Block specific licenses (e.g., GPL family)
python tools/supplychain/policy_check.py \
  --findings bazel-bin/sca_findings_filtered.json \
  --license-report bazel-bin/license_report.json \
  --blocked-licenses GPL-2.0 GPL-3.0 AGPL-3.0 \
  --block-license-conflicts \
  --flag-copyleft
```

### VEX Statement Requirements

```bash
# Require VEX statements for accepted risks
python tools/supplychain/policy_check.py \
  --findings bazel-bin/sca_findings_filtered.json \
  --require-vex-for-accepted \
  --max-critical 0
```

### Supply Chain Risk Policies

```bash
# Block typosquatting and unmaintained dependencies
python tools/supplychain/policy_check.py \
  --risk-report bazel-bin/supply_chain_risks.json \
  --block-typosquatting \
  --unmaintained-threshold 0
```

### CI/CD Integration

Add to `.github/workflows/supplychain.yml`:

```yaml
- name: Enforce Security Policies
  run: |
    bazel build //:policy_check_report
    
    # Policy check will fail (exit 1) if violations found
    python tools/supplychain/policy_check.py \
      --findings bazel-bin/sca_findings_filtered.json \
      --license-report bazel-bin/license_report.json \
      --conflicts bazel-bin/conflicts.json \
      --risk-report bazel-bin/supply_chain_risks.json \
      --max-critical 0 \
      --max-high 5 \
      --block-license-conflicts \
      --block-typosquatting
```

### Policy Violation Reports

Policy violations are categorized by severity:

```json
{
  "total_violations": 2,
  "violations": [
    {
      "severity": "CRITICAL",
      "rule": "max_critical_vulnerabilities",
      "message": "Found 2 critical vulnerabilities (max allowed: 0)",
      "details": {
        "count": 2,
        "threshold": 0
      }
    },
    {
      "severity": "HIGH",
      "rule": "blocked_license",
      "message": "Package foo uses blocked license: GPL-3.0",
      "details": {
        "package": "foo",
        "version": "1.0.0",
        "license": "GPL-3.0"
      }
    }
  ]
}
```

### Policy Configuration Examples

**Strict Production Policy:**
```bash
--max-critical 0 \
--max-high 0 \
--max-medium 5 \
--blocked-licenses GPL-2.0 GPL-3.0 AGPL-3.0 \
--block-license-conflicts \
--require-vex-for-accepted \
--block-typosquatting
```

**Development/Staging Policy:**
```bash
--max-critical 0 \
--max-high 10 \
--max-medium 50 \
--flag-copyleft \
--block-typosquatting
```

**Audit/Reporting Policy:**
```bash
--max-critical 999 \
--max-high 999 \
--flag-copyleft \
--output full-audit-report.json
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

## Dependency Analysis

### Detect Version Conflicts

Identify dependencies with multiple versions in the same build:

```bash
bazel build //:conflict_report
cat bazel-bin/conflicts.json
```

The report includes:
- Package names with conflicts
- All conflicting versions
- Recommended resolution version
- Affected targets

### License Compliance Checking

Generate a comprehensive license compliance report:

```bash
bazel build //:license_report
cat bazel-bin/license_report.json
```

With additional flags:

```bash
# Flag copyleft licenses
bazel run //tools/supplychain:license_analyzer -- \
  --input bazel-bin/workspace_deps.json \
  --output license_report.json \
  --flag-copyleft

# Check for license conflicts
bazel run //tools/supplychain:license_analyzer -- \
  --input bazel-bin/workspace_deps.json \
  --output license_report.json \
  --check-conflicts \
  --flag-copyleft
```

### Generate PURL for Dependencies

Convert Maven coordinates to Package URLs (PURLs):

```bash
# Process dependencies file
bazel run //tools/supplychain:purl_generator -- \
  --input bazel-bin/workspace_deps.json \
  --output deps_with_purls.json

# Single coordinate conversion (for testing)
bazel run //tools/supplychain:purl_generator -- \
  --coordinates "com.google.guava:guava:31.1-jre"
```

### Aggregate Supply Chain Metrics

Generate comprehensive metrics dashboard:

```bash
bazel build //:metrics_report
cat bazel-bin/supply_chain_metrics.json
```

The metrics include:
- Vulnerability counts by severity
- Dependency statistics (total, direct, transitive, conflicts)
- License distribution
- Copyleft and unknown license counts

For text format output:

```bash
bazel run //tools/supplychain:metrics_aggregator -- \
  --sbom bazel-bin/workspace_sbom.spdx.json \
  --sca-findings bazel-bin/sca_findings.json \
  --license-report bazel-bin/license_report.json \
  --conflicts bazel-bin/conflicts.json \
  --output metrics.txt \
  --format text
```

## Performance Optimization

### Use Configuration Profiles

BazBOM includes pre-configured Bazel profiles:

```bash
# Standard supply chain analysis
bazel build --config=supplychain //:sbom_all

# Incremental mode (faster for PRs)
bazel build --config=supplychain-incremental //:sbom_all

# Full analysis with all features
bazel build --config=supplychain-full //:supply_chain_all

# Offline mode (no network access)
bazel build --config=supplychain-offline //:sbom_all
```

### Enable Remote Caching

For team environments, configure remote cache in `.bazelrc`:

```bash
build:remote-cache --remote_cache=https://cache.example.com
build:remote-cache --experimental_remote_cache_compression
```

Then use:

```bash
bazel build --config=remote-cache //...
```

## Supply Chain Risk Analysis

### Detect Typosquatting and Outdated Packages

Run supply chain risk analysis to detect:
- Typosquatting attempts (similar package names)
- Outdated dependencies
- Deprecated packages

```bash
bazel build //:supply_chain_risk_report
cat bazel-bin/supply_chain_risks.json
```

The report includes:
- Typosquatting findings (packages similar to popular ones)
- Outdated version information with latest available versions
- Unmaintained dependency detection
- Risk severity levels

Manual execution with options:

```bash
# Check for typosquatting only
bazel run //tools/supplychain:supply_chain_risk -- \
  --sbom bazel-bin/workspace_sbom.spdx.json \
  --output risks.json \
  --check-typosquatting

# Check for deprecated packages (requires network)
bazel run //tools/supplychain:supply_chain_risk -- \
  --sbom bazel-bin/workspace_sbom.spdx.json \
  --output risks.json \
  --check-deprecated

# Offline mode
bazel run //tools/supplychain:supply_chain_risk -- \
  --sbom bazel-bin/workspace_sbom.spdx.json \
  --output risks.json \
  --offline-mode
```

## VEX (Vulnerability Exploitability eXchange)

### Create VEX Statements

Create VEX statements to suppress false positives or document accepted risks:

```bash
# Create a VEX statement file
cat > vex/statements/CVE-2023-12345.json <<EOF
{
  "cve": "CVE-2023-12345",
  "package": "pkg:maven/com.example/vulnerable@1.0.0",
  "status": "not_affected",
  "justification": "Vulnerable code path not used in our application",
  "created": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "author": "Security Team"
}
EOF
```

Valid status values:
- `not_affected` - Vulnerability doesn't affect this package/version
- `false_positive` - Scanner incorrectly flagged this
- `mitigated` - Risk mitigated by other controls
- `accepted_risk` - Known and accepted by security team

### Apply VEX Statements

Apply VEX statements to filter vulnerability findings:

```bash
# Apply during build
bazel build //:sca_findings_with_vex
cat bazel-bin/sca_findings_filtered.json

# Manual application
bazel run //tools/supplychain:vex_processor -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=sca_findings_filtered.json \
  --suppressed-output=suppressed.json
```

### Validate VEX Statements

Validate VEX statement format before committing:

```bash
bazel run //tools/supplychain:vex_processor -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=/tmp/filtered.json \
  --validate-only
```

See [vex/statements/README.md](../vex/statements/README.md) for more details on VEX format and best practices.

## Incremental Analysis

### Git-Based Incremental Builds

For faster CI/CD on large repositories, analyze only changed targets:

```bash
# Detect changed targets since last commit
bazel run //tools/supplychain:incremental_analyzer -- \
  --workspace=. \
  --base-ref=HEAD~1 \
  --output-format=targets

# Use output in build commands
TARGETS=$(bazel run //tools/supplychain:incremental_analyzer -- \
  --base-ref=origin/main \
  --output-format=targets)
bazel build $TARGETS
```

Output formats:
- `targets` - Space-separated Bazel target list
- `json` - Detailed JSON with changed files and affected targets
- `bazel-query` - Format suitable for `bazel query` commands

Example in CI:

```yaml
- name: Incremental Analysis
  run: |
    CHANGED=$(bazel run //tools/supplychain:incremental_analyzer -- \
      --base-ref=${{ github.event.pull_request.base.sha }} \
      --output-format=targets)
    if [ -n "$CHANGED" ]; then
      bazel build $CHANGED
    else
      echo "No changes detected, running full analysis"
      bazel build //...
    fi
```

## Troubleshooting

For common issues and solutions, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).
