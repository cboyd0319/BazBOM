# Rust-First Single Binary Packaging Plan

Goal: Ship BazBOM as a memory-safe, single binary for macOS, Linux, and Windows, with signed releases and provenance.

Principles
- Memory safety: Implement core in Rust; avoid embedding CPython
- Determinism: Identical inputs produce identical outputs
- Reproducibility: Documented, automated build pipeline

Approach
1) Rust CLI (`bazbom`)
   - Crate layout: `bin/bazbom` (CLI), `bazbom-core` (pipeline), `bazbom-formats` (SPDX/CycloneDX/SARIF), `bazbom-advisories` (OSV/NVD/GHSA/KEV/EPSS), `bazbom-policy`, `bazbom-graph`
   - Async runtime with `tokio`; JSON with `serde`; hashing with `blake3`
   - SQLite (optional) using `rusqlite` for caches (or file-based JSON cache)

2) OPAL Integration (JVM)
   - Build `bazbom-reachability.jar` using OPAL; package as an asset in releases
   - Invoke via `java -jar` with classpath input and entrypoints; output JSON
   - Strict input/output schemas; cache results by content hash

3) Distribution
   - macOS (x86_64, arm64) and Linux (x86_64, aarch64): static-ish builds
   - Windows (x86_64): MSVC toolchain; code-sign where possible
   - Sign with Sigstore (cosign) and attach SLSA provenance

4) Homebrew + Windows Repos
   - Homebrew tap (user-owned) to start; upstream to homebrew-core when stable
   - Chocolatey/winget manifests post Windows packaging

Security Hardening
- No telemetry; explicit `db sync` for advisories
- Verify third-party downloads (checksums/signatures)
- `RUSTFLAGS` deny warnings; Clippy/pedantic; `cargo-audit` gated in CI

Migration Strategy from Python
- Keep Python tooling for dev-only scripts
- Port modules in priority order (see EPICS_PORTING.md)
- Maintain feature parity milestones and deprecate Python code once matched

