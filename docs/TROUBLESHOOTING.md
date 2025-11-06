# Troubleshooting

Common issues and solutions for BazBOM.

## Build Issues

### Bazel Not Found

**Symptom**: `bazel: command not found`

**Solution**: Install Bazelisk or Bazel:

```bash
# Install Bazelisk (recommended)
# macOS
brew install bazelisk

# Linux
wget https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64
chmod +x bazelisk-linux-amd64
sudo mv bazelisk-linux-amd64 /usr/local/bin/bazel

# Windows
choco install bazelisk
```

### Wrong Bazel Version

**Symptom**: `This version of Bazel requires Java X but you have Java Y`

**Solution**: The `.bazelversion` file specifies the required version. Bazelisk automatically downloads it.

```bash
# Verify version
cat .bazelversion

# Use Bazelisk (auto-downloads correct version)
bazelisk build //...
```

### Build Fails with "No such target"

**Symptom**: `ERROR: no such target '//:sbom_all'`

**Solution**: The target may not exist yet. Check available targets:

```bash
# List all targets
bazel query //...

# List targets in root package
bazel query //:*

# Build what exists
bazel build //...
```

### Dependency Download Fails

**Symptom**: `Error downloading Maven dependency`

**Solution**: Check network connectivity and proxy settings:

```bash
# Set proxy in .bazelrc (if needed)
echo "startup --host_jvm_args=-Dhttp.proxyHost=proxy.example.com" >> .bazelrc
echo "startup --host_jvm_args=-Dhttp.proxyPort=8080" >> .bazelrc

# Clear cache and retry
bazel clean --expunge
bazel build //...
```

## SBOM Generation Issues

### No SBOM Files Generated

**Symptom**: `find bazel-bin -name "*.spdx.json"` returns nothing

**Solution**:

1. Verify SBOM targets exist:
   ```bash
   bazel query 'filter("\.sbom$", //...)'
   ```

2. Build SBOM targets explicitly:
   ```bash
   bazel build //examples/minimal_java:app.sbom
   ```

3. Check aspect is properly applied:
   ```bash
   bazel build //examples/minimal_java:app --aspects=//tools/supplychain:aspects.bzl%sbom_aspect
   ```

### SBOM Missing Dependencies

**Symptom**: SBOM contains fewer packages than expected

**Solution**: Ensure transitive dependencies are collected:

```python
# In tools/supplychain/aspects.bzl
sbom_aspect = aspect(
    attr_aspects = ["deps", "runtime_deps", "exports"],  # Add all dep types
    # ...
)
```

### Invalid SPDX Format

**Symptom**: Validation fails with schema errors

**Solution**: Check the `write_sbom.py` output format:

```bash
# Validate against schema
pip install check-jsonschema
check-jsonschema \
  --schemafile https://raw.githubusercontent.com/spdx/spdx-spec/v2.3/schemas/spdx-schema.json \
  bazel-bin/path/to/package.spdx.json

# Check for common issues
jq '.spdxVersion' bazel-bin/path/to/package.spdx.json  # Should be "SPDX-2.3"
jq '.dataLicense' bazel-bin/path/to/package.spdx.json  # Should be "CC0-1.0"
```

### Shaded JARs Not Analyzed

**Symptom**: Dependencies in shaded/uber JARs not detected

**Solution**: Shaded JARs require special handling:

```python
# Option 1: Analyze before shading
bazel build //path/to:unshaded_lib.sbom

# Option 2: Extract and analyze shaded JAR manifest
# (Manual process - requires custom tooling)
```

**Workaround**: Generate SBOMs from un-shaded dependencies, then mark the shaded artifact as containing them.

## SCA Issues

### OSV API Rate Limiting

**Symptom**: `429 Too Many Requests` from OSV

**Solution**: Implement rate limiting and caching:

```python
# In tools/supplychain/osv_query.py
import time

def query_osv_with_backoff(package, version, max_retries=3):
    for attempt in range(max_retries):
        response = requests.post("https://api.osv.dev/v1/query", ...)
        if response.status_code == 429:
            wait_time = 2 ** attempt  # Exponential backoff
            time.sleep(wait_time)
            continue
        return response
    raise Exception("Rate limited after retries")
```

### OSV API Unreachable

**Symptom**: `Connection refused` or `Connection timeout`

**Solution**: Check network connectivity and firewall rules:

```bash
# Test connectivity
curl -X POST https://api.osv.dev/v1/query \
  -H "Content-Type: application/json" \
  -d '{"package":{"name":"lodash","ecosystem":"npm"},"version":"4.17.0"}'

# Use proxy if needed
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
```

### No Vulnerabilities Found

**Symptom**: SARIF file is empty or has no results

**Possible Causes**:

1. **Good news**: No vulnerabilities in your dependencies!
2. **Package ecosystem mismatch**: Ensure correct ecosystem in purl

```bash
# Check package URLs in SBOM
jq '.packages[].externalRefs[] | select(.referenceType=="purl")' \
  bazel-bin/path/to/package.spdx.json

# Should be like: pkg:maven/org.example/my-lib@1.0.0
```

3. **Version format mismatch**: OSV is sensitive to version format

```bash
# These are different to OSV:
# pkg:maven/org.example/my-lib@1.0.0       Correct
# pkg:maven/org.example/my-lib@1.0         Different version
# pkg:maven/org.example/my-lib@v1.0.0      Extra 'v' prefix
```

### SARIF Upload Fails

**Symptom**: GitHub Actions fails to upload SARIF to Code Scanning

**Solution**: Verify SARIF format and workflow permissions:

```yaml
# .github/workflows/supplychain.yml
permissions:
  contents: read
  security-events: write  # Required for SARIF upload

jobs:
  scan:
    steps:
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: bazel-bin/vulnerabilities.sarif.json
          # Ensure file exists and is valid SARIF 2.1.0
```

Common SARIF upload errors:

- **Invalid JSON**: Validate with `jq . file.sarif.json`
- **Wrong schema version**: Must be SARIF 2.1.0
- **File too large**: GitHub has a 10MB limit per SARIF file
- **Missing permissions**: Workflow needs `security-events: write`

## Vulnerability Enrichment Issues

### KEV Enrichment Fails

**Symptom**: `Warning: KEV fetch failed` in console output

**Possible Causes**:

1. **Network connectivity**: Cannot reach CISA KEV API
2. **Stale cache used**: Enrichment continues with older data

**Solution**:

```bash
# Test KEV API connectivity
curl -s "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json" | jq '.catalogVersion'

# Clear KEV cache to force refresh
rm -rf .bazel-cache/kev/

# Test KEV enrichment directly
python tools/supplychain/kev_enrichment.py CVE-2021-44228
```

### EPSS Enrichment Slow

**Symptom**: Enrichment takes several minutes for many CVEs

**Solution**: EPSS enrichment uses batching (100 CVEs/request) but large vulnerability lists take time.

```bash
# Check batch size in enrichment
# Default: 100 CVEs per request (optimal)

# Monitor EPSS requests
python tools/supplychain/osv_query.py \
  --sbom bazel-bin/workspace.spdx.json \
  --output vulnerabilities.json \
  --enrich 2>&1 | grep "EPSS"
```

**Expected performance**:
- 100 CVEs: ~5-10 seconds
- 500 CVEs: ~30-40 seconds  
- 1000+ CVEs: ~60-90 seconds

Cache is used for subsequent runs (24-hour TTL).

### GHSA Rate Limiting

**Symptom**: `Warning: GHSA query failed` with rate limit errors

**Solution**: Use GitHub token for higher rate limits:

```bash
# Without token: 60 requests/hour
# With token: 5000 requests/hour

export GITHUB_TOKEN="ghp_xxxxxxxxxxxxx"

bazel run //tools/supplychain:osv_query -- \
  --sbom bazel-bin/workspace.spdx.json \
  --output vulnerabilities.json \
  --github-token "${GITHUB_TOKEN}"
```

**Generate GitHub token**:
1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Click "Generate new token (classic)"
3. Select scope: `public_repo` (for public repos) or `repo` (for private)
4. Generate and copy token

### VulnCheck API Key Invalid

**Symptom**: `VulnCheck API authentication failed - check API key`

**Solution**: Verify VulnCheck API key:

```bash
# Test VulnCheck API key
curl -H "Authorization: Bearer YOUR_API_KEY" \
  "https://api.vulncheck.com/v3/index/vulncheck-kev?cve=CVE-2021-44228"

# Set API key environment variable
export VULNCHECK_API_KEY="your-api-key"

# Or pass directly
python tools/supplychain/osv_query.py \
  --sbom bazel-bin/workspace.spdx.json \
  --output vulnerabilities.json \
  --vulncheck-api-key "${VULNCHECK_API_KEY}"
```

**Get VulnCheck API key**: Sign up at https://vulncheck.com (free tier: 100 req/day)

### No Enrichment Data

**Symptom**: Enrichment runs but findings don't have KEV/EPSS data

**Possible Causes**:

1. **CVE ID not found**: Vulnerability uses non-CVE identifier (e.g., GHSA-xxxx)
2. **Recent CVE**: Not yet in KEV/EPSS databases
3. **Enrichment disabled**: `--no-enrich` flag used

**Solution**:

```bash
# Check if CVE has enrichment data available
python tools/supplychain/kev_enrichment.py CVE-2021-44228
python tools/supplychain/epss_enrichment.py CVE-2021-44228

# Verify enrichment is enabled
python tools/supplychain/osv_query.py \
  --sbom bazel-bin/workspace.spdx.json \
  --output vulnerabilities.json \
  --enrich  # Explicitly enable
```

### Risk Score Seems Wrong

**Symptom**: Risk score doesn't match expectation

**Explanation**: Risk score is composite (0-100) based on:

```
Risk Score = (CVSS × 0.40) + (EPSS × 0.30) + (KEV × 0.20) + (Exploit × 0.10)
```

**Example calculations**:

```
High Risk (97.5):
- CVSS: 10.0 → 40 points (40%)
- EPSS: 0.95 → 28.5 points (30%)
- KEV: true → 20 points (20%)
- Exploit weaponized: true → 10 points (10%)
= 98.5 total

Medium Risk (45.0):
- CVSS: 7.0 → 28 points (40%)
- EPSS: 0.10 → 3 points (30%)  
- KEV: false → 0 points (20%)
- Exploit available: true → 5 points (10%)
= 36 total
```

**Verify calculation**:

```bash
# Check individual components
jq '.vulnerabilities[] | select(.cve=="CVE-xxxx") | {cve, cvss_score, epss, kev, exploit, risk_score}' \
  bazel-bin/vulnerabilities_enriched.json
```

## Python Tool Issues

### Python Dependencies Not Found

**Symptom**: `ModuleNotFoundError: No module named 'requests'`

**Solution**: Ensure Python dependencies are properly declared:

```python
# In BUILD.bazel for Python tools
py_binary(
    name = "osv_query",
    srcs = ["osv_query.py"],
    deps = [
        "@pip//requests",
        "@pip//jsonschema",
    ],
)
```

Install dependencies:
```bash
# Using rules_python
bazel run @python//pip install requests jsonschema
```

### Python Version Mismatch

**Symptom**: `SyntaxError` or incompatible features

**Solution**: Verify Python version requirements:

```bash
# Check Python version
python --version  # Should be 3.9+

# Specify Python version in WORKSPACE
load("@rules_python//python:repositories.bzl", "python_register_toolchains")

python_register_toolchains(
    name = "python39",
    python_version = "3.9",
)
```

## CI/CD Issues

### CI Fails Locally Passing

**Symptom**: Tests pass locally but fail in CI

**Common Causes**:

1. **Bazel cache differences**:
   ```bash
   # In CI, use clean build
   bazel clean
   bazel build //...
   ```

2. **Environment differences**:
   ```bash
   # Check Bazel info
   bazel info | grep -E "(release|workspace)"
   ```

3. **Network access in CI**:
   ```yaml
   # Ensure CI has internet access for OSV queries
   # Or use offline mode with pre-downloaded data
   ```

### Workflow Permissions Errors

**Symptom**: `403 Forbidden` or `Permission denied` in GitHub Actions

**Solution**: Add necessary permissions to workflow:

```yaml
# .github/workflows/supplychain.yml
permissions:
  contents: read      # Read code
  security-events: write  # Upload SARIF
  actions: read       # Read workflow artifacts
```

### Artifacts Not Uploaded

**Symptom**: Can't find SBOM/SARIF files in workflow artifacts

**Solution**: Ensure files are uploaded correctly:

```yaml
- name: Upload SBOMs
  uses: actions/upload-artifact@v4
  with:
    name: sboms
    path: |
      bazel-bin/**/*.spdx.json
    if-no-files-found: error  # Fail if no files found
```

## Performance Issues

### Slow SBOM Generation

**Symptom**: `bazel build //:sbom_all` takes too long

**Solutions**:

1. **Use remote cache**:
   ```bash
   # In .bazelrc
   build --remote_cache=https://cache.example.com
   ```

2. **Parallelize builds**:
   ```bash
   # Increase job count
   build --jobs=8
   ```

3. **Profile the build**:
   ```bash
   bazel build //:sbom_all --profile=profile.json
   bazel analyze-profile profile.json
   ```

### Slow OSV Queries

**Symptom**: SCA takes too long to query OSV

**Solutions**:

1. **Batch queries**:
   ```python
   # Query multiple packages in one request
   response = requests.post(
       "https://api.osv.dev/v1/querybatch",
       json={"queries": [{"package": {"name": "...", "ecosystem": "..."}, "version": "..."}]}
   )
   ```

2. **Cache results**:
   ```python
   # Cache OSV responses locally
   import json
   cache_file = ".osv_cache.json"
   # Load/save cache between runs
   ```

3. **Limit scope**:
   ```bash
   # Only scan production dependencies
   bazel build //src:app.sbom  # Not //:sbom_all
   ```

## Lockfile Issues

### Maven Lockfile Out of Date

**Symptom**: `ERROR: maven_install.json is out of date`

**Solution**: Regenerate the lockfile:

```bash
# Regenerate lockfile
bazel run @maven//:pin

# Or with unpinned dependencies
bazel run @unpinned_maven//:pin

# Commit the updated file
git add maven_install.json
git commit -m "chore: update Maven lockfile"
```

### Lockfile Conflicts in PRs

**Symptom**: Merge conflicts in `maven_install.json`

**Solution**: Regenerate after merge:

```bash
# After resolving other conflicts
git merge main
bazel run @maven//:pin
git add maven_install.json
git commit -m "chore: regenerate Maven lockfile after merge"
```

## Documentation Issues

### Broken Links in Docs

**Symptom**: Links in documentation return 404

**Solution**: Use relative links and validate:

```bash
# Check links locally
npm install -g markdown-link-check
find docs -name "*.md" -exec markdown-link-check {} \;

# Or use the workflow
# .github/workflows/docs-links-check.yml runs automatically
```

### Markdown Lint Failures

**Symptom**: CI fails on markdown lint

**Solution**: Fix linting errors:

```bash
# Install markdownlint
npm install -g markdownlint-cli

# Check locally
markdownlint docs/**/*.md

# Auto-fix
markdownlint --fix docs/**/*.md

# Check against config
markdownlint --config .markdownlint.json docs/**/*.md
```

## Getting More Help

If you're still stuck:

1. **Check existing issues**: Search [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
2. **Review logs**: Include full error messages when reporting
3. **Minimal reproduction**: Create a minimal example that reproduces the issue
4. **Open an issue**: Provide environment details (OS, Bazel version, Java version)

**Useful debugging commands**:

```bash
# Bazel info
bazel info

# Build with verbose output
bazel build //... --verbose_failures

# Show full command lines
bazel build //... --subcommands

# Debug aspects
bazel build //... --aspects=//tools/supplychain:aspects.bzl%sbom_aspect --output_groups=sbom
```

## Rust CLI Issues

### Binary Not Found After Installation

**Symptom**: `bazbom: command not found` after installing via Homebrew or downloading binary

**Solution**: Verify binary is in PATH:

```bash
# Check if binary exists
which bazbom

# If installed via Homebrew
brew list bazbom | grep bin

# Manually add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"

# Or for Homebrew
export PATH="$(brew --prefix)/bin:$PATH"
```

Add the PATH export to your shell configuration file (`.bashrc`, `.zshrc`, etc.) for persistence.

### Build System Not Detected

**Symptom**: `Error: Could not detect build system`

**Solution**: Ensure you are in a project directory with one of the supported build files:

- Maven: `pom.xml`
- Gradle: `build.gradle` or `build.gradle.kts`
- Bazel: `WORKSPACE`, `MODULE.bazel`, or `BUILD.bazel`

```bash
# Check current directory
ls -la | grep -E "(pom.xml|build.gradle|WORKSPACE|MODULE.bazel)"

# Run from project root
cd /path/to/project/root
bazbom scan .
```

### SPDX Output Validation Fails

**Symptom**: Generated SPDX file fails validation

**Solution**: Validate output against JSON schema:

```bash
# Install JSON schema validator
pip install check-jsonschema

# Validate SPDX output
check-jsonschema \
  --schemafile https://raw.githubusercontent.com/spdx/spdx-spec/v2.3/schemas/spdx-schema.json \
  output.spdx.json

# Check required fields
jq '.spdxVersion, .dataLicense, .SPDXID, .name' output.spdx.json
```

Expected output:
```json
"SPDX-2.3"
"CC0-1.0"
"SPDXRef-DOCUMENT"
"project-name"
```

### SARIF Output Not Valid

**Symptom**: SARIF file fails GitHub Code Scanning upload

**Solution**: Verify SARIF format:

```bash
# Validate SARIF schema
jq '.$schema' output.sarif.json
# Should be: https://json.schemastore.org/sarif-2.1.0.json

# Check version
jq '.version' output.sarif.json
# Should be: "2.1.0"

# Verify runs array exists
jq '.runs | length' output.sarif.json
# Should be: 1 or more
```

### Cargo Build Fails

**Symptom**: `cargo build` fails with compilation errors

**Solution**: Ensure you have the correct Rust version:

```bash
# Check Rust version
rustc --version

# Update Rust
rustup update stable

# Clean and rebuild
cargo clean
cargo build --release
```

If errors persist, check for:
- Missing system dependencies (OpenSSL, pkg-config)
- Network connectivity for crate downloads
- Sufficient disk space

### Advisory Database Sync Fails

**Symptom**: `bazbom db sync` fails with network errors

**Solution**: Check network connectivity and proxy settings:

```bash
# Test OSV connectivity
curl -I https://osv.dev

# Test NVD connectivity  
curl -I https://nvd.nist.gov

# Set proxy if needed
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080

# Retry sync
bazbom db sync
```

### Cache Directory Permission Denied

**Symptom**: `Error: Permission denied` when accessing cache

**Solution**: Fix cache directory permissions:

```bash
# Default cache location
CACHE_DIR="$HOME/.bazbom/cache"

# Check permissions
ls -la "$CACHE_DIR"

# Fix permissions
chmod -R u+rw "$CACHE_DIR"

# Or use custom cache location
export BAZBOM_CACHE_DIR=/tmp/bazbom-cache
mkdir -p "$BAZBOM_CACHE_DIR"
bazbom scan .
```

### Reachability Analysis Fails

**Symptom**: `Error: Java not found` or reachability fails

**Solution**: Reachability requires Java 11 or later:

```bash
# Check Java installation
java -version

# Install Java if needed (macOS)
brew install openjdk@11

# Install Java if needed (Linux)
sudo apt-get install openjdk-11-jdk

# Add Java to PATH
export PATH="/usr/local/opt/openjdk@11/bin:$PATH"

# Disable reachability if Java not available
bazbom scan . --no-reachability
```

Note: Reachability analysis is optional and can be omitted for basic SBOM generation.

### Output File Already Exists

**Symptom**: `Error: Output file already exists`

**Solution**: Remove existing file or use overwrite flag:

```bash
# Remove existing output
rm output.spdx.json

# Or force overwrite (if supported)
bazbom scan . --force

# Or use different output location
bazbom scan . --output custom-output.spdx.json
```

### Policy Check Fails Unexpectedly

**Symptom**: Policy check fails but no violations are visible

**Solution**: Verify policy configuration:

```bash
# Check for policy file
ls -la bazbom.yml .bazbom.yml

# Validate policy syntax
cat bazbom.yml

# Run with verbose output
bazbom policy check --verbose

# Use default policy if custom policy is invalid
bazbom policy check --default-policy
```

### Binary Signature Verification Fails

**Symptom**: Cosign verification fails for downloaded binary

**Solution**: Ensure you have matching binary and signature files:

```bash
# Download both files
curl -LO https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-apple-darwin.tar.gz
curl -LO https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-apple-darwin.tar.gz.sig

# Install cosign
brew install cosign

# Verify signature
cosign verify-blob \
  --signature bazbom-x86_64-apple-darwin.tar.gz.sig \
  bazbom-x86_64-apple-darwin.tar.gz
```

If verification fails:
- Ensure signature and binary are from the same release
- Check cosign is up to date
- Verify binary was not modified after download

### Performance Issues with Large Projects

**Symptom**: Scan takes too long on large projects

**Solution**: Use incremental analysis and caching:

```bash
# Enable caching
export BAZBOM_CACHE_DIR="$HOME/.bazbom/cache"

# Scan specific modules (Maven)
bazbom scan ./module-name

# Exclude test dependencies
bazbom scan . --exclude-test-scope

# Parallel processing (if supported)
bazbom scan . --parallel
```

### Memory Issues

**Symptom**: Process killed or out of memory errors

**Solution**: Increase memory limits or reduce scope:

```bash
# Increase Rust stack size
export RUST_MIN_STACK=8388608

# Scan smaller subsets
bazbom scan ./src

# For large monorepos, scan incrementally
for dir in */; do
  bazbom scan "$dir"
done
```
