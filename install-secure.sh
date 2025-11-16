#!/bin/bash
# Secure installation script for BazBOM
# This script provides enhanced security over the basic install.sh:
# - Checksum verification (SHA-256)
# - GPG signature verification (when available)
# - No pipe-to-shell (download first, verify, then install)
# - Version pinning

set -e
set -u
set -o pipefail

# Configuration
REPO="cboyd0319/BazBOM"
VERSION="${VERSION:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TEMP_DIR=$(mktemp -d)
trap 'rm -rf "$TEMP_DIR"' EXIT

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
log_info() {
    echo -e "${GREEN}[+]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[x]${NC} $1"
}

# Detect platform
detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case "$os" in
        linux) OS="linux" ;;
        darwin) OS="darwin" ;;
        *) log_error "Unsupported OS: $os"; exit 1 ;;
    esac

    case "$arch" in
        x86_64|amd64) ARCH="x86_64" ;;
        aarch64|arm64) ARCH="arm64" ;;
        *) log_error "Unsupported architecture: $arch"; exit 1 ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    log_info "Detected platform: $PLATFORM"
}

# Get latest version from GitHub API
get_latest_version() {
    if [ "$VERSION" = "latest" ]; then
        log_info "Fetching latest version from GitHub..."
        VERSION=$(curl -sSf "https://api.github.com/repos/${REPO}/releases/latest" \
            | grep '"tag_name":' \
            | sed -E 's/.*"([^"]+)".*/\1/')

        if [ -z "$VERSION" ]; then
            log_error "Failed to fetch latest version"
            exit 1
        fi
        log_info "Latest version: $VERSION"
    fi
}

# Download file with retry
download_file() {
    local url=$1
    local output=$2
    local max_retries=3
    local retry=0

    while [ $retry -lt $max_retries ]; do
        if curl -sSfL "$url" -o "$output"; then
            return 0
        fi
        retry=$((retry + 1))
        log_warn "Download failed, retrying ($retry/$max_retries)..."
        sleep 2
    done

    log_error "Failed to download: $url"
    return 1
}

# Verify SHA-256 checksum
verify_checksum() {
    local file=$1
    local expected_checksum=$2

    log_info "Verifying checksum..."

    local actual_checksum
    if command -v sha256sum > /dev/null; then
        actual_checksum=$(sha256sum "$file" | awk '{print $1}')
    elif command -v shasum > /dev/null; then
        actual_checksum=$(shasum -a 256 "$file" | awk '{print $1}')
    else
        log_error "Neither sha256sum nor shasum found. Cannot verify checksum."
        exit 1
    fi

    if [ "$actual_checksum" = "$expected_checksum" ]; then
        log_info "Checksum verified: $actual_checksum"
        return 0
    else
        log_error "Checksum verification FAILED!"
        log_error "Expected: $expected_checksum"
        log_error "Actual:   $actual_checksum"
        exit 1
    fi
}

# Verify GPG signature
verify_signature() {
    local file=$1
    local signature=$2

    if ! command -v gpg > /dev/null; then
        log_warn "GPG not found. Skipping signature verification."
        log_warn "Install GPG for enhanced security: apt-get install gnupg"
        return 0
    fi

    log_info "Verifying GPG signature..."

    # Import BazBOM public key (if not already imported)
    # TODO: Replace with actual public key ID once GPG signing is set up
    # gpg --recv-keys BAZBOM_KEY_ID

    if gpg --verify "$signature" "$file" 2>/dev/null; then
        log_info "GPG signature verified"
        return 0
    else
        log_warn "GPG signature verification failed or not available"
        log_warn "Continuing based on checksum verification only"
        return 0
    fi
}

# Main installation
main() {
    log_info "BazBOM Secure Installation Script"
    log_info "=================================="

    # Check if running as root for system-wide installation
    if [ "$INSTALL_DIR" = "/usr/local/bin" ] && [ "$(id -u)" -ne 0 ]; then
        log_warn "Installation to $INSTALL_DIR requires root privileges"
        log_warn "Either run with sudo, or set INSTALL_DIR to a user-writable location:"
        log_warn "  export INSTALL_DIR=\$HOME/.local/bin"
        log_warn "  $0"
        exit 1
    fi

    detect_platform
    get_latest_version

    # Construct download URL
    local base_url="https://github.com/${REPO}/releases/download/${VERSION}"
    local binary_name="bazbom-${PLATFORM}.tar.gz"
    local checksum_file="bazbom-${PLATFORM}.tar.gz.sha256"
    local signature_file="bazbom-${PLATFORM}.tar.gz.asc"

    log_info "Downloading BazBOM ${VERSION}..."

    # Download binary
    download_file "${base_url}/${binary_name}" "${TEMP_DIR}/${binary_name}"

    # Download checksum
    if download_file "${base_url}/${checksum_file}" "${TEMP_DIR}/${checksum_file}"; then
        expected_checksum=$(cat "${TEMP_DIR}/${checksum_file}" | awk '{print $1}')
        verify_checksum "${TEMP_DIR}/${binary_name}" "$expected_checksum"
    else
        log_warn "Checksum file not found. Skipping checksum verification."
        log_warn "This is less secure. Checksums will be available in future releases."
    fi

    # Download and verify signature (if available)
    if download_file "${base_url}/${signature_file}" "${TEMP_DIR}/${signature_file}"; then
        verify_signature "${TEMP_DIR}/${binary_name}" "${TEMP_DIR}/${signature_file}"
    else
        log_warn "GPG signature not found. Skipping signature verification."
        log_warn "GPG signatures will be available in future releases."
    fi

    # Extract tarball
    log_info "Extracting..."
    tar -xzf "${TEMP_DIR}/${binary_name}" -C "$TEMP_DIR"

    # Install binary
    log_info "Installing to $INSTALL_DIR..."
    install -m 755 "${TEMP_DIR}/bazbom" "${INSTALL_DIR}/bazbom"

    # Verify installation
    if command -v bazbom > /dev/null; then
        log_info "Installation successful!"
        log_info "BazBOM version: $(bazbom --version)"
        log_info ""
        log_info "Get started with: bazbom scan"
    else
        log_warn "Installation completed, but bazbom not found in PATH"
        log_warn "You may need to add $INSTALL_DIR to your PATH:"
        log_warn "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Run main function
main
