# Container Security Scanning

BazBOM's container scanning is one of the most sophisticated container security tools available, combining SBOM generation, vulnerability detection, layer attribution, and intelligent analysis.

## Multi-Tool Orchestration

BazBOM runs **6 security tools in parallel** for comprehensive coverage:

| Tool | Purpose | Output |
|------|---------|--------|
| **Trivy** | Vulnerability scanning | CVEs, severity, fixes |
| **Grype** | Vulnerability cross-validation | Additional CVE coverage |
| **Syft** | SBOM generation | Package inventory |
| **Dockle** | CIS benchmark checks | Dockerfile best practices |
| **Dive** | Layer efficiency analysis | Image optimization |
| **TruffleHog** | Secrets scanning | Exposed credentials |

All tools run concurrently for fast results, with automatic image pre-fetching to avoid network timeouts.

## What Makes BazBOM Container Scanning Different

### 1. Layer Attribution (Unique!)
Unlike traditional scanners that just list vulnerabilities, BazBOM **maps each vulnerability to the exact Docker layer that introduced it**. This helps you understand:
- Which base image layers have vulnerabilities
- Which application layers introduced security issues
- Whether vulnerabilities come from OS packages or application dependencies

### 2. Intelligent Prioritization
Every vulnerability gets a **P0-P4 priority score** based on:
- **CVSS severity** - Industry-standard risk scoring
- **EPSS score** - Exploit Prediction Scoring System (likelihood of exploitation)
- **CISA KEV** - Known Exploited Vulnerabilities catalog
- **Fix availability** - Whether a patch exists
- **Breaking changes** - Whether the fix requires major refactoring

### 3. Actionable Intelligence
BazBOM doesn't just tell you what's wrong - it tells you **what to do about it**:
- **Top 5 Fixes by Impact** - Aggregated view showing highest-impact fixes sorted by severity weight (Critical=10, High=5, Medium=2, Low=1)
- **Quick Wins** - Easy fixes with high impact (non-breaking patches)
- **Action Plan** - Prioritized roadmap with time estimates
- **Multi-Language Copy-Paste Fixes** - Ready-to-use dependency updates for:
  - ☕ Java (Maven/Gradle)
  - 🐍 Python (pip/Poetry/Pipfile)
  - 📦 JavaScript (npm/yarn/package.json)
  - 🐹 Go (go.mod/go get)
  - 🦀 Rust (Cargo.toml/cargo add)
  - 💎 Ruby (Gemfile/bundle)
  - 🐘 PHP (composer.json/composer require)
- **Framework-Specific Migration Guides** - Actionable upgrade paths for Spring Boot, Django, Rails, React, Vue, Angular, Express, and more
- **Multi-CVE Grouping** - Consolidates related vulnerabilities ("Fixes 3 CVEs: CVE-2024-1234, CVE-2024-5678, CVE-2024-9012" instead of 3 separate actions)
- **Remediation Difficulty Scoring** - 0-100 difficulty score for each fix:
  - Algorithm factors: breaking changes (+40), version jumps (+15 each), framework migrations (+25), no fix available (100)
  - Visual indicators: 🟢 Trivial (0-20) → 🔴 Hard (61-80) → 🚫 No Fix (100)
  - Helps estimate remediation effort and prioritize work
- **Effort Analysis** - Estimated time to remediate each vulnerability

### 4. Full Call Graph Reachability Analysis
Reduce noise by 70-90% using **AST-based static analysis** to determine which vulnerabilities are actually **reachable** in your container's code:
- 🎯 **REACHABLE** - Vulnerable code is in execution paths (prioritize these!)
- 🛡️ **unreachable** - Vulnerable dependencies not used (lower priority)
- **7 languages with full call graph analysis**: JavaScript/TypeScript, Python, Go, Rust, Ruby, PHP, Java
- Uses language-specific AST parsers (SWC, tree-sitter, syn, RustPython)
- Analyzes actual execution paths from entrypoints, not heuristics
- **Always enabled** - runs automatically on every scan

**Call Path Visualization**: For reachable vulnerabilities, BazBOM shows the exact call chain from your code to the vulnerable function:

```
Call Path: main() → UserController.login() → Logger.debug() → log4j.error()
```

This tells you exactly HOW your code reaches the vulnerable function, making remediation decisions clearer - you can see if it's in a critical request handler or a rarely-used admin endpoint.

### 5. Supply Chain Threat Intelligence
Detect supply chain attacks with real-time threat analysis:
- **Malicious package detection** - Known bad packages
- **Typosquatting detection** - Lookalike package names
- **Dependency confusion** - Private/public namespace attacks
- **Compromised maintainer detection** - Account takeover indicators

### 6. Compliance Reports
Automatically generates compliance reports for major frameworks:
- **PCI-DSS v4.0** - Payment card industry requirements
- **HIPAA** - Healthcare data protection
- **SOC 2 Type II** - Service organization controls

Reports map scan findings to specific compliance requirements and controls. The HTML report shows **actual failure reasons** based on your scan results:
- Critical/high vulnerability counts
- CISA KEV (Known Exploited Vulnerabilities) presence
- Other security policy violations

Each compliance framework shows Pass/Fail status with specific issues that need remediation.

### 7. PDF Executive Reports
Generate professional PDF reports for stakeholders:
- Executive summary with security score
- Vulnerability breakdown by severity
- SBOM statistics
- Policy compliance status

Perfect for board presentations and audit documentation.

### 8. Interactive HTML Reports
Developer reports include **client-side JavaScript** for enhanced usability:
- **Filtering** - Filter by severity (Critical/High/Medium/Low) or fixability
- **Search** - Instant search for CVEs, packages, or descriptions (press `/` to focus)
- **Expand/Collapse** - Toggle vulnerability details for cleaner initial view
- **Keyboard shortcuts** - `/` to search, `Escape` to clear

Reports are standalone HTML files with no external dependencies.

### 9. Scan Warnings Summary
At the end of each scan, BazBOM displays a summary of any warnings or issues encountered:
- Image pull failures
- Reachability analysis errors
- Filesystem extraction issues
- Tool availability problems

This ensures you're aware of any incomplete analysis while still getting partial results.

### 10. Container Signature & Provenance Verification
BazBOM verifies container supply chain security:
- **Cosign signature verification** - Validates image signatures using Sigstore
- **SLSA provenance verification** - Checks SLSA (Supply-chain Levels for Software Artifacts) provenance attestations

Verification results are shown in scan output and reports. Unsigned or unverified images are flagged but don't block the scan.

### 11. Native OS Package Scanning
BazBOM includes native scanners for OS packages as a fallback/supplement to Trivy and Grype:
- **Alpine** - Uses Alpine secdb for vulnerability data
- **Debian/Ubuntu** - Uses security-tracker.debian.org
- **RHEL/CentOS/Fedora** - Uses Red Hat Security Data API

These native scanners provide additional vulnerability coverage, especially for packages that external tools might miss. All native scan results are enriched with EPSS/KEV data.

### 12. OS Upgrade Intelligence
For vulnerable OS packages, BazBOM provides intelligent upgrade recommendations:
- Queries deps.dev for available versions
- Identifies which upgrades fix which CVEs
- Calculates risk level (Low/Medium/High based on version jumps)
- Groups multiple CVE fixes into single upgrade recommendations

```
📦 Upgrade Recommendations:
  1. Update libexpat: 2.6.2-r0 → 2.6.3-r0
     Fixes: CVE-2024-45491, CVE-2024-45492, CVE-2024-45490
     Risk: LOW (patch update)
```

## Quick Start

### Zero-Config Full Scan
```bash
bazbom container-scan nginx:latest
```

That's it! With zero flags, you get **ALL capabilities enabled**:
- Reachability analysis
- Compliance reports (PCI-DSS, HIPAA, SOC2)
- PDF executive report
- Jira ticket templates for Critical/High vulns
- Upgrade intelligence with breaking change detection

Output goes to `~/Documents/container-scans/nginx_latest/`

### Scan Presets

For different use cases:
```bash
# Fast CI check (no reachability)
bazbom container-scan nginx:latest --preset quick

# Standard scan without reachability
bazbom container-scan nginx:latest --preset standard

# Full analysis (default behavior)
bazbom container-scan nginx:latest --preset full

# Focus on compliance reports
bazbom container-scan nginx:latest --preset compliance
```

### What Gets Generated

Every scan produces:
- `scan-results.json` - Full enriched results
- `report.html` - Interactive HTML report with collapsible sections
- `report.pdf` - Executive PDF report
- `compliance/` - PCI-DSS, HIPAA, SOC2 reports
- `jira-tickets/` - Copy-paste markdown files for Critical/High vulns
- SBOM and tool-specific outputs

### Jira Ticket Templates

For every Critical and High vulnerability, BazBOM generates ready-to-use Jira tickets:
- `CVE-XXXX-TRIAGE.md` - For security team triage
- `CVE-XXXX-REMEDIATION.md` - For engineering remediation

Each ticket includes severity, EPSS score, KEV status, fix version, and reachability status.

### Output includes:
- Total packages and vulnerabilities
- Layer-by-layer breakdown
- Severity distribution
- Top vulnerabilities per layer
- Security score (0-100)
- Upgrade recommendations with breaking change analysis

### Save as Baseline
```bash
bazbom container-scan myapp:v1.0 --baseline
```

Creates a baseline for future comparisons. Stored in `.bazbom/baselines/`.

### Compare with Baseline
```bash
# After updating your image
bazbom container-scan myapp:v1.1 --compare-baseline
```

Shows:
- New vulnerabilities introduced
- Vulnerabilities fixed
- Net change in security posture

### Compare Two Images
```bash
bazbom container-scan ubuntu:20.04 --compare ubuntu:22.04
```

Side-by-side comparison to help choose the more secure base image.

## Advanced Features

### Smart Filtering

Show only critical vulnerabilities:
```bash
bazbom container-scan myapp:latest --show critical
```

Show only P0 (urgent) vulnerabilities:
```bash
bazbom container-scan myapp:latest --show p0
```

Show only vulnerabilities with fixes available:
```bash
bazbom container-scan myapp:latest --show fixable
```

Show only "quick wins" (fixable + non-breaking):
```bash
bazbom container-scan myapp:latest --show quick-wins
```

Show only CISA Known Exploited Vulnerabilities:
```bash
bazbom container-scan myapp:latest --show kev
```

Available filters:
- `p0`, `p1`, `p2` - Filter by priority
- `critical`, `high`, `medium`, `low` - Filter by severity
- `fixable` - Only vulnerabilities with patches
- `quick-wins` - Easy fixes (non-breaking patches)
- `kev` - CISA Known Exploited Vulnerabilities only

### Reachability Analysis

Reachability analysis runs automatically on every scan, using **full AST-based call graph analysis** to determine which vulnerabilities are actually exploitable.

How it works:
1. Extracts container filesystem (docker/podman)
2. Detects languages and ecosystems in the container
3. **Runs language-specific call graph analyzers** for each detected ecosystem:
   - **JavaScript/TypeScript**: SWC-based AST parsing with import/require tracking
   - **Python**: RustPython AST parser with framework-aware analysis
   - **Go**: tree-sitter AST with goroutine and reflection tracking
   - **Rust**: Native syn parser with trait implementation tracking
   - **Ruby**: tree-sitter with Rails/RSpec/metaprogramming support
   - **PHP**: tree-sitter with Laravel/Symfony/WordPress detection
4. Determines if vulnerable code is reachable from entrypoints
5. Marks vulnerabilities as 🎯 REACHABLE or 🛡️ unreachable

Benefits:
- **70-90% noise reduction** - Focus only on exploitable vulnerabilities
- **AST-based precision** - Real call graph analysis, not heuristics
- **Framework-aware** - Understands framework-specific execution patterns
- **Smart prioritization** - Combine reachability with P0-P4 scoring

Example output:
```
🔴 CVE-2024-1234 [P0] 🎯 REACHABLE
   in log4j-core 2.14.1 → 2.17.1
   Call chain: main() → processRequest() → Logger.log()

🟡 CVE-2024-5678 [P2] 🛡️ unreachable
   in unused-lib 1.0.0 → 1.0.1
   Unused transitive dependency
```

**Note**: Uses the same battle-tested reachability analyzers that power BazBOM's core vulnerability scanning (70-98% accuracy depending on language).

### Interactive TUI Mode

Explore vulnerabilities interactively:
```bash
bazbom container-scan myapp:latest --interactive
```

Features:
- Navigate with arrow keys
- Filter and sort vulnerabilities
- View detailed CVE information
- Copy remediation commands
- Press 'q' to quit

### GitHub Integration

Automatically create GitHub issues for P0/P1 vulnerabilities:
```bash
bazbom container-scan myapp:latest --create-issues owner/repo
```

Creates issues with:
- CVE details and severity
- Package information
- Remediation steps
- References and links
- `security` label

Requires [GitHub CLI](https://cli.github.com/):
```bash
brew install gh
gh auth login
```

### Executive Reports

Generate HTML reports for management:
```bash
bazbom container-scan myapp:latest --report security-report.html
```

Report includes:
- Executive summary
- Vulnerability metrics
- Priority breakdown
- Recommended actions
- Visual charts and graphs

Perfect for sharing with non-technical stakeholders.

## Understanding the Output

### 1. Layer Attribution

```
Layer 1: sha256:abc123... (Base OS packages)
  Size: 28.5 MB | Packages: 142 | 🔴 12 vulns (3C/5H/3M/1L)
  📦 Packages: glibc, openssl, curl, and 139 more...
  🔍 Top vulnerabilities:
     🔴 CVE-2025-41249 [P0] 🚨 KEV (due: 2025-12-31)
        in openssl → 3.0.8 ⚠️ breaking | EPSS: 85.0%
        CVSS: 9.8 | https://nvd.nist.gov/vuln/detail/CVE-2025-41249
        💡 Major version upgrade 1→3 may require code changes
```

- **Layer digest** - Identifies the Docker layer
- **Layer description** - What this layer contains (Base OS, Java runtime, App files)
- **Size** - Layer size in MB
- **Package count** - Number of packages in this layer
- **Vulnerability breakdown** - Count by severity (C/H/M/L)
- **Top vulnerabilities** - Most critical issues in this layer with full context

### 2. Quick Wins with Multi-CVE Grouping

```
⚡ QUICK WINS (15 minutes, 8 vulns fixed!)

  1. Update commons-io: 2.4 → 2.11.0
     ✅ Fixes 2 CVEs: CVE-2021-29425, CVE-2024-47554
     🟢 Difficulty: 15/100 (Trivial - patch update)
     ⏱  Time: ~5 minutes

  2. Update jackson-databind: 2.13.0 → 2.17.1
     ✅ Fixes 2 CVEs: CVE-2023-35116, CVE-2024-12345
     🟡 Difficulty: 35/100 (Easy - minor version jump)
     ⏱  Time: ~5 minutes
```

Quick wins are:
- **Fixable** - Patch available
- **Non-breaking** - Minor or patch version updates
- **High impact** - Fixes multiple vulnerabilities (consolidated via multi-CVE grouping)
- **Fast** - Estimated < 30 minutes each
- **Low difficulty** - 0-40 difficulty score

### 3. Action Plan

```
📋 RECOMMENDED ACTION PLAN

🔥 URGENT (Do TODAY):
  1. [P0/KEV] CVE-2025-41249 in openssl
     ⏱  Est: 1 hour
     ⚠️  Breaking change - review migration guide
     📊 EPSS: 85% (high exploitation risk)

⚠️  HIGH PRIORITY (This week):
  2. [P1] CVE-2024-47554 in jackson-databind
     ⏱  Est: 15 minutes

🟡 MEDIUM PRIORITY (This sprint):
  12 vulnerabilities requiring attention
  ⏱  Estimated total: 3.5 hours
```

Prioritization logic:
- **P0** - KEV present, CVSS ≥ 9.0, or EPSS ≥ 90%
- **P1** - CVSS ≥ 7.0 AND (KEV or EPSS ≥ 50%)
- **P2** - CVSS ≥ 7.0 OR (CVSS ≥ 4.0 AND EPSS ≥ 10%)
- **P3** - CVSS ≥ 4.0
- **P4** - Everything else

### 4. Copy-Paste Fixes (7 Languages)

```
📋 COPY-PASTE FIXES

  ☕ Package: commons-io:commons-io
     ✅ Fixes 2 CVEs: CVE-2021-29425, CVE-2024-47554

  Maven (pom.xml):
  ```xml
  <dependency>
    <groupId>commons-io</groupId>
    <artifactId>commons-io</artifactId>
    <version>2.11.0</version>
  </dependency>
  ```

  Gradle (build.gradle):
  ```groovy
  implementation 'commons-io:commons-io:2.11.0'
  ```
```

Ready-to-use dependency updates for 7 languages (Java, Python, JavaScript, Go, Rust, Ruby, PHP) with multi-CVE grouping showing all vulnerabilities fixed by each upgrade.

### 5. Security Score

```
🏆 SECURITY SCORE

  Score: 62/100 - ⚠️  Needs Work

  🚀 To improve:
    • Fix 2 KEV vulnerabilities: +10 points
    • Fix 3 CRITICAL vulnerabilities: +30 points
    • Fix 5 HIGH vulnerabilities: +10 points

  📊 Industry average: 65/100
  🎯 Target: 75/100
```

Score calculation:
- Start at 100
- -10 per CRITICAL (max -100)
- -2 per HIGH (max -40)
- -1 per MEDIUM (max -50)
- -5 per KEV (max -25)

## Requirements

BazBOM's container scanning uses 6 external tools for comprehensive analysis:

### Install All Tools (macOS)
```bash
brew install syft trivy grype dockle dive trufflehog
```

### Install All Tools (Linux)
```bash
# Syft - SBOM generation
curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh

# Trivy - Vulnerability scanning
curl -sSfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh

# Grype - Vulnerability cross-validation
curl -sSfL https://raw.githubusercontent.com/anchore/grype/main/install.sh | sh

# Dockle - CIS benchmark checks
VERSION=$(curl -s https://api.github.com/repos/goodwithtech/dockle/releases/latest | grep tag_name | cut -d '"' -f 4 | sed 's/v//')
curl -L -o dockle.tar.gz https://github.com/goodwithtech/dockle/releases/download/v${VERSION}/dockle_${VERSION}_Linux-64bit.tar.gz
tar xzf dockle.tar.gz && sudo mv dockle /usr/local/bin/

# Dive - Layer efficiency analysis
wget https://github.com/wagoodman/dive/releases/latest/download/dive_*_linux_amd64.deb
sudo dpkg -i dive_*_linux_amd64.deb

# TruffleHog - Secrets scanning
curl -sSfL https://raw.githubusercontent.com/trufflesecurity/trufflehog/main/scripts/install.sh | sh
```

BazBOM automatically checks for these tools and provides installation instructions if missing. Missing tools are skipped gracefully - the scan continues with available tools.

## Output Formats

### Standard Output
```bash
bazbom container-scan myapp:latest -o ./output
```

Generates:
- `./output/scan-results.json` - BazBOM enriched results with all analysis
- `./output/report.html` - Executive HTML report
- `./output/report.pdf` - Executive PDF report
- `./output/compliance/pci-dss.html` - PCI-DSS compliance report
- `./output/compliance/hipaa.html` - HIPAA compliance report
- `./output/compliance/soc2.html` - SOC 2 compliance report
- `./output/syft-sbom.json` - Syft SBOM with layer metadata
- `./output/trivy-results.json` - Trivy vulnerability report
- `./output/grype-results.json` - Grype vulnerability report
- `./output/dockle-results.json` - CIS benchmark results
- `./output/dive-analysis.json` - Layer efficiency analysis
- `./output/trufflehog-secrets.json` - Secrets scan results

### scan-results.json Structure

The main `scan-results.json` contains:

```json
{
  "image_name": "myapp:latest",
  "total_packages": 142,
  "total_vulnerabilities": 23,
  "critical_count": 2,
  "high_count": 5,
  "medium_count": 10,
  "low_count": 6,
  "base_image": "alpine:3.19",
  "layers": [...],
  "upgrade_recommendations": [...],
  "reachability_summary": {
    "total_analyzed": 23,
    "reachable_count": 8,
    "unreachable_count": 15,
    "noise_reduction_percent": 65.2
  },
  "compliance_results": {
    "pci_dss": {
      "status": "Fail",
      "issues": [
        "2 critical vulnerabilities present",
        "5 high severity vulnerabilities",
        "1 known exploited vulnerability (CISA KEV)"
      ]
    },
    "hipaa": {
      "status": "Fail",
      "issues": [...]
    },
    "soc2": {
      "status": "Pass",
      "issues": []
    }
  }
}
```

**Key fields:**
- `reachability_summary` - Shows noise reduction from reachability analysis
- `compliance_results` - Shows pass/fail status with specific failure reasons for each framework

### Vulnerability Enrichment

Each vulnerability in `scan-results.json` includes enriched data:

```json
{
  "cve_id": "CVE-2024-1234",
  "package_name": "openssl",
  "installed_version": "1.1.1k",
  "fixed_version": "3.0.8",
  "severity": "CRITICAL",
  "cvss_score": 9.8,
  "epss_score": 0.85,
  "epss_percentile": 0.98,
  "is_kev": true,
  "kev_due_date": "2025-01-15",
  "priority": "P0",
  "is_reachable": true,
  "difficulty_score": 55,
  "breaking_change": true
}
```

### Severity Lookup Chain

BazBOM uses a multi-source severity lookup to ensure accurate severity data:

1. **Primary source** - Trivy/Grype vulnerability databases
2. **OSV fallback** - For vulnerabilities with UNKNOWN severity, queries the Open Source Vulnerabilities database
3. **NVD API fallback** - If OSV doesn't have severity, falls back to NVD API lookup

For OS packages (Alpine, Debian, Ubuntu, RHEL), native advisory databases are used:
- Alpine secdb (doesn't include severity - uses OSV/NVD fallback)
- Debian Security Tracker (includes urgency mapping)
- Ubuntu CVE Tracker
- Red Hat Security Data API

### SARIF Output
```bash
bazbom container-scan myapp:latest --format sarif -o ./output
```

Compatible with GitHub Security tab and other SARIF consumers.

## CI/CD Integration

### GitHub Actions
```yaml
name: Container Security Scan

on: [push, pull_request]

jobs:
  container-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Syft
        run: |
          curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh
          sudo mv syft /usr/local/bin/

      - name: Install Trivy
        run: |
          curl -sSfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh
          sudo mv trivy /usr/local/bin/

      - name: Build container
        run: docker build -t myapp:${{ github.sha }} .

      - name: Scan with BazBOM
        run: |
          bazbom container-scan myapp:${{ github.sha }} \
            --format sarif \
            -o ./scan-results \
            --show p0

      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: ./scan-results/findings/sarif.json

      # Fail on P0 vulnerabilities
      - name: Check for P0 vulnerabilities
        run: |
          if grep -q '"priority": "P0"' ./scan-results/scan-results.json; then
            echo "❌ P0 vulnerabilities found - build failed"
            exit 1
          fi
```

### GitLab CI
```yaml
container-scan:
  stage: security
  image: ubuntu:latest
  before_script:
    - apt-get update && apt-get install -y curl docker.io
    - curl -sSfL https://raw.githubusercontent.com/anchore/syft/main/install.sh | sh
    - curl -sSfL https://raw.githubusercontent.com/aquasecurity/trivy/main/contrib/install.sh | sh
  script:
    - docker build -t $CI_PROJECT_NAME:$CI_COMMIT_SHA .
    - bazbom container-scan $CI_PROJECT_NAME:$CI_COMMIT_SHA --show p0
  artifacts:
    reports:
      container_scanning: scan-results.json
  only:
    - main
    - merge_requests
```

## Best Practices

### 1. Regular Baseline Updates
Update your baseline after every release:
```bash
bazbom container-scan myapp:v1.5.0 --baseline
```

### 2. Use Filters in CI
Only fail on P0 vulnerabilities:
```bash
bazbom container-scan myapp:latest --show p0 || exit 1
```

### 3. Track Progress
Compare before and after remediation:
```bash
# Before fixes
bazbom container-scan myapp:latest --baseline

# After fixes
bazbom container-scan myapp:latest --compare-baseline
```

### 4. Choose Secure Base Images
Compare different base images:
```bash
bazbom container-scan alpine:3.18 --compare alpine:3.19
bazbom container-scan ubuntu:20.04 --compare ubuntu:22.04
```

### 5. Fix Quick Wins First
Focus on easy wins:
```bash
bazbom container-scan myapp:latest --show quick-wins
```

## Troubleshooting

### "Syft not found"
Install Syft:
```bash
brew install syft  # macOS
# or visit: https://github.com/anchore/syft#installation
```

### "Trivy not found"
Install Trivy:
```bash
brew install trivy  # macOS
# or visit: https://trivy.dev/latest/getting-started/installation/
```

### "docker inspect failed"
Ensure the image is pulled:
```bash
docker pull nginx:latest
bazbom container-scan nginx:latest
```

### Empty layer attribution
This can happen with some base images. BazBOM will still show all vulnerabilities, just without layer-specific mapping.

## Comparison with Other Tools

| Feature | BazBOM | Trivy | Syft | Grype | Snyk |
|---------|--------|-------|------|-------|------|
| SBOM Generation | ✅ | ✅ | ✅ | ❌ | ✅ |
| Vulnerability Scanning | ✅ | ✅ | ❌ | ✅ | ✅ |
| **Multi-Tool Orchestration** | ✅ (6 tools) | ❌ | ❌ | ❌ | ❌ |
| **Layer Attribution** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **EPSS Enrichment** | ✅ | ❌ | ❌ | ❌ | Paid |
| **KEV Detection** | ✅ | ❌ | ❌ | ❌ | Paid |
| **Priority Scoring** | ✅ (P0-P4) | ❌ | ❌ | ❌ | Paid |
| **Threat Intelligence** | ✅ | ❌ | ❌ | ❌ | Paid |
| **Quick Wins Analysis** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Breaking Change Detection** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Reachability Analysis** | ✅ | ❌ | ❌ | ❌ | Paid |
| Baseline Comparison | ✅ | ❌ | ❌ | ❌ | ✅ |
| Image Comparison | ✅ | ❌ | ❌ | ❌ | Paid |
| GitHub Integration | ✅ | ❌ | ❌ | ❌ | ✅ |
| Interactive TUI | ✅ | ❌ | ❌ | ❌ | ❌ |
| **Compliance Reports** | ✅ (PCI/HIPAA/SOC2) | ❌ | ❌ | ❌ | Paid |
| **PDF Reports** | ✅ | ❌ | ❌ | ❌ | Paid |
| Executive Reports | ✅ | ❌ | ❌ | ❌ | Paid |

## CLI Reference

```bash
bazbom container-scan <image> [OPTIONS]

OPTIONS:
  --preset <MODE>                 Scan preset: quick, standard, full (default), compliance
  --output <DIR>                  Output directory (default: ~/Documents/container-scans/<image>)
  --format <FORMAT>               Output format: spdx, sarif (default: spdx)
  --baseline                      Save as baseline for future comparisons
  --compare-baseline              Compare with saved baseline
  --compare <IMAGE>               Compare with another image
  --create-issues <OWNER/REPO>    Create GitHub issues for P0/P1 vulnerabilities
  --interactive                   Launch interactive TUI explorer
  --report <FILE>                 Generate executive report (HTML)
  --show <FILTER>                 Filter results (p0, p1, p2, critical, high,
                                  medium, low, fixable, quick-wins, kev)
  --with-reachability             Enable reachability (enabled by default)
  -h, --help                      Print help
```

## Examples

### Complete workflow for a production image:
```bash
# 1. Initial scan and baseline
bazbom container-scan prod-app:v2.3.0 --baseline

# 2. Generate executive report for stakeholders
bazbom container-scan prod-app:v2.3.0 --report security-report.html

# 3. Create GitHub issues for urgent vulnerabilities
bazbom container-scan prod-app:v2.3.0 --create-issues myorg/prod-app --show p0

# 4. Identify quick wins
bazbom container-scan prod-app:v2.3.0 --show quick-wins

# 5. After fixes, compare progress
bazbom container-scan prod-app:v2.3.1 --compare-baseline
```

### Choosing a base image:
```bash
# Compare different Node.js base images
bazbom container-scan node:18-alpine --compare node:20-alpine
bazbom container-scan node:18-alpine --compare node:18-slim
bazbom container-scan node:18-alpine --compare node:18-bullseye
```

### Pre-deployment validation:
```bash
# Ensure no P0 vulnerabilities before deploying
bazbom container-scan myapp:staging --show p0 && \
  echo "✅ Safe to deploy" || \
  echo "❌ Fix P0 vulnerabilities first"
```

---

**Container scanning is one of BazBOM's most powerful features.** The combination of layer attribution, intelligent prioritization, and actionable remediation guidance makes it the most developer-friendly container security tool available.

For general BazBOM documentation, see the [main README](../../README.md).
