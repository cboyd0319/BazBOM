#!/usr/bin/env bash
# BazBOM System Dependency Checker
# Usage: curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/scripts/check-system.sh | sh
# Or: ./scripts/check-system.sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
CHECKS_PASSED=0
CHECKS_FAILED=0
CHECKS_WARNED=0

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    CHECKS_WARNED=$((CHECKS_WARNED + 1))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    CHECKS_FAILED=$((CHECKS_FAILED + 1))
}

section() {
    echo ""
    echo -e "${CYAN}══════ $1 ══════${NC}"
}

# Print header
print_header() {
    echo ""
    echo "╔════════════════════════════════════════════════╗"
    echo "║      BazBOM System Dependency Checker          ║"
    echo "║  Validates your system is ready for BazBOM     ║"
    echo "╚════════════════════════════════════════════════╝"
    echo ""
}

# Check operating system
check_os() {
    section "Operating System"

    local os=$(uname -s)
    local os_version=""

    case "$os" in
        Darwin)
            os_version=$(sw_vers -productVersion 2>/dev/null || echo "unknown")
            success "macOS detected: $os_version"
            if [[ "$os_version" =~ ^([0-9]+) ]]; then
                local major_version="${BASH_REMATCH[1]}"
                if [ "$major_version" -ge 10 ]; then
                    success "macOS version is compatible"
                else
                    warn "macOS version may be too old (10.15+ recommended)"
                fi
            fi
            ;;
        Linux)
            if [ -f /etc/os-release ]; then
                os_version=$(grep "PRETTY_NAME" /etc/os-release | cut -d '"' -f 2)
            else
                os_version="unknown"
            fi
            success "Linux detected: $os_version"
            ;;
        *)
            warn "Operating system: $os (may not be fully supported)"
            ;;
    esac

    local arch=$(uname -m)
    case "$arch" in
        x86_64|amd64)
            success "Architecture: x86_64 (fully supported)"
            ;;
        aarch64|arm64)
            success "Architecture: ARM64 (fully supported)"
            ;;
        *)
            warn "Architecture: $arch (may not be supported)"
            ;;
    esac
}

# Check BazBOM installation
check_bazbom() {
    section "BazBOM Installation"

    if command -v bazbom >/dev/null 2>&1; then
        local version=$(bazbom --version 2>&1 | head -n1 || echo "unknown")
        success "BazBOM is installed: $version"
        success "Location: $(which bazbom)"

        # Test if it actually runs
        if bazbom --help >/dev/null 2>&1; then
            success "BazBOM is functional"
        else
            fail "BazBOM is installed but not working properly"
        fi
    else
        fail "BazBOM is not installed"
        info "Install with: curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh"
    fi
}

# Check core dependencies
check_core_deps() {
    section "Core Dependencies"

    # Git
    if command -v git >/dev/null 2>&1; then
        local git_version=$(git --version | awk '{print $3}')
        success "Git is installed: v$git_version"
    else
        warn "Git is not installed (needed for most workflows)"
        info "Install with: brew install git (macOS) or apt-get install git (Linux)"
    fi

    # Curl
    if command -v curl >/dev/null 2>&1; then
        success "curl is installed"
    else
        warn "curl is not installed (may affect downloads)"
    fi

    # Tar
    if command -v tar >/dev/null 2>&1; then
        success "tar is installed"
    else
        warn "tar is not installed (needed for archive extraction)"
    fi
}

# Check JVM ecosystem dependencies
check_jvm_deps() {
    section "JVM Dependencies (for Java/Kotlin/Scala projects)"

    # Java
    if command -v java >/dev/null 2>&1; then
        local java_version=$(java -version 2>&1 | head -n1 | awk -F '"' '{print $2}')
        success "Java is installed: $java_version"

        # Check Java version
        if [[ "$java_version" =~ ^([0-9]+) ]]; then
            local major_version="${BASH_REMATCH[1]}"
            if [ "$major_version" -ge 11 ]; then
                success "Java version is compatible (11+)"
            else
                warn "Java version is old (11+ recommended for reachability analysis)"
            fi
        fi

        # Check JAVA_HOME
        if [ -n "$JAVA_HOME" ]; then
            success "JAVA_HOME is set: $JAVA_HOME"
        else
            warn "JAVA_HOME is not set (may be needed for some features)"
        fi
    else
        warn "Java is not installed"
        info "Required for: JVM reachability analysis (Java, Kotlin, Scala, Groovy, Clojure)"
        info "Install with: brew install openjdk@21 (macOS) or apt-get install openjdk-21-jdk (Linux)"
    fi

    # Maven
    if command -v mvn >/dev/null 2>&1; then
        local mvn_version=$(mvn -version 2>&1 | head -n1 | awk '{print $3}')
        success "Maven is installed: v$mvn_version"
    else
        info "Maven not found (only needed if you use Maven)"
    fi

    # Gradle
    if command -v gradle >/dev/null 2>&1; then
        local gradle_version=$(gradle --version 2>&1 | grep "Gradle" | awk '{print $2}')
        success "Gradle is installed: v$gradle_version"
    else
        info "Gradle not found (only needed if you use Gradle)"
    fi

    # Bazel
    if command -v bazel >/dev/null 2>&1; then
        local bazel_version=$(bazel --version 2>&1 | awk '{print $2}')
        success "Bazel is installed: v$bazel_version"
    else
        info "Bazel not found (only needed if you use Bazel)"
    fi
}

# Check polyglot dependencies
check_polyglot_deps() {
    section "Polyglot Language Support"

    # Node.js/npm
    if command -v node >/dev/null 2>&1; then
        local node_version=$(node --version)
        success "Node.js is installed: $node_version"
    else
        info "Node.js not found (only needed for JavaScript/TypeScript projects)"
    fi

    # Python
    if command -v python3 >/dev/null 2>&1; then
        local python_version=$(python3 --version | awk '{print $2}')
        success "Python is installed: v$python_version"
    else
        info "Python not found (only needed for Python projects)"
    fi

    # Go
    if command -v go >/dev/null 2>&1; then
        local go_version=$(go version | awk '{print $3}')
        success "Go is installed: $go_version"
    else
        info "Go not found (only needed for Go projects)"
    fi

    # Rust
    if command -v rustc >/dev/null 2>&1; then
        local rust_version=$(rustc --version | awk '{print $2}')
        success "Rust is installed: v$rust_version"
    else
        info "Rust not found (only needed for Rust projects)"
    fi

    # Ruby
    if command -v ruby >/dev/null 2>&1; then
        local ruby_version=$(ruby --version | awk '{print $2}')
        success "Ruby is installed: v$ruby_version"
    else
        info "Ruby not found (only needed for Ruby projects)"
    fi

    # PHP
    if command -v php >/dev/null 2>&1; then
        local php_version=$(php --version | head -n1 | awk '{print $2}')
        success "PHP is installed: v$php_version"
    else
        info "PHP not found (only needed for PHP projects)"
    fi
}

# Check PATH configuration
check_path() {
    section "PATH Configuration"

    if echo "$PATH" | grep -q "/usr/local/bin"; then
        success "/usr/local/bin is in PATH"
    else
        warn "/usr/local/bin is not in PATH (may prevent bazbom from being found)"
        info "Add to ~/.zshrc or ~/.bashrc: export PATH=\"/usr/local/bin:\$PATH\""
    fi

    # Check for ~/.cargo/bin in PATH (expand $HOME properly)
    local cargo_bin_expanded="${HOME}/.cargo/bin"
    if echo "$PATH" | grep -q "$cargo_bin_expanded"; then
        success "~/.cargo/bin is in PATH (for Rust tools)"
    else
        info "~/.cargo/bin is not in PATH (only needed for Rust development)"
    fi
}

# Check disk space
check_disk_space() {
    section "System Resources"

    local os=$(uname -s)
    local available_space

    if [ "$os" = "Darwin" ]; then
        available_space=$(df -h / | tail -n1 | awk '{print $4}')
        success "Available disk space: $available_space"
    elif [ "$os" = "Linux" ]; then
        available_space=$(df -h / | tail -n1 | awk '{print $4}')
        success "Available disk space: $available_space"
    fi
}

# Run a test scan (if BazBOM is installed)
run_test_scan() {
    section "Test Scan"

    if ! command -v bazbom >/dev/null 2>&1; then
        warn "Skipping test scan (BazBOM not installed)"
        return
    fi

    # Create a minimal test project
    local test_dir=$(mktemp -d)
    local original_dir=$(pwd)
    cd "$test_dir"

    # Create a minimal pom.xml
    cat > pom.xml <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0"
         xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
         xsi:schemaLocation="http://maven.apache.org/POM/4.0.0
         http://maven.apache.org/xsd/maven-4.0.0.xsd">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test</artifactId>
    <version>1.0.0</version>
</project>
EOF

    info "Running test scan on minimal Maven project..."
    if bazbom check 2>/dev/null; then
        success "Test scan completed successfully"
    else
        warn "Test scan failed (this may be expected for a minimal project)"
    fi

    # Cleanup
    cd "$original_dir"
    rm -rf "$test_dir"
}

# Print summary
print_summary() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo -e "${CYAN}Summary:${NC}"
    echo -e "  ${GREEN}✓ Passed:${NC} $CHECKS_PASSED"
    echo -e "  ${YELLOW}⚠ Warnings:${NC} $CHECKS_WARNED"
    echo -e "  ${RED}✗ Failed:${NC} $CHECKS_FAILED"
    echo ""

    if [ $CHECKS_FAILED -eq 0 ]; then
        if [ $CHECKS_WARNED -eq 0 ]; then
            echo -e "${GREEN}✓ Your system is fully ready for BazBOM!${NC}"
        else
            echo -e "${YELLOW}⚠ Your system is mostly ready for BazBOM.${NC}"
            echo -e "  Review warnings above for optional improvements."
        fi
    else
        echo -e "${RED}✗ Your system has issues that need to be resolved.${NC}"
        echo -e "  Review failed checks above and install missing dependencies."
    fi
    echo ""
}

# Main function
main() {
    print_header
    check_os
    check_bazbom
    check_core_deps
    check_jvm_deps
    check_polyglot_deps
    check_path
    check_disk_space
    run_test_scan
    print_summary
}

# Run main
main
