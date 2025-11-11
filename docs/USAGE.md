# Usage

Common tasks with BazBOM: generate SBOMs, scan for vulnerabilities, apply policies.

## Prerequisites

- Rust CLI: [Homebrew](getting-started/homebrew-installation.md) or [build from source](../README.md#option-2-build-from-source-rust-cli)
- Java 11+ (optional, only for reachability analysis)
- For Maven projects: Maven plugin in `pom.xml`
- For Gradle projects: Gradle plugin in `build.gradle`
- For Bazel projects: Native aspects (no plugin needed)

## Quick Reference

```bash
# Scan any JVM project (auto-detects build system)
bazbom scan .

# Sync advisory database (OSV/NVD/GHSA/KEV/EPSS)
bazbom db sync

# Check policies
bazbom policy check

# Generate fix suggestions
bazbom fix --suggest

# Install pre-commit hooks
bazbom install-hooks
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
- `--format spdx` (default) or `--format cyclonedx`
- `--out-dir <path>` - Custom output directory
- `--reachability` - Enable bytecode analysis (slower, more accurate)
- `--fast` - Skip reachability for speed (<10 seconds)

### For Maven Projects

**Step 1:** Add Maven plugin to `pom.xml`:

```xml
<plugin>
    <groupId>io.bazbom</groupId>
    <artifactId>bazbom-maven-plugin</artifactId>
    <version>6.0.0</version>
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
    id("io.bazbom.gradle-plugin") version "6.0.0"
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

**Gotcha:** Container scanning is beta. Only detects JVM dependencies. Use Syft for non-JVM layers.

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

| Flag | Type | Default | Purpose |
|------|------|---------|---------|
| `--format` | `spdx` \| `cyclonedx` | `spdx` | Output format |
| `--out-dir` | path | `.` | Output directory |
| `--reachability` | bool | false | Enable bytecode analysis |
| `--fast` | bool | false | Skip reachability (speed) |
| `--offline-mode` | bool | false | No network calls |
| `--db-path` | path | `~/.bazbom/db` | Advisory database path |
| `--bazel-targets` | targets | — | Explicit Bazel targets |
| `--bazel-targets-query` | query | — | Bazel query expression |
| `--bazel-affected-by-files` | files | — | Incremental scan (git diff) |

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
