# BazBOM: World-Class JVM SBOM, SCA, and Dependency Graph

Audience: Enterprise and Application Security engineers (primary), Platform/DevSecOps (secondary), JVM developers (tertiary)

Privacy: 100% privacy, zero telemetry. MIT-licensed. Air-gapped friendly.

Target Platforms: macOS → Linux → Windows (in that order)

Vision: Deliver the most accurate, fast, and easy JVM-focused SBOM/SCA/Graph solution for Maven, Gradle, and Bazel — exceeding both open-source and commercial tools — while remaining hermetic, private-by-default, and delightful to operate at scale.

---

## Executive Summary

BazBOM will be the authoritative source of truth for JVM dependency intelligence across Maven, Gradle, and Bazel. We achieve this by integrating at build-time, enriching with best-in-class security context (OSV, NVD, GHSA, CISA KEV, EPSS), performing bytecode reachability to cut false positives, and providing precise remediation with explainable “why fix this?” guidance. Distribution will converge on a single binary, effortless to install and operate in CI/CD and air‑gapped environments.

Key Differentiators
- Build-native accuracy across Maven/Gradle/Bazel with full scope fidelity
- Bytecode-level reachability (call graph) to prioritize real risk
- Shading/relocation mapping for shaded and fat JARs (true attribution)
- Policy-as-code (YAML/Rego/CUE) with VEX auto-application and CI enforcement
- Automated, safe remediation (suggest by default; can apply fixes end-to-end)
- Hermetic, offline-first operation with local cached advisories
- Single binary distribution with cryptographic signing and supply-chain provenance

---

## Product Principles

- Accuracy over heuristics: Use build system authorities and compiled artifacts
- Zero surprises: Private-by-default, no telemetry, deterministic outputs
- Explainability: Always include “why this matters,” “how to fix,” and “what changed”
- Secure-by-design: SLSA provenance, signed SBOMs, verified downloads
- Ergonomics: One command fits all; suggest-only by default; safe apply when asked

---

## Personas & Top Jobs-To-Be-Done

Enterprise/AppSec Engineer
- Get a complete SBOM and reachable vulnerabilities for any JVM project
- Enforce policies across org; produce compliance bundles on demand
- Create auditable, signed artifacts (SPDX/CycloneDX/SARIF/VEX/Provenance)

Platform/DevSecOps
- Operate at monorepo scale (5K+ targets) with incremental performance
- Integrate easily into CI/CD and developer workflows (GitHub Action)
- Run offline with mirrored advisory data and no external calls

Developers
- Receive actionable, explainable findings with safe remediation PRs
- Understand dependency conflicts and shading/relocation impacts

---

## Architecture & Packaging Strategy

Rust-First Single Binary (memory-safe by design)
1) Near-term: Introduce a Rust CLI (`bazbom`) as the primary distribution
   - Implement core pipeline in Rust: detection, graph model, advisory fetch/merge, exporters, policy checks
   - Use OPAL (JVM) as a helper for reachability via a small, bundled jar invoked with `java -jar`
   - Do not embed Python in distributed artifacts; Python remains for dev utilities only
   - Sign artifacts with Sigstore (cosign); publish checksums and provenance
   - Homebrew tap (user-owned) + GitHub Releases; winget/choco later

2) Mid-term: Expand Rust coverage of existing Python modules
   - Port high-value modules to Rust: purl generation, OSV/NVD/GHSA merge, KEV/EPSS enrichment, SARIF adapter, SPDX/CycloneDX writers
   - Keep strict unsafe-free policy; audit any required FFI

3) Long-term: Full Rust CLI with JVM helpers
   - JVM analyzers remain as jars (memory-safe managed runtime)
   - Maintain strict offline mode through pluggable “data providers”

Data Flow
- Detection: Identify build system and project structure
- Discovery: Build-native graph extraction (plugins/aspects)
- Normalize: Convert to canonical graph model (components, edges, metadata)
- Enrich: OSV/NVD/GHSA + KEV + EPSS (dedup, merge, classify, score)
- Reachability: Bytecode call graph to tag reachable/unreachable findings
- Policy: Apply policy bundles and VEX; produce SARIF and policy verdicts
- Remediate: Suggest/apply fixes, run checks/tests, and open PRs

Storage & Caching
- Local cache (SQLite or files) for normalized graphs and advisories
- Deterministic builds: hash inputs, cache by content, persist across runs
- Offline sync command to mirror advisory datasets

Security & Integrity
- SLSA provenance for BazBOM releases and outputs
- Sigstore keyless signing for binaries and SBOMs
- No telemetry, no background network calls; explicit `db sync` for updates
- Memory-safe implementation preference: Rust for CLI/core; JVM/OPAL for reachability

---

## Deep Build System Integrations

Maven (Authoritative Plugin)
- New module: `bazbom-maven-plugin`
- Emits authoritative JSON including:
  - Full scopes (compile/runtime/test/provided), dependencyManagement, BOM resolution
  - Effective POM capture, conflict resolution results
  - Shading and relocation mappings (read shade plugin configs and produced manifests)
  - Artifact coordinates, PURLs, licenses, hashes
- Modes: per-module, reactor-aggregate, and workspace summary
- Output stability guarantees (schema versioning + JSON schema)

Gradle (Init Script + Plugin)
- New plugin: `io.bazbom.gradle-plugin`
- Outputs machine-readable graphs per configuration/variant
- Android support (flavors/build types via Variant API)
- Shading detection for Shadow plugin; dependency insight fallback path
- Tasks: `bazbomGraph`, `bazbomSbom`, `bazbomFindings`

Bazel (Aspects First-Class)
- Extend existing aspects for:
  - `java_*` (priority), then Kotlin, then broader JVM rules
  - bzlmod + rules_jvm_external sophistication
  - Precise scope mapping and artifact provenance
- Incremental analysis using target diffs; workspace-level SBOM merge
- Expose outputs as providers to other Bazel rules (SBOM/VEX/SARIF targets)

Container/JRE Awareness (Adjacency, non-primary)
- Optionally analyze JVM apps packaged into containers (rules_oci or CLI)
- Include OS packages + embedded JARs; treat as separate layers in SBOM

---

## Vulnerability Intelligence Pipeline

Sources (initial): OSV, NVD, GHSA, CISA KEV, EPSS
- Dedup & Merge Engine: unify CVE/GHSA/OSV IDs; compute canonical severity
- Add EPSS probability, KEV presence, exploit maturity
- Normalize CVSS (v3/v4 as available); compute priority (P0–P4)
- Offline data: `bazbom db sync` downloads and refreshes local mirrors

Outputs
- `sbom.spdx.json` (primary), optional CycloneDX 1.5
- `sca_findings.json` (machine-readable), `sca_findings.sarif` (GitHub)
- CSV export (licenses, vulns, components) for business users

Explainability
- For each finding: include “why fix this?” with:
  - KEV presence, EPSS percentile, reachability, exploit maturity
  - Impacted call sites, suggested upgrade/rule, and links to vendor advisories

---

## Reachability & Shading/Relocation

Bytecode Reachability (opt-in, powerful)
- Use OPAL to build call graphs (selected backbone)
- Inputs: compiled classes + runtime classpath from Maven/Gradle/Bazel
- Tag vulnerabilities as reachable/unreachable; include method-level traces
- Performance controls: scope by module/target; cache call graphs

Shaded/Fat JAR Attribution
- Parse shading plugin configs (maven-shade, gradle-shadow)
- Build relocation maps; discover nested JARs
- Fingerprint classes to map shaded content back to original GAV/PURL
- Output precise component attribution and fix paths

---

## Policy-as-Code & VEX

Policy Engine
- Central config: `bazbom.yml` (simple) + optional Rego/CUE bundles
- Rules: severity thresholds, license allow/deny, KEV/EPSS gates, reachability gates
- CI gating with rich PR comments and SARIF annotations

VEX Workflow
- Generate and consume VEX (CSAF 2.0) to suppress false positives
- Auto-produce VEX when reachability is false and policy allows
- Keep auditability and change history

---

## Remediation Automation (Suggest-first)

Strategy
- Default: “suggest-only,” with educational context
- Capable: `--apply` to open PRs and run verification

Capabilities
- Maven: update versions, insert BOMs, manage exclusions; validate with `mvn -DskipTests=false test`
- Gradle: update constraints/platforms, version catalogs, Shadow plugin configs; validate with `gradle test`
- Bazel: update `maven_install.json` and artifact rules; validate builds/tests
- Compatibility checks: CI canary branches; rollbacks on failure
- PR metadata: changelog snippets, security rationale, and VEX updates

---

## UX & CLI

CLI Principles
- One-liners that “just work” across Maven/Gradle/Bazel
- Deterministic outputs with clear file locations and summaries
- Educational, contextual messages; minimal noise

Key Commands (illustrative)
- `bazbom scan .` → SBOM + findings + SARIF
- `bazbom scan --format spdx --reachability` → enable call graph
- `bazbom graph --export graphml` → visual graph export
- `bazbom policy check` → apply policy bundles, exit non-zero on violation
- `bazbom fix --suggest` → suggestions report
- `bazbom fix --apply` → opens PRs with guardrails
- `bazbom db sync` → offline advisory data refresh
- `bazbom package` → produce single-file binary

---

## Performance & Scale

- Incremental analysis keyed by changed modules/targets
- Parallel processing with safe concurrency
- SQLite/file caches for advisory merges and graph snapshots
- Remote cache friendliness (Bazel) and idempotent CLI

Targets
- Small repo full scan < 2 min; PR incremental < 1 min
- Large monorepo: full < 30 min; incremental < 10 min

---

## Security & Compliance

- SLSA Level 3 provenance for outputs and releases
- Sigstore signing, checksum publication
- Strict offline mode; zero telemetry
- Extensive validation: SPDX/CycloneDX/SARIF/VEX schema checks

---

## Testing Strategy

- Coverage goals: >90% repo-wide; ~100% on critical modules (merge engine, policy, exporters, graph normalizer)
- Enforce coverage in CI: `--cov-branch --cov-fail-under=90` and per-package minimums for critical modules
- Golden-file tests for JSON schemas and SBOM content
- Property-based tests for graph normalization and dedup logic
- Integration tests against example Maven/Gradle/Bazel projects (including Android)
- Performance regression tests and benchmarks
- Fuzz targeted parsers (POM, Gradle metadata, shaded JAR exploration)

Quality Gates (CI)
- Linting (ruff/flake8 for existing Python until ported; clippy/pedantic for Rust)
- Unit + integration tests required; coverage thresholds enforced; artifact schema validation
- Docs checks: markdownlint + Vale; links validation; docs-only location policy

---

## Documentation & Adoption

- Quickstarts for each build system with copy‑paste commands
- “Why fix this?” knowledge snippets linked from CLI output
- Compliance bundle walkthrough (audit-ready artifacts)
- Troubleshooting: shading, relocations, conflicts, offline data

Documentation Standards
- All documentation resides under `docs/` (canonical). Root-level files should be minimal pointers only if absolutely necessary for GitHub conventions.
- Consistent structure: H1 title, context-first intro, tasks/examples, references
- File naming: UPPER_SNAKE_CASE.md for conceptual/reference; lower-kebab-case.md for guides
- Style: Enforced with markdownlint and Vale (TechDocs styles). Prefer active voice, consistent terminology, and command examples with copy-paste blocks
- Cross-link with relative paths; avoid duplicate content; deprecate split variants
- ADRs in `docs/ADR/` with numbered records and clear status
 - Prohibited: Zero emojis in code, ever (source files, generated code, code comments, and copy/paste code samples)
 - Doc creation criteria: prefer updating canonical docs over adding new pages; only create new docs for clear, reusable gaps; index new docs in `docs/README.md`

---

## Risks & Mitigations

- Single-binary with JVM analyzers: bundle small helper jars and rely on local JRE; fall back to non‑reachability mode when JRE unavailable
- Windows packaging: Rust cross-compile and code-sign; WSL fallback optional
- Advisory consistency: deterministic merge rules + versioned schemas
- Performance of reachability: cache aggressively; opt-in; scoped per target

---

## Metrics of Success (Local-only; no telemetry)

- Time to first SBOM (goal: <60s typical)
- False positive reduction via reachability (>50% fewer actionable items)
- PR remediation success rate (>85% apply cleanly)
- Policy conformance in CI (violations trend down across sprints)

---

## Immediate Epics (Acceptance Criteria summarized)

1) Rust Single-Binary Packaging (macOS/Linux/Windows)
   - Produce signed, single-file artifacts; Homebrew tap formula published

2) Maven Plugin (Authoritative Graph JSON)
   - Effective POM, BOM, scopes, conflicts, shading/relocation captured

3) Gradle Plugin (Variants + Android)
   - Per-configuration/variant graphs; Shadow plugin support

4) Bazel Aspects Expansion
   - bzlmod + rules_jvm_external + java_* complete; Kotlin next

5) Vulnerability Merge Engine
   - OSV/NVD/GHSA dedup; KEV + EPSS enrichment; canonical severity & priority

6) Reachability Engine (Opt-in, OPAL)
   - OPAL-based call graph; reachable tagging; method-level traces

7) Policy-as-Code + VEX
   - YAML + optional Rego; CI gating; auto-VEX on unreachable where policy allows

8) Remediation Automation
   - Suggest-only by default; `--apply` opens PRs; runs tests/canaries

9) Offline DB Sync
   - `bazbom db sync` to refresh advisory mirrors; deterministic cache

---

## Roadmap Cadence

- 2-week sprints; 90-day objectives; release train model
- See ROADMAP.md for a time-sequenced breakdown

---

## Open Research Items

- OPAL tuning: performance/precision profiles and caching strategies
- Best fingerprinting strategy for shaded class → GAV mapping at scale
- CVSS v4 adoption timeline and mapping strategy
