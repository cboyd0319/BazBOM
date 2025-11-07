# BazBOM Examples

This directory contains examples demonstrating BazBOM usage with real-world scenarios.

## Orchestrated Scan Quickstart  NEW

The **`orchestrated-scan-quickstart.sh`** script demonstrates BazBOM's comprehensive security scanning capabilities:

```bash
cd docs/examples
./orchestrated-scan-quickstart.sh
```

This interactive demo showcases:
1. **Fast Scan** - SBOM + SCA (2 min) - Perfect for PR checks
2. **Comprehensive Scan** - + Semgrep pattern analysis (5 min) - Main branch
3. **Deep Security Scan** - + CodeQL dataflow analysis (15-20 min) - Security audits

All scans produce unified SARIF 2.1.0 output for GitHub Code Scanning.

See [Orchestrated Scanning Guide](../integrations/orchestrated-scan.md) for details.

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

## Dependency Analysis

BazBOM leverages **maven_install.json** as the source of truth, providing:

- **Complete Transitive Dependencies** - All dependencies (not just direct)
- **SHA256 Checksums** - Verification for all artifacts
- **Dependency Relationships** - Full parent→child mapping
- **Depth Tracking** - Blast radius analysis
- **Direct vs Transitive** - Clear classification

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

## Bazel/Java Example

BazBOM provides full Software Composition Analysis (SCA) for Bazel projects using `rules_jvm_external`.

### Prerequisites

For Bazel projects, you need:
1. A Bazel workspace with `maven_install()` defined
2. Generated `maven_install.json` (run `bazel run @maven//:pin`)
3. The `bazbom` CLI (built with `cargo build --release`)

### Generating SBOM with BazBOM CLI (Recommended)

```bash
# From the repository root (which is a Bazel workspace)
./target/release/bazbom scan . --out-dir ./output

# Or from any Bazel workspace
./target/release/bazbom scan /path/to/bazel/project
```

**What BazBOM extracts:**
-  Complete dependency graph from maven_install.json
-  Maven coordinates (group:artifact:version)
-  Package URLs (PURLs) for all artifacts
-  SHA256 checksums for verification
-  Dependency relationships (DEPENDS_ON edges)
-  Repository URLs
-  SPDX 2.3 compliant SBOM

**Output files:**
- `sbom.spdx.json` - SPDX 2.3 document with packages and relationships
- `bazel_deps.json` - Raw dependency graph (components + edges)
- `sca_findings.json` - Vulnerability findings (if advisory cache exists)
- `sca_findings.sarif` - GitHub Security format

### Example: Scanning BazBOM itself

```bash
# BazBOM can scan itself (it's a Bazel workspace)
cd /path/to/BazBOM
cargo build --release

# Scan the workspace
./target/release/bazbom scan . --out-dir ./reports

# View results
cat ./reports/sbom.spdx.json | jq . | head -50
cat ./reports/bazel_deps.json | jq .
```

**Expected output:**
```
[bazbom] scan path=. reachability=false format=spdx system=Bazel
[bazbom] extracting Bazel dependencies from "./maven_install.json"
Extracted 7 components
Extracted 6 edges
[bazbom] extracted 7 components and 6 edges
[bazbom] wrote dependency graph to "./reports/bazel_deps.json"
[bazbom] wrote "./reports/sbom.spdx.json"
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

## BazBOM CLI Features

The `bazbom` CLI provides all functionality for dependency extraction and analysis:

### Dependency Extraction

Extracts complete dependency information:
- All transitive dependencies from lockfile
- SHA256 checksums for verification
- Dependency relationships (parent→child)
- Direct vs transitive classification

**Usage:**
```bash
# Scan any project
bazbom scan . --out-dir ./output

# Specify build system
bazbom scan . --build-system bazel --out-dir ./output
```

### SBOM Generation

Generates SPDX 2.3 compliant SBOMs with complete transitive relationships:
- SHA256 checksums for all packages
- Proper DEPENDS_ON relationships
- Distinguishes direct from transitive deps
- Package URLs (PURLs) for all artifacts

### Dependency Graph Analysis

Creates dependency graph visualizations:
- **JSON format**: Machine-readable graph with depth info
- **GraphML format**: Import into Gephi, yEd, or other tools
- **Depth tracking**: BFS-based shortest path calculation
- **Statistics**: Max depth, direct/transitive counts

**Usage:**
```bash
# Generate dependency graph
bazbom scan . --format graph --out-dir ./output
```

### Provenance Generation

The BazBOM CLI generates SLSA provenance v1.0 attestations:
- Build environment details
- Input materials
- Build timestamps

### Vulnerability Scanning

BazBOM queries vulnerability databases:
- OSV (Open Source Vulnerabilities)
- NVD (National Vulnerability Database)
- GHSA (GitHub Security Advisories)
- CISA KEV (Known Exploited Vulnerabilities)

### SARIF Output

Converts vulnerability findings into SARIF 2.1.0 format for GitHub Code Scanning integration.

## Viewing Generated Artifacts

### SBOM (SPDX)
```bash
cat bazel-bin/workspace_sbom.spdx.json | jq . | less
```

Key sections:
- `creationInfo`: When and how the SBOM was created
- `packages`: List of all dependencies with PURLs and checksums
- `relationships`: Complete dependency graph (DEPENDS_ON)

Each package includes:
- `checksums`: SHA256 for verification
- `externalRefs`: PURLs for ecosystem identification

### Dependency Graph (JSON)
```bash
cat /tmp/demo_graph.json | jq . | less
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
cat /tmp/demo_provenance.json | jq . | less
```

Contains:
- Build environment (builder ID, timestamp)

