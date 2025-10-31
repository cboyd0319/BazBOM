# Copilot Instructions — BazBOM

Purpose: Provide clear, enforceable guidance so changes remain aligned with BazBOM’s mission, security posture, testing rigor, and documentation standards.

## Mission & Non‑Negotiables

- World‑class JVM SBOM, SCA, and dependency graph across Maven, Gradle, and Bazel.
- Private-by-default: 100% privacy, zero telemetry. Offline-first operation is required.
- Memory‑safe distribution: Rust‑first single binary; OPAL (JVM) helper for reachability. Avoid unsafe; no embedded Python in shipped binaries.
- Policy‑as‑code at the core: YAML (plus optional Rego/CUE), VEX auto‑application, CI gating.
- Explainability and remediation: default “suggest‑only” with “why fix this?”; safe `--apply` opens PRs and runs checks.
- Deterministic, reproducible outputs with signed artifacts and SLSA provenance.

CRITICAL Repo Rules (must follow)
- Zero emojis in code, ever. Do not add emojis to source files, generated code, or code comments. Code examples in docs that users might copy/paste must also be emoji‑free.
- Avoid doc sprawl. Do not create a new doc for every small task. Prefer updating canonical docs under `docs/`. Create new documents only when a clear gap exists, and then link them from `docs/README.md`.

Primary audience: Enterprise/AppSec engineers; secondary: Platform/DevSecOps; tertiary: JVM developers.
Target OS: macOS → Linux → Windows.

## Architecture Snapshot

- Rust workspace: `bazbom` (CLI), `bazbom-core`, `bazbom-formats`, `bazbom-advisories`, `bazbom-policy`, `bazbom-graph`.
- Reachability: ASM‑based `bazbom-reachability.jar` invoked with `java -jar`; no network; JSON I/O; call graph generation.
- Shading detection: Maven Shade and Gradle Shadow plugin parsing; class fingerprinting with Blake3 hashing.
- Build integrations:
  - Maven: `bazbom-maven-plugin` emits authoritative JSON (scopes, dependencies, PURLs).
  - Gradle: `io.bazbom.gradle-plugin` with per‑configuration graphs; Shadow support.
  - Bazel: aspects for `java_*` (priority), then Kotlin, then broader JVM rules; bzlmod + rules_jvm_external.
- Intelligence: OSV/NVD/GHSA + KEV + EPSS; canonical severity + P0–P4 priority.
- Outputs: SPDX 2.3 (primary), CycloneDX 1.5 (optional), SARIF 2.1.0, CSAF VEX, CSV.

## Documentation Policy (must follow)

- All canonical docs live under `docs/` only.
- Allowed root stubs (minimal link‑only): `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `MAINTAINERS.md`.
- This file (`.github/copilot-instructions.md`) is an operational exception.
- Standards: see `docs/copilot/DOCUMENTATION_STANDARDS.md`.
  - markdownlint + Vale enforced; active voice; consistent terminology; relative links.
  - ADRs in `docs/ADR/`; images in `docs/images/`; diagrams in `docs/diagrams/`.

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
   - `docs/USAGE.md` for CLI changes; examples for Maven/Gradle/Bazel
   - `docs/PROVENANCE.md`, `docs/VEX.md`, `docs/PERFORMANCE.md` as needed
   - `docs/copilot/PHASE_4_PROGRESS.md` for Phase 4 IDE/remediation features
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

- Cover all three build systems in examples and tests:
  - Maven (pom.xml)
  - Gradle (build.gradle / build.gradle.kts; Android variants)
  - Bazel (WORKSPACE / MODULE.bazel; rules_jvm_external; aspects)
- Include: offline mode, VEX flow, GitHub Action, shaded/fat JAR examples.

## Security & Supply Chain Requirements

- SLSA Level 3 provenance; Sigstore keyless signing; checksums.
- Zero telemetry; explicit `bazbom db sync` for advisory updates.
- Policy‑as‑code (YAML; optional Rego/CUE). VEX auto‑generation on unreachable when policy allows.
- CWE mapping, SARIF 2.1.0 validation, SPDX 2.3 and CycloneDX 1.5 validation.

## Homebrew Tap and Distribution

- Create and use a user‑owned tap before upstreaming to homebrew‑core.
- See `docs/copilot/HOMEBREW_TAP.md` for formula template and steps.
- Release assets: macOS (x86_64/arm64), Linux (x86_64/aarch64); signatures + provenance.

## Sanity Checks Before Merge

- [ ] Capabilities Reference updated and consistent with README
- [ ] CLI docs updated; examples for Maven/Gradle/Bazel verified
- [ ] Schema changes versioned; golden tests updated; validators pass
- [ ] Coverage gates met (repo ≥90%; critical pkgs ≥98%; branch coverage on)
- [ ] Docs only under `docs/` (except allowed stubs and this file); links valid
- [ ] Action examples tested; pre‑commit, tests, and build pipelines green

## Phase 4: Developer Experience (IDE Integration)

**Status:** In Progress (30% Complete) - See `docs/copilot/PHASE_4_PROGRESS.md`

### IDE Plugin Development Rules

**IntelliJ IDEA Plugin (`crates/bazbom-intellij-plugin/`):**
- Built with Gradle and Kotlin
- Uses IntelliJ Platform SDK 2023.3+
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
- Support all three build systems: Maven, Gradle, Bazel

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

## Additional Sources

- Documentation standards: `docs/copilot/DOCUMENTATION_STANDARDS.md`
- Rust packaging: `docs/copilot/RUST_PACKAGING_PLAN.md`
- OPAL reachability plan: `docs/copilot/REACHABILITY_OPAL.md`
- Python→Rust porting plan: `docs/copilot/EPICS_PORTING.md`
- Repo reorg checklist: `docs/copilot/REPO_REORG_RECOMMENDATIONS.md`
- **Phase 4 specification:** `docs/copilot/PHASE_4_DEVELOPER_EXPERIENCE.md`
- **Phase 4 progress:** `docs/copilot/PHASE_4_PROGRESS.md`

Questions? Open a docs issue and tag `@cboyd0319`.
