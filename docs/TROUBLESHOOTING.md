# Troubleshooting

Top failures with exact error text, root cause, and fixes.

## Installation & Setup

### 1. "bazbom: command not found"

**Error text:**
```
bash: bazbom: command not found
```

**Cause:** Binary not in PATH

**Fix:**
```bash
# Verify installation
which bazbom

# If missing, reinstall
brew install bazbom
# OR
sudo cp target/release/bazbom /usr/local/bin/

# Verify
bazbom --version
```

### 2. "Java not found (required for reachability analysis)"

**Error text:**
```
Error: Java 11+ required for --reachability flag
```

**Cause:** Java not installed or not in PATH

**Fix:**
```bash
# Check Java
java -version

# Install if missing (macOS)
brew install openjdk@17

# Install if missing (Ubuntu)
sudo apt install openjdk-17-jdk

# Verify
java -version
```

**Alternative:** Skip reachability:
```bash
bazbom scan . --fast
```

## Build System Detection

### 3. "No build system detected"

**Error text:**
```
Error: No supported build system found in /path/to/project
Searched for: pom.xml, build.gradle, build.gradle.kts, BUILD, MODULE.bazel
```

**Cause:** Missing build files or unsupported project structure

**Fix:**
```bash
# Verify build files exist
ls -la pom.xml build.gradle* BUILD* MODULE.bazel

# Force build system detection
bazbom scan . --build-system maven
# OR
bazbom scan . --build-system gradle
# OR
bazbom scan . --build-system bazel
```

### 4. "maven_install.json not found (Bazel)"

**Error text:**
```
Error: maven_install.json not found. Required for Bazel JVM dependency extraction.
```

**Cause:** `rules_jvm_external` not configured or lockfile not pinned

**Fix:**
```bash
# Generate lockfile
bazel run @maven//:pin

# Verify
ls -la maven_install.json

# Re-scan
bazbom scan . --bazel-targets //...
```

## SBOM Generation

### 5. "SBOM empty / zero packages"

**Error text:**
```
Warning: Generated SBOM contains 0 packages
```

**Cause:** No dependencies found or wrong target

**Fix:**
```bash
# Maven: Verify dependencies exist
mvn dependency:tree

# Gradle: Verify dependencies exist
./gradlew dependencies

# Bazel: Verify targets have deps
bazel query 'deps(//src/java:app)'

# Force full scan
bazbom scan . --include-test-deps
```

### 6. "Aspect not found / no packages emitted (Bazel)"

**Error text:**
```
Error: Bazel aspect 'packages_used' failed to execute
No dependency data emitted
```

**Cause:** Incompatible Bazel version or aspect not loaded

**Fix:**
```bash
# Check Bazel version
bazel --version
# Required: 6.0+

# Update if needed
echo "7.6.2" > .bazelversion

# Verify aspect exists
ls tools/supplychain/aspects.bzl

# Re-run with explicit aspect
bazbom scan . --bazel-targets //... --aspect-path tools/supplychain/aspects.bzl
```

### 7. "SPDX validation failed"

**Error text:**
```
Error: Generated SBOM failed SPDX 2.3 validation
Invalid field: packages[3].versionInfo (missing)
```

**Cause:** Incomplete dependency metadata

**Fix:**
```bash
# Maven: Ensure all artifacts have versions
mvn versions:display-dependency-updates

# Gradle: Check for version mismatches
./gradlew dependencyInsight --dependency <name>

# Bazel: Regenerate lockfile
bazel run @maven//:pin

# Re-scan with validation
bazbom scan . --validate-schemas
```

## Vulnerability Scanning

### 8. "Advisory database sync timeout"

**Error text:**
```
Error: Failed to sync advisory database
Timeout after 300 seconds
```

**Cause:** Network issues, rate limiting, or large database

**Fix:**
```bash
# Retry with longer timeout
bazbom db sync --timeout 600

# Use cached database
bazbom scan . --offline-mode

# Manual sync to specific path
bazbom db sync --out /path/to/db
```

### 9. "OSV API rate limit exceeded"

**Error text:**
```
Error: OSV API returned 429 Too Many Requests
Rate limit: 100 requests/minute
```

**Cause:** Too many API calls in short period

**Fix:**
```bash
# Use local database (no API calls)
bazbom db sync
bazbom scan . --offline-mode

# Wait and retry
sleep 60
bazbom scan .
```

### 10. "Reachability analysis failed"

**Error text:**
```
Error: bazbom-reachability.jar exited with code 1
Failed to analyze bytecode for com.example.App
```

**Cause:** Corrupted JAR or incompatible bytecode

**Fix:**
```bash
# Skip reachability
bazbom scan . --fast

# Check JAR integrity
jar tf target/app.jar

# Re-build and retry
mvn clean package
bazbom scan . --reachability
```

## Policy & Configuration

### 11. "Policy file parse error"

**Error text:**
```
Error: Failed to parse policy file: bazbom.yml
YAML syntax error at line 15: invalid indentation
```

**Cause:** Invalid YAML syntax

**Fix:**
```bash
# Validate YAML
yamllint bazbom.yml

# Use template
bazbom policy init --template pci-dss

# Test policy
bazbom policy check --dry-run
```

### 12. "Policy violation: blocked license"

**Error text:**
```
Error: Policy violation - GPL-3.0 license found
Blocked by policy: PCI-DSS
Package: commons-io:commons-io:2.11.0
```

**Cause:** Dependency has non-compliant license

**Fix:**
```bash
# View full policy report
bazbom policy check --verbose

# Exclude specific package (if allowed)
# Add to bazbom.yml:
policy:
  exceptions:
    - package: "commons-io:commons-io:2.11.0"
      reason: "Approved by legal team"

# Replace with compliant alternative
# Update pom.xml/build.gradle to use different package
```

## CI/CD Issues

### 13. "SARIF upload failed: security-events permission"

**Error text:**
```
Error: Resource not accessible by integration
Requires: security-events: write permission
```

**Cause:** Missing GitHub Actions permissions

**Fix:**
```yaml
# Add to workflow file
jobs:
  scan:
    permissions:
      contents: read
      security-events: write
    steps:
      # ... rest of workflow
```

### 14. "GitHub Actions timeout (6 hours)"

**Error text:**
```
Error: The job running on runner ubuntu-latest has exceeded the maximum execution time of 360 minutes.
```

**Cause:** Full monorepo scan too slow

**Fix:**
```yaml
# Use incremental scanning
- name: Scan Affected Targets
  run: |
    bazbom scan . \
      --bazel-affected-by-files $(git diff --name-only origin/main...HEAD)
```

**OR: Split into multiple jobs**

```yaml
jobs:
  scan-services:
    # ...
    run: bazbom scan . --bazel-targets-query 'kind(java_binary, //services/...)'

scan-libraries:
  # ...
  run: bazbom scan . --bazel-targets-query 'kind(java_library, //lib/...)'
```

### 15. "Artifact upload size exceeded (10 GB limit)"

**Error text:**
```
Error: Artifact size 11.2 GB exceeds GitHub's 10 GB limit
```

**Cause:** SBOMs + logs + build outputs too large

**Fix:**
```yaml
# Upload only SBOMs
- uses: actions/upload-artifact@v4
  with:
    name: sbom
    path: |
      ./reports/sbom.spdx.json
      ./reports/sca_findings.sarif
    # Exclude large files
```

## Performance Issues

### 16. "Scan too slow (>30 minutes)"

**Symptom:** Full workspace scan takes 30-45 minutes

**Cause:** No caching or parallel execution

**Fix:**

**For Bazel:**
```bash
# Enable remote cache (.bazelrc)
build --remote_cache=https://cache.example.com

# Parallel execution
bazel build --jobs=auto //...

# Incremental scanning
bazbom scan . --bazel-affected-by-files $(git diff --name-only HEAD~1)
```

**For Maven:**
```bash
# Parallel builds
mvn -T 1C clean install

# Skip tests during scan
mvn install -DskipTests
```

**For Gradle:**
```bash
# Enable daemon
echo "org.gradle.daemon=true" >> gradle.properties

# Parallel execution
./gradlew build --parallel --max-workers=8
```

### 17. "Memory exhausted (OOM)"

**Error text:**
```
Error: java.lang.OutOfMemoryError: Java heap space
```

**Cause:** Large JAR analysis or too many dependencies

**Fix:**
```bash
# Increase heap size (Maven)
export MAVEN_OPTS="-Xmx4g"

# Increase heap size (Gradle)
echo "org.gradle.jvmargs=-Xmx4g" >> gradle.properties

# Bazel: Increase JVM heap
bazel build --host_jvm_args=-Xmx4g //...

# Skip reachability (reduces memory)
bazbom scan . --fast
```

## Platform-Specific

### 18. "Permission denied (macOS)"

**Error text:**
```
zsh: permission denied: bazbom
```

**Cause:** macOS Gatekeeper blocking unsigned binary

**Fix:**
```bash
# Remove quarantine attribute
xattr -d com.apple.quarantine /usr/local/bin/bazbom

# OR: Allow in System Preferences
# System Preferences → Security & Privacy → Allow

# Verify
bazbom --version
```

### 19. "DLL load failed (Windows)"

**Error text:**
```
ImportError: DLL load failed while importing _sqlite3
```

**Cause:** Missing Visual C++ Redistributable

**Fix:**
```powershell
# Install VC++ Redistributable
choco install vcredist-all

# Verify
bazbom --version
```

### 20. "SSL certificate verify failed"

**Error text:**
```
Error: [SSL: CERTIFICATE_VERIFY_FAILED] certificate verify failed
```

**Cause:** Corporate proxy or self-signed certificates

**Fix:**
```bash
# Add CA bundle
export REQUESTS_CA_BUNDLE=/path/to/ca-bundle.crt

# OR: Disable verification (not recommended)
export CURL_CA_BUNDLE=""

# Re-run
bazbom db sync
```

## Debugging Tips

### Enable Verbose Logging

```bash
bazbom scan . --log-level debug
```

### Dry Run (No Output)

```bash
bazbom scan . --dry-run
```

### Inspect Outputs

```bash
# Validate SBOM
cat sbom.spdx.json | jq '.packages | length'

# Check for CVEs
cat sca_findings.json | jq '.vulnerabilities'

# Validate SARIF
cat sca_findings.sarif | jq '.runs[0].results | length'
```

### Check Environment

```bash
# Verify versions
bazbom --version
java -version
mvn --version
gradle --version
bazel --version

# Check paths
which bazbom
which java

# Verify advisory database
ls -lh ~/.bazbom/db/
```

## Getting Help

**Still stuck?**

1. Check [documentation](README.md)
2. Search [existing issues](https://github.com/cboyd0319/BazBOM/issues)
3. Ask in [discussions](https://github.com/cboyd0319/BazBOM/discussions)
4. File a [bug report](https://github.com/cboyd0319/BazBOM/issues/new?template=bug_report.md)

**Include in bug reports:**
- Error message (full text)
- Command that failed
- `bazbom --version` output
- Build system and version
- OS and architecture

## Quick Fixes by Symptom

| Symptom | Quick Fix |
|---------|-----------|
| Command not found | `brew install bazbom` |
| Java not found | Install OpenJDK 17+ |
| No build system | Check for pom.xml/build.gradle/BUILD |
| Empty SBOM | Verify dependencies with `mvn dependency:tree` |
| Aspect failed | Update Bazel to 6.0+ |
| Database sync timeout | Use `--offline-mode` |
| Policy violation | Check `bazbom.yml` syntax |
| SARIF upload failed | Add `security-events: write` permission |
| Too slow | Use `--bazel-affected-by-files` |
| OOM error | Increase heap: `-Xmx4g` |

## Related Docs

- [Usage guide](USAGE.md) - Commands and flags
- [Architecture](ARCHITECTURE.md) - How components work
- [CI integration](CI.md) - GitHub Actions debugging
- [User guide](user-guide/troubleshooting.md) - Extended troubleshooting
