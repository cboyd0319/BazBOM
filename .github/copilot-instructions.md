# Copilot Instructions — BazBOM

Purpose: Provide clear, enforceable guidance so changes remain aligned with BazBOM’s mission, security posture, testing rigor, and documentation standards.

## Mission & Non‑Negotiables

- **POLYGLOT SBOM/SCA TOOL**: BazBOM supports 13+ languages across JVM and polyglot ecosystems with world-class reachability analysis
- **Supported JVM Languages**: Java, Kotlin, Scala, Groovy, Clojure, Android
- **Supported Polyglot Languages**: JavaScript/TypeScript, Python, Go, Rust, Ruby, PHP
- **Supported Build Systems**: Maven, Gradle, Bazel, sbt, Ant, Buildr, npm (yarn/pnpm), pip (poetry/pipenv), Go modules, Cargo, Bundler, Composer
- World‑class SBOM, SCA, and dependency graph analysis with 70-90% noise reduction via reachability analysis for 7 languages.
- Private-by-default: 100% privacy, zero telemetry. Offline-first operation is required.
- Memory‑safe distribution: Rust‑first single binary; OPAL (JVM) helper for reachability. Avoid unsafe; no embedded Python in shipped binaries.
- Policy‑as‑code at the core: YAML (plus optional Rego/CUE), VEX auto‑application, CI gating.
- Explainability and remediation: default “suggest‑only” with “why fix this?”; safe `--apply` opens PRs and runs checks.
- Deterministic, reproducible outputs with signed artifacts and SLSA provenance.

CRITICAL Repo Rules (must follow)
- **POLYGLOT SUPPORT**: BazBOM supports 13+ languages including JVM (Java, Kotlin, Scala, Groovy, Clojure, Android) and polyglot ecosystems (JavaScript/TypeScript, Python, Go, Rust, Ruby, PHP). All language support must maintain high quality standards.
- Zero emojis in code, ever. Do not add emojis to source files, generated code, or code comments. Code examples in docs that users might copy/paste must also be emoji‑free.
- Avoid doc sprawl. Do not create a new doc for every small task. Prefer updating canonical docs under `docs/`. Create new documents only when a clear gap exists, and then link them from `docs/README.md`.

Primary audience: Enterprise/AppSec engineers; secondary: Platform/DevSecOps; tertiary: JVM developers.
Target OS: macOS → Linux → Windows.

## Architecture Snapshot

- **Rust workspace**: 30 production crates (v6.5) + 3 planned for v6.8:
  - Core: `bazbom` (CLI), `bazbom-core`, `bazbom-formats`, `bazbom-advisories`, `bazbom-policy`, `bazbom-graph`
  - Polyglot: `bazbom-polyglot` (multi-language support)
  - Reachability: `bazbom-java-reachability` (JVM bytecode), `bazbom-js-reachability`, `bazbom-python-reachability`, `bazbom-go-reachability`, `bazbom-rust-reachability`, `bazbom-ruby-reachability`, `bazbom-php-reachability`
  - Intelligence: `bazbom-upgrade-analyzer`, `bazbom-depsdev`, `bazbom-ml`
  - UI: `bazbom-dashboard`, `bazbom-tui`, `bazbom-lsp`
  - Infrastructure: `bazbom-containers`, `bazbom-operator`, `bazbom-cache`, `bazbom-threats`, `bazbom-reports`, `bazbom-tool-verify`, `bazbom-verify`
  - Enterprise (v7.0-beta): `bazbom-auth`, `bazbom-crypto`
  - **Planned for v6.8 (Q2 2026)**: `bazbom-jira` (~2,500 LOC), `bazbom-github` (~3,000 LOC), Intelligence Hub component (~1,500 LOC)
- **Reachability Analysis**: Language-specific AST/bytecode analysis for 7 languages (70-90% noise reduction)
  - JVM: OPAL-based bytecode analysis
  - JavaScript/TypeScript: SWC-based AST parsing
  - Python: RustPython parser with framework detection
  - Go: tree-sitter with goroutine tracking
  - Rust: syn parser with trait tracking (>98% accuracy)
  - Ruby: tree-sitter with Rails/metaprogramming support
  - PHP: tree-sitter with Laravel/Symfony support
- Shading detection: Maven Shade and Gradle Shadow plugin parsing; class fingerprinting with Blake3 hashing.
- **Build integrations**:
  - **JVM**: Maven plugin, Gradle plugin, Bazel aspects (java_*/kotlin_*/scala_*), sbt, Ant+Ivy, Buildr
  - **JavaScript**: npm (package-lock.json, yarn.lock, pnpm-lock.yaml), Yarn workspaces, pnpm workspaces
  - **Python**: pip (requirements.txt), poetry (poetry.lock), pipenv (Pipfile.lock), PDM
  - **Go**: go.mod/go.sum with replace/indirect support
  - **Rust**: Cargo.toml/Cargo.lock with workspace support
  - **Ruby**: Bundler (Gemfile.lock) with Rails support
  - **PHP**: Composer (composer.lock) with Laravel/Symfony support
- Intelligence: OSV/NVD/GHSA + KEV + EPSS; canonical severity + P0–P4 priority.
- Outputs: SPDX 2.3 (primary), CycloneDX 1.5 (optional), SARIF 2.1.0, CSAF VEX, CSV.

## Version Planning & Roadmap

**Current Version:** v6.5 (stable)
**Next Major Release:** v6.8 - Full DevSecOps Automation Platform (Q2 2026)

**v6.8 Planning Documentation** (Nov 2025 - COMPLETE):
- `docs/development/versions/6.8/README.md` - Overview and index
- `docs/development/versions/6.8/jira-integration-plan.md` - Complete Jira + GitHub PR automation plan (9 feature categories)
- `docs/development/versions/6.8/technical-specifications.md` - Architecture and API specs
- `docs/development/versions/6.8/implementation-roadmap.md` - 20-week timeline (Q1-Q2 2026)
- `docs/development/versions/6.8/integration-analysis.md` - 48 integration points across 8 categories
- `docs/development/versions/6.8/pr-template-complete.md` - Complete PR template with ALL 14+ intelligence modules

**v6.8 Key Features (Planned):**
- Automatic Jira ticket creation with full intelligence from ALL BazBOM modules
- Automatic GitHub PR creation with AI-powered fixes and complete context
- Bidirectional sync: Jira ↔ BazBOM ↔ GitHub (three-way synchronization)
- Multi-PR orchestration for batch remediation across repositories
- Auto-merge capabilities (optional, with safety controls and test gates)
- Intelligence Hub aggregating ALL 14+ BazBOM intelligence modules
- 90% reduction in manual remediation work
- 80% faster time-to-fix for automated-eligible vulnerabilities
- Complete automation loop: Scan → Ticket → PR → Review → Merge → Close

**When working on v6.8:**
- Reference planning docs in `docs/development/versions/6.8/`
- Follow the 20-week implementation roadmap (7 phases)
- Ensure ALL 14+ intelligence modules are integrated into PRs and tickets
- Maintain tri-directional sync (Jira ↔ BazBOM ↔ GitHub)
- Follow security considerations outlined in planning docs

## Documentation Policy (must follow)

- All canonical docs live under `docs/` only.
- Allowed root stubs (minimal link‑only): `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `MAINTAINERS.md`.
- This file (`.github/copilot-instructions.md`) is an operational exception.
- Documentation standards:
  - markdownlint + Vale enforced; active voice; consistent terminology; relative links.
  - ADRs in `docs/ADR/`; images in `docs/images/`; diagrams in `docs/diagrams/`.
  - Follow tech writer persona guidance in `docs/tech_writer_persona.md`

## Testing & Coverage Requirements

- Repo‑wide coverage ≥ 90%; critical modules ~100% (merge engine, policy, exporters, graph normalizer).
- Branch coverage on; coverage enforced in CI (fail‑under gates).
- Python (transition period): pytest + pytest‑cov.
- Rust: `cargo test` (or nextest), coverage via tarpaulin (Linux) or grcov/llvm‑profile; enforce thresholds.
- Golden files for schema outputs; property‑based tests for normalization/dedup; fuzz parsers; perf benchmarks.

## CI Rules & Required Checks

- Coverage job with fail‑under thresholds (repo ≥90%; critical pkgs ≥98%).
- Docs checks: markdownlint, Vale, link check; verify docs are under `docs/` except allowlisted stubs and this file.
- Security: CodeQL, dependency review, supply chain policies, signed releases.
- Build: reproducible outputs; schema validation (SPDX/CycloneDX/SARIF/VEX) in tests.

## Single Source of Truth

- Capabilities Reference: `docs/reference/capabilities-reference.md` (complete feature catalog).
- Root README: overview, quickstart, and links into `docs/` (don’t duplicate).
- All user/developer docs: under `docs/` (reference, guides, ADRs, testing, copilot).

## When Adding or Changing Features

1) Update reference and guides:
   - `docs/reference/capabilities-reference.md`
   - `docs/user-guide/usage.md` for CLI changes; examples for Maven/Gradle/Bazel
   - `docs/operations/provenance.md`, `docs/security/vex.md`, `docs/operations/performance.md` as needed
   - `docs/integrations/ide/ide-integration.md` for IDE/remediation features
2) Update root `README.md` where applicable (Features bullets, Quickstart snippets, performance table).
3) If outputs/schemas change: bump schema versions; update JSON Schemas and golden tests.
4) If GitHub Action changes: update `action.yml`, README snippets, and docs examples.
5) Run validation:
   ```bash
   pre-commit run --all-files
   pytest -q  # until fully ported to Rust
   cargo test --all --locked
   # For IDE plugins:
   cd crates/bazbom-vscode-extension && npm test
   cd crates/bazbom-intellij-plugin && ./gradlew test
   ```

## Build Systems & Examples Checklist

- **JVM Build Systems**:
  - Ant (build.xml), Maven (pom.xml), Gradle (build.gradle/.kts; Android variants)
  - Bazel (WORKSPACE/MODULE.bazel; rules_jvm_external; aspects), sbt (build.sbt), Buildr (buildfile, Rakefile)
- **Polyglot Package Managers**:
  - npm (package.json, package-lock.json, yarn.lock, pnpm-lock.yaml)
  - Python (requirements.txt, poetry.lock, Pipfile.lock, pyproject.toml)
  - Go (go.mod, go.sum), Rust (Cargo.toml, Cargo.lock)
  - Ruby (Gemfile, Gemfile.lock), PHP (composer.json, composer.lock)
- Include: offline mode, VEX flow, GitHub Action, shaded/fat JAR examples, polyglot monorepo examples.

## Security & Supply Chain Requirements

**Supply Chain Security:**
- SLSA Level 3 provenance; Sigstore keyless signing; checksums.
- Zero telemetry; explicit `bazbom db sync` for advisory updates.
- Policy‑as‑code (YAML; optional Rego/CUE). VEX auto‑generation on unreachable when policy allows.
- CWE mapping, SARIF 2.1.0 validation, SPDX 2.3 and CycloneDX 1.5 validation.

**Security Module Architecture:**
- Security utilities in `crates/bazbom/src/security/`
  - `log_sanitizer.rs`: Prevents log injection attacks (sanitize newlines, ANSI codes, control characters)
  - `audit_log.rs`: Comprehensive security event logging in JSON Lines format
- Tool verification in `crates/bazbom/src/toolchain/verify.rs`
  - SHA-256 checksum verification for external tools
  - `ToolChecksums` struct for integrity validation

**Dashboard Security Standards:**
- **Authentication**: Bearer token with constant-time comparison using `subtle::ConstantTimeEq`
- **Credential Storage**: OS keyring integration via `keyring` crate (falls back to environment variables)
- **Transport Security**: Optional TLS/HTTPS support using `axum-server` with rustls
  - Configure via `BAZBOM_TLS_CERT` and `BAZBOM_TLS_KEY` environment variables
- **Security Headers**:
  - Strict CSP without `unsafe-inline`: `default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'`
  - `X-Frame-Options: DENY`
  - `X-Content-Type-Options: nosniff`
  - `Strict-Transport-Security: max-age=31536000; includeSubDomains`
- **Input Validation**:
  - File size limits (10 MB max) to prevent DoS attacks
  - Path canonicalization to prevent path traversal
- **CORS**: Restricted to localhost only

**Container Security Standards:**
- Multi-stage Docker builds with distroless base images (`gcr.io/distroless/cc-debian12`)
- Non-root user (UID 65532) for minimal privilege
- Minimal attack surface (no shell, package manager, or unnecessary binaries)
- Alternative `Dockerfile.debian` available for debugging (includes shell)
- Comprehensive `.dockerignore` for build optimization

**Kubernetes Security Standards:**
- Namespace-scoped RBAC (Role instead of ClusterRole) in `crates/bazbom-operator/k8s/rbac-namespaced.yaml`
- NetworkPolicy for ingress/egress restrictions
- ResourceQuota and LimitRange for resource isolation
- Pod Security Standards: restricted profile
- No privileged containers, no host namespaces, no privilege escalation

**Fuzzing Requirements:**
- Fuzzing tests in `fuzz/` directory using cargo-fuzz
- Targets: SBOM parser, dependency graph, policy engine
- Run in CI for 60 seconds minimum: `cargo fuzz run sbom_parser -- -max_total_time=60`
- Coverage reports: `cargo fuzz coverage sbom_parser`

**Security Documentation:**
- All security docs in `docs/security/`
  - `SECURITY_ANALYSIS.md`: Comprehensive security audit report
  - `GPG_SIGNING.md`: GPG signing guide for releases
  - `JWT_AUTHENTICATION.md`: JWT implementation plan for future enhancement
- Installation script verification: `install-secure.sh` with SHA-256 and GPG verification

**Security Best Practices:**
- NEVER use timing-unsafe string comparisons for secrets (use `subtle::ConstantTimeEq`)
- ALWAYS sanitize user input before logging (use `security::log_sanitizer`)
- ALWAYS validate file paths with canonicalization before filesystem access
- ALWAYS verify external tool checksums before execution
- ALWAYS use TLS/HTTPS for production dashboard deployments
- Log all security-relevant events to audit log (authentication, authorization, vulnerabilities)

## Homebrew Tap and Distribution

- Create and use a user‑owned tap before upstreaming to homebrew‑core.
- See `docs/operations/homebrew-tap-creation.md` for formula template and steps.
- Release assets: macOS (x86_64/arm64), Linux (x86_64/aarch64); signatures + provenance.

## Sanity Checks Before Merge

**Code Quality:**
- [ ] `cargo check --workspace --all-features --all-targets` passes
- [ ] `cargo clippy --workspace --all-features --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] `cargo test --workspace --all-features` passes
- [ ] `cargo doc --workspace --no-deps` produces no warnings
- [ ] No new unsafe code blocks introduced
- [ ] All new Cargo.toml files include: name, version, edition, license, repository

**Documentation & Features:**
- [ ] Capabilities Reference updated and consistent with README
- [ ] CLI docs updated; examples for Maven/Gradle/Bazel verified
- [ ] Schema changes versioned; golden tests updated; validators pass
- [ ] Coverage gates met (repo ≥90%; critical pkgs ≥98%; branch coverage on)
- [ ] Docs only under `docs/` (except allowed stubs and this file); links valid
- [ ] Action examples tested; pre‑commit, tests, and build pipelines green

## Developer Experience (IDE Integration)

**Status:** In Progress - See `docs/integrations/ide/ide-integration.md`

### IDE Plugin Development Rules

**IntelliJ IDEA Plugin (`crates/bazbom-intellij-plugin/`):**
- Built with Gradle and Kotlin
- Uses IntelliJ Platform SDK 2025.2+
- Target: IntelliJ IDEA Community & Ultimate
- Key features: dependency tree view, real-time annotations, quick fixes
- Build: `./gradlew build`, Run: `./gradlew runIde`
- Publish to JetBrains Marketplace after testing

**VS Code Extension (`crates/bazbom-vscode-extension/`):**
- Built with TypeScript and npm
- Uses Language Server Protocol (LSP) via `bazbom-lsp` crate
- Target: VS Code 1.85+
- Key features: diagnostics, code actions, commands
- Build: `npm run compile`, Package: `npx vsce package`
- Publish to VS Code Marketplace after testing

**LSP Server (`crates/bazbom-lsp/`):**
- Rust implementation using `tower-lsp` crate
- Reusable across editors (VS Code, Vim, Emacs, Sublime)
- Provides: diagnostics, code actions, hover info
- Fast mode scanning (<10 seconds)
- Must be cross-platform compatible

### Automated Remediation Rules

**`bazbom fix` Commands:**
- `--suggest`: Safe, read-only mode (default)
- `--apply`: Writes to files, requires testing
- `--pr`: Opens GitHub PR (requires token)
- Always explain "why fix this" (CVSS, KEV, EPSS context)
- Always run tests after applying fixes
- Always rollback on test failure
- Support all JVM build systems: Ant, Maven, Gradle, Bazel, Buildr

**Safety Requirements:**
- Never apply fixes without user consent (unless `--apply` flag)
- Always create backups before modifying files
- Always run project tests after applying fixes
- Always rollback changes if tests fail
- Never lose user data or break working code

### Pre-Commit Hooks Rules

**`bazbom install-hooks` Command:**
- Installs Git pre-commit hook at `.git/hooks/pre-commit`
- Fast mode (--fast) for speed (<10 seconds)
- Policy enforcement blocks commits with violations
- Must be bypassable with `git commit --no-verify`
- Works on macOS, Linux, Windows (Git Bash)

## Code Quality Standards

**Quality Metrics (Latest Deep Analysis):**
- ✅ All Rust code compiles without errors
- ✅ Zero clippy warnings with `-D warnings` flag
- ✅ All tests passing (100% success rate)
- ✅ Code formatting verified with `cargo fmt --check`
- ✅ Zero unsafe code blocks across entire codebase
- ✅ All YAML configuration files validated
- ✅ All Cargo.toml files include license and repository metadata
- ✅ No debug statements (dbg!) in production code
- ✅ Proper error handling patterns (minimal unwrap/expect in critical paths)

**Continuous Quality Enforcement:**
- Run `cargo check --workspace --all-features --all-targets` before commits
- Run `cargo clippy --workspace --all-features --all-targets -- -D warnings` to catch issues
- Run `cargo fmt --all -- --check` to verify formatting
- Run `cargo test --workspace --all-features` to verify all tests pass
- All Cargo.toml files must include: name, version, edition, license, repository
- Document TODOs with context (not just placeholders)
- Use proper error handling (Result types, descriptive errors)
- Prefer logging over println! in production code paths

## Version-Specific Documentation

**v6.8 Planning (Q2 2026 Release):**
- Overview: `docs/development/versions/6.8/README.md`
- Integration Plan: `docs/development/versions/6.8/jira-integration-plan.md`
- Technical Specs: `docs/development/versions/6.8/technical-specifications.md`
- Implementation Roadmap: `docs/development/versions/6.8/implementation-roadmap.md`
- Integration Analysis: `docs/development/versions/6.8/integration-analysis.md`
- PR Template: `docs/development/versions/6.8/pr-template-complete.md`

**Key v6.8 Planning Stats:**
- 6 planning documents, 5,078 lines, 151KB total
- 9 feature categories (including GitHub PR automation)
- 48 integration points across Jira, GitHub, and BazBOM
- 14+ intelligence modules integrated into every automated PR
- 20-week implementation timeline (January 2026 - June 2026)
- 4 milestones: M1 Alpha (Feb 2026), M2 Beta (Apr 2026), M3 RC (May 2026), M4 GA (Jun 2026)

## Additional Sources

- Tech writer persona: `docs/tech_writer_persona.md`
- IDE integration: `docs/integrations/ide/ide-integration.md`
- Architecture documentation: `docs/ARCHITECTURE.md`
- Development guide: `docs/development/README.md`
- v6.8 planning: `docs/development/versions/6.8/README.md`

Questions? Open a docs issue and tag `@cboyd0319`.
