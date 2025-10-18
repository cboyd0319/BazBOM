# BazBOM Evolution Roadmap
## Making Supply Chain Security Easy and Secure for Everyone

**Document Version:** 1.0  
**Date:** October 17, 2025  
**Project:** [BazBOM](https://github.com/cboyd0319/BazBOM)

---

## Executive Summary

BazBOM has a solid foundation: leveraging Bazel's build graph for accurate SBOM generation is brilliant. The next phase is about **accessibility** (work everywhere) and **depth** (go beyond basic CVE scanning). This roadmap prioritizes features that maximize adoption while maintaining security rigor.

**Three Strategic Pillars:**
1. **Universal Tool** - Work on any JVM project, not just Bazel
2. **Proactive Security** - Catch issues before they ship
3. **Enterprise Ready** - Meet compliance and scale requirements

---

## Part 1: Ease of Use (Removing Barriers)

### 1. Standalone CLI Tool ⭐ HIGHEST PRIORITY

**Why:** This is the gateway to universal adoption. Most Java shops don't use Bazel.

**Implementation:**
```bash
# Vision: one command that works anywhere
bazbom scan .

# Auto-detects:
# - Maven (pom.xml)
# - Gradle (build.gradle/build.gradle.kts)
# - Bazel (BUILD/WORKSPACE files)
# - SBT (build.sbt)
```

**Technical Approach:**
- **Core abstraction:** Create `DependencyResolver` interface with implementations for each build tool
- **Maven:** Parse `pom.xml` + resolve via `maven-resolver-api` (Aether)
- **Gradle:** Use Gradle Tooling API or parse `gradle dependencies --configuration runtimeClasspath` output
- **Bazel:** Use existing aspect-based approach
- **Detection logic:** Check for files in order: `BUILD` → `pom.xml` → `build.gradle` → `build.sbt`

**Packaging:**
```bash
# Single binary with Go wrapper
go build -o bazbom cmd/main.go

# Embedded JVM for dependency resolution
# Bundle Python scripts for SBOM generation
# Pack OSV database snapshot for offline mode
```

**Distribution:**
- GitHub Releases (Linux/Mac/Windows binaries)
- Homebrew: `brew install bazbom`
- Docker: `docker run bazbom/cli scan /workspace`
- NPM wrapper: `npx bazbom scan` (for convenience)

**Configuration file:** `bazbom.yml`
```yaml
build_system: auto  # or maven, gradle, bazel
include_test_deps: false
output_formats: [spdx, cyclonedx]
severity_threshold: MEDIUM
policy:
  block_critical: true
  fail_on_policy_violation: true
```

**Success Metric:** Developer can scan any Java project in < 30 seconds without reading docs.

---

### 2. Policy-as-Code with Build Blocking

**Why:** Security teams need enforcement, not just reporting.

**Implementation:**
```python
# .bazbom/policy.py or policy.yml
from bazbom.policy import Policy, Severity, Action

policy = Policy(
    name="company-standard",
    rules=[
        Rule(
            name="block-critical-in-production",
            condition=lambda dep: dep.severity == Severity.CRITICAL 
                                  and "prod" in dep.tags,
            action=Action.BLOCK,
            message="Critical CVE in production dependency: {cve_id}"
        ),
        Rule(
            name="warn-outdated-major",
            condition=lambda dep: dep.versions_behind > 2,
            action=Action.WARN,
            message="{dep} is 2+ major versions behind"
        ),
        Rule(
            name="allow-with-vex",
            condition=lambda vuln: vuln.has_vex_statement(),
            action=Action.ALLOW,
            message="VEX statement found: {vex_id}"
        )
    ],
    exceptions=[
        Exception(
            cve_id="CVE-2023-12345",
            reason="False positive - we don't use affected code path",
            expires="2025-12-31",
            approved_by="security-team"
        )
    ]
)
```

**Bazel integration:**
```python
# BUILD.bazel
load("@bazbom//:defs.bzl", "security_policy_test")

security_policy_test(
    name = "security_check",
    policy = "//:.bazbom/policy.py",
    sbom = ":app_sbom",
    # This can be added as a test dependency
    # bazel test //... will fail if policy violated
)
```

**Exit codes:**
- `0` - All checks pass
- `1` - Policy violations found (BLOCK severity)
- `2` - Warnings only (WARN severity)
- `3` - Policy file invalid

**Advanced features:**
- Policy inheritance (company → team → project)
- Conditional policies based on environment (`if env == "production"`)
- Time-based exceptions (auto-expire)
- Integration with issue trackers (auto-create Jira tickets for violations)

---

### 3. IDE Plugins (Real-Time Feedback)

**Why:** Shift security left - catch issues while coding, not in CI.

**IntelliJ IDEA Plugin:**
```kotlin
// Manifest
<idea-plugin>
  <name>BazBOM Security</name>
  <depends>com.intellij.modules.java</depends>
  <extensions defaultExtensionNs="com.intellij">
    <externalAnnotator 
      language="XML" 
      implementationClass="com.bazbom.PomAnnotator"/>
    <inspectionToolProvider 
      implementation="com.bazbom.VulnerabilityInspection"/>
  </extensions>
</idea-plugin>
```

**Features:**
- **Inline warnings:** Red squiggles under vulnerable dependencies in `pom.xml`/`build.gradle`
- **Hover tooltips:** 
  ```
  ⚠️ CVE-2023-12345 (CRITICAL)
  Remote code execution in log4j 2.14.1
  Fix: Upgrade to 2.17.1+
  [Quick Fix] [View Details] [Suppress]
  ```
- **Gutter icons:** Traffic light indicators (🔴🟡🟢) next to dependencies
- **Quick fixes:** One-click version upgrades
- **Dependency tree view:** Visual graph with vulnerability highlighting

**VS Code Extension:**
```typescript
// extension.ts
import * as vscode from 'vscode';
import { BazBOMClient } from './client';

export function activate(context: vscode.ExtensionContext) {
    const diagnosticCollection = vscode.languages.createDiagnosticCollection('bazbom');
    
    // Watch for file changes
    const watcher = vscode.workspace.createFileSystemWatcher('**/pom.xml');
    watcher.onDidChange(async (uri) => {
        const results = await BazBOMClient.scan(uri.fsPath);
        updateDiagnostics(uri, results, diagnosticCollection);
    });
}
```

**Performance considerations:**
- Background scanning (don't block IDE)
- Cache results (invalidate on file change)
- Incremental analysis (only changed deps)
- Language Server Protocol for cross-editor support

---

### 4. Zero-Config Installation

**Current Problem:** Still requires WORKSPACE editing, manual SHA calculation.

**Solution: Self-Configuring Setup**
```bash
# Option 1: Installer script (comprehensive)
curl -fsSL https://bazbom.dev/install.sh | bash
# Detects OS, downloads binary, adds to PATH
# For Bazel projects: auto-updates WORKSPACE and BUILD

# Option 2: Bazel rule that self-installs
bazel run @bazbom_installer//:setup
# Introspects workspace, generates correct config, creates rules

# Option 3: Container (no install)
docker run -v $(pwd):/workspace bazbom/cli scan /workspace
```

**Installer script features:**
```bash
#!/bin/bash
# install.sh

set -euo pipefail

# Detect OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

# Download correct binary
URL="https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-${OS}-${ARCH}"
curl -L "$URL" -o /usr/local/bin/bazbom
chmod +x /usr/local/bin/bazbom

# Detect project type
if [ -f "WORKSPACE" ]; then
    echo "Bazel project detected. Setting up..."
    
    # Get latest version and SHA
    VERSION=$(curl -s https://api.github.com/repos/cboyd0319/BazBOM/releases/latest | jq -r .tag_name)
    SHA256=$(curl -L "https://github.com/cboyd0319/BazBOM/releases/download/${VERSION}/bazbom.tar.gz.sha256")
    
    # Append to WORKSPACE if not present
    if ! grep -q "bazbom" WORKSPACE; then
        cat >> WORKSPACE << EOF

# BazBOM - Auto-configured by installer
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
http_archive(
    name = "bazbom",
    urls = ["https://github.com/cboyd0319/BazBOM/archive/${VERSION}.tar.gz"],
    sha256 = "${SHA256}",
    strip_prefix = "BazBOM-${VERSION}",
)
load("@bazbom//:deps.bzl", "bazbom_dependencies")
bazbom_dependencies()
EOF
    fi
    
    # Create root BUILD.bazel if missing
    if [ ! -f "BUILD.bazel" ] || ! grep -q "sbom_all" BUILD.bazel; then
        cat >> BUILD.bazel << EOF

# BazBOM targets
load("@bazbom//:defs.bzl", "sbom_all", "sca_scan")
sbom_all(name = "sbom_all")
sca_scan(name = "sca_scan")
EOF
    fi
    
    echo "✅ BazBOM configured! Try: bazel build //:sbom_all"
    
elif [ -f "pom.xml" ]; then
    echo "Maven project detected. Run: bazbom scan"
elif [ -f "build.gradle" ]; then
    echo "Gradle project detected. Run: bazbom scan"
fi

echo "Installation complete! Run 'bazbom --help' for usage."
```

**Verification step:**
```bash
bazbom verify-install
# Checks:
# - Binary is in PATH
# - Can connect to OSV API
# - Project configuration is valid
# - Sample scan works
```

---

### 5. GitHub Action (Instant CI Integration)

**Why:** GitHub is where most open source lives. Make it trivial to add security scanning.

**Implementation:**
```yaml
# .github/workflows/security.yml
name: Supply Chain Security

on: [push, pull_request]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run BazBOM Security Scan
        uses: cboyd0319/bazbom-action@v1
        with:
          # Auto-detects build system
          fail-on-critical: true
          upload-sbom: true
          upload-sarif: true
          
      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: sbom.spdx.json
          
      - name: Upload to GitHub Security
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: bazbom-results.sarif
```

**Action features:**
```typescript
// action.yml
name: 'BazBOM Security Scan'
description: 'SBOM generation and vulnerability scanning for JVM projects'
inputs:
  build-system:
    description: 'Build system (auto|maven|gradle|bazel)'
    default: 'auto'
  fail-on-critical:
    description: 'Fail workflow on CRITICAL vulnerabilities'
    default: 'true'
  policy-file:
    description: 'Path to policy file'
    default: '.bazbom/policy.yml'
  upload-sbom:
    description: 'Upload SBOM as artifact'
    default: 'true'
  upload-sarif:
    description: 'Upload SARIF to GitHub Security'
    default: 'true'
outputs:
  vulnerabilities-found:
    description: 'Number of vulnerabilities found'
  critical-count:
    description: 'Number of CRITICAL vulnerabilities'
  sbom-path:
    description: 'Path to generated SBOM'
```

**Smart features:**
- **Incremental scanning:** Only scan changed dependencies in PRs
- **Comment on PR:** Add summary comment with findings
- **Status checks:** Block merge if policy violated
- **Badge generation:** Create shields.io badge for README

**Example PR comment:**
```markdown
## 🔒 BazBOM Security Scan Results

**Status:** ⚠️ 2 vulnerabilities found

### Critical Issues (1)
- 🔴 **CVE-2023-12345** in `log4j-core:2.14.1`
  - Severity: CRITICAL (CVSS 10.0)
  - Fix: Upgrade to `2.17.1`
  - [Details](https://osv.dev/CVE-2023-12345)

### High Issues (1)
- 🟠 **CVE-2023-54321** in `jackson-databind:2.13.0`
  - Severity: HIGH (CVSS 8.1)
  - Fix: Upgrade to `2.13.4.2`

📊 Full report: [View SARIF](./bazbom-results.sarif)
```

---

## Part 2: Advanced Security Features

### 6. SBOM Signing and Verification

**Why:** Prove SBOMs haven't been tampered with. Required for supply chain integrity.

**Implementation with Sigstore:**
```bash
# Generate and sign SBOM
bazbom generate --sign ./app
# Outputs:
# - app.sbom.json (SBOM)
# - app.sbom.json.sig (signature)
# - app.sbom.json.cert (certificate)

# Verify SBOM
bazbom verify app.sbom.json
# ✅ Signature valid
# ✅ Certificate trusted (Fulcio root)
# ✅ Transparency log entry found (Rekor)
```

**Technical details:**
```python
# Use sigstore-python library
from sigstore.sign import Signer
from sigstore.verify import Verifier
from sigstore.models import Bundle

def sign_sbom(sbom_path: str) -> Bundle:
    """Sign SBOM using Sigstore keyless signing."""
    with open(sbom_path, 'rb') as f:
        sbom_bytes = f.read()
    
    signer = Signer.production()
    bundle = signer.sign_artifact(sbom_bytes)
    
    # Write bundle (contains signature + certificate + Rekor entry)
    with open(f"{sbom_path}.sigstore", 'w') as f:
        f.write(bundle.to_json())
    
    return bundle

def verify_sbom(sbom_path: str) -> bool:
    """Verify SBOM signature."""
    with open(sbom_path, 'rb') as f:
        sbom_bytes = f.read()
    
    with open(f"{sbom_path}.sigstore", 'r') as f:
        bundle = Bundle.from_json(f.read())
    
    verifier = Verifier.production()
    verifier.verify_artifact(sbom_bytes, bundle)
    return True  # Raises exception if invalid
```

**Workflow integration:**
```yaml
# CI/CD pipeline
steps:
  - name: Generate SBOM
    run: bazbom generate --output app.sbom.json
    
  - name: Sign SBOM
    run: bazbom sign app.sbom.json
    env:
      COSIGN_EXPERIMENTAL: "1"  # Keyless signing
      
  - name: Upload signed SBOM
    run: |
      aws s3 cp app.sbom.json s3://company-sboms/
      aws s3 cp app.sbom.json.sigstore s3://company-sboms/
```

**Verification in deployment:**
```bash
# Before deploying, verify SBOM
bazbom verify s3://company-sboms/app.sbom.json

# Check against policy
bazbom policy-check s3://company-sboms/app.sbom.json \
  --policy ./policies/production.yml

# Exit code 0 = safe to deploy
```

---

### 7. Private CVE Database Support

**Why:** Enterprises have internal vulnerability data not in public databases.

**Architecture:**
```
┌─────────────────┐
│  BazBOM CLI     │
└────────┬────────┘
         │
    ┌────┴─────┐
    │ Adapter  │ (plugin interface)
    └────┬─────┘
         │
    ┌────┴────────────────────────┐
    │  Vulnerability Sources      │
    ├─────────────────────────────┤
    │ • OSV (public)              │
    │ • NVD (public)              │
    │ • GitHub Advisory (public)  │
    │ • Company DB (private)      │
    │ • Vendor alerts (private)   │
    └─────────────────────────────┘
```

**Plugin interface:**
```python
# vulnerability_source.py
from abc import ABC, abstractmethod
from typing import List, Optional

class VulnerabilitySource(ABC):
    """Base class for vulnerability data sources."""
    
    @abstractmethod
    def query_package(self, package: str, version: str) -> List[Vulnerability]:
        """Query vulnerabilities for a specific package version."""
        pass
    
    @abstractmethod
    def is_available(self) -> bool:
        """Check if source is accessible."""
        pass

# Example: Private database plugin
class PrivateCVEDatabase(VulnerabilitySource):
    def __init__(self, db_url: str, api_key: str):
        self.db_url = db_url
        self.api_key = api_key
        
    def query_package(self, package: str, version: str) -> List[Vulnerability]:
        response = requests.get(
            f"{self.db_url}/api/v1/vulnerabilities",
            params={"package": package, "version": version},
            headers={"Authorization": f"Bearer {self.api_key}"}
        )
        return [Vulnerability.from_json(v) for v in response.json()]
    
    def is_available(self) -> bool:
        try:
            requests.head(self.db_url, timeout=5)
            return True
        except:
            return False
```

**Configuration:**
```yaml
# bazbom.yml
vulnerability_sources:
  - type: osv
    enabled: true
    cache_ttl: 3600
    
  - type: nvd
    enabled: true
    api_key: ${NVD_API_KEY}
    
  - type: custom
    name: company-cve-db
    plugin: /opt/bazbom/plugins/company_cve.py
    config:
      db_url: https://internal.company.com/cve-api
      api_key: ${COMPANY_CVE_API_KEY}
      timeout: 30
      
  - type: vendor
    name: oracle-alerts
    plugin: /opt/bazbom/plugins/oracle_alerts.py
    config:
      alert_feed: https://www.oracle.com/security-alerts/rss.xml
```

**Aggregation logic:**
```python
def aggregate_vulnerabilities(package: str, version: str) -> List[Vulnerability]:
    """Query all enabled sources and merge results."""
    all_vulns = []
    
    for source in enabled_sources:
        try:
            vulns = source.query_package(package, version)
            all_vulns.extend(vulns)
        except Exception as e:
            logger.warning(f"Source {source.name} failed: {e}")
    
    # Deduplicate by CVE ID
    unique_vulns = {v.cve_id: v for v in all_vulns}
    
    # Merge metadata if same CVE from multiple sources
    # (take highest severity, union of affected versions, etc.)
    return merge_duplicate_cves(unique_vulns.values())
```

**Air-gapped deployment:**
```bash
# On internet-connected machine
bazbom download-databases \
  --output /tmp/vuln-db.tar.gz \
  --include osv,nvd,github

# Transfer to air-gapped environment
scp /tmp/vuln-db.tar.gz secure-env:/opt/bazbom/

# On air-gapped machine
bazbom scan --offline-mode \
  --vuln-db /opt/bazbom/vuln-db.tar.gz

# Periodic updates (weekly)
# Download fresh DB, transfer via approved process
```

---

### 8. Dependency Risk Scoring (Beyond CVEs)

**Why:** A CVE-free package can still be risky. Consider: unmaintained, typosquatting, suspicious activity.

**Risk Score Formula:**
```
Risk Score = (CVE_Score × 0.4) + 
             (Maintenance_Score × 0.2) + 
             (Trust_Score × 0.2) + 
             (Supply_Chain_Score × 0.2)
```

**Implementation:**
```python
class DependencyRiskAnalyzer:
    def analyze(self, package: Package) -> RiskScore:
        score = RiskScore()
        
        # 1. CVE Score (40% weight)
        score.cve_score = self._calculate_cve_score(package)
        
        # 2. Maintenance Score (20% weight)
        score.maintenance_score = self._calculate_maintenance_score(package)
        
        # 3. Trust Score (20% weight)
        score.trust_score = self._calculate_trust_score(package)
        
        # 4. Supply Chain Score (20% weight)
        score.supply_chain_score = self._calculate_supply_chain_score(package)
        
        score.total = (
            score.cve_score * 0.4 +
            score.maintenance_score * 0.2 +
            score.trust_score * 0.2 +
            score.supply_chain_score * 0.2
        )
        
        return score
    
    def _calculate_maintenance_score(self, package: Package) -> float:
        """Lower score = more risk."""
        factors = []
        
        # Last commit recency
        days_since_commit = (datetime.now() - package.last_commit_date).days
        if days_since_commit > 730:  # 2 years
            factors.append(0.0)
        elif days_since_commit > 365:  # 1 year
            factors.append(0.3)
        else:
            factors.append(1.0)
        
        # Active maintainers
        if package.active_maintainers < 2:
            factors.append(0.2)
        else:
            factors.append(1.0)
        
        # Release cadence
        releases_per_year = len(package.releases_last_year)
        if releases_per_year == 0:
            factors.append(0.0)
        elif releases_per_year < 3:
            factors.append(0.5)
        else:
            factors.append(1.0)
        
        return sum(factors) / len(factors)
    
    def _calculate_trust_score(self, package: Package) -> float:
        """Higher score = more trustworthy."""
        factors = []
        
        # Package age (older = more established)
        years = (datetime.now() - package.created_date).days / 365
        if years > 5:
            factors.append(1.0)
        elif years > 2:
            factors.append(0.7)
        else:
            factors.append(0.5)
        
        # Download count (popular = more eyes on it)
        if package.monthly_downloads > 1_000_000:
            factors.append(1.0)
        elif package.monthly_downloads > 100_000:
            factors.append(0.8)
        elif package.monthly_downloads > 10_000:
            factors.append(0.6)
        else:
            factors.append(0.3)
        
        # Maintainer reputation
        if package.has_verified_maintainers:
            factors.append(1.0)
        else:
            factors.append(0.5)
        
        # Typosquatting detection
        if self._is_potential_typosquat(package.name):
            factors.append(0.0)
        else:
            factors.append(1.0)
        
        # OpenSSF Scorecard (if available)
        if package.openssf_score:
            factors.append(package.openssf_score / 10)
        
        return sum(factors) / len(factors)
    
    def _is_potential_typosquat(self, package_name: str) -> bool:
        """Detect packages with names similar to popular packages."""
        popular_packages = load_popular_packages()
        
        for popular in popular_packages:
            # Check Levenshtein distance
            if levenshtein_distance(package_name, popular) <= 2:
                return True
            
            # Check common substitutions
            if self._has_suspicious_substitutions(package_name, popular):
                return True
        
        return False
```

**Report output:**
```
┌─────────────────────────────────────────────────────────┐
│ Dependency Risk Report                                  │
├─────────────────────────────────────────────────────────┤
│ Package: log4j-core:2.14.1                             │
│ Overall Risk: 🔴 HIGH (8.5/10)                         │
├─────────────────────────────────────────────────────────┤
│ CVE Score:         🔴 9.8/10  (CRITICAL CVE-2021-44228)│
│ Maintenance:       🟢 8.2/10  (Active, 5+ maintainers) │
│ Trust:             🟢 9.5/10  (1M+ downloads, 15yo)    │
│ Supply Chain:      🟡 7.0/10  (Some transitive risks)  │
├─────────────────────────────────────────────────────────┤
│ Recommendations:                                        │
│ • URGENT: Upgrade to 2.17.1 (patches CVE-2021-44228)  │
│ • Consider: Switch to logback (lower risk profile)     │
└─────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────┐
│ Package: some-crypto-lib:1.0.0                         │
│ Overall Risk: 🔴 CRITICAL (9.2/10)                     │
├─────────────────────────────────────────────────────────┤
│ CVE Score:         🟢 0.0/10  (No known CVEs)          │
│ Maintenance:       🔴 1.0/10  (Last commit 3 years ago)│
│ Trust:             🔴 2.0/10  (< 100 downloads/month)  │
│ Supply Chain:      🔴 8.0/10  (Suspicious patterns)    │
├─────────────────────────────────────────────────────────┤
│ ⚠️  WARNING: Potential typosquat detected!              │
│    Similar to popular package: "apache-commons-crypto" │
│                                                         │
│ Risk Factors:                                           │
│ • Created 6 months ago (very new)                      │
│ • Only 1 maintainer (no verified identity)             │
│ • No GitHub repository linked                          │
│ • Extremely low download count                         │
│ • Name similar to trusted package                      │
│                                                         │
│ Recommendations:                                        │
│ • DO NOT USE - High probability of malicious intent    │
│ • Use official package instead                         │
│ • Report to security@company.com                       │
└─────────────────────────────────────────────────────────┘
```

---

### 9. Transitive Dependency Override Recommendations

**Why:** Most vulnerabilities are in transitive deps. Devs don't know how to fix them.

**Problem:**
```
Your app
└── good-library:1.0
    └── vulnerable-dep:2.0 (CVE-2023-12345)
```

**Solution:** Auto-generate overrides
```bash
bazbom fix --interactive

# Analysis:
# ✓ Found 3 vulnerabilities in transitive dependencies
# ✓ Analyzing compatibility...
# ✓ Generating fixes...

# Fix 1/3: vulnerable-dep:2.0 → 2.1
# Used by: good-library:1.0
# Breaking changes: None detected
# Apply fix? [y/N/skip all] y

# Generated fix for Maven:
```

```xml
<!-- Add to pom.xml -->
<dependencyManagement>
  <dependencies>
    <!-- BazBOM auto-generated fix for CVE-2023-12345 -->
    <dependency>
      <groupId>com.example</groupId>
      <artifactId>vulnerable-dep</artifactId>
      <version>2.1</version>
    </dependency>
  </dependencies>
</dependencyManagement>
```

**Bazel fix:**
```python
# Append to WORKSPACE
maven_install(
    artifacts = [
        # ... existing artifacts ...
    ],
    # BazBOM auto-generated override
    version_conflict_policy = "pinned",
    maven_install_json = "@//:maven_install.json",
    override_targets = {
        "com.example:vulnerable-dep": "@maven//:com_example_vulnerable_dep_2_1",
    },
)
```

**Gradle fix:**
```kotlin
// Add to build.gradle.kts
configurations.all {
    resolutionStrategy {
        // BazBOM auto-generated fix for CVE-2023-12345
        force("com.example:vulnerable-dep:2.1")
    }
}
```

**Compatibility testing:**
```python
def test_compatibility(old_version: str, new_version: str) -> CompatibilityReport:
    """Test if upgrade breaks anything."""
    
    # 1. Check semantic versioning
    if is_breaking_change(old_version, new_version):
        return CompatibilityReport(
            safe=False,
            reason="Major version change detected"
        )
    
    # 2. Run tests with new version
    test_results = run_tests_with_override(new_version)
    if not test_results.passed:
        return CompatibilityReport(
            safe=False,
            reason=f"Tests failed: {test_results.failures}"
        )
    
    # 3. Check API compatibility (if bytecode available)
    api_diff = compare_apis(old_version, new_version)
    if api_diff.has_breaking_changes:
        return CompatibilityReport(
            safe=False,
            reason=f"API breakage: {api_diff.removed_methods}"
        )
    
    return CompatibilityReport(safe=True)
```

---

### 10. Maven Plugin ⭐ HIGH PRIORITY

**Why:** Maven dominates enterprise Java. This unlocks massive adoption.

**Usage:**
```xml
<!-- Add to pom.xml -->
<build>
  <plugins>
    <plugin>
      <groupId>com.bazbom</groupId>
      <artifactId>bazbom-maven-plugin</artifactId>
      <version>1.0.0</version>
      <executions>
        <execution>
          <goals>
            <goal>generate-sbom</goal>
            <goal>scan</goal>
          </goals>
        </execution>
      </executions>
      <configuration>
        <outputFormat>spdx</outputFormat>
        <failOnCritical>true</failOnCritical>
      </configuration>
    </plugin>
  </plugins>
</build>
```

```bash
# Generate SBOM
mvn bazbom:sbom

# Scan for vulnerabilities
mvn bazbom:scan

# Both (typical CI usage)
mvn verify bazbom:scan
```

**Implementation:**
```java
@Mojo(name = "scan", defaultPhase = LifecyclePhase.VERIFY)
public class ScanMojo extends AbstractMojo {
    
    @Parameter(defaultValue = "${project}", required = true, readonly = true)
    private MavenProject project;
    
    @Parameter(property = "bazbom.failOnCritical", defaultValue = "true")
    private boolean failOnCritical;
    
    @Parameter(property = "bazbom.outputFile", defaultValue = "${project.build.directory}/bazbom-results.json")
    private File outputFile;
    
    @Component
    private RepositorySystem repositorySystem;
    
    @Component
    private ArtifactResolver artifactResolver;
    
    public void execute() throws MojoExecutionException {
        getLog().info("BazBOM: Scanning dependencies...");
        
        // 1. Resolve all dependencies (including transitive)
        Set<Artifact> artifacts = project.getArtifacts();
        List<Dependency> dependencies = resolveDependencies(artifacts);
        
        // 2. Generate SBOM
        SBOM sbom = SBOMGenerator.generate(dependencies);
        
        // 3. Scan for vulnerabilities
        ScanResults results = VulnerabilityScanner.scan(dependencies);
        
        // 4. Write results
        writeResults(results, outputFile);
        
        // 5. Check policy
        if (failOnCritical && results.hasCritical()) {
            throw new MojoExecutionException(
                "Critical vulnerabilities found: " + results.getCriticalCount()
            );
        }
        
        getLog().info("BazBOM: Scan complete. Results written to " + outputFile);
    }
    
    private List<Dependency> resolveDependencies(Set<Artifact> artifacts) {
        // Use maven-resolver to get full dependency tree
        CollectRequest collectRequest = new CollectRequest();
        for (Artifact artifact : artifacts) {
            collectRequest.addDependency(
                new org.eclipse.aether.graph.Dependency(
                    new DefaultArtifact(artifact.getGroupId(), 
                                       artifact.getArtifactId(), 
                                       artifact.getVersion()),
                    artifact.getScope()
                )
            );
        }
        
        DependencyResult result = repositorySystem.resolveDependencies(
            session, 
            new DependencyRequest(collectRequest, null)
        );
        
        return result.getRoot().getChildren();
    }
}
```

**Configuration options:**
```xml
<configuration>
  <!-- Output formats -->
  <outputFormat>spdx</outputFormat> <!-- or cyclonedx, both -->
  <outputDirectory>${project.build.directory}/sbom</outputDirectory>
  
  <!-- Scanning options -->
  <failOnCritical>true</failOnCritical>
  <failOnHigh>false</failOnHigh>
  <includeTestDeps>false</includeTestDeps>
  
  <!-- Policy -->
  <policyFile>${project.basedir}/.bazbom/policy.yml</policyFile>
  <vexDirectory>${project.basedir}/.bazbom/vex</vexDirectory>
  
  <!-- Vulnerability sources -->
  <sources>
    <source>osv</source>
    <source>nvd</source>
  </sources>
  
  <!-- Exclusions -->
  <excludes>
    <exclude>com.example:test-utils</exclude>
  </excludes>
</configuration>
```

---

### 11. Gradle Plugin

**Why:** Gradle is increasingly popular, especially for Android and Kotlin projects.

**Usage:**
```kotlin
// build.gradle.kts
plugins {
    id("com.bazbom.security") version "1.0.0"
}

bazbom {
    outputFormat = "spdx"
    failOnCritical = true
    policy = file(".bazbom/policy.yml")
}
```

```bash
# Generate SBOM
./gradlew generateSbom

# Scan for vulnerabilities
./gradlew bazbomScan

# Typical CI usage
./gradlew build bazbomScan
```

**Implementation:**
```kotlin
// BazbomPlugin.kt
class BazbomPlugin : Plugin<Project> {
    override fun apply(project: Project) {
        val extension = project.extensions.create("bazbom", BazbomExtension::class.java)
        
        // Register tasks
        project.tasks.register("generateSbom", GenerateSbomTask::class.java) {
            outputFormat.set(extension.outputFormat)
            outputFile.set(project.layout.buildDirectory.file("sbom.json"))
        }
        
        project.tasks.register("bazbomScan", ScanTask::class.java) {
            dependsOn("generateSbom")
            failOnCritical.set(extension.failOnCritical)
            policyFile.set(extension.policy)
        }
        
        // Hook into build lifecycle
        project.tasks.named("check").configure {
            dependsOn("bazbomScan")
        }
    }
}

// GenerateSbomTask.kt
abstract class GenerateSbomTask : DefaultTask() {
    
    @get:Input
    abstract val outputFormat: Property<String>
    
    @get:OutputFile
    abstract val outputFile: RegularFileProperty
    
    @TaskAction
    fun generate() {
        // 1. Resolve all configurations
        val dependencies = mutableSetOf<ResolvedDependency>()
        project.configurations.forEach { config ->
            if (config.isCanBeResolved) {
                dependencies.addAll(
                    config.resolvedConfiguration.firstLevelModuleDependencies
                )
            }
        }
        
        // 2. Build dependency tree (including transitive)
        val tree = buildDependencyTree(dependencies)
        
        // 3. Generate SBOM
        val sbom = when (outputFormat.get()) {
            "spdx" -> SpdxGenerator.generate(tree)
            "cyclonedx" -> CycloneDxGenerator.generate(tree)
            else -> throw IllegalArgumentException("Unknown format: ${outputFormat.get()}")
        }
        
        // 4. Write to file
        outputFile.get().asFile.writeText(sbom.toJson())
        
        logger.lifecycle("SBOM generated: ${outputFile.get().asFile}")
    }
}
```

**Advanced features:**
```kotlin
bazbom {
    // Multiple output formats
    formats = setOf("spdx", "cyclonedx")
    
    // Configuration filtering
    configurations = setOf("runtimeClasspath", "compileClasspath")
    excludeConfigurations = setOf("testRuntimeClasspath")
    
    // Variant-aware (Android)
    variants = setOf("release", "debug")
    
    // Custom vulnerability sources
    vulnerabilitySources {
        osv {
            enabled = true
            cacheDir = file("build/osv-cache")
        }
        custom {
            name = "company-db"
            url = "https://vuln.company.com/api"
            apiKey = System.getenv("COMPANY_CVE_API_KEY")
        }
    }
    
    // Policy
    policy {
        failOn = setOf(Severity.CRITICAL, Severity.HIGH)
        allowList = file(".bazbom/allowlist.yml")
        vexStatements = fileTree(".bazbom/vex")
    }
}
```

---

## Part 3: Expansion Beyond Bazel

### 12. Multi-Language Support

**Current:** Java/JVM only  
**Target:** Full JVM ecosystem + gradual expansion

**Phase 1: Full JVM Coverage**
```bash
bazbom scan --language kotlin    # Kotlin/Native detection
bazbom scan --language scala     # SBT + Maven
bazbom scan --language groovy    # Gradle + Grape
bazbom scan --language clojure   # Leiningen
```

**Implementation strategy:**
```python
# Language detector
def detect_languages(project_dir: Path) -> List[Language]:
    languages = []
    
    # Kotlin
    if any(project_dir.rglob("*.kt")):
        languages.append(Language.KOTLIN)
    
    # Scala
    if (project_dir / "build.sbt").exists():
        languages.append(Language.SCALA)
    
    # Groovy
    if any(project_dir.rglob("*.groovy")):
        languages.append(Language.GROOVY)
    
    # Clojure
    if (project_dir / "project.clj").exists():
        languages.append(Language.CLOJURE)
    
    return languages

# Multi-language dependency resolver
class MultiLanguageResolver:
    def __init__(self):
        self.resolvers = {
            Language.JAVA: MavenResolver(),
            Language.KOTLIN: KotlinResolver(),
            Language.SCALA: SbtResolver(),
            Language.GROOVY: GradleResolver(),
            Language.CLOJURE: LeiningenResolver(),
        }
    
    def resolve(self, project_dir: Path) -> List[Dependency]:
        languages = detect_languages(project_dir)
        all_deps = []
        
        for lang in languages:
            resolver = self.resolvers.get(lang)
            if resolver:
                deps = resolver.resolve(project_dir)
                all_deps.extend(deps)
        
        # Deduplicate cross-language dependencies
        return deduplicate_deps(all_deps)
```

**Phase 2: Adjacent Ecosystems**
- JavaScript/TypeScript (NPM/Yarn/PNPM)
- Python (pip/poetry/pipenv)
- Go (go.mod)
- Rust (Cargo.toml)

**Why stop at JVM?** Because security is universal. The architecture (dependency resolution → SBOM → vulnerability scan) works everywhere.

---

### 13. Visual Dependency Graph UI

**Why:** Teams need to understand their dependency tree. Text reports don't cut it for complex apps.

**Features:**
- **Interactive graph:** Zoom, pan, filter by severity
- **Vulnerability heatmap:** Color nodes by risk score
- **Path tracing:** "Why is this vulnerable package in my app?"
- **Diff mode:** Compare dependency trees across versions
- **Export:** PNG, SVG, PDF for reports

**Tech stack:**
```typescript
// React + D3.js + Cytoscape.js
import React from 'react';
import Cytoscape from 'cytoscape';
import CytoscapeComponent from 'react-cytoscapejs';

interface DependencyNode {
    id: string;
    label: string;
    version: string;
    vulnerabilities: Vulnerability[];
    riskScore: number;
}

const DependencyGraph: React.FC<{sbom: SBOM}> = ({sbom}) => {
    const [elements, setElements] = useState([]);
    const [selectedNode, setSelectedNode] = useState<DependencyNode | null>(null);
    
    useEffect(() => {
        // Convert SBOM to graph structure
        const nodes = sbom.packages.map(pkg => ({
            data: {
                id: pkg.id,
                label: pkg.name,
                version: pkg.version,
                riskScore: pkg.riskScore,
                color: getRiskColor(pkg.riskScore),
            }
        }));
        
        const edges = sbom.relationships.map(rel => ({
            data: {
                source: rel.from,
                target: rel.to,
            }
        }));
        
        setElements([...nodes, ...edges]);
    }, [sbom]);
    
    const layout = {
        name: 'dagre',  // Directed graph layout
        rankDir: 'TB',  // Top to bottom
        animate: true,
    };
    
    const stylesheet = [
        {
            selector: 'node',
            style: {
                'background-color': 'data(color)',
                'label': 'data(label)',
                'width': 'mapData(riskScore, 0, 10, 50, 150)',
                'height': 'mapData(riskScore, 0, 10, 50, 150)',
            }
        },
        {
            selector: 'edge',
            style: {
                'width': 2,
                'line-color': '#ccc',
                'target-arrow-color': '#ccc',
                'target-arrow-shape': 'triangle',
                'curve-style': 'bezier'
            }
        },
        {
            selector: '.critical',
            style: {
                'background-color': '#d32f2f',
                'border-width': 4,
                'border-color': '#fff',
            }
        }
    ];
    
    return (
        <div style={{display: 'flex', height: '100vh'}}>
            <CytoscapeComponent
                elements={elements}
                layout={layout}
                stylesheet={stylesheet}
                style={{width: '70%', height: '100%'}}
                cy={(cy) => {
                    cy.on('tap', 'node', (evt) => {
                        setSelectedNode(evt.target.data());
                    });
                }}
            />
            
            <div style={{width: '30%', padding: '20px', overflow: 'auto'}}>
                {selectedNode && (
                    <DependencyDetails node={selectedNode} />
                )}
            </div>
        </div>
    );
};

function getRiskColor(score: number): string {
    if (score >= 8) return '#d32f2f';  // Critical - red
    if (score >= 6) return '#f57c00';  // High - orange
    if (score >= 4) return '#fbc02d';  // Medium - yellow
    if (score >= 2) return '#689f38';  // Low - light green
    return '#388e3c';                   // None - green
}
```

**Web interface:**
```bash
# Start local server
bazbom serve --port 8080

# Opens browser to http://localhost:8080
# Shows all SBOMs in current project
# Live updates on file changes (watch mode)
```

**Path analysis:**
```
Why is log4j-core:2.14.1 in my app?

app:1.0.0
  └── spring-boot-starter-web:2.5.0
      └── spring-boot-starter-logging:2.5.0
          └── logback-classic:1.2.3
              └── log4j-to-slf4j:2.14.1
                  └── log4j-core:2.14.1 ← VULNERABLE

Recommendation: Exclude log4j-to-slf4j or upgrade spring-boot
```

---

### 14. Compliance Reporting (SOC2, ISO27001, NIST)

**Why:** Enterprises need to prove they're managing supply chain risk for audits.

**Generated reports:**

**SOC2 Supply Chain Controls:**
```markdown
# SOC2 Trust Services Criteria - Supply Chain Security

## CC6.1: Logical and Physical Access Controls

**Control Implementation:**
- All dependencies scanned with BazBOM before deployment
- Vulnerability scan results retained for 90 days
- SBOM generated for all production deployments
- Access to vulnerability database restricted to security team

**Evidence:**
- [SBOM Report - Q3 2025](./sboms/q3-2025/)
- [Vulnerability Scan Logs](./scans/q3-2025/)
- [Access Control Audit Log](./access-logs/)

**Test Results:**
✅ 100% of production deployments have associated SBOM
✅ 0 critical vulnerabilities in production as of audit date
✅ Average remediation time: 2.3 days for HIGH severity
```

**ISO 27001 A.14.2 - Security in Development:**
```markdown
# ISO 27001 Annex A.14.2.1 - Secure Development Policy

**Policy Statement:**
All software dependencies must be scanned for vulnerabilities before 
deployment to production. Critical vulnerabilities must be remediated 
or mitigated within 48 hours.

**Implementation:**
- BazBOM integrated into CI/CD pipeline
- Automated vulnerability scanning on every commit
- Policy enforcement: builds fail on critical CVEs
- Monthly dependency review meetings

**Metrics (Last 12 Months):**
| Metric | Target | Actual |
|--------|--------|--------|
| Critical CVEs in prod | 0 | 0 |
| Mean time to remediate HIGH | < 7 days | 3.2 days |
| SBOM coverage | 100% | 100% |
| False positive rate | < 5% | 2.1% |
```

**NIST SP 800-218 (SSDF):**
```markdown
# NIST Secure Software Development Framework (SSDF) Compliance

## PO.3.2: Maintain Awareness of Component Risks

**Practice Implementation:**
- Automated vulnerability scanning with BazBOM
- Software Bill of Materials (SBOM) for all releases
- Continuous monitoring of dependency risk scores
- Integration with CVE databases (OSV, NVD, GitHub)

**Evidence Artifacts:**
1. SBOM Archive (all releases): s3://company/sboms/
2. Scan Results: s3://company/scan-results/
3. Risk Score Trends: [Grafana Dashboard](https://metrics.company.com/bazbom)
4. Incident Response Log: [Jira Board](https://jira.company.com/SECURITY)

## PO.5.1: Respond to Vulnerabilities

**Response Process:**
1. BazBOM detects vulnerability (automated)
2. Ticket created in Jira (automated)
3. Security team triaged within 4 hours
4. Remediation plan created within 24 hours
5. Fix deployed based on severity:
   - CRITICAL: 24-48 hours
   - HIGH: 7 days
   - MEDIUM: 30 days
   - LOW: Next scheduled release

**SLA Compliance:**
- CRITICAL: 98% within SLA (Q3 2025)
- HIGH: 95% within SLA (Q3 2025)
```

**Implementation:**
```python
def generate_compliance_report(
    audit_type: str,
    start_date: date,
    end_date: date
) -> ComplianceReport:
    """Generate compliance report for audit."""
    
    # Collect evidence
    evidence = {
        'sboms': collect_sboms(start_date, end_date),
        'scans': collect_scan_results(start_date, end_date),
        'incidents': collect_security_incidents(start_date, end_date),
        'remediations': collect_remediations(start_date, end_date),
    }
    
    # Calculate metrics
    metrics = {
        'sbom_coverage': calculate_sbom_coverage(evidence),
        'critical_vulns_in_prod': count_critical_in_production(evidence),
        'mean_time_to_remediate': calculate_mttr(evidence),
        'false_positive_rate': calculate_false_positive_rate(evidence),
    }
    
    # Generate report based on audit type
    if audit_type == 'soc2':
        return generate_soc2_report(evidence, metrics)
    elif audit_type == 'iso27001':
        return generate_iso27001_report(evidence, metrics)
    elif audit_type == 'nist':
        return generate_nist_report(evidence, metrics)
    
    raise ValueError(f"Unknown audit type: {audit_type}")
```

```bash
# CLI usage
bazbom compliance-report \
  --type soc2 \
  --period q3-2025 \
  --output report.pdf

# Generates PDF with:
# - Control implementation details
# - Evidence artifacts
# - Metrics and trends
# - Non-compliance issues (if any)
```

---

### 15. Automated Dependency Update PRs

**Why:** Keeping dependencies current is tedious. Automate it.

**How it works:**
1. BazBOM runs nightly scan
2. Detects outdated dependencies with security fixes
3. Tests upgrades in isolation
4. Creates PR if tests pass

**Example PR:**
```markdown
## 🔒 Security Update: jackson-databind

**Current version:** 2.13.0  
**Proposed version:** 2.13.4.2

### Vulnerabilities Fixed
- 🔴 **CVE-2022-42003** (CRITICAL, CVSS 9.8)
  - Remote code execution via deserialization
  - [NVD Details](https://nvd.nist.gov/vuln/detail/CVE-2022-42003)

### Compatibility Analysis
✅ Semantic versioning: Patch release (safe)
✅ All tests passing (2,453 tests, 0 failures)
✅ No API changes detected
✅ Backward compatible

### Impact Analysis
**Affected modules:** 3
- `/services/api` - Direct dependency
- `/services/auth` - Transitive via spring-boot
- `/common/utils` - Direct dependency

**Build time impact:** +0.2 seconds (negligible)

### Deployment Plan
1. Merge this PR
2. Auto-deploy to staging (via CI/CD)
3. Soak for 24 hours
4. Manual approval for production

---

**Generated by BazBOM Auto-Update**
Review policy: [SECURITY.md](./SECURITY.md)
```

**Implementation:**
```python
# scheduled_updater.py
class DependencyUpdater:
    def __init__(self, repo: GitRepo):
        self.repo = repo
    
    def run_nightly_update(self):
        """Nightly job to check for security updates."""
        
        # 1. Scan current dependencies
        scan_results = scan_project(self.repo.path)
        
        # 2. Find dependencies with security fixes
        outdated = [
            dep for dep in scan_results.dependencies
            if dep.has_security_fix_available()
        ]
        
        # 3. For each outdated dependency
        for dep in outdated:
            # Create feature branch
            branch_name = f"bazbom/update-{dep.name}-{dep.latest_version}"
            self.repo.create_branch(branch_name)
            
            # Update dependency
            self.update_dependency(dep, dep.latest_version)
            
            # Run tests
            test_results = self.run_tests()
            
            if test_results.passed:
                # Create PR
                self.create_pull_request(
                    branch=branch_name,
                    title=f"Security Update: {dep.name}",
                    body=self.generate_pr_body(dep),
                    labels=['security', 'dependencies', 'automated']
                )
            else:
                # Log failure, notify team
                self.notify_team(
                    f"Auto-update failed for {dep.name}: tests failed"
                )
                self.repo.delete_branch(branch_name)
    
    def generate_pr_body(self, dep: Dependency) -> str:
        """Generate comprehensive PR description."""
        
        template = """
## 🔒 Security Update: {name}

**Current version:** {current_version}  
**Proposed version:** {new_version}

### Vulnerabilities Fixed
{vulnerabilities}

### Compatibility Analysis
{compatibility_checks}

### Impact Analysis
{impact_analysis}

### Deployment Plan
{deployment_plan}

---
**Generated by BazBOM Auto-Update**
        """
        
        return template.format(
            name=dep.name,
            current_version=dep.current_version,
            new_version=dep.latest_version,
            vulnerabilities=self.format_vulnerabilities(dep),
            compatibility_checks=self.analyze_compatibility(dep),
            impact_analysis=self.analyze_impact(dep),
            deployment_plan=self.get_deployment_plan(),
        )
```

**Configuration:**
```yaml
# .bazbom/auto-update.yml
auto_update:
  enabled: true
  schedule: "0 2 * * *"  # 2 AM daily
  
  # Update strategy
  strategy:
    security_only: true  # Only update if security fix
    patch_only: false    # Also update minor versions
    
  # Testing requirements
  tests:
    required: true
    timeout: 30m
    coverage_threshold: 80
    
  # PR settings
  pull_request:
    auto_merge: false   # Require manual review
    reviewers: ['@security-team']
    labels: ['dependencies', 'security', 'automated']
    
  # Filters
  include:
    - 'com.fasterxml.jackson.*'
    - 'org.springframework.*'
    - 'org.apache.*'
  exclude:
    - '*-test'
    - '*-dev'
```

---

### 16. Multi-Repo / Monorepo Support

**Why:** Large orgs have 100+ repos or massive monorepos. Need aggregate view.

**Multi-repo orchestration:**
```bash
# Scan all repos in organization
bazbom scan-org \
  --org mycompany \
  --output aggregate-report.json

# Output: Combined SBOM + vulnerability report for all repos
```

**Architecture:**
```
┌──────────────────────────────────────┐
│       BazBOM Orchestrator            │
├──────────────────────────────────────┤
│ • Discovers repos via GitHub API     │
│ • Clones/fetches latest              │
│ • Runs scan on each repo (parallel)  │
│ • Aggregates results                 │
│ • Deduplicates dependencies          │
│ • Generates org-wide report          │
└──────────────────────────────────────┘
         │
         ├── Repo 1: frontend
         ├── Repo 2: backend
         ├── Repo 3: mobile
         └── Repo 4: data-pipeline
```

**Aggregate report:**
```json
{
  "organization": "mycompany",
  "scan_date": "2025-10-17T00:00:00Z",
  "repositories_scanned": 47,
  "total_dependencies": 2834,
  "unique_dependencies": 892,
  "vulnerabilities": {
    "critical": 3,
    "high": 12,
    "medium": 45,
    "low": 89
  },
  "top_vulnerable_dependencies": [
    {
      "name": "log4j-core",
      "version": "2.14.1",
      "used_in_repos": 23,
      "vulnerabilities": ["CVE-2021-44228", "CVE-2021-45046"],
      "impact": "47% of repositories affected"
    }
  ],
  "recommendations": [
    "Upgrade log4j-core to 2.17.1 across 23 repositories",
    "Deprecate commons-collections 3.x (unmaintained, 12 repos using)"
  ]
}
```

**Monorepo support:**
```bash
# Scan specific targets in monorepo
bazbom scan \
  --targets //services/api,//services/auth \
  --output services-sbom.json

# Incremental scan (only changed targets)
bazbom scan \
  --incremental \
  --base-commit HEAD~1 \
  --output incremental-scan.json
```

**Dashboard:**
```
┌─────────────────────────────────────────────────────┐
│ Organization Security Dashboard                     │
├─────────────────────────────────────────────────────┤
│ 📊 Repository Health                                │
│                                                     │
│ ████████████░░░░ 75%  Compliant                    │
│ ░░░░░░░░░░░░████ 25%  Non-compliant                │
│                                                     │
│ 🔴 Critical Issues: 3                               │
│   • log4j RCE in 23 repos                          │
│   • jackson deserialization in 8 repos             │
│   • spring4shell in 2 repos (legacy apps)          │
│                                                     │
│ 🔧 Most Used Dependencies                           │
│   1. spring-boot-starter     (42 repos)            │
│   2. jackson-databind        (38 repos)            │
│   3. guava                   (35 repos)            │
│                                                     │
│ 📈 Trends (Last 30 Days)                            │
│   • Vulnerabilities found:  +12                    │
│   • Vulnerabilities fixed:  -18                    │
│   • Net improvement:        -6  ✅                 │
│                                                     │
│ 🎯 Action Items                                     │
│   [ ] Upgrade log4j in 23 repositories             │
│   [ ] Review VEX statements for 5 false positives │
│   [ ] Deprecated: commons-lang 2.x (EOL)          │
└─────────────────────────────────────────────────────┘
```

---

### 17. API for Integration

**Why:** Teams want to integrate BazBOM into their own tools and workflows.

**RESTful API:**
```bash
# Start API server
bazbom serve --api --port 8080

# Available endpoints:
# POST /api/v1/scan          - Submit project for scanning
# GET  /api/v1/scan/{id}     - Get scan results
# POST /api/v1/sbom/validate - Validate SBOM format
# GET  /api/v1/vulns/{cve}   - Get vulnerability details
# POST /api/v1/policy/check  - Check SBOM against policy
```

**Example usage:**
```python
import requests

# Submit scan
response = requests.post('http://localhost:8080/api/v1/scan', json={
    'repository': 'https://github.com/mycompany/myapp',
    'branch': 'main',
    'include_test_deps': False
})

scan_id = response.json()['scan_id']

# Poll for results
import time
while True:
    result = requests.get(f'http://localhost:8080/api/v1/scan/{scan_id}')
    status = result.json()['status']
    
    if status == 'completed':
        vulnerabilities = result.json()['vulnerabilities']
        print(f"Found {len(vulnerabilities)} vulnerabilities")
        break
    
    time.sleep(5)
```

**Webhook support:**
```yaml
# .bazbom/config.yml
webhooks:
  - url: https://slack.company.com/hooks/security
    events: [scan_complete, vulnerability_found]
    
  - url: https://jira.company.com/api/issues
    events: [critical_vulnerability]
    headers:
      Authorization: Bearer ${JIRA_TOKEN}
```

**GraphQL API (for complex queries):**
```graphql
# Query dependencies and their vulnerabilities
query {
  project(id: "myapp") {
    dependencies {
      name
      version
      vulnerabilities {
        cveId
        severity
        cvssScore
        fixedInVersion
      }
      dependents {
        name
      }
    }
  }
}

# Response
{
  "data": {
    "project": {
      "dependencies": [
        {
          "name": "log4j-core",
          "version": "2.14.1",
          "vulnerabilities": [
            {
              "cveId": "CVE-2021-44228",
              "severity": "CRITICAL",
              "cvssScore": 10.0,
              "fixedInVersion": "2.17.1"
            }
          ],
          "dependents": [
            {"name": "spring-boot-starter-log4j2"}
          ]
        }
      ]
    }
  }
}
```

---

### 18. Container Image SBOM Support

**Why:** Modern apps run in containers. Need SBOMs for entire runtime, not just app code.

**Integration with Docker:**
```dockerfile
# Dockerfile
FROM openjdk:11-jre-slim

# Add BazBOM layer scanning
RUN curl -fsSL https://bazbom.dev/install.sh | bash

COPY target/app.jar /app/app.jar

# Generate SBOM during build
RUN bazbom scan-container \
    --output /app/sbom.spdx.json \
    --include-os-packages

# Attach SBOM as OCI artifact
LABEL org.opencontainers.image.sbom=/app/sbom.spdx.json
```

**Scan existing images:**
```bash
# Scan image from registry
bazbom scan-image \
  myregistry.io/myapp:v1.2.3 \
  --output myapp-sbom.json

# Includes:
# - Application dependencies (JAR files)
# - OS packages (apt/yum)
# - Base image layers
# - All transitive dependencies
```

**Integration with rules_oci (Bazel):**
```python
# BUILD.bazel
load("@rules_oci//oci:defs.bzl", "oci_image")
load("@bazbom//:defs.bzl", "sbom_layer")

# Generate SBOM as layer
sbom_layer(
    name = "app_sbom_layer",
    deps = [":app"],
)

# Include SBOM in container
oci_image(
    name = "app_image",
    base = "@distroless_java",
    layers = [
        ":app_layer",
        ":app_sbom_layer",  # SBOM embedded in image
    ],
)
```

**Multi-stage SBOM:**
```
Container SBOM
├── Base Image (openjdk:11)
│   ├── OS packages (Debian)
│   └── JRE components
├── Application Layer
│   ├── app.jar
│   └── Application dependencies
└── Runtime Dependencies
    ├── Config files
    └── Secrets (excluded from SBOM)
```

---

### 19. Performance Optimizations

**Current bottlenecks:**
- Full dependency resolution on every scan
- Sequential vulnerability queries
- No caching across builds

**Optimizations:**

**1. Incremental analysis:**
```python
def incremental_scan(
    current_sbom: SBOM,
    previous_sbom: SBOM,
    changed_files: List[Path]
) -> ScanResults:
    """Only scan changed dependencies."""
    
    # Compute diff
    added = current_sbom.packages - previous_sbom.packages
    removed = previous_sbom.packages - current_sbom.packages
    changed = [
        pkg for pkg in current_sbom.packages
        if pkg in previous_sbom.packages
        and pkg.version != previous_sbom.get_package(pkg.name).version
    ]
    
    # Only scan new/changed packages
    to_scan = added | set(changed)
    
    # Reuse previous results for unchanged
    results = previous_results.copy()
    results.update(scan_packages(to_scan))
    
    return results
```

**2. Parallel scanning:**
```python
from concurrent.futures import ThreadPoolExecutor, as_completed

def parallel_scan(dependencies: List[Dependency]) -> ScanResults:
    """Scan dependencies in parallel."""
    
    results = []
    with ThreadPoolExecutor(max_workers=10) as executor:
        # Submit all scans
        futures = {
            executor.submit(scan_dependency, dep): dep
            for dep in dependencies
        }
        
        # Collect results as they complete
        for future in as_completed(futures):
            dep = futures[future]
            try:
                result = future.result()
                results.append(result)
            except Exception as e:
                logger.error(f"Failed to scan {dep}: {e}")
    
    return ScanResults(results)
```

**3. Smart caching:**
```python
# Three-tier cache
# 1. In-memory (current session)
# 2. Local disk (across sessions)
# 3. Remote (shared across team)

class CacheManager:
    def get_vulnerabilities(self, package: str, version: str) -> Optional[List[Vuln]]:
        # Try memory cache
        key = f"{package}:{version}"
        if key in self.memory_cache:
            return self.memory_cache[key]
        
        # Try disk cache
        cache_file = self.cache_dir / f"{package}-{version}.json"
        if cache_file.exists() and not self.is_stale(cache_file):
            vulns = self.load_from_disk(cache_file)
            self.memory_cache[key] = vulns
            return vulns
        
        # Try remote cache (Redis/S3)
        if self.remote_cache_enabled:
            vulns = self.remote_cache.get(key)
            if vulns:
                self.save_to_disk(cache_file, vulns)
                self.memory_cache[key] = vulns
                return vulns
        
        return None
    
    def is_stale(self, cache_file: Path) -> bool:
        """Cache is stale after 24 hours."""
        age = datetime.now() - datetime.fromtimestamp(cache_file.stat().st_mtime)
        return age > timedelta(hours=24)
```

**4. Dependency resolution caching:**
```python
# Cache resolved dependency trees
# Key: hash of pom.xml/build.gradle + lockfile
# Value: resolved dependency tree

def resolve_dependencies_cached(project: Project) -> DependencyTree:
    cache_key = compute_cache_key(project)
    
    if cached := cache.get(cache_key):
        logger.info("Using cached dependency resolution")
        return cached
    
    logger.info("Resolving dependencies (not cached)")
    tree = resolve_dependencies(project)
    
    cache.set(cache_key, tree, ttl=3600)
    return tree

def compute_cache_key(project: Project) -> str:
    """Hash of all files that affect dependency resolution."""
    files_to_hash = [
        project.path / "pom.xml",
        project.path / "maven_install.json",
        project.path / "BUILD.bazel",
    ]
    
    combined_hash = hashlib.sha256()
    for file in files_to_hash:
        if file.exists():
            combined_hash.update(file.read_bytes())
    
    return combined_hash.hexdigest()
```

**Performance targets:**
| Repo Size | Current | Optimized | Improvement |
|-----------|---------|-----------|-------------|
| Small     | 2 min   | 20 sec    | 6x faster   |
| Medium    | 5 min   | 45 sec    | 6.7x faster |
| Large     | 15 min  | 3 min     | 5x faster   |
| Massive   | 30 min  | 6 min     | 5x faster   |

---

### 20. Community & Ecosystem

**Why:** Open source thrives on community. Build an ecosystem around BazBOM.

**Package registries:**
- **Homebrew:** `brew install bazbom`
- **APT:** `apt install bazbom`
- **Docker Hub:** `docker pull bazbom/cli`
- **Maven Central:** For plugins
- **Gradle Plugin Portal:** For Gradle plugin

**Documentation site:**
```
https://bazbom.dev
├── /docs/getting-started
├── /docs/guides
│   ├── maven
│   ├── gradle
│   ├── bazel
│   └── ci-cd
├── /docs/api
├── /docs/plugins
├── /blog
└── /community
```

**Plugin ecosystem:**
```bash
# Plugin manager
bazbom plugins list
bazbom plugins install custom-reporter
bazbom plugins install slack-notifier
bazbom plugins install jira-integration
```

**Community features:**
- GitHub Discussions for Q&A
- Monthly community calls
- Contributor recognition program
- Security research bounties (for new vuln sources)
- Showcase page (who's using BazBOM)

**Training & certification:**
- Free online course: "Supply Chain Security with BazBOM"
- Hands-on labs
- Certification program for enterprise users

---

## Implementation Priority Matrix

### Phase 1: Foundation (Months 1-3)
**Goal:** Make BazBOM work everywhere, not just Bazel

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Standalone CLI | 🔥🔥🔥 | High | P0 |
| Maven Plugin | 🔥🔥🔥 | Medium | P0 |
| Policy Blocking | 🔥🔥 | Low | P0 |
| GitHub Action | 🔥🔥 | Low | P1 |
| Zero-Config Install | 🔥 | Medium | P1 |

### Phase 2: Security Depth (Months 4-6)
**Goal:** Go beyond basic CVE scanning

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| SBOM Signing | 🔥🔥 | Medium | P0 |
| Risk Scoring | 🔥🔥 | Medium | P0 |
| Private CVE DB | 🔥🔥 | Medium | P1 |
| Auto-fix PRs | 🔥 | High | P1 |
| IDE Plugins | 🔥 | High | P2 |

### Phase 3: Enterprise Features (Months 7-9)
**Goal:** Meet enterprise requirements

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Compliance Reports | 🔥🔥 | Low | P0 |
| Multi-repo Support | 🔥🔥 | Medium | P0 |
| API | 🔥 | Medium | P1 |
| Visual Graph UI | 🔥 | High | P2 |
| Container SBOM | 🔥 | Medium | P2 |

### Phase 4: Expansion (Months 10-12)
**Goal:** Support more ecosystems

| Feature | Impact | Effort | Priority |
|---------|--------|--------|----------|
| Gradle Plugin | 🔥🔥 | Medium | P0 |
| Multi-language | 🔥 | High | P1 |
| Performance Optimization | 🔥 | High | P1 |
| Community Building | 🔥🔥 | Ongoing | P0 |

**Legend:**
- 🔥🔥🔥 = Game changer
- 🔥🔥 = Major improvement
- 🔥 = Nice to have

---

## Quick Wins (Ship This Month)

These require minimal effort but have immediate impact:

1. **CSV export** (2 hours)
   ```bash
   bazbom scan --output results.csv
   ```

2. **Badge generation** (4 hours)
   ```markdown
   ![Security](https://bazbom.dev/badge/myorg/myrepo)
   ```

3. **Watch mode** (4 hours)
   ```bash
   bazbom scan --watch
   # Re-scans on file changes
   ```

4. **Better error messages** (8 hours)
   - Replace stack traces with helpful suggestions
   - Add troubleshooting URLs

5. **Homebrew tap** (2 hours)
   ```bash
   brew tap cboyd0319/bazbom
   brew install bazbom
   ```

---

## Technical Architecture Considerations

### For Standalone CLI

**Build system abstraction:**
```python
# Core abstraction
class BuildSystem(ABC):
    @abstractmethod
    def detect(self, path: Path) -> bool:
        """Can this build system handle this project?"""
        pass
    
    @abstractmethod
    def resolve_dependencies(self, path: Path) -> List[Dependency]:
        """Resolve all dependencies."""
        pass

# Implementations
class MavenBuildSystem(BuildSystem):
    def detect(self, path: Path) -> bool:
        return (path / "pom.xml").exists()
    
    def resolve_dependencies(self, path: Path) -> List[Dependency]:
        # Use maven-resolver API
        pass

class GradleBuildSystem(BuildSystem):
    def detect(self, path: Path) -> bool:
        return (path / "build.gradle").exists() or \
               (path / "build.gradle.kts").exists()
    
    def resolve_dependencies(self, path: Path) -> List[Dependency]:
        # Use Gradle Tooling API
        pass

# Factory
def detect_build_system(path: Path) -> Optional[BuildSystem]:
    systems = [BazelBuildSystem(), MavenBuildSystem(), GradleBuildSystem()]
    for system in systems:
        if system.detect(path):
            return system
    return None
```

### For Scale

**Architecture for massive repos (5K+ targets):**
```
┌────────────────────────────────────────────┐
│         BazBOM Distributed Scanner         │
├────────────────────────────────────────────┤
│                                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐│
│  │ Worker 1 │  │ Worker 2 │  │ Worker N ││
│  └────┬─────┘  └────┬─────┘  └────┬─────┘│
│       │             │             │       │
│  ┌────┴─────────────┴─────────────┴────┐ │
│  │      Redis Queue (work items)        │ │
│  └──────────────────────────────────────┘ │
│                                            │
│  ┌────────────────────────────────────┐  │
│  │   Result Aggregator                 │  │
│  │   • Deduplicates findings          │  │
│  │   • Merges SBOMs                   │  │
│  │   • Generates final report         │  │
│  └────────────────────────────────────┘  │
└────────────────────────────────────────────┘
```

**Horizontal scaling:**
- Each worker scans a subset of targets
- Results stored in shared cache (Redis/S3)
- Final aggregation step merges everything
- Scales to hundreds of workers

---

## Success Metrics

**Adoption metrics:**
- Downloads per month
- GitHub stars
- Active installations (telemetry opt-in)
- Community contributions

**Security metrics:**
- Average vulnerabilities per project (should decrease over time)
- Mean time to detect (MTTD)
- Mean time to remediate (MTTR)
- False positive rate (should be < 5%)

**Performance metrics:**
- Scan time (< 5 min for 90% of projects)
- Cache hit rate (> 80%)
- API uptime (> 99.9%)

**Enterprise adoption:**
- Number of paying customers
- Seat count
- Renewal rate

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| False positives annoy users | High | Rigorous VEX support, easy suppression |
| Performance issues at scale | High | Aggressive caching, parallel execution |
| Maintainability burden | Medium | Strong contributor guidelines, automated testing |
| Competing tools emerge | Medium | Focus on unique value (Bazel integration, accuracy) |
| CVE database outages | Low | Multiple sources, offline mode, caching |

---

## Conclusion

BazBOM has the foundation to become the **go-to security tool for JVM projects**. The key is removing barriers to adoption (standalone CLI, Maven/Gradle plugins) while deepening security capabilities (risk scoring, policy enforcement, signing).

**Next steps:**
1. Validate priorities with early users
2. Build standalone CLI (4-6 weeks)
3. Ship Maven plugin (2-3 weeks)
4. Launch with blog post + demos
5. Gather feedback and iterate

**The North Star:** Any developer, on any JVM project, can run `bazbom scan` and get actionable security insights in under 30 seconds—no configuration required.

---

## Resources & References

**Standards:**
- [SPDX 2.3 Specification](https://spdx.github.io/spdx-spec/)
- [CycloneDX 1.5](https://cyclonedx.org/specification/overview/)
- [SARIF 2.1](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
- [SLSA Framework](https://slsa.dev/)
- [OpenSSF Scorecard](https://github.com/ossf/scorecard)

**Vulnerability Databases:**
- [OSV.dev](https://osv.dev/)
- [NVD](https://nvd.nist.gov/)
- [GitHub Advisory Database](https://github.com/advisories)

**Similar Tools (for inspiration):**
- Dependabot
- Snyk
- Syft (container SBOM)
- Grype (vulnerability scanning)
- OWASP Dependency-Check

**Bazel Ecosystem:**
- [rules_jvm_external](https://github.com/bazelbuild/rules_jvm_external)
- [rules_oci](https://github.com/bazel-contrib/rules_oci)
- [Aspect CLI](https://aspect.build/cli)

---

**Document End**

Ready to make supply chain security easy and ubiquitous? Let's build it. 🚀
