# BazBOM 7.0.0 - Quick Start Plan

**Goal:** Ship first GitHub Action in 30 days

---

## 🚀 Sprint 0: Foundation (Week 1-2)

### Day 1-2: Security Baseline
```bash
# 1. Fix the one unmaintained dependency
- Update ratatui or replace TUI library
- Run cargo audit --deny warnings

# 2. Add automated security scanning
cat > .github/workflows/security.yml <<'EOF'
name: Security Audit
on: [push, pull_request, schedule]
jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/audit@v1
        with:
          denyWarnings: true
EOF

# 3. Generate SBOM for BazBOM itself
./target/release/bazbom scan . --spdx --out-dir ./sbom-self
```

### Day 3-5: SLSA Provenance Setup
```bash
# Add SLSA provenance generation to release workflow
cat > .github/workflows/release.yml <<'EOF'
name: Release with SLSA
on:
  push:
    tags: ['v*']
jobs:
  build:
    permissions:
      contents: write
      id-token: write
    uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.0.0
    with:
      base64-subjects: "${{ needs.build.outputs.hashes }}"
EOF
```

### Day 6-10: First GitHub Action
Create `bazbom-action` repository with basic scan action:

```typescript
// action.yml
name: 'BazBOM Scan'
description: 'Generate SBOM and scan for vulnerabilities'
inputs:
  fail-on:
    description: 'Fail on severity level'
    required: false
    default: 'critical'
runs:
  using: 'composite'
  steps:
    - run: |
        curl -sSL https://github.com/cboyd0319/BazBOM/releases/latest/download/bazbom-linux-x86_64 -o bazbom
        chmod +x bazbom
        ./bazbom scan . --fail-on ${{ inputs.fail-on }}
      shell: bash
```

---

## 🎯 30-Day Deliverables

1. ✅ **Zero vulnerabilities** in BazBOM itself
2. ✅ **SLSA Level 3** provenance on releases
3. ✅ **bazbom-action/scan** published to Marketplace
4. ✅ **Documentation** for GitHub Actions integration
5. ✅ **3 example repositories** using the action

---

## 📊 Week-by-Week Breakdown

### Week 1: Security Foundation
- Mon-Tue: Fix unmaintained dep, add security workflows
- Wed-Thu: SLSA provenance setup
- Fri: Testing and validation

### Week 2: First Action
- Mon-Tue: Create bazbom-action repo, basic scan action
- Wed-Thu: Add SARIF upload, dependency graph integration
- Fri: Documentation and examples

### Week 3: Testing & Polish
- Mon-Tue: Integration tests for the action
- Wed-Thu: Create 3 example repos (Java, Node.js, Rust)
- Fri: Beta testing with friendly users

### Week 4: Launch
- Mon-Tue: Marketplace submission
- Wed: Marketing materials (blog post, tweets)
- Thu: Public launch 🚀
- Fri: Monitor feedback, fix critical issues

---

## 🛠️ Immediate Actions (This Week)

1. **Create GitHub organization:** `bazbom-actions`
2. **Reserve repository names:**
   - bazbom-action/scan
   - bazbom-action/verify
   - bazbom-action/container-scan
   - bazbom-action/policy-check

3. **Set up infrastructure:**
   - GitHub Actions runners (use default)
   - Sigstore keyless signing (free)
   - SLSA generators (free)

4. **Documentation structure:**
   ```
   docs/github-actions/
   ├── README.md (getting started)
   ├── scan-action.md
   ├── examples/
   │   ├── java-maven.yml
   │   ├── node-npm.yml
   │   └── rust-cargo.yml
   └── troubleshooting.md
   ```

---

## 🎨 Design Principles

### Developer Joy
- **< 5 minutes:** First SBOM generated
- **< 10 minutes:** Full setup with vulnerability scanning
- **< 1 minute:** Add to existing project

### Sensible Defaults
```yaml
# This should work out of the box:
- uses: bazbom-action/scan@v1

# Advanced users can customize:
- uses: bazbom-action/scan@v1
  with:
    build-system: maven
    formats: spdx,cyclonedx
    fail-on: high
    policy-file: .bazbom/policy.yml
```

### Zero Config for Common Cases
- Auto-detect build system
- Auto-detect package managers
- Auto-upload to GitHub (SARIF, Dependency Graph)
- Auto-comment on PRs with summary

---

## 📈 Success Criteria

### Week 1
- ✅ cargo audit passes with --deny warnings
- ✅ SLSA provenance generated for test release
- ✅ Security workflow running on every commit

### Week 2
- ✅ bazbom-action/scan repository created
- ✅ Basic action working in test repo
- ✅ SARIF upload functioning

### Week 3
- ✅ 3 language examples working
- ✅ 5+ beta testers onboarded
- ✅ All critical bugs fixed

### Week 4
- ✅ Marketplace listing approved
- ✅ Public announcement published
- ✅ 100+ stars on bazbom-action repo

---

## 🚧 Known Challenges

1. **Binary distribution**
   - Solution: Pre-built binaries on GitHub Releases
   - Checksums + Sigstore signatures
   - Fast download with caching

2. **Action execution time**
   - Solution: Aggressive caching of dependencies
   - Parallel scanning where possible
   - Skip unchanged files

3. **GitHub API rate limits**
   - Solution: Use GraphQL for efficiency
   - Cache API responses
   - Batch operations

---

## 💡 Quick Wins

1. **Dogfooding:** Use BazBOM Action in BazBOM CI/CD
2. **Template workflows:** Users can add with one click
3. **PR comments:** Automatic summary of findings
4. **Badge support:** ![SBOM](badge) in README
5. **Slack notifications:** Webhook integration

---

## 🎯 Next Steps

**Today:**
1. Review and approve V7_ROADMAP.md
2. Create GitHub organization: bazbom-actions
3. Start fixing the unmaintained dependency

**Tomorrow:**
1. Set up security workflows
2. SLSA provenance configuration
3. Create bazbom-action/scan repository skeleton

**This Week:**
1. Ship first working action (even if minimal)
2. Test with one real project
3. Get feedback from 2-3 developers

---

**Let's make supply chain security something developers actually WANT to do!** 🚀
