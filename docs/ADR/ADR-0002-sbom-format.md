# ADR-0002: SBOM Format Selection

**Status**: Accepted

**Date**: 2025-10-17

**Context**: We need to choose a standard format for Software Bill of Materials (SBOM) generation that balances interoperability, tooling support, and compliance requirements.

## Decision

We will use **SPDX 2.3 in JSON format** as our primary SBOM format.

## Rationale

### Options Considered

1. **SPDX 2.3 (Chosen)**
   -  ISO/IEC standard (5962:2021)
   -  Strong license compliance features
   -  Excellent tooling ecosystem
   -  Government/regulatory acceptance
   -  Well-defined JSON schema
   -  Comprehensive package metadata

2. **CycloneDX**
   -  Purpose-built for security
   -  Good vulnerability metadata
   -  Weaker license compliance features
   -  Less mature standard

3. **SWID (Software Identification Tags)**
   -  XML-focused (less developer-friendly)
   -  Limited security metadata
   -  Less tooling support

## Comparison Matrix

<table>
  <thead>
    <tr>
      <th>Feature</th>
      <th>SPDX 2.3</th>
      <th>CycloneDX</th>
      <th>SWID</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>License Compliance</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>Security Metadata</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>Standard Status</td>
      <td>ISO/IEC</td>
      <td>OWASP</td>
      <td>ISO/IEC</td>
    </tr>
    <tr>
      <td>Tooling Ecosystem</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>JSON Support</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
    <tr>
      <td>Adoption</td>
      <td></td>
      <td></td>
      <td></td>
    </tr>
  </tbody>
</table>

## Implementation

### SPDX Document Structure

```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "Package-SBOM",
  "documentNamespace": "https://example.com/sboms/...",
  "creationInfo": {
    "created": "2025-10-17T12:00:00Z",
    "creators": ["Tool: BazBOM"]
  },
  "packages": [...],
  "relationships": [...]
}
```

### Package Identification

We use Package URLs (purl) for ecosystem-agnostic package identification:

```text
pkg:maven/org.example/my-library@1.2.3
pkg:npm/lodash@4.17.21
pkg:pypi/requests@2.28.0
```

### License Handling

SPDX license identifiers from the [SPDX License List](https://spdx.org/licenses/):

- Use exact identifiers: `Apache-2.0`, `MIT`, `GPL-3.0-only`
- Use `NOASSERTION` when license cannot be determined
- Support compound licenses: `Apache-2.0 OR MIT`

## Consequences

### Positive

- **Compliance Ready**: Meets NTIA minimum elements
- **Interoperability**: Works with many commercial and open-source tools
- **Validation**: Strong schema and validation tools available
- **License Focus**: Best-in-class for license compliance tracking
- **Relationship Modeling**: Rich dependency relationship representation

### Negative

- **Verbosity**: SPDX documents can be large for projects with many dependencies
- **Complexity**: Full SPDX spec is comprehensive (but we use a subset)
- **Security Metadata**: Less detailed than CycloneDX for vulnerability info

### Neutral

- **Format Choice**: JSON chosen over RDF/XML for developer friendliness

## Future Considerations

### SPDX 3.0 Migration

SPDX 3.0 is in development with:

- Improved security metadata
- Better link integrity
- Enhanced relationship modeling

**Action**: Monitor SPDX 3.0 spec; plan migration when stable and tools mature.

### Multi-Format Support

**UPDATE (2025-10-17)**: Multi-format support has been implemented.

BazBOM now generates both SPDX and CycloneDX formats:

```bash
# Generate SPDX 2.3 (primary format)
bazel build //:workspace_sbom

# Generate CycloneDX 1.5 (optional format)
bazel build //:workspace_sbom_cyclonedx

# Generate both formats
bazel build //:sbom_all_formats
```

Implementation in `tools/supplychain/write_sbom.py`:

```python
def write_sbom(data, format="spdx"):
    if format == "spdx":
        return generate_spdx_document(data)
    elif format == "cyclonedx":
        return generate_cyclonedx_document(data)
```

**Rationale for dual format support:**
- SPDX remains primary for legal compliance and regulatory requirements
- CycloneDX provided as optional format for security-focused tools
- Zero overhead when CycloneDX not needed (opt-in via target)
- Maintains single source of truth (dependency data) for both formats

### Format Conversion

Use existing tools for format conversion when needed:

```bash
# SPDX to CycloneDX
sbom-utility convert --input-file package.spdx.json \
  --output-file package.cdx.json --format cyclonedx

# CycloneDX to SPDX (limited)
cyclonedx-cli convert --input-file package.cdx.json \
  --output-file package.spdx.json --output-format spdxjson
```

## Validation Strategy

### Schema Validation

```bash
# Validate against JSON schema
check-jsonschema \
  --schemafile https://raw.githubusercontent.com/spdx/spdx-spec/v2.3/schemas/spdx-schema.json \
  package.spdx.json
```

### NTIA Compliance Validation

Ensure all NTIA minimum elements are present:

1. Supplier Name
2. Component Name
3. Version
4. Other Unique Identifiers (purl)
5. Dependency Relationships
6. SBOM Author
7. Timestamp

### Tool Compatibility Testing

Test with multiple SBOM consumers:

- Dependency-Track
- SPDX Tools
- GitHub Dependency Graph
- OSV Scanner
- Grype

## Implementation Notes

### Generation Process

```text
1. Bazel aspect collects dependency data
2. write_sbom.py processes data
3. Generate SPDX package entries
4. Create dependency relationships (DEPENDS_ON)
5. Add document metadata
6. Validate against schema
7. Write JSON file
```

### Performance Considerations

- Stream large SBOMs (don't load entire document in memory)
- Cache package metadata lookups
- Parallelize SBOM generation for multiple targets

### Testing

```python
# Test SBOM generation
def test_spdx_generation():
    sbom = generate_sbom(test_deps)
    assert sbom["spdxVersion"] == "SPDX-2.3"
    assert len(sbom["packages"]) == expected_count
    validate_spdx_schema(sbom)
```

## References

- [SPDX 2.3 Specification](https://spdx.github.io/spdx-spec/v2.3/)
- [NTIA SBOM Minimum Elements](https://www.ntia.gov/report/2021/minimum-elements-software-bill-materials-sbom)
- [Package URL Specification](https://github.com/package-url/purl-spec)
- [SPDX License List](https://spdx.org/licenses/)
- [CycloneDX Specification](https://cyclonedx.org/specification/overview/) (for comparison)

## Review Date

**Next Review**: 2026-01-01 (or when SPDX 3.0 is stable)
