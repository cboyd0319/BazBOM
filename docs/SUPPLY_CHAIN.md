# Supply Chain Security

This document describes BazBOM's approach to software supply chain security, including SBOM generation and Software Composition Analysis (SCA).

## Overview

BazBOM provides a complete supply chain security solution:

1. **Dependency Discovery** - Automatically identify all dependencies
2. **SBOM Generation** - Create standards-compliant inventory documents
3. **Vulnerability Scanning** - Identify known security issues
4. **Reporting** - Generate actionable security reports
5. **Integration** - Upload to GitHub Code Scanning for visibility

## SBOM Architecture

### What is an SBOM?

A Software Bill of Materials (SBOM) is a comprehensive inventory of all components in a software application. It includes:

- Package names and versions
- Licenses
- Suppliers/originators
- Dependencies and relationships
- File checksums
- Security metadata

### Why SBOMs Matter

- **Vulnerability Management**: Quickly identify affected components when new CVEs are disclosed
- **License Compliance**: Track open source licenses across dependencies
- **Supply Chain Transparency**: Understand what's in your software
- **Regulatory Compliance**: Meet requirements like EO 14028 (US) and NIS2 (EU)

### SPDX Format

BazBOM generates SBOMs in SPDX (Software Package Data Exchange) format:

- **Standard**: ISO/IEC 5962:2021
- **Version**: SPDX 2.3
- **Format**: JSON
- **License**: CC0-1.0 (for SBOM documents themselves)

**Example SPDX Document**:
```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "my-application-1.0.0",
  "documentNamespace": "https://example.com/sboms/my-app-1.0.0-uuid",
  "creationInfo": {
    "created": "2025-10-17T12:00:00Z",
    "creators": ["Tool: BazBOM"],
    "licenseListVersion": "3.21"
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-org.example:my-library:1.2.3",
      "name": "my-library",
      "versionInfo": "1.2.3",
      "supplier": "Organization: Example Corp",
      "downloadLocation": "https://repo1.maven.org/...",
      "filesAnalyzed": false,
      "licenseConcluded": "Apache-2.0",
      "licenseDeclared": "Apache-2.0",
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:maven/org.example/my-library@1.2.3"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-Package-root"
    },
    {
      "spdxElementId": "SPDXRef-Package-root",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-org.example:my-library:1.2.3"
    }
  ]
}
```

### SBOM Generation Process

BazBOM uses Bazel aspects to generate SBOMs:

```
1. Bazel aspect traverses build graph
   └─ For each dependency:
      ├─ Extract package coordinates (group:artifact:version)
      ├─ Determine license (from POM or heuristics)
      ├─ Generate checksums (if available)
      └─ Collect metadata

2. Aspect aggregates all dependency info

3. write_sbom.py converts to SPDX format
   ├─ Create SPDX document structure
   ├─ Add package entries
   ├─ Add relationship entries (DEPENDS_ON)
   └─ Add creation metadata

4. Output written to bazel-bin/
```

## Software Composition Analysis (SCA)

### What is SCA?

Software Composition Analysis identifies security vulnerabilities, license issues, and quality problems in open source dependencies.

### OSV Database Integration

BazBOM integrates with [OSV (Open Source Vulnerabilities)](https://osv.dev/):

- **Coverage**: 20+ ecosystems (Maven, npm, PyPI, Go, etc.)
- **Sources**: CVE, GitHub Security Advisories, ecosystem-specific databases
- **API**: Free, public API with no authentication required
- **Updates**: Continuously updated as new vulnerabilities are disclosed

### Vulnerability Scanning Process

```
1. Read SBOM files (SPDX JSON)

2. For each package:
   ├─ Extract package URL (purl)
   ├─ Query OSV API: POST /v1/query
   │  └─ Request: {"package": {"name": "...", "ecosystem": "Maven"}, "version": "..."}
   └─ Receive vulnerability list

3. For each vulnerability:
   ├─ Extract CVE ID, severity, description
   ├─ Map to SARIF format
   └─ Add to report

4. Write SARIF file
```

### SARIF Format

SARIF (Static Analysis Results Interchange Format) is the standard for security findings:

- **Standard**: OASIS SARIF 2.1.0
- **Purpose**: Interoperable security tool output
- **Integration**: Native GitHub Code Scanning support

**Example SARIF Document**:
```json
{
  "version": "2.1.0",
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "BazBOM SCA",
          "version": "1.0.0",
          "informationUri": "https://github.com/cboyd0319/BazBOM"
        }
      },
      "results": [
        {
          "ruleId": "CVE-2021-12345",
          "level": "error",
          "message": {
            "text": "Arbitrary code execution vulnerability in my-library 1.2.3"
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "pom.xml"
                }
              }
            }
          ],
          "properties": {
            "severity": "CRITICAL",
            "package": "org.example:my-library",
            "version": "1.2.3",
            "fixedVersion": "1.2.4"
          }
        }
      ]
    }
  ]
}
```

## Security Workflows

### Local Development

```bash
# Generate SBOMs
bazel build //:sbom_all

# Scan for vulnerabilities
bazel run //:sca_from_sbom

# Review findings
cat bazel-bin/**/*.sarif.json | jq '.runs[0].results'
```

### CI/CD Pipeline

The `.github/workflows/supplychain.yml` workflow:

1. **Build**: Generate SBOMs for all targets
2. **Scan**: Query OSV for vulnerabilities
3. **Report**: Upload SARIF to GitHub Code Scanning
4. **Artifact**: Store SBOMs and reports as workflow artifacts

### GitHub Code Scanning

SARIF reports uploaded to GitHub provide:

- **Security Tab**: Centralized view of all alerts
- **PR Annotations**: Comments on PRs introducing new vulnerabilities
- **Trends**: Track security posture over time
- **Dismissal**: Mark false positives or accepted risks

## Best Practices

### 1. Regular Scanning

Run SCA on every build:
```yaml
# .github/workflows/ci.yml
- name: Security Scan
  run: bazel run //:sca_from_sbom
```

### 2. Dependency Updates

Keep dependencies current:
```bash
# Update Maven dependencies
bazel run @unpinned_maven//:pin

# Review and commit changes
git diff maven_install.json
```

### 3. Vulnerability Triage

Establish a process for addressing vulnerabilities:

1. **Critical**: Patch within 24 hours
2. **High**: Patch within 7 days
3. **Medium**: Patch within 30 days
4. **Low**: Patch in next release

### 4. SBOM Distribution

Publish SBOMs with releases:
```bash
# Include SBOM in release artifacts
gh release upload v1.0.0 bazel-bin/myapp.spdx.json
```

### 5. License Compliance

Review licenses in SBOMs:
```bash
# Extract all licenses
jq -r '.packages[].licenseDeclared' bazel-bin/**/*.spdx.json | sort -u
```

## Advanced Features

### Custom Vulnerability Policies

Create policy files to customize SCA behavior:

```json
{
  "ignore": ["CVE-2021-12345"],
  "fail_on": ["critical", "high"],
  "exceptions": {
    "org.example:legacy-lib": ["CVE-2020-99999"]
  }
}
```

Apply policies:
```bash
bazel run //:sca_from_sbom -- --policy policy.json
```

### Integration with Other Tools

Export SBOMs for use with commercial tools:

- **Dependency-Track**: SBOM analysis and visualization
- **Snyk**: Additional vulnerability sources
- **JFrog Xray**: License and security scanning
- **Grype**: Alternative vulnerability scanner

### Custom SBOM Metadata

Enhance SBOMs with custom metadata:

```python
# tools/supplychain/write_sbom.py
sbom_data["packages"].append({
    "SPDXID": "SPDXRef-Package-...",
    # ... standard fields ...
    "annotations": [
        {
            "annotator": "Tool: internal-review",
            "annotationType": "REVIEW",
            "comment": "Approved for production use",
            "annotationDate": "2025-10-17T12:00:00Z"
        }
    ]
})
```

## Compliance

BazBOM helps meet compliance requirements:

- **NTIA Minimum Elements**: All required SBOM fields included
- **EO 14028**: SBOM generation for federal software
- **NIS2**: Supply chain security for EU critical infrastructure
- **SLSA**: Build provenance and dependency tracking

## Troubleshooting

See [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common SCA issues:

- OSV API rate limiting
- Network connectivity issues
- SPDX validation errors
- SARIF upload failures
