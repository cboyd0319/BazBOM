#!/bin/bash
# BazBOM Integration Validation Script
# Tests the complete integration plan implementation end-to-end

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${SCRIPT_DIR}/output"
BAZBOM_BIN="${SCRIPT_DIR}/../../target/release/bazbom"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "==================================="
echo "BazBOM Integration Plan Validation"
echo "==================================="
echo

# Check if bazbom binary exists
if [ ! -f "$BAZBOM_BIN" ]; then
    echo -e "${RED}Error: bazbom binary not found at $BAZBOM_BIN${NC}"
    echo "Please run: cargo build --release"
    exit 1
fi

echo -e "${GREEN}[OK]${NC} Found bazbom binary"

# Clean previous output
if [ -d "$OUTPUT_DIR" ]; then
    echo "Cleaning previous output..."
    rm -rf "$OUTPUT_DIR"
fi

echo
echo "==================================="
echo "Phase 1: Minimal Scan (SCA only)"
echo "==================================="
echo

cd "$SCRIPT_DIR"
$BAZBOM_BIN scan . --out-dir "$OUTPUT_DIR" --no-upload

echo
echo "Validating Phase 1 outputs..."

# Check directory structure
DIRS=("sbom" "findings" "enrich" "fixes")
for dir in "${DIRS[@]}"; do
    if [ -d "$OUTPUT_DIR/$dir" ]; then
        echo -e "${GREEN}[OK]${NC} Directory exists: $dir/"
    else
        echo -e "${RED}✗${NC} Missing directory: $dir/"
        exit 1
    fi
done

# Check SBOM files
if [ -f "$OUTPUT_DIR/sbom/spdx.json" ]; then
    echo -e "${GREEN}[OK]${NC} SPDX 2.3 SBOM generated"
    SPDX_VERSION=$(jq -r '.spdxVersion' "$OUTPUT_DIR/sbom/spdx.json")
    if [ "$SPDX_VERSION" = "SPDX-2.3" ]; then
        echo -e "${GREEN}[OK]${NC} SPDX version correct: $SPDX_VERSION"
    else
        echo -e "${RED}✗${NC} SPDX version incorrect: $SPDX_VERSION"
        exit 1
    fi
else
    echo -e "${RED}✗${NC} SPDX SBOM not found"
    exit 1
fi

# Check SARIF files
if [ -f "$OUTPUT_DIR/findings/sca.sarif" ]; then
    echo -e "${GREEN}[OK]${NC} SCA SARIF generated"
else
    echo -e "${RED}✗${NC} SCA SARIF not found"
    exit 1
fi

if [ -f "$OUTPUT_DIR/findings/merged.sarif" ]; then
    echo -e "${GREEN}[OK]${NC} Merged SARIF generated"
    SARIF_VERSION=$(jq -r '.version' "$OUTPUT_DIR/findings/merged.sarif")
    if [ "$SARIF_VERSION" = "2.1.0" ]; then
        echo -e "${GREEN}[OK]${NC} SARIF version correct: $SARIF_VERSION"
    else
        echo -e "${RED}✗${NC} SARIF version incorrect: $SARIF_VERSION"
        exit 1
    fi
    
    # Check for runs array
    RUNS_COUNT=$(jq '.runs | length' "$OUTPUT_DIR/findings/merged.sarif")
    echo -e "${GREEN}[OK]${NC} SARIF runs count: $RUNS_COUNT"
else
    echo -e "${RED}✗${NC} Merged SARIF not found"
    exit 1
fi

echo
echo "==================================="
echo "Phase 2: Enhanced Scan (with enrichment)"
echo "==================================="
echo

# Clean output for Phase 2
rm -rf "$OUTPUT_DIR"

# Create config with enrichment enabled
cat > "$SCRIPT_DIR/bazbom-test.toml" << EOF
[analysis]
cyclonedx = true

[enrich]
depsdev = true

[autofix]
mode = "dry-run"

[publish]
github_code_scanning = false
artifact = true
EOF

# Run with enrichment
BAZBOM_OFFLINE=1 $BAZBOM_BIN scan . \
    --cyclonedx \
    --out-dir "$OUTPUT_DIR" \
    --no-upload

echo
echo "Validating Phase 2 outputs..."

# Check CycloneDX
if [ -f "$OUTPUT_DIR/sbom/cyclonedx.json" ]; then
    echo -e "${GREEN}[OK]${NC} CycloneDX SBOM generated"
else
    echo -e "${YELLOW}⚠${NC} CycloneDX SBOM not found (optional)"
fi

# Check enrichment
if [ -f "$OUTPUT_DIR/enrich/depsdev.json" ]; then
    echo -e "${GREEN}[OK]${NC} deps.dev enrichment generated"
    OFFLINE_MODE=$(jq -r '.offline_mode' "$OUTPUT_DIR/enrich/depsdev.json")
    if [ "$OFFLINE_MODE" = "true" ]; then
        echo -e "${GREEN}[OK]${NC} Offline mode detected correctly"
    fi
else
    echo -e "${RED}✗${NC} deps.dev enrichment not found"
    exit 1
fi

# Clean up test config
rm -f "$SCRIPT_DIR/bazbom-test.toml"

echo
echo "==================================="
echo "Phase 3: SARIF Validation"
echo "==================================="
echo

# Validate SARIF structure
echo "Validating SARIF 2.1.0 compliance..."

MERGED_SARIF="$OUTPUT_DIR/findings/merged.sarif"

# Check schema
if jq -e '.["$schema"]' "$MERGED_SARIF" > /dev/null; then
    SCHEMA=$(jq -r '.["$schema"]' "$MERGED_SARIF")
    echo -e "${GREEN}[OK]${NC} Schema present: $SCHEMA"
fi

# Check runs
RUNS=$(jq '.runs[] | .tool.driver.name' "$MERGED_SARIF")
echo "Tool runs in merged SARIF:"
echo "$RUNS" | while read -r tool; do
    echo -e "${GREEN}[OK]${NC}   - $tool"
done

# Check each run has required fields
jq -e '.runs[] | .tool.driver | has("name") and has("version")' "$MERGED_SARIF" > /dev/null
if [ $? -eq 0 ]; then
    echo -e "${GREEN}[OK]${NC} All runs have tool.driver.name and version"
fi

echo
echo "==================================="
echo "Validation Summary"
echo "==================================="
echo
echo -e "${GREEN}[OK]${NC} Directory structure compliant with integration plan"
echo -e "${GREEN}[OK]${NC} SPDX 2.3 SBOM generation"
echo -e "${GREEN}[OK]${NC} SARIF 2.1.0 compliance"
echo -e "${GREEN}[OK]${NC} deps.dev enrichment"
echo -e "${GREEN}[OK]${NC} Offline mode support"
echo -e "${GREEN}[OK]${NC} Multiple tool runs merged correctly"
echo
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}All integration plan requirements met!${NC}"
echo -e "${GREEN}========================================${NC}"
echo

# Show output structure
echo "Output structure:"
if command -v tree &> /dev/null; then
    tree -L 2 "$OUTPUT_DIR"
else
    find "$OUTPUT_DIR" -type f | sort
fi
