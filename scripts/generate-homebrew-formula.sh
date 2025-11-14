#!/usr/bin/env bash
# Generate Homebrew formula with SHA256 checksums for a release
# Usage: ./scripts/generate-homebrew-formula.sh <version>

set -euo pipefail

# Cross-platform SHA256 function
sha256_hash() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        echo "Error: Neither sha256sum nor shasum found" >&2
        exit 1
    fi
}

VERSION="${1:-6.5.0}"
REPO="cboyd0319/BazBOM"

echo "Generating Homebrew formula for BazBOM v${VERSION}"
echo ""

# Download release artifacts and generate checksums
echo "Downloading release artifacts..."
mkdir -p /tmp/bazbom-checksums
cd /tmp/bazbom-checksums

declare -A CHECKSUMS
declare -a PLATFORMS=(
    "aarch64-apple-darwin"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "x86_64-unknown-linux-gnu"
)

for platform in "${PLATFORMS[@]}"; do
    url="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-${platform}.tar.gz"
    echo "  Downloading: $url"

    if curl -fsSL "$url" -o "bazbom-${platform}.tar.gz" 2>/dev/null; then
        checksum=$(sha256_hash "bazbom-${platform}.tar.gz")
        CHECKSUMS[$platform]=$checksum
        echo "    ✓ SHA256: $checksum"
    else
        echo "    ✗ Failed to download. Using placeholder."
        CHECKSUMS[$platform]="INSERT_SHA256_HERE"
    fi
done

echo ""
echo "Generating Formula/bazbom.rb..."

# Generate the formula
cat > Formula-bazbom.rb << EOF
class Bazbom < Formula
  desc "Polyglot reachability-first SBOM, SCA, and dependency graph"
  homepage "https://github.com/${REPO}"
  version "${VERSION}"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-aarch64-apple-darwin.tar.gz"
      sha256 "${CHECKSUMS[aarch64-apple-darwin]}"
    else
      url "https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-x86_64-apple-darwin.tar.gz"
      sha256 "${CHECKSUMS[x86_64-apple-darwin]}"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "${CHECKSUMS[aarch64-unknown-linux-gnu]}"
    else
      url "https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "${CHECKSUMS[x86_64-unknown-linux-gnu]}"
    end
  end

  def install
    bin.install "bazbom"

    # TODO: Add shell completions once 'bazbom completions' subcommand is implemented
    # generate_completions_from_executable(bin/"bazbom", "completions")
  end

  test do
    assert_match "bazbom", shell_output("#{bin}/bazbom --version")

    # Basic functionality test
    system bin/"bazbom", "--help"
  end
end
EOF

echo ""
echo "Formula generated at: /tmp/bazbom-checksums/Formula-bazbom.rb"
echo ""
echo "To use this formula:"
echo "1. Create the homebrew-bazbom repository:"
echo "   gh repo create homebrew-bazbom --public --description 'Homebrew tap for BazBOM'"
echo ""
echo "2. Clone and set up the repository:"
echo "   git clone https://github.com/${REPO%/*}/homebrew-bazbom.git"
echo "   cd homebrew-bazbom"
echo "   mkdir -p Formula"
echo "   cp /tmp/bazbom-checksums/Formula-bazbom.rb Formula/bazbom.rb"
echo ""
echo "3. Commit and push:"
echo "   git add Formula/bazbom.rb"
echo "   git commit -m 'Add bazbom formula v${VERSION}'"
echo "   git push origin main"
echo ""
echo "4. Test locally:"
echo "   brew tap ${REPO%/*}/bazbom"
echo "   brew install bazbom"
echo ""

# Cleanup
cd - > /dev/null
echo "Temporary files remain at: /tmp/bazbom-checksums"
