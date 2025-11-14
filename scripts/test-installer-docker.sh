#!/usr/bin/env bash
# Test installer in Docker containers (multi-platform)
# Usage: ./scripts/test-installer-docker.sh [platform]
#
# Platforms: ubuntu, debian, alpine, amazonlinux, fedora, all
# Example: ./scripts/test-installer-docker.sh ubuntu

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }
error() { echo -e "${RED}✗${NC} $1"; exit 1; }

PLATFORM="${1:-ubuntu}"

# Define test platforms
declare -A PLATFORMS=(
    ["ubuntu"]="ubuntu:22.04"
    ["debian"]="debian:bookworm-slim"
    ["alpine"]="alpine:latest"
    ["amazonlinux"]="amazonlinux:2023"
    ["fedora"]="fedora:latest"
)

# Check Docker
if ! command -v docker &> /dev/null; then
    error "Docker is not installed. Install from: https://docs.docker.com/get-docker/"
fi

# Build binary if not exists
if [ ! -f target/release/bazbom ]; then
    info "Building binary..."
    cargo build --release -p bazbom
fi

# Test function
test_platform() {
    local platform_name=$1
    local image=$2

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    info "Testing on $platform_name ($image)"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Create test script
    cat > /tmp/docker-test-$platform_name.sh << 'DOCKEREOF'
#!/bin/sh
set -e

echo "Installing dependencies..."
if command -v apt-get >/dev/null 2>&1; then
    apt-get update -qq
    apt-get install -y -qq curl tar > /dev/null 2>&1
elif command -v yum >/dev/null 2>&1; then
    yum install -y -q curl tar 2>&1 | grep -v "^$"
elif command -v apk >/dev/null 2>&1; then
    apk add --no-cache curl tar bash > /dev/null 2>&1
fi

echo "Testing binary directly..."
/bazbom/bazbom --version
/bazbom/bazbom --help > /dev/null

echo "Testing packaged binary..."
cd /tmp
tar -xzf /bazbom/dist/*.tar.gz
./bazbom --version
./bazbom --help > /dev/null

echo "✓ All tests passed on $1"
DOCKEREOF

    chmod +x /tmp/docker-test-$platform_name.sh

    # Run Docker container
    if docker run --rm \
        -v "$(pwd)/target/release:/bazbom:ro" \
        -v "$(pwd)/dist:/bazbom/dist:ro" \
        -v "/tmp/docker-test-$platform_name.sh:/test.sh:ro" \
        "$image" \
        /test.sh "$platform_name" 2>&1 | grep -v "debconf"; then
        success "$platform_name: All tests passed"
        return 0
    else
        warn "$platform_name: Tests failed"
        return 1
    fi
}

# Package binary first
if [ ! -d dist ] || [ -z "$(ls -A dist 2>/dev/null)" ]; then
    info "Packaging binary..."
    ./scripts/package-local-build.sh
fi

# Test single platform or all
if [ "$PLATFORM" = "all" ]; then
    echo ""
    echo "╔════════════════════════════════════════════════╗"
    echo "║     Testing BazBOM on Multiple Platforms      ║"
    echo "╚════════════════════════════════════════════════╝"

    SUCCESS_COUNT=0
    FAIL_COUNT=0

    for platform_name in "${!PLATFORMS[@]}"; do
        if test_platform "$platform_name" "${PLATFORMS[$platform_name]}"; then
            ((SUCCESS_COUNT++))
        else
            ((FAIL_COUNT++))
        fi
    done

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Summary:"
    echo "  ✓ Passed: $SUCCESS_COUNT"
    if [ $FAIL_COUNT -gt 0 ]; then
        echo "  ✗ Failed: $FAIL_COUNT"
        exit 1
    else
        success "All platforms passed!"
    fi

elif [ -n "${PLATFORMS[$PLATFORM]}" ]; then
    test_platform "$PLATFORM" "${PLATFORMS[$PLATFORM]}"
else
    error "Unknown platform: $PLATFORM. Use one of: ${!PLATFORMS[@]} all"
fi

echo ""
success "Docker testing complete!"
