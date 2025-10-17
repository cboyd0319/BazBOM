<div align="center">

<img src="docs/images/logo.svg" alt="BazBOM Logo" width="200">

# BazBOM

### **Build-time SBOM generation and vulnerability scanning for Bazel projects**

Automatic dependency discovery • Zero configuration • Production-ready

[![Build](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![SLSA 3](https://img.shields.io/badge/SLSA-Level%203-green)](docs/PROVENANCE.md)
[![Bazel](https://img.shields.io/badge/Bazel-7.6.2-43A047?logo=bazel)](https://bazel.build)

[Quickstart](#-quickstart) •
[Features](#-features) •
[Documentation](docs/README.md) •
[Contributing](CONTRIBUTING.md)

</div>

---

## What is BazBOM?

BazBOM generates **Software Bills of Materials (SBOMs)** and performs **Software Composition Analysis (SCA)** for Java/JVM projects built with Bazel. It uses Bazel's build graph as the source of truth—no guessing, no manual maintenance.

**The problem:** Manual SBOM creation is error-prone. Post-build scanners miss transitive dependencies or include test artifacts.

**The solution:** BazBOM uses Bazel aspects to traverse your dependency graph automatically. Every build produces an accurate SBOM. Maven lockfiles provide exact versions and licenses.

### Who is this for?

- **Security teams** enforcing supply chain policies (SBOM + VEX + SLSA)
- **DevSecOps engineers** automating vulnerability scanning in CI/CD
- **Organizations** with large Bazel+Java monorepos (5000+ targets)

---

## ⚡ Quickstart

### 1. Add BazBOM to your WORKSPACE

```python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazbom",
    urls = ["https://github.com/cboyd0319/BazBOM/archive/v1.0.0.tar.gz"],
    sha256 = "...",  # Get from releases page
    strip_prefix = "BazBOM-1.0.0",
)
```

### 2. Generate your first SBOM

```bash
bazel build //app:app_sbom
cat bazel-bin/app/app_sbom.spdx.json
```

**Output:** Valid SPDX 2.3 JSON with all dependencies, licenses, and hashes.

### 3. Run vulnerability scan

```bash
bazel run //:sca_scan
```

**Output:**
- `sca_findings.json` - Machine-readable findings (OSV + NVD)
- `sca_findings.sarif` - GitHub Code Scanning format

That's it. No configuration files, no manual dependency lists.

📖 **New to BazBOM?** Follow the [5-minute tutorial](docs/QUICKSTART.md)

---

## ✨ Features

<table>
<tr>
<td width="50%">

**SBOM Generation**
- ✅ SPDX 2.3 (JSON) primary format
- ✅ CycloneDX 1.5 (optional)
- ✅ Per-target or workspace-wide
- ✅ Automatic version/license extraction

**Vulnerability Scanning**
- ✅ OSV (Open Source Vulnerabilities)
- ✅ NVD (National Vulnerability Database)
- ✅ **CISA KEV** (actively exploited CVEs)
- ✅ **EPSS** (ML-based exploit probability)
- ✅ GitHub Security Advisories (GHSA)
- ✅ **Risk scoring & priority mapping (P0-P4)**
- ✅ Offline mode (air-gapped environments)

**GitHub Integration**
- ✅ SARIF 2.1.0 output
- ✅ Code Scanning alerts
- ✅ PR comments with findings
- ✅ Policy enforcement (block on critical CVEs)

</td>
<td width="50%">

**Supply Chain Security**
- ✅ SLSA Level 3 provenance
- ✅ Sigstore keyless signing
- ✅ VEX (false positive suppression)
- ✅ License compliance checking
- ✅ Typosquatting detection
- ✅ Outdated dependency detection

**Dependency Analysis**
- ✅ Full transitive graph (JSON + GraphML)
- ✅ Reverse dependency lookups
- ✅ Conflict detection
- ✅ Visualize with Gephi/yEd

**Performance**
- ✅ Incremental analysis (5-10x faster PRs)
- ✅ Remote caching support
- ✅ Parallel processing
- ✅ Scales to 5000+ target monorepos

</td>
</tr>
</table>

---

## 🎯 Core Workflows

### Workflow 1: Daily Development

```bash
# Generate SBOM for what you're working on
bazel build //my-service:sbom

# Check for vulnerabilities
bazel run //my-service:sca_check

# View dependency graph
bazel run //my-service:dep_graph
```

### Workflow 2: CI/CD Pipeline

```yaml
- name: Supply Chain Analysis
  run: |
    bazel build //:sbom_all
    bazel run //:sca_scan

- name: Upload to GitHub Security
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: bazel-bin/sca_findings.sarif
```

See [CI/CD examples](.github/workflows/) for complete workflows.

### Workflow 3: Compliance & Audit

```bash
# Generate compliance bundle
bazel build //:compliance_bundle

# Outputs:
# - All SBOMs (SPDX + CycloneDX)
# - SLSA provenance (signed)
# - License report
# - Dependency graph
# - VEX statements
```

---

## 📊 How It Works

```mermaid
graph LR
    A[Bazel Build] -->|Aspect| B[Dependency Discovery]
    B --> C[maven_install.json]
    C --> D[SBOM Generator]
    D --> E[SPDX 2.3 JSON]
    E --> F[Vulnerability Scanner]
    F --> G[OSV Database]
    G --> H[SARIF Output]
    H --> I[GitHub Security]

    style A fill:#43A047
    style E fill:#1976D2
    style H fill:#F57C00
    style I fill:#E91E63
```

1. **Build** - Bazel aspects traverse the dependency graph
2. **Extract** - Parse `maven_install.json` for versions/licenses
3. **Generate** - Create SPDX 2.3 compliant SBOM
4. **Scan** - Query OSV/NVD for known vulnerabilities
5. **Report** - Output SARIF for GitHub Code Scanning

No external tools. No network access during build (hermetic). Fully reproducible.

---

## 🚀 Installation

### Prerequisites

<table>
  <thead>
    <tr>
      <th>Tool</th>
      <th>Version</th>
      <th>Purpose</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Bazel</strong></td>
      <td>≥ 6.0</td>
      <td>Build system</td>
    </tr>
    <tr>
      <td><strong>Java</strong></td>
      <td>≥ 11</td>
      <td>JVM runtime</td>
    </tr>
    <tr>
      <td><strong>Python</strong></td>
      <td>≥ 3.9</td>
      <td>SBOM generation scripts</td>
    </tr>
    <tr>
      <td><strong>Git</strong></td>
      <td>≥ 2.30</td>
      <td>Incremental analysis (optional)</td>
    </tr>
  </tbody>
</table>

### Option 1: Quick Install (Recommended)

```bash
# In your Bazel workspace root
curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
```

### Option 2: Manual Installation

Add to `WORKSPACE`:

```python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazbom",
    urls = ["https://github.com/cboyd0319/BazBOM/archive/v1.0.0.tar.gz"],
    sha256 = "...",
    strip_prefix = "BazBOM-1.0.0",
)

load("@bazbom//:deps.bzl", "bazbom_dependencies")
bazbom_dependencies()
```

Add to root `BUILD.bazel`:

```python
load("@bazbom//:defs.bzl", "sbom_all", "sca_scan")

sbom_all(name = "sbom_all")
sca_scan(name = "sca_scan")
```

### Verify Installation

```bash
bazel build //:sbom_all
# Should complete without errors and produce SBOMs in bazel-bin/
```

---

## 📖 Usage Examples

### Generate SBOM for Single Target

```bash
bazel build //services/api:api_sbom
```

**Output:** `bazel-bin/services/api/api_sbom.spdx.json`

### Generate SBOMs for Entire Workspace

```bash
bazel build //:sbom_all
```

**Output:** One SBOM per `java_binary` and `java_library`

### Include Test Dependencies

```bash
bazel build //:sbom_all --define=include_test_deps=true
```

**Use case:** Comprehensive security audit including test frameworks

### Generate CycloneDX Format

```bash
bazel build //:sbom_all --define=cyclonedx=true
```

**Output:** Both SPDX and CycloneDX files

### Run Vulnerability Scan

```bash
bazel run //:sca_scan
```

**Output:**
- `bazel-bin/sca_findings.json`
- `bazel-bin/sca_findings.sarif`

### Scan with Offline CVE Database

```bash
bazel run //:sca_scan -- --offline-mode --osv-db-path=/opt/osv-db
```

**Use case:** Air-gapped environments

### Generate Dependency Graph

```bash
bazel build //:dep_graph_all
```

**Output:**
- `bazel-bin/dep_graph.json` - Query with jq
- `bazel-bin/dep_graph.graphml` - Visualize with Gephi

### Apply VEX Statements (Suppress False Positives)

```bash
bazel run //:apply_vex -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

See [VEX Guide](docs/VEX.md) for creating VEX statements.

---

## ⚙️ Configuration

BazBOM works **zero-config** for most projects. Advanced options:

<table>
  <thead>
    <tr>
      <th>Flag</th>
      <th>Default</th>
      <th>Purpose</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>--define=include_test_deps=true</code></td>
      <td><code>false</code></td>
      <td>Include test scope dependencies</td>
    </tr>
    <tr>
      <td><code>--define=cyclonedx=true</code></td>
      <td><code>false</code></td>
      <td>Generate CycloneDX + SPDX</td>
    </tr>
    <tr>
      <td><code>--define=max_depth=N</code></td>
      <td>unlimited</td>
      <td>Limit transitive depth</td>
    </tr>
    <tr>
      <td><code>--define=offline_mode=true</code></td>
      <td><code>false</code></td>
      <td>Use local CVE database</td>
    </tr>
  </tbody>
</table>

**.bazelrc example:**

```bash
# Add to your .bazelrc
build:sbom --aspects=@bazbom//tools:aspects.bzl%sbom_aspect
build:sbom --output_groups=+sbom

# Use with: bazel build --config=sbom //...
```

See [Usage Guide](docs/USAGE.md) for full reference.

---

## ⚡ Performance

Expected times with **remote cache enabled**:

<table>
  <thead>
    <tr>
      <th>Repo Size</th>
      <th>Targets</th>
      <th>Dependencies</th>
      <th>Full Analysis</th>
      <th>Incremental (PR)</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><strong>Small</strong></td>
      <td>&lt; 50</td>
      <td>&lt; 100</td>
      <td>&lt; 2 min</td>
      <td>&lt; 1 min</td>
    </tr>
    <tr>
      <td><strong>Medium</strong></td>
      <td>50-500</td>
      <td>100-500</td>
      <td>&lt; 5 min</td>
      <td>&lt; 2 min</td>
    </tr>
    <tr>
      <td><strong>Large</strong></td>
      <td>500-5K</td>
      <td>500-2K</td>
      <td>&lt; 15 min</td>
      <td>&lt; 5 min</td>
    </tr>
    <tr>
      <td><strong>Massive</strong></td>
      <td>5K+</td>
      <td>2K+</td>
      <td>&lt; 30 min</td>
      <td>&lt; 10 min</td>
    </tr>
  </tbody>
</table>

**Optimization tips:**
- Enable Bazel remote cache (`--remote_cache`)
- Use incremental mode in PRs (only changed targets)
- Parallel execution (`--jobs=auto`)

See [Performance Guide](docs/PERFORMANCE.md) for tuning details.

---

## 🔒 Security

### Threat Model

BazBOM operates with **least privilege**:
- **Read-only** access to source code and dependencies
- **No secrets required** (OSV API is public, GHSA via GitHub token if available)
- **Hermetic builds** (no network during SBOM generation)
- **Signed releases** (Sigstore keyless signing)

See [Threat Model](docs/THREAT_MODEL.md) for complete analysis.

### SLSA Compliance

BazBOM targets **SLSA Level 3**:
- ✅ Provenance generated for all builds
- ✅ Provenance signed with Sigstore
- ✅ GitHub-hosted runners (hardened platform)
- ✅ Build logs retained (90 days)

See [Provenance Guide](docs/PROVENANCE.md) for verification steps.

### Reporting Vulnerabilities

Report security issues via [SECURITY.md](SECURITY.md). We respond within 48 hours.

---

## 🔧 Troubleshooting

### Common Issues

<details>
<summary><strong>Error: "No such package: @maven"</strong></summary>

**Cause:** `rules_jvm_external` not configured

**Fix:** Add to WORKSPACE:

```python
load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    artifacts = ["com.google.guava:guava:31.1-jre"],
    repositories = ["https://repo1.maven.org/maven2"],
    maven_install_json = "//:maven_install.json",
)
```

</details>

<details>
<summary><strong>SBOM missing dependencies</strong></summary>

**Cause:** Aspect not applied to all targets

**Fix:** Clear cache and rebuild:

```bash
bazel clean
bazel build //:sbom_all
```

</details>

<details>
<summary><strong>Slow analysis on large repo</strong></summary>

**Cause:** Full workspace analysis on every build

**Fix:** Use incremental mode:

```bash
# Analyze only changed targets
bazel run //tools/supplychain:incremental_analyzer
```

See [Performance Guide](docs/PERFORMANCE.md) for more optimizations.

</details>

**More help:** [Troubleshooting Guide](docs/TROUBLESHOOTING.md) • [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)

---

## 🗺️ Roadmap

**In Progress:**
- [ ] Gradle support (in addition to Maven)
- [ ] Container image SBOM (`rules_oci` integration)

**Planned:**
- [ ] Kotlin Multiplatform support
- [ ] Dependency conflict auto-resolution
- [ ] Visual dependency graph UI (web-based)
- [ ] NPM/Node.js support

**Completed:**
- [x] SPDX 2.3 SBOM generation
- [x] OSV vulnerability scanning
- [x] SLSA Level 3 provenance
- [x] VEX statement support
- [x] Large monorepo optimization

Vote on features: [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions/categories/feature-requests)

---

## 📚 Documentation

### Getting Started
- **[Quickstart](docs/QUICKSTART.md)** - 5-minute setup
- **[Usage Guide](docs/USAGE.md)** - All commands and workflows
- **[Installation](docs/QUICKSTART.md#installation)** - Detailed setup

### Architecture & Design
- **[Architecture](docs/ARCHITECTURE.md)** - System design and data flow
- **[Supply Chain](docs/SUPPLY_CHAIN.md)** - SBOM/SCA implementation
- **[Threat Model](docs/THREAT_MODEL.md)** - Security analysis
- **[ADRs](docs/ADR/)** - Architecture Decision Records

### Advanced Features
- **[Performance](docs/PERFORMANCE.md)** - Large monorepo optimization
- **[Provenance](docs/PROVENANCE.md)** - SLSA Level 3 attestation
- **[VEX](docs/VEX.md)** - False positive management
- **[Dependency Graphs](docs/GRAPH_ANALYSIS.md)** - Visualization and queries

### Operations
- **[Validation](docs/VALIDATION.md)** - SBOM/SARIF schema validation
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Common issues and fixes

### Full Documentation Index
See [docs/README.md](docs/README.md) for complete documentation map.

---

## 🤝 Contributing

Contributions are welcome! BazBOM is open-source and community-driven.

**Before you start:**
1. Read [CONTRIBUTING.md](CONTRIBUTING.md) for setup instructions
2. Check [existing issues](https://github.com/cboyd0319/BazBOM/issues) for duplicates
3. Discuss major changes in [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions) first

**Quick links:**
- [Code of Conduct](CODE_OF_CONDUCT.md)
- [Development Setup](CONTRIBUTING.md#development-setup)
- [Running Tests](CONTRIBUTING.md#testing)
- [Maintainers](MAINTAINERS.md)

**Good first issues:** Look for [`good-first-issue`](https://github.com/cboyd0319/BazBOM/labels/good-first-issue) label.

---

## 📄 License

**MIT License** - See [LICENSE](LICENSE) for full text.

```
✅ Commercial use allowed
✅ Modification allowed
✅ Distribution allowed
✅ Private use allowed
📋 License and copyright notice required
```

**TL;DR:** Use it however you want. Just include the license.

Learn more: https://choosealicense.com/licenses/mit/

---

## 💬 Support & Community

**Need help?**
- 🐛 [File a bug report](https://github.com/cboyd0319/BazBOM/issues/new?template=bug_report.md)
- 💡 [Request a feature](https://github.com/cboyd0319/BazBOM/discussions/new?category=feature-requests)
- 💬 [Ask a question](https://github.com/cboyd0319/BazBOM/discussions/new?category=q-a)
- 🔒 [Report a security issue](SECURITY.md) (private)

**Resources:**
- [Maintainers](MAINTAINERS.md) - Who maintains BazBOM
- [Changelog](CHANGELOG.md) - Release history
- [Bazel Slack](https://slack.bazel.build) - `#bazbom` channel (coming soon)

---

<div align="center">

## ⭐ Spread the Word

If BazBOM helps secure your supply chain, **give us a star** ⭐

[![Star History](https://img.shields.io/github/stars/cboyd0319/BazBOM?style=social)](https://github.com/cboyd0319/BazBOM/stargazers)

**Active Development** • **Production-Ready** • **Community-Driven**

Made with ❤️ for the Bazel ecosystem

[⬆ Back to top](#bazbom)

</div>
