# GitHub Copilot Instructions for BazBOM

## Repository Overview

BazBOM is a **Bazel-native, Java-focused SBOM (Software Bill of Materials) and SCA (Software Composition Analysis)** system that:

- Generates **SPDX 2.3 SBOMs** for every build/deliverable
- Performs comprehensive Software Composition Analysis
- Produces **CI-ready, machine-readable outputs** (JSON + SARIF)
- Integrates with GitHub Code Scanning for security alerts
- Maintains **documentation as a first-class deliverable**

## Core Principles

### 1. Bazel-Native First
- Use **Bazel aspects** for automatic dependency discovery
- Zero manual dependency lists or config files to maintain
- Leverage Bazel's build graph for accurate, complete SBOMs
- All tools run hermetically via Bazel

### 2. Universal Coverage
- Produce an SBOM for **every relevant target**: libraries, binaries, deployables, container images
- Single command (`bazel build //:sbom_all`) generates all SBOMs
- Support incremental analysis for large monorepos

### 3. Minimal Invasiveness
- Assume **little/no package metadata** in existing `BUILD` files
- Solution must **infer** details (versions, licenses, PURLs) without code changes
- Works out-of-box with existing `rules_jvm_external` setups

## Technology Stack

- **Build system:** Bazel 6.0+ (tested up to 7.x)
- **Primary language:** Java/JVM via `rules_java` + `rules_jvm_external`
- **SBOM tooling:** `bazel-contrib/supply-chain` (via `http_archive`, **not BCR**)
- **Dependency source of truth:** `maven_install.json` lockfile
- **Python:** For tooling scripts (write_sbom.py, osv_query.py, sarif_adapter.py, etc.)

## Output Specifications

| Artifact Type | Format | Schema Version | Primary Use |
|--------------|--------|----------------|-------------|
| **SBOM** | SPDX (JSON) | 2.3 | Compliance, attestation, analysis |
| **SBOM** (optional) | CycloneDX (JSON) | 1.5 | Tool compatibility (flag: `--cyclonedx`) |
| **Dependency Graph** | JSON | Custom | Visualization, impact analysis |
| **SCA Findings** | JSON | Custom schema | Machine processing, auditing |
| **SARIF** | SARIF | 2.1.0 | GitHub Code Scanning integration |
| **Provenance** | SLSA Provenance | v1.0 | Build attestation, supply chain verification |
| **VEX** (optional) | CSAF VEX | 2.0 | Vulnerability Exploitability eXchange |
| **License Report** | SPDX License | 2.3 | Legal compliance analysis |

## Repository Structure

```
/
├── WORKSPACE                     # Bazel workspace, fetch bazel-contrib/supply-chain
├── BUILD.bazel                   # Root build file with //:sbom_all target
├── .bazelrc                      # Convenience aliases, remote cache config
├── .bazelversion                 # Pin Bazel version
│
├── tools/supplychain/
│   ├── BUILD.bazel              # Tooling targets (py_binary for scripts)
│   ├── defs.bzl                 # Public macros: sbom_for, sbom_all
│   ├── aspects.bzl              # Aspect implementation for dep traversal
│   ├── write_sbom.py            # Converts dep graph → SPDX 2.3 JSON
│   ├── sarif_adapter.py         # SCA findings → SARIF 2.1.0
│   ├── osv_query.py             # Query OSV for vulnerabilities
│   ├── purl_generator.py        # Maven coords → PURL conversion
│   ├── license_extractor.py     # JAR inspection for license metadata
│   ├── graph_generator.py       # Dependency graph → JSON/GraphML
│   ├── provenance_builder.py    # SLSA provenance generation
│   ├── vex_processor.py         # VEX statement application
│   ├── conflict_detector.py     # Detect version conflicts
│   ├── license_analyzer.py      # License compatibility checks
│   ├── supply_chain_risk.py     # Typosquatting, malware detection
│   ├── metrics_aggregator.py    # Dashboard JSON generation
│   ├── incremental_analyzer.py  # Git diff → affected targets
│   ├── sbom_schemas/            # Schema validation resources
│   ├── validators/              # Schema validation scripts
│   └── tests/                   # Unit tests for Python scripts
│
├── .github/
│   └── workflows/
│       ├── supplychain.yml      # Main CI: SBOM + SCA on every PR/push
│       ├── docs-lint.yml        # Markdown/docs validation
│       └── release.yml          # Release automation
│
├── docs/
│   ├── README.md                # Docs index / navigation
│   ├── SUPPLY_CHAIN.md          # Complete supply chain implementation guide
│   ├── USAGE.md                 # Daily developer commands & workflows
│   ├── ARCHITECTURE.md          # System design, diagrams, data flows
│   ├── VALIDATION.md            # SBOM/SARIF validation procedures
│   ├── TROUBLESHOOTING.md       # Common errors & solutions
│   ├── PERFORMANCE.md           # Optimization guide for large monorepos
│   ├── PROVENANCE.md            # SLSA provenance setup & signing
│   ├── VEX.md                   # VEX statement creation & management
│   ├── GRAPH_ANALYSIS.md        # Dependency graph querying & visualization
│   └── ADR/                     # Architecture Decision Records
│       ├── ADR-0001-fetch-strategy.md
│       ├── ADR-0002-sbom-format.md
│       ├── ADR-0003-aspect-scope.md
│       ├── ADR-0004-sarif-mapping.md
│       ├── ADR-0005-incremental-analysis.md
│       ├── ADR-0006-graph-storage.md
│       └── ADR-0007-slsa-level.md
│
├── examples/
│   ├── minimal_java/            # Smallest working example
│   ├── multi_module/            # Complex monorepo example
│   └── shaded_jar/              # Fat JAR / shaded dependencies
│
└── vex/statements/              # VEX statements for false positive suppression
```

## Documentation Standards (Mandatory)

Documentation quality is a **gate** for merging. Treat docs as code: versioned, reviewed, and validated in CI.

### Documentation Quality Gates
- **Linting:** `markdownlint` (enforced in CI, blocking)
- **Link validation:** All internal/external links must resolve
- **Code samples:** Must be runnable and produce expected output
- **Diagrams:** Keep Mermaid diagrams in sync with implementation

### Documentation Review Checklist
- [ ] Every new feature has corresponding documentation
- [ ] Examples are tested and up-to-date
- [ ] Architecture diagrams reflect current state
- [ ] ADRs document all major decisions
- [ ] Troubleshooting covers actual user issues
- [ ] All commands are copy-pasteable with correct flags

## Key Implementation Details

### Dependency Graph Generation
- Use Bazel aspects to traverse `java_library`, `java_binary`, `jvm_import`, and `maven_install` deps
- Collect: coordinates (group, artifact, version), PURLs, licenses, file SHA256
- Emit stable JSON per target to `bazel-out/.../<target>.deps.json`

### SBOM Generation
- `/tools/supplychain/write_sbom.py` converts `<target>.deps.json` → SPDX 2.3 JSON
- Include: Document, Packages, Files, Relationships (`CONTAINS`, `DEPENDS_ON`)
- Include license expressions (SPDX IDs), provenance (Bazel version, target label, commit)
- Optional: Flag `--cyclonedx` to emit CycloneDX JSON as secondary output

### SCA Integration
- Extract PURLs from SBOM, batch query OSV (or read offline DB)
- Output canonical `sca_findings.json`
- Map findings → SARIF (rules, results, `level`, CWEs)
- Point `artifactLocation` to package or manifest when possible

### Maven/JVM Specifics
- **Source of truth:** `maven_install.json` lockfile
- Parse lockfile for all resolved dependencies with exact versions
- Extract POM metadata for licenses, developers, SCM URLs
- Handle shaded/fat JARs by unpacking and reconstructing dependencies
- Support Kotlin, Scala, and Groovy artifacts

### Performance Optimization for Large Monorepos
- **Incremental analysis:** Only regenerate SBOMs for changed targets
- **Parallelization:** Bazel automatically parallelizes aspect analysis
- **Deduplication:** Store unique dep metadata once, reference by hash
- **Caching:** Full Bazel remote cache support for incremental builds
- **Target filtering:** Exclude test targets from production SBOMs

## Common Commands

### SBOM Generation
```bash
# Generate SBOMs for all targets
bazel build //:sbom_all

# Generate SBOM for single target
bazel build //app:myapp_sbom

# Generate with CycloneDX (in addition to SPDX)
bazel build //:sbom_all --cyclonedx

# Incremental (only changed targets)
bazel build //:sbom_all --config=supplychain-incremental
```

### Vulnerability Scanning
```bash
# Full SCA scan (OSV + NVD + GHSA)
bazel run //:sca_scan

# Scan with custom severity threshold
bazel run //:sca_scan -- --severity-threshold=high

# Offline scan (requires local database)
bazel run //:sca_scan -- --offline-mode --osv-db-path=/opt/osv-db
```

### License Analysis
```bash
# Generate license compliance report
bazel run //:license_report

# Check for license conflicts
bazel run //:license_report -- --check-conflicts

# Flag copyleft licenses
bazel run //:license_report -- --flag-copyleft
```

### Validation
```bash
# Validate all SBOMs against SPDX schema
bazel run //tools/supplychain/validators:validate_sbom -- bazel-bin/**/*.spdx.json

# Validate SARIF output
bazel run //tools/supplychain/validators:validate_sarif -- bazel-bin/sca_findings.sarif
```

## Code Style & Conventions

### Python Scripts
- Use type hints for all function signatures
- Include docstrings for all public functions
- Follow PEP 8 style guide
- Use `argparse` for command-line interfaces
- Handle errors gracefully with meaningful error messages

### Bazel Files
- Use lowercase, snake_case for rule names
- Document macros with usage examples
- Keep BUILD files minimal and readable
- Use aspects for cross-cutting concerns (SBOM, SCA)

### Documentation
- Use Mermaid for diagrams
- Keep examples copy-paste ready
- Include expected outputs for examples
- Update CHANGELOG.md following Keep a Changelog format
- Write ADRs for major architectural decisions

## CI/CD Integration

### Workflow Triggers
- **Every PR:** SBOM generation, SCA scan, SARIF upload
- **Push to main:** Full analysis, artifact upload, metrics aggregation
- **Weekly schedule:** Fresh CVE data updates

### Required Permissions
```yaml
permissions:
  contents: read
  security-events: write
  actions: read
  id-token: write  # For SLSA provenance signing
```

### Artifact Upload
- SBOMs (SPDX + CycloneDX)
- Dependency graphs (JSON + GraphML)
- SCA findings (JSON + SARIF)
- SLSA provenance (signed)
- License reports
- Metrics dashboard

## Guardrails & Best Practices

### Code Changes
- **Hermetic builds:** Only declared repositories, no network access beyond `http_archive`
- **Aspect-first:** Use aspects for dependency discovery, not shell scripts
- **Schema validation:** Validate all outputs (SPDX, SARIF, SLSA) in CI
- **Incremental builds:** Support incremental analysis for large repos

### Security
- Never commit secrets or credentials
- Sign provenance artifacts with Sigstore/cosign
- Validate all external inputs
- Use VEX statements to document false positives
- Enforce policy thresholds (max critical/high vulnerabilities)

### Performance
- Cache JAR metadata by SHA256
- Process JARs in parallel (thread pool)
- Use streaming JSON for large outputs
- Support offline mode for air-gapped environments

## Success Criteria

### Functional Requirements
- `bazel build //:sbom_all` succeeds locally and in CI
- Each target emits valid SPDX 2.3 JSON (schema-validated)
- SCA outputs `sca_findings.json` and `sca_findings.sarif`
- Dependency graph exports as both JSON and GraphML
- SLSA provenance generated for all deployable artifacts

### Performance Requirements
- Small repo (< 50 targets): **< 2 min** end-to-end with remote cache
- Medium repo (50-500 targets): **< 5 min** end-to-end with remote cache
- Large repo (500-5000 targets): **< 15 min** end-to-end with remote cache
- Incremental mode (PRs): **< 5 min** for typical changes

### Documentation Requirements
- All docs present and accurate
- CI enforces docs lint and fails on broken examples
- All code samples are runnable and produce expected output
- Mermaid diagrams render correctly and match implementation

## Working with GitHub Copilot

When suggesting code or documentation changes:

1. **Understand context:** Review related files in `tools/supplychain/` and `docs/`
2. **Follow patterns:** Match existing code style and structure
3. **Include tests:** Add unit tests in `tools/supplychain/tests/`
4. **Update docs:** Update relevant docs in `docs/` for any feature changes
5. **Validate outputs:** Ensure generated SBOMs/SARIF are schema-compliant
6. **Consider scale:** Design for large monorepos (1000+ targets, 1000+ deps)
7. **Security first:** Always consider supply chain security implications
8. **Examples:** Provide runnable examples in `examples/` directory

## Quick Reference

### Key Files to Modify
- **WORKSPACE:** For external dependencies
- **tools/supplychain/aspects.bzl:** For dependency traversal logic
- **tools/supplychain/write_sbom.py:** For SBOM generation logic
- **tools/supplychain/osv_query.py:** For vulnerability scanning
- **tools/supplychain/sarif_adapter.py:** For SARIF output generation
- **.github/workflows/supplychain.yml:** For CI/CD pipeline

### Key Documentation Files
- **README.md:** High-level overview and quickstart
- **docs/SUPPLY_CHAIN.md:** Detailed implementation guide
- **docs/ARCHITECTURE.md:** System design and data flows
- **docs/USAGE.md:** Command reference and examples
- **docs/TROUBLESHOOTING.md:** Common issues and solutions

### Testing Changes
```bash
# Build all SBOMs
bazel build //:sbom_all

# Run unit tests
bazel test //tools/supplychain/tests/...

# Validate SBOM schema
bazel run //tools/supplychain/validators:validate_sbom -- bazel-bin/**/*.spdx.json

# Run full SCA scan
bazel run //:sca_scan

# Lint documentation
npm run lint:docs  # or equivalent markdownlint command
```

## Non-Goals

- No ad-hoc scanners outside Bazel unless executed **via Bazel** for hermeticity
- No manual dependency lists maintained outside lockfiles
- No BCR (Bazel Central Registry) usage - use `http_archive` for `bazel-contrib/supply-chain`

## Additional Resources

- **SPDX Specification:** https://spdx.github.io/spdx-spec/
- **SARIF Specification:** https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html
- **SLSA Provenance:** https://slsa.dev/provenance/
- **OSV Database:** https://osv.dev/
- **Package URL (PURL):** https://github.com/package-url/purl-spec
- **Bazel Aspects:** https://bazel.build/extending/aspects
