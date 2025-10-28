# Test Fixtures

This directory contains sample data files used for testing BazBOM components.

## Files

### sample_maven_install.json
Sample Maven dependency data extracted from `maven_install.json` lockfile.
Contains example dependencies with versions, coordinates, and checksums.

**Usage:** Testing dependency extraction and parsing

### sample_sbom.spdx.json
Valid SPDX 2.3 SBOM document with sample package information.
Includes packages, relationships, and provenance metadata.

**Usage:** Testing SBOM generation, validation, and parsing

### sample_provenance.json
Valid SLSA v1.0 provenance attestation.
Contains build metadata, builder information, and subject artifacts.

**Usage:** Testing provenance generation and validation

### sample_osv_response.json
Sample OSV API response with vulnerability findings.
Contains vulnerability details, severity information, and remediation guidance.

**Usage:** Testing vulnerability scanning and SARIF generation

### enriched_finding_example.json
Complete enriched vulnerability findings with KEV, EPSS, GHSA, and VulnCheck data.
Demonstrates the full enrichment pipeline output with three example vulnerabilities:

- **P0-IMMEDIATE** (CVE-2021-44228 - Log4Shell): In KEV, 97.5% EPSS, weaponized exploit
- **P1-CRITICAL** (CVE-2022-42889 - Text4Shell): High EPSS (82.5%), public PoC
- **P3-MEDIUM** (CVE-2023-12345): Medium CVSS, very low EPSS (0.1%)

**Usage:** Testing enrichment pipeline, risk scoring, priority mapping, and enriched SARIF generation

## Creating New Fixtures

When adding new fixtures:

1. Base them on real outputs from the tools
2. Anonymize any sensitive information
3. Ensure they're valid according to relevant schemas
4. Document their purpose in this README
5. Keep files small (< 10KB) for fast tests

## Fixture Guidelines

- **Realistic**: Use realistic data structures and values
- **Minimal**: Include only what's needed for testing
- **Valid**: Ensure fixtures pass schema validation
- **Versioned**: Update when schema versions change
- **Documented**: Add comments explaining unusual values
