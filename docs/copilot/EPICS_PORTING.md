# Python → Rust Porting Plan (Incremental)

Objective: Replace Python runtime in distributed artifacts with a Rust-first implementation while maintaining feature parity and quality.

Porting Order
1) Core Graph Model and PURL Generation
   - Port `purl_generator.py` and normalized graph schema

2) Advisory Fetch and Merge
   - Port `osv_query.py`, `vulnerability_enrichment.py`, `kev_enrichment.py`, `epss_enrichment.py`, GHSA ingestion
   - Deterministic merge and canonical severity/priority computation

3) Exporters
   - Port `write_sbom.py` (SPDX/CycloneDX), `sarif_adapter.py`, `csv_exporter.py`

4) Policy Engine
   - Port `policy_check.py` with YAML schema; add optional Rego/CUE integration

5) Remediation
   - Port `upgrade_recommender.py`, `interactive_fix.py` to Rust; implement PR openers for Maven/Gradle/Bazel

6) Incremental Analyzer & Caching
   - Port `incremental_analyzer.py`, content-addressed caches, and performance knobs

7) Compliance & Signing
   - Port `sbom_signing.py`, `provenance_builder.py` to Rust; keep in-toto formats

8) Containers (Adjacency)
   - Port `container_scanner.py`, `scan_container.py` with OS package detection

Testing & Parity Gates
- Golden tests for each module; parity score must reach 100% before deprecating Python module
- Performance not worse than Python baseline on benchmark set

