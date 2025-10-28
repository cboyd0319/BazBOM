# Phase 0 Issues (Seed List)

Use these as templates to create GitHub issues.

---

## Issue: Rust CLI Skeleton (bazbom)

Title: feat(cli): scaffold Rust CLI (bazbom) with core subcommands

Body:
- Implement Rust workspace and `bazbom` binary crate
- Subcommands: `scan`, `policy check`, `fix --suggest/--apply`, `db sync`
- Print basic messages; wire to placeholder core crates
- Add version, help, and completion generation
- Acceptance:
  - `cargo build` and `cargo test` pass in CI
  - `bazbom --help` and `bazbom scan .` print expected output

Labels: area:cli, rust, phase:0, priority:P0

---

## Issue: Signed Single Binaries + Provenance

Title: chore(release): produce signed single binaries (macOS/Linux) with provenance

Body:
- Build release artifacts for macOS (x86_64/arm64) and Linux (x86_64/aarch64)
- Sign with Sigstore (cosign) and attach provenance
- Update release workflow; publish checksums
- Acceptance:
  - Binaries downloadable and verified with cosign
  - Checksums published; `bazbom --version` executes on both platforms

Labels: release, security, supply-chain, phase:0, priority:P0

---

## Issue: Advisory DB Offline Sync

Title: feat(db): `bazbom db sync` for OSV/NVD/GHSA + KEV/EPSS

Body:
- Implement offline cache layout and hashing
- Add `db sync` that fetches and normalizes sources
- No telemetry; explicit command only
- Acceptance:
  - Cache populated deterministically
  - Subsequent runs use cache; no network when scanning

Labels: advisories, offline, phase:0, priority:P0

---

## Issue: Coverage + Lint Gates

Title: ci: enforce coverage (>=90%) and docs location policy

Body:
- Add `--cov-fail-under=90` to coverage job
- Add docs-location workflow (docs/ only + allowed stubs)
- Add markdownlint + Vale checks (reuse existing config)
- Acceptance:
  - CI fails if coverage drops <90%
  - CI fails if docs outside docs/ (except allowlist)

Labels: ci, quality, docs, phase:0, priority:P0

---

## Issue: Documentation Updates (Phase 0)

Title: docs: update installation, security model, offline mode (Rust-first)

Body:
- Update docs to reflect Rust-first single binary
- Add Homebrew tap instructions
- Clarify offline advisory sync and zero telemetry
- Acceptance:
  - docs/QUICKSTART.md includes Rust CLI usage
  - docs/USAGE.md shows `scan`, `db sync`, policy checks

Labels: docs, phase:0, priority:P0

