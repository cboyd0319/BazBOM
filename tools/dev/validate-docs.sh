#!/bin/bash
# Validate documentation files
# Checks markdown lint and broken links

set -e

echo "Validating documentation..."

# Markdown lint
if command -v markdownlint &> /dev/null; then
    echo "Running markdownlint..."
    markdownlint --config .markdownlint.json docs/**/*.md *.md
else
    echo "Error: markdownlint not installed"
    echo "Install with: npm install -g markdownlint-cli"
    exit 1
fi

# Check for broken links (optional)
if command -v markdown-link-check &> /dev/null; then
    echo "Checking for broken links..."
    find docs -name "*.md" -exec markdown-link-check {} \;
else
    echo "Warning: markdown-link-check not installed, skipping link check"
    echo "Install with: npm install -g markdown-link-check"
fi

echo "Documentation validation passed!"
