# BazBOM Examples

This directory contains examples demonstrating BazBOM usage with real-world scenarios.

## Quick Demo

The `demo_workflow.sh` script demonstrates the complete BazBOM workflow:

```bash
cd examples
./demo_workflow.sh
```

This will:
1. Extract Maven dependencies from WORKSPACE and maven_install.json
2. Generate SPDX 2.3 SBOM with transitive dependencies
3. Validate SBOM schema
4. Generate dependency graphs (JSON + GraphML) with depth tracking
5. Generate SLSA provenance
6. Run vulnerability scan via OSV API
7. Generate SARIF report for GitHub Code Scanning
8. Validate SARIF schema

All artifacts are generated in `/tmp/` for inspection.

## What's New: Enhanced Dependency Analysis

BazBOM now leverages **maven_install.json** as the source of truth, providing:

✅ **Complete Transitive Dependencies** - All 7 dependencies (not just direct)
✅ **SHA256 Checksums** - Verification for all artifacts
✅ **Dependency Relationships** - Full parent→child mapping
✅ **Depth Tracking** - Blast radius analysis (max depth: 2)
✅ **Direct vs Transitive** - Clear classification

### Example Output

```json
{
  "statistics": {
    "total_packages": 7,
    "max_depth": 2,
    "direct_dependencies": 1,
    "transitive_dependencies": 6
  },
  "graph": {
    "nodes": [
      {"name": "guava", "depth": 1, "is_direct": true},
      {"name": "failureaccess", "depth": 2, "is_direct": false}
    ]
  }
}
```

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
# Using Bazel (recommended)
bazel build //:workspace_sbom

# View the SBOM
cat bazel-bin/workspace_sbom.spdx.json | python3 -m json.tool

# Or manually with enhanced extraction
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --maven-install-json maven_install.json \
  --output /tmp/deps.json

python3 tools/supplychain/write_sbom.py \
  --input /tmp/deps.json \
  --output /tmp/app.spdx.json \
  --name "minimal-java-app"
```

## What Each Tool Does

### extract_maven_deps.py ⭐ ENHANCED
**NEW:** Now reads from maven_install.json (preferred) or WORKSPACE (fallback).

Extracts complete dependency information:
- ✅ All transitive dependencies from lockfile
- ✅ SHA256 checksums for verification
- ✅ Dependency relationships (parent→child)
- ✅ Direct vs transitive classification

**Usage:**
```bash
# With maven_install.json (recommended)
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --maven-install-json maven_install.json \
  --output deps.json

# Fallback to WORKSPACE only
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --output deps.json \
  --prefer-lockfile=false
```

### write_sbom.py ⭐ ENHANCED
**NEW:** Generates SPDX SBOMs with complete transitive relationships.

Converts dependency JSON into SPDX 2.3 compliant SBOM:
- ✅ SHA256 checksums for all packages
- ✅ Proper DEPENDS_ON relationships
- ✅ Distinguishes direct from transitive deps
- ✅ Package URLs (PURLs) for all artifacts

### graph_generator.py ⭐ ENHANCED
**NEW:** Calculates dependency depth for blast radius analysis.

Creates dependency graph visualizations:
- **JSON format**: Machine-readable graph with depth info
- **GraphML format**: Import into Gephi, yEd, or other tools
- **Depth tracking**: BFS-based shortest path calculation
- **Statistics**: Max depth, direct/transitive counts

**Usage:**
```bash
# From workspace_deps.json (recommended - has full info)
python3 tools/supplychain/graph_generator.py \
  --deps bazel-bin/workspace_deps.json \
  --output-json /tmp/graph.json \
  --output-graphml /tmp/graph.graphml

# From SBOM (fallback)
python3 tools/supplychain/graph_generator.py \
  --sbom bazel-bin/workspace_sbom.spdx.json \
  --output-json /tmp/graph.json
```

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
cat bazel-bin/workspace_sbom.spdx.json | python3 -m json.tool | less
```

Key sections:
- `creationInfo`: When and how the SBOM was created
- `packages`: List of all dependencies with PURLs and checksums
- `relationships`: Complete dependency graph (DEPENDS_ON)

**NEW:** Each package now includes:
- `checksums`: SHA256 for verification
- `externalRefs`: PURLs for ecosystem identification

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
