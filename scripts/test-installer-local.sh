#!/usr/bin/env bash
# One-command local installer testing
# This script builds, packages, and tests the BazBOM installer locally
# without requiring a GitHub release
#
# Usage: ./scripts/test-installer-local.sh [--skip-build] [--keep-server]

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

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

# Parse arguments
SKIP_BUILD=false
KEEP_SERVER=false
for arg in "$@"; do
    case $arg in
        --skip-build) SKIP_BUILD=true ;;
        --keep-server) KEEP_SERVER=true ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --skip-build    Skip cargo build step (use existing binary)"
            echo "  --keep-server   Keep the HTTP server running after test"
            echo "  --help          Show this help message"
            exit 0
            ;;
    esac
done

echo ""
echo "╔════════════════════════════════════════════════╗"
echo "║     BazBOM Installer Local Test Suite         ║"
echo "╚════════════════════════════════════════════════╝"
echo ""

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux*) OS_NAME="linux" ;;
    darwin*) OS_NAME="darwin" ;;
    *) error "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH_NAME="x86_64" ;;
    aarch64|arm64) ARCH_NAME="aarch64" ;;
    *) error "Unsupported architecture: $ARCH" ;;
esac

if [ "$OS_NAME" = "linux" ]; then
    TARGET="${ARCH_NAME}-unknown-linux-gnu"
elif [ "$OS_NAME" = "darwin" ]; then
    TARGET="${ARCH_NAME}-apple-darwin"
fi

info "Platform: $OS_NAME ($ARCH_NAME) - Target: $TARGET"
echo ""

# Step 1: Build binary
if [ "$SKIP_BUILD" = false ]; then
    info "Step 1/7: Building BazBOM binary..."
    if ! cargo build --release -p bazbom 2>&1 | grep -E "(Compiling|Finished|error)" | tail -20; then
        error "Build failed"
    fi
    success "Build complete"
else
    info "Step 1/7: Skipping build (--skip-build)"
fi
echo ""

# Verify binary exists
BINARY="target/release/bazbom"
if [ ! -f "$BINARY" ]; then
    error "Binary not found at $BINARY"
fi

VERSION=$($BINARY --version | awk '{print $2}')
info "Version: $VERSION"
echo ""

# Step 2: Package binary
info "Step 2/7: Packaging binary..."
DIST_DIR="dist"
mkdir -p "$DIST_DIR"
ARCHIVE="$DIST_DIR/bazbom-${TARGET}.tar.gz"
tar -czf "$ARCHIVE" -C target/release bazbom
SHA256=$(sha256_hash "$ARCHIVE")
echo "$SHA256" > "$ARCHIVE.sha256"
success "Package created: $ARCHIVE"
echo "         SHA256: $SHA256"
echo ""

# Step 3: Create mock release structure
info "Step 3/7: Creating mock release structure..."
MOCK_DIR="/tmp/bazbom-mock-release"
rm -rf "$MOCK_DIR"
mkdir -p "$MOCK_DIR/releases/download/v${VERSION}"
cp "$ARCHIVE" "$MOCK_DIR/releases/download/v${VERSION}/"
cp "$ARCHIVE.sha256" "$MOCK_DIR/releases/download/v${VERSION}/"

# Create mock GitHub API response
cat > "$MOCK_DIR/releases/latest" << EOF
{
  "tag_name": "v${VERSION}",
  "name": "v${VERSION}",
  "draft": false,
  "prerelease": false
}
EOF
success "Mock release structure created"
echo ""

# Step 4: Start local HTTP server
info "Step 4/7: Starting local HTTP server on port 8888..."
cd "$MOCK_DIR"
python3 -m http.server 8888 > /dev/null 2>&1 &
SERVER_PID=$!
cd - > /dev/null

# Function to cleanup
cleanup() {
    if [ "$KEEP_SERVER" = false ]; then
        if [ ! -z "$SERVER_PID" ] && kill -0 $SERVER_PID 2>/dev/null; then
            info "Stopping HTTP server (PID: $SERVER_PID)..."
            kill $SERVER_PID 2>/dev/null || true
            wait $SERVER_PID 2>/dev/null || true
        fi
        rm -rf "$MOCK_DIR"
    else
        success "HTTP server still running (PID: $SERVER_PID)"
        info "Mock release available at: http://localhost:8888"
        info "Stop with: kill $SERVER_PID"
    fi
}
trap cleanup EXIT

sleep 2
if ! kill -0 $SERVER_PID 2>/dev/null; then
    error "Failed to start HTTP server"
fi
success "HTTP server running (PID: $SERVER_PID)"
echo ""

# Step 5: Test HTTP server
info "Step 5/7: Verifying mock server..."
if curl -sSf "http://localhost:8888/releases/latest" > /dev/null 2>&1; then
    success "Mock GitHub API responding"
else
    error "Mock server not responding"
fi
if curl -sSf "http://localhost:8888/releases/download/v${VERSION}/bazbom-${TARGET}.tar.gz" > /dev/null 2>&1; then
    success "Mock release artifact accessible"
else
    error "Release artifact not accessible"
fi
echo ""

# Step 6: Create modified install script
info "Step 6/7: Creating test install script..."
TEST_INSTALL_SCRIPT="/tmp/bazbom-test-install.sh"
sed "s|https://github.com/cboyd0319/BazBOM|http://localhost:8888|g" install.sh > "$TEST_INSTALL_SCRIPT"
sed -i.bak "s|https://api.github.com/repos/cboyd0319/BazBOM|http://localhost:8888|g" "$TEST_INSTALL_SCRIPT"
chmod +x "$TEST_INSTALL_SCRIPT"
success "Test install script created: $TEST_INSTALL_SCRIPT"
echo ""

# Step 7: Test installation
info "Step 7/7: Testing installation..."
TEST_INSTALL_DIR="/tmp/bazbom-test-install"
rm -rf "$TEST_INSTALL_DIR"
mkdir -p "$TEST_INSTALL_DIR"

echo ""
info "Running installer..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if INSTALL_DIR="$TEST_INSTALL_DIR" VERSION="$VERSION" bash "$TEST_INSTALL_SCRIPT"; then
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    success "Installation completed successfully!"
else
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    error "Installation failed"
fi
echo ""

# Verify installation
info "Verifying installation..."
if [ -f "$TEST_INSTALL_DIR/bazbom" ]; then
    success "Binary installed to: $TEST_INSTALL_DIR/bazbom"

    # Test binary
    INSTALLED_VERSION=$("$TEST_INSTALL_DIR/bazbom" --version 2>&1 | head -n1)
    if [ ! -z "$INSTALLED_VERSION" ]; then
        success "Binary is executable: $INSTALLED_VERSION"
    else
        error "Binary is not executable"
    fi

    # Test help command
    if "$TEST_INSTALL_DIR/bazbom" --help > /dev/null 2>&1; then
        success "Help command works"
    else
        warn "Help command failed"
    fi
else
    error "Binary not found at $TEST_INSTALL_DIR/bazbom"
fi
echo ""

# Summary
echo "╔════════════════════════════════════════════════╗"
echo "║              Test Results Summary              ║"
echo "╚════════════════════════════════════════════════╝"
echo ""
success "All tests passed!"
echo ""
echo "Test artifacts:"
echo "  • Binary:           $BINARY"
echo "  • Package:          $ARCHIVE"
echo "  • Installed binary: $TEST_INSTALL_DIR/bazbom"
echo "  • Mock server:      http://localhost:8888 (PID: $SERVER_PID)"
echo ""
echo "Next steps:"
echo "  1. Test the installed binary:"
echo "     $TEST_INSTALL_DIR/bazbom --version"
echo ""
echo "  2. When ready, create a real release:"
echo "     git tag -a v$VERSION -m 'Release v$VERSION'"
echo "     git push origin v$VERSION"
echo ""
echo "  3. Or trigger a test build via GitHub Actions:"
echo "     ./scripts/trigger-installer-build.sh"
echo ""
