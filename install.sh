#!/usr/bin/env bash
# BazBOM installer script
# Usage: curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
# Or: wget -qO- https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
#
# Environment variables:
#   INSTALL_DIR - Installation directory (default: /usr/local/bin)
#   VERSION - Specific version to install (default: latest)
#   SKIP_JAVA_CHECK - Skip Java dependency check (default: 0)
#   SKIP_POST_INSTALL_TEST - Skip post-install test (default: 0)

set -e

# Configuration
REPO="cboyd0319/BazBOM"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
VERSION="${VERSION:-latest}"
SKIP_JAVA_CHECK="${SKIP_JAVA_CHECK:-0}"
SKIP_POST_INSTALL_TEST="${SKIP_POST_INSTALL_TEST:-0}"

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

note() {
    echo -e "${BLUE}→${NC} $1"
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

# Check for Java installation (optional but recommended for JVM projects)
check_java() {
    if [ "$SKIP_JAVA_CHECK" = "1" ]; then
        return 0
    fi

    if command -v java >/dev/null 2>&1; then
        local java_version=$(java -version 2>&1 | head -n1 | awk -F '"' '{print $2}')
        success "Java detected: $java_version"
        return 0
    fi

    warn "Java not found - Required for scanning JVM projects (Java, Kotlin, Scala, etc.)"
    echo ""
    note "Install Java 11+ to enable full reachability analysis on JVM projects:"
    if [ "$OS" = "darwin" ]; then
        echo "    brew install openjdk@21"
        echo "    # Or download from: https://adoptium.net/"
    else
        echo "    sudo apt-get install openjdk-21-jdk  # Debian/Ubuntu"
        echo "    sudo yum install java-21-openjdk     # RHEL/CentOS"
    fi
    echo ""
    note "You can still use BazBOM for non-JVM projects without Java"
    echo ""
}

# Remove macOS quarantine attribute
remove_macos_quarantine() {
    if [ "$OS" != "darwin" ]; then
        return 0
    fi

    local binary_path="$INSTALL_DIR/bazbom"

    if xattr "$binary_path" 2>/dev/null | grep -q "com.apple.quarantine"; then
        info "Removing macOS quarantine attribute..."
        if [ -w "$binary_path" ]; then
            xattr -d com.apple.quarantine "$binary_path" 2>/dev/null || true
            success "macOS quarantine removed"
        else
            if command -v sudo >/dev/null 2>&1; then
                sudo xattr -d com.apple.quarantine "$binary_path" 2>/dev/null || true
                success "macOS quarantine removed (with sudo)"
            else
                warn "Could not remove quarantine attribute - you may see a security warning"
                note "Run manually: sudo xattr -d com.apple.quarantine $binary_path"
            fi
        fi
    fi
}

# Run post-install test
run_post_install_test() {
    if [ "$SKIP_POST_INSTALL_TEST" = "1" ]; then
        return 0
    fi

    echo ""
    info "Running post-install test..."

    # Test 1: Version check
    if ! bazbom --version >/dev/null 2>&1; then
        warn "Version check failed - BazBOM may not be working correctly"
        return 1
    fi

    # Test 2: Help command
    if ! bazbom --help >/dev/null 2>&1; then
        warn "Help command failed - BazBOM may not be working correctly"
        return 1
    fi

    success "Post-install test passed!"
    return 0
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

    # Remove macOS quarantine if needed
    remove_macos_quarantine
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
    echo "     $ cd /path/to/your/project"
    echo "     $ bazbom check"
    echo ""
    echo "  2. Run with reachability analysis (70-90% noise reduction):"
    echo "     $ bazbom scan --reachability"
    echo ""
    echo "  3. Auto-fix vulnerabilities:"
    echo "     $ bazbom fix --suggest"
    echo ""
    echo "  4. Get help:"
    echo "     $ bazbom --help"
    echo ""
    if [ "$OS" = "darwin" ]; then
        note "macOS users: See docs/getting-started/MACOS_QUICK_START.md for detailed guide"
    fi
    echo ""
    echo "  Documentation: https://github.com/${REPO}"
    echo ""
}

# Main installation flow
main() {
    echo ""
    echo "╔════════════════════════════════════════════════╗"
    echo "║          BazBOM Installer v2.0                 ║"
    echo "║  Polyglot reachability-first SBOM & SCA        ║"
    echo "╚════════════════════════════════════════════════╝"
    echo ""

    detect_platform
    check_java
    get_latest_version
    install_bazbom

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    if verify_installation; then
        run_post_install_test
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
