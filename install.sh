#!/usr/bin/env bash
# BazBOM installer script
# Usage: curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
# Or: wget -qO- https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh

set -e

# Configuration
REPO="cboyd0319/BazBOM"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux*)
            OS="linux"
            ;;
        darwin*)
            OS="darwin"
            ;;
        msys*|mingw*|cygwin*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $os"
            ;;
    esac

    case "$arch" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    # Construct target triple
    if [ "$OS" = "linux" ]; then
        TARGET="${ARCH}-unknown-linux-gnu"
    elif [ "$OS" = "darwin" ]; then
        TARGET="${ARCH}-apple-darwin"
    elif [ "$OS" = "windows" ]; then
        TARGET="${ARCH}-pc-windows-msvc"
        error "Windows installation via script not yet supported. Please download from: https://github.com/${REPO}/releases"
    fi

    info "Detected platform: $OS ($ARCH) - Target: $TARGET"
}

# Get the latest version from GitHub releases
get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        info "Fetching latest version..."
        VERSION=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/' | sed 's/^v//')
        if [ -z "$VERSION" ]; then
            error "Failed to fetch latest version"
        fi
        success "Latest version: $VERSION"
    else
        info "Installing version: $VERSION"
        # Remove 'v' prefix if present
        VERSION=$(echo "$VERSION" | sed 's/^v//')
    fi
}

# Download and install BazBOM
install_bazbom() {
    local download_url="https://github.com/${REPO}/releases/download/v${VERSION}/bazbom-${TARGET}.tar.gz"
    local tmp_dir=$(mktemp -d)
    local archive="${tmp_dir}/bazbom.tar.gz"

    info "Downloading BazBOM v${VERSION} for ${TARGET}..."
    if ! curl -sSfL "$download_url" -o "$archive"; then
        rm -rf "$tmp_dir"
        error "Failed to download BazBOM from $download_url"
    fi
    success "Downloaded successfully"

    # Extract archive
    info "Extracting archive..."
    tar -xzf "$archive" -C "$tmp_dir"

    # Determine installation directory
    local needs_sudo=false
    if [ ! -w "$INSTALL_DIR" ]; then
        needs_sudo=true
        warn "Installation directory $INSTALL_DIR requires sudo privileges"
    fi

    # Install binary
    info "Installing to $INSTALL_DIR/bazbom..."
    if [ "$needs_sudo" = true ]; then
        if ! command -v sudo >/dev/null 2>&1; then
            error "sudo is required but not available. Please install manually or run as root."
        fi
        sudo install -m 755 "${tmp_dir}/bazbom" "$INSTALL_DIR/bazbom"
    else
        install -m 755 "${tmp_dir}/bazbom" "$INSTALL_DIR/bazbom"
    fi

    # Cleanup
    rm -rf "$tmp_dir"

    success "BazBOM installed successfully!"
}

# Verify installation
verify_installation() {
    info "Verifying installation..."

    if ! command -v bazbom >/dev/null 2>&1; then
        warn "bazbom command not found in PATH"
        warn "You may need to add $INSTALL_DIR to your PATH:"
        echo ""
        echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
        echo ""
        warn "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
        return 1
    fi

    local installed_version=$(bazbom --version 2>&1 | head -n1 || echo "unknown")
    success "BazBOM is ready to use!"
    echo ""
    echo "    Installed version: $installed_version"
    echo "    Location: $(which bazbom)"
    echo ""
}

# Print usage information
print_usage() {
    echo ""
    success "Quick Start:"
    echo ""
    echo "  1. Scan a project:"
    echo "     $ bazbom check"
    echo ""
    echo "  2. Run with reachability analysis (70-90% noise reduction):"
    echo "     $ bazbom scan --reachability"
    echo ""
    echo "  3. Auto-fix vulnerabilities:"
    echo "     $ bazbom fix"
    echo ""
    echo "  4. Get help:"
    echo "     $ bazbom --help"
    echo ""
    echo "  Documentation: https://github.com/${REPO}"
    echo ""
}

# Main installation flow
main() {
    echo ""
    echo "╔════════════════════════════════════════════════╗"
    echo "║          BazBOM Installer v1.0                 ║"
    echo "║  Polyglot reachability-first SBOM & SCA        ║"
    echo "╚════════════════════════════════════════════════╝"
    echo ""

    detect_platform
    get_latest_version
    install_bazbom

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if verify_installation; then
        print_usage
    else
        echo ""
        warn "Installation complete, but bazbom is not in PATH"
        echo ""
    fi

    success "Installation complete!"
}

# Run main installation
main
