# macOS Quick Start Guide

**Get BazBOM running on your Mac in 5-10 minutes**

This guide is designed for macOS users who want to quickly build and test BazBOM on internal company repositories.

---

## Prerequisites

Before starting, ensure you have:

- **macOS 10.15+** (Catalina or later)
- **Terminal access** (Applications → Utilities → Terminal)
- **Rust toolchain** (we'll install this below)
- **Admin rights** (for installation to `/usr/local/bin`)

**Optional but recommended:**
- **Java 11+** - Required if scanning Java/JVM projects (check with `java -version`)
- **Git** - Usually pre-installed (check with `git --version`)

---

## 🚀 Installation (5-10 minutes)

### Step 1: Install Rust (if not already installed)

BazBOM is written in Rust, so you need the Rust toolchain. This takes about 2 minutes:

```bash
# Install Rust using rustup (the official installer)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Follow the prompts (just press Enter to accept defaults)
# Then restart your terminal or run:
source ~/.cargo/env

# Verify Rust is installed:
rustc --version
cargo --version
```

**Expected output:**
```
rustc 1.75.0 (or newer)
cargo 1.75.0 (or newer)
```

### Step 2: Build BazBOM from Source

Now build BazBOM. This takes about 3-5 minutes on the first build:

```bash
# Clone the repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM

# Build the release binary (optimized, smaller, faster)
cargo build --release -p bazbom

# Install to /usr/local/bin (may prompt for password)
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

**What to expect:**
- First build takes 3-5 minutes (Rust compiles everything)
- You'll see compilation progress for ~29 crates
- Final binary will be optimized and ready to use
- Subsequent builds are much faster (< 1 minute)

---

## ✅ Verify Installation

Run these commands to confirm BazBOM is working:

```bash
# Check version
bazbom --version

# View help
bazbom --help
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

### Test on the BazBOM Repository Itself

```bash
# Already in the BazBOM directory from installation
cd ~/BazBOM  # or wherever you cloned it

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
| `cargo: command not found` | Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| No Java installed | Install Java 11+: `brew install openjdk@21` (or download from Oracle/Adoptium) |
| Build fails with linker error | Install Xcode Command Line Tools: `xcode-select --install` |
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
# Build from source (first time)
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom

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

## 🔄 Updating BazBOM

To update to the latest version:

```bash
cd ~/BazBOM  # or wherever you cloned it
git pull origin main
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
bazbom --version
```

---

**Ready to scan? Run `bazbom check` in any project directory!**
