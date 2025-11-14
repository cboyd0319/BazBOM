#!/usr/bin/env bash
# Trigger the installer build workflow via GitHub CLI
# Usage: ./scripts/trigger-installer-build.sh [version]

set -euo pipefail

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${BLUE}ℹ${NC} $1"; }
success() { echo -e "${GREEN}✓${NC} $1"; }
warn() { echo -e "${YELLOW}⚠${NC} $1"; }

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed"
    echo ""
    echo "Install it with:"
    echo "  macOS:   brew install gh"
    echo "  Linux:   See https://github.com/cli/cli/blob/trunk/docs/install_linux.md"
    echo "  Windows: See https://github.com/cli/cli/releases"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI"
    echo ""
    echo "Run: gh auth login"
    exit 1
fi

VERSION="${1:-6.5.0}"

echo ""
echo "Triggering installer build workflow..."
echo "  Version: $VERSION"
echo ""

# Trigger the workflow
if gh workflow run build-installers.yml -f version="$VERSION"; then
    success "Workflow triggered successfully!"
    echo ""

    # Wait a moment for the run to be created
    sleep 3

    info "Fetching workflow run..."
    RUN_ID=$(gh run list --workflow=build-installers.yml --limit 1 --json databaseId --jq '.[0].databaseId')

    if [ ! -z "$RUN_ID" ]; then
        success "Workflow run created: $RUN_ID"
        echo ""
        echo "Monitor progress:"
        echo "  • View in browser: gh run view $RUN_ID --web"
        echo "  • View in terminal: gh run view $RUN_ID"
        echo "  • Watch logs: gh run watch $RUN_ID"
        echo ""

        # Ask if they want to watch
        read -p "Watch the workflow progress now? [y/N] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            gh run watch $RUN_ID
            echo ""

            # Check if successful
            STATUS=$(gh run view $RUN_ID --json conclusion --jq '.conclusion')
            if [ "$STATUS" = "success" ]; then
                success "Workflow completed successfully!"
                echo ""
                echo "Download artifacts:"
                echo "  gh run download $RUN_ID"
                echo ""
                echo "Or download specific artifact:"
                echo "  gh run download $RUN_ID -n signed-dist"
            else
                warn "Workflow completed with status: $STATUS"
            fi
        else
            info "You can watch it later with: gh run watch $RUN_ID"
        fi
    fi
else
    echo "Error: Failed to trigger workflow"
    exit 1
fi
