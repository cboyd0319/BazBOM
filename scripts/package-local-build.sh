#!/usr/bin/env bash
# Package the locally built BazBOM binary for testing
# Usage: ./scripts/package-local-build.sh

set -euo pipefail

echo "Packaging BazBOM local build..."
echo ""

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux*)
        OS_NAME="linux"
        ;;
    darwin*)
        OS_NAME="darwin"
        ;;
    *)
        echo "Error: Unsupported OS: $OS"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        ARCH_NAME="x86_64"
        ;;
    aarch64|arm64)
        ARCH_NAME="aarch64"
        ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Construct target triple
if [ "$OS_NAME" = "linux" ]; then
    TARGET="${ARCH_NAME}-unknown-linux-gnu"
elif [ "$OS_NAME" = "darwin" ]; then
    TARGET="${ARCH_NAME}-apple-darwin"
fi

echo "Platform: $OS_NAME ($ARCH_NAME)"
echo "Target: $TARGET"
echo ""

# Check if binary exists
BINARY="target/release/bazbom"
if [ ! -f "$BINARY" ]; then
    echo "Error: Binary not found at $BINARY"
    echo "Run 'cargo build --release -p bazbom' first"
    exit 1
fi

# Get version
VERSION=$($BINARY --version | awk '{print $2}')
echo "Version: $VERSION"
echo ""

# Create dist directory
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

# Package binary
ARCHIVE="$DIST_DIR/bazbom-${TARGET}.tar.gz"
echo "Creating archive: $ARCHIVE"
tar -czf "$ARCHIVE" -C target/release bazbom

# Generate SHA256 checksum
SHA256=$(sha256sum "$ARCHIVE" | awk '{print $1}')
echo "$SHA256" > "$ARCHIVE.sha256"

# Display results
echo ""
echo "✓ Package created successfully!"
echo ""
echo "Archive: $ARCHIVE"
echo "SHA256:  $SHA256"
echo "Checksum file: $ARCHIVE.sha256"
echo ""
echo "To test the install script:"
echo "  1. Start a local HTTP server:"
echo "     python3 -m http.server 8000"
echo ""
echo "  2. In another terminal, test the installer:"
echo "     VERSION=$VERSION INSTALL_DIR=\$HOME/.local/bin bash install.sh"
echo ""
echo "Binary size: $(du -h $ARCHIVE | cut -f1)"
