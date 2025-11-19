# BazBOM Project Memory

**Project:** BazBOM - Developer-friendly security scanner that cuts vulnerability noise by 70-90%
**Repository:** https://github.com/cboyd0319/BazBOM
**Language:** Rust (29 crates, 700+ tests)
**License:** Business Source License (BSL) 1.1

## What BazBOM Is

Comprehensive software supply chain security platform solving three problems:

1. **Native Bazel support** - Only tool with real Bazel monorepo support (tested on 5000+ targets)
2. **Reachability analysis** - Shows which vulnerabilities are ACTUALLY exploitable (cuts noise 70-90%)
3. **Developer-friendly** - Plain English instead of CVE jargon

**Ecosystems:** 8 languages validated (Python, Ruby, Java/Maven, npm, Go, Rust, PHP, Gradle)
**SBOM Formats:** SPDX 2.3, CycloneDX 1.5
**Build Systems:** 13 supported (Maven, Gradle, Bazel, npm, pip, Go, Cargo, Ruby, PHP, sbt, Ant+Ivy, Buildr, Android)

---

## 🚨 CRITICAL STATUS (2025-11-18)

### Recent Validation Status

**Phases 1-4 COMPLETE** ✅
- ✅ Phase 1: Vulnerability detection validated across 7 ecosystems (701 vulns detected)
- ✅ Phase 2: SBOM formats validated (5 formats working)
- ✅ Phase 3: SBOM content flags (legacy-only, accepted)
- ✅ Phase 4: Reachability integration COMPLETE (100% noise reduction validated on 166 vulns across 3 real apps)

**Phase 5: Bazel Flag Validation - NEXT** ⏳
- 4 Bazel flags implemented but untested
- Need to validate they actually work end-to-end
- See `docs/COMPREHENSIVE_TESTING_PLAN.md` Phase 5

### Test Coverage Reality Check

**Validated Ecosystems:** 8/8 (100%)
- Python, Ruby, Java/Maven, npm, Go, Rust, PHP, Gradle - ALL validated

**Test Repositories:**
- 11 vulnerable test projects created (701 vulnerabilities total)
- 3 real-world apps tested (django.nV, rails_5_2_sample, WebGoat)
- bazel-examples (59 packages)

**See:** `docs/COMPREHENSIVE_TESTING_PLAN.md` for full testing status

---

## Terminology

**SBOM** - What you have (inventory of packages)
**SCA** - What is known to be vulnerable (CVE scanning)
**Reachability** - What is actually dangerous (exploitability analysis)

```bash
bazbom scan .              # SBOM + SCA
bazbom scan -r             # + Reachability (70-90% noise reduction)
bazbom scan --include-cicd # + CI/CD tooling in SBOM
```

---

## Core Capabilities

**Commands:** scan, container-scan, policy, fix, license, db, install-hooks, init, explore, dashboard, team, report

**Security:**
- Reachability analysis (7 languages: Java, Rust >98%, Go ~90%, JS/TS ~85%, Python ~80%, Ruby ~75%, PHP ~70%)
- Vulnerability scanning (OSV, NVD, CISA KEV, GHSA, EPSS)
- SAST integration (Semgrep, CodeQL)
- Container scanning (layer attribution, P0-P4 prioritization)

**Developer Experience:**
- Zero-config workflows (quick commands: check, ci, pr, full, quick)
- Beautiful output (plain English, color-coded, clickable CVE links)
- TUI explorer + web dashboard
- Watch mode + status dashboard
- CI/CD templates (GitHub, GitLab, CircleCI, Jenkins, Travis)
- IDE integration (LSP) + pre-commit hooks

---

## Project Agents (8 Available)

**Usage:** Agents have dedicated files in `.claude/agents/`. Invoke explicitly when needed:
- `bazel-expert` - Bazel build system issues
- `reachability-expert` - Reachability false positives/negatives
- `container-expert` - Container scanning, layer attribution
- `security-analyst` - Vulnerability enrichment, policy enforcement
- `polyglot-expert` - Multi-language monorepos, lockfile parsing
- `upgrade-intelligence-expert` - Breaking change analysis, transitive impact
- `test-runner` - Comprehensive test suites, regression checking
- `code-reviewer` - PR reviews, code quality checks

**Example:** `"Use bazel-expert to investigate why Bazel dependencies aren't detected"`

**See:** `docs/AGENTS_AND_SKILLS_GUIDE.md` for complete documentation

---

## Key Files and Locations

### Crate Structure
```
crates/
├── bazbom/               # Main CLI
│   ├── src/bazel.rs      # Bazel dependency extraction (CRITICAL)
│   ├── src/scan.rs       # Legacy scan path
│   └── src/scan_orchestrator.rs  # Orchestrated scans
├── bazbom-core/          # Core types, build detection
├── bazbom-formats/       # SBOM generation (SPDX, CycloneDX)
├── bazbom-polyglot/      # Multi-language ecosystem support
└── bazbom-advisories/    # Vulnerability data
```

### Documentation
- `docs/COMPREHENSIVE_TESTING_PLAN.md` - Testing status (Phases 1-4 complete, 5-15 pending)
- `docs/BAZEL_FEATURE_AUDIT.md` - Bazel feature gap analysis (4/21 features, 19% parity)
- `docs/BENCHMARKS_AND_METRICS.md` - Performance metrics
- `docs/BAZEL.md` - Bazel integration guide (user-facing)
- `docs/ARCHITECTURE.md` - System architecture
- `docs/AGENTS_AND_SKILLS_GUIDE.md` - Agent/skill documentation

### Test Repositories
```
~/Documents/BazBOM_Testing/
├── real-repos/
│   ├── bazel-examples/           # 59 packages (Bazel)
│   ├── vulnerable-npm-test/      # 23 vulns (npm)
│   ├── django.nV/                # 35 vulns (Python real app)
│   ├── rails_5_2_sample/         # 99 vulns (Ruby real app)
│   └── WebGoat/                  # 32 vulns (Java real app)
└── vulnerable-projects/
    ├── vulnerable-python/        # 239 vulns
    ├── vulnerable-go/            # 56 vulns
    ├── vulnerable-rust/          # 23 vulns
    ├── vulnerable-ruby/          # 80 vulns
    ├── vulnerable-php/           # 60 vulns
    ├── vulnerable-maven/         # 107 vulns
    └── vulnerable-gradle/        # 136 vulns
```

---

## Build and Test Commands

### Building
```bash
cargo build --release                      # Production build
cargo install --path crates/bazbom --force # Install locally
```

### Testing
```bash
cargo test                                 # All unit tests
RUST_LOG=debug cargo test                 # With logging

# Integration tests
cd ~/Documents/BazBOM_Testing
BAZBOM_BIN=~/Documents/GitHub/BazBOM/target/release/bazbom ./test-bazel-fix.sh

# Quick manual test
cd ~/Documents/BazBOM_Testing/real-repos/bazel-examples
bazbom scan .
jq '.packages | length' sbom.spdx.json
```

### Linting
```bash
cargo clippy                               # Check warnings
cargo fix                                  # Auto-fix
cargo audit                                # Security audit
```

---

## Debugging

```bash
# Enable verbose logging
RUST_LOG=debug bazbom scan .

# Specific module logging
RUST_LOG=bazbom::bazel=trace bazbom scan .

# Check SBOM output
jq '.packages | length' sbom.spdx.json
jq '.packages[0:3]' sbom.spdx.json

# Check SARIF vulnerabilities
jq '.runs[0].results | length' findings/sca.sarif
jq '.runs[0].results[0]' findings/sca.sarif
```

---

## Critical Historical Context

### Major Bug Fixes (2025-11-18)

**Bug 1: Bazel Dependency Detection**
- **Problem:** 0 packages detected for Bazel projects
- **Root Cause:** `bazel.rs` extraction code existed but never called
- **Fix:** Added Bazel handling to both scan paths (scan.rs, scan_orchestrator.rs)
- **Status:** ✅ Fixed, validated on 5 repos (59-2,067 packages)

**Bug 2: Vulnerability Detection (ALL Polyglot Ecosystems)**
- **Problem:** Vulnerabilities detected but discarded before SARIF output
- **Root Cause:** Orchestrator called `scan_directory_sbom_only()` instead of `scan_directory()`
- **Fix:** Save polyglot vulnerabilities to intermediate JSON, load in SCA analyzer
- **Status:** ✅ Fixed, validated across all 7 polyglot ecosystems (701 vulns detected)

**Bug 3: Reachability Integration**
- **Problem:** Reachability analysis ran but results didn't reach SARIF
- **Root Cause:** Missing `polyglot-sbom.json` write in orchestrator
- **Fix:** Added 8 lines to write reachability data to intermediate file
- **Status:** ✅ Fixed, validated with 99.6% noise reduction (406/408 unreachable)

**Key Lessons:**
1. Test end-to-end, not just components
2. Validate ALL claimed ecosystems
3. Track data through entire pipeline: detection → storage → output
4. Test with vulnerable projects (clean projects hide bugs)
5. Use `tracing` for debugging, not println!

---

## Coding Standards

- **Logging:** Use `tracing` crate (never `eprintln!` for debug)
- **Error Handling:** `anyhow::Result` with context
- **Testing:** Validate end-to-end flows
- **Consistency:** Keep both scan paths synchronized (scan.rs + scan_orchestrator.rs)
- **Documentation:** Update user-facing docs when changing behavior

---

## Performance Expectations

| Repository Size | Packages | Scan Time | Memory |
|----------------|----------|-----------|--------|
| Small (<10MB) | <100 | <1s | ~50MB |
| Medium (10-50MB) | 100-1K | 1-3s | ~100MB |
| Large (50-100MB) | 1K-5K | 3-10s | ~150MB |
| Huge (>100MB) | 5K+ | 10-30s | ~200MB |

*Apple Silicon M1/M2*

---

## Next Steps

**Phase 5: Bazel Flag Validation (NEXT)**
- Test `--bazel-targets-query`
- Test `--bazel-targets`
- Test `--bazel-affected-by-files`
- Test `--bazel-universe`
- Create 5 Bazel test repositories (basic, incremental, non-root, unbuilt, Bazel 7+)

**See:** `docs/COMPREHENSIVE_TESTING_PLAN.md` Phase 5 for details

---

**Last Updated:** 2025-11-18
**Status:** Phases 1-4 COMPLETE, Phase 5 pending
