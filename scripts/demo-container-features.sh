#!/bin/bash

# 🐳 BazBOM Container Scanner - Feature Demo
# This script demonstrates all the advanced container scanning capabilities

set -e

echo "╔═══════════════════════════════════════════════════════════════════╗"
echo "║          🐳 BazBOM Container Scanner - Feature Demo               ║"
echo "╚═══════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Test image
IMAGE="openjdk:17-alpine"

echo -e "${CYAN}📋 Available Features:${NC}"
echo ""
echo "  1. ⚡ Quick Wins Analysis - Easy patches with high impact"
echo "  2. 📋 Prioritized Action Plan - P0-P4 classification with time estimates"
echo "  3. 📋 Copy-Paste Remediation - Maven/Gradle ready snippets"
echo "  4. 💰 Effort Analysis - Cost/time breakdown"
echo "  5. 🏆 Security Score - Gamified 0-100 metric"
echo "  6. 🔍 Smart Filtering - Filter by priority/type (--show p0, fixable, quick-wins)"
echo "  7. 💾 Baseline Tracking - Save & compare scans (--baseline, --compare-baseline)"
echo "  8. 🔍 Image Comparison - Side-by-side comparison (--compare)"
echo "  9. 📝 GitHub Issues - Auto-create issues (--create-issues)"
echo " 10. 📊 Executive Reports - HTML reports (--report)"
echo " 11. 🚀 Interactive TUI - Terminal UI (--interactive)"
echo ""

echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Function to pause between demos
pause() {
    echo ""
    echo -e "${MAGENTA}Press Enter to continue...${NC}"
    read
}

# Demo 1: Basic scan with all intelligence features
echo -e "${GREEN}Demo 1: Basic Scan with Intelligence${NC}"
echo "Command: bazbom container-scan $IMAGE --output demo-scan"
echo ""
pause

bazbom container-scan "$IMAGE" --output demo-scan || true

# Demo 2: Scan with baseline save
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 2: Save Baseline${NC}"
echo "Command: bazbom container-scan $IMAGE --baseline --output demo-baseline"
echo ""
pause

bazbom container-scan "$IMAGE" --baseline --output demo-baseline || true
echo ""
echo -e "${CYAN}✅ Baseline saved to .bazbom/baselines/${NC}"
ls -lh .bazbom/baselines/ 2>/dev/null || echo "Baselines will be saved here"

# Demo 3: Filter by priority
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 3: Smart Filtering - Show Only P0 Vulnerabilities${NC}"
echo "Command: bazbom container-scan $IMAGE --show p0 --output demo-p0"
echo ""
pause

bazbom container-scan "$IMAGE" --show p0 --output demo-p0 || true

# Demo 4: Show quick wins
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 4: Show Only Quick Wins (Easy Patches)${NC}"
echo "Command: bazbom container-scan $IMAGE --show quick-wins --output demo-qw"
echo ""
pause

bazbom container-scan "$IMAGE" --show quick-wins --output demo-qw || true

# Demo 5: Generate executive report
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 5: Generate Executive Report${NC}"
echo "Command: bazbom container-scan $IMAGE --report executive-report.html --output demo-report"
echo ""
pause

bazbom container-scan "$IMAGE" --report executive-report.html --output demo-report || true
echo ""
if [ -f executive-report.html ]; then
    echo -e "${CYAN}✅ Report generated: executive-report.html${NC}"
    echo -e "${CYAN}   Open with: open executive-report.html${NC}"
fi

# Demo 6: Compare baseline
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 6: Compare Against Baseline${NC}"
echo "Command: bazbom container-scan $IMAGE --compare-baseline --output demo-compare-base"
echo ""
pause

bazbom container-scan "$IMAGE" --compare-baseline --output demo-compare-base || true

# Demo 7: Help
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}Demo 7: Show All Available Options${NC}"
echo "Command: bazbom container-scan --help"
echo ""
pause

bazbom container-scan --help

# Summary
echo ""
echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${CYAN}🎉 Demo Complete!${NC}"
echo ""
echo "Key Takeaways:"
echo "  ✅ Quick wins analysis identifies easy patches"
echo "  ✅ Prioritized action plans with time estimates"
echo "  ✅ Smart filtering focuses on what matters"
echo "  ✅ Baseline tracking measures improvement"
echo "  ✅ Executive reports for stakeholders"
echo "  ✅ GitHub integration for workflow automation"
echo ""
echo "For more details, see: CONTAINER_SCAN_FEATURES.md"
echo ""
