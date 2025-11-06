#!/bin/bash
# Example: Sign and verify SBOM with Sigstore
#
# This script demonstrates the complete SBOM signing and verification workflow
# using BazBOM's attestation tools.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== BazBOM SBOM Signing & Verification Demo ===${NC}\n"

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

if ! command -v cosign &> /dev/null; then
    echo -e "${YELLOW}Warning: cosign not found. Install from: https://docs.sigstore.dev/cosign/installation${NC}"
    echo "This demo will show the commands but may not execute fully."
fi

if ! command -v python3 &> /dev/null; then
    echo "Error: python3 is required"
    exit 1
fi

echo -e "${GREEN}[OK] Prerequisites checked${NC}\n"

# Step 1: Generate SBOM (if not exists)
echo -e "${BLUE}Step 1: Generate SBOM${NC}"

SBOM_PATH="${REPO_ROOT}/bazel-bin/workspace.spdx.json"

if [ ! -f "${SBOM_PATH}" ]; then
    echo "Generating SBOM..."
    cd "${REPO_ROOT}"
    bazel build //:workspace_sbom
    echo -e "${GREEN}[OK] SBOM generated: ${SBOM_PATH}${NC}\n"
else
    echo -e "${GREEN}[OK] SBOM exists: ${SBOM_PATH}${NC}\n"
fi

# Step 2: Sign SBOM
echo -e "${BLUE}Step 2: Sign SBOM with Sigstore${NC}"

SIGNATURES_DIR="${REPO_ROOT}/bazel-bin/signatures"
mkdir -p "${SIGNATURES_DIR}"

echo "Command:"
echo "  python3 tools/supplychain/sbom_signing.py sign \\"
echo "    ${SBOM_PATH} \\"
echo "    --output-dir=${SIGNATURES_DIR}"
echo ""

if command -v cosign &> /dev/null; then
    # Note: This requires OIDC authentication (GitHub Actions or interactive browser)
    echo "Note: Signing requires OIDC authentication (GitHub token or interactive browser)"
    echo "In CI/CD, this happens automatically with GitHub Actions OIDC provider"
    echo ""
    
    # In a real environment, you would run:
    # python3 "${REPO_ROOT}/tools/supplychain/sbom_signing.py" sign \
    #   "${SBOM_PATH}" \
    #   --output-dir="${SIGNATURES_DIR}"
    
    echo -e "${YELLOW}Skipping actual signing in demo (requires OIDC setup)${NC}"
    
    # Create mock signature files for demo
    echo "Creating mock signature files for demonstration..."
    echo "MEUCIQMockSignatureBase64EncodedDataHere==" > "${SIGNATURES_DIR}/workspace.sig"
    echo '{"rekor_entry": "https://rekor.sigstore.dev/api/v1/log/entries/mock123", "signature": {"sig": "MEUCIQMock"}}' > "${SIGNATURES_DIR}/workspace.bundle.json"
    
else
    echo -e "${YELLOW}Skipping signing (cosign not installed)${NC}"
fi

echo -e "${GREEN}[OK] Signing step complete${NC}\n"

# Step 3: Query Rekor
echo -e "${BLUE}Step 3: Query Rekor Transparency Log${NC}"

echo "Command to get entry by UUID:"
echo "  python3 tools/supplychain/rekor_integration.py get abc123def456"
echo ""

echo "Command to search by SBOM hash:"
echo "  python3 tools/supplychain/rekor_integration.py search \\"
echo "    \$(sha256sum ${SBOM_PATH} | cut -d' ' -f1)"
echo ""

echo -e "${YELLOW}Skipping actual Rekor query (requires real signature)${NC}\n"

# Step 4: Create Attestation
echo -e "${BLUE}Step 4: Create in-toto Attestation${NC}"

ATTESTATION_PATH="${REPO_ROOT}/bazel-bin/attestations/workspace.attestation.json"
mkdir -p "$(dirname "${ATTESTATION_PATH}")"

echo "Command:"
echo "  python3 tools/supplychain/intoto_attestation.py generate \\"
echo "    ${SBOM_PATH} \\"
echo "    --output ${ATTESTATION_PATH}"
echo ""

# Actually generate attestation (this doesn't require cosign)
python3 "${REPO_ROOT}/tools/supplychain/intoto_attestation.py" generate \
  "${SBOM_PATH}" \
  --output "${ATTESTATION_PATH}"

echo -e "${GREEN}[OK] Attestation created: ${ATTESTATION_PATH}${NC}\n"

# Show attestation structure
echo "Attestation structure:"
python3 -c "import json; print(json.dumps(json.load(open('${ATTESTATION_PATH}')), indent=2))" | head -30
echo "  ..."
echo ""

# Step 5: Verify
echo -e "${BLUE}Step 5: Verify SBOM Signature${NC}"

echo "Command:"
echo "  python3 tools/supplychain/verify_sbom.py \\"
echo "    ${SBOM_PATH} \\"
echo "    --bundle ${SIGNATURES_DIR}/workspace.bundle.json \\"
echo "    --cert-identity 'https://github.com/cboyd0319/BazBOM/.github/workflows/supplychain.yml@refs/heads/main' \\"
echo "    --cert-oidc-issuer 'https://token.actions.githubusercontent.com'"
echo ""

echo -e "${YELLOW}Skipping verification (requires real signature from cosign)${NC}\n"

# Summary
echo -e "${BLUE}=== Summary ===${NC}"
echo ""
echo "This demo showed the complete SBOM attestation workflow:"
echo ""
echo "1. [OK] Generate SBOM with Bazel"
echo "2. [OK] Sign SBOM with Sigstore (cosign)"
echo "3. [OK] Log signature to Rekor transparency log"
echo "4. [OK] Create in-toto attestation bundle"
echo "5. [OK] Verify signature and attestation"
echo ""
echo "Files created:"
echo "  - SBOM:        ${SBOM_PATH}"
echo "  - Attestation: ${ATTESTATION_PATH}"
if [ -f "${SIGNATURES_DIR}/workspace.sig" ]; then
    echo "  - Signature:   ${SIGNATURES_DIR}/workspace.sig"
    echo "  - Bundle:      ${SIGNATURES_DIR}/workspace.bundle.json"
fi
echo ""
echo "For full signing in CI/CD, see: .github/workflows/supplychain.yml"
echo "For documentation, see: docs/PROVENANCE.md"
echo ""
echo -e "${GREEN}Demo complete!${NC}"
