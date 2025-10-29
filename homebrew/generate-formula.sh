#!/usr/bin/env bash
# Generate Homebrew formula with SHA256 hashes from release artifacts
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>"
  echo "Example: $0 0.1.0"
  exit 1
fi

REPO="cboyd0319/BazBOM"
TEMPLATE="homebrew/bazbom.rb.template"
OUTPUT="homebrew/bazbom.rb"

# Download URLs
MACOS_ARM64_URL="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-aarch64-apple-darwin.tar.gz"
MACOS_X86_64_URL="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-x86_64-apple-darwin.tar.gz"
LINUX_ARM64_URL="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-aarch64-unknown-linux-gnu.tar.gz"
LINUX_X86_64_URL="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-x86_64-unknown-linux-gnu.tar.gz"

echo "Generating Homebrew formula for BazBOM v${VERSION}"
echo "================================================"

# Function to fetch SHA256 from GitHub release
fetch_sha256() {
  local url="$1"
  local sha256_url="${url}.sha256"
  echo "Fetching SHA256 from ${sha256_url}..." >&2
  
  # Try to fetch the .sha256 file first
  if sha=$(curl -sSfL "$sha256_url" 2>/dev/null); then
    echo "$sha"
    return 0
  fi
  
  # If .sha256 file doesn't exist, download the tarball and compute SHA256
  echo "Warning: .sha256 file not found, downloading tarball to compute SHA256..." >&2
  local tmpfile=$(mktemp)
  if curl -sSfL "$url" -o "$tmpfile"; then
    sha=$(sha256sum "$tmpfile" | awk '{print $1}')
    rm "$tmpfile"
    echo "$sha"
    return 0
  else
    rm "$tmpfile"
    echo "Error: Failed to download $url" >&2
    return 1
  fi
}

# Fetch SHA256 hashes
echo ""
echo "Fetching SHA256 hashes from release assets..."
SHA256_MACOS_ARM64=$(fetch_sha256 "$MACOS_ARM64_URL")
SHA256_MACOS_X86_64=$(fetch_sha256 "$MACOS_X86_64_URL")
SHA256_LINUX_ARM64=$(fetch_sha256 "$LINUX_ARM64_URL")
SHA256_LINUX_X86_64=$(fetch_sha256 "$LINUX_X86_64_URL")

echo ""
echo "SHA256 Hashes:"
echo "  macOS ARM64:  $SHA256_MACOS_ARM64"
echo "  macOS x86_64: $SHA256_MACOS_X86_64"
echo "  Linux ARM64:  $SHA256_LINUX_ARM64"
echo "  Linux x86_64: $SHA256_LINUX_X86_64"

# Generate formula from template
echo ""
echo "Generating formula..."
sed -e "s/VERSION/$VERSION/g" \
    -e "s/SHA256_MACOS_ARM64/$SHA256_MACOS_ARM64/g" \
    -e "s/SHA256_MACOS_X86_64/$SHA256_MACOS_X86_64/g" \
    -e "s/SHA256_LINUX_ARM64/$SHA256_LINUX_ARM64/g" \
    -e "s/SHA256_LINUX_X86_64/$SHA256_LINUX_X86_64/g" \
    "$TEMPLATE" > "$OUTPUT"

echo ""
echo "Formula generated successfully: $OUTPUT"
echo ""
echo "Next steps:"
echo "1. Review the generated formula: cat $OUTPUT"
echo "2. Copy it to your homebrew-bazbom repository: cp $OUTPUT /path/to/homebrew-bazbom/Formula/"
echo "3. Commit and push to the tap repository"
echo "4. Users can install with: brew tap ${REPO%/*}/bazbom && brew install bazbom"
