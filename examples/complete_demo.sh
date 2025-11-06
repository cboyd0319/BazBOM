#!/bin/bash
# BazBOM Complete Feature Demonstration
# This script demonstrates all major BazBOM features including:
# - SBOM generation (SPDX and CycloneDX)
# - Vulnerability scanning
# - Policy enforcement
# - License analysis
# - Supply chain risk detection

set -e

echo "======================================================================"
echo "BazBOM Complete Feature Demonstration"
echo "======================================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}[OK] $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Step 1: Extract dependencies
print_step "Step 1: Extract dependencies from maven_install.json"
bazel build //:extract_deps
print_success "Dependencies extracted"
echo ""

# Step 2: Generate SPDX SBOM
print_step "Step 2: Generate SPDX 2.3 SBOM"
bazel build //:workspace_sbom
print_success "SPDX SBOM generated: bazel-bin/workspace_sbom.spdx.json"
echo ""

# Show SBOM summary
echo "SBOM Summary (SPDX):"
jq -r '.packages | length as $count | "  - Total packages: \($count)"' bazel-bin/workspace_sbom.spdx.json
jq -r '.spdxVersion as $version | "  - SPDX version: \($version)"' bazel-bin/workspace_sbom.spdx.json
jq -r '.creationInfo.created as $created | "  - Created: \($created)"' bazel-bin/workspace_sbom.spdx.json
echo ""

# Step 3: Generate CycloneDX SBOM
print_step "Step 3: Generate CycloneDX 1.5 SBOM"
bazel build //:workspace_sbom_cyclonedx
print_success "CycloneDX SBOM generated: bazel-bin/workspace_sbom.cdx.json"
echo ""

# Show CycloneDX summary
echo "SBOM Summary (CycloneDX):"
jq -r '.components | length as $count | "  - Total components: \($count)"' bazel-bin/workspace_sbom.cdx.json
jq -r '.specVersion as $version | "  - CycloneDX version: \($version)"' bazel-bin/workspace_sbom.cdx.json
jq -r '.metadata.timestamp as $ts | "  - Timestamp: \($ts)"' bazel-bin/workspace_sbom.cdx.json
echo ""

# Step 4: Generate both formats
print_step "Step 4: Generate both SBOM formats together"
bazel build //:sbom_all_formats
print_success "Both formats generated"
echo ""

# Step 5: Generate dependency graph
print_step "Step 5: Generate dependency graph (JSON + GraphML)"
bazel build //:dep_graph_all
print_success "Dependency graphs generated"
echo "  - JSON: bazel-bin/dep_graph.json"
echo "  - GraphML: bazel-bin/dep_graph.graphml"
echo ""

# Step 6: Run vulnerability scan
print_step "Step 6: Run vulnerability scan (OSV)"
if bazel build //:sca_scan_osv 2>/dev/null; then
    print_success "Vulnerability scan complete"
    
    # Show vulnerability summary
    echo "Vulnerability Summary:"
    if [ -f bazel-bin/sca_findings.json ]; then
        jq -r '.packages_scanned as $scanned | "  - Packages scanned: \($scanned)"' bazel-bin/sca_findings.json || echo "  - Packages scanned: N/A"
        jq -r '.vulnerabilities_found as $found | "  - Vulnerabilities found: \($found)"' bazel-bin/sca_findings.json || echo "  - Vulnerabilities found: 0"
    fi
else
    print_warning "Vulnerability scan skipped (requires network)"
fi
echo ""

# Step 7: Generate SARIF report
print_step "Step 7: Generate SARIF report for GitHub Code Scanning"
if bazel build //:sca_sarif 2>/dev/null; then
    print_success "SARIF report generated: bazel-bin/sca_findings.sarif"
    
    if [ -f bazel-bin/sca_findings.sarif ]; then
        echo "SARIF Summary:"
        jq -r '.runs[0].results | length as $count | "  - Total results: \($count)"' bazel-bin/sca_findings.sarif 2>/dev/null || echo "  - Total results: 0"
    fi
else
    print_warning "SARIF generation skipped"
fi
echo ""

# Step 8: Detect dependency conflicts
print_step "Step 8: Detect dependency version conflicts"
bazel build //:conflict_report
print_success "Conflict detection complete: bazel-bin/conflicts.json"

if [ -f bazel-bin/conflicts.json ]; then
    echo "Conflict Summary:"
    jq -r '.conflicts | length as $count | "  - Total conflicts: \($count)"' bazel-bin/conflicts.json
fi
echo ""

# Step 9: Generate license report
print_step "Step 9: Generate license compliance report"
bazel build //:license_report
print_success "License report generated: bazel-bin/license_report.json"

if [ -f bazel-bin/license_report.json ]; then
    echo "License Summary:"
    jq -r '.summary.total as $total | "  - Total packages analyzed: \($total)"' bazel-bin/license_report.json 2>/dev/null || echo "  - Total packages analyzed: N/A"
    jq -r '.summary.copyleft as $copyleft | "  - Copyleft licenses: \($copyleft)"' bazel-bin/license_report.json 2>/dev/null || echo "  - Copyleft licenses: 0"
fi
echo ""

# Step 10: Supply chain risk analysis
print_step "Step 10: Supply chain risk analysis"
if bazel build //:supply_chain_risk_report 2>/dev/null; then
    print_success "Risk analysis complete: bazel-bin/supply_chain_risks.json"
    
    if [ -f bazel-bin/supply_chain_risks.json ]; then
        echo "Risk Summary:"
        jq -r '.total_risks as $total | "  - Total risks identified: \($total)"' bazel-bin/supply_chain_risks.json 2>/dev/null || echo "  - Total risks identified: N/A"
    fi
else
    print_warning "Risk analysis skipped (requires network)"
fi
echo ""

# Step 11: Generate SLSA provenance
print_step "Step 11: Generate SLSA provenance attestation"
bazel build //:workspace_provenance
print_success "Provenance generated: bazel-bin/workspace_sbom.provenance.json"

if [ -f bazel-bin/workspace_sbom.provenance.json ]; then
    echo "Provenance Summary:"
    jq -r '.predicate.buildType as $type | "  - Build type: \($type)"' bazel-bin/workspace_sbom.provenance.json 2>/dev/null || echo "  - Build type: N/A"
fi
echo ""

# Step 12: Apply VEX statements
print_step "Step 12: Apply VEX statements to filter findings"
bazel build //:sca_findings_with_vex
print_success "VEX processing complete: bazel-bin/sca_findings_filtered.json"
echo ""

# Step 13: Policy enforcement
print_step "Step 13: Enforce security policies"
echo "Testing different policy configurations:"
echo ""

# Test 1: Development policy (flexible)
echo "  Policy Test 1: Development policy (max 10 high vulns)"
if python3 tools/supplychain/policy_check.py \
    --findings bazel-bin/sca_findings_filtered.json \
    --max-critical 0 \
    --max-high 10 \
    --quiet 2>/dev/null; then
    print_success "  Development policy: PASSED"
else
    print_error "  Development policy: FAILED"
fi

# Test 2: Staging policy (moderate)
echo "  Policy Test 2: Staging policy (max 3 high vulns)"
if python3 tools/supplychain/policy_check.py \
    --findings bazel-bin/sca_findings_filtered.json \
    --max-critical 0 \
    --max-high 3 \
    --quiet 2>/dev/null; then
    print_success "  Staging policy: PASSED"
else
    print_warning "  Staging policy: FAILED (expected if high vulns > 3)"
fi

# Test 3: Production policy (strict)
echo "  Policy Test 3: Production policy (no critical/high vulns)"
if python3 tools/supplychain/policy_check.py \
    --findings bazel-bin/sca_findings_filtered.json \
    --max-critical 0 \
    --max-high 0 \
    --quiet 2>/dev/null; then
    print_success "  Production policy: PASSED"
else
    print_warning "  Production policy: FAILED (expected if any critical/high vulns)"
fi
echo ""

# Generate comprehensive policy report
print_step "Step 14: Generate comprehensive policy report"
bazel build //:policy_check_report
print_success "Policy report generated: bazel-bin/policy_check.json"

if [ -f bazel-bin/policy_check.json ]; then
    echo "Policy Report Summary:"
    jq -r '.total_violations as $total | "  - Total violations: \($total)"' bazel-bin/policy_check.json
    
    # Show violations by severity
    for severity in CRITICAL HIGH MEDIUM LOW; do
        count=$(jq -r "[.violations[] | select(.severity == \"$severity\")] | length" bazel-bin/policy_check.json 2>/dev/null || echo "0")
        if [ "$count" -gt 0 ]; then
            echo "  - $severity violations: $count"
        fi
    done
fi
echo ""

# Step 15: Aggregate metrics
print_step "Step 15: Aggregate supply chain metrics"
bazel build //:metrics_report
print_success "Metrics aggregated: bazel-bin/supply_chain_metrics.json"

if [ -f bazel-bin/supply_chain_metrics.json ]; then
    echo "Metrics Dashboard:"
    echo ""
    
    # Vulnerabilities
    echo "  Vulnerabilities:"
    jq -r '.vulnerabilities.total as $total | "    - Total: \($total)"' bazel-bin/supply_chain_metrics.json
    jq -r '.vulnerabilities.critical as $crit | "    - Critical: \($crit)"' bazel-bin/supply_chain_metrics.json
    jq -r '.vulnerabilities.high as $high | "    - High: \($high)"' bazel-bin/supply_chain_metrics.json
    echo ""
    
    # Dependencies
    echo "  Dependencies:"
    jq -r '.dependencies.total as $total | "    - Total: \($total)"' bazel-bin/supply_chain_metrics.json
    jq -r '.dependencies.direct as $direct | "    - Direct: \($direct)"' bazel-bin/supply_chain_metrics.json
    jq -r '.dependencies.transitive as $trans | "    - Transitive: \($trans)"' bazel-bin/supply_chain_metrics.json
    jq -r '.dependencies.conflicts as $conf | "    - Conflicts: \($conf)"' bazel-bin/supply_chain_metrics.json
    echo ""
    
    # Licenses
    echo "  Licenses:"
    jq -r '.licenses.copyleft as $copyleft | "    - Copyleft: \($copyleft)"' bazel-bin/supply_chain_metrics.json
    jq -r '.licenses.permissive as $permissive | "    - Permissive: \($permissive)"' bazel-bin/supply_chain_metrics.json
    jq -r '.licenses.unknown as $unknown | "    - Unknown: \($unknown)"' bazel-bin/supply_chain_metrics.json
fi
echo ""

# Summary
echo "======================================================================"
echo "Demonstration Complete!"
echo "======================================================================"
echo ""
echo "Generated artifacts:"
echo "  - SPDX SBOM: bazel-bin/workspace_sbom.spdx.json"
echo "  - CycloneDX SBOM: bazel-bin/workspace_sbom.cdx.json"
echo "  - Dependency graphs: bazel-bin/dep_graph.{json,graphml}"
echo "  - SCA findings: bazel-bin/sca_findings.json"
echo "  - SARIF report: bazel-bin/sca_findings.sarif"
echo "  - License report: bazel-bin/license_report.json"
echo "  - Conflict report: bazel-bin/conflicts.json"
echo "  - Risk analysis: bazel-bin/supply_chain_risks.json"
echo "  - Provenance: bazel-bin/workspace_sbom.provenance.json"
echo "  - Policy report: bazel-bin/policy_check.json"
echo "  - Metrics: bazel-bin/supply_chain_metrics.json"
echo ""
echo "Next steps:"
echo "  1. Review the generated SBOMs and reports"
echo "  2. Upload SARIF to GitHub: gh api repos/{owner}/{repo}/code-scanning/sarifs -f sarif=@bazel-bin/sca_findings.sarif"
echo "  3. Integrate policy checks into CI/CD pipeline"
echo "  4. Visualize dependency graph with Gephi or yEd"
echo "  5. Sign provenance with cosign for SLSA compliance"
echo ""

print_success "All BazBOM features demonstrated successfully!"
