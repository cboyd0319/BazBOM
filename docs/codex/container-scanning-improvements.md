# Container Scanning Improvements (Code-Derived)

## Resilience & Preflight (Observed in code)
- Expand `check_tools()` (handler.rs) to verify Docker/daemon plus cosign; rely on orchestrator `check_tools()` to gate Grype/Dockle/Dive/TruffleHog/Syft/Trivy before running. Abort or degrade if required tools for later steps (Syft/Trivy for layer attribution) are missing.
- Add explicit handling when no tools run: `run_orchestrated_scan` currently returns empty aggregates; downstream steps assume `syft-sbom.json` and `trivy-results.json` exist and will fail late.
- Introduce command timeouts in `run_command` (tools/mod.rs) so long pulls/scans cannot hang indefinitely; surface timeout vs. execution errors distinctly.
- Provide `--skip-pull`/`--offline` flags to avoid unconditional `docker pull` in `handle_container_scan` and to bypass network-dependent steps cleanly.

## Security & Trust
- In `verify_container_signature`/`verify_slsa_provenance`, treat `Invalid` states as failures unless a `--allow-unsigned` flag is set; propagate into overall exit status.
- Emit signature/provenance verdicts into SARIF/HTML generation (handler.rs) instead of only printing to stdout, enabling policy enforcement downstream.

## UX & Reporting
- Replace emoji-heavy banners and labels in `handle_container_scan`/HTML generation with ASCII or styled text to comply with the “no emojis in code” rule.
- Normalize step numbering (avoid “5.5/8”) and add structured progress/log output (JSONL) so CI can parse runs without scraping colored text.
- When a tool is skipped (orchestrator warns), record that in final summaries and reports (e.g., “Dockle unavailable—CIS checks skipped”) rather than silent omissions.

## Performance & Resource Safety
- Guard disk usage before writing multiple outputs (`syft-sbom.json`, SPDX/CycloneDX, Trivy JSON) and consider streaming/chunking large files; current code writes synchronously with no size checks.
- Add optional concurrency limits for orchestrator tasks to avoid overloading small CI runners; presently all enabled tools launch in parallel.
- Cache immutable intermediate artifacts (Syft SBOM, Trivy results) under `bazbom_core::cache_dir()` keyed by image digest to accelerate repeat scans.

## Coverage & Hardening
- Add tests for orchestrator error paths (tool missing, tool returns non-zero, malformed JSON) and for the handler when required files are absent; current tests only check construction.
- Validate inputs before layer attribution: confirm `syft-sbom.json` and `trivy-results.json` exist and have expected schema, with actionable errors if not.
- Deduplication in orchestrator is by CVE only; consider merging severity/fix data from multiple tools rather than dropping alternate records.***
