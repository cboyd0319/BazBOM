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

    local needs_java=false
    local java_too_old=false

    if command -v java >/dev/null 2>&1; then
        # Java is installed, check version
        # Try to extract version from java -version output (handles multiple formats)
        local java_output=$(java -version 2>&1 | head -n1)
        local java_version=""

        # Try extracting version from quoted format: java version "11.0.20"
        if echo "$java_output" | grep -q '"'; then
            java_version=$(echo "$java_output" | awk -F '"' '{print $2}')
        # Try extracting from unquoted format: openjdk version 11.0.20
        elif echo "$java_output" | grep -qE 'version [0-9]'; then
            java_version=$(echo "$java_output" | grep -oE '[0-9]+\.[0-9]+(\.[0-9]+)?(_[0-9]+)?' | head -n1)
        fi

        # Extract major version (handles both 1.8.x and 11+ formats)
        local java_major=""
        if [ -n "$java_version" ]; then
            java_major=$(echo "$java_version" | awk -F. '{if ($1 == 1) print $2; else print $1}')
        fi

        # Validate that we got a numeric major version
        if [ -n "$java_major" ] && [ "$java_major" -ge 11 ] 2>/dev/null; then
            success "Java detected: $java_version (Java $java_major)"
            return 0
        elif [ -n "$java_version" ] && [ -n "$java_major" ]; then
            warn "Java $java_version detected, but BazBOM requires Java 11+"
            java_too_old=true
        else
            warn "Java found but could not determine version - BazBOM requires Java 11+"
            warn "Java output: $java_output"
            needs_java=true
        fi
    else
        warn "Java not found - Required for scanning JVM projects (Java, Kotlin, Scala, etc.)"
        needs_java=true
    fi

    echo ""

    # Offer auto-install on macOS if Homebrew is available
    if [ "$OS" = "darwin" ] && command -v brew >/dev/null 2>&1; then
        if [ "$needs_java" = true ] || [ "$java_too_old" = true ]; then
            read -p "Install Java 21 via Homebrew? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                info "Installing OpenJDK 21 via Homebrew..."
                if brew install openjdk@21; then
                    success "Java 21 installed successfully!"

                    # Add to PATH if needed
                    if [ -d "/opt/homebrew/opt/openjdk@21" ]; then
                        note "Add to your PATH: export PATH=\"/opt/homebrew/opt/openjdk@21/bin:\$PATH\""
                    elif [ -d "/usr/local/opt/openjdk@21" ]; then
                        note "Add to your PATH: export PATH=\"/usr/local/opt/openjdk@21/bin:\$PATH\""
                    fi
                    return 0
                else
                    warn "Java installation failed. Please install manually."
                fi
            else
                note "Skipping Java installation"
            fi
        fi
    fi

    # Show manual installation instructions
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

# Setup shell completions
setup_shell_completions() {
    echo ""
    info "Shell completions can provide tab-completion for BazBOM commands"

    # Detect shell
    local detected_shell=""
    if [ -n "$ZSH_VERSION" ]; then
        detected_shell="zsh"
    elif [ -n "$BASH_VERSION" ]; then
        detected_shell="bash"
    elif [ -n "$FISH_VERSION" ]; then
        detected_shell="fish"
    else
        # Try to detect from SHELL environment variable
        case "$SHELL" in
            */zsh)
                detected_shell="zsh"
                ;;
            */bash)
                detected_shell="bash"
                ;;
            */fish)
                detected_shell="fish"
                ;;
        esac
    fi

    if [ -z "$detected_shell" ]; then
        note "Could not auto-detect shell. Run 'bazbom --help' to see completion setup instructions"
        return 0
    fi

    read -p "Install $detected_shell completions? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        note "Skipping completions. You can set them up later - see docs/getting-started/shell-completions.md"
        return 0
    fi

    case "$detected_shell" in
        zsh)
            setup_zsh_completions
            ;;
        bash)
            setup_bash_completions
            ;;
        fish)
            setup_fish_completions
            ;;
    esac
}

# Setup Zsh completions
setup_zsh_completions() {
    local completion_dir="$HOME/.zsh/completion"
    local completion_file="$completion_dir/_bazbom"

    mkdir -p "$completion_dir"

    cat > "$completion_file" << 'EOF'
#compdef bazbom

_bazbom() {
    local -a commands
    commands=(
        'scan:Scan a project and generate SBOM + findings'
        'check:Fast vulnerability scan (< 10 seconds)'
        'ci:CI/CD mode with SARIF output'
        'pr:PR-optimized incremental scanning'
        'full:Complete analysis with all analyzers'
        'quick:5-second smoke test'
        'policy:Apply policy checks and output SARIF/JSON verdicts'
        'fix:Show remediation suggestions or apply fixes'
        'container-scan:Scan container images'
        'db:Advisory database operations (offline sync)'
        'license:License compliance operations'
        'install-hooks:Install git pre-commit hooks'
        'init:Interactive setup wizard'
        'explore:Interactive dependency graph explorer (TUI)'
        'dashboard:Start web dashboard server'
        'team:Team coordination and assignment management'
        'report:Generate security and compliance reports'
        'help:Print help message'
    )

    _arguments -C \
        '1: :->command' \
        '*:: :->args'

    case $state in
        command)
            _describe 'command' commands
            ;;
        args)
            case $words[1] in
                scan)
                    _arguments \
                        '--reachability[Enable reachability analysis]' \
                        '--fast[Fast mode: skip reachability analysis]' \
                        '--output-format=[Output format]:format:(spdx cyclonedx)' \
                        '--output=[Output file path]:file:_files'
                    ;;
                fix)
                    _arguments \
                        '--suggest[Suggest fixes without applying]' \
                        '--apply[Apply fixes automatically]'
                    ;;
            esac
            ;;
    esac
}

_bazbom "$@"
EOF

    # Add to .zshrc if not already present
    local zshrc="$HOME/.zshrc"
    if [ -f "$zshrc" ]; then
        if ! grep -q "fpath=.*\.zsh/completion" "$zshrc"; then
            echo "" >> "$zshrc"
            echo "# BazBOM shell completions" >> "$zshrc"
            echo "fpath=(~/.zsh/completion \$fpath)" >> "$zshrc"
            echo "autoload -Uz compinit && compinit" >> "$zshrc"
        fi
    fi

    success "Zsh completions installed to $completion_file"
    note "Restart your shell or run: source ~/.zshrc"
}

# Setup Bash completions
setup_bash_completions() {
    local completion_dir="$HOME/.bash_completion.d"
    local completion_file="$completion_dir/bazbom"

    mkdir -p "$completion_dir"

    cat > "$completion_file" << 'EOF'
#!/bin/bash

_bazbom_completions() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local commands="scan check ci pr full quick policy fix container-scan db license install-hooks init explore dashboard team report help"

    if [[ ${COMP_CWORD} -eq 1 ]]; then
        COMPREPLY=( $(compgen -W "${commands}" -- ${cur}) )
        return 0
    fi

    case "${prev}" in
        scan|check|ci|pr|full)
            COMPREPLY=( $(compgen -W "--reachability --fast --output-format --output" -- ${cur}) )
            ;;
        fix)
            COMPREPLY=( $(compgen -W "--suggest --apply" -- ${cur}) )
            ;;
        --output-format)
            COMPREPLY=( $(compgen -W "spdx cyclonedx" -- ${cur}) )
            ;;
        *)
            COMPREPLY=( $(compgen -f -- ${cur}) )
            ;;
    esac
}

complete -F _bazbom_completions bazbom
EOF

    # Add to .bashrc if not already present
    local bashrc="$HOME/.bashrc"
    if [ -f "$bashrc" ]; then
        if ! grep -q "source.*\.bash_completion\.d/bazbom" "$bashrc"; then
            echo "" >> "$bashrc"
            echo "# BazBOM shell completions" >> "$bashrc"
            echo "[ -f ~/.bash_completion.d/bazbom ] && source ~/.bash_completion.d/bazbom" >> "$bashrc"
        fi
    fi

    success "Bash completions installed to $completion_file"
    note "Restart your shell or run: source ~/.bashrc"
}

# Setup Fish completions
setup_fish_completions() {
    local completion_dir="$HOME/.config/fish/completions"
    local completion_file="$completion_dir/bazbom.fish"

    mkdir -p "$completion_dir"

    cat > "$completion_file" << 'EOF'
# bazbom completions for Fish shell

# Commands
complete -c bazbom -f -n '__fish_use_subcommand' -a 'scan' -d 'Scan a project and generate SBOM + findings'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'check' -d 'Fast vulnerability scan'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'ci' -d 'CI/CD mode with SARIF output'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'fix' -d 'Show remediation suggestions or apply fixes'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'policy' -d 'Apply policy checks'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'license' -d 'License compliance operations'
complete -c bazbom -f -n '__fish_use_subcommand' -a 'install-hooks' -d 'Install git pre-commit hooks'

# Scan command options
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l reachability -d 'Enable reachability analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l fast -d 'Fast mode: skip reachability analysis'
complete -c bazbom -n '__fish_seen_subcommand_from scan' -l output-format -a 'spdx cyclonedx' -d 'Output format'
EOF

    success "Fish completions installed to $completion_file"
    note "Completions will be available in new Fish shells"
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
        echo ""

        # Detect shell config file
        local shell_config=""
        if [ -n "$ZSH_VERSION" ] || [ "$SHELL" = */zsh ]; then
            shell_config="$HOME/.zshrc"
        elif [ -n "$BASH_VERSION" ] || [ "$SHELL" = */bash ]; then
            if [ "$(uname)" = "Darwin" ]; then
                # macOS uses .bash_profile for login shells
                shell_config="$HOME/.bash_profile"
                [ ! -f "$shell_config" ] && shell_config="$HOME/.bashrc"
            else
                shell_config="$HOME/.bashrc"
            fi
        fi

        # Offer to auto-add to PATH
        if [ -n "$shell_config" ]; then
            read -p "Add $INSTALL_DIR to PATH in $(basename "$shell_config")? (y/n) " -n 1 -r
            echo
            if [[ $REPLY =~ ^[Yy]$ ]]; then
                if ! grep -q "$INSTALL_DIR" "$shell_config" 2>/dev/null; then
                    echo "" >> "$shell_config"
                    echo "# BazBOM" >> "$shell_config"
                    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$shell_config"
                    success "Added to $shell_config"
                    note "Restart your shell or run: source $shell_config"

                    # Update PATH for current session
                    export PATH="$INSTALL_DIR:$PATH"
                else
                    note "$INSTALL_DIR already in $shell_config"
                fi
            else
                note "Skipping PATH configuration"
                warn "You may need to add $INSTALL_DIR to your PATH manually:"
                echo ""
                echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
                echo ""
            fi
        else
            warn "You may need to add $INSTALL_DIR to your PATH:"
            echo ""
            echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
            echo ""
            warn "Add this line to your shell profile (~/.bashrc, ~/.zshrc, etc.)"
        fi

        # Try again after PATH update
        if ! command -v bazbom >/dev/null 2>&1; then
            return 1
        fi
    fi

    local installed_version=$(bazbom --version 2>&1 | head -n1 || echo "unknown")
    success "BazBOM is ready to use!"
    echo ""
    echo "    Installed version: $installed_version"
    echo "    Location: $(which bazbom)"
    echo ""
}

# Offer to run interactive setup
offer_interactive_setup() {
    echo ""
    info "BazBOM includes an interactive setup wizard"
    echo ""
    read -p "Run setup wizard now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        info "Starting interactive setup wizard..."
        echo ""

        # Detect if we're in a project directory
        local setup_path="."
        if [ -f "pom.xml" ] || [ -f "build.gradle" ] || [ -f "package.json" ] || [ -f "Cargo.toml" ] || [ -f "go.mod" ]; then
            note "Detected project in current directory, initializing here"
        else
            read -p "Enter project path (or press Enter to skip): " setup_path
            if [ -z "$setup_path" ]; then
                note "Skipping setup wizard"
                return 0
            fi
        fi

        if command -v bazbom >/dev/null 2>&1; then
            bazbom init "$setup_path" || warn "Setup wizard failed - you can run 'bazbom init .' later"
        else
            warn "bazbom not in PATH yet - restart your shell first, then run: bazbom init ."
        fi
    else
        note "Skipping setup wizard - you can run 'bazbom init .' later"
    fi
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

        # Post-installation enhancements
        setup_shell_completions
        offer_interactive_setup

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
