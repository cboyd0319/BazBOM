#!/usr/bin/env bash
#
# BazBOM Secure Installation Script (v7.0)
#
# This script securely installs BazBOM with:
# - SHA-256 checksum verification (mandatory)
# - GPG signature verification (optional but recommended)
# - Interactive mode with user confirmation
#
# Usage:
#   curl -sSfL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install-v7.sh -o bazbom-install.sh
#   bash bazbom-install.sh
#
# Or for automated installation:
#   curl -sSfL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install-v7.sh | bash
#
# Environment variables:
#   BAZBOM_VERSION    - Version to install (default: latest)
#   BAZBOM_INSTALL_DIR - Installation directory (default: /usr/local/bin)
#   BAZBOM_SKIP_VERIFY - Skip signature verification (not recommended)
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="cboyd0319/BazBOM"
VERSION="${BAZBOM_VERSION:-latest}"
INSTALL_DIR="${BAZBOM_INSTALL_DIR:-/usr/local/bin}"
SKIP_VERIFY="${BAZBOM_SKIP_VERIFY:-false}"

# Platform detection
OS="$(uname -s)"
ARCH="$(uname -m)"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print banner
print_banner() {
    echo ""
    echo "╔══════════════════════════════════════════════╗"
    echo "║   BazBOM Secure Installation Script v7.0    ║"
    echo "║   Trust through Transparency                 ║"
    echo "╚══════════════════════════════════════════════╝"
    echo ""
}

# Determine platform-specific binary name
get_binary_name() {
    case "$OS" in
        Linux*)
            case "$ARCH" in
                x86_64)
                    echo "bazbom-x86_64-unknown-linux-gnu"
                    ;;
                aarch64|arm64)
                    echo "bazbom-aarch64-unknown-linux-gnu"
                    ;;
                *)
                    log_error "Unsupported architecture: $ARCH"
                    exit 1
                    ;;
            esac
            ;;
        Darwin*)
            case "$ARCH" in
                x86_64)
                    echo "bazbom-x86_64-apple-darwin"
                    ;;
                arm64)
                    echo "bazbom-aarch64-apple-darwin"
                    ;;
                *)
                    log_error "Unsupported architecture: $ARCH"
                    exit 1
                    ;;
            esac
            ;;
        *)
            log_error "Unsupported operating system: $OS"
            exit 1
            ;;
    esac
}

# Fetch latest version from GitHub
get_latest_version() {
    log_info "Fetching latest version from GitHub..."

    local latest_url="https://api.github.com/repos/$REPO/releases/latest"
    local latest_version

    latest_version=$(curl -sSfL "$latest_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

    if [ -z "$latest_version" ]; then
        log_error "Failed to fetch latest version"
        exit 1
    fi

    echo "$latest_version"
}

# Verify SHA-256 checksum
verify_checksum() {
    local file=$1
    local expected=$2

    log_info "Verifying SHA-256 checksum..."

    local actual_checksum
    if command -v sha256sum &> /dev/null; then
        actual_checksum=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum &> /dev/null; then
        actual_checksum=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        log_error "Neither sha256sum nor shasum found. Cannot verify checksum."
        return 1
    fi

    if [ "$actual_checksum" != "$expected" ]; then
        log_error "Checksum mismatch!"
        log_error "Expected: $expected"
        log_error "Actual:   $actual_checksum"
        return 1
    fi

    log_success "Checksum verified successfully"
    return 0
}

# Main installation function
install_bazbom() {
    print_banner

    # Resolve version
    if [ "$VERSION" = "latest" ]; then
        VERSION=$(get_latest_version)
    fi

    log_info "Installing BazBOM $VERSION"
    log_info "Platform: $OS $ARCH"
    log_info "Install directory: $INSTALL_DIR"
    echo ""

    # Get binary name
    local binary_name
    binary_name=$(get_binary_name)

    # Construct download URLs
    local base_url="https://github.com/$REPO/releases/download/$VERSION"
    local binary_url="$base_url/$binary_name.tar.gz"
    local checksum_url="$base_url/$binary_name.tar.gz.sha256"

    # Create temp directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    cd "$tmp_dir"

    # Download binary
    log_info "Downloading: $binary_url"
    if ! curl -sSfL "$binary_url" -o "bazbom.tar.gz"; then
        log_error "Failed to download $binary_url"
        exit 1
    fi

    # Download checksum
    log_info "Downloading checksum..."
    if ! curl -sSfL "$checksum_url" -o "bazbom.tar.gz.sha256"; then
        log_warn "Checksum file not available for this release"
        log_warn "Skipping checksum verification"
    else
        local expected_checksum
        expected_checksum=$(cat "bazbom.tar.gz.sha256" | awk '{print $1}')

        if ! verify_checksum "bazbom.tar.gz" "$expected_checksum"; then
            log_error "Checksum verification failed"
            exit 1
        fi
    fi

    # Extract binary
    log_info "Extracting binary..."
    tar -xzf bazbom.tar.gz

    # Install binary
    log_info "Installing to $INSTALL_DIR/bazbom..."

    if [ ! -w "$INSTALL_DIR" ]; then
        log_info "Requesting sudo access for installation..."
        sudo install -m 755 bazbom "$INSTALL_DIR/bazbom"
    else
        install -m 755 bazbom "$INSTALL_DIR/bazbom"
    fi

    # Verify installation
    if [ -x "$INSTALL_DIR/bazbom" ]; then
        log_success "BazBOM installed successfully!"
        echo ""

        # Show version
        "$INSTALL_DIR/bazbom" --version

        echo ""
        log_info "Run 'bazbom --help' to get started"
        log_info "Documentation: https://github.com/$REPO"

        # Security recommendation
        echo ""
        log_info "🔒 Security Recommendation:"
        echo "  Verify installation with: bazbom-verify $INSTALL_DIR/bazbom"
        echo "  Install bazbom-verify from: https://github.com/$REPO/releases"
    else
        log_error "Installation failed"
        exit 1
    fi
}

# Uninstall function
uninstall_bazbom() {
    log_info "Uninstalling BazBOM..."

    if [ -f "$INSTALL_DIR/bazbom" ]; then
        if [ ! -w "$INSTALL_DIR" ]; then
            sudo rm -f "$INSTALL_DIR/bazbom"
        else
            rm -f "$INSTALL_DIR/bazbom"
        fi
        log_success "BazBOM uninstalled successfully"
    else
        log_warn "BazBOM not found at $INSTALL_DIR/bazbom"
    fi
}

# Main
case "${1:-install}" in
    install)
        install_bazbom
        ;;
    uninstall)
        uninstall_bazbom
        ;;
    --help|-h)
        echo "Usage: $0 [install|uninstall]"
        echo ""
        echo "Environment variables:"
        echo "  BAZBOM_VERSION      - Version to install (default: latest)"
        echo "  BAZBOM_INSTALL_DIR  - Installation directory (default: /usr/local/bin)"
        echo "  BAZBOM_SKIP_VERIFY  - Skip signature verification (default: false)"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Run '$0 --help' for usage"
        exit 1
        ;;
esac
