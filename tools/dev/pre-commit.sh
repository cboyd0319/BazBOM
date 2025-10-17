#!/bin/bash
# Pre-commit hook for BazBOM
# Run linting and formatting checks before commits

set -e

echo "Running pre-commit checks..."

# Check if bazel is installed
if ! command -v bazel &> /dev/null; then
    echo "Error: Bazel is not installed"
    exit 1
fi

# Format check
echo "Checking code formatting..."
# TODO: Add buildifier for Bazel files
# buildifier -mode=check BUILD.bazel WORKSPACE

# Markdown lint
echo "Linting markdown files..."
if command -v markdownlint &> /dev/null; then
    markdownlint docs/**/*.md *.md
else
    echo "Warning: markdownlint not installed, skipping markdown lint"
fi

# Build check
echo "Building project..."
bazel build //... || {
    echo "Build failed. Please fix build errors before committing."
    exit 1
}

# Run tests
echo "Running tests..."
bazel test //... || {
    echo "Tests failed. Please fix test failures before committing."
    exit 1
}

echo "Pre-commit checks passed!"
