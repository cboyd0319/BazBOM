#!/bin/bash
# Demo script showing complete BazBOM workflow
# This script demonstrates all major features of BazBOM

set -e  # Exit on error

echo "=========================================="
echo "BazBOM Complete Workflow Demo"
echo "=========================================="
echo ""

# Change to repository root
cd "$(dirname "$0")/.."

echo "Step 1: Extract Maven dependencies from WORKSPACE"
echo "--------------------------------------------------"
python3 tools/supplychain/extract_maven_deps.py \
  --workspace WORKSPACE \
  --output /tmp/demo_deps.json
echo "[OK] Dependencies extracted to /tmp/demo_deps.json"
echo ""

echo "Step 2: Generate SPDX SBOM"
echo "--------------------------------------------------"
python3 tools/supplychain/write_sbom.py \
  --input /tmp/demo_deps.json \
  --output /tmp/demo_sbom.spdx.json \
  --name "bazbom-demo"
echo "[OK] SBOM generated at /tmp/demo_sbom.spdx.json"
echo ""

echo "Step 3: Validate SBOM"
echo "--------------------------------------------------"
python3 tools/supplychain/validators/validate_sbom.py \
  /tmp/demo_sbom.spdx.json \
  --verbose
echo ""

echo "Step 4: Generate dependency graphs"
echo "--------------------------------------------------"
python3 tools/supplychain/graph_generator.py \
  --sbom /tmp/demo_sbom.spdx.json \
  --output-json /tmp/demo_graph.json \
  --output-graphml /tmp/demo_graph.graphml
echo "[OK] JSON graph: /tmp/demo_graph.json"
echo "[OK] GraphML: /tmp/demo_graph.graphml"
echo ""

echo "Step 5: Generate SLSA provenance"
echo "--------------------------------------------------"
python3 tools/supplychain/provenance_builder.py \
  --artifact "bazbom-demo" \
  --output /tmp/demo_provenance.json \
  --commit "$(git rev-parse HEAD)" \
  --builder "cboyd0319/BazBOM"
echo "[OK] Provenance generated at /tmp/demo_provenance.json"
echo ""

echo "Step 6: Run vulnerability scan (OSV)"
echo "--------------------------------------------------"
echo "Note: This step requires network access to OSV API"
if python3 tools/supplychain/osv_query.py \
  --sbom /tmp/demo_sbom.spdx.json \
  --output /tmp/demo_sca.json \
  --batch 2>/dev/null; then
  echo "[OK] SCA scan completed: /tmp/demo_sca.json"
  
  echo ""
  echo "Step 7: Generate SARIF report"
  echo "--------------------------------------------------"
  python3 tools/supplychain/sarif_adapter.py \
    --input /tmp/demo_sca.json \
    --output /tmp/demo_findings.sarif
  echo "[OK] SARIF report: /tmp/demo_findings.sarif"
  
  echo ""
  echo "Step 8: Validate SARIF report"
  echo "--------------------------------------------------"
  python3 tools/supplychain/validators/validate_sarif.py \
    /tmp/demo_findings.sarif \
    --verbose
else
  echo "⚠ SCA scan skipped (network unavailable or API error)"
fi
echo ""

echo "=========================================="
echo "Demo complete! Generated artifacts:"
echo "=========================================="
echo ""
echo "SBOM & Graphs:"
echo "  /tmp/demo_sbom.spdx.json     - SPDX 2.3 SBOM"
echo "  /tmp/demo_graph.json         - Dependency graph (JSON)"
echo "  /tmp/demo_graph.graphml      - Dependency graph (GraphML)"
echo ""
echo "Security & Provenance:"
echo "  /tmp/demo_provenance.json    - SLSA provenance"
if [ -f /tmp/demo_sca.json ]; then
  echo "  /tmp/demo_sca.json           - SCA findings"
  echo "  /tmp/demo_findings.sarif     - SARIF report"
fi
echo ""
echo "You can now:"
echo "  - View SBOM: cat /tmp/demo_sbom.spdx.json | python3 -m json.tool | less"
echo "  - View graph: cat /tmp/demo_graph.json | python3 -m json.tool | less"
echo "  - Import GraphML into Gephi or yEd for visualization"
if [ -f /tmp/demo_findings.sarif ]; then
  echo "  - Upload SARIF to GitHub Code Scanning"
fi
echo ""
