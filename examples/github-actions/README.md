# GitHub Actions Integration Examples

This directory contains example GitHub Actions workflows for integrating BazBOM into your CI/CD pipeline.

## Quick Start

Copy `bazbom-scan.yml` to `.github/workflows/` in your repository:

```bash
mkdir -p .github/workflows
cp examples/github-actions/bazbom-scan.yml .github/workflows/
```

## Workflow Modes

The example workflow includes three modes:

### 1. Pull Request Mode (Fast)
- Runs on every PR
- Includes: SBOM + SCA + Semgrep
- Skips heavy analysis (CodeQL)
- **Execution time:** ~2-5 minutes

```yaml
bazbom scan . \
  --cyclonedx \
  --with-semgrep \
  --out-dir ./bazbom-output
```

### 2. Main Branch Mode (Comprehensive)
- Runs on pushes to main
- Includes: SBOM + SCA + Semgrep + CodeQL + Autofix suggestions
- Full security-extended suite
- **Execution time:** ~10-20 minutes

```yaml
bazbom scan . \
  --cyclonedx \
  --with-semgrep \
  --with-codeql security-extended \
  --autofix dry-run \
  --out-dir ./bazbom-output
```

### 3. Scheduled Mode (Weekly Deep Scan)
- Runs weekly on Monday at 00:00 UTC
- Same as main branch mode
- Catches new vulnerabilities in dependencies

## Outputs

The workflow produces several outputs:

### 1. SARIF Upload to GitHub Code Scanning
```yaml
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: bazbom-output/findings/merged.sarif
```

View results in: **Security → Code scanning alerts**

### 2. Workflow Artifacts
```
bazbom-results-{run_id}/
├── sbom/
│   ├── spdx.json              # Primary SBOM (SPDX 2.3)
│   └── cyclonedx.json         # Optional CycloneDX SBOM
├── findings/
│   ├── sca.sarif              # SCA findings (OSV/NVD/GHSA)
│   ├── semgrep.sarif          # Semgrep findings
│   ├── codeql.sarif           # CodeQL findings (if enabled)
│   └── merged.sarif           # All findings merged
├── enrich/
│   └── depsdev.json           # deps.dev enrichment data
└── fixes/
    ├── openrewrite-recipes.json  # Generated fix recipes
    └── *.patch                    # Dry-run patches
```

Retention: **90 days** (configurable)

### 3. PR Comment
Automatically posts a summary table to the PR with finding counts per tool.

## Configuration Options

### Minimal (SBOM only)
```yaml
bazbom scan . --out-dir ./bazbom-output
```

### Add Semgrep
```yaml
bazbom scan . --with-semgrep --out-dir ./bazbom-output
```

### Add CodeQL (default suite)
```yaml
bazbom scan . --with-codeql default --out-dir ./bazbom-output
```

### Add CodeQL (security-extended suite)
```yaml
bazbom scan . --with-codeql security-extended --out-dir ./bazbom-output
```

### Enable CycloneDX
```yaml
bazbom scan . --cyclonedx --out-dir ./bazbom-output
```

### Enable Autofix (dry-run)
```yaml
bazbom scan . --autofix dry-run --out-dir ./bazbom-output
```

### Container SBOM
```yaml
bazbom scan . --containers syft --out-dir ./bazbom-output
```

### Target specific module
```yaml
bazbom scan . --target my-module --out-dir ./bazbom-output
```

### Skip GitHub upload (local testing)
```yaml
bazbom scan . --no-upload --out-dir ./bazbom-output
```

## Project Configuration (bazbom.toml)

You can also configure defaults in your repository using `bazbom.toml`:

```toml
[analysis]
cyclonedx = true
semgrep = { enabled = true, ruleset = "curated-jvm@sha256:..." }
codeql = { enabled = false, suite = "default" }

[enrich]
depsdev = true

[autofix]
mode = "dry-run"
recipe_allowlist = ["commons-io", "jackson", "log4j", "spring-core"]

[containers]
strategy = "auto"

[publish]
github_code_scanning = true
artifact = true
```

With this config, you can simplify the workflow command:

```yaml
bazbom scan . --out-dir ./bazbom-output
```

## Advanced Workflows

### Matrix Strategy (Multiple JDK Versions)

```yaml
strategy:
  matrix:
    java: [11, 17, 21]
steps:
  - uses: actions/setup-java@v4
    with:
      distribution: 'temurin'
      java-version: ${{ matrix.java }}
  - run: bazbom scan . --out-dir ./bazbom-output-java${{ matrix.java }}
```

### Conditional Analysis Based on Changed Files

```yaml
- name: Check changed files
  id: changes
  uses: dorny/paths-filter@v3
  with:
    filters: |
      backend:
        - 'src/**'
      dependencies:
        - 'pom.xml'
        - 'build.gradle'
        - 'build.gradle.kts'

- name: Run full scan on dependency changes
  if: steps.changes.outputs.dependencies == 'true'
  run: bazbom scan . --with-semgrep --with-codeql
```

### Fail on Critical Findings

```yaml
- name: Check for critical findings
  run: |
    # Count critical-level findings in merged SARIF
    CRITICAL_COUNT=$(jq '[.runs[].results[] | select(.level == "error")] | length' bazbom-output/findings/merged.sarif)
    echo "Found $CRITICAL_COUNT critical findings"
    if [ "$CRITICAL_COUNT" -gt "0" ]; then
      echo "::error::Critical security issues found"
      exit 1
    fi
```

## Permissions

The workflow requires these permissions:

```yaml
permissions:
  contents: read           # Read repository code
  security-events: write   # Upload SARIF to Code Scanning
  actions: read            # Read workflow artifacts (optional)
```

For PR comments:
```yaml
permissions:
  pull-requests: write     # Post PR comments
```

## Troubleshooting

### SARIF Upload Fails
- Ensure `merged.sarif` exists: `ls -la bazbom-output/findings/`
- Validate SARIF: `jq . bazbom-output/findings/merged.sarif`
- Check file size < 10 MB (GitHub limit)

### Semgrep Not Found
```yaml
- name: Install Semgrep
  run: pipx install semgrep
```

### CodeQL Not Found
```yaml
- name: Install CodeQL
  run: |
    CODEQL_VERSION=2.19.4
    curl -sL https://github.com/github/codeql-cli-binaries/releases/download/v${CODEQL_VERSION}/codeql-linux64.zip -o codeql.zip
    unzip -q codeql.zip -d $HOME
    echo "$HOME/codeql" >> $GITHUB_PATH
```

### Timeout on Large Repositories
- Use `--target` to limit scope
- Separate Semgrep and CodeQL into different jobs
- Increase timeout: `timeout-minutes: 30`

## Cost Optimization

### Free Tier (Public Repos)
- All features free for public repositories
- Unlimited minutes and storage

### Private Repos
- Semgrep: ~2-5 minutes per scan
- CodeQL: ~10-20 minutes per scan
- **Recommendation:** Run CodeQL only on `main` and scheduled jobs

```yaml
- name: Run CodeQL
  if: github.ref == 'refs/heads/main' || github.event_name == 'schedule'
  run: bazbom scan . --with-codeql security-extended
```

## References

- [BazBOM Documentation](../../docs/README.md)
- [Integration Plan](../../docs/copilot/BAZBOM_INTEGRATION_PLAN.md)
- [GitHub Code Scanning](https://docs.github.com/en/code-security/code-scanning)
- [SARIF 2.1.0 Spec](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
