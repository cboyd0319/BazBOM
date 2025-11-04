# BazBOM Examples

This directory contains example configurations, CI workflows, and sample projects for BazBOM.

## Build System Examples

BazBOM supports all major JVM build systems. Example projects demonstrate build system detection:

### Maven (`maven_spring_boot/`)
Standard Maven project with Spring Boot.
- **Detection:** `pom.xml`
- **Features:** Full dependency resolution, plugin integration

### Gradle (`gradle_kotlin/`)
Gradle project with Kotlin DSL.
- **Detection:** `build.gradle.kts`, `settings.gradle.kts`
- **Features:** Multi-configuration scanning, Shadow plugin support

### Bazel (multiple examples)
Bazel workspace with JVM rules.
- **Detection:** `MODULE.bazel`, `WORKSPACE`, `BUILD.bazel`
- **Features:** Target-specific scanning, incremental analysis

### Ant (`ant_project/`) ✨ NEW
Apache Ant project with traditional XML build configuration.
- **Detection:** `build.xml`
- **Features:** JAR dependency scanning, legacy project support

### Buildr (`buildr_project/`) ✨ NEW
Apache Buildr project with Ruby-based DSL.
- **Detection:** `buildfile` or `Rakefile` with Buildr
- **Features:** Maven coordinate resolution, Ruby DSL builds

### sbt (`sbt_project/`) ✨ NEW
sbt (Scala Build Tool) project for Scala applications.
- **Detection:** `build.sbt` or `project/build.properties`
- **Features:** Incremental compilation, Ivy/Maven repository support

Each example includes a README with build instructions and BazBOM usage.

## Policy Configurations

BazBOM supports policy-as-code through YAML configuration files. Three example configurations are provided:

### `bazbom.yml` - Balanced Policy

A balanced security policy suitable for most production applications.

- Severity threshold: HIGH
- KEV gate: Enabled
- EPSS threshold: 0.5
- License allowlist: Common open source licenses (MIT, Apache-2.0, BSD variants, ISC)

**Use when:**
- Standard production application
- Need balance between security and development velocity
- Want to catch most critical and high severity issues

### `bazbom-strict.yml` - Strict Policy

A strict security policy for critical systems and compliance-heavy environments.

- Severity threshold: CRITICAL
- KEV gate: Enabled
- EPSS threshold: 0.8 (high exploitation probability only)
- License allowlist: Very restrictive (MIT, Apache-2.0, BSD variants only)

**Use when:**
- Financial services, healthcare, or other regulated industries
- Critical infrastructure systems
- High-security requirements
- Need to minimize false positives

### `bazbom-permissive.yml` - Permissive Policy

A permissive policy for development and early-stage projects.

- Severity threshold: MEDIUM
- KEV gate: Disabled
- EPSS threshold: 0.1 (catch more potential issues)
- License restrictions: None

**Use when:**
- Development or staging environments
- Early-stage projects
- Prototyping
- Want more visibility into medium-severity issues

## Using Policy Configurations

Copy one of the example files to your project root:

```bash
# Standard policy
cp examples/bazbom.yml .

# Strict policy
cp examples/bazbom-strict.yml bazbom.yml

# Permissive policy
cp examples/bazbom-permissive.yml bazbom.yml
```

Customize the policy to fit your needs:

```yaml
# bazbom.yml
severity_threshold: HIGH
license_allowlist:
  - MIT
  - Apache-2.0
kev_gate: true
epss_threshold: 0.5
reachability_required: false
vex_auto_apply: true
```

Then run BazBOM:

```bash
# Scan with automatic policy checking
bazbom scan .

# Or explicit policy check
bazbom policy check
```

## CI Enforcement Examples

### GitHub Actions (`ci/policy-check.yml`)

Complete GitHub Actions workflow that:
- Installs BazBOM
- Syncs advisory database
- Generates SBOM and findings
- Runs policy checks
- Uploads SARIF to GitHub Security
- Comments on PRs with violations
- Fails the build if policy violations found

**Usage:**

Copy to your repository:

```bash
mkdir -p .github/workflows
cp examples/ci/policy-check.yml .github/workflows/
```

Customize as needed:
- Adjust Java version if needed
- Modify policy configuration path
- Customize PR comment format
- Add additional steps (build, test, etc.)

**Integration with GitHub Security:**

The workflow automatically uploads SARIF reports to GitHub's Security tab, enabling:
- Security alerts on the Security tab
- Code Scanning annotations on pull requests
- Integration with GitHub Advanced Security features

## Policy Rule Reference

### Severity Threshold

Block vulnerabilities at or above the specified severity level.

```yaml
severity_threshold: HIGH  # Options: NONE, LOW, MEDIUM, HIGH, CRITICAL
```

### KEV Gate

Block any vulnerability in CISA's Known Exploited Vulnerabilities catalog.

```yaml
kev_gate: true  # true = block KEV vulnerabilities, false = allow
```

### EPSS Threshold

Block vulnerabilities with an EPSS (Exploit Prediction Scoring System) score at or above the threshold.

```yaml
epss_threshold: 0.5  # Range: 0.0 to 1.0 (probability of exploitation)
```

Common thresholds:
- `0.1` - Low bar (catches ~10% most likely to be exploited)
- `0.5` - Medium bar (catches ~50% most likely to be exploited)
- `0.8` - High bar (catches ~80% most likely to be exploited)

### License Allowlist

Only allow dependencies with licenses in this list.

```yaml
license_allowlist:
  - MIT
  - Apache-2.0
  - BSD-2-Clause
  - BSD-3-Clause
```

### License Denylist

Block dependencies with licenses in this list.

```yaml
license_denylist:
  - GPL-3.0
  - AGPL-3.0
```

**Note:** Allowlist and denylist are mutually exclusive. Use one or the other, not both.

### Reachability Required

When enabled, only block vulnerabilities that are reachable through bytecode analysis.

```yaml
reachability_required: false  # Future capability
```

### VEX Auto-Apply

Automatically generate VEX (Vulnerability Exploitability eXchange) statements for unreachable vulnerabilities.

```yaml
vex_auto_apply: true  # Future capability
```

## Testing Policy Configurations

Test your policy configuration locally before committing:

```bash
# 1. Copy policy configuration
cp examples/bazbom-strict.yml bazbom.yml

# 2. Sync advisory database
bazbom db sync

# 3. Run scan
bazbom scan .

# 4. Check policy violations
bazbom policy check

# 5. Review outputs
cat policy_result.json
cat policy_violations.sarif
```

## Questions or Issues?

If you have questions or need help with policy configurations:
- Open an issue: https://github.com/cboyd0319/BazBOM/issues
- See documentation: `docs/USAGE.md`
