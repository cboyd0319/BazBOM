#!/bin/bash
# Sign BazBOM release artifacts with GPG
#
# Usage: ./scripts/sign-release.sh <version> <artifacts-dir>
# Example: ./scripts/sign-release.sh v7.0.0 dist/

set -euo pipefail

VERSION="${1:-}"
ARTIFACTS_DIR="${2:-dist}"

if [ -z "$VERSION" ]; then
  echo "Error: Version required"
  echo "Usage: $0 <version> [artifacts-dir]"
  exit 1
fi

if [ ! -d "$ARTIFACTS_DIR" ]; then
  echo "Error: Artifacts directory not found: $ARTIFACTS_DIR"
  exit 1
fi

echo "🔏 Signing BazBOM $VERSION release artifacts"
echo "   Artifacts directory: $ARTIFACTS_DIR"
echo ""

# Check GPG is available
if ! command -v gpg &> /dev/null; then
  echo "Error: GPG not found. Install gnupg:"
  echo "  - macOS: brew install gnupg"
  echo "  - Ubuntu: apt-get install gnupg"
  exit 1
fi

# Check for GPG key
if [ -z "${GPG_KEY_ID:-}" ]; then
  echo "⚠️  GPG_KEY_ID not set. Using default key."
  GPG_SIGN_ARGS=""
else
  echo "✅ Using GPG key: $GPG_KEY_ID"
  GPG_SIGN_ARGS="--local-user $GPG_KEY_ID"
fi

# Sign all binaries
SIGNED_COUNT=0
for artifact in "$ARTIFACTS_DIR"/*; do
  # Skip non-files and already-signed files
  if [ ! -f "$artifact" ] || [[ "$artifact" == *.asc ]] || [[ "$artifact" == *.sha256 ]]; then
    continue
  fi

  echo "📝 Signing: $(basename "$artifact")"

  # Create detached signature
  gpg --detach-sign --armor $GPG_SIGN_ARGS --output "${artifact}.asc" "$artifact"

  # Create checksum
  sha256sum "$artifact" | awk '{print $1}' > "${artifact}.sha256"

  # Sign checksum (clearsign for readability)
  gpg --clearsign $GPG_SIGN_ARGS --output "${artifact}.sha256.asc" "${artifact}.sha256"

  echo "   ✅ Signature: ${artifact}.asc"
  echo "   ✅ Checksum: ${artifact}.sha256.asc"

  SIGNED_COUNT=$((SIGNED_COUNT + 1))
done

echo ""
echo "✅ Signed $SIGNED_COUNT artifacts"
echo ""
echo "📋 Verification instructions for users:"
echo "   gpg --import bazbom-public.asc"
echo "   gpg --verify bazbom-linux-amd64.asc bazbom-linux-amd64"
echo ""
echo "📤 Ready to upload to GitHub Release"
