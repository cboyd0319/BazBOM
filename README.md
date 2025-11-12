<div align="center">

<img src="docs/images/logo.svg" alt="BazBOM Logo" width="200">

# BazBOM

### Build-time SBOM & SCA for Bazel, JVM, and Polyglot Monorepos

Security for developers, not security engineers • 100% Rust • Zero telemetry • Actually works with Bazel

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions)
[![Tests](https://img.shields.io/badge/tests-67%20suites%20passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions/workflows/rust.yml)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![SLSA 3](https://img.shields.io/badge/SLSA-Level%203-green)](docs/operations/provenance.md)

[Install](#installation) •
[Quick Reference](docs/QUICKREF.md) •
[Documentation](docs/README.md) •
[Examples](docs/examples/README.md)

</div>

---

##  **v6.5.0 - 100% Feature Complete** ✅

> **Production-ready supply chain security with full reachability across 7 languages**
>
> - ✅ **Full reachability integration** - All 6 analyzers wired into scan workflow
> - ✅ **CLI feature complete** - `--json`, `--profile`, `--diff`, `explain` command
> - 🦀 **Rust** (>98%), 💎 **Ruby** (~75%), 🐘 **PHP** (~70%)
> - 🟨 **JavaScript/TypeScript** (~85%), 🐍 **Python** (~80%), 🐹 **Go** (~90%), ☕ **JVM** (~85%)
> - **67 test suites passing** • **26 crates** • **Zero warnings** • **Zero errors**
> - **18MB single binary** • **Homebrew available** • **GitHub Action ready**

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
- Rust (>98% accuracy), Ruby (~75%), PHP (~70%)
- Go (~90%), Python (~80%), JS/TS (~85%), JVM (~85%)

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

### Homebrew (macOS/Linux)
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

# Linux (x86_64)
curl -LO https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-unknown-linux-gnu.tar.gz
tar -xzf bazbom-x86_64-unknown-linux-gnu.tar.gz
sudo mv bazbom /usr/local/bin/
```

### From Source (Rust)
```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release -p bazbom
sudo cp target/release/bazbom /usr/local/bin/
```

See [Installation Guide](docs/getting-started/quickstart.md) for more options.

---

## Quick Start

### 1. Scan any project (auto-detects build system)
```bash
cd /path/to/your/project
bazbom scan .
```

Auto-detects: Maven, Gradle, Bazel, npm, Python, Go, Rust, Ruby, PHP

### 2. Add reachability analysis
```bash
bazbom scan -r  # -r = --reachability
```

Shows which vulnerabilities are **actually exploitable** vs dead code.

### 3. CI/CD Integration
```yaml
# .github/workflows/security.yml
- name: Security Scan
  run: |
    brew install bazbom
    bazbom scan . --json > findings.json
    bazbom scan . --format sarif > findings.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: findings.sarif
```

---

## Features

<table>
<tr><td width="50%" valign="top">

### **Core Capabilities**
- ✅ **SBOM Generation** (SPDX 2.3, CycloneDX 1.5)
- ✅ **Vulnerability Scanning** (OSV, NVD, CISA KEV, GHSA)
- ✅ **Reachability Analysis** (7 languages)
- ✅ **Build-Time Accuracy** (Maven/Gradle/Bazel native)
- ✅ **SLSA Level 3 Provenance**
- ✅ **VEX Support** (false positive suppression)
- ✅ **Policy Enforcement** (Rego-based)
- ✅ **Offline/Air-Gapped Mode**

### **Build System Support**
- **Bazel** (native aspects)
- **Maven** (bazbom-maven-plugin)
- **Gradle** (bazbom-gradle-plugin)
- **npm** (package.json, lockfiles)
- **Python** (requirements.txt, poetry, pipenv)
- **Go** (go.mod)
- **Rust** (Cargo.toml/lock)
- **Ruby** (Gemfile/Bundler)
- **PHP** (composer.json/lock)

</td><td width="50%" valign="top">

### **Developer Experience**
- ✅ **Interactive TUI** (explore dependencies, filter CVEs)
- ✅ **Web Dashboard** (visualize security posture)
- ✅ **JSON Output** (`--json` for CI/CD integration)
- ✅ **Named Profiles** (`--profile=prod` from bazbom.toml)
- ✅ **Diff Mode** (`--diff --baseline` for incremental scans)
- ✅ **Explain Command** (deep dive into specific CVEs)
- ✅ **Upgrade Intelligence** (see breaking changes BEFORE upgrading)
- ✅ **Auto-Fix** (PR generation with testing)
- ✅ **Pre-commit Hooks** (catch issues before commit)
- ✅ **IDE Integration** (IntelliJ, VS Code)
- ✅ **Clickable CVE Links** (open in browser from terminal)
- ✅ **Short Flags** (`-r`, `-f`, `-o`, `-s`, `-c`)

### **Advanced Features**
- **Container Scanning** (layer attribution)
- **ML Risk Scoring** (EPSS-based prioritization)
- **Team Assignment** (who owns which CVE)
- **Compliance Reports** (PCI-DSS, HIPAA, FedRAMP, SOC2)
- **GraphML/DOT Export** (visualize dependencies)
- **Diff Mode** (track security posture over time)
- **Named Profiles** (reusable configurations)

</td></tr>
</table>

---

## Documentation

### **Getting Started**
- [90-Second Quickstart](docs/getting-started/quickstart-90-seconds.md) - Fastest path to first scan
- [5-Minute Tutorial](docs/getting-started/quickstart.md) - Complete getting started guide
- [Homebrew Installation](docs/getting-started/homebrew-installation.md) - macOS/Linux installation
- [Shell Completions](docs/getting-started/shell-completions.md) - bash/zsh/fish completions

### **User Guides**
- [Usage Guide](docs/USAGE.md) - Common tasks and workflows
- [Bazel Integration](docs/BAZEL.md) - Bazel-specific features
- [CI/CD Integration](docs/CI.md) - GitHub Actions, GitLab, Jenkins
- [Quick Reference](docs/QUICKREF.md) - Command cheat sheet
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and fixes

### **Advanced Topics**
- [Architecture](docs/ARCHITECTURE.md) - System design and components
- [Reachability Analysis](docs/reachability/README.md) - How it works, accuracy by language
- [Container Scanning](docs/integrations/container-scanning.md) - Docker/OCI image analysis
- [Policy Integration](docs/user-guide/policy-integration.md) - Custom security policies
- [Performance Tuning](docs/operations/performance.md) - Scale to 5K+ targets

### **Reference**
- [Capabilities Matrix](docs/reference/capabilities-reference.md) - Complete feature list
- [JVM Support](docs/reference/jvm-language-support.md) - Java/Kotlin/Scala/Groovy
- [Polyglot Support](docs/polyglot/README.md) - Multi-language monorepos
- [SPDX Format](docs/FORMAT_SPDX.md) - SBOM specification details

[📚 Full Documentation Index](docs/README.md)

---

## Performance

**Scales to massive monorepos with incremental analysis:**

| Repo Size | Targets | Full Scan | Incremental (PR) |
|-----------|---------|-----------|------------------|
| Small | <50 | <2 min | <1 min |
| Medium | 50-500 | <5 min | <2 min |
| Large | 500-5K | <15 min | <5 min |
| Massive | 5K+ | <30 min | <10 min |

**6x faster** with incremental scanning. Tested on real-world enterprise monorepos.

---

## Comparison with Alternatives

| Feature | BazBOM | Syft | Trivy | Grype | OWASP DT |
|---------|--------|------|-------|-------|----------|
| **Bazel Support** | ✅ Native | ❌ | ❌ | ❌ | ❌ |
| **Build-Time Accuracy** | ✅ | ❌ | ❌ | ❌ | ✅ |
| **Reachability Analysis** | ✅ 7 langs | ❌ | ❌ | ❌ | ❌ |
| **SLSA Level 3** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Developer UX** | ✅ Plain English | ⚠️ Technical | ⚠️ Technical | ⚠️ Technical | ⚠️ Technical |
| **Monorepo Scale** | ✅ 5K+ targets | ⚠️ Slow | ⚠️ Slow | ⚠️ Slow | ⚠️ Limited |
| **Offline Mode** | ✅ | ✅ | ✅ | ✅ | ⚠️ Limited |

**Why BazBOM wins:**
- **Only tool** with native Bazel support (essential for modern monorepos)
- **Only tool** with polyglot reachability analysis (cuts noise 70-90%)
- **Only tool** with developer-friendly output (no security jargon)

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
