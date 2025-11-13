# BazBOM Quick Reference

**One-page cheat sheet for common BazBOM operations**

---

## Installation

```bash
# Clone and build (current distribution)
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
bazbom --version
```

> Package manager installs (Homebrew, winget, etc.) are not yet published. Track [docs/getting-started/homebrew-installation.md](getting-started/homebrew-installation.md) for the latest status.

---

## Quick Start (3 Commands)

```bash
# 1. Scan project (auto-detects build system)
bazbom scan .

# 2. Check policy violations
bazbom policy check

# 3. Get fix suggestions
bazbom fix --suggest
```

---

## Common Scan Operations

```bash
# Basic SBOM generation
bazbom scan .                           # Auto-detect build system, SPDX output
bazbom scan . --format cyclonedx        # CycloneDX format
bazbom scan . --out-dir ./reports       # Custom output directory

# Short flag aliases (v6.5.0+)
bazbom scan -f spdx -o ./reports        # Short form of above
bazbom scan -r -s -m                    # Reachability + Semgrep + ML risk

# Named profiles (v6.5.0+)
bazbom scan --profile strict            # Use predefined "strict" profile from bazbom.toml
bazbom scan -p fast                     # Use "fast" profile (short form)
bazbom scan -p ci                       # Use "ci" profile for pipelines

# Fast mode (skip reachability analysis)
bazbom scan . --fast                    # <10 second scans

# Full analysis
bazbom scan . --reachability            # Include call graph analysis
bazbom scan -r                          # Short form (v6.5.0+)
bazbom scan . --ml-risk                 # ML-enhanced risk scoring
bazbom scan -m                          # Short form (v6.5.0+)

# Machine-readable output (v6.5.0+)
bazbom scan . --json                    # JSON output for automation/CI/CD
bazbom scan --json | jq '.vulnerabilities[] | select(.severity == "CRITICAL")'

# Diff mode (v6.5.0+)
bazbom scan --diff --baseline=baseline.json  # Compare with previous scan
bazbom scan -d --baseline=baseline.json      # Short form

# Bazel-specific
bazbom scan . --bazel-targets //app:main                              # Specific target
bazbom scan . --bazel-targets-query 'kind(java_binary, //...)'       # Query expression
bazbom scan . --bazel-affected-by-files src/main.java src/util.java  # Incremental
```

---

## Policy Operations

```bash
# Check policy violations
bazbom policy check                     # Use default policy
bazbom policy check --policy custom.yml # Custom policy file

# Initialize policy template
bazbom policy init                      # Interactive wizard
bazbom policy init --template strict    # Strict template
bazbom policy init --template permissive # Permissive template

# Validate policy file
bazbom policy validate --policy policy.yml
```

---

## Fix Operations

```bash
# Get fix suggestions (safe, read-only)
bazbom fix --suggest                    # Show recommended fixes
bazbom fix --suggest --ml-prioritize    # ML-enhanced prioritization

# Apply fixes
bazbom fix --apply                      # Apply to local files
bazbom fix --pr                         # Create GitHub PR
bazbom fix --interactive                # Interactive batch mode

# LLM-powered fixes (privacy-first)
bazbom fix --llm                        # Uses local Ollama
bazbom fix --llm --llm-provider anthropic --llm-model claude-3-opus-20240229
```

---

## Advisory Database

```bash
# Sync local mirrors (for offline use)
bazbom db sync                          # Download OSV, NVD, GHSA, KEV, EPSS
bazbom db sync --sources osv,nvd        # Specific sources only
```

---

## License Compliance

```bash
# Generate obligations report
bazbom license obligations

# Check license compatibility
bazbom license compatibility

# Detect copyleft contamination
bazbom license contamination
```

---

## Interactive Tools

```bash
# Terminal UI for exploring dependencies (v6.5.0+ with enhancements)
bazbom explore --sbom sbom.spdx.json
# New features:
#  - Regex/glob search modes (press 'r' to toggle)
#  - Case-insensitive search (press 'i' to toggle)
#  - Clickable CVE links (in supported terminals)

# Explain vulnerability details (v6.5.0+)
bazbom explain CVE-2024-1234             # Quick vulnerability lookup
bazbom explain CVE-2024-1234 --verbose  # Show full call chain
bazbom explain CVE-2024-1234 -v --findings=./findings.json  # Custom findings file

# Web dashboard
bazbom dashboard                        # Starts on http://localhost:3000

# Team assignment management
bazbom team assign                      # Assign vulnerabilities to team members
```

---

## Advanced Scanning

```bash
# Orchestrated security scan (SCA + SAST)
bazbom scan . --with-semgrep --with-codeql=security-extended --autofix=dry-run

# Container scanning
bazbom scan . --containers=auto         # Auto-detect strategy
bazbom scan . --containers=syft         # Use Syft

# Incremental analysis (for PRs)
bazbom scan . --incremental --base=main # Only scan changes since main branch

# Performance benchmarking
bazbom scan . --benchmark               # Show detailed performance metrics
```

---

## Output Formats

| Format | Flag | Use Case |
|--------|------|----------|
| SPDX 2.3 | `--format spdx` (default) | Industry standard, compliance |
| CycloneDX 1.5 | `--format cyclonedx` | OWASP ecosystem integration |
| SARIF 2.1.0 | Auto-generated | GitHub Code Scanning |
| CSV | Export via dashboard | Spreadsheet analysis |
| GraphML | Auto-generated | Dependency visualization (Gephi) |

---

## Short Flag Reference (v6.5.0+)

Save typing with convenient short flags:

| Long Flag | Short | Description |
|-----------|-------|-------------|
| `--reachability` | `-r` | Enable reachability analysis |
| `--format` | `-f` | Output format (spdx/cyclonedx) |
| `--out-dir` | `-o` | Output directory |
| `--with-semgrep` | `-s` | Run Semgrep analysis |
| `--with-codeql` | `-c` | Run CodeQL analysis |
| `--incremental` | `-i` | Incremental analysis mode |
| `--ml-risk` | `-m` | ML-enhanced risk scoring |
| `--base` | `-b` | Git base reference |
| `--profile` | `-p` | Use named profile |
| `--diff` | `-d` | Show diff vs baseline |

**Example**:
```bash
# Before
bazbom scan --reachability --with-semgrep --format spdx --out-dir ./output

# After
bazbom scan -r -s -f spdx -o ./output
```

---

## Build System Support

| Build System | Auto-Detection | Command |
|--------------|----------------|---------|
| Maven | `pom.xml` | `bazbom scan .` |
| Gradle | `build.gradle*` | `bazbom scan .` |
| Bazel | `BUILD*`, `MODULE.bazel` | `bazbom scan .` |
| Ant | `build.xml` | `bazbom scan .` |
| Sbt | `build.sbt` | `bazbom scan .` |
| Buildr | `Buildfile` | `bazbom scan .` |

---

## GitHub Actions Integration

```yaml
# .github/workflows/security.yml
- name: Run BazBOM
  uses: cboyd0319/BazBOM@main
  with:
    fail-on-critical: true
    upload-sbom: true
    upload-sarif: true
```

---

## Configuration Files

```bash
# Project-level config (bazbom.toml) with named profiles (v6.5.0+)
cat > bazbom.toml <<EOF
[scan]
format = "spdx"
reachability = true
fast = false

# Named profiles for different scenarios
[profile.strict]
reachability = true
with_semgrep = true
with_codeql = "security-extended"
ml_risk = true
fail_on = ["critical", "high"]

[profile.fast]
fast = true
incremental = true
no_upload = true

[profile.ci]
reachability = true
benchmark = true
format = "spdx"
cyclonedx = true

[policy]
severity_threshold = "HIGH"
fail_on_violations = true

[fix]
interactive = true
ml_prioritize = true
EOF

# Policy file (bazbom.yml)
cat > bazbom.yml <<EOF
version: 1.0
severity_gates:
  critical: block
  high: warn
kev_policy: block
epss_threshold: 0.1
EOF
```

---

## Common Workflows

### Daily Development
```bash
# Quick check before commit
bazbom scan . --fast
bazbom policy check
```

### CI/CD Pipeline
```bash
# Full security scan
bazbom scan . --with-semgrep --with-codeql=default
bazbom policy check --fail-on-violations
```

### Security Audit
```bash
# Comprehensive analysis
bazbom scan . --reachability --ml-risk
bazbom fix --suggest --ml-prioritize
bazbom license obligations
bazbom report generate --format html
```

### Large Monorepo PR
```bash
# Incremental scan (fast)
bazbom scan . --incremental --base=main --bazel-affected-by-files $(git diff --name-only main)
```

---

## Troubleshooting

```bash
# Verify installation
bazbom --version

# Test on sample project
bazbom scan examples/maven_spring_boot/

# Enable verbose logging
RUST_LOG=debug bazbom scan .

# Check for updates (manual build)
git -C /path/to/BazBOM pull
cargo build --release -p bazbom
sudo install -m 0755 /path/to/BazBOM/target/release/bazbom /usr/local/bin/bazbom
```

---

## Performance Tips

- **Fast mode**: Use `--fast` for quick feedback (<10s)
- **Incremental**: Use `--incremental` in CI/CD for PRs
- **Bazel monorepos**: Use `--bazel-targets-query` to scan specific targets
- **Remote cache**: Enable Bazel remote cache for faster builds
- **Parallel execution**: BazBOM automatically uses all CPU cores

---

## Getting Help

```bash
# Command help
bazbom --help
bazbom scan --help
bazbom policy --help

# Documentation
open https://github.com/cboyd0319/BazBOM/tree/main/docs

# Report issues
open https://github.com/cboyd0319/BazBOM/issues
```

---

## Useful Links

- **Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs
- **Examples**: https://github.com/cboyd0319/BazBOM/tree/main/examples
- **Releases**: https://github.com/cboyd0319/BazBOM/releases
- **Issues**: https://github.com/cboyd0319/BazBOM/issues
- **Discussions**: https://github.com/cboyd0319/BazBOM/discussions

---

**Quick Reference Version:** 1.0
**Last Updated:** 2025-11-11
**BazBOM Version:** 1.0.0

For complete documentation, see: [docs/README.md](docs/README.md)
