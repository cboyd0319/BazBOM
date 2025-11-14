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
.PHONY: installer-test installer-build package homebrew-formula release-check

# Default target
help:
	@echo "BazBOM Development Makefile"
	@echo ""
	@echo "Build & Test:"
	@echo "  make build        - Build release binary (optimized)"
	@echo "  make dev          - Build debug binary (fast)"
	@echo "  make test         - Run all tests"
	@echo "  make quick        - Quick build + test (debug mode)"
	@echo "  make check        - Run all quality checks (fmt, clippy, test)"
	@echo "  make fmt          - Check code formatting"
	@echo "  make clippy       - Run clippy linter"
	@echo "  make coverage     - Generate test coverage report"
	@echo ""
	@echo "Installer Testing:"
	@echo "  make installer-test       - Test installer locally (ONE COMMAND!)"
	@echo "  make installer-test-quick - Test installer (skip build)"
	@echo "  make installer-build      - Trigger GitHub Actions build"
	@echo "  make package              - Package local build"
	@echo ""
	@echo "Release:"
	@echo "  make homebrew-formula     - Generate Homebrew formula"
	@echo "  make release-check        - Verify release readiness"
	@echo ""
	@echo "Install:"
	@echo "  make install      - Install bazbom to /usr/local/bin"
	@echo "  make install-user - Install to ~/.local/bin (no sudo)"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make completions  - Generate shell completions"
	@echo ""
	@echo "Examples:"
	@echo "  make quick             # Fast iteration during development"
	@echo "  make check             # Pre-commit quality checks"
	@echo "  make installer-test    # Test the installer end-to-end"
	@echo "  make build install     # Build and install release binary"

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

# Installer testing targets
.PHONY: installer-test installer-test-quick installer-build package homebrew-formula release-check install-user

# Test installer locally (builds, packages, and tests end-to-end)
installer-test:
	@echo "Running full installer test..."
	@./scripts/test-installer-local.sh

# Test installer without rebuilding
installer-test-quick:
	@echo "Running quick installer test (skip build)..."
	@./scripts/test-installer-local.sh --skip-build

# Trigger GitHub Actions installer build
installer-build:
	@echo "Triggering GitHub Actions installer build..."
	@./scripts/trigger-installer-build.sh

# Package local build for distribution
package:
	@echo "Packaging local build..."
	@./scripts/package-local-build.sh

# Generate Homebrew formula for current version
homebrew-formula:
	@echo "Generating Homebrew formula..."
	@./scripts/generate-homebrew-formula.sh $$(cargo pkgid -p bazbom | cut -d'#' -f2)

# Check if ready for release
release-check:
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "Release Readiness Check"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "📦 Version Information:"
	@echo "  Cargo.toml:  $$(cargo pkgid -p bazbom | cut -d'#' -f2)"
	@if [ -f target/release/bazbom ]; then \
		echo "  Binary:      $$(./target/release/bazbom --version | awk '{print $$2}')"; \
	else \
		echo "  Binary:      ⚠️  NOT BUILT (run 'make build')"; \
	fi
	@echo ""
	@echo "🔍 Git Status:"
	@if [ -z "$$(git status --porcelain)" ]; then \
		echo "  ✓ Working directory clean"; \
	else \
		echo "  ⚠️  Uncommitted changes:"; \
		git status --short | head -10; \
	fi
	@echo ""
	@echo "📝 Recent Commits:"
	@git log --oneline -5
	@echo ""
	@echo "✅ Next Steps:"
	@echo "  1. Run tests:     make check"
	@echo "  2. Test installer: make installer-test"
	@echo "  3. Create tag:    git tag -a v$$(cargo pkgid -p bazbom | cut -d'#' -f2) -m 'Release v$$(cargo pkgid -p bazbom | cut -d'#' -f2)'"
	@echo "  4. Push tag:      git push origin v$$(cargo pkgid -p bazbom | cut -d'#' -f2)"
	@echo ""

# Install to user directory (no sudo required)
install-user: build
	@echo "Installing to ~/.local/bin..."
	@mkdir -p ~/.local/bin
	@install -m 755 target/release/bazbom ~/.local/bin/bazbom
	@echo "✓ Installed to ~/.local/bin/bazbom"
	@echo ""
	@echo "Make sure ~/.local/bin is in your PATH:"
	@echo "  export PATH=\"\$$HOME/.local/bin:\$$PATH\""
	@echo ""
	@~/.local/bin/bazbom --version
