# macOS Quick Start Guide

**Get BazBOM running on your Mac in 3 minutes**

This guide is designed for macOS users who want to quickly install and test BazBOM on internal company repositories.

---

## Prerequisites

- **macOS 10.15+** (Catalina or later) - Works great on macOS 26.1
- **Terminal access** (Applications → Utilities → Terminal)
- **Admin rights** (for installation to `/usr/local/bin`)

**Optional but recommended:**
- **Java 11+** - Required if scanning Java/JVM projects (check with `java -version`)
- **Git** - Usually pre-installed (check with `git --version`)

---

## 🚀 Installation (2 minutes)

### Option 1: One-Line Install (Recommended)

Open Terminal and run:

```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

**What this does:**
- Downloads the latest BazBOM binary for your Mac (Intel or Apple Silicon)
- Installs to `/usr/local/bin/bazbom` (may prompt for your password)
- Verifies the installation works
- Shows you next steps

**Expected output:**
```
╔════════════════════════════════════════════════╗
║          BazBOM Installer v1.0                 ║
║  Polyglot reachability-first SBOM & SCA        ║
╚════════════════════════════════════════════════╝

ℹ Detected platform: darwin (arm64) - Target: aarch64-apple-darwin
ℹ Fetching latest version...
✓ Latest version: 6.5.0
ℹ Downloading BazBOM v6.5.0 for aarch64-apple-darwin...
✓ Downloaded successfully
ℹ Installing to /usr/local/bin/bazbom...
Password: [enter your password]
✓ BazBOM installed successfully!
```

### Option 2: Manual Install (If Script Fails)

1. **Download the binary for your Mac:**
   - [Apple Silicon (M1/M2/M3)](https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-aarch64-apple-darwin.tar.gz)
   - [Intel Mac](https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-x86_64-apple-darwin.tar.gz)

2. **Extract and install:**
   ```bash
   cd ~/Downloads
   tar -xzf bazbom-*.tar.gz
   sudo mv bazbom /usr/local/bin/
   sudo chmod +x /usr/local/bin/bazbom
   ```

3. **Remove macOS quarantine (important!):**
   ```bash
   sudo xattr -d com.apple.quarantine /usr/local/bin/bazbom
   ```

---

## ✅ Verify Installation

Run these commands to confirm BazBOM is working:

```bash
# Check version
bazbom --version

# View help
bazbom --help

# Quick system check
bazbom doctor  # (if available)
```

**Expected output:**
```
bazbom 6.5.0
```

---

## 🎯 Quick Test Scan

### Test on a Sample Java Project

```bash
# Navigate to any Java/Maven project
cd /path/to/your/java-project

# Quick scan (fast, basic results)
bazbom check

# Full scan with reachability analysis (70-90% noise reduction)
bazbom scan --reachability

# Auto-fix vulnerabilities
bazbom fix --suggest
```

### Test on This Repository

```bash
# Clone BazBOM repo as a test
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM

# Scan it (it's a Rust project)
bazbom check
```

---

## 🏢 Scanning Your Company Repos

### Step 1: Navigate to Your Project

```bash
cd /path/to/company/project
```

### Step 2: Run BazBOM

For **quick results** (< 10 seconds):
```bash
bazbom check
```

For **full analysis** with reachability (recommended):
```bash
bazbom scan --reachability --output-format spdx --output report.json
```

For **container images**:
```bash
bazbom scan-container your-image:latest
```

### Step 3: Review Results

BazBOM outputs:
- **Priority ratings** (P0 = critical, P1 = high, etc.)
- **Plain English explanations** ("Hackers are actively exploiting this")
- **Fix suggestions** (version upgrades, patches)
- **Reachability status** (Is this CVE actually reachable in your code?)

---

## 🔧 Common Issues & Fixes

| Issue | Solution |
|-------|----------|
| `bazbom: command not found` | Run: `export PATH="/usr/local/bin:$PATH"` then add to `~/.zshrc` |
| "Cannot verify developer" warning | Run: `sudo xattr -d com.apple.quarantine /usr/local/bin/bazbom` |
| No Java installed | Install Java 11+: `brew install openjdk@21` (or download from Oracle/Adoptium) |
| Permission denied | Use `sudo` for installation, or install to `~/.local/bin` instead |
| Slow scans on large repos | Use `--fast` flag or `bazbom check` for quick results |

---

## 📊 Understanding Output

BazBOM uses **plain English** prioritization:

| Priority | What It Means | Example |
|----------|---------------|---------|
| **P0** | CRITICAL - Fix immediately | Remote code execution, actively exploited |
| **P1** | HIGH - Fix this week | SQL injection, high CVSS, proof-of-concept exists |
| **P2** | MEDIUM - Fix this month | Moderate CVSS, no known exploits |
| **P3** | LOW - Fix when convenient | Low severity, unlikely to be exploited |
| **P4** | INFO - Awareness only | Deprecated dependency, no security impact |

**Reachability Analysis:**
- ✅ **REACHABLE** - Your code calls this vulnerable function → **Fix immediately**
- ⚠️ **POTENTIALLY_REACHABLE** - Might be called through reflection/dynamic code → **Investigate**
- ❌ **UNREACHABLE** - You don't use this vulnerable code → **Low priority**

---

## 💡 Pro Tips for Company Repos

### 1. **Run in CI/CD**

Add to your pipeline:
```bash
bazbom ci --fail-on P0,P1 --reachability
```

This fails the build if critical (P0) or high (P1) vulnerabilities are found.

### 2. **Generate Reports**

```bash
bazbom scan --output-format spdx --output sbom.json
bazbom scan --output-format sarif --output security.sarif
```

Share these with security/compliance teams.

### 3. **Check Dependencies Before Merging**

```bash
bazbom scan --pr  # Optimized for PR checks
```

### 4. **Batch Scans Across Multiple Repos**

```bash
#!/bin/bash
for repo in /path/to/repos/*; do
  echo "Scanning $repo..."
  cd "$repo"
  bazbom check --output "$repo/bazbom-report.json"
done
```

---

## 🆘 Need Help?

1. **View built-in help:** `bazbom --help`
2. **Check specific command:** `bazbom scan --help`
3. **Read full docs:** [BazBOM Documentation](https://github.com/cboyd0319/BazBOM/tree/main/docs)
4. **Report issues:** [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)

---

## 🚀 Next Steps

After testing on a few repos:

1. **Set up CI integration** - [CI Guide](../CI.md)
2. **Configure policies** - [Policy Guide](../user-guide/policy-integration.md)
3. **Explore reachability** - [Reachability Guide](../reachability/README.md)
4. **Container scanning** - [Container Guide](../features/container-scanning.md)

---

## Quick Reference Commands

```bash
# Installation
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh

# Basic scan
bazbom check

# Full scan with reachability
bazbom scan --reachability

# Auto-fix suggestions
bazbom fix --suggest

# CI mode
bazbom ci --fail-on P0,P1

# Container scan
bazbom scan-container image:tag

# Update advisory database
bazbom db sync

# View version
bazbom --version

# View help
bazbom --help
```

---

**Ready to scan? Run `bazbom check` in any project directory!**
