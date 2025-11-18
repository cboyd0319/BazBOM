# CycloneDX Format

BazBOM's CycloneDX 1.5 output format specification.

## Overview

**Format:** CycloneDX 1.5 (JSON and XML)
**Spec:** <https://cyclonedx.org/specification/overview/>
**Output:** `sbom.cyclonedx.json` (JSON) or `sbom.cyclonedx.xml` (XML)
**Status:** Production-ready, optional format

**Why CycloneDX?**
- Alternative to SPDX for organizations preferring CycloneDX
- Rich vulnerability and licensing metadata
- Broad tooling ecosystem (Dependency-Track, etc.)
- OWASP-backed standard
- Supports 13 build systems and 7 language ecosystems

## Generation

BazBOM supports two CycloneDX output formats:

```bash
# JSON format (default)
bazbom scan --format cyclonedx
# Output: sbom.cyclonedx.json

# Shorthand flag
bazbom scan --cyclonedx
# Output: sbom.cyclonedx.json

# XML format
bazbom scan --format cyclonedx-xml
# Output: sbom.cyclonedx.xml
```

**JSON format** is recommended for modern tooling and Dependency-Track.
**XML format** is useful for legacy systems and XML-based processing pipelines.

**Default:** BazBOM uses SPDX 2.3 as primary format. Use `--format cyclonedx` or `--cyclonedx` to generate CycloneDX instead.

## Enhanced Features

### SHA256 Checksum Fetching

BazBOM can optionally fetch SHA256 checksums from package registries:

```bash
# Fetch checksums from package registries (slower but adds integrity)
bazbom scan --format cyclonedx --fetch-checksums

# Without checksums (faster, default)
bazbom scan --format cyclonedx
```

**Supported ecosystems:** Maven, npm, PyPI, Cargo, RubyGems (see SPDX docs for full table)

### Download Location URLs

BazBOM automatically populates `externalReferences` with download URLs for all packages using ecosystem-specific registry patterns.

### Polyglot Ecosystem Support

BazBOM supports **7 language ecosystems** beyond JVM:

- Maven (JVM) - pom.xml, maven_install.json
- Gradle (JVM) - build.gradle, build.gradle.kts
- npm/Yarn/pnpm (JavaScript/TypeScript)
- Python - requirements.txt, Pipfile.lock, poetry.lock
- Go - go.mod, go.sum
- Rust - Cargo.toml, Cargo.lock
- Ruby - Gemfile, Gemfile.lock
- PHP - composer.json, composer.lock

All ecosystems are merged into a single unified CycloneDX SBOM.

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

### CycloneDX XML Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<bom xmlns="http://cyclonedx.org/schema/bom/1.5" version="1" serialNumber="urn:uuid:12345678-1234-5678-1234-567812345678">
  <metadata>
    <timestamp>2025-11-18T23:33:08Z</timestamp>
    <tools>
      <tool>
        <name>BazBOM</name>
        <version>6.5.0</version>
      </tool>
    </tools>
  </metadata>
  <components>
    <component type="library">
      <name>log4j-core</name>
      <version>2.17.0</version>
      <purl>pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0</purl>
      <licenses>
        <license>
          <id>Apache-2.0</id>
        </license>
      </licenses>
      <hashes>
        <hash alg="SHA-256">f4f3e0d3c9f9b5e7d6f1e2c3a4b5c6d7e8f9a0b1c2d3e4f5g6h7i8j9k0l1m2n3</hash>
      </hashes>
      <externalReferences>
        <reference type="distribution">
          <url>https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.17.0/log4j-core-2.17.0.jar</url>
        </reference>
      </externalReferences>
    </component>
  </components>
</bom>
```

**Use cases for XML format:**
- Legacy systems requiring XML
- XML processing pipelines
- XSLT transformations
- XML databases (eXist-db, BaseX)

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
