# BazBOM Makefile - Developer Convenience Tasks
# Usage: make <target>
#
# Common targets:
#   make build       - Build release binary
#   make test        - Run all tests
#   make check       - Run all quality checks (fmt, clippy, test)
#   make install     - Install bazbom to /usr/local/bin
#   make clean       - Clean build artifacts
#   make help        - Show this help message

.PHONY: help build test check install clean fmt clippy coverage dev quick

# Default target
help:
	@echo "BazBOM Development Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build        - Build release binary (optimized)"
	@echo "  make dev          - Build debug binary (fast)"
	@echo "  make test         - Run all tests"
	@echo "  make quick        - Quick build + test (debug mode)"
	@echo "  make check        - Run all quality checks (fmt, clippy, test)"
	@echo "  make fmt          - Check code formatting"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make coverage     - Generate test coverage report"
	@echo "  make install      - Install bazbom to /usr/local/bin"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make completions  - Generate shell completions"
	@echo "  make help         - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make quick        # Fast iteration during development"
	@echo "  make check        # Pre-commit quality checks"
	@echo "  make build install # Build and install release binary"

# Build release binary (optimized, slower build)
build:
	@echo "Building release binary..."
	cargo build --release -p bazbom
	@echo "✓ Binary built: target/release/bazbom"

# Build debug binary (fast build, for development)
dev:
	@echo "Building debug binary..."
	cargo build -p bazbom
	@echo "✓ Binary built: target/debug/bazbom"

# Run all tests
test:
	@echo "Running tests..."
	cargo test --all --all-features
	@echo "✓ All tests passed"

# Quick development cycle: debug build + tests
quick: dev
	@echo "Running quick tests..."
	cargo test -p bazbom --lib
	@echo "✓ Quick check complete"

# Run all quality checks
check: fmt clippy test
	@echo "✓ All quality checks passed"

# Check code formatting
fmt:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check
	@echo "✓ Code formatting is correct"

# Run clippy linter
clippy:
	@echo "Running clippy..."
	cargo clippy --all --all-targets --all-features -- -D warnings
	@echo "✓ Clippy passed with no warnings"

# Generate test coverage report
coverage:
	@echo "Generating coverage report..."
	cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
	@echo "✓ Coverage report generated: lcov.info"
	@echo ""
	@echo "Coverage summary:"
	@cargo llvm-cov --all-features --workspace --summary-only

# Install bazbom to /usr/local/bin
install: build
	@echo "Installing bazbom..."
	sudo cp target/release/bazbom /usr/local/bin/
	@echo "✓ Installed to /usr/local/bin/bazbom"
	@bazbom --version

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "✓ Build artifacts cleaned"

# Generate shell completions
completions: build
	@echo "Generating shell completions..."
	@mkdir -p completions
	@# Note: Completions generation requires adding clap_complete to dependencies
	@echo "Note: Shell completion generation requires implementing a 'completions' subcommand"
	@echo "See: https://docs.rs/clap_complete/latest/clap_complete/"

# Development workflow helpers
.PHONY: watch run scan-example

# Watch for changes and run tests (requires cargo-watch)
watch:
	@command -v cargo-watch >/dev/null 2>&1 || { echo "Installing cargo-watch..."; cargo install cargo-watch; }
	cargo watch -x 'test --all' -c

# Run bazbom on itself
scan-example: build
	@echo "Scanning BazBOM repository..."
	./target/release/bazbom scan . --format spdx
	@echo "✓ Self-scan complete"

# Run bazbom with development binary
run: dev
	./target/debug/bazbom $(ARGS)
