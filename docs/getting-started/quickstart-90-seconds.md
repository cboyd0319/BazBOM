# BazBOM - 90 Second Quickstart

Get from zero to first security scan in 90 seconds. No configuration required.

## 1. Install (~10 seconds)

**Quick install (macOS/Linux):**
```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

**Or download pre-built binary:**
Visit [github.com/cboyd0319/BazBOM/releases/latest](https://github.com/cboyd0319/BazBOM/releases/latest)

**Or build from source (~60 seconds):**
```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

**Or use Cargo (Rust developers):**
```bash
cargo install --git https://github.com/cboyd0319/BazBOM bazbom
```

> See [Installation Guide](homebrew-installation.md) for all methods including Homebrew (coming soon).

## 2. First Scan (30 seconds)

```bash
# Navigate to any JVM project (Maven, Gradle, or Bazel)
cd /path/to/your/java/project

# One command - generates SBOM and scans for vulnerabilities
bazbom scan .
```

**That's it!** You now have:
-  `sbom/spdx.json` - Complete software bill of materials
-  `findings/sca.sarif` - Security vulnerabilities (OSV/NVD/GHSA)
-  `findings/merged.sarif` - GitHub Code Scanning ready

## 3. Understand Results (45 seconds)

### View SBOM
```bash
# How many dependencies?
jq '.packages | length' sbom/spdx.json

# List all packages
jq '.packages[].name' sbom/spdx.json
```

### View Vulnerabilities
```bash
# Count findings
jq '.runs[0].results | length' findings/merged.sarif

# List critical/high severity issues
jq '.runs[0].results[] | select(.level == "error") | .message.text' findings/merged.sarif
```

### Upload to GitHub (Optional)
Add to `.github/workflows/security.yml`:
```yaml
- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: findings/merged.sarif
```

---

## What Just Happened?

BazBOM automatically:
1.  **Detected** your build system (Maven/Gradle/Bazel)
2.  **Extracted** all dependencies (direct + transitive)
3.  **Scanned** against 3 vulnerability databases
4.  **Generated** standards-compliant SBOM (SPDX 2.3)
5.  **Created** GitHub-ready SARIF report

**No configuration required. Zero telemetry. Runs offline.**

---

## Next Steps (Pick Your Path)

### For Quick Scans (Development)
```bash
# That's it - just run the scan command above
bazbom scan .
```

### For Comprehensive Analysis (Main Branch)
```bash
# Add Semgrep pattern analysis
bazbom scan . --with-semgrep

# Requires: pipx install semgrep
```

### For Deep Security Audits (Weekly/Monthly)
```bash
# Add CodeQL dataflow analysis
bazbom scan . --with-semgrep --with-codeql security-extended

# Requires: CodeQL CLI (see below)
```

### For CI/CD Integration
Copy workflow from: `examples/github-actions/bazbom-scan.yml`

---

## Tool Requirements by Scan Type

| Scan Type | Required | Optional | Time |
|-----------|----------|----------|------|
| **Basic** (SBOM + SCA) | Java (any version) | None | 1-2 min |
| **+ Semgrep** | + Semgrep | None | 3-5 min |
| **+ CodeQL** | + CodeQL CLI | None | 10-20 min |

### Installing Optional Tools

**Semgrep** (for pattern analysis):
```bash
pipx install semgrep
# Or: brew install semgrep
```

**CodeQL** (for dataflow analysis):
```bash
CODEQL_VERSION=2.19.4
curl -LO https://github.com/github/codeql-cli-binaries/releases/download/v${CODEQL_VERSION}/codeql-linux64.zip
unzip codeql-linux64.zip && export PATH="$PWD/codeql:$PATH"
```

---

## Common Use Cases

### "I just want an SBOM for compliance"
```bash
bazbom scan .
# Use: sbom/spdx.json
```

### "I want to know my security vulnerabilities"
```bash
bazbom scan .
# Review: findings/merged.sarif
```

### "I want to upload to GitHub Security tab"
```bash
bazbom scan . --out-dir bazbom-output
# Add GitHub Action with: upload-sarif step
```

### "I want everything (SBOM + vulnerabilities + SAST)"
```bash
bazbom scan . --cyclonedx --with-semgrep
# Or for weekly audits:
bazbom scan . --cyclonedx --with-semgrep --with-codeql security-extended
```

---

## Configuration (Optional)

Create `bazbom.toml` in your repo to set defaults:

```toml
[analysis]
cyclonedx = true          # Always generate CycloneDX too
semgrep = { enabled = true }  # Always run Semgrep

[publish]
github_code_scanning = true   # Enable GitHub upload
```

Now `bazbom scan .` uses these defaults. CLI flags override config.

---

## Troubleshooting

### "No dependencies found"
- Ensure you're in the project root (where `pom.xml`, `build.gradle`, or `WORKSPACE` exists)
- For Bazel: Run `bazel run @maven//:pin` first to generate `maven_install.json`

### "SARIF upload failed"
- Check file size < 10 MB (GitHub limit)
- Validate: `jq . findings/merged.sarif > /dev/null`

### "Semgrep not found"
- Install: `pipx install semgrep` or `brew install semgrep`
- Or skip: Remove `--with-semgrep` flag

---

## Learn More

- **[Complete Usage Guide](../user-guide/usage.md)** - All commands and options
- **[Orchestrated Scanning](../integrations/orchestrated-scan.md)** - Advanced workflows
- **[Architecture Overview](../ARCHITECTURE.md)** - Architecture details
- **[CI Integration](../CI.md)** - CI/CD workflows

---

## Support

-  [Documentation](README.md)
-  [Report Issues](https://github.com/cboyd0319/BazBOM/issues)
-  [Discussions](https://github.com/cboyd0319/BazBOM/discussions)
-  [Security Issues](SECURITY.md)

---

**Time to first scan: ~90 seconds**  
**Time to understand results: +45 seconds**  
**Time to production-grade security: ~2 minutes total**

That's BazBOM. Simple, fast, secure. 
