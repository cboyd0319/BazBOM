# BazBOM Orchestrated Scan Example

This example demonstrates the complete BazBOM integration plan with SBOM generation, SCA, Semgrep, and merged SARIF output.

## Prerequisites

- Java 11 or later
- Maven (for this example)
- Semgrep (install with `pip install semgrep`)

## Running the Example

```bash
# From the examples/orchestrated-scan directory
bazbom scan . --cyclonedx --with-semgrep --out-dir ./bazbom-output
```

## Expected Output

```
bazbom-output/
├── sbom/
│   ├── spdx.json              # SPDX 2.3 SBOM
│   └── cyclonedx.json         # CycloneDX 1.5 SBOM
├── findings/
│   ├── sca.sarif              # OSV/NVD/GHSA findings
│   ├── semgrep.sarif          # Semgrep findings
│   └── merged.sarif           # Merged SARIF for GitHub
└── enrich/
    └── depsdev.json           # deps.dev enrichment data
```

## GitHub Actions Integration

```yaml
- name: BazBOM Scan
  run: bazbom scan . --cyclonedx --with-semgrep

- uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: bazbom-output/findings/merged.sarif
```

## Features Demonstrated

- SBOM generation (SPDX + CycloneDX)
- Software Composition Analysis (SCA)
- Static analysis with Semgrep
- Merged SARIF 2.1.0 output
- deps.dev enrichment

See `docs/strategy/product-roadmap/BAZBOM_INTEGRATION_PLAN.md` for complete integration details.
