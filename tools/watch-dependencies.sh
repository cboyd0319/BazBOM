#!/bin/bash
# Watch mode for BazBOM - Continuously monitor file changes and re-run SBOM generation
#
# Uses RipGrep to efficiently find BUILD files and 'entr' to watch for changes

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
WORKSPACE_ROOT="${WORKSPACE_ROOT:-.}"
SCAN_COMMAND="${SCAN_COMMAND:-bazbom scan --incremental}"

# Check for required tools
check_dependencies() {
    local missing_tools=()
    
    if ! command -v rg &> /dev/null; then
        missing_tools+=("ripgrep")
    fi
    
    if ! command -v entr &> /dev/null; then
        missing_tools+=("entr")
    fi
    
    if [ ${#missing_tools[@]} -gt 0 ]; then
        echo -e "${YELLOW}⚠️  Missing tools: ${missing_tools[*]}${NC}" >&2
        echo "" >&2
        echo "Installation instructions:" >&2
        for tool in "${missing_tools[@]}"; do
            case "$tool" in
                ripgrep)
                    echo "  RipGrep:" >&2
                    echo "    - Debian/Ubuntu: apt install ripgrep" >&2
                    echo "    - RHEL/CentOS: yum install ripgrep" >&2
                    echo "    - macOS: brew install ripgrep" >&2
                    echo "    - Or see: https://github.com/BurntSushi/ripgrep#installation" >&2
                    ;;
                entr)
                    echo "  entr:" >&2
                    echo "    - Debian/Ubuntu: apt install entr" >&2
                    echo "    - macOS: brew install entr" >&2
                    echo "    - Or see: https://github.com/eradman/entr" >&2
                    ;;
            esac
        done
        echo "" >&2
        return 1
    fi
    
    return 0
}

# Display usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Watch BUILD files and dependency manifests for changes and automatically
regenerate SBOMs when changes are detected.

Options:
  -h, --help              Show this help message
  -w, --workspace PATH    Workspace root path (default: current directory)
  -c, --command CMD       Command to run on change (default: bazbom scan --incremental)

Examples:
  # Watch current directory
  $0

  # Watch specific workspace
  $0 --workspace /path/to/project

  # Use custom scan command
  $0 --command "bazel build //:sbom_all"

EOF
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h|--help)
                usage
                exit 0
                ;;
            -w|--workspace)
                WORKSPACE_ROOT="$2"
                shift 2
                ;;
            -c|--command)
                SCAN_COMMAND="$2"
                shift 2
                ;;
            *)
                echo "Unknown option: $1" >&2
                usage
                exit 1
                ;;
        esac
    done
}

# Main watch loop
watch_dependencies() {
    echo -e "${BLUE}🔍 BazBOM Watch Mode${NC}"
    echo -e "Workspace: ${GREEN}$WORKSPACE_ROOT${NC}"
    echo -e "Command: ${GREEN}$SCAN_COMMAND${NC}"
    echo ""
    echo "Watching for changes in:"
    echo "  - BUILD.bazel / BUILD files"
    echo "  - maven_install.json"
    echo "  - pom.xml files"
    echo ""
    echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
    echo ""
    
    # Use RipGrep to find files to watch, pipe to entr
    rg --files \
       --glob "BUILD.bazel" \
       --glob "BUILD" \
       --glob "maven_install.json" \
       --glob "pom.xml" \
       "$WORKSPACE_ROOT" | \
    entr -c sh -c "$SCAN_COMMAND"
}

# Main entry point
main() {
    parse_args "$@"
    
    # Check for required tools
    if ! check_dependencies; then
        exit 1
    fi
    
    # Verify workspace exists
    if [ ! -d "$WORKSPACE_ROOT" ]; then
        echo -e "${YELLOW}⚠️  Workspace directory not found: $WORKSPACE_ROOT${NC}" >&2
        exit 1
    fi
    
    # Start watching
    watch_dependencies
}

# Run main
main "$@"
