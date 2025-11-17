# Reproducible Builds for BazBOM

**Status**: 🚧 In Implementation (Target: SLSA v1.1 Level 4)
**Last Updated**: 2025-11-16 (Rust v1.91.1 update)
**Owner**: Security Team

## Overview

BazBOM implements reproducible builds to achieve SLSA v1.1 Level 4 compliance. Reproducible builds ensure that building the same source code with the same inputs produces bit-for-bit identical binaries, enabling independent verification of build artifacts.

## Benefits

- ✅ **Supply Chain Security**: Verify binaries match published source
- ✅ **Tamper Detection**: Detect unauthorized modifications
- ✅ **Independent Verification**: Anyone can verify official builds
- ✅ **SLSA Level 4 Compliance**: Highest supply chain security standard

## Requirements for Reproducibility

### 1. Pinned Dependencies

- **Cargo.lock**: Committed to repository (✅ Done)
- **rust-toolchain.toml**: Exact Rust version specified (✅ Done - v1.91.1)
- **Build tools**: All tools pinned to specific versions in CI

### 2. Deterministic Build Environment

```bash
# Required environment variables
export SOURCE_DATE_EPOCH=1700000000  # Fixed timestamp
export RUSTFLAGS="-C metadata=bazbom-v7.0.0"  # Deterministic metadata
export CARGO_INCREMENTAL=0  # Disable incremental compilation
```

### 3. Build Configuration

All settings configured in `.cargo/config.toml`:

- `incremental = false` - No incremental compilation
- `codegen-units = 1` - Single codegen unit (deterministic)
- `strip = true` - Deterministic debug info removal
- `lto = true` - Link-time optimization

### 4. Toolchain Pinning

**rust-toolchain.toml**:
```toml
[toolchain]
channel = "1.91.1"
components = ["rustfmt", "clippy", "llvm-tools-preview"]
profile = "minimal"
```

## How to Build Reproducibly

### Local Reproducible Build

```bash
# Set reproducible build environment
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
export RUSTFLAGS="-C metadata=bazbom-$(git rev-parse --short HEAD)"
export CARGO_INCREMENTAL=0

# Clean previous builds
cargo clean

# Build in release mode
cargo build --release --locked

# Compute checksum
sha256sum target/release/bazbom
```

### CI Reproducible Build

Our GitHub Actions workflow automatically:

1. Pins Rust toolchain via `rust-toolchain.toml`
2. Sets `SOURCE_DATE_EPOCH` to commit timestamp
3. Uses `--locked` to respect Cargo.lock
4. Builds with `.cargo/config.toml` settings
5. Generates and publishes SHA-256 checksums

## Verifying Reproducibility

### Method 1: Local Verification

```bash
# Build twice and compare
cargo clean && cargo build --release --locked
sha256sum target/release/bazbom > checksum1.txt

cargo clean && cargo build --release --locked
sha256sum target/release/bazbom > checksum2.txt

# Should be identical
diff checksum1.txt checksum2.txt
```

### Method 2: Compare with CI Build

```bash
# Download official release binary
curl -sSfL https://github.com/cboyd0319/BazBOM/releases/download/v7.0.0/bazbom-linux-amd64 -o bazbom-official

# Build locally from same commit
git checkout v7.0.0
export SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)
cargo build --release --locked

# Compare checksums
sha256sum bazbom-official target/release/bazbom
```

### Method 3: Independent Build Service

Use [Reproducible Builds](https://reproducible-builds.org/) verification service:

```bash
# Submit build for independent verification
curl -X POST https://reproducible-builds.org/api/verify \
  -d "repo=https://github.com/cboyd0319/BazBOM" \
  -d "ref=v7.0.0" \
  -d "platform=linux-amd64"
```

## Known Non-Determinism Sources

### Currently Mitigated

- ✅ **Timestamps**: Using `SOURCE_DATE_EPOCH`
- ✅ **Metadata hashes**: Using `RUSTFLAGS=-C metadata=...`
- ✅ **Incremental compilation**: Disabled via config
- ✅ **Parallel codegen**: Single codegen unit
- ✅ **Toolchain version**: Pinned in rust-toolchain.toml

### Remaining Challenges

- ⚠️ **Platform differences**: Linux vs macOS vs Windows binaries differ
- ⚠️ **CPU arch**: x86_64 vs ARM64 produce different binaries
- ⚠️ **LLVM version**: Different Rust versions use different LLVM versions

### Solutions

**Platform-specific builds**: Build on each platform separately
**Arch-specific builds**: Build for each architecture separately
**Exact toolchain match**: Use same Rust version as CI (1.91.1)

## Reproducibility Status by Target

| Target | Reproducibility | Status |
|--------|-----------------|--------|
| **linux-x86_64** | 🟢 Reproducible | ✅ Verified |
| **linux-aarch64** | 🟢 Reproducible | ✅ Verified |
| **darwin-x86_64** | 🟡 Mostly | 🚧 Testing |
| **darwin-aarch64** | 🟡 Mostly | 🚧 Testing |
| **windows-x86_64** | 🔴 Partial | 🚧 In Progress |

## Troubleshooting

### Different checksums on rebuilds

**Problem**: Same source, different checksums

**Solutions**:
1. Verify `SOURCE_DATE_EPOCH` is set correctly
2. Check `RUSTFLAGS` is consistent
3. Ensure `cargo clean` was run before build
4. Verify same Rust toolchain version (run `rustc --version`)

### CI build differs from local

**Problem**: Local build doesn't match CI

**Solutions**:
1. Use exact same Rust version (1.91.1)
2. Set `SOURCE_DATE_EPOCH` to commit timestamp
3. Build on same OS (Ubuntu 22.04 for linux-x86_64)
4. Check for local modifications (`git status`)

## SLSA v1.1 Level 4 Requirements

### ✅ Implemented

- **Hermetic builds**: All inputs pinned (toolchain, dependencies)
- **Reproducible builds**: Bit-for-bit identical with same inputs
- **Provenance**: SLSA provenance generated by GitHub Actions
- **Two-party review**: Branch protection requires 2+ reviewers

### 🚧 In Progress

- **Build verification**: Automated reproducibility checks in CI
- **Independent rebuilds**: Third-party verification service
- **Build environment documentation**: Complete build env specification

### 📋 Planned

- **Hermetic build container**: Fully isolated build environment
- **Build attestation**: Signed reproducibility attestations
- **Transparency log**: Publish all builds to Rekor

## References

- [Reproducible Builds Project](https://reproducible-builds.org/)
- [SLSA Framework](https://slsa.dev/)
- [Rust Reproducible Builds](https://github.com/rust-lang/rust/issues/34902)
- [SOURCE_DATE_EPOCH Specification](https://reproducible-builds.org/docs/source-date-epoch/)

## Maintenance

### Updating Rust Version

When updating Rust:

1. Update `rust-toolchain.toml` with new version
2. Rebuild locally and verify reproducibility
3. Update CI workflow if needed
4. Test on all platforms
5. Document any new non-determinism sources
6. Update this guide with findings

### Monthly Verification

- Run reproducibility tests on main branch
- Compare CI builds with local rebuilds
- Document any deviations
- Update mitigations as needed

---

**Next Steps**:
1. Implement automated reproducibility verification in CI
2. Set up third-party independent rebuild service
3. Add reproducibility badges to README
4. Document platform-specific build environments
