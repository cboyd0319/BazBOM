# BazBOM Product Roadmap

**Document Version:** 1.0  
**Last Updated:** 2025-11-03  
**Status:** Active Development  

> **Complete feature tracking for BazBOM development phases and milestones.**
>
> This document provides a consolidated view of all planned and implemented features across BazBOM.
> For detailed specifications, see individual phase documents in `docs/copilot/`.

---

## Quick Navigation

- [Current Status](#current-status)
- [Distribution & Marketplaces](#distribution--marketplaces)
- [Phase Checklist](#phase-checklist)
- [Detailed Phase Plans](#detailed-phase-plans)
- [Success Metrics](#success-metrics)

---

## Current Status

**Version:** 0.5.1  
**Overall Completion:** ~61% toward market leadership (⬆️ +3% this session)

### ✅ Completed Phases (0-3 + 5)
- **Phase 0-3:** Core infrastructure, Rust CLI, Maven/Gradle plugins, advisory system
- **Phase 5:** Enterprise policy templates, license compliance, Rego/OPA support

### 🚧 In Progress
- **Phase 4:** Developer experience (IDE plugins 95% complete, needs testing & publishing)
- **Phase 6:** Visualization (98% complete - CLI report integration ⬆️ +3%)
- **Phase 7:** Threat intelligence (80% complete - OSV/GHSA APIs fully implemented)
- **Phase 8:** Scale & performance (45% complete - cache integration complete ⬆️ +15%)
- **Phase 9:** Ecosystem expansion (35% complete - Docker integration)
- **Implementation Roadmap (Phases 1-2):** Quick wins & visual excellence ✅ COMPLETE

### 📋 Planned
- **Phase 10:** AI-powered intelligence
- **Phase 11:** Enterprise distribution (Windows, Kubernetes, air-gapped)
- **Implementation Roadmap Phase 3:** IDE marketplace publishing

---

## Distribution & Marketplaces

> **🎯 CRITICAL PRIORITY:** Ensuring BazBOM is easily and securely distributed through multiple channels.

### Distribution Channels Status

#### ✅ Completed
- [x] Homebrew tap (macOS/Linux) - `brew tap cboyd0319/bazbom`
- [x] GitHub Releases with signed binaries
- [x] Sigstore cosign signing
- [x] Shell script installer (`install.sh`)
- [x] GitHub Action (`action.yml`)
- [x] Source builds (Cargo)

#### 🚧 In Progress
- [ ] VS Code Marketplace - Extension ready, needs publishing
- [ ] JetBrains Marketplace - Plugin ready, needs publishing

#### 📋 Planned (Phase 11)
- [ ] GitHub Marketplace (Actions marketplace listing)
- [ ] Chocolatey (Windows)
- [ ] winget (Windows)
- [ ] Homebrew bottles (pre-built binaries)
- [ ] APT/DEB packages (Debian/Ubuntu)
- [ ] RPM packages (RHEL/Fedora)
- [ ] Docker Hub (container images)
- [ ] Kubernetes Operator
- [ ] Air-gapped enterprise bundles
- [ ] MSI installer (Windows)

### Marketplace Readiness

| Channel | Status | Documentation | Security | Priority |
|---------|--------|---------------|----------|----------|
| **Homebrew** | ✅ Live | [HOMEBREW_INSTALLATION.md](HOMEBREW_INSTALLATION.md) | ✅ Signed | P0 |
| **GitHub Releases** | ✅ Live | [RELEASE_PROCESS.md](RELEASE_PROCESS.md) | ✅ Signed | P0 |
| **VS Code Marketplace** | 🚧 Ready | [IDE_INTEGRATION.md](IDE_INTEGRATION.md) | ✅ Built-in | P0 |
| **JetBrains Marketplace** | 🚧 Ready | [IDE_INTEGRATION.md](IDE_INTEGRATION.md) | ✅ Built-in | P0 |
| **GitHub Marketplace** | 📋 Planned | [action.yml](../action.yml) exists | ✅ Actions | P1 |
| **Windows (Chocolatey)** | 📋 Planned | [PHASE_11](copilot/PHASE_11_DISTRIBUTION.md) | 🔲 Pending | P1 |
| **Windows (winget)** | 📋 Planned | [PHASE_11](copilot/PHASE_11_DISTRIBUTION.md) | 🔲 Pending | P1 |

---

## Phase Checklist

> **Master tracking checklist for all BazBOM development phases.**
> Use this for quick status checks and planning.

### Phase 0-3: Foundation ✅ COMPLETE

**Core Infrastructure & Rust Transition**

- [x] Rust workspace setup (7 crates)
- [x] CLI argument parsing and command structure
- [x] Build system detection (Maven, Gradle, Bazel)
- [x] Maven plugin (`bazbom-maven-plugin`)
- [x] Gradle plugin (`bazbom-gradle-plugin`)
- [x] Advisory database sync (OSV, NVD, GHSA, KEV, EPSS)
- [x] SBOM generation (SPDX 2.3, CycloneDX 1.5)
- [x] SARIF 2.1.0 output
- [x] VEX statement support
- [x] Reachability analysis (ASM-based)
- [x] Shading detection (Maven Shade, Gradle Shadow)
- [x] GitHub Action integration
- [x] Memory-safe Rust implementation (100%)
- [x] Zero Python runtime dependencies
- [x] Homebrew tap creation
- [x] Signed binary releases

### Phase 4: Developer Experience 🚧 95% COMPLETE

**IDE Integration, Auto-Remediation, Pre-Commit Hooks**

See [PHASE_4_PROGRESS.md](copilot/PHASE_4_PROGRESS.md) for detailed status.

#### 4.1 IDE Integration (95% - Code Complete, Needs Testing)
- [x] LSP server implementation (`bazbom-lsp`)
- [x] VS Code extension scaffolding
- [x] IntelliJ IDEA plugin implementation
  - [x] Dependency tree visualization
  - [x] Real-time vulnerability highlighting
  - [x] Quick fix actions
  - [x] Settings panel
  - [x] Auto-scan on project open
- [ ] Manual testing with real projects (5% remaining)
- [ ] VS Code Marketplace publishing
- [ ] JetBrains Marketplace publishing
- [ ] Performance profiling and optimization
- [ ] Demo videos and screenshots

#### 4.2 Automated Remediation (100% ✅)
- [x] `bazbom fix --suggest` command
- [x] Educational "why fix this?" explanations
- [x] `bazbom fix --apply` command
- [x] Test execution framework
- [x] Backup and rollback system
- [x] `bazbom fix --pr` command (GitHub PR generation)
- [ ] Real-world testing with vulnerable projects
- [ ] GitLab/Bitbucket support (future)

#### 4.3 Pre-Commit Hooks (100% ✅)
- [x] `bazbom install-hooks` command
- [x] Fast mode support (<10 seconds)
- [x] Policy enforcement
- [x] Bypass mechanism (`--no-verify`)
- [x] Unit tests (4 passing)

### Phase 5: Enterprise Policy ✅ COMPLETE

**Policy Templates, License Compliance, Rego/OPA Support**

See [PHASE_5_ENTERPRISE_POLICY.md](copilot/PHASE_5_ENTERPRISE_POLICY.md) for details.

- [x] Policy-as-code YAML format
- [x] Enterprise templates (PCI-DSS, HIPAA, FedRAMP, SOC 2)
- [x] License compliance engine (200+ SPDX licenses)
- [x] License compatibility matrix
- [x] Copyleft contamination detection
- [x] Rego/OPA integration (optional)
- [x] Policy inheritance (org → team → project)
- [x] CI enforcement examples
- [x] Policy validation command
- [x] Policy reporting

### Phase 6: Visualization 🚧 IN PROGRESS (98% Complete, ⬆️ +3%)

**Web Dashboard, Executive Reports, Dependency Graph UI**

See [PHASE_6_VISUALIZATION.md](copilot/PHASE_6_VISUALIZATION.md), [IMPLEMENTATION_ROADMAP.md](copilot/IMPLEMENTATION_ROADMAP.md), and [DASHBOARD_D3_IMPLEMENTATION.md](copilot/DASHBOARD_D3_IMPLEMENTATION.md) for details.

- [x] Embedded web dashboard (Axum backend)
- [x] Interactive dependency graph (D3.js force-directed)
- [x] Vulnerability timeline charts (Chart.js)
- [x] SBOM explorer interface with search/filter
- [x] Summary cards with key metrics
- [x] Responsive design (mobile, tablet, desktop)
- [x] Auto-refresh capability
- [x] Export SBOM to JSON
- [x] Executive summary reports (HTML, can convert to PDF)
- [x] Report generation crate (bazbom-reports)
- [x] Framework-specific compliance reports (7 frameworks: PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST CSF)
- [x] Detailed developer reports with remediation steps and code examples
- [x] Trend reports with security metrics and insights
- [x] CLI integration for report generation (`bazbom report` command) (NEW 2025-11-04)
- [ ] Static HTML export for sharing (reports already in HTML)
- [ ] Email integration for reports

### Phase 7: Threat Intelligence 🚧 IN PROGRESS (90% Complete, ⬆️ +10%)

**Supply Chain Attack Detection, Malicious Package Detection**

See [PHASE_7_THREAT_INTELLIGENCE.md](copilot/PHASE_7_THREAT_INTELLIGENCE.md) for details.

- [x] Threat detection crate (bazbom-threats)
- [x] Malicious package detection framework
- [x] Typosquatting detection with string similarity
- [x] Supply chain attack indicators
- [x] Continuous monitoring service
- [x] Threat level classification (Critical/High/Medium/Low)
- [x] Dependency confusion detection
- [x] OSV/GHSA database integration framework
- [x] OSV API client with HTTP implementation
- [x] GHSA GraphQL API client with authentication
- [x] Malicious keyword filtering for vulnerabilities
- [x] Fallback to curated data for offline operation
- [x] Integration tests for threat database APIs (NEW 2025-11-04)
- [x] Notification integrations (Slack, email, Teams, GitHub Issues) (NEW 2025-11-04)
- [x] Severity-based notification filtering (NEW 2025-11-04)
- [x] Color-coded and emoji-enhanced notifications (NEW 2025-11-04)
- [ ] Integration with scan command
- [ ] Maintainer takeover detection
- [ ] Integration with OpenSSF Scorecard
- [ ] Integration with Socket.dev signals
- [ ] Custom threat intelligence feeds

### Phase 8: Scale & Performance 🚧 IN PROGRESS (55% Complete, ⬆️ +10%)

**Incremental Analysis, Large Monorepo Optimization**

See [PHASE_8_SCALE_PERFORMANCE.md](copilot/PHASE_8_SCALE_PERFORMANCE.md) for details.

- [x] Intelligent caching framework (bazbom-cache)
- [x] LRU eviction policy
- [x] TTL-based expiration (1-hour default)
- [x] SHA-256 content hashing
- [x] Scan cache module with cache key generation
- [x] ScanResult caching infrastructure
- [x] Active integration with scan command execution
- [x] Cache hit/miss detection and logging
- [x] Environment variable to disable cache for testing
- [x] Incremental analysis framework with git-based change detection (NEW 2025-11-04)
- [x] ChangeSet tracking for modified/added/deleted files (NEW 2025-11-04)
- [x] Build file detection (pom.xml, build.gradle, BUILD.bazel, etc.) (NEW 2025-11-04)
- [x] Dependency file detection (lock files) (NEW 2025-11-04)
- [x] Smart rescan decision making (NEW 2025-11-04)
- [ ] Integration with scan orchestrator
- [ ] Bazel query optimization
- [ ] Parallel processing improvements
- [ ] Memory optimization for large projects
- [ ] Remote caching support
- [ ] Performance benchmarks (1K, 10K, 50K targets)
- [ ] Profile-guided optimization (PGO)
- [ ] 10x faster PR scans
- [ ] Support for 50K+ target monorepos

### Phase 9: Ecosystem Expansion 🚧 IN PROGRESS (45% Complete, ⬆️ +10%)

**Container Support, Multi-Language, Kubernetes**

See [PHASE_9_ECOSYSTEM_EXPANSION.md](copilot/PHASE_9_ECOSYSTEM_EXPANSION.md) for details.

- [x] Container scanning crate (bazbom-containers)
- [x] Java artifact detection in containers
- [x] Container SBOM generation framework
- [x] Docker daemon integration (DockerClient)
- [x] Docker API client architecture with Unix socket support
- [x] Pull, export, list, inspect operations with real/stub modes
- [x] Hyperlocal dependency for Unix socket HTTP
- [x] OCI image parsing implementation (NEW 2025-11-04)
- [x] OCI manifest parsing (NEW 2025-11-04)
- [x] OCI image configuration parsing (NEW 2025-11-04)
- [x] Java artifact scanning in layers (NEW 2025-11-04)
- [ ] Full HTTP client integration with hyperlocal
- [ ] Container layer extraction and analysis workflow
- [ ] Integration with scan command
- [ ] Container image SBOM (`rules_oci` integration)
- [ ] Kubernetes manifest scanning
- [ ] Node.js/npm support
- [ ] Python/pip support
- [ ] Go modules support
- [ ] Kotlin Multiplatform support
- [ ] Scala support
- [ ] Android-specific features
- [ ] Multi-language monorepo support

### Phase 10: AI Intelligence 📋 PLANNED

**ML Prioritization, LLM-Powered Fix Generation**

See [PHASE_10_AI_INTELLIGENCE.md](copilot/PHASE_10_AI_INTELLIGENCE.md) for details.

- [ ] ML-based vulnerability prioritization
- [ ] LLM-powered fix generation
- [ ] Natural language policy queries
- [ ] Automated remediation suggestions
- [ ] Code change impact analysis
- [ ] False positive prediction
- [ ] Semantic dependency search
- [ ] Smart batch fixing with conflict prediction
- [ ] Privacy-preserving ML (local models)
- [ ] Integration with GitHub Copilot

### Phase 11: Enterprise Distribution 📋 PLANNED

**Windows, Kubernetes, Air-Gapped, Enterprise Package Managers**

See [PHASE_11_DISTRIBUTION.md](copilot/PHASE_11_DISTRIBUTION.md) for details.

#### Windows Support
- [ ] Windows binary compilation
- [ ] MSI installer (WiX Toolset)
- [ ] Chocolatey package
- [ ] winget package
- [ ] Code signing (Authenticode)
- [ ] Windows Defender exclusions

#### Kubernetes & Cloud
- [ ] Kubernetes Operator
- [ ] Helm chart
- [ ] Service mesh integration
- [ ] Multi-cluster support
- [ ] Cloud marketplace listings (AWS, Azure, GCP)

#### Air-Gapped Deployments
- [ ] Offline bundle creation
- [ ] Advisory database export/import
- [ ] Zero internet requirement
- [ ] Sneakernet update mechanism
- [ ] Internal mirror support

#### Enterprise Package Managers
- [ ] SCCM integration
- [ ] Jamf integration
- [ ] Puppet module
- [ ] Ansible playbook
- [ ] Chef cookbook
- [ ] Salt formula

### Implementation Roadmap: 8-Week UX Sprint 📋 PLANNED

**Making BazBOM the Ultimate Easy-to-Use Solution**

See [IMPLEMENTATION_ROADMAP.md](copilot/IMPLEMENTATION_ROADMAP.md) for detailed 8-week plan.

#### Weeks 1-2: Quick Wins
- [ ] Interactive `bazbom init` command
- [ ] Expanded policy template library (20+ templates)
- [ ] Terminal-based dependency graph (TUI)
- [ ] Enhanced `bazbom fix --interactive` with batch processing

#### Weeks 3-4: Visual Excellence
- [ ] Embedded web dashboard MVP
- [ ] D3.js dependency graph visualization
- [ ] Executive reports (PDF)
- [ ] Compliance reports
- [ ] Shareable HTML exports

#### Weeks 5-6: IDE Polish
- [ ] VS Code extension 1.0 release
- [ ] IntelliJ IDEA plugin beta release
- [ ] Marketplace publishing
- [ ] One-click remediation polish
- [ ] Real-world testing campaign

#### Weeks 7-8: Team Features
- [ ] Git-based assignment system
- [ ] Team notifications (Slack, Email, Teams)
- [ ] Audit trail tracking
- [ ] Team dashboard with metrics
- [ ] Round-robin auto-assignment

---

## Detailed Phase Plans

### Complete Phase Documentation

Each phase has a dedicated specification document with detailed requirements, implementation plans, and acceptance criteria:

- **[PHASE_4_DEVELOPER_EXPERIENCE.md](copilot/PHASE_4_DEVELOPER_EXPERIENCE.md)** - IDE integration, auto-remediation, pre-commit hooks
- **[PHASE_4_PROGRESS.md](copilot/PHASE_4_PROGRESS.md)** - Current implementation status (95% complete)
- **[PHASE_5_ENTERPRISE_POLICY.md](copilot/PHASE_5_ENTERPRISE_POLICY.md)** - Policy templates and license compliance (COMPLETE)
- **[PHASE_6_VISUALIZATION.md](copilot/PHASE_6_VISUALIZATION.md)** - Web dashboard and reporting
- **[PHASE_7_THREAT_INTELLIGENCE.md](copilot/PHASE_7_THREAT_INTELLIGENCE.md)** - Supply chain attack detection
- **[PHASE_8_SCALE_PERFORMANCE.md](copilot/PHASE_8_SCALE_PERFORMANCE.md)** - Monorepo optimization
- **[PHASE_9_ECOSYSTEM_EXPANSION.md](copilot/PHASE_9_ECOSYSTEM_EXPANSION.md)** - Multi-language and containers
- **[PHASE_10_AI_INTELLIGENCE.md](copilot/PHASE_10_AI_INTELLIGENCE.md)** - ML and LLM features
- **[PHASE_11_DISTRIBUTION.md](copilot/PHASE_11_DISTRIBUTION.md)** - Enterprise distribution channels
- **[IMPLEMENTATION_ROADMAP.md](copilot/IMPLEMENTATION_ROADMAP.md)** - 8-week UX improvement sprint
- **[STRATEGIC_ROADMAP.md](copilot/STRATEGIC_ROADMAP.md)** - 12-18 month strategic vision

### Supporting Documentation

- **[IMPLEMENTATION_STATUS.md](copilot/IMPLEMENTATION_STATUS.md)** - Comprehensive audit of actual vs documented capabilities
- **[OPEN_SOURCE_SUSTAINABILITY.md](copilot/OPEN_SOURCE_SUSTAINABILITY.md)** - Funding and sustainability model
- **[BAZBOM_INTEGRATION_PLAN.md](copilot/BAZBOM_INTEGRATION_PLAN.md)** - Integration patterns
- **[EPICS_PORTING.md](copilot/EPICS_PORTING.md)** - Python to Rust porting plan (COMPLETE)
- **[REACHABILITY_OPAL.md](copilot/REACHABILITY_OPAL.md)** - OPAL reachability analysis plan

---

## Success Metrics

### Distribution & Adoption
- **Homebrew:** ✅ Live, needs usage analytics
- **IDE Plugins:** Target 1000+ VS Code, 500+ IntelliJ installs in first month
- **GitHub Marketplace:** Target listing by Q1 2026
- **Windows:** Target support by Q2 2026
- **Enterprise:** Target 10+ enterprises using policy templates

### Developer Experience
- **First Scan:** Target <60 seconds from install to first scan
- **Fix Success Rate:** Target 95% auto-fix success
- **IDE Performance:** Target <10 second scans
- **Team Adoption:** Target 80% voluntary adoption (not mandated)

### Technical Excellence
- **Coverage:** Maintain 90%+ test coverage
- **Performance:** Support 50K+ target monorepos
- **Security:** Maintain SLSA Level 3 certification
- **Stability:** Zero data loss incidents

### Market Position
- **Bazel Ecosystem:** Target 80% market share
- **Feature Parity:** Target 95% parity with commercial tools by Month 12
- **Community:** Target 10K+ GitHub stars by end of 2026

---

## Priority Guidance

### P0 - Critical Path (Must Have)
- Phase 4: IDE plugins (marketplace publishing)
- Phase 8: Scale & performance (monorepo optimization)
- Phase 11: Distribution (GitHub Marketplace, Windows support)

### P1 - High Impact (Should Have)
- Phase 6: Visualization (web dashboard)
- Phase 7: Threat intelligence
- Phase 9: Ecosystem expansion (containers)
- Phase 11: Enterprise distribution (K8s, air-gapped)

### P2 - Innovation (Nice to Have)
- Phase 10: AI intelligence
- Advanced team features
- Multi-language support

---

## Timeline Overview

| Quarter | Focus Areas | Key Deliverables |
|---------|-------------|------------------|
| **Q4 2025** | Phase 4 completion, Marketplace publishing | VS Code/IntelliJ live, GitHub Marketplace |
| **Q1 2026** | Phase 6-7, UX improvements | Web dashboard, threat intelligence |
| **Q2 2026** | Phase 8, Windows support | Monorepo optimization, Windows binaries |
| **Q3 2026** | Phase 9, Kubernetes | Containers, multi-language, K8s operator |
| **Q4 2026** | Phase 10-11, Enterprise | AI features, air-gapped deployments |

---

## Contributing to the Roadmap

The BazBOM roadmap is community-driven. Contributions welcome:

1. **Feature Requests:** Open an issue with `feature-request` label
2. **Vote on Features:** Comment on existing issues with 👍
3. **Implement Features:** See [CONTRIBUTING.md](../CONTRIBUTING.md)
4. **Discussion:** Join [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)

---

## Document Maintenance

This roadmap is updated regularly:

- **Monthly:** Phase progress updates
- **Quarterly:** Milestone reviews and priority adjustments
- **Major Releases:** Feature completion updates
- **Community Input:** Continuous incorporation of feedback

**Last Review:** 2025-11-03  
**Next Review:** 2025-12-01  
**Maintained By:** @cboyd0319 and [maintainers](../MAINTAINERS.md)

---

**[Back to Documentation Index](README.md)** | **[Strategic Vision](copilot/STRATEGIC_ROADMAP.md)** | **[Contributing Guide](../CONTRIBUTING.md)**
