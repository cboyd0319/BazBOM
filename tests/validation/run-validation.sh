#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BAZBOM="${BAZBOM_BIN:-$SCRIPT_DIR/../../target/release/bazbom}"
RESULTS_DIR="$SCRIPT_DIR/results"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "=== BazBOM Validation Suite ==="
echo "Binary: $BAZBOM"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to run validation
validate() {
    local name="$1"
    local dir="$SCRIPT_DIR/$2"
    local extra_flags="$3"

    echo -e "${YELLOW}Testing: $name${NC}"

    if [ ! -d "$dir" ]; then
        echo -e "${RED}  Directory not found: $dir${NC}"
        return 1
    fi

    # Run fast scan first
    echo "  Running fast scan..."
    if $BAZBOM scan "$dir" --fast --json > "$RESULTS_DIR/${name}-fast.json" 2>&1; then
        echo -e "${GREEN}  ✓ Fast scan passed${NC}"
    else
        echo -e "${RED}  ✗ Fast scan failed${NC}"
        cat "$RESULTS_DIR/${name}-fast.json"
        return 1
    fi

    # Run with reachability
    echo "  Running reachability scan..."
    if $BAZBOM scan "$dir" -r --json $extra_flags > "$RESULTS_DIR/${name}-reachability.json" 2>&1; then
        echo -e "${GREEN}  ✓ Reachability scan passed${NC}"
    else
        echo -e "${YELLOW}  ⚠ Reachability scan had issues${NC}"
    fi

    # Check for expected vulnerabilities
    echo "  Checking results..."
    local vuln_count=$(grep -c '"severity"' "$RESULTS_DIR/${name}-fast.json" 2>/dev/null || echo "0")
    echo "    Vulnerabilities found: $vuln_count"

    echo ""
}

# Run specific test or all
if [ -n "$1" ]; then
    case "$1" in
        java-maven)
            validate "java-maven" "java-maven"
            ;;
        java-gradle)
            validate "java-gradle" "java-gradle"
            ;;
        bazel)
            validate "bazel" "bazel"
            ;;
        typescript)
            validate "typescript" "typescript"
            ;;
        python)
            validate "python" "python"
            ;;
        go)
            validate "go" "go"
            ;;
        rust)
            validate "rust" "rust"
            ;;
        ruby)
            validate "ruby" "ruby"
            ;;
        php)
            validate "php" "php"
            ;;
        *)
            echo "Unknown test: $1"
            exit 1
            ;;
    esac
else
    # Run all available tests
    for dir in "$SCRIPT_DIR"/*/; do
        name=$(basename "$dir")
        if [ -f "$dir/pom.xml" ] || [ -f "$dir/package.json" ] || [ -f "$dir/requirements.txt" ] || \
           [ -f "$dir/go.mod" ] || [ -f "$dir/Cargo.toml" ] || [ -f "$dir/Gemfile" ] || \
           [ -f "$dir/composer.json" ] || [ -f "$dir/BUILD" ]; then
            validate "$name" "$name"
        fi
    done
fi

echo "=== Validation Complete ==="
echo "Results saved to: $RESULTS_DIR"
