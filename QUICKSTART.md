# BazBOM Quickstart Guide

Get started with BazBOM in under 5 minutes.

## Prerequisites

- Bazel 6.0+ (or Python 3.9+ for manual usage)
- Git (for the demo)

## Quick Demo

The fastest way to see BazBOM in action:

```bash
# Clone the repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM

# Run the complete workflow demo
bash examples/demo_workflow.sh
```

This generates:
- ✅ SPDX 2.3 SBOM
- ✅ Dependency graphs (JSON + GraphML)
- ✅ SLSA provenance
- ✅ OSV vulnerability scan
- ✅ SARIF report

All artifacts are created in `/tmp/` for inspection.

## Manual Usage (Without Bazel)

### 1. Extract Dependencies

```bash
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --output deps.json
```

### 2. Generate SBOM

```bash
python3 tools/supplychain/write_sbom.py \
  --input deps.json \
  --output app.spdx.json \
  --name my-app
```

### 3. Validate SBOM

```bash
python3 tools/supplychain/validators/validate_sbom.py \
  app.spdx.json \
  --verbose
```

### 4. Generate Dependency Graph

```bash
python3 tools/supplychain/graph_generator.py \
  --sbom app.spdx.json \
  --output-json graph.json \
  --output-graphml graph.graphml
```

### 5. Run Vulnerability Scan

```bash
# Requires: pip install requests
python3 tools/supplychain/osv_query.py \
  --sbom app.spdx.json \
  --output findings.json \
  --batch
```

### 6. Generate SARIF Report

```bash
python3 tools/supplychain/sarif_adapter.py \
  --input findings.json \
  --output findings.sarif
```

### 7. Generate Provenance

```bash
python3 tools/supplychain/provenance_builder.py \
  --artifact my-app \
  --output provenance.json \
  --commit $(git rev-parse HEAD) \
  --builder "my-org/my-repo"
```

## Using with Bazel

### Generate SBOM

```bash
bazel build //:sbom_all
ls bazel-bin/workspace_sbom.spdx.json
```

### Generate Graphs

```bash
bazel build //:dep_graph_all
ls bazel-bin/dep_graph.json
ls bazel-bin/dep_graph.graphml
```

### Generate Everything

```bash
bazel build //:supply_chain_all
ls bazel-bin/
```

## View Generated Artifacts

### SBOM

```bash
cat bazel-bin/workspace_sbom.spdx.json | python3 -m json.tool | less
```

Look for:
- `packages`: List of dependencies
- `externalRefs`: Package URLs (PURLs)
- `relationships`: Dependency relationships

### Dependency Graph

```bash
cat bazel-bin/dep_graph.json | python3 -m json.tool | less
```

Import GraphML into visualization tools:
- Gephi: File → Open
- yEd: File → Open
- Cytoscape: File → Import → Network

### SARIF Report

```bash
cat bazel-bin/sca_findings.sarif | python3 -m json.tool | less
```

Upload to GitHub:
- Go to repository Security tab
- Code scanning → Upload SARIF

## Integration with GitHub Actions

Create `.github/workflows/sbom.yml`:

```yaml
name: SBOM & SCA
on: [push, pull_request]

jobs:
  supply-chain:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      security-events: write
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Bazel
        uses: bazel-contrib/setup-bazel@0.8.1
      
      - name: Generate SBOM
        run: bazel build //:sbom_all
      
      - name: Run SCA
        run: bazel build //:sca_scan_osv || true
      
      - name: Generate SARIF
        run: bazel build //:sca_sarif || true
      
      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: bazel-bin/**/*.spdx.json
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: bazel-bin/sca_findings.sarif
        continue-on-error: true
```

## Troubleshooting

### "Network error" during OSV scan
Install requests: `pip install requests`

### "Command not found: bazel"
Use manual Python scripts or install Bazel: https://bazel.build/install

### "Invalid SBOM" errors
Check that all required fields are present. Run validator with `--verbose` for details.

### "Empty SBOM" (no dependencies)
Ensure `maven_install()` is configured in WORKSPACE with at least one artifact.

## Next Steps

1. **Customize for your project**: Adapt the tools to your specific needs
2. **Set up CI**: Use the GitHub Actions workflow or adapt for your CI system
3. **Monitor vulnerabilities**: Set up GitHub Code Scanning alerts
4. **Visualize dependencies**: Import GraphML into Gephi or yEd
5. **Automate compliance**: Add policy checks and automated remediation

For more details, see:
- [Usage Guide](docs/USAGE.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Examples](examples/README.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)
