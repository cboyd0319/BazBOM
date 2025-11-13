# Usage

Common tasks with BazBOM: generate SBOMs, scan for vulnerabilities, apply policies.

## Prerequisites

- Rust CLI: [Build from source](getting-started/homebrew-installation.md) (clone this repo, compile, add to PATH)
- Java 11+ (optional, only for reachability analysis)
- For Maven projects: Maven plugin in `pom.xml`
- For Gradle projects: Gradle plugin in `build.gradle`
- For Bazel projects: Native aspects (no plugin needed)

## Quick Reference

```bash
# Scan any JVM project (auto-detects build system)
bazbom scan .

# Quick scan with short flags (v6.5.0+)
bazbom scan -r -s -f spdx -o ./reports

# Use named profile (v6.5.0+)
bazbom scan -p strict

# JSON output for CI/CD (v6.5.0+)
bazbom scan --json | jq '.vulnerabilities[] | select(.severity == "CRITICAL")'

# Diff mode - compare with baseline (v6.5.0+)
bazbom scan --diff --baseline=baseline.json

# Explain vulnerability details (v6.5.0+)
bazbom explain CVE-2024-1234 --verbose

# Sync advisory database (OSV/NVD/GHSA/KEV/EPSS)
bazbom db sync

# Check policies
bazbom policy check

# Generate fix suggestions
bazbom fix --suggest

# Install pre-commit hooks
bazbom install-hooks

# Interactive TUI with enhanced search (v6.5.0+)
bazbom explore --sbom sbom.spdx.json
# Press 'r' to toggle regex/glob search modes
# Press 'i' to toggle case-sensitive search
# Click CVE links in supported terminals
```

## Generate SBOM Locally

### For Any Project

```bash
cd /path/to/your-jvm-project
bazbom scan .
```

**Output:**
- `sbom.spdx.json` (SPDX 2.3 format)
- `sca_findings.json` (vulnerabilities with metadata)
- `sca_findings.sarif` (GitHub Security format)

**Flags:**
- `--format spdx` (or `-f spdx`) - Output format (default) or `cyclonedx`
- `--out-dir <path>` (or `-o <path>`) - Custom output directory
- `--reachability` (or `-r`) - Enable bytecode analysis (slower, more accurate)
- `--fast` - Skip reachability for speed (<10 seconds)
- `--profile <name>` (or `-p <name>`) - Use named profile from bazbom.toml (NEW in v6.5.0)
- `--json` - Machine-readable JSON output for CI/CD automation (NEW in v6.5.0)
- `--diff` (or `-d`) - Show vulnerability diff vs baseline (NEW in v6.5.0)
- `--with-semgrep` (or `-s`) - Run Semgrep analysis
- `--ml-risk` (or `-m`) - ML-enhanced risk scoring

### For Maven Projects

**Step 1:** Add Maven plugin to `pom.xml`:

```xml
<plugin>
    <groupId>io.bazbom</groupId>
    <artifactId>bazbom-maven-plugin</artifactId>
    <version>6.5.0</version>
</plugin>
```

**Step 2:** Generate dependency graph:

```bash
mvn bazbom:graph
```

**Step 3:** Run scan:

```bash
bazbom scan .
```

**Why:** Maven plugin provides authoritative dependency tree with scopes, versions, and licenses.

### For Gradle Projects

**Step 1:** Add Gradle plugin to `build.gradle.kts`:

```kotlin
plugins {
    id("io.bazbom.gradle-plugin") version "6.5.0"
}
```

**Step 2:** Generate dependency graph:

```bash
./gradlew bazbomGraph
```

**Step 3:** Run scan:

```bash
bazbom scan .
```

**Gotcha:** Gradle has multiple configurations. Plugin extracts all by default. Use `--configuration runtimeClasspath` to filter.

### For Bazel Projects

```bash
# Scan all JVM targets
bazbom scan . --bazel-targets //...

# Scan specific target
bazbom scan . --bazel-targets //src/java:app

# Scan affected targets (incremental)
bazbom scan . --bazel-affected-by-files $(git diff --name-only HEAD~1)

# Query-based scanning
bazbom scan . --bazel-targets-query 'kind(java_binary, //services/...)'
```

**Default:** Uses `maven_install.json` from `rules_jvm_external`.

## Generate SBOM for Container Image

**Prerequisites:** Docker image with JVM artifacts

```bash
# Scan OCI image
bazbom scan-container my-app:latest

# Save SBOM to file
bazbom scan-container my-app:latest --out sbom.json
```

**Output:** SBOM with extracted JAR metadata (groupId:artifactId:version).

**Coverage:** Container scanning shares the same polyglot detectors used for source scans (JVM, npm, pip, Go modules, Cargo, Bundler, Composer) and reuses reachability analysis to flag exploitable packages. When an ecosystem is missing, BazBOM gracefully emits a warning so you can fall back to Syft/Trivy if needed.

## Vulnerability Scanning

### Offline Mode (Air-Gapped)

**Step 1:** Sync database on internet-connected machine:

```bash
bazbom db sync --out /path/to/osv-db
```

**Step 2:** Copy database to air-gapped machine

**Step 3:** Scan with local database:

```bash
bazbom scan . --offline-mode --db-path /path/to/osv-db
```

**Why:** Zero telemetry. No network calls during scan.

### Online Mode

```bash
# Scan with latest CVEs
bazbom scan .
```

**Default:** Checks local cache first. Downloads updates if >24 hours old.

## Flags Reference

### `bazbom scan`

| Flag | Short | Type | Default | Purpose |
|------|-------|------|---------|---------|
| `--format` | `-f` | `spdx` \| `cyclonedx` | `spdx` | Output format |
| `--out-dir` | `-o` | path | `.` | Output directory |
| `--reachability` | `-r` | bool | false | Enable bytecode analysis |
| `--fast` | — | bool | false | Skip reachability (speed) |
| `--profile` | `-p` | string | — | Named profile from bazbom.toml (v6.5.0+) |
| `--json` | — | bool | false | Machine-readable JSON output (v6.5.0+) |
| `--diff` | `-d` | bool | false | Show vulnerability diff vs baseline (v6.5.0+) |
| `--baseline` | — | path | — | Baseline findings file for diff mode |
| `--with-semgrep` | `-s` | bool | false | Run Semgrep analysis |
| `--with-codeql` | `-c` | suite | — | Run CodeQL analysis |
| `--ml-risk` | `-m` | bool | false | ML-enhanced risk scoring |
| `--incremental` | `-i` | bool | false | Incremental analysis mode |
| `--base` | `-b` | ref | `main` | Git base reference for incremental |
| `--offline-mode` | — | bool | false | No network calls |
| `--db-path` | — | path | `~/.bazbom/db` | Advisory database path |
| `--bazel-targets` | — | targets | — | Explicit Bazel targets |
| `--bazel-targets-query` | — | query | — | Bazel query expression |
| `--bazel-affected-by-files` | — | files | — | Incremental scan (git diff) |

### `bazbom db sync`

| Flag | Type | Default | Purpose |
|------|------|---------|---------|
| `--out` | path | `~/.bazbom/db` | Database output path |
| `--sources` | list | `osv,nvd,ghsa,kev,epss` | Advisory sources |

### `bazbom policy check`

| Flag | Type | Default | Purpose |
|------|------|---------|---------|
| `--policy-file` | path | `bazbom.yml` | Policy definition |
| `--fail-on` | severity | `critical` | Exit 1 on violations |

## Policy Enforcement

**Create policy from template:**

```bash
bazbom policy init --template pci-dss
```

**Output:** `bazbom.yml` with PCI-DSS rules

**Check compliance:**

```bash
bazbom policy check
```

**Exit codes:**
- `0` - Pass
- `1` - Policy violations found

**Gotcha:** Use in CI to gate deployments:

```yaml
- name: Policy Check
  run: bazbom policy check --fail-on critical
```

## Outputs

### SBOM Files

**SPDX 2.3 (primary):**
```json
{
  "spdxVersion": "SPDX-2.3",
  "packages": [
    {
      "name": "log4j-core",
      "versionInfo": "2.17.0",
      "licenseConcluded": "Apache-2.0"
    }
  ]
}
```

**CycloneDX 1.5 (optional):**
```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "components": [...]
}
```

### SARIF 2.1.0

```json
{
  "version": "2.1.0",
  "runs": [{
    "results": [
      {
        "ruleId": "CVE-2024-1234",
        "level": "error",
        "message": { "text": "Log4j Remote Code Execution" }
      }
    ]
  }]
}
```

**Why:** Upload to GitHub Security tab for code scanning alerts.

### VEX Statements

**Generate VEX for false positives:**

```bash
bazbom vex create \
  --cve CVE-2024-1234 \
  --status not_affected \
  --justification code_not_reachable \
  --out vex/CVE-2024-1234.json
```

**Apply VEX statements:**

```bash
bazbom scan . --vex-dir vex/
```

**Result:** CVEs in VEX statements are filtered from findings.

## Examples

**Full list:** [examples/](examples/)

**Key workflows:**
- [Maven Spring Boot](examples/maven_spring_boot.md)
- [Gradle Kotlin](examples/gradle_kotlin.md)
- [Bazel Monorepo](examples/bazel-monorepo-workflows.md)
- [Shaded JAR](examples/shaded_jar.md)

## Next Steps

- [Advanced Bazel features](user-guide/advanced-bazel-features.md)
- [Policy integration](user-guide/policy-integration.md)
- [Troubleshooting](user-guide/troubleshooting.md)
- [CI/CD integration](CI.md)
