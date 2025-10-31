# BazBOM Examples

This directory contains examples demonstrating BazBOM usage with real-world scenarios.

## Orchestrated Scan Quickstart ⭐ NEW

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

See [Orchestrated Scanning Guide](../ORCHESTRATED_SCAN.md) and [Integration Plan Status](../INTEGRATION_PLAN_STATUS.md) for details.

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
- ✅ Complete dependency graph from maven_install.json
- ✅ Maven coordinates (group:artifact:version)
- ✅ Package URLs (PURLs) for all artifacts
- ✅ SHA256 checksums for verification
- ✅ Dependency relationships (DEPENDS_ON edges)
- ✅ Repository URLs
- ✅ SPDX 2.3 compliant SBOM

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
cat ./reports/sbom.spdx.json | python3 -m json.tool | head -50
cat ./reports/bazel_deps.json | python3 -m json.tool
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

### Legacy: Generating SBOM with Python tools

From the repository root:

```bash
# Using Bazel (original approach)
bazel build //:workspace_sbom

# View the SBOM
cat bazel-bin/workspace_sbom.spdx.json | python3 -m json.tool

# Or manually with Python scripts (from repository root)
python3 tools/supplychain/bazbom_extract_bazel_deps.py \
  --workspace /path/to/workspace \
  --maven-install-json /path/to/workspace/maven_install.json \
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

