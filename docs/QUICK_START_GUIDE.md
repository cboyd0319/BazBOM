# BazBOM Quick Start Guide

**Get started with BazBOM in under 5 minutes**

---

## Installation

### Homebrew (macOS/Linux)
```bash
brew tap cboyd0319/bazbom
brew install bazbom
```

### Shell Script (All platforms)
```bash
curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
```

### From Source
```bash
git clone https://github.com/cboyd0319/BazBOM
cd BazBOM
cargo build --release
```

---

## Quick Commands

### 1. Interactive Setup (Recommended for First Time)

```bash
# Initialize BazBOM in your project
bazbom init
```

This will:
- ✅ Detect your build system (Maven, Gradle, Bazel, Ant, Buildr, sbt)
- ✅ Guide you through policy template selection
- ✅ Run your first security scan
- ✅ Show vulnerability summary

### 2. Scan Your Project

```bash
# Basic scan (current directory)
bazbom scan .

# Fast scan (skip reachability analysis)
bazbom scan . --fast

# With specific output format
bazbom scan . --format spdx --out-dir output/
```

### 3. View Results

**Interactive Dashboard:**
```bash
bazbom dashboard --open
```

**Terminal UI:**
```bash
bazbom explore
```

**Generate Reports:**
```bash
# Executive summary
bazbom report executive --output executive-report.html

# All reports
bazbom report all --output-dir reports/
```

### 4. Fix Vulnerabilities

```bash
# Show fix suggestions
bazbom fix --suggest

# Apply fixes automatically
bazbom fix --apply

# Create pull request with fixes
bazbom fix --pr
```

---

## Typical Workflows

### For Developers

**1. Check project security:**
```bash
bazbom scan .
bazbom dashboard --open
```

**2. Fix vulnerabilities:**
```bash
bazbom fix --suggest
bazbom fix --apply
```

**3. Install pre-commit hooks:**
```bash
bazbom install-hooks --fast
```

### For Security Teams

**1. Full security audit:**
```bash
bazbom scan . --reachability
bazbom report all --output-dir audit-reports/
```

**2. Compliance check:**
```bash
bazbom policy check
bazbom report compliance pci-dss --output pci-dss.html
```

**3. Monitor trends:**
```bash
bazbom dashboard
bazbom report trend --output trend.html
```

### For Executives

**1. Quick security overview:**
```bash
bazbom scan .
bazbom report executive --output executive-summary.html
open executive-summary.html
```

---

## CI/CD Integration

### GitHub Actions

```yaml
- name: BazBOM Security Scan
  uses: cboyd0319/BazBOM@v1
  with:
    scan-path: '.'
    generate-reports: 'true'
```

### GitLab CI

```yaml
bazbom-scan:
  script:
    - bazbom scan .
    - bazbom report all --output-dir reports/
  artifacts:
    paths:
      - reports/
```

---

## Key Features at a Glance

| Feature | Command | Output |
|---------|---------|--------|
| 🔍 **SBOM Generation** | `bazbom scan .` | SPDX 2.3, CycloneDX 1.5 |
| 🛡️ **Vulnerability Scan** | `bazbom scan .` | SARIF 2.1.0, JSON |
| 📊 **Interactive Dashboard** | `bazbom dashboard` | Web UI (D3.js graphs) |
| 📈 **Reports** | `bazbom report all` | HTML (executive, developer, compliance, trend) |
| 🎯 **Policy Enforcement** | `bazbom policy check` | YAML/Rego/OPA |
| 🔧 **Auto-Fix** | `bazbom fix --apply` | Automated remediation |
| 📜 **License Compliance** | `bazbom license obligations` | License report |
| 🌳 **Dependency Explorer** | `bazbom explore` | Terminal UI (TUI) |

---

## Build System Support

BazBOM automatically detects and supports:

- ✅ **Maven** - pom.xml
- ✅ **Gradle** - build.gradle, build.gradle.kts
- ✅ **Bazel** - BUILD.bazel, WORKSPACE, MODULE.bazel
- ✅ **Ant** - build.xml
- ✅ **Buildr** - buildfile, Rakefile
- ✅ **sbt** - build.sbt (Scala Build Tool)

---

## Policy Templates

Choose from 21 pre-configured templates:

**Regulatory:**
- PCI-DSS v4.0
- HIPAA Security Rule
- FedRAMP Moderate
- SOC 2 Type II
- GDPR
- ISO 27001
- NIST Cybersecurity Framework

**Industry:**
- Financial Services
- Healthcare Provider
- Government/Defense
- SaaS/Cloud Provider

**Framework:**
- Spring Boot Applications
- Android Applications
- Microservices Architecture
- Kubernetes Deployments

**Development Stages:**
- Development (Permissive)
- Staging (Moderate)
- Production (Strict)

---

## Getting Help

```bash
# General help
bazbom --help

# Command-specific help
bazbom scan --help
bazbom report --help
bazbom fix --help
```

**Documentation:**
- 📖 [Full Documentation](README.md)
- 📊 [Report Generation Guide](REPORT_GENERATION_GUIDE.md)
- 🎛️ [Dashboard Guide](DASHBOARD_GUIDE.md)
- 🔒 [Policy Guide](POLICY_GUIDE.md)

**Community:**
- 💬 [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)
- 🐛 [Issue Tracker](https://github.com/cboyd0319/BazBOM/issues)
- 🌟 [Star on GitHub](https://github.com/cboyd0319/BazBOM)

---

## Next Steps

1. ✅ Install BazBOM
2. ✅ Run `bazbom init` in your project
3. ✅ Explore results with `bazbom dashboard`
4. ✅ Fix vulnerabilities with `bazbom fix --apply`
5. ✅ Install pre-commit hooks: `bazbom install-hooks`
6. ✅ Generate reports: `bazbom report all`
7. ✅ Integrate with CI/CD

---

**Happy scanning! 🚀**
