# BazBOM Roadmap (Sprints & Milestones)

Cadence: 2-week sprints, release train model. No external deadlines.

This roadmap is aligned with the MASTER_PLAN and is designed to push BazBOM beyond both OSS and commercial tools in accuracy, capability, and UX — while preserving zero telemetry and offline operation.

---

## Phase 0 (Week 0–2): Rust CLI Foundation & Packaging

- Rust CLI skeleton (`bazbom`) with core commands (`scan`, `policy check`, `fix --suggest`, `db sync`)
- Single-binary artifacts for macOS and Linux; signed via Sigstore; provenance included
- Offline cache layout and hashing; reproducible outputs for `scan`
- Docs: installation matrix, offline mode, security model
- Testing: establish CI with coverage enforcement (repo-wide `--cov-fail-under=90`), linting, formatting; add initial golden tests

Exit Criteria
- Single binary artifacts for macOS and Linux, verified and signed
- `bazbom db sync` creates deterministic advisory cache

---

## Phase 1 (Week 3–6): Authoritative Graphs

- Maven: `bazbom-maven-plugin` with effective POM, BOM, scopes, conflicts, shading/relocation
- Gradle: `io.bazbom.gradle-plugin` (init script + plugin) with per-configuration/variant output, Android support
- Bazel: refine `java_*` aspects and bzlmod + rules_jvm_external support; workspace SBOM merge
- Schema: versioned JSON schemas for normalized graph outputs

Exit Criteria
- Maven/Gradle/Bazel emit canonical graph JSON with tests and golden files

---

## Phase 2 (Week 7–10): Intelligence Merge & Policy

- Advisory merge engine (OSV/NVD/GHSA) with KEV + EPSS enrichment
- Canonical severity and P0–P4 priority scoring
- Policy-as-code (YAML) MVP with license, severity, reachability toggles
- SARIF output mapped to policy verdicts; PR-friendly summaries
- Testing: reach >=90% coverage on merge engine and policy packages; add mutation tests (optional)

Exit Criteria
- Deterministic findings across sources; policy gates with clean UX

---

## Phase 3 (Week 11–14): Reachability & Shading Precision

- Bytecode reachability engine using OPAL
- Caching and scoping controls; CLI flag `--reachability`
- Shaded/fat JAR attribution via relocation maps and fingerprinting
- Testing: integration tests on representative shaded/fat JARs; coverage target ~100% for mappers

Exit Criteria
- Reachable/unreachable flags emitted; method-level traces available

---

## Phase 4 (Week 15–18): Remediation Automation (Suggest-first)

- `bazbom fix --suggest` with educational “why fix” rationale
- `bazbom fix --apply` to open PRs for Maven/Gradle/Bazel with compatibility checks
- CI canary flows and automatic rollback on failure

Exit Criteria
- Clean PRs opened across all three build systems for representative repos

---

## Phase 5 (Week 19–22): Windows + Distribution Hardening

- Windows single-binary packaging via Rust cross-compile and signing
- Chocolatey/winget manifests; improved Homebrew support (bottles)
- Provenance and release process hardening; reproducible builds checklist

Exit Criteria
- All three OS targets supported with signed, verified binaries

---

## Phase 6 (Week 23–26): Scale, Performance, and UX Quality

- Incremental analysis tuning for monorepos (Bazel + Maven/Gradle multimodule)
- Parallelism/autoscaling knobs; memory caps; progress bars
- CSV exports for business users; compliance bundle improvements
- Testing: performance regression thresholds; end-to-end determinism checks

Exit Criteria
- Large-repo targets met (full <30m, incremental <10m) with stable memory footprint

---

## Phase 7 (Quarterly, ongoing): Advanced & Adjacent Ecosystems

- Container SBOM (rules_oci + CLI); OS packages; multi-layer attributions
- Kotlin rules parity in Bazel, then broader JVM rules
- Optional advisory sources expansion (OSS Index, etc.) behind explicit flags
- Exploration: Rust-native modules for performance hotspots

Exit Criteria
- Adjacent ecosystem support without compromising JVM-first quality

---

## Epics → Issues Seeding (High-Level)

- EPIC: Single Binary Packaging & Distribution
  - TASK: PyInstaller spec, signing, checksums, CI pipeline
  - TASK: Homebrew formula automation; release notes generator

- EPIC: Maven Plugin
  - TASK: Effective POM + BOM resolution
  - TASK: Scopes/conflicts graph JSON schema + tests
  - TASK: Shade/relocation mapping + nested JAR exploration

- EPIC: Gradle Plugin
  - TASK: Variant API integration (Android flavors/types)
  - TASK: Shadow plugin mapping; dependency insight fallback

- EPIC: Bazel Aspects
  - TASK: bzlmod + rules_jvm_external full fidelity
  - TASK: Kotlin aspects; workspace SBOM merge and dedupe

- EPIC: Advisory Merge & Enrichment
  - TASK: Source dedup rules; canonical severity mapping
  - TASK: KEV + EPSS tagging; priority P0–P4

- EPIC: Reachability Engine
  - TASK: Choose OPAL/Soot; prototype and cache layer
  - TASK: Method-level trace export; CLI integration

- EPIC: Policy + VEX
  - TASK: YAML policy schema + Rego option
  - TASK: VEX auto-generation rules + schema validation

- EPIC: Remediation Automation
  - TASK: Suggest reports with “why fix”; PR opening for Maven/Gradle/Bazel
  - TASK: Compatibility checks; canary/rollback process

- EPIC: Offline DB Sync
  - TASK: `bazbom db sync` command; deterministic cache structure
  - TASK: Air-gapped docs and validation

---

## Acceptance Gates (per Release Train)

- Determinism: identical inputs → identical outputs
- Schema stability: versioned, validated, and backward compatible
- Security: signed artifacts, provenance included, no telemetry
- Performance: no regression vs prior train on benchmark set
- Coverage: repo-wide ≥90%; critical modules ≥98%; branch coverage enabled
- Docs: pass markdownlint/Vale; no docs outside `docs/` (except minimal pointers if approved)
