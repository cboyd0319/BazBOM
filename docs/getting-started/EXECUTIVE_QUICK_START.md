# BazBOM Executive Quick Start

**Security scanning that focuses on what matters - for busy teams**

This guide is for executives, managers, and non-technical users who need to quickly understand and deploy BazBOM across their organization.

---

## What is BazBOM?

BazBOM is a **security scanner** that tells you which vulnerabilities in your code actually matter.

**The Problem:** Traditional security scanners report hundreds of alerts. Most are false positives.

**BazBOM's Solution:** Advanced "reachability analysis" cuts noise by 70-90%. Only see vulnerabilities that are actually exploitable in your code.

**Example:**
- Traditional scanner: **237 vulnerabilities found**
- BazBOM: **28 vulnerabilities that actually matter** (the other 209 are in unused code paths)

---

## Why BazBOM?

| Feature | Benefit |
|---------|---------|
| **70-90% Noise Reduction** | Security teams focus on real threats, not false alarms |
| **Plain English Reports** | "Hackers are using this now" vs "EPSS threshold exceeded" |
| **Zero Configuration** | Works out-of-the-box with Java, JavaScript, Python, Go, Rust, Ruby, PHP |
| **Fast Scans** | Results in <10 seconds for quick checks |
| **Auto-Fix** | One command to upgrade vulnerable dependencies |
| **100% Private** | No data leaves your systems, no telemetry, works offline |

---

## Installation (For Your Team)

### Option 1: One-Line Install (Easiest)

Share this with your development team:

```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

**That's it!** This installs BazBOM on macOS or Linux in under 2 minutes.

### Option 2: System Health Check First

If you want to verify systems are ready:

```bash
# Check if system is ready for BazBOM
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/scripts/check-system.sh | sh

# Then install
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

### Platform Support

| Platform | Support | Installation Time |
|----------|---------|-------------------|
| macOS (Intel/Apple Silicon) | ✅ Full | 2 minutes |
| Linux (x86_64/ARM64) | ✅ Full | 2 minutes |
| Windows | ✅ Experimental | 5 minutes (manual) |

---

## Using BazBOM (For Your Team)

### Basic Workflow

1. **Navigate to your project:**
   ```bash
   cd /path/to/your/project
   ```

2. **Run a scan:**
   ```bash
   bazbom check
   ```

3. **Review results:**
   - BazBOM shows priority ratings (P0 = critical, P4 = low)
   - Plain English explanations
   - Fix suggestions

4. **Auto-fix vulnerabilities:**
   ```bash
   bazbom fix --suggest
   ```

### Common Commands

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `bazbom check` | Quick scan (10 sec) | Daily development, PR checks |
| `bazbom scan --reachability` | Full analysis | Weekly security reviews |
| `bazbom fix --suggest` | Get fix recommendations | After finding vulnerabilities |
| `bazbom ci` | CI/CD integration | Automated pipelines |
| `bazbom scan-container IMAGE` | Scan Docker images | Before deploying containers |

---

## Integration Options

### 1. Developer Workstations

Each developer installs BazBOM locally and runs before committing code:

```bash
bazbom check  # Quick scan before commit
```

### 2. CI/CD Pipelines

Add to your build pipeline to block vulnerable code:

```bash
bazbom ci --fail-on P0,P1 --reachability
```

**Blocks the build if:** Critical (P0) or High (P1) vulnerabilities are found.

### 3. Scheduled Scans

Set up weekly security scans across all repositories:

```bash
#!/bin/bash
for repo in /path/to/repos/*; do
  cd "$repo"
  bazbom scan --reachability --output "reports/$(basename $repo).json"
done
```

### 4. Container Image Scanning

Scan Docker images before deployment:

```bash
bazbom scan-container your-app:latest
```

---

## Understanding Output

### Priority Levels

| Level | Meaning | Action |
|-------|---------|--------|
| **P0** | CRITICAL - Active exploitation | Fix immediately (same day) |
| **P1** | HIGH - Proof-of-concept exists | Fix this week |
| **P2** | MEDIUM - Moderate risk | Fix this month |
| **P3** | LOW - Unlikely exploitation | Fix when convenient |
| **P4** | INFO - No security impact | Awareness only |

### Reachability Status

| Status | What It Means | Priority |
|--------|---------------|----------|
| ✅ **REACHABLE** | Your code uses this vulnerable function | **Fix immediately** |
| ⚠️ **POTENTIALLY_REACHABLE** | Might be used (reflection/dynamic code) | **Investigate** |
| ❌ **UNREACHABLE** | You don't call this code | **Low priority** |

**Example Report:**
```
P0 [REACHABLE] CVE-2024-1234: Remote Code Execution in log4j
  Severity: CRITICAL (CVSS 9.8)
  Status: Hackers are actively exploiting this vulnerability
  Fix: Upgrade log4j from 2.14.0 to 2.17.1
  Your code calls: org.apache.logging.log4j.Logger.error()
```

---

## Cost & Licensing

- **License:** MIT (Free for commercial use)
- **Cost:** $0 (Open source)
- **Support:** Community support via GitHub Issues
- **Enterprise:** Contact for custom support/training

---

## Deployment Timeline

### Day 1: Pilot (2 hours)
1. Install BazBOM on 2-3 developer machines
2. Run scans on 3-5 key repositories
3. Review results with security team

### Week 1: Team Rollout (1 day)
1. Share installation instructions with all developers
2. Add BazBOM to CI/CD pipelines
3. Set up automated weekly scans

### Month 1: Full Integration (ongoing)
1. Monitor scan results
2. Track vulnerability remediation
3. Integrate into security workflows

---

## Success Metrics

Track these KPIs to measure BazBOM's impact:

| Metric | Before BazBOM | After BazBOM (Expected) |
|--------|---------------|-------------------------|
| Security alerts per scan | 200-500 | 20-50 (70-90% reduction) |
| Time to triage alerts | 4-8 hours | 30-60 minutes |
| False positive rate | 60-80% | 5-15% |
| Developer scan frequency | Weekly | Daily (lower friction) |
| Time to remediation | 2-4 weeks | 1-2 weeks |

---

## Security & Compliance

### Data Privacy
- ✅ **Zero telemetry** - No data leaves your systems
- ✅ **Offline-first** - Works without internet (after initial advisory DB sync)
- ✅ **No accounts** - No sign-up or authentication required
- ✅ **No cloud** - Runs entirely on your infrastructure

### Standards Compliance
- ✅ **SBOM:** SPDX 2.3, CycloneDX 1.5
- ✅ **Vulnerability Reports:** SARIF 2.1.0, CSV, JSON
- ✅ **VEX:** CSAF VEX for vulnerability status
- ✅ **Provenance:** SLSA Level 3 signed releases

### Supported Ecosystems

| Ecosystem | Languages | Build Tools |
|-----------|-----------|-------------|
| **JVM** | Java, Kotlin, Scala, Groovy, Clojure | Maven, Gradle, Bazel, sbt, Ant |
| **JavaScript** | JavaScript, TypeScript | npm, yarn, pnpm |
| **Python** | Python | pip, poetry, pipenv |
| **Other** | Go, Rust, Ruby, PHP | Native package managers |

---

## Common Questions

### Q: Does this replace our existing security tools?
**A:** BazBOM complements existing tools by providing:
- Better vulnerability prioritization (reachability analysis)
- Faster scans for development workflows
- Developer-friendly output (plain English, not CVE jargon)

Use alongside: SAST tools, penetration testing, security audits.

### Q: How much does this cost?
**A:** $0. BazBOM is open source (MIT license) and free for commercial use.

### Q: What about enterprise support?
**A:** Community support is available via GitHub Issues. For SLA-backed support, training, or custom features, contact the maintainers.

### Q: Is our code sent to the cloud?
**A:** No. BazBOM runs entirely on your infrastructure. Zero telemetry, 100% private.

### Q: How often should we scan?
**A:** Recommended:
- **Developers:** Daily (`bazbom check` before commits)
- **CI/CD:** Every build (`bazbom ci`)
- **Security Team:** Weekly full scans (`bazbom scan --reachability`)

### Q: What if we find vulnerabilities?
**A:** BazBOM provides:
1. Priority rating (P0-P4)
2. Plain English explanation
3. Fix suggestions (version upgrades)
4. Auto-fix command: `bazbom fix --suggest`

### Q: Can this scan containers?
**A:** Yes! Use `bazbom scan-container IMAGE` to scan Docker images with full reachability analysis.

---

## Getting Help

### For Installation Issues
1. **macOS:** See [macOS Quick Start](MACOS_QUICK_START.md)
2. **System Check:** Run `curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/scripts/check-system.sh | sh`
3. **Manual Install:** See [Installation Guide](homebrew-installation.md)

### For Usage Questions
1. **Built-in help:** `bazbom --help` or `bazbom COMMAND --help`
2. **Documentation:** [Full Documentation](../README.md)
3. **Examples:** [Usage Examples](../user-guide/usage.md)

### For Issues & Support
1. **GitHub Issues:** [Report a bug](https://github.com/cboyd0319/BazBOM/issues)
2. **Community:** GitHub Discussions (coming soon)
3. **Enterprise:** Contact maintainers for commercial support

---

## Next Steps

### For Executives
1. ✅ Read this guide (you're done!)
2. 📊 Review [success metrics](#success-metrics) with your security team
3. 🚀 Start a pilot with 2-3 developers
4. 📈 Track KPIs monthly

### For Managers
1. ✅ Share installation link with team: https://github.com/cboyd0319/BazBOM
2. 📝 Add BazBOM to team's security workflow
3. 🎯 Set expectations: `bazbom check` before commits
4. 📅 Schedule weekly security review meetings

### For Developers
1. ✅ Install BazBOM: `curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh`
2. 🔍 Run first scan: `bazbom check`
3. 📖 Read [macOS Quick Start](MACOS_QUICK_START.md) or [User Guide](../user-guide/usage.md)
4. 🔧 Integrate into workflow

---

## Quick Reference Card

**Installation:**
```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

**Essential Commands:**
```bash
bazbom check                          # Quick scan (10 sec)
bazbom scan --reachability            # Full analysis
bazbom fix --suggest                  # Get fix recommendations
bazbom ci --fail-on P0,P1            # CI/CD integration
bazbom scan-container IMAGE           # Scan Docker image
bazbom --help                         # Show all commands
```

**Key Benefits:**
- 70-90% noise reduction via reachability analysis
- Plain English reports, not CVE jargon
- Zero config, works out-of-the-box
- 100% private, no telemetry
- Free & open source (MIT license)

---

**Ready to get started? Install now:** `curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh`
