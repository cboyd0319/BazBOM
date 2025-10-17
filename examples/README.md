# BazBOM Examples

This directory contains examples demonstrating BazBOM usage.

## Quick Demo

The `demo_workflow.sh` script demonstrates the complete BazBOM workflow:

```bash
cd examples
./demo_workflow.sh
```

This will:
1. Extract Maven dependencies from WORKSPACE
2. Generate SPDX 2.3 SBOM
3. Validate SBOM schema
4. Generate dependency graphs (JSON + GraphML)
5. Generate SLSA provenance
6. Run vulnerability scan via OSV API
7. Generate SARIF report
8. Validate SARIF schema

All artifacts are generated in `/tmp/` for inspection.

## Minimal Java Example

The `minimal_java` directory contains a simple Java application that demonstrates:
- Maven dependency on Guava
- Basic Java binary target
- SBOM generation for the application

### Building the example

```bash
cd minimal_java
bazel build :app
bazel run :app
```

### Generating SBOM for the example

From the repository root:

```bash
# Using Bazel (if configured)
bazel build //:minimal_java_sbom

# Or manually
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --output /tmp/deps.json

python3 tools/supplychain/write_sbom.py \
  --input /tmp/deps.json \
  --output /tmp/app.spdx.json \
  --name "minimal-java-app"
```

## What Each Tool Does

### extract_maven_deps.py
Parses WORKSPACE file to extract Maven artifacts declared in `maven_install()`.
Creates a JSON file with package metadata (group, artifact, version, PURL).

### write_sbom.py
Converts dependency JSON into SPDX 2.3 compliant SBOM.
Includes packages, relationships, and provenance metadata.

### graph_generator.py
Creates dependency graph visualizations:
- **JSON format**: Machine-readable graph with nodes and edges
- **GraphML format**: Import into Gephi, yEd, or other graph tools

### provenance_builder.py
Generates SLSA provenance v1.0 attestations.
Documents build environment, inputs, and materials.

### osv_query.py
Queries OSV (Open Source Vulnerabilities) database.
Checks each package for known security vulnerabilities.

### sarif_adapter.py
Converts vulnerability findings into SARIF 2.1.0 format.
Enables upload to GitHub Code Scanning for security alerts.

## Viewing Generated Artifacts

### SBOM (SPDX)
```bash
cat /tmp/demo_sbom.spdx.json | python3 -m json.tool | less
```

Key sections:
- `creationInfo`: When and how the SBOM was created
- `packages`: List of all dependencies with PURLs
- `relationships`: Dependency relationships (DEPENDS_ON, etc.)

### Dependency Graph (JSON)
```bash
cat /tmp/demo_graph.json | python3 -m json.tool | less
```

Structure:
- `nodes`: Packages with name, version, type
- `edges`: Dependencies between packages

### Dependency Graph (GraphML)
Import into visualization tools:
- **Gephi**: File → Open → Select .graphml file
- **yEd**: File → Open → Select .graphml file
- **Cytoscape**: File → Import → Network from File

### SLSA Provenance
```bash
cat /tmp/demo_provenance.json | python3 -m json.tool | less
```

Contains:
- Build environment (builder ID, timestamp)
- External parameters (repository, commit SHA)
- Resolved dependencies

### SCA Findings
```bash
cat /tmp/demo_sca.json | python3 -m json.tool | less
```

Structure:
- `packages_scanned`: Total packages checked
- `vulnerabilities_found`: Count of vulnerabilities
- `vulnerabilities`: Array of findings with CVE IDs, severity, remediation

### SARIF Report
```bash
cat /tmp/demo_findings.sarif | python3 -m json.tool | less
```

Compatible with:
- GitHub Code Scanning
- Azure DevOps
- Visual Studio Code SARIF Viewer extension
- Any SARIF 2.1.0 compatible tool

## Testing in CI/CD

See `.github/workflows/supplychain.yml` for GitHub Actions integration.

Key steps:
1. Generate SBOMs for all targets
2. Run SCA vulnerability scanning
3. Generate SARIF reports
4. Upload artifacts for review
5. Upload SARIF to GitHub Security tab

## Troubleshooting

### "Network error" during OSV scan
The OSV query requires internet access. In air-gapped environments, use offline mode (coming soon).

### "Invalid JSON" errors
Ensure all scripts are executable and Python 3.9+ is installed.

### "Missing dependency" errors
Run `pip install requests` if OSV scanning fails.

## Next Steps

1. **Integrate into your project**: Copy the tooling to your Bazel workspace
2. **Configure CI**: Adapt the GitHub Actions workflow to your needs
3. **Customize**: Add VEX statements, license checks, or policy enforcement
4. **Visualize**: Import GraphML into your preferred graph visualization tool
5. **Monitor**: Set up GitHub Code Scanning to track vulnerabilities over time

For more information, see the [documentation](../docs/README.md).
