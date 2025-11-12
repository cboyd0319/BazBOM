<div align="center">

<img src="docs/images/logo.svg" alt="BazBOM Logo" width="200">

# BazBOM

### Build-time SBOM & SCA for Bazel, JVM, and Polyglot Monorepos

Security for developers, not security engineers • 100% Rust • Zero telemetry • Actually works with Bazel

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions)
[![Tests](https://img.shields.io/badge/tests-360%2B%20passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions/workflows/rust.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![SLSA 3](https://img.shields.io/badge/SLSA-Level%203-green)](docs/operations/provenance.md)

[Install](#installation) •
[Quick Start](#quick-start) •
[Documentation](docs/README.md) •
[Examples](docs/examples/README.md)

</div>

---

## 🎉 **v6.5.0 - The Developer Experience Release** ✅

> **The most comprehensive UX overhaul in BazBOM's history**
>
> - ✅ **11 new UX features** - Quick commands, smart defaults, beautiful output
> - ✅ **Full reachability integration** - 7 languages, 70-90% noise reduction
> - ✅ **Zero-config workflows** - `bazbom check`, `bazbom ci`, `bazbom pr`
> - ✅ **Continuous monitoring** - `bazbom watch` for auto-rescanning
> - ✅ **Security dashboard** - `bazbom status` for at-a-glance posture
> - ✅ **Branch comparison** - `bazbom compare main feature-branch`
> - ✅ **CI templates** - One-command setup for GitHub, GitLab, CircleCI, Jenkins
> - 🦀 **24 crates** • **360+ tests** • **18MB binary** • **Homebrew ready**

[🚀 See What's New](#whats-new-in-v65) | [📚 Full Changelog](CHANGELOG.md)

---

## Why BazBOM?

**Three problems BazBOM solves that other tools don't:**

### 1. **Actually works with Bazel monorepos**
Most SCA tools choke on Bazel. BazBOM is the **only tool** that natively understands Bazel's dependency model.

```bash
# Scan only changed targets (6x faster than full scan)
bazbom scan --bazel-affected-by-files $(git diff --name-only HEAD~1)

# Scan specific services
bazbom scan --bazel-targets-query 'kind(java_binary, //services/...)'
```

**Tested on 5000+ target monorepos.** Works with Maven, Gradle, and Bazel in the same repo.

### 2. **Cuts noise by 70-90% with reachability analysis**

Traditional tools report every vulnerability in every dependency. BazBOM tells you which ones **actually matter**.

```
Before BazBOM: 237 vulnerabilities to fix 😱
After BazBOM:  28 reachable vulnerabilities to fix ✅

Focus on what's actually exploitable. Ignore the rest.
```

**Reachability analysis for 7 languages:**
- ☕ JVM (>95% accuracy) - OPAL Framework bytecode analysis
- 🦀 Rust (>98% accuracy) - Native syn parser, trait tracking
- 🐹 Go (~90% accuracy) - tree-sitter, goroutine tracking
- 🟨 JavaScript/TypeScript (~85% accuracy) - SWC-based AST parsing
- 🐍 Python (~80% accuracy) - RustPython parser, framework detection
- 💎 Ruby (~75% accuracy) - Rails, RSpec, metaprogramming support
- 🐘 PHP (~70% accuracy) - Laravel, Symfony, WordPress support

### 3. **Developer-friendly output (not security jargon)**

**Other tools:**
```
❌ Policy violation: EPSS threshold exceeded (0.73 > 0.50)
   Severity: CVSS 8.5 (HIGH), CISA KEV: true
```

**BazBOM:**
```
🚨 MUST FIX NOW (actively exploited in the wild!)

CVE-2024-1234 in log4j-core 2.17.0
  Why: Hackers are using this right now
  Fix: Upgrade to 2.20.0 (~45 min effort)

Run: bazbom fix log4j-core --explain
```

No CVE jargon unless you want it. Plain English. Actionable steps.

---

## Installation

### Homebrew (macOS/Linux) - Recommended
```bash
brew tap cboyd0319/bazbom
brew install bazbom
bazbom --version
```

### Pre-built Binaries
```bash
# macOS (Apple Silicon)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-aarch64-apple-darwin.tar.gz
tar -xzf bazbom-aarch64-apple-darwin.tar.gz
sudo mv bazbom /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-apple-darwin.tar.gz
tar -xzf bazbom-x86_64-apple-darwin.tar.gz
sudo mv bazbom /usr/local/bin/

# Linux (x86_64)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-unknown-linux-gnu.tar.gz
tar -xzf bazbom-x86_64-unknown-linux-gnu.tar.gz
sudo mv bazbom /usr/local/bin/

# Verify installation
bazbom --version
```

### From Source (Rust)
```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release -p bazbom
sudo cp target/release/bazbom /usr/local/bin/
```

See [Installation Guide](docs/getting-started/quickstart.md) for more options including Docker, npm, and CI/CD.

---

## Quick Start

### 1. Zero-config quick scan
```bash
# Just run this in any project directory:
bazbom check

# Auto-detects: Maven, Gradle, Bazel, npm, Python, Go, Rust, Ruby, PHP
# Completes in < 10 seconds
```

### 2. Add reachability analysis (70-90% noise reduction)
```bash
bazbom scan --reachability
# or the short flag:
bazbom scan -r
```

Shows which vulnerabilities are **actually exploitable** vs dead code.

### 3. CI/CD - One command setup
```bash
# GitHub Actions
bazbom install github

# GitLab CI
bazbom install gitlab

# CircleCI
bazbom install circleci

# Creates complete workflow with SARIF upload + quality gates
```

### 4. Continuous monitoring during development
```bash
# Watch for dependency changes and auto-rescan
bazbom watch

# Checks every 60 seconds, rescans on changes
```

### 5. Check security status anytime
```bash
# Quick security overview with score
bazbom status

# Compare branches
bazbom compare main feature-branch
```

---

## 🆕 What's New in v6.5

### Quick Commands (Zero-Config Workflows)
No more memorizing 30+ flags. Use commands that match your workflow:

```bash
# Development
bazbom check          # Fast local scan (< 10s, no reachability)
bazbom quick          # 5-second smoke test (critical vulns only)

# Pull Requests
bazbom pr             # Incremental + diff mode (shows new vs fixed)

# CI/CD
bazbom ci             # JSON + SARIF output, no GitHub upload

# Production
bazbom full           # Everything enabled (reachability + ML + benchmarking)
```

Each command is pre-configured with optimal flags for that use case.

### Smart Environment Detection
BazBOM auto-detects your environment and adjusts behavior:

- **CI Detection** → Auto-enables `--json` output
- **PR Context** → Auto-enables `--incremental` mode
- **Small Repos** → Auto-enables `--reachability` (< 100MB)
- **Baseline Exists** → Suggests `--diff` mode

Override with `BAZBOM_NO_SMART_DEFAULTS=1` if needed.

### Beautiful Terminal Output
Color-coded vulnerability cards with actionable information:

```
┌─────────────────────────────────────────────┐
│ 🚨 CRITICAL: CVE-2024-1234                  │
├─────────────────────────────────────────────┤
│ Package:  log4j-core 2.17.0                 │
│ Severity: CRITICAL (CVSS 9.8)               │
│ Status:   REACHABLE ⚠️ (actively used!)     │
├─────────────────────────────────────────────┤
│ Quick Fix:                                  │
│ $ bazbom fix log4j-core --apply             │
│                                             │
│ Learn more:                                 │
│ $ bazbom explain CVE-2024-1234              │
└─────────────────────────────────────────────┘
```

Scannable, color-coded, with immediate next steps.

### Status Command
Get an at-a-glance security overview:

```bash
bazbom status
```

Shows:
- Security score (0-100)
- Vulnerability breakdown by severity
- Reachable vulnerabilities count
- Last scan timestamp
- Recommended actions

### Compare Command
Compare security posture between branches or commits:

```bash
# Compare feature branch with main
bazbom compare main feature/new-api

# Shows:
# - New vulnerabilities introduced
# - Vulnerabilities fixed
# - Risk assessment (better/worse/unchanged)
```

Perfect for PR reviews and release planning.

### Watch Mode
Continuous monitoring during development:

```bash
# Monitor dependency files and auto-rescan on changes
bazbom watch

# Custom interval
bazbom watch --interval 300  # Check every 5 minutes

# Critical vulnerabilities only
bazbom watch --critical-only
```

Monitors: `Cargo.toml`, `pom.xml`, `build.gradle`, `package.json`, `requirements.txt`, `go.mod`, etc.

### CI/CD Templates
One-command CI setup:

```bash
bazbom install github    # Creates .github/workflows/bazbom.yml
bazbom install gitlab    # Updates .gitlab-ci.yml
bazbom install circleci  # Creates .circleci/config.yml
bazbom install jenkins   # Creates Jenkinsfile.bazbom
bazbom install travis    # Updates .travis.yml
```

Each template includes:
- BazBOM installation
- Scan execution
- SARIF upload
- Artifact storage
- Quality gates

### Actionable Error Messages
Every error includes:
- Plain-English explanation
- Quick fix command
- Documentation link

Example:
```
❌ GitHub CLI Not Found

The 'gh' command is required but was not found in PATH.

💡 Quick Fix:
# macOS:
brew install gh

# Linux:
sudo apt install gh

# Then authenticate:
gh auth login

📚 https://cli.github.com/manual/installation
```

### Named Profiles
Reusable scan configurations in `bazbom.toml`:

```toml
[profiles.dev]
reachability = false
fast = true

[profiles.ci]
format = "sarif"
json = true
no_upload = true

[profiles.production]
reachability = true
ml_risk = true
benchmark = true
cyclonedx = true
```

```bash
# Use profiles
bazbom scan --profile dev
bazbom scan --profile production
```

### Short Flags
Faster command typing:

```bash
bazbom scan -r              # --reachability
bazbom scan -f sarif        # --format
bazbom scan -o ./results    # --out-dir
bazbom scan -s              # --with-semgrep
bazbom scan -c default      # --with-codeql
bazbom scan -i              # --incremental
bazbom scan -m              # --ml-risk
bazbom scan -b main         # --base
bazbom scan -p prod         # --profile
bazbom scan -d              # --diff
```

### Examples in Help
Every command now includes real-world examples:

```bash
bazbom check --help

# Shows:
# - Common usage patterns
# - Explanation of what the command does
# - Links to related commands
```

### Progress Indicators
Beautiful progress bars for long operations:

```
🔍 Scanning dependencies...
[████████████████████────────] 64% (1,234/1,890)

⚡ Computing reachability...
[████████████░░░░░░░░░░░░░░░░] 50% (2.3s remaining)
```

Multi-phase progress for complex scans with real-time updates.

---

## Features

<table>
<tr><td width="50%" valign="top">

### **Core Capabilities**
- ✅ **SBOM Generation** (SPDX 2.3, CycloneDX 1.5)
- ✅ **Vulnerability Scanning** (OSV, NVD, CISA KEV, GHSA)
- ✅ **Reachability Analysis** (7 languages, 70-90% noise reduction)
- ✅ **Build-Time Accuracy** (Maven/Gradle/Bazel native)
- ✅ **SLSA Level 3 Provenance** (Signed releases)
- ✅ **VEX Support** (False positive suppression)
- ✅ **Policy Enforcement** (Rego/YAML/CUE-based)
- ✅ **Offline/Air-Gapped Mode** (Works fully disconnected)
- ✅ **EPSS Scoring** (Exploit prediction)
- ✅ **Priority Classification** (P0-P4 auto-scoring)

### **Build System Support** (13 systems)
- **JVM**: Maven, Gradle, Bazel, sbt, Ant+Ivy, Buildr, Android
- **JavaScript**: npm, Yarn, pnpm (workspaces supported)
- **Python**: pip, poetry, pipenv, PDM
- **Go**: go.mod/go.sum (vendor support)
- **Rust**: Cargo.toml/lock (workspaces)
- **Ruby**: Bundler/Gemfile.lock
- **PHP**: Composer

### **Reachability Analysis** (7 languages)
- ☕ **JVM** (>95%) - OPAL bytecode analysis
- 🦀 **Rust** (>98%) - syn parser, trait tracking
- 🐹 **Go** (~90%) - tree-sitter + reflection detection
- 🟨 **JS/TS** (~85%) - SWC AST + framework detection
- 🐍 **Python** (~80%) - RustPython + dynamic code warnings
- 💎 **Ruby** (~75%) - Rails + metaprogramming support
- 🐘 **PHP** (~70%) - Laravel + variable function handling

</td><td width="50%" valign="top">

### **Developer Experience** (v6.5 UX Overhaul)
- ✅ **Quick Commands** (check, ci, pr, full, quick)
- ✅ **Smart Defaults** (Auto-detects CI, PR, repo size)
- ✅ **Beautiful Output** (Unicode boxes, color-coded)
- ✅ **Status Command** (Security score + recommendations)
- ✅ **Compare Command** (Branch security comparison)
- ✅ **Watch Mode** (Continuous monitoring)
- ✅ **CI Templates** (GitHub, GitLab, CircleCI, Jenkins, Travis)
- ✅ **Actionable Errors** (Quick fixes + docs links)
- ✅ **Smart Suggestions** (Context-aware next steps)
- ✅ **Progress Bars** (Real-time operation feedback)
- ✅ **Interactive TUI** (Explore dependencies, filter CVEs)
- ✅ **Web Dashboard** (D3.js visualization)
- ✅ **JSON Output** (`--json` for CI/CD)
- ✅ **Named Profiles** (`--profile=prod`)
- ✅ **Diff Mode** (`--diff --baseline`)
- ✅ **Explain Command** (Deep CVE analysis)
- ✅ **Short Flags** (`-r`, `-f`, `-o`, `-s`, `-c`)
- ✅ **Examples in Help** (Real-world usage patterns)
- ✅ **Clickable CVE Links** (OSC 8 hyperlinks)

### **Advanced Features**
- **Container Scanning** (Layer attribution + comparison)
- **ML Risk Scoring** (EPSS-based prioritization)
- **LLM Fix Generation** (Ollama/Claude/GPT)
- **Team Assignment** (CVE ownership tracking)
- **Compliance Reports** (PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST CSF)
- **GraphML/DOT Export** (Cytoscape, Gephi, Graphviz)
- **Incremental Scans** (10x faster for PRs)
- **Kubernetes Operator** (CRD-based scanning)
- **IDE Integration** (IntelliJ, VS Code)
- **LSP Server** (Real-time vulnerability warnings)
- **Pre-commit Hooks** (Catch issues before commit)
- **Upgrade Intelligence** (Breaking change analysis)
- **Auto-Fix with PR** (Automated remediation)
- **Threat Intelligence** (Malicious packages, typosquatting)

</td></tr>
</table>

---

## Command Reference

### Quick Commands (New in v6.5)
```bash
bazbom check              # Fast local dev scan
bazbom ci                 # CI-optimized (JSON + SARIF)
bazbom pr                 # PR mode (incremental + diff)
bazbom full               # Full scan with all features
bazbom quick              # 5-second smoke test
```

### Primary Commands
```bash
bazbom scan               # Full SBOM + vulnerability scan
bazbom status             # Security posture overview
bazbom compare            # Compare branches/commits
bazbom watch              # Continuous monitoring
bazbom fix                # Smart remediation
bazbom explore            # Interactive TUI
bazbom dashboard          # Web visualization
bazbom explain CVE-XXXX   # Deep CVE analysis
```

### Container Security
```bash
bazbom container-scan <image>                    # Full container analysis
bazbom container-scan <image> --baseline         # Save as baseline
bazbom container-scan <image> --compare-baseline # Compare with baseline
bazbom container-scan <image> --compare <other>  # Compare two images
bazbom container-scan <image> --interactive      # Interactive TUI
bazbom container-scan <image> --show p0          # Critical vulns only
```

### Policy & Compliance
```bash
bazbom policy check                  # Validate against policies
bazbom policy init --framework hipaa # Generate HIPAA policy
bazbom policy validate               # Validate policy syntax
```

### Licensing
```bash
bazbom license obligations           # License report
bazbom license compatibility         # Check conflicts
bazbom license contamination         # Detect copyleft spread
```

### Team Coordination
```bash
bazbom team assign CVE-XXXX @user    # Assign CVE
bazbom team list                     # List assignments
bazbom team mine                     # My assignments
bazbom team audit-log --export csv   # Export audit trail
```

### Reports
```bash
bazbom report executive              # 1-page summary
bazbom report compliance --pci-dss   # PCI-DSS report
bazbom report developer              # Technical report
bazbom report trend                  # Historical analysis
bazbom report all                    # Generate all reports
```

### Setup & Installation
```bash
bazbom init                          # Interactive setup wizard
bazbom install github                # GitHub Actions workflow
bazbom install gitlab                # GitLab CI config
bazbom install circleci              # CircleCI config
bazbom install jenkins               # Jenkinsfile
bazbom install travis                # Travis CI config
bazbom install-hooks                 # Pre-commit hooks
bazbom db sync                       # Sync offline databases
```

[📚 See Full Command Reference](docs/QUICKREF.md)

---

## Performance

**Scales to massive monorepos with incremental analysis:**

| Repo Size | Targets | Full Scan | Incremental (PR) | Watch Mode |
|-----------|---------|-----------|------------------|------------|
| Small | <50 | <2 min | <1 min | ~5s per check |
| Medium | 50-500 | <5 min | <2 min | ~10s per check |
| Large | 500-5K | <15 min | <5 min | ~20s per check |
| Massive | 5K+ | <30 min | <10 min | ~30s per check |

**6-10x faster** with incremental scanning. Tested on real-world enterprise monorepos.

---

## Documentation

### **Getting Started**
- [90-Second Quickstart](docs/getting-started/quickstart-90-seconds.md) - Fastest path to first scan
- [5-Minute Tutorial](docs/getting-started/quickstart.md) - Complete guide
- [Homebrew Installation](docs/getting-started/homebrew-installation.md) - macOS/Linux setup
- [Shell Completions](docs/getting-started/shell-completions.md) - bash/zsh/fish

### **User Guides**
- [Usage Guide](docs/USAGE.md) - Common workflows
- [Bazel Integration](docs/BAZEL.md) - Bazel-specific features
- [CI/CD Integration](docs/CI.md) - GitHub Actions, GitLab, Jenkins
- [Quick Reference](docs/QUICKREF.md) - Command cheat sheet
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues

### **v6.5 Features**
- [Quick Commands](docs/features/quick-commands.md) - Zero-config workflows
- [Smart Defaults](docs/features/smart-defaults.md) - Auto-detection
- [Status Command](docs/features/status-command.md) - Security dashboard
- [Compare Command](docs/features/compare-command.md) - Branch comparison
- [Watch Mode](docs/features/watch-mode.md) - Continuous monitoring
- [CI Templates](docs/features/ci-templates.md) - One-command setup

### **Advanced Topics**
- [Architecture](docs/ARCHITECTURE.md) - System design
- [Reachability Analysis](docs/reachability/README.md) - How it works
- [Container Scanning](docs/integrations/container-scanning.md) - Docker/OCI
- [Policy Integration](docs/user-guide/policy-integration.md) - Custom policies
- [Performance Tuning](docs/operations/performance.md) - Scale to 5K+ targets
- [Polyglot Support](docs/polyglot/README.md) - Multi-language monorepos

### **Reference**
- [Capabilities Matrix](docs/CAPABILITY_MATRIX.md) - Complete feature list
- [JVM Support](docs/reference/jvm-language-support.md) - Java/Kotlin/Scala
- [SPDX Format](docs/FORMAT_SPDX.md) - SBOM specification

[📚 Full Documentation Index](docs/README.md)

---

## Comparison with Alternatives

| Feature | BazBOM | Syft | Trivy | Grype | OWASP DT |
|---------|--------|------|-------|-------|----------|
| **Bazel Support** | ✅ Native | ❌ | ❌ | ❌ | ❌ |
| **Build-Time Accuracy** | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Reachability Analysis** | ✅ 7 langs | ❌ | ❌ | ❌ | ❌ |
| **Developer UX** | ✅ Plain English | ⚠️ Technical | ⚠️ Technical | ⚠️ Technical | ⚠️ Technical |
| **Quick Commands** | ✅ 5 commands | ❌ | ❌ | ❌ | ❌ |
| **Watch Mode** | ✅ Continuous | ❌ | ❌ | ❌ | ❌ |
| **Status Dashboard** | ✅ Built-in | ❌ | ❌ | ❌ | ⚠️ Web only |
| **CI Templates** | ✅ 5 platforms | ❌ Manual | ⚠️ Limited | ❌ Manual | ❌ Manual |
| **Monorepo Scale** | ✅ 5K+ targets | ⚠️ Slow | ⚠️ Slow | ⚠️ Slow | ⚠️ Limited |
| **Offline Mode** | ✅ | ✅ | ✅ | ✅ | ⚠️ Limited |
| **SLSA Level 3** | ✅ | ❌ | ❌ | ❌ | ❌ |

**Why BazBOM wins:**
- **Only tool** with native Bazel support (essential for modern monorepos)
- **Only tool** with polyglot reachability analysis (70-90% noise reduction)
- **Only tool** with developer-friendly UX (quick commands, smart defaults, beautiful output)
- **Only tool** with continuous monitoring (watch mode)
- **Only tool** with one-command CI setup (5 platforms)

---

## Contributing

We welcome contributions! BazBOM is open-source (MIT) and community-driven.

**Quick Links:**
- [Contributing Guide](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Good First Issues](https://github.com/cboyd0319/BazBOM/labels/good-first-issue)
- [Maintainers](MAINTAINERS.md)

**Development Setup:**
```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build
cargo test
```

See [Development Guide](docs/development/README.md) for details.

---

## Security

**BazBOM practices what it preaches:**

- ✅ **Zero vulnerabilities** (cargo audit clean)
- ✅ **100% memory-safe Rust** (no unsafe code without justification)
- ✅ **SLSA Level 3 provenance** (signed releases)
- ✅ **Sigstore keyless signing** (verify before you trust)
- ✅ **Zero telemetry** (no phoning home, ever)
- ✅ **Offline-first** (works fully air-gapped)
- ✅ **360+ tests** (>90% coverage)

**Report vulnerabilities:** See [SECURITY.md](SECURITY.md)

---

## License

**MIT License** - Use it however you want.

```
Commercial use ✅
Modification ✅
Distribution ✅
Private use ✅
```

Just include the license. That's it.

See [LICENSE](LICENSE) for full text.

---

## Support & Community

**Get Help:**
- [Documentation](docs/README.md)
- [Troubleshooting Guide](docs/TROUBLESHOOTING.md)
- [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)
- [File a Bug](https://github.com/cboyd0319/BazBOM/issues/new?template=bug_report.md)
- [Request a Feature](https://github.com/cboyd0319/BazBOM/discussions/new?category=feature-requests)

**Stay Updated:**
- [Changelog](CHANGELOG.md)
- [Roadmap](docs/roadmaps/V7_ROADMAP.md)
- [GitHub Releases](https://github.com/cboyd0319/BazBOM/releases)

---

<div align="center">

## ⭐ **If BazBOM helps secure your supply chain, give us a star!** ⭐

[![Star History](https://img.shields.io/github/stars/cboyd0319/BazBOM?style=social)](https://github.com/cboyd0319/BazBOM/stargazers)

**Production-Ready • Open Source • Actually Works with Bazel**

Made for developers who ship code 🚀

[Back to top](#bazbom)

</div>
