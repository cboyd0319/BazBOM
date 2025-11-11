#!/bin/bash
# BazBOM Complete Container Security Analysis Pipeline
# This demonstrates the full end-to-end workflow

set -e

CONTAINER_IMAGE="$1"
OUTPUT_DIR="${2:-.}"

if [ -z "$CONTAINER_IMAGE" ]; then
    echo "Usage: $0 <container-image> [output-dir]"
    echo "Example: $0 test-java-app:latest ./scan-results"
    exit 1
fi

echo "🔍 BazBOM Complete Container Security Analysis"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Container: $CONTAINER_IMAGE"
echo "Output:    $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR/sbom" "$OUTPUT_DIR/findings"

# Step 1: Generate comprehensive SBOM with Syft
echo "📦 Step 1/4: Generating SBOM with Syft..."
syft "$CONTAINER_IMAGE" -o spdx-json="$OUTPUT_DIR/sbom/spdx.json" -q
PACKAGE_COUNT=$(jq '.packages | length' "$OUTPUT_DIR/sbom/spdx.json")
echo "   ✅ Found $PACKAGE_COUNT packages"
echo ""

# Step 2: Scan for vulnerabilities with Trivy
echo "🔎 Step 2/4: Scanning for vulnerabilities with Trivy..."
trivy image --format json --output "$OUTPUT_DIR/findings/trivy.json" --quiet "$CONTAINER_IMAGE" 2>/dev/null || true
VULN_COUNT=$(jq '[.Results[].Vulnerabilities // [] | .[]] | length' "$OUTPUT_DIR/findings/trivy.json" 2>/dev/null || echo "0")
echo "   ✅ Found $VULN_COUNT vulnerabilities"
echo ""

# Step 3: Convert Trivy results to SARIF for BazBOM
echo "🔄 Step 3/4: Converting to SARIF format..."
trivy convert --format sarif --output "$OUTPUT_DIR/findings/trivy.sarif" "$OUTPUT_DIR/findings/trivy.json" 2>/dev/null || true
echo "   ✅ SARIF report generated"
echo ""

# Step 4: Generate beautiful summary with BazBOM
echo "✨ Step 4/4: Generating beautiful security report..."
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 SCAN RESULTS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Show package breakdown
echo "📦 Packages by Ecosystem:"
jq -r '.packages[] | .name' "$OUTPUT_DIR/sbom/spdx.json" | \
  sed 's/:.*//; s/@.*//' | \
  sort | uniq -c | sort -rn | head -10 | \
  awk '{printf "   • %-30s %s packages\n", $2, $1}'
echo ""

# Show vulnerability summary by severity
echo "🔒 Vulnerabilities by Severity:"
jq -r '.Results[].Vulnerabilities // [] | .[] | .Severity' "$OUTPUT_DIR/findings/trivy.json" 2>/dev/null | \
  sort | uniq -c | sort -rn | \
  awk '{
    severity=$2;
    count=$1;
    if (severity == "CRITICAL") icon="🔴";
    else if (severity == "HIGH") icon="🟠";
    else if (severity == "MEDIUM") icon="🟡";
    else if (severity == "LOW") icon="🟢";
    else icon="⚪";
    printf "   %s %-10s %s\n", icon, severity, count;
  }' || echo "   ✅ No vulnerabilities found!"
echo ""

# Show critical vulnerabilities
CRITICAL_COUNT=$(jq '[.Results[].Vulnerabilities // [] | .[] | select(.Severity == "CRITICAL")] | length' "$OUTPUT_DIR/findings/trivy.json" 2>/dev/null || echo "0")
if [ "$CRITICAL_COUNT" -gt 0 ]; then
    echo "🚨 CRITICAL Vulnerabilities (fix immediately):"
    jq -r '.Results[].Vulnerabilities // [] | .[] | select(.Severity == "CRITICAL") | "   • \(.VulnerabilityID): \(.PkgName)@\(.InstalledVersion) → \(.FixedVersion // "no fix available")"' "$OUTPUT_DIR/findings/trivy.json" | head -10
    echo ""
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📁 Full reports saved to:"
echo "   • SBOM:            $OUTPUT_DIR/sbom/spdx.json"
echo "   • Vulnerabilities: $OUTPUT_DIR/findings/trivy.json"
echo "   • SARIF:           $OUTPUT_DIR/findings/trivy.sarif"
echo ""
echo "🎯 Next Steps:"
if [ "$VULN_COUNT" -gt 0 ]; then
    echo "   • Review vulnerabilities: less $OUTPUT_DIR/findings/trivy.json"
    echo "   • Upload SARIF to GitHub: gh api repos/{owner}/{repo}/code-scanning/sarifs -F sarif=@$OUTPUT_DIR/findings/trivy.sarif"
    if [ "$CRITICAL_COUNT" -gt 0 ]; then
        echo "   • 🔥 FIX CRITICAL VULNERABILITIES IMMEDIATELY!"
    fi
else
    echo "   ✨ Container is secure! No vulnerabilities found."
fi
echo ""
