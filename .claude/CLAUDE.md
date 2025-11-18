# BazBOM Project Memory

**Project:** BazBOM - Developer-friendly security scanner that cuts vulnerability noise by 70-90%
**Tagline:** "Find vulnerabilities that actually matter - cut alert noise by 70-90%"
**Repository:** https://github.com/cboyd0319/BazBOM
**Language:** Rust (29 crates, 700+ tests, zero clippy warnings)
**Primary Maintainer:** Chad Boyd (@cboyd0319)

## What BazBOM Actually Is

BazBOM is a **comprehensive software supply chain security platform** that solves three critical problems:

1. **Actually works with Bazel monorepos** - The ONLY tool with native Bazel support (tested on 5000+ target monorepos)
2. **Cuts noise by 70-90% with reachability analysis** - Shows which vulnerabilities are ACTUALLY exploitable vs just present (237 vulns → 28 that matter)
3. **Developer-friendly output** - Plain English instead of CVE jargon ("Hackers are using this right now" vs "EPSS threshold exceeded")

### Terminology

BazBOM distinguishes between four related but distinct concepts:

- **SBOM (Software Bill of Materials)** - What you have
  - Inventory of packages and versions
  - Generated via `bazbom scan` → SPDX/CycloneDX output
  - Default: Code dependencies only (cleaner)
  - With `--include-cicd`: Also includes GitHub Actions and CI/CD tooling

- **Dependency Graph/Tree** - How you got it
  - Relationships between dependencies (direct vs transitive)
  - Transitive dependency resolution from lockfiles
  - Visualized via `bazbom explore` (TUI) or `bazbom dashboard` (web)

- **SCA (Software Composition Analysis)** - What is known to be vulnerable
  - Vulnerability scanning via OSV/NVD/GHSA databases
  - Result: List of all CVEs present in dependencies
  - Enriched with KEV/EPSS threat intelligence

- **Reachability Analysis** - What is actually dangerous to you
  - Call graph analysis to determine exploitability
  - Result: CVEs filtered to only reachable/exploitable code
  - Reduces alerts by 70-90% through static analysis

**Example workflow:**
```bash
bazbom scan .              # → Generates SBOM + dependency graph
                           # → Runs SCA (finds all CVEs)
bazbom scan -r             # → Same + reachability analysis (finds exploitable CVEs)
bazbom scan --include-cicd # → Includes CI/CD tooling in SBOM
```

### Core Capabilities

**Command Line Tools (11 commands)**:
- `scan` - SBOM generation, vulnerability scanning, plugin integration
- `container-scan` - OCI image scanning with layer attribution
- `policy` - Policy enforcement (Rego/YAML), custom rules, compliance
- `fix` - Upgrade intelligence with breaking change detection and LLM integration
- `license` - License obligations, compatibility, contamination analysis
- `db` - Offline advisory database sync (air-gapped mode)
- `install-hooks` - Git pre-commit hook installation
- `init` - Interactive project setup wizard
- `explore` - TUI-based SBOM exploration (Ratatui)
- `dashboard` - Web-based visualization (Axum/Tokio)
- `team` - Team coordination, CVE assignment, audit logs
- `report` - Executive, compliance, developer, trend reports

**Security Analysis**:
- **🎯 Reachability Analysis** - AST-based call graph for 7 languages (Java, Rust >98%, Go ~90%, JS/TS ~85%, Python ~80%, Ruby ~75%, PHP ~70%)
- **🔐 Vulnerability Scanning** - OSV, NVD, CISA KEV, GHSA with EPSS exploit scoring
- **🕵️ Threat Intelligence** - Malicious package detection, typosquatting alerts
- **🛡️ SAST Integration** - Semgrep and CodeQL for deeper analysis
- **🐳 Container Scanning** - Layer attribution, P0-P4 prioritization, baseline comparison
- **🔧 Universal Auto-Fix** - 9 package managers with multi-CVE grouping, effort scoring (0-100)

**Build System & Language Support**:
- **Build Systems (13)**: Maven, Gradle, Bazel, npm, pip, Go, Cargo, Ruby, PHP, sbt, Ant+Ivy, Buildr, Android
- **JVM Languages (6)**: Java, Kotlin, Scala, Groovy, Clojure, Android
- **Polyglot Ecosystems (6)**: JavaScript/TypeScript, Python, Go, Rust, Ruby, PHP (full monorepo support)

**Developer Experience**:
- **⚡ Zero-Config Workflows** - Quick commands (`check`, `ci`, `pr`, `full`, `quick`) with smart defaults
- **📊 Beautiful Output** - Plain English, Unicode boxes, color-coded, clickable CVE links
- **🎨 TUI Explorer** - Interactive SBOM visualization with ASCII tree view
- **🌐 Web Dashboard** - Axum-based real-time visualization
- **👀 Watch Mode** - Continuous monitoring with auto-rescan on changes
- **📈 Status Dashboard** - Security posture overview, branch comparison
- **🚀 CI/CD Templates** - One-command setup for 5 platforms (GitHub, GitLab, CircleCI, Jenkins, Travis)
- **💻 IDE Integration** - LSP server for real-time vulnerability warnings
- **🔗 Pre-commit Hooks** - Catch issues before commit

**Compliance & Reporting**:
- **📋 Policy Enforcement** - Rego/YAML/CUE policies with validation
- **📑 SBOM Standards** - SPDX 2.3, CycloneDX 1.5 with SLSA v1.1 Level 3 provenance
- **📊 Compliance Reports** - PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST CSF
- **📄 Report Formats** - Executive (1-page), technical, compliance, trend analysis
- **🔍 VEX Support** - False positive suppression with justification tracking

**Advanced Features**:
- **🤖 LLM Integration** - Ollama, Anthropic Claude, OpenAI for fix generation
- **🔄 JAR Bytecode Comparison** - API change detection, method signature analysis
- **⚙️ Config Migration** - Spring Boot 2→3, Log4j 1→2 auto-detection
- **📊 GraphML/DOT Export** - Cytoscape, Gephi, Graphviz visualization
- **☸️ Kubernetes Operator** - CRD-based scanning for cluster deployments
- **🔄 Incremental Scanning** - 10x faster for PR workflows
- **🗄️ Caching** - Deterministic caching for CI/CD optimization

**Production Quality**:
- **30 crates** in unified architecture
- **360+ tests** (100% passing, ≥90% coverage)
- **Zero clippy warnings** - comprehensive code quality
- **Zero vulnerabilities** - cargo audit clean
- **100% memory-safe Rust** - no unsafe code without justification
- **Offline-first** - works fully air-gapped
- **Zero telemetry** - no phoning home, ever

### Performance at Scale
- Small repos (<50 targets): <2 min full, <1 min incremental, ~5s watch
- Medium repos (50-500): <5 min full, <2 min incremental, ~10s watch
- Large repos (500-5K): <15 min full, <5 min incremental, ~20s watch
- Massive repos (5K+): <30 min full, <10 min incremental, ~30s watch
- **6-10x faster** with incremental scanning - tested on real enterprise monorepos

---

## Project Agents and Skills

BazBOM has **8 specialized agents** and **5 automated skills** covering the full development workflow. **Use them proactively** when tasks match their expertise.

### Subagents (Explicit Invocation - 8 total)

#### 1. Bazel Expert (`bazel-expert`)
**When to use:** Bazel build system issues, maven_install.json parsing, dependency detection problems
**Invoke:** `"Use bazel-expert to investigate why Bazel dependencies aren't detected"`
**Expertise:** bazel.rs internals (lines 104-285), both scan paths, BUILD/MODULE.bazel files

#### 2. Reachability Expert (`reachability-expert`)
**When to use:** Reachability false positives/negatives, call graph analysis, 70-90% noise reduction debugging
**Invoke:** `"Use reachability-expert to investigate why this is marked unreachable"`
**Expertise:** 7-language AST analysis (Java, Rust >98%, Go ~90%, JS/TS ~85%, Python ~80%, Ruby ~75%, PHP ~70%), framework detection, entrypoint patterns

#### 3. Container Expert (`container-expert`)
**When to use:** Container scanning, layer attribution, P0-P4 prioritization, EPSS/KEV enrichment
**Invoke:** `"Use container-expert to debug layer attribution issues"`
**Expertise:** OCI image analysis, Dockerfile layer mapping, container reachability analysis (6 languages)

#### 4. Security Analyst (`security-analyst`)
**When to use:** Vulnerability enrichment, EPSS/KEV integration, policy enforcement, compliance reports, threat intelligence
**Invoke:** `"Use security-analyst to explain this CVE prioritization"`
**Expertise:** OSV/NVD/GHSA advisories, malicious package detection, policy engines (Rego/YAML), 7 compliance frameworks

#### 5. Polyglot Expert (`polyglot-expert`)
**When to use:** Multi-language monorepos, lockfile parsing (12 ecosystems), universal auto-fix, workspace detection
**Invoke:** `"Use polyglot-expert to debug npm/pip/cargo detection"`
**Expertise:** Maven, Gradle, npm/Yarn/pnpm, Python, Go, Cargo, Ruby, PHP, unified SBOM generation, 9 package managers

#### 6. Upgrade Intelligence Expert (`upgrade-intelligence-expert`)
**When to use:** Breaking change analysis, transitive upgrade impact, effort scoring, migration guides
**Invoke:** `"Use upgrade-intelligence-expert to analyze this upgrade's breaking changes"`
**Expertise:** Recursive dependency analysis, JAR bytecode comparison, config migration (Spring Boot, Log4j), GitHub release parsing, effort estimation (0-100)

#### 7. Test Runner (`test-runner`)
**When to use:** Comprehensive test suites, regression checking, performance testing, SBOM validation
**Invoke:** `"Use test-runner to validate this fix against all test repositories"`
**Capabilities:** Rust unit tests (360+), integration tests (5 repos), performance profiling

#### 8. Code Reviewer (`code-reviewer`)
**When to use:** PR reviews, code quality checks, BazBOM pattern enforcement, security audits
**Invoke:** `"Use code-reviewer to review my changes"`
**Focus:** Rust best practices, tracing logging, error handling, test coverage, both scan paths consistency

---

### Skills (Automatic Activation - 5 total)

#### 1. SBOM Validator (`sbom-validator`)
**Activates when:** "Is this SBOM valid?", "Check the generated SBOM", "How many packages?"
**Does:** Validates SPDX 2.3/CycloneDX 1.5 structure, verifies PURLs, checks package completeness

#### 2. Reachability Validator (`reachability-validator`)
**Activates when:** "Is reachability correct?", "Validate call graph", "Check reachability accuracy"
**Does:** Validates entrypoint detection, checks reduction rates (45-90%), identifies false positives/negatives

#### 3. Vulnerability Reporter (`vulnerability-reporter`)
**Activates when:** "Explain this CVE", "Why is this P0?", "Show exploit details", "How do I fix CVE-X?"
**Does:** Deep-dive CVE analysis with EPSS/KEV, exploit links (ExploitDB, Metasploit), remediation guidance, prioritization rationale

#### 4. Compliance Checker (`compliance-checker`)
**Activates when:** "Check PCI-DSS compliance", "Generate HIPAA report", "Validate policy"
**Does:** Validates 7 frameworks (PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST), generates audit-ready reports

#### 5. Performance Profiler (`performance-profiler`)
**Activates when:** "Why is this scan slow?", "Performance test", "How much memory?"
**Does:** Analyzes execution time, identifies bottlenecks (maven_install parsing, SBOM gen, reachability), optimization recommendations

---

### Coverage Map

| Feature Area | Agent | Skills |
|--------------|-------|--------|
| **Build Systems** | bazel-expert, polyglot-expert | sbom-validator |
| **Reachability** | reachability-expert | reachability-validator |
| **Container Security** | container-expert | vulnerability-reporter |
| **Vulnerability Management** | security-analyst | vulnerability-reporter, compliance-checker |
| **Upgrade Intelligence** | upgrade-intelligence-expert | - |
| **Code Quality** | code-reviewer | - |
| **Testing** | test-runner | performance-profiler |

### Usage Patterns

**By Development Phase:**
- **Investigation:** Use reachability-expert, bazel-expert, polyglot-expert, container-expert
- **Implementation:** Code, iterate
- **Review:** Use code-reviewer to check quality
- **Testing:** Use test-runner to validate
- **Validation:** Skills auto-activate (sbom-validator, reachability-validator, etc.)

**By Problem Type:**
- **0 packages detected:** Use bazel-expert or polyglot-expert
- **Reachability issues:** Use reachability-expert
- **Container scanning:** Use container-expert
- **CVE questions:** Use security-analyst or ask to trigger vulnerability-reporter skill
- **Upgrade impact:** Use upgrade-intelligence-expert
- **Performance:** Ask to trigger performance-profiler skill
- **Compliance:** Ask to trigger compliance-checker skill

**Chaining Workflows:**
```
"Use reachability-expert to analyze the issue,
then test-runner to validate the fix,
then code-reviewer to check code quality"

"Use upgrade-intelligence-expert to analyze the upgrade,
then security-analyst to check for new vulnerabilities"
```

**Reference:** See `docs/AGENTS_AND_SKILLS_GUIDE.md` for complete documentation

---

## Build and Development Commands

### Building
```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p bazbom

# Clean build
cargo clean && cargo build --release
```

### Installation
```bash
# Install from source (preferred during development)
cargo install --path crates/bazbom --force

# Binary location after build
target/release/bazbom

# Standard install location
/usr/local/bin/bazbom
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p bazbom

# Run tests with logging
RUST_LOG=debug cargo test

# Integration tests
cd ~/Documents/BazBOM_Testing
./test-bazel-fix.sh

# Test on real repos
cd ~/Documents/BazBOM_Testing/real-repos/bazel-examples
bazbom scan .
```

### Linting and Format
```bash
# Check code
cargo check

# Run clippy
cargo clippy

# Fix warnings
cargo fix

# Check outdated dependencies
cargo outdated

# Security audit
cargo audit
```

---

## Architecture and Code Organization

### Crate Structure
```
crates/
├── bazbom/               # Main CLI binary
│   ├── src/
│   │   ├── main.rs      # CLI entry point
│   │   ├── scan.rs      # Legacy scan implementation
│   │   ├── bazel.rs     # Bazel dependency extraction (CRITICAL)
│   │   ├── scan_orchestrator.rs  # Orchestrated scans
│   │   └── commands/    # Command handlers
├── bazbom-core/         # Core types and build detection
├── bazbom-formats/      # SBOM format generation (SPDX, CycloneDX)
├── bazbom-polyglot/     # Multi-language ecosystem support
├── bazbom-advisories/   # Vulnerability data
└── [other crates]/      # Specialized functionality
```

### Key Files and Their Purpose
- **`bazel.rs`** - Maven dependency extraction from maven_install.json (lines 104-285)
- **`scan.rs`** - Legacy scan path with Bazel handling (lines 34-87)
- **`scan_orchestrator.rs`** - Orchestrated scans with Bazel handling (lines 1214-1269)
- **`commands/scan.rs`** - CLI command handler and smart defaults

### Critical Code Patterns

#### Bazel Detection Pattern
```rust
if system == bazbom_core::BuildSystem::Bazel {
    let maven_install_json = workspace.join("maven_install.json");
    if maven_install_json.exists() {
        match crate::bazel::extract_bazel_dependencies(&workspace, &output) {
            Ok(graph) => {
                // Convert to SPDX and write SBOM
                let spdx_doc = graph.to_spdx(workspace_name);
                // ...
            }
            Err(e) => {
                // Fall back to stub SBOM with helpful message
            }
        }
    }
}
```

#### Logging Pattern
```rust
// Use tracing, not eprintln!
tracing::info!("Successfully extracted {} packages", count);
tracing::debug!("Processing file: {:?}", path);
tracing::warn!("Optional feature unavailable: {}", feature);

// Enable with: RUST_LOG=debug bazbom scan .
```

---

## Coding Standards

### Rust Style
- **Format:** Use `rustfmt` defaults (2-space indent, 100 char line length)
- **Linting:** All `clippy` warnings must be addressed
- **Error Handling:** Use `anyhow::Result` for functions that can fail
- **Async:** Use `tokio` runtime for async operations
- **Logging:** Use `tracing` crate, NOT `eprintln!` or `println!` for debug output

### Error Messages
- Always provide context: `.context("failed to parse file")?`
- Give users actionable hints: "Run 'bazel run @maven//:pin' to generate maven_install.json"
- Fall back gracefully: Write stub SBOM if extraction fails
- Use proper log levels: `warn!` for recoverable, `error!` for critical

### Documentation
- All public APIs must have doc comments
- Include examples in doc comments where helpful
- Update user-facing docs (docs/*.md) when changing behavior
- Keep CHANGELOG.md updated with all notable changes

---

## Common Workflows

### Making Changes to Bazel Support

1. **Understand the flow:** Scan command → Build system detection → Bazel extraction
   - **TIP:** Use `bazel-expert` agent for deep analysis of existing flow
2. **Modify both paths:** Update `scan.rs` AND `scan_orchestrator.rs`
3. **Keep them consistent:** Same logic in both code paths
   - **TIP:** Use `code-reviewer` agent to verify consistency
4. **Test extensively:** Run against real repos with various configs
   - **TIP:** Use `test-runner` agent to run full test suite
5. **Update docs:** BAZEL.md and FIXES_SUMMARY.md

**Recommended agent workflow:**
```
1. "Use bazel-expert to explain the current Bazel detection flow"
2. [Make your changes]
3. "Use code-reviewer to verify both scan paths are updated consistently"
4. "Use test-runner to validate against all test repositories"
```

### Adding New Build System Support

1. Add variant to `BuildSystem` enum in `bazbom-core/src/lib.rs`
2. Add detection logic in `detect_build_system()`
3. Create extraction module (e.g., `gradle.rs`, `maven.rs`)
4. Add handling in `scan.rs` and `scan_orchestrator.rs`
5. Write tests and update documentation

**TIP:** Use `code-reviewer` agent to ensure new build system follows BazBOM patterns

### Debugging Scans

**First step:** Use `bazel-expert` agent for Bazel-specific issues, or debug manually:



```bash
# Enable verbose logging
RUST_LOG=debug bazbom scan .

# Specific module logging
RUST_LOG=bazbom::bazel=trace bazbom scan .

# Check what was detected
bazbom scan . 2>&1 | grep "detected\|found\|system"

# Validate SBOM output
jq '.packages | length' sbom.spdx.json
jq '.packages[0:3]' sbom.spdx.json
```

### Release Process

1. Update version in `Cargo.toml` files
2. Update `CHANGELOG.md` with release notes
3. Run full test suite: `cargo test --all`
4. Build release binary: `cargo build --release`
5. Test on real repositories
6. Tag release: `git tag v6.5.0 && git push --tags`
7. Update documentation if needed

---

## Testing Infrastructure

### Test Repositories Location
`~/Documents/BazBOM_Testing/`

### Key Test Scripts
- **`test-bazel-fix.sh`** - Automated validation (requires bash 4+)
- **`test-all-repos.sh`** - Comparison across all repos
- **`simple-test.sh`** - Quick smoke tests
- **`full-stress-test.sh`** - Performance stress testing

### Expected Test Results
- bazel-examples: 59 packages
- Synthetic monorepo: 2,067 packages
- bzlmod-examples: 0 (no maven_install.json)
- bazel-monorepo: 0 (no maven_install.json)

### Running Tests
```bash
# Automated test suite
cd ~/Documents/BazBOM_Testing
BAZBOM_BIN=/path/to/bazbom ./test-bazel-fix.sh

# Quick manual test
cd ~/Documents/BazBOM_Testing/real-repos/bazel-examples
bazbom scan --format spdx -o /tmp/test
jq '.packages | length' /tmp/test/sbom.spdx.json
```

---

## Important Historical Context

### Critical Bug Fix (2025-11-18) - Bazel Dependency Detection
**Context:** BazBOM supports 13 build systems. This bug affected ONLY Bazel (1 of 13). All other build systems (Maven, Gradle, npm, pip, Go, Cargo, Ruby, PHP, etc.) were unaffected.

**Problem:** Bazel projects detected but returned 0 packages - breaking ALL downstream features for Bazel users (vulnerability scanning, reachability analysis, SBOM generation, auto-fix)

**Root Cause:** `bazel.rs` extraction code existed but was never called during scans

**Impact:** Without package detection, BazBOM couldn't scan vulnerabilities, analyze reachability, or provide any security value for Bazel users. This was a critical blocker for Bazel monorepo adoption.

**Solution:** Added Bazel handling to both scan paths:
- Legacy path: `scan.rs` lines 34-87
- Orchestrator path: `scan_orchestrator.rs` lines 1214-1269

**Testing:** Validated on 5 repositories (59 to 2,067 packages) - now functioning end-to-end

**Documentation:** See `docs/FIXES_SUMMARY.md` for full technical details

**Important:** This was about fixing the Bazel-specific dependency extraction pipeline. The rest of BazBOM's 11 commands, 7-language reachability analysis, container scanning, policy enforcement, compliance reporting, etc. were all working - this bug only affected Bazel projects specifically.

### Key Lessons Learned
1. Always check if detection code is actually called during scans
2. Test with both small and large repositories
3. Use `tracing` for debugging, not manual debug statements
4. Maintain consistency between scan paths (legacy + orchestrator)
5. Document architecture decisions immediately
6. Dependency extraction is the foundation - without it, ALL features break

---

## Dependencies and External Tools

### Required for Development
- Rust 1.70+ (stable toolchain)
- Cargo
- Git

### Optional but Recommended
- `jq` - JSON manipulation for testing
- `bazel` - For testing Bazel projects
- Python 3.8+ with `pyyaml`, `tqdm` - For synthetic repo generation

### External Services (Optional)
- deps.dev API - Dependency enrichment
- CISA KEV catalog - Known exploited vulnerabilities
- EPSS API - Exploit prediction scores

---

## Known Issues and Workarounds

### maven_install.json in Non-Root Locations
**Issue:** Some projects put maven_install.json in subdirectories (e.g., `3rdparty/`)
**Workaround:** Currently not supported, manual SBOM generation needed
**Future:** Could add recursive search or configuration option

### Test Script Bash Compatibility
**Issue:** `test-bazel-fix.sh` requires bash 4+ (macOS ships with 3.2)
**Workaround:** Install newer bash: `brew install bash`, run with `/usr/local/bin/bash`
**Alternative:** Use manual testing commands shown in README

### Smart Defaults Auto-Reachability
**Issue:** Small repos auto-enable reachability analysis which can be slow
**Workaround:** `export BAZBOM_NO_SMART_DEFAULTS=1` before scanning
**Toggle:** `--fast` flag to explicitly disable

---

## Performance Expectations

| Repository Size | Packages | Scan Time | Memory Usage |
|----------------|----------|-----------|--------------|
| Small (<10MB) | <100 | <1s | ~50MB |
| Medium (10-50MB) | 100-1K | 1-3s | ~100MB |
| Large (50-100MB) | 1K-5K | 3-10s | ~150MB |
| Huge (>100MB) | 5K+ | 10-30s | ~200MB |

*Times measured on Apple Silicon M1/M2*

---

## Communication Style for This Project

- **Be direct and technical** - This is a production-ready security platform with 700+ tests
- **Explain trade-offs** - Help Chad understand architectural decisions and security implications
- **Show examples** - Code snippets are better than abstract descriptions
- **Reference docs** - Point to existing docs when relevant
- **Be proactive** - Suggest improvements and catch issues early
- **Security-focused** - BazBOM is about vulnerability detection, not just SBOM generation
- **UX matters** - Developer experience is a core feature, not an afterthought

---

## Quick Reference Commands

```bash
# Full development cycle
cargo clean && cargo build --release && cargo test

# Install and test
cargo install --path crates/bazbom --force
cd ~/Documents/BazBOM_Testing/real-repos/bazel-examples
bazbom scan .

# Check SBOM output
jq '.packages | length' sbom.spdx.json

# Enable debug logging
RUST_LOG=debug bazbom scan .

# Run automated tests
cd ~/Documents/BazBOM_Testing
BAZBOM_BIN=/path/to/bazbom ./test-bazel-fix.sh
```

---

## Quick Agent/Skill Reference

### Get Expert Help
```
"Use bazel-expert to investigate Bazel detection issues"
"Use test-runner to validate across all test repos"
"Use code-reviewer to check my changes"
```

### Automatic Validation
Just ask naturally and skills activate:
- "Is this SBOM valid?" → SBOM Validator activates
- "Why is this slow?" → Performance Profiler activates

**See:** Section "Project Agents and Skills" at top of this file for complete details

---

## Documentation Structure

- **`docs/ARCHITECTURE.md`** - System architecture overview
- **`docs/BAZEL.md`** - Bazel integration guide (user-facing)
- **`docs/FIXES_SUMMARY.md`** - Technical details of fixes
- **`docs/IMPLEMENTATION_SUMMARY.md`** - Implementation overview
- **`docs/AGENTS_AND_SKILLS_GUIDE.md`** - Complete agent/skill documentation
- **`docs/MEMORY_GUIDE.md`** - Memory system guide
- **`CHANGELOG.md`** - Version history and notable changes
- **`CONTRIBUTING.md`** - Contribution guidelines
- **`README.md`** - Project overview and quick start

---

## Import References

For additional context, team members can create personal instruction files:
```markdown
@~/Documents/BazBOM_Testing/README.md
@~/.config/bazbom/personal-preferences.md
```

---

**Last Updated:** 2025-11-18
**By:** Claude Code (automated)
**Version:** Enhanced with agents, skills, and comprehensive memory system

