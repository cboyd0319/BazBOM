# SBOM Schema Files

This directory contains JSON schema files used for validating SBOM and related security artifacts.

## Schema Files

### SPDX 2.3 Schema
- **File:** `spdx-2.3-schema.json`
- **Purpose:** Validate SPDX Software Bill of Materials
- **Specification:** https://spdx.github.io/spdx-spec/v2.3/
- **Schema Source:** https://github.com/spdx/spdx-spec/tree/v2.3/schemas

### CycloneDX 1.5 Schema
- **File:** `cyclonedx-1.5-schema.json`
- **Purpose:** Validate CycloneDX BOMs
- **Specification:** https://cyclonedx.org/docs/1.5/
- **Schema Source:** https://github.com/CycloneDX/specification/tree/1.5/schema

### SARIF 2.1.0 Schema
- **File:** `sarif-2.1.0-schema.json`
- **Purpose:** Validate SARIF security analysis results
- **Specification:** https://docs.oasis-open.org/sarif/sarif/v2.1.0/
- **Schema Source:** https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json

### SLSA Provenance v1.0 Schema
- **File:** `slsa-provenance-v1.0-schema.json`
- **Purpose:** Validate SLSA provenance attestations
- **Specification:** https://slsa.dev/provenance/v1
- **Schema Source:** https://slsa.dev/provenance/v1

### CSAF VEX 2.0 Schema
- **File:** `csaf-vex-2.0-schema.json`
- **Purpose:** Validate VEX (Vulnerability Exploitability eXchange) statements
- **Specification:** https://docs.oasis-open.org/csaf/csaf/v2.0/
- **Schema Source:** https://docs.oasis-open.org/csaf/csaf/v2.0/

## Usage

These schemas are used by the validators in `tools/supplychain/validators/`:

```bash
# Validate SPDX SBOM
bazel run //tools/supplychain/validators:validate_sbom -- bazel-bin/**/*.spdx.json

# Validate SARIF report
bazel run //tools/supplychain/validators:validate_sarif -- bazel-bin/**/*.sarif

# Validate SLSA provenance
bazel run //tools/supplychain/validators:validate_provenance -- bazel-bin/**/*.provenance.json
```

## Updating Schemas

To update schemas to newer versions:

1. Download the latest schema from the official source
2. Update the filename to reflect the version
3. Update validators to use the new schema
4. Update this README with new links

## Schema Validation

All schemas should be valid JSON Schema Draft 7 or later.

