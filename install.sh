#!/bin/bash
# BazBOM Installer Script
# Zero-configuration installation for BazBOM
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
#
# Or for local development:
#   ./install.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="cboyd0319"
REPO_NAME="BazBOM"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.bazbom}"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"

# Functions
log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}[OK]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

log_error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="darwin"
            ;;
        *)
            log_error "Unsupported OS: $OS"
            exit 1
            ;;
    esac
    
    case "$ARCH" in
        x86_64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            log_error "Unsupported architecture: $ARCH"
            exit 1
            ;;
    esac
    
    log_info "Detected platform: $OS-$ARCH"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for curl or wget
    if ! command -v curl &> /dev/null && ! command -v wget &> /dev/null; then
        log_error "Neither curl nor wget is installed. Please install one and try again."
        exit 1
    fi
    
    # Check for Python 3
    if ! command -v python3 &> /dev/null; then
        log_error "Python 3 is required but not installed."
        log_info "Please install Python 3.8 or later and try again."
        exit 1
    fi
    
    # Check Python version
    PYTHON_VERSION=$(python3 -c 'import sys; print(".".join(map(str, sys.version_info[:2])))')
    log_info "Found Python $PYTHON_VERSION"
    
    # Check for git (optional but recommended)
    if ! command -v git &> /dev/null; then
        log_warning "Git is not installed. Some features may not work."
    fi
    
    # Check for ripgrep (optional but recommended for fast scanning)
    if ! command -v rg &> /dev/null; then
        log_warning "RipGrep not found - fast scanning disabled"
        log_info "   For 100x faster dependency scanning, install RipGrep:"
        log_info "   - Debian/Ubuntu: apt install ripgrep"
        log_info "   - RHEL/CentOS: yum install ripgrep"
        log_info "   - macOS: brew install ripgrep"
        log_info "   - Or see: https://github.com/BurntSushi/ripgrep#installation"
    else
        RG_VERSION=$(rg --version | head -n1)
        log_success "RipGrep detected - enabling fast mode"
        log_info "   $RG_VERSION"
    fi
    
    log_success "Prerequisites check passed"
}

# Download or clone BazBOM
install_bazbom() {
    log_info "Installing BazBOM to $INSTALL_DIR..."
    
    # Create install directory
    mkdir -p "$INSTALL_DIR"
    
    # Clone repository
    if command -v git &> /dev/null; then
        log_info "Cloning BazBOM repository..."
        if [ -d "$INSTALL_DIR/.git" ]; then
            log_info "BazBOM already installed, updating..."
            cd "$INSTALL_DIR"
            git pull origin main
        else
            git clone "https://github.com/${REPO_OWNER}/${REPO_NAME}.git" "$INSTALL_DIR"
        fi
    else
        log_error "Git not found. Cannot clone repository."
        log_info "Please install git or download BazBOM manually."
        exit 1
    fi
    
    log_success "BazBOM installed to $INSTALL_DIR"
}

# Create wrapper script
create_wrapper() {
    log_info "Creating command-line wrapper..."
    
    # Create wrapper script
    cat > "$INSTALL_DIR/bazbom" << 'EOF'
#!/bin/bash
# BazBOM CLI wrapper script

# Find Python 3
if command -v python3 &> /dev/null; then
    PYTHON=python3
else
    echo "Error: Python 3 is required but not found" >&2
    exit 1
fi

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Run BazBOM CLI
exec "$PYTHON" "$SCRIPT_DIR/tools/supplychain/bazbom_cli.py" "$@"
EOF
    
    chmod +x "$INSTALL_DIR/bazbom"
    
    log_success "Wrapper script created"
}

# Install to system PATH
install_to_path() {
    log_info "Installing to system PATH..."
    
    # Check if we have write permission
    if [ -w "$BIN_DIR" ]; then
        ln -sf "$INSTALL_DIR/bazbom" "$BIN_DIR/bazbom"
        log_success "Installed to $BIN_DIR/bazbom"
    else
        log_warning "Cannot write to $BIN_DIR (need sudo)"
        log_info "Attempting installation with sudo..."
        
        if sudo ln -sf "$INSTALL_DIR/bazbom" "$BIN_DIR/bazbom"; then
            log_success "Installed to $BIN_DIR/bazbom"
        else
            log_warning "Could not install to $BIN_DIR"
            log_info "You can manually add to PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
        fi
    fi
}

# Detect and configure project
configure_project() {
    log_info "Detecting project type..."
    
    # Check if we're in a project directory
    if [ -f "WORKSPACE" ] || [ -f "MODULE.bazel" ]; then
        log_success "Bazel project detected"
        configure_bazel_project
    elif [ -f "pom.xml" ]; then
        log_success "Maven project detected"
        log_info "Run: bazbom scan"
    elif [ -f "build.gradle" ] || [ -f "build.gradle.kts" ]; then
        log_success "Gradle project detected"
        log_info "Run: bazbom scan"
    else
        log_info "No project detected in current directory"
        log_info "Navigate to your project and run: bazbom init"
    fi
}

# Configure Bazel project
configure_bazel_project() {
    log_info "Setting up BazBOM for Bazel project..."
    
    # Check if BazBOM is already configured
    if grep -q "bazbom" WORKSPACE 2>/dev/null || grep -q "bazbom" MODULE.bazel 2>/dev/null; then
        log_info "BazBOM already configured in workspace"
        return
    fi
    
    # Get latest version
    log_info "Fetching latest BazBOM version..."
    
    if command -v curl &> /dev/null; then
        VERSION=$(curl -s "https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/' || echo "main")
    else
        VERSION="main"
    fi
    
    log_info "Using version: $VERSION"
    
    # Add to WORKSPACE if not present
    if [ -f "WORKSPACE" ] && ! grep -q "bazbom" WORKSPACE; then
        log_info "Adding BazBOM to WORKSPACE..."
        
        cat >> WORKSPACE << EOF

# BazBOM - Auto-configured by installer
# For manual configuration, see: https://github.com/${REPO_OWNER}/${REPO_NAME}
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# Note: For production use, pin to a specific version and verify SHA256
# http_archive(
#     name = "bazbom",
#     urls = ["https://github.com/${REPO_OWNER}/${REPO_NAME}/archive/${VERSION}.tar.gz"],
#     strip_prefix = "${REPO_NAME}-${VERSION}",
# )

# Development: use local repository
local_repository(
    name = "bazbom",
    path = "${INSTALL_DIR}",
)
EOF
        
        log_success "Added BazBOM to WORKSPACE"
    fi
    
    # Create or update BUILD.bazel
    if [ ! -f "BUILD.bazel" ] || ! grep -q "sbom_all" BUILD.bazel 2>/dev/null; then
        log_info "Adding BazBOM targets to BUILD.bazel..."
        
        if [ ! -f "BUILD.bazel" ]; then
            cat > BUILD.bazel << 'EOF'
# BazBOM targets
load("@bazbom//:defs.bzl", "sbom_all")

sbom_all(name = "sbom_all")
EOF
        else
            cat >> BUILD.bazel << 'EOF'

# BazBOM targets - Auto-configured by installer
# load("@bazbom//:defs.bzl", "sbom_all")
# sbom_all(name = "sbom_all")
EOF
        fi
        
        log_success "Added BazBOM targets to BUILD.bazel"
    fi
    
    log_info "Bazel configuration complete!"
    log_info "Try: bazel build //:sbom_all"
}

# Verify installation
verify_installation() {
    log_info "Verifying installation..."
    
    # Check if bazbom command works
    if command -v bazbom &> /dev/null; then
        VERSION_OUTPUT=$(bazbom version 2>&1 || echo "")
        if [[ "$VERSION_OUTPUT" == *"BazBOM"* ]]; then
            log_success "Installation verified"
            echo ""
            bazbom version
        else
            log_error "Installation verification failed"
            exit 1
        fi
    else
        log_warning "bazbom command not in PATH"
        log_info "Add to your PATH: export PATH=\"$INSTALL_DIR:\$PATH\""
        log_info "Or use directly: $INSTALL_DIR/bazbom"
    fi
}

# Print usage instructions
print_usage() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    log_success "BazBOM installation complete!"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Quick Start:"
    echo ""
    echo "  # Scan a project"
    echo "  $ bazbom scan /path/to/project"
    echo ""
    echo "  # Initialize configuration"
    echo "  $ bazbom init"
    echo ""
    echo "  # Watch for changes"
    echo "  $ bazbom scan --watch"
    echo ""
    echo "  # For Bazel projects"
    echo "  $ bazel build //:sbom_all"
    echo ""
    echo "Documentation: https://github.com/${REPO_OWNER}/${REPO_NAME}"
    echo ""
}

# Main installation flow
main() {
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  BazBOM Installer"
    echo "  Software Bill of Materials and Security Analysis"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    
    detect_platform
    check_prerequisites
    install_bazbom
    create_wrapper
    install_to_path
    verify_installation
    
    # Only configure project if we're in a project directory
    if [ "$PWD" != "$HOME" ] && [ "$PWD" != "/" ]; then
        configure_project
    fi
    
    print_usage
}

# Run main installation
main
