#!/usr/bin/env bash
# BazBOM Version Management Script
# Updates version across all Cargo.toml files and CHANGELOG.md
# Usage: ./bump-version.sh <new-version>
# Example: ./bump-version.sh 0.2.1

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

info() {
    echo -e "${GREEN}$1${NC}"
}

warn() {
    echo -e "${YELLOW}$1${NC}"
}

# Validate version format (semantic versioning)
validate_version() {
    local version=$1
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
        error "Invalid version format: $version. Expected: X.Y.Z or X.Y.Z-suffix"
    fi
}

# Get current version from main bazbom crate
get_current_version() {
    grep '^version = ' "$REPO_ROOT/crates/bazbom/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Update version in a Cargo.toml file
update_cargo_toml() {
    local file=$1
    local new_version=$2
    
    # Use sed to replace the first occurrence of version = "..."
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    else
        sed -i "s/^version = \".*\"/version = \"$new_version\"/" "$file"
    fi
    
    info "Updated $file"
}

# Update CHANGELOG.md
update_changelog() {
    local version=$1
    local date=$(date +%Y-%m-%d)
    local changelog="$REPO_ROOT/CHANGELOG.md"
    
    if ! grep -q "## \[Unreleased\]" "$changelog"; then
        warn "No [Unreleased] section found in CHANGELOG.md"
        return
    fi
    
    # Replace [Unreleased] with the new version and date
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/## \[Unreleased\]/## [Unreleased]\n\n## [$version] - $date/" "$changelog"
    else
        sed -i "s/## \[Unreleased\]/## [Unreleased]\n\n## [$version] - $date/" "$changelog"
    fi
    
    info "Updated CHANGELOG.md with version $version"
}

# Main script
main() {
    if [ $# -ne 1 ]; then
        echo "Usage: $0 <new-version>"
        echo "Example: $0 0.2.1"
        exit 1
    fi
    
    local new_version=$1
    validate_version "$new_version"
    
    local current_version=$(get_current_version)
    info "Current version: $current_version"
    info "New version: $new_version"
    
    echo ""
    read -p "Continue with version bump? (y/N) " -n 1 -r
    echo ""
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        error "Version bump cancelled"
    fi
    
    info "Updating Cargo.toml files..."
    
    # Update all crate Cargo.toml files
    for cargo_file in "$REPO_ROOT"/crates/*/Cargo.toml; do
        update_cargo_toml "$cargo_file" "$new_version"
    done
    
    # Update changelog
    update_changelog "$new_version"
    
    # Update Cargo.lock
    info "Updating Cargo.lock..."
    cd "$REPO_ROOT"
    if ! cargo update --workspace; then
        warn "Warning: cargo update failed, but continuing..."
    fi
    
    info ""
    info "Version bump complete!"
    info "Next steps:"
    info "  1. Review changes: git diff"
    info "  2. Commit changes: git add -A && git commit -m 'Bump version to $new_version'"
    info "  3. Create tag: git tag -a v$new_version -m 'Release v$new_version'"
    info "  4. Push changes: git push origin main --tags"
}

main "$@"
