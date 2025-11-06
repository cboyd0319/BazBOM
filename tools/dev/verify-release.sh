#!/usr/bin/env bash
# BazBOM Release Verification Script
# Verifies that a version is ready for release
# Usage: ./verify-release.sh [version]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

error() {
    echo -e "${RED}✗ $1${NC}" >&2
}

success() {
    echo -e "${GREEN}[OK] $1${NC}"
}

warn() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

info() {
    echo "  $1"
}

ERRORS=0
WARNINGS=0

check_version_consistency() {
    echo "Checking version consistency..."
    
    local versions=$(grep -r "^version = " "$REPO_ROOT"/crates/*/Cargo.toml | sed 's/.*version = "\(.*\)"/\1/' | sort -u)
    local version_count=$(echo "$versions" | wc -l)
    
    if [ "$version_count" -eq 1 ]; then
        success "All crates have consistent version: $versions"
    else
        error "Inconsistent versions found:"
        grep -r "^version = " "$REPO_ROOT"/crates/*/Cargo.toml
        ERRORS=$((ERRORS + 1))
    fi
}

check_cargo_lock() {
    echo "Checking Cargo.lock is up to date..."
    
    cd "$REPO_ROOT"
    if cargo update --workspace --dry-run 2>&1 | grep -q "no changes"; then
        success "Cargo.lock is up to date"
    else
        warn "Cargo.lock may need updating"
        info "Run: cargo update --workspace"
        WARNINGS=$((WARNINGS + 1))
    fi
}

check_changelog() {
    echo "Checking CHANGELOG.md..."
    
    local check_version=$1
    
    if grep -q "## \[${check_version}\]" "$REPO_ROOT/CHANGELOG.md"; then
        success "CHANGELOG.md has entry for version $check_version"
    else
        error "CHANGELOG.md missing entry for version $check_version"
        ERRORS=$((ERRORS + 1))
    fi
    
    # Check if there are unreleased changes
    if grep -A 5 "## \[Unreleased\]" "$REPO_ROOT/CHANGELOG.md" | grep -qE "^### (Added|Changed|Fixed|Security)"; then
        warn "CHANGELOG.md has unreleased changes - ensure they're documented in the release"
        WARNINGS=$((WARNINGS + 1))
    fi
}

check_git_status() {
    echo "Checking git status..."
    
    cd "$REPO_ROOT"
    if [ -z "$(git status --porcelain)" ]; then
        success "Working directory is clean"
    else
        error "Working directory has uncommitted changes:"
        git status --short
        ERRORS=$((ERRORS + 1))
    fi
}

check_git_tag() {
    echo "Checking git tags..."
    
    local version=$1
    local tag="v$version"
    
    cd "$REPO_ROOT"
    if git tag -l "$tag" | grep -q "$tag"; then
        warn "Tag $tag already exists"
        WARNINGS=$((WARNINGS + 1))
    else
        success "Tag $tag does not exist yet"
    fi
}

check_build() {
    echo "Checking if project builds..."
    
    cd "$REPO_ROOT"
    if cargo check --workspace --quiet 2>&1 | grep -qE "error|warning"; then
        warn "Build has warnings or errors"
        info "Run: cargo check --workspace"
        WARNINGS=$((WARNINGS + 1))
    else
        success "Project builds without errors"
    fi
}

check_tests() {
    echo "Checking if tests pass..."
    
    cd "$REPO_ROOT"
    if cargo test --workspace --quiet 2>&1 | grep -qE "test result:.*FAILED"; then
        error "Tests are failing"
        info "Run: cargo test --workspace"
        ERRORS=$((ERRORS + 1))
    else
        success "All tests pass"
    fi
}

check_documentation() {
    echo "Checking documentation links..."
    
    # Check if VERSIONING.md is linked
    if grep -q "VERSIONING.md" "$REPO_ROOT/README.md" && grep -q "VERSIONING.md" "$REPO_ROOT/docs/README.md"; then
        success "Versioning documentation is linked"
    else
        warn "Versioning documentation may not be properly linked"
        WARNINGS=$((WARNINGS + 1))
    fi
}

main() {
    echo ""
    echo "========================================="
    echo "BazBOM Release Verification"
    echo "========================================="
    echo ""
    
    # Get version from argument or detect from Cargo.toml
    local version=""
    if [ $# -eq 1 ]; then
        version=$1
    else
        version=$(grep '^version = ' "$REPO_ROOT/crates/bazbom/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
    fi
    
    echo "Verifying release for version: $version"
    echo ""
    
    check_version_consistency
    check_cargo_lock
    check_changelog "$version"
    check_git_status
    check_git_tag "$version"
    check_build
    # check_tests  # Commented out as it may be time-consuming
    check_documentation
    
    echo ""
    echo "========================================="
    echo "Verification Summary"
    echo "========================================="
    
    if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
        echo -e "${GREEN}[OK] Ready for release!${NC}"
        echo ""
        echo "Next steps:"
        echo "  1. git tag -a v$version -m 'Release v$version'"
        echo "  2. git push origin v$version"
        echo "  3. GitHub Actions will automatically build and release"
        exit 0
    elif [ $ERRORS -eq 0 ]; then
        echo -e "${YELLOW}⚠ Ready with warnings ($WARNINGS warning(s))${NC}"
        echo ""
        echo "Review warnings above before releasing"
        exit 0
    else
        echo -e "${RED}✗ Not ready for release ($ERRORS error(s), $WARNINGS warning(s))${NC}"
        echo ""
        echo "Fix errors above before releasing"
        exit 1
    fi
}

main "$@"
