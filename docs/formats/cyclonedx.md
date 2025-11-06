# CycloneDX Format

BazBOM's CycloneDX 1.5 output format specification.

## Overview

**Format:** CycloneDX 1.5 (JSON/XML)  
**Spec:** <https://cyclonedx.org/specification/overview/>  
**Output:** `sbom.cyclonedx.json`  
**Status:** Production-ready, optional format

**Why CycloneDX?**
- Alternative to SPDX for organizations preferring CycloneDX
- Rich vulnerability and licensing metadata
- Broad tooling ecosystem (Dependency-Track, etc.)
- OWASP-backed standard

## Generation

```bash
# Generate CycloneDX SBOM
bazbom scan . --format cyclonedx

# Output: sbom.cyclonedx.json
```

**Default:** BazBOM uses SPDX 2.3 as primary format. Use `--format cyclonedx` to generate CycloneDX instead.

## Field Mapping

| BazBOM Data | CycloneDX Field | Notes |
|-------------|-----------------|-------|
| Artifact name | `components[].name` | Maven: artifactId |
| Version | `components[].version` | Exact resolved version |
| PURL | `components[].purl` | pkg:maven/... format |
| License | `components[].licenses[]` | SPDX license ID |
| Hash | `components[].hashes[]` | SHA256, SHA1, MD5 |
| Dependencies | `dependencies[]` | Ref to component bom-ref |

## Example Output

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "version": 1,
  "metadata": {
    "timestamp": "2025-11-06T23:33:08Z",
    "tools": [{
      "name": "BazBOM",
      "version": "1.0.0"
    }],
    "component": {
      "type": "application",
      "name": "myapp",
      "version": "1.0.0"
    }
  },
  "components": [
    {
      "type": "library",
      "name": "log4j-core",
      "version": "2.17.0",
      "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0",
      "licenses": [{
        "license": {
          "id": "Apache-2.0"
        }
      }],
      "hashes": [{
        "alg": "SHA-256",
        "content": "abc123..."
      }]
    }
  ],
  "dependencies": [
    {
      "ref": "pkg:maven/myapp@1.0.0",
      "dependsOn": [
        "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0"
      ]
    }
  ]
}
```

## Differences from SPDX

| Feature | SPDX 2.3 | CycloneDX 1.5 |
|---------|----------|---------------|
| License format | SPDX IDs | SPDX IDs or expressions |
| Component ID | SPDXRef-* | bom-ref (PURL) |
| Relationships | Typed relationships | dependsOn array |
| Vulnerability data | External | Inline (optional) |
| File-level analysis | Supported | Supported |

**When to use CycloneDX:**
- Dependency-Track integration
- OWASP ecosystem tools
- Prefer inline vulnerability data

**When to use SPDX:**
- Government compliance (NTIA)
- Broader tooling support
- License compliance focus

## Validation

```bash
# BazBOM validates by default
bazbom scan . --format cyclonedx --validate-schemas

# Manual validation with cyclonedx-cli
npm install -g @cyclonedx/cyclonedx-cli
cyclonedx validate --input-file sbom.cyclonedx.json
```

## Conversion

```bash
# SPDX → CycloneDX
# Generate both formats
bazbom scan . --format spdx
bazbom scan . --format cyclonedx

# Use external tools for conversion if needed
```

## Integration with Dependency-Track

See [../INTEGRATIONS.md](../INTEGRATIONS.md#dependency-track) for upload instructions.

## Next Steps

- [SPDX format](../FORMAT_SPDX.md) - Primary format
- [SARIF format](sarif.md) - Vulnerability reporting
- [Validation guide](../operations/validation.md) - Schema validation
