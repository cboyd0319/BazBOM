# Production Dockerfile for BazBOM
# Multi-stage build for minimal, secure container images
#
# Features:
# - Multi-stage build (reduces image size by ~90%)
# - Non-root user (enhanced security)
# - Distroless base (minimal attack surface)
# - Layer caching optimization
# - Security scanning ready

# ============================================================================
# Build Stage
# ============================================================================
FROM rust:1.91.1-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /build

# Copy dependency manifests (for layer caching)
COPY Cargo.toml Cargo.lock ./
COPY crates/*/Cargo.toml ./crates/

# Create dummy source files to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    mkdir -p crates/bazbom/src && \
    echo "fn main() {}" > crates/bazbom/src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && \
    rm -rf src target/release/bazbom* target/release/deps/bazbom*

# Copy actual source code
COPY . .

# Build the actual application
RUN cargo build --release --locked --bin bazbom

# Strip debug symbols to reduce binary size
RUN strip /build/target/release/bazbom

# ============================================================================
# Runtime Stage (Distroless)
# ============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

# Labels for metadata
LABEL org.opencontainers.image.title="BazBOM" \
      org.opencontainers.image.description="SBOM and Software Composition Analysis tool for Bazel monorepos" \
      org.opencontainers.image.url="https://github.com/cboyd0319/BazBOM" \
      org.opencontainers.image.source="https://github.com/cboyd0319/BazBOM" \
      org.opencontainers.image.version="6.5.0" \
      org.opencontainers.image.vendor="BazBOM" \
      org.opencontainers.image.licenses="MIT"

# Copy binary from builder
COPY --from=builder /build/target/release/bazbom /usr/local/bin/bazbom

# Set working directory
WORKDIR /workspace

# Run as non-root user (distroless nonroot user: 65532)
USER 65532:65532

# Entrypoint
ENTRYPOINT ["/usr/local/bin/bazbom"]

# Default command
CMD ["--help"]

# Health check (for orchestrators like Kubernetes)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/bazbom", "--version"]
