# SPDX 2.3 Format

How BazBOM maps JVM dependency data to SPDX 2.3 specification.

## Overview

**Format:** SPDX 2.3 (JSON)  
**Spec:** <https://spdx.github.io/spdx-spec/v2.3/>  
**Primary output:** `sbom.spdx.json`  
**Status:** Production-ready, NTIA-compliant

**Why SPDX 2.3?**
- Industry standard for SBOMs
- Government mandate (NTIA minimum elements)
- Broad tooling support (Dependency-Track, GUAC, etc.)
- Machine-readable and human-reviewable

## Field Mapping

### Document-Level Fields

| BazBOM Data | SPDX Field | Example | Notes |
|-------------|------------|---------|-------|
| Project name | `name` | `"MyApp-SBOM"` | From build file or detected |
| SBOM ID | `SPDXID` | `"SPDXRef-DOCUMENT"` | Always this value |
| Version | `spdxVersion` | `"SPDX-2.3"` | Fixed |
| Creator | `creationInfo.creators` | `["Tool: BazBOM-1.0.0"]` | Tool name + version |
| Created | `creationInfo.created` | `"2025-11-06T23:33:08Z"` | ISO 8601 timestamp |
| Namespace | `documentNamespace` | `"https://bazbom.io/sbom/<uuid>"` | Unique per SBOM |

### Package-Level Fields

| BazBOM Data | SPDX Field | Example | Notes |
|-------------|------------|---------|-------|
| Artifact name | `packages[].name` | `"log4j-core"` | Maven: artifactId |
| Version | `packages[].versionInfo` | `"2.17.0"` | Exact resolved version |
| Source URL | `packages[].downloadLocation` | `"https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.17.0/log4j-core-2.17.0.jar"` | Maven Central or VCS |
| License | `packages[].licenseConcluded` | `"Apache-2.0"` | SPDX license ID |
| License declared | `packages[].licenseDeclared` | `"Apache-2.0"` | From POM/manifest |
| Hash | `packages[].checksums[]` | `[{"algorithm": "SHA256", "checksumValue": "abc123..."}]` | From lockfile or computed |
| PURL | `packages[].externalRefs[]` | `{"referenceType": "purl", "referenceLocator": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0"}` | Package URL |
| Supplier | `packages[].supplier` | `"Organization: Apache Software Foundation"` | From POM |
| Home page | `packages[].homepage` | `"https://logging.apache.org/log4j/2.x/"` | From POM |

### Relationship Fields

| BazBOM Data | SPDX Field | Example | Notes |
|-------------|------------|---------|-------|
| Direct dependency | `relationships[]` | `{"spdxElementId": "SPDXRef-Package-app", "relationshipType": "DEPENDS_ON", "relatedSpdxElement": "SPDXRef-Package-log4j-core"}` | Top-level deps |
| Transitive | `relationships[]` | Same structure | All deps included |
| Runtime scope | `relationships[]` | `"RUNTIME_DEPENDENCY_OF"` | Shipped in production |
| Test scope | `relationships[]` | `"TEST_DEPENDENCY_OF"` | Test-only (excluded by default) |
| Build scope | `relationships[]` | `"BUILD_DEPENDENCY_OF"` | Build tools only |

## Example SBOM

### Minimal Example

```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "myapp-sbom",
  "documentNamespace": "https://bazbom.io/sbom/12345678-1234-1234-1234-123456789abc",
  "creationInfo": {
    "created": "2025-11-06T23:33:08Z",
    "creators": [
      "Tool: BazBOM-1.0.0"
    ]
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-app",
      "name": "myapp",
      "versionInfo": "1.0.0",
      "downloadLocation": "NOASSERTION",
      "filesAnalyzed": false
    },
    {
      "SPDXID": "SPDXRef-Package-log4j-core",
      "name": "log4j-core",
      "versionInfo": "2.17.0",
      "downloadLocation": "https://repo1.maven.org/maven2/org/apache/logging/log4j/log4j-core/2.17.0/log4j-core-2.17.0.jar",
      "filesAnalyzed": false,
      "licenseConcluded": "Apache-2.0",
      "licenseDeclared": "Apache-2.0",
      "checksums": [
        {
          "algorithm": "SHA256",
          "checksumValue": "f4f3e0d3c9f9b5e7d6f1e2c3a4b5c6d7e8f9a0b1c2d3e4f5g6h7i8j9k0l1m2n3"
        }
      ],
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:maven/org.apache.logging.log4j/log4j-core@2.17.0"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-Package-app"
    },
    {
      "spdxElementId": "SPDXRef-Package-app",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-log4j-core"
    }
  ]
}
```

### Complete Example with Transitive Dependencies

```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "spring-boot-app-sbom",
  "documentNamespace": "https://bazbom.io/sbom/87654321-4321-4321-4321-210987654321",
  "creationInfo": {
    "created": "2025-11-06T23:33:08Z",
    "creators": [
      "Tool: BazBOM-1.0.0",
      "Organization: Example Corp"
    ],
    "licenseListVersion": "3.21"
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-app",
      "name": "spring-boot-app",
      "versionInfo": "1.0.0",
      "supplier": "Organization: Example Corp",
      "downloadLocation": "NOASSERTION",
      "filesAnalyzed": false,
      "homepage": "https://example.com/app"
    },
    {
      "SPDXID": "SPDXRef-Package-spring-boot-starter-web",
      "name": "spring-boot-starter-web",
      "versionInfo": "2.7.0",
      "supplier": "Organization: Pivotal Software, Inc.",
      "downloadLocation": "https://repo1.maven.org/maven2/org/springframework/boot/spring-boot-starter-web/2.7.0/spring-boot-starter-web-2.7.0.jar",
      "filesAnalyzed": false,
      "licenseConcluded": "Apache-2.0",
      "licenseDeclared": "Apache-2.0",
      "checksums": [
        {
          "algorithm": "SHA256",
          "checksumValue": "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6"
        }
      ],
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:maven/org.springframework.boot/spring-boot-starter-web@2.7.0"
        }
      ]
    },
    {
      "SPDXID": "SPDXRef-Package-logback-classic",
      "name": "logback-classic",
      "versionInfo": "1.2.11",
      "downloadLocation": "https://repo1.maven.org/maven2/ch/qos/logback/logback-classic/1.2.11/logback-classic-1.2.11.jar",
      "filesAnalyzed": false,
      "licenseConcluded": "EPL-1.0 OR LGPL-2.1-only",
      "licenseDeclared": "EPL-1.0 OR LGPL-2.1-only",
      "checksums": [
        {
          "algorithm": "SHA256",
          "checksumValue": "b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6a1"
        }
      ],
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:maven/ch.qos.logback/logback-classic@1.2.11"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-Package-app"
    },
    {
      "spdxElementId": "SPDXRef-Package-app",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-spring-boot-starter-web"
    },
    {
      "spdxElementId": "SPDXRef-Package-spring-boot-starter-web",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-logback-classic"
    }
  ]
}
```

## Known Gaps

### Current Limitations

| Field | Status | Notes |
|-------|--------|-------|
| `filesAnalyzed` | Always `false` | BazBOM doesn't analyze JAR contents (use Syft for that) |
| `files[]` | Not included | Package-level SBOM only |
| `snippets[]` | Not included | Not applicable for JVM deps |
| `annotations[]` | Not included | May add for VEX integration |
| `originator` | Omitted | Use `supplier` instead |
| `sourceInfo` | Omitted | Source URL in `downloadLocation` |
| `comment` | Omitted | Reserved for future metadata |

### NTIA Minimum Elements

**Required by NTIA:** ✅ All satisfied

| Element | Field | Status |
|---------|-------|--------|
| Supplier name | `packages[].supplier` | ✅ From POM |
| Component name | `packages[].name` | ✅ Always |
| Version | `packages[].versionInfo` | ✅ Always |
| Dependencies | `relationships[]` | ✅ Full graph |
| Author | `creationInfo.creators` | ✅ BazBOM |
| Timestamp | `creationInfo.created` | ✅ ISO 8601 |
| Unique ID | `documentNamespace` | ✅ UUID-based |

### Additional Features

- **File-level hashes** - For shaded/fat JARs
- **Annotations** - For VEX statements
- **Package verification** - Signature validation
- **External document refs** - For multi-SBOM linking

## Validation

**Schema validation:**
```bash
# BazBOM validates by default
bazbom scan . --validate-schemas

# Manual validation with spdx-tools
docker run --rm -v $(pwd):/work spdx/spdx-tools-java verify /work/sbom.spdx.json
```

**Common validation errors:**

| Error | Cause | Fix |
|-------|-------|-----|
| Missing `versionInfo` | Dependency has no version | Update lockfile |
| Invalid `SPDXID` | Duplicate ID | BazBOM bug - report it |
| Invalid `downloadLocation` | Malformed URL | Check Maven repo config |
| Missing `licenseConcluded` | License not detected | Add `<license>` to POM |

## Differences from Other Tools

### Syft

**Syft:** Scans built JARs (post-build)  
**BazBOM:** Scans build system (build-time)

**Result:**
- Syft: May miss transitive deps or include test deps
- BazBOM: Accurate dependency graph with scopes

### CycloneDX Gradle Plugin

**CycloneDX:** Gradle-specific, CycloneDX format  
**BazBOM:** Maven + Gradle + Bazel, SPDX primary

**Result:**
- CycloneDX: Gradle only
- BazBOM: Universal JVM build systems

### Trivy

**Trivy:** Container scanning focus  
**BazBOM:** Build-time JVM dependency focus

**Result:**
- Trivy: Best for container images
- BazBOM: Best for JVM source projects

**Gotcha:** Use both together:
```bash
# Build-time SBOM
bazbom scan .

# Container SBOM
trivy image --format spdx myapp:latest
```

## Conversion to Other Formats

### SPDX → CycloneDX

```bash
# BazBOM natively supports both
bazbom scan . --format cyclonedx

# Output: sbom.cyclonedx.json
```

### SPDX → CSV

```bash
# Extract packages to CSV
jq -r '.packages[] | [.name, .versionInfo, .licenseConcluded] | @csv' sbom.spdx.json > packages.csv
```

### SPDX → GraphML

```bash
# Convert relationships to graph
# Requires custom script or tool like spdx-to-graphml
```

## Related Specs

- [SPDX 2.3 Specification](https://spdx.github.io/spdx-spec/v2.3/)
- [NTIA Minimum Elements](https://www.ntia.gov/files/ntia/publications/sbom_minimum_elements_report.pdf)
- [Package URL (PURL) Spec](https://github.com/package-url/purl-spec)
- [SPDX License List](https://spdx.org/licenses/)

## Next Steps

- [CycloneDX format](formats/cyclonedx.md) - Alternative format
- [SARIF format](formats/sarif.md) - Vulnerability reporting
- [Validation guide](operations/validation.md) - Schema validation
- [Architecture](ARCHITECTURE.md) - Data model
