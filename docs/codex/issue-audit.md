# BazBOM Issue Audit (Code-Derived)

## Scope & Method
- Code-first scan across CLI commands (`crates/bazbom/src/commands/*`), container tooling (`crates/bazbom-containers/src/tools/*`), and select shared crates. Used `rg` to spot policy violations (emoji usage) and sampled error-handling patterns (e.g., unchecked `unwrap`, subprocess handling). Not an exhaustive formal verification, but a deep pass across primary surfaces.

## High-Risk / Blocking
- Tool/daemon preflight gaps: `container_scan::check_tools()` only checks Syft/Trivy while the flow depends on Docker, cosign, Grype, Dockle, Dive, TruffleHog, etc. (`handler.rs`). Missing binaries or Docker daemon failures surface mid-run instead of upfront.
- Required artifact assumptions: Later steps require `syft-sbom.json` and `trivy-results.json` for layer attribution; `run_orchestrated_scan` proceeds even when tools are missing and will later fail or misreport.
- Unhandled panics via `unwrap` in production code (e.g., `crates/bazbom-formats/src/{cyclonedx,spdx,sarif}.rs` serialize with `unwrap`), risking crashes on serialization errors or invalid data.

## Medium-Risk / Reliability Gaps
- Signature/provenance checks: `Invalid` results from cosign verification do not fail the run; scans can “pass” despite verification failures.
- No timeouts on external tools: `Command::new`/`run_command` calls lack execution limits; hung Docker pulls or scanners can deadlock the command.
- Partial/ambiguous coverage: Orchestrator logs tool failures but still returns aggregates without indicating which tools actually ran; deduplication by CVE discards severity/fix data from secondary sources.
- Network/dependency assumptions: `docker pull` runs unconditionally; no offline/air-gap handling, leading to noisy failures in restricted environments.
- Policy conflicts: Emojis exist in CLI output (`container_scan` banner), templates (`crates/bazbom-jira`, `crates/bazbom-github`), and other surfaces, violating the “Zero emojis in code” rule in `CONTRIBUTING.md`.

## Low-Risk / Quality Issues
- Inconsistent progress UX (e.g., “Step 5.5/8”) and heavy color/emoji usage hinder log parsing/CI consumption.
- Orchestrator dedup logic keeps first CVE occurrence only, losing richer fields (fix versions, references) from other tools.
- Missing diagnostics for disk usage and output size when writing multiple SBOM/report files; large images could exhaust space without clear messaging.

## Follow-Ups to Confirm in Code
- Add tests for missing-tool paths, malformed/missing Syft/Trivy outputs, and tool timeouts; orchestrator tests currently only cover construction.
- Validate behavior when zero tools run (fail fast) and surface a machine-readable list of executed/skipped tools in summaries.
- Replace production `unwrap`/`expect` with fallible handling in formats/reporting paths to avoid crash-on-serialization.***
