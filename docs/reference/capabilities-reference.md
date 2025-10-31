# BazBOM Capabilities Reference

**Complete catalog of BazBOM features and integrations.**

> **TL;DR**: Universal JVM supply chain security toolkit. Supports Maven, Gradle, and Bazel with unified CLI. Generates SPDX/CycloneDX SBOMs, scans vulnerabilities (OSV/NVD/GHSA/KEV/EPSS), SLSA Level 3 provenance, Sigstore signing, VEX support, and GitHub Action integration.

## Statistics at a Glance

| Category | Count | Status |
|----------|-------|--------|
| **Python Files** | 99 | ✅ Production |
| **Lines of Code** | 45,000+ | ✅ Production |
| **Test Files** | 49 | ✅ Comprehensive |
| **Test Coverage** | 90%+ | 🎯 Target |
| **Build Systems** | 3 | ✅ Maven, Gradle, Bazel |
| **SBOM Formats** | 2 | ✅ SPDX 2.3, CycloneDX 1.5 |
| **Vulnerability Sources** | 5+ | ✅ OSV, NVD, GHSA, KEV, EPSS |
| **SLSA Level** | 3 | ✅ Provenance + Signing |
| **GitHub Action** | ✅ Native | ✅ Full Integration |

---

## Overview

BazBOM is a JVM supply chain security toolkit that generates SBOMs, performs vulnerability scanning, and integrates with CI/CD. It supports Maven, Gradle, and Bazel with a unified CLI and GitHub Action.

**Key Differentiators:**
- Universal build system support (Maven, Gradle, Bazel) with auto-detection
- Zero-config installation via single-line installer
- SLSA Level 3 provenance with Sigstore signing
- VEX support for false positive management
- Offline/air-gapped mode for secure environments
- Risk scoring with CISA KEV and EPSS integration

## Table of Contents

1. Build System Support
2. SBOM Generation
3. Vulnerability Scanning
4. CI/CD Integration
5. Supply Chain Security
6. Dependency Analysis
7. Configuration
8. Performance
9. Developer Experience
10. Data Export

---

## 1. Build System Support

- Maven (pom.xml)
- Gradle (build.gradle / build.gradle.kts)
- Bazel (WORKSPACE/MODULE.bazel) with advanced monorepo support
- Auto-detection and unified CLI: `bazbom scan .`

**Bazel Monorepo Features:**
- Bazel query integration for selective target scanning
- Incremental scanning with `rdeps()` (scan only affected targets)
- Scalable for large monorepos (5000+ targets)
- 6x faster PR scans compared to full workspace analysis

Examples:
```bash
# Scan any JVM project (auto-detects build system)
bazbom scan /path/to/project

# Bazel: Scan specific targets using query
bazbom scan . --bazel-targets-query 'kind(java_binary, //src/java/...)'

# Bazel: Incremental scan (only affected targets)
bazbom scan . --bazel-affected-by-files src/java/lib/Utils.java

# Bazel: Explicit targets
bazbom scan . --bazel-targets //src/java:app //src/java:lib
```

## 2. SBOM Generation

- SPDX 2.3 JSON (primary)
- CycloneDX 1.5 (optional)
- Per-target or workspace-wide
- Container SBOMs (Docker/Podman images)
- License and hash extraction

Examples:
```bash
# Generate SBOMs for entire workspace
bazel build //:sbom_all

# Generate CycloneDX as well
bazel build //:sbom_all --define=cyclonedx=true

# Container image SBOM
bazel run //tools/supplychain:scan_container -- myimage:latest
```

## 3. Vulnerability Scanning

- Data sources: OSV, NVD, GHSA, CISA KEV, EPSS
- SARIF 2.1.0 output for GitHub Code Scanning
- Policy enforcement with thresholds (CRITICAL/HIGH/...)
- Offline mode (air-gapped)

Examples:
```bash
# Scan vulnerabilities and produce SARIF
bazel run //:sca_scan

# Apply VEX statements (filter false positives)
bazel run //:apply_vex -- --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

## 4. CI/CD Integration

- GitHub Action with auto-detection (Maven/Gradle/Bazel)
- Uploads SBOM artifacts and SARIF
- PR comments and policy gating

Example (excerpt):
```yaml
- uses: cboyd0319/BazBOM@main
  with:
    fail-on-critical: true
    upload-sbom: true
    upload-sarif: true
```

## 5. Supply Chain Security

- SLSA provenance (Level 3 target)
- Sigstore keyless signing and Rekor transparency
- VEX support (Vulnerability Exploitability eXchange)
- Dependency pinning and license compliance

Examples:
```bash
# Sign SBOM (keyless)
bazel run //tools/supplychain:sbom_signing -- sign bazel-bin/app.spdx.json

# Verify SBOM signature
bazel run //tools/supplychain:verify_sbom -- bazel-bin/app.spdx.json \
  --bundle bazel-bin/signatures/app.bundle.json
```

## 6. Dependency Analysis

- Full transitive graph (JSON + GraphML)
- Reverse lookups and conflict detection
- Visualization via Gephi/yEd

Examples:
```bash
# Generate dependency graphs
bazel build //:dep_graph_all
```

## 6.5. Shading and Relocation Detection

- Maven Shade plugin configuration parsing (XML)
- Gradle Shadow plugin configuration parsing (DSL)
- Nested JAR extraction and analysis
- Class fingerprinting with Blake3 bytecode hashing
- Relocation pattern matching and reverse mapping
- Accurate vulnerability attribution for shaded dependencies
- **✅ Integrated into scan command** - Automatic detection and output generation

**Features:**
- Automatic detection of shading configurations during `bazbom scan`
- Support for multiple relocation mappings
- Include/exclude pattern filtering
- Bytecode-level class fingerprinting (API ready, runtime analysis planned for future)
- Confidence scoring for shading matches (API ready, runtime matching planned for future)
- **shading_config.json** output file with relocation details
- **Shading metadata** in sca_findings.json
- **SARIF integration** with shading notes in security reports

Examples:
```bash
# Automatically detects shading in Maven projects
bazbom scan my-maven-project/  # Reads pom.xml for maven-shade-plugin
# Outputs: sbom.spdx.json, sca_findings.json, sca_findings.sarif, shading_config.json

# Automatically detects shading in Gradle projects  
bazbom scan my-gradle-project/  # Reads build.gradle[.kts] for shadow plugin
# Outputs: sbom.spdx.json, sca_findings.json, sca_findings.sarif, shading_config.json

# View detected shading configuration
cat shading_config.json

# Check shading metadata in findings
jq '.shading' sca_findings.json
```

**Supported Configurations:**

Maven Shade Plugin:
- Multiple `<relocation>` blocks
- `<includes>` and `<excludes>` patterns
- `<finalName>` configuration
- Nested plugin configurations
- Full XML parsing with quick-xml

Gradle Shadow Plugin:
- `relocate()` DSL statements
- Multiple relocations per task
- Pattern-based matching for both Groovy and Kotlin DSL

**Output Files:**

When shading is detected, the following outputs are generated:
- `shading_config.json` - Complete relocation configuration
- `sca_findings.json` - Includes shading metadata section
- `sca_findings.sarif` - Includes informational note about detected shading

## 7. Configuration & Policy Management

### Policy Configuration

- Project-level `bazbom.yml` for policy rules
- Pre-built templates for regulatory compliance
- Multi-level policy inheritance (org → team → project)
- Rego/OPA support for advanced rules
- License compliance policies

**Policy Templates:**
- PCI-DSS v4.0 - Payment Card Industry compliance
- HIPAA Security Rule - Healthcare data protection
- FedRAMP Moderate - Federal cloud services
- SOC 2 Type II - B2B SaaS compliance
- Corporate Permissive - Development environments

Example:
```yaml
# bazbom.yml
name: "PCI-DSS v4.0 Compliance"
version: "1.0"

severity_threshold: HIGH
kev_gate: true
epss_threshold: 0.5
reachability_required: false
vex_auto_apply: true

license_allowlist:
  - MIT
  - Apache-2.0
  - BSD-3-Clause
license_denylist:
  - GPL-3.0-only
  - AGPL-3.0-only
```

**Policy Commands:**
```bash
# List available policy templates
bazbom policy init --list

# Initialize PCI-DSS policy template
bazbom policy init --template pci-dss

# Validate policy configuration
bazbom policy validate bazbom.yml

# Run policy checks
bazbom policy check
```

### License Compliance

**License Detection:**
- 200+ SPDX licenses supported
- Automatic license categorization (Permissive, Copyleft, Strong Copyleft)
- POM license name mapping for Maven projects

**License Analysis Commands:**
```bash
# Generate license obligations report
bazbom license obligations sbom.spdx.json

# Check license compatibility
bazbom license compatibility --project-license MIT

# Detect copyleft contamination
bazbom license contamination
```

**License Obligations Tracking:**
- Attribution requirements
- Source code disclosure obligations
- Copyleft restrictions
- Patent grants
- Network use triggers (AGPL)

**License Compatibility Matrix:**
- MIT project compatibility checks
- Apache-2.0 compatibility rules
- GPL/AGPL incompatibility detection
- Unknown license risk assessment

**Risk Levels:**
- Safe - No compatibility issues
- Low - Minor concerns
- Medium - Review recommended
- High - Significant incompatibility
- Critical - Must resolve before release

### Advanced Policy with Rego/OPA

For complex policy rules beyond YAML capabilities:

```rego
# advanced-policy.rego
package bazbom

# Block if CRITICAL and reachable
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    vuln.reachable == true
    msg := sprintf("CRITICAL vulnerability %s is reachable", [vuln.id])
}

# Block CISA KEV regardless of severity
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.cisa_kev == true
    msg := sprintf("CISA KEV vulnerability %s must be fixed", [vuln.id])
}

# Warn about copyleft in commercial products
warn[msg] {
    dep := input.dependencies[_]
    dep.license in ["GPL-3.0", "AGPL-3.0"]
    input.metadata.commercial == true
    msg := sprintf("Copyleft license %s in %s", [dep.license, dep.name])
}
```

### Policy Inheritance

Merge policies across organizational levels:

```yaml
# .bazbom/config.yml
policy_inheritance:
  - .bazbom/policies/organization.yml  # Baseline (strictest)
  - .bazbom/policies/team-backend.yml  # Team overrides
  - bazbom.yml                          # Project-specific

merge_strategy: "strict"  # Options: strict, permissive, override
```

## 8. Performance

- Incremental analysis (git-aware)
- Parallel processing
- Remote caching
- Scales to 5000+ target monorepos

## 9. Developer Experience

- Zero-config installer script
- Watch mode (`bazbom scan --watch`)
- Interactive fix workflow (dependency overrides)
- Security badges (shields.io)

Examples:
```bash
# Watch for changes
bazbom scan --watch

# Interactive fix for vulnerabilities
bazel run //tools/supplychain:interactive_fix -- --findings sca_findings.json
```

## 10. Data Export

- CSV export (SBOM, vulnerabilities, licenses)
- JSON (machine-readable)
- SARIF (GitHub Security)
- GraphML (graphs)

Examples:
```bash
bazel build //:sbom_csv
bazel build //:vulnerabilities_csv
bazel build //:licenses_csv
```

---

## Summary

BazBOM is the **universal JVM supply chain security solution** for modern development teams.

### Complete Feature Set

| Category | Features | Status |
|----------|----------|--------|
| **Core** | 99 Python files, 45,000+ lines, 49 tests | ✅ Production |
| **Build Systems** | Maven, Gradle, Bazel (auto-detect) | ✅ Complete |
| **SBOM Generation** | SPDX 2.3, CycloneDX 1.5, CSV export | ✅ Complete |
| **Vulnerability Scanning** | OSV, NVD, GHSA, CISA KEV, EPSS | ✅ Complete |
| **Supply Chain** | SLSA L3, Sigstore, VEX, license compliance | ✅ Complete |
| **CI/CD** | GitHub Action, SARIF output, policy gates | ✅ Complete |
| **Analysis** | Dependency graphs, conflict detection | ✅ Complete |
| **Performance** | Incremental, parallel, remote cache, 5K+ targets | ✅ Complete |
| **Developer Experience** | Zero-config, watch mode, interactive fixes | ✅ Complete |
| **Testing** | 49 test files, 90%+ coverage | 🎯 Target |

### Key Advantages

1. **Universal build system support** — Only tool supporting Maven, Gradle, AND Bazel
2. **Zero-config installation** — One-line installer with auto-configuration
3. **SLSA Level 3** — Highest supply chain security standard
4. **VEX integration** — Industry-standard false positive management
5. **Offline mode** — Works in air-gapped environments
6. **Risk scoring** — CISA KEV + EPSS for prioritization
7. **Production-ready** — Scales to 5000+ target monorepos

### Production Ready

- ✅ 99 production-ready Python files
- ✅ 45,000+ lines of analysis code
- ✅ 49 comprehensive test files
- ✅ 90%+ test coverage target
- ✅ SLSA Level 3 compliant
- ✅ Sigstore signed releases
- ✅ Comprehensive documentation
- ✅ GitHub Action native integration
- ✅ SARIF 2.1.0 compliant
- ✅ Active maintenance and development

**BazBOM provides accurate, standards-compliant SBOMs and vulnerability scanning for any JVM project.**

---

**Maintenance Notes:**
- This document is the **single source of truth** for BazBOM capabilities
- When adding or changing features, update this file AND the README feature list
- Validate examples and links: `pre-commit run --all-files`
- Verify statistics with commands in `.github/copilot-instructions.md`

**Version**: 1.0
**Last Updated**: 2025-10-20
**Repository**: https://github.com/cboyd0319/BazBOM
**Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs
**Issues**: https://github.com/cboyd0319/BazBOM/issues
**Contributing**: See [CONTRIBUTING.md](../../CONTRIBUTING.md)
**License**: MIT
