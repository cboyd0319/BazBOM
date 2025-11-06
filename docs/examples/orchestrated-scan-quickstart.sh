#!/bin/bash
# BazBOM Orchestrated Scan Quickstart Demo
# This script demonstrates the complete orchestrated scanning workflow
# as defined in docs/strategy/product-roadmap/BAZBOM_INTEGRATION_PLAN.md
#
# Usage: ./orchestrated-scan-quickstart.sh [project-dir]

set -e

PROJECT_DIR="${1:-.}"
OUTPUT_DIR="bazbom-output"

echo "========================================"
echo "BazBOM Orchestrated Scan Quickstart"
echo "========================================"
echo ""
echo "This demo showcases BazBOM's comprehensive security analysis:"
echo "  - SBOM generation (SPDX 2.3 + optional CycloneDX 1.5)"
echo "  - SCA findings (OSV/NVD/GHSA)"
echo "  - Optional Semgrep pattern analysis"
echo "  - Optional CodeQL dataflow analysis"
echo "  - Enrichment with deps.dev"
echo "  - Merged SARIF 2.1.0 output for GitHub Code Scanning"
echo ""

# Check if bazbom is available
if ! command -v bazbom >/dev/null 2>&1; then
    echo "ERROR: Error: bazbom not found in PATH"
    echo ""
    echo "Install options:"
    echo "  1. Homebrew: brew tap cboyd0319/bazbom && brew install bazbom"
    echo "  2. Build from source: cargo build --release -p bazbom"
    echo "  3. Pre-built binary: Download from GitHub Releases"
    echo ""
    exit 1
fi

echo "OK: Found bazbom: $(which bazbom)"
echo "  Version: $(bazbom --version)"
echo ""

# Navigate to project directory
cd "$PROJECT_DIR"
echo "Directory: Project directory: $(pwd)"
echo ""

# Detect build system
if [ -f "pom.xml" ]; then
    BUILD_SYSTEM="Maven"
elif [ -f "build.gradle" ] || [ -f "build.gradle.kts" ]; then
    BUILD_SYSTEM="Gradle"
elif [ -f "BUILD" ] || [ -f "BUILD.bazel" ] || [ -f "WORKSPACE" ]; then
    BUILD_SYSTEM="Bazel"
else
    BUILD_SYSTEM="Unknown"
fi

echo "Detected: Detected build system: $BUILD_SYSTEM"
echo ""

# Check for optional tools
SEMGREP_AVAILABLE=false
CODEQL_AVAILABLE=false

if command -v semgrep >/dev/null 2>&1; then
    SEMGREP_AVAILABLE=true
    echo "OK: Semgrep available: $(semgrep --version | head -1)"
else
    echo "WARNING: Semgrep not found (install: pipx install semgrep)"
fi

if command -v codeql >/dev/null 2>&1; then
    CODEQL_AVAILABLE=true
    echo "OK: CodeQL available: $(codeql version --format=terse 2>/dev/null || echo 'unknown')"
else
    echo "WARNING: CodeQL not found (optional for deep analysis)"
fi

echo ""
echo "=========================================="
echo "SCENARIO 1: Fast Scan (SBOM + SCA)"
echo "=========================================="
echo "Perfect for: Pull Request checks, quick validation"
echo "Outputs: SPDX SBOM, SCA findings, merged SARIF"
echo ""

read -p "Press Enter to run fast scan..."

bazbom scan . \
    --out-dir "$OUTPUT_DIR/fast-scan" \
    --no-upload

echo ""
echo "OK: Fast scan complete!"
echo "  Outputs in: $OUTPUT_DIR/fast-scan/"
echo "  - sbom/spdx.json - SPDX 2.3 SBOM"
echo "  - findings/sca.sarif - SCA findings"
echo "  - findings/merged.sarif - Combined SARIF for GitHub"
echo ""

# Count results
if [ -f "$OUTPUT_DIR/fast-scan/findings/merged.sarif" ]; then
    TOTAL_FINDINGS=$(jq '[.runs[].results | length] | add' "$OUTPUT_DIR/fast-scan/findings/merged.sarif" 2>/dev/null || echo "0")
    echo "  Total findings: $TOTAL_FINDINGS"
fi

echo ""
echo "=========================================="
echo "SCENARIO 2: Comprehensive Scan"
echo "=========================================="
echo "Perfect for: Main branch, nightly scans"
echo "Adds: CycloneDX SBOM, Semgrep analysis"
echo ""

if [ "$SEMGREP_AVAILABLE" = true ]; then
    read -p "Press Enter to run comprehensive scan..."
    
    bazbom scan . \
        --cyclonedx \
        --with-semgrep \
        --out-dir "$OUTPUT_DIR/comprehensive-scan" \
        --no-upload
    
    echo ""
    echo "OK: Comprehensive scan complete!"
    echo "  Outputs in: $OUTPUT_DIR/comprehensive-scan/"
    echo "  - sbom/spdx.json - SPDX 2.3 SBOM"
    echo "  - sbom/cyclonedx.json - CycloneDX 1.5 SBOM"
    echo "  - findings/sca.sarif - SCA findings"
    echo "  - findings/semgrep.sarif - Semgrep findings"
    echo "  - findings/merged.sarif - Combined SARIF"
    echo ""
    
    if [ -f "$OUTPUT_DIR/comprehensive-scan/findings/merged.sarif" ]; then
        TOTAL_RUNS=$(jq '.runs | length' "$OUTPUT_DIR/comprehensive-scan/findings/merged.sarif" 2>/dev/null || echo "0")
        TOTAL_FINDINGS=$(jq '[.runs[].results | length] | add' "$OUTPUT_DIR/comprehensive-scan/findings/merged.sarif" 2>/dev/null || echo "0")
        echo "  Total analyzer runs: $TOTAL_RUNS"
        echo "  Total findings: $TOTAL_FINDINGS"
    fi
else
    echo "WARNING: Skipping comprehensive scan (Semgrep not available)"
fi

echo ""
echo "=========================================="
echo "SCENARIO 3: Deep Security Scan"
echo "=========================================="
echo "Perfect for: Security audits, compliance"
echo "Adds: CodeQL dataflow analysis, autofix recipes"
echo ""

if [ "$CODEQL_AVAILABLE" = true ]; then
    read -p "Press Enter to run deep security scan (this may take 10-20 minutes)..."
    
    bazbom scan . \
        --cyclonedx \
        --with-semgrep \
        --with-codeql default \
        --autofix dry-run \
        --out-dir "$OUTPUT_DIR/deep-scan" \
        --no-upload
    
    echo ""
    echo "OK: Deep security scan complete!"
    echo "  Outputs in: $OUTPUT_DIR/deep-scan/"
    echo "  - sbom/ - Both SPDX and CycloneDX SBOMs"
    echo "  - findings/ - SCA, Semgrep, CodeQL findings"
    echo "  - findings/merged.sarif - All findings combined"
    echo "  - fixes/ - OpenRewrite autofix recipes"
    echo ""
    
    if [ -f "$OUTPUT_DIR/deep-scan/findings/merged.sarif" ]; then
        TOTAL_RUNS=$(jq '.runs | length' "$OUTPUT_DIR/deep-scan/findings/merged.sarif" 2>/dev/null || echo "0")
        TOTAL_FINDINGS=$(jq '[.runs[].results | length] | add' "$OUTPUT_DIR/deep-scan/findings/merged.sarif" 2>/dev/null || echo "0")
        echo "  Total analyzer runs: $TOTAL_RUNS"
        echo "  Total findings: $TOTAL_FINDINGS"
    fi
else
    echo "WARNING: Skipping deep scan (CodeQL not available)"
fi

echo ""
echo "=========================================="
echo "Summary"
echo "=========================================="
echo ""
echo "All scan results are in: $OUTPUT_DIR/"
echo ""
echo "Next steps:"
echo "  1. Review findings in findings/merged.sarif"
echo "  2. Upload SARIF to GitHub Code Scanning:"
echo "     - Use: github/codeql-action/upload-sarif@v3"
echo "  3. Archive artifacts in CI:"
echo "     - Use: actions/upload-artifact@v4"
echo "  4. Apply autofix recipes (if generated):"
echo "     - Review: fixes/openrewrite-recipes.json"
echo ""
echo "For GitHub Actions integration, see:"
echo "  - examples/github-actions/bazbom-scan.yml"
echo "  - .github/workflows/bazbom-orchestrated-scan.yml"
echo ""
echo "Documentation:"
echo "  - docs/ORCHESTRATED_SCAN.md - Complete guide"
echo "  - docs/strategy/product-roadmap/BAZBOM_INTEGRATION_PLAN.md - Integration plan"
echo "  - docs/USAGE.md - Command reference"
echo ""
echo "OK: Quickstart demo complete!"
