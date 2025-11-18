# BazBOM 7.0.0 Roadmap: GitHub Marketplace Domination

**Release Target:** Q1 2026
**Mission:** Make BazBOM the #1 supply chain security tool developers WANT to use

---

## 🎯 Core Pillars

### 1. **ZERO VULNERABILITIES** - Security First
### 2. **GITHUB ACTIONS** - Seamless CI/CD Integration
### 3. **DEVELOPER JOY** - Easy, Fast, Helpful
### 4. **SUPPLY CHAIN EXCELLENCE** - Practice what we preach
### 5. **MARKETPLACE READY** - Polish, docs, support

---

## 📊 Current State (6.5.0 Baseline)

### ✅ Strengths
- ✅ **Zero critical vulnerabilities** (cargo audit + CodeQL + Semgrep clean)
- ✅ Comprehensive polyglot + reachability support (7 languages with AST/call graph analysis)
- ✅ Build-time accuracy (Bazel, Maven, Gradle, npm, pip, Go modules, Cargo, Bundler, Composer)
- ✅ Developer-friendly UX (quick commands, TUI, watch mode, smart defaults)
- ✅ 360+ passing tests + 30 production crates with unified versioning
- ✅ Supply chain posture: SLSA Level 3 provenance, Sigstore signing, SBOM+VEX artifacts

### ⚠️ Gaps for Marketplace
- ⚠️ Need turnkey GitHub Marketplace listing (pricing, billing, SLA, legal)
- ⚠️ Need polished onboarding inside `bazbom install github` (guided auth, status checks)
- ⚠️ Need signed IDE extension packages + marketplace marketing assets
- ⚠️ Need distribution parity (winget, apt, brew bottles) + auto-update story
- ⚠️ Need customer-facing audit artifacts (attestations + compliance bundles per release)
- ⚠️ Need dedicated support rotation + escalation SOP for Marketplace customers

---

## 🚀 Phase 1: ZERO VULNERABILITIES (Sprint 1-2)

### Dependency Health
- [ ] Replace `paste` dependency in ratatui chain
  - **Action:** Update ratatui or find alternative TUI library
  - **Owner:** Core team
  - **Timeline:** 1 week

- [ ] Automated dependency scanning
  - **Tool:** cargo-audit in CI
  - **Frequency:** Daily + every PR
  - **Action:** Fail CI on HIGH/CRITICAL
  - **Timeline:** 1 day

- [ ] Dependency pinning strategy
  - **Action:** Lock all dependencies with exact versions
  - **Rationale:** Reproducible builds
  - **Timeline:** 1 day

- [ ] SBOM for BazBOM itself
  - **Action:** Generate SPDX 2.3 SBOM for every release
  - **Publish:** Attach to GitHub releases
  - **Timeline:** 2 days

### Supply Chain Security
- [ ] SLSA Level 3 provenance
  - **Tool:** slsa-github-generator
  - **Action:** Generate provenance for all releases
  - **Verify:** Users can verify builds
  - **Timeline:** 1 week

- [ ] Sigstore signing
  - **Tool:** cosign
  - **Action:** Sign all releases with keyless signing
  - **Transparency:** Rekor log entries
  - **Timeline:** 1 week

- [ ] Reproducible builds
  - **Action:** Pin Rust version, disable timestamps
  - **Verify:** Two builds produce identical binaries
  - **Timeline:** 1 week

- [ ] Binary attestations
  - **Tool:** GitHub Artifact Attestations
  - **Action:** Attest all binaries (Linux, macOS, Windows)
  - **Timeline:** 3 days

---

## 🤖 Phase 2: GITHUB ACTIONS INTEGRATION (Sprint 3-5)

### Core Actions

#### 1. `bazbom-action/scan`
**Purpose:** Scan repository and generate SBOM

```yaml
- uses: bazbom-action/scan@v1
  with:
    # Build system (auto-detected by default)
    build-system: auto  # auto | maven | gradle | bazel | npm | python | go | rust

    # Output formats
    formats: spdx,cyclonedx  # spdx | cyclonedx | both
    output-dir: ./sbom

    # Vulnerability scanning
    scan-vulnerabilities: true
    fail-on: critical  # critical | high | medium | low | never

    # Policy enforcement
    policy-file: .bazbom/policy.yml
    fail-on-policy-violation: true

    # Upload results
    upload-sarif: true
    upload-dependency-graph: true
```

**Features:**
- Auto-detect build system
- Multi-format SBOM generation
- Vulnerability scanning with OSV
- Policy enforcement
- SARIF upload for GitHub Code Scanning
- Dependency graph upload for Dependabot

**Timeline:** 2 weeks

---

#### 2. `bazbom-action/verify`
**Purpose:** Verify SBOM signatures and attestations

```yaml
- uses: bazbom-action/verify@v1
  with:
    sbom-file: sbom/spdx.json
    signature-file: sbom/spdx.json.sig
    public-key: ${{ secrets.BAZBOM_PUBLIC_KEY }}
    verify-attestations: true
```

**Timeline:** 1 week

---

#### 3. `bazbom-action/container-scan`
**Purpose:** Scan container images

```yaml
- uses: bazbom-action/container-scan@v1
  with:
    image: myapp:latest
    output-dir: ./container-scan
    fail-on: critical
    upload-sarif: true
```

**Timeline:** 1 week

---

#### 4. `bazbom-action/policy-check`
**Purpose:** Standalone policy checking

```yaml
- uses: bazbom-action/policy-check@v1
  with:
    sbom-file: sbom/spdx.json
    policy-file: .bazbom/policy.yml
    fail-on-violation: true
```

**Timeline:** 3 days

---

### GitHub Integration Features

- [ ] **Dependency Graph Integration**
  - Upload SBOMs to GitHub Dependency Graph
  - Enable Dependabot alerts for all ecosystems
  - Timeline: 1 week

- [ ] **Code Scanning Integration**
  - Convert findings to SARIF
  - Upload to GitHub Code Scanning
  - Show results in PRs
  - Timeline: 3 days

- [ ] **Pull Request Comments**
  - Summary of vulnerabilities found
  - Breaking changes detected
  - Policy violations
  - Quick fix suggestions
  - Timeline: 1 week

- [ ] **Security Advisory Integration**
  - Query GitHub Security Advisories
  - Match against detected packages
  - Link to advisories in output
  - Timeline: 3 days

- [ ] **Attestation Verification**
  - Verify GitHub Artifact Attestations
  - Check SLSA provenance
  - Verify Sigstore signatures
  - Timeline: 1 week

---

## 🎨 Phase 3: DEVELOPER EXPERIENCE (Sprint 6-8)

### Quick Start Templates

#### Starter Workflows
Pre-built GitHub Actions workflows for common scenarios:

1. **Java/Maven Project**
   ```yaml
   name: SBOM Generation
   on: [push, pull_request]
   jobs:
     sbom:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: bazbom-action/scan@v1
           with:
             build-system: maven
             fail-on: high
   ```

2. **Multi-Language Monorepo**
3. **Container Image Scanning**
4. **Policy Enforcement**
5. **Continuous Compliance**

**Timeline:** 1 week

---

### Interactive Setup

- [ ] **GitHub App** (optional)
  - One-click installation
  - Auto-configures workflows
  - Creates policy files
  - Sets up secrets
  - Timeline: 2 weeks

- [ ] **CLI Init Command**
  ```bash
  bazbom init --github-actions
  # Creates:
  # - .github/workflows/bazbom.yml
  # - .bazbom/policy.yml
  # - .bazbom/.gitignore
  ```
  Timeline: 3 days

---

### Documentation

- [ ] **GitHub Actions Guide**
  - Complete setup tutorial
  - Common use cases
  - Troubleshooting
  - Best practices
  - Timeline: 1 week

- [ ] **Video Tutorials**
  - 5-minute quickstart
  - Advanced configurations
  - Policy writing
  - Timeline: 2 weeks

- [ ] **Live Examples**
  - Public repos using BazBOM
  - Example configurations
  - Real-world policies
  - Timeline: 1 week

---

## 🏆 Phase 4: CODE QUALITY & TESTING (Sprint 9-10)

### Test Coverage

- [ ] **Target: 80%+ coverage**
  - Current: ~60% (estimated)
  - Action: Add integration tests
  - Tool: tarpaulin or llvm-cov
  - Timeline: 2 weeks

- [ ] **GitHub Actions Integration Tests**
  - Test all actions end-to-end
  - Mock GitHub API
  - Verify SARIF uploads
  - Timeline: 1 week

- [ ] **Fuzzing**
  - Fuzz SBOM parsers
  - Fuzz policy engine
  - Tool: cargo-fuzz
  - Timeline: 1 week

- [ ] **Property-Based Testing**
  - Policy evaluation
  - SBOM generation
  - Tool: proptest
  - Timeline: 1 week

### Static Analysis

- [ ] **Clippy (strict mode)**
  - Enable all lints
  - Fix all warnings
  - CI enforcement
  - Timeline: 3 days

- [ ] **cargo-deny**
  - License compliance
  - Banned dependencies
  - Security advisories
  - Timeline: 2 days

- [ ] **MIRI**
  - Undefined behavior detection
  - Run on critical paths
  - Timeline: 1 week

---

## 🔒 Phase 5: SUPPLY CHAIN EXCELLENCE (Sprint 11-12)

### Release Process

- [ ] **Automated Release Pipeline**
  ```
  1. Version bump (semantic versioning)
  2. Run full test suite
  3. Build binaries (Linux, macOS, Windows)
  4. Generate SBOMs for each binary
  5. Sign with Sigstore (keyless)
  6. Generate SLSA provenance
  7. Create GitHub release
  8. Publish to crates.io
  9. Update GitHub Actions marketplace
  10. Verify everything
  ```
  Timeline: 2 weeks

- [ ] **Multi-Platform Builds**
  - Linux (x86_64, aarch64)
  - macOS (Intel, Apple Silicon)
  - Windows (x86_64)
  - Static linking where possible
  - Timeline: 1 week

- [ ] **Binary Verification**
  - SHA256 checksums
  - Sigstore signatures
  - SLSA provenance
  - Attestation bundles
  - Timeline: 3 days

### Transparency

- [ ] **Public Build Logs**
  - All builds on GitHub Actions
  - No private build steps
  - Reproducible locally
  - Timeline: 1 day

- [ ] **Vulnerability Disclosure**
  - security.txt file
  - SECURITY.md with process
  - Bug bounty program (future)
  - Timeline: 1 day

- [ ] **Release Notes Automation**
  - Auto-generate from commits
  - Highlight breaking changes
  - Link to security fixes
  - Timeline: 3 days

---

## 🌟 Phase 6: MARKETPLACE POLISH (Sprint 13-14)

### GitHub Marketplace Listing

- [ ] **Professional Branding**
  - Logo (256x256)
  - Banner image
  - Screenshots
  - Video demo
  - Timeline: 1 week

- [ ] **Marketplace Metadata**
  - Clear description
  - Category: Security, DevOps, SBOM
  - Tags: sbom, sca, vulnerability-scanning
  - Pricing: Free (open source)
  - Timeline: 2 days

- [ ] **README Excellence**
  - Badge: GitHub Marketplace
  - Quick start in 30 seconds
  - Common use cases
  - Support channels
  - Timeline: 3 days

### Support Infrastructure

- [ ] **GitHub Discussions**
  - Q&A category
  - Feature requests
  - Show and tell
  - Timeline: 1 day

- [ ] **Issue Templates**
  - Bug report
  - Feature request
  - GitHub Actions issue
  - Timeline: 1 day

- [ ] **Example Repositories**
  - bazbom-action-examples
  - Multiple language demos
  - Various configurations
  - Timeline: 1 week

---

## 📈 Success Metrics

### Technical KPIs
- ✅ Zero HIGH/CRITICAL vulnerabilities
- ✅ 80%+ test coverage
- ✅ SLSA Level 3 provenance
- ✅ <10s Action execution time (small repos)
- ✅ 99.9% Action success rate

### Adoption KPIs
- 🎯 1,000+ Action installs (month 1)
- 🎯 5,000+ Action installs (month 3)
- 🎯 100+ public repos using it
- 🎯 4.5+ star rating on Marketplace
- 🎯 50+ GitHub Discussions posts

### Developer Experience KPIs
- 🎯 <5 minutes: First SBOM generated
- 🎯 <10 minutes: Full setup with policies
- 🎯 <30 minutes: Custom policy written
- 🎯 >90% user satisfaction (survey)

---

## 🚧 Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|-----------|
| GitHub API rate limits | HIGH | MEDIUM | Cache aggressively, use GraphQL |
| Action execution time | MEDIUM | HIGH | Parallel scanning, smart caching |
| Breaking changes in Actions API | HIGH | LOW | Version pinning, testing |
| Marketplace approval delays | MEDIUM | MEDIUM | Early submission, clear docs |
| Sigstore/SLSA complexity | MEDIUM | MEDIUM | Good docs, templates |

---

## 💰 Resource Requirements

### Development
- **Core team:** 2-3 engineers
- **Duration:** 14 sprints (7 months)
- **Infrastructure:** GitHub Actions runners (free tier sufficient)

### Tools/Services
- ✅ GitHub Actions (free for public repos)
- ✅ Sigstore (free)
- ✅ SLSA generators (free)
- ✅ Rekor transparency log (free)
- ✅ crates.io publishing (free)

---

## 🎉 Launch Plan

### Week -2: Soft Launch
- Beta testing with 10 friendly users
- Dogfooding: BazBOM uses BazBOM Actions
- Fix critical bugs

### Week -1: Pre-Launch
- Final polish
- Documentation review
- Marketing materials ready
- Press kit prepared

### Week 0: PUBLIC LAUNCH 🚀
- GitHub Marketplace listing live
- Blog post announcement
- Twitter/LinkedIn/Reddit posts
- Email newsletter to subscribers
- Submit to Hacker News
- DevOps.com article

### Week 1-4: Post-Launch
- Monitor adoption metrics
- Respond to issues quickly
- Collect user feedback
- Iterate on pain points
- Weekly status updates

---

## 🔮 Future Vision (8.0+)

### Beyond GitHub Actions
- GitLab CI integration
- CircleCI orb
- Jenkins plugin
- Azure Pipelines task

### Advanced Features
- AI-powered vulnerability triaging
- Auto-PR for dependency updates
- Policy-as-Code IDE extensions
- Real-time dependency monitoring
- License compliance automation

### Enterprise Features
- SSO integration
- Air-gapped deployments
- On-premises scanning
- Custom advisory sources
- Advanced reporting

---

## ✅ Definition of Done (7.0.0 Release Criteria)

- [ ] Zero HIGH/CRITICAL vulnerabilities in dependencies
- [ ] All 4 GitHub Actions published to Marketplace
- [ ] SLSA Level 3 provenance for all binaries
- [ ] Sigstore signing for all releases
- [ ] 80%+ test coverage
- [ ] All documentation complete and reviewed
- [ ] 5+ example repositories
- [ ] GitHub Marketplace listing approved
- [ ] 10+ beta users successfully onboarded
- [ ] All migration guides from 6.0 ready

---

**Status:** PLANNING
**Last Updated:** 2025-11-11
**Next Review:** Sprint Planning (Week 1)

---

*Built with ❤️ for developers who care about supply chain security*
