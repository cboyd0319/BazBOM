# Release Process

This document describes the release workflow for BazBOM, including binary generation, signing, and distribution.

## Overview

BazBOM follows a fully automated release process that produces signed, single-binary artifacts with SLSA provenance. The release workflow is triggered by pushing a version tag and handles building, signing, and publishing binaries for multiple platforms.

## Supported Platforms

The release process builds binaries for the following platforms:

- **macOS**
  - x86_64 (Intel)
  - aarch64 (Apple Silicon)
- **Linux**
  - x86_64
  - aarch64

## Release Workflow

### Step 1: Prepare Release

1. Ensure all tests pass:

   ```bash
   cargo test --all
   ```

2. Update version in `Cargo.toml` files if needed.

3. Update `CHANGELOG.md` with release notes.

4. Commit changes:

   ```bash
   git add .
   git commit -m "chore: prepare release v0.1.0"
   git push
   ```

### Step 2: Create Release Tag

Create and push a version tag to trigger the release workflow:

```bash
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The tag must follow the pattern `v*` (e.g., `v0.1.0`, `v1.2.3`).

### Step 3: Automated Build Process

The release workflow (`.github/workflows/release.yml`) automatically:

1. **Builds binaries** for all supported platforms using Rust stable toolchain
2. **Packages artifacts** as `.tar.gz` archives
3. **Signs binaries** using Sigstore cosign (keyless signing)
4. **Generates checksums** (SHA256) for all artifacts
5. **Creates GitHub Release** with all artifacts attached

### Step 4: Verify Release

After the workflow completes:

1. Navigate to the [Releases page](https://github.com/cboyd0319/BazBOM/releases)
2. Verify the release includes:
   - Binary archives for all platforms (`.tar.gz`)
   - Signature files (`.tar.gz.sig`)
   - Checksum files (`.tar.gz.sha256`)

## Binary Verification

Users can verify downloaded binaries using cosign and checksums.

### Verify with Cosign

Install cosign:

```bash
# macOS
brew install cosign

# Linux
wget https://github.com/sigstore/cosign/releases/latest/download/cosign-linux-amd64
chmod +x cosign-linux-amd64
sudo mv cosign-linux-amd64 /usr/local/bin/cosign
```

Verify the signature:

```bash
# Download the binary, signature, and certificate (if provided)
cosign verify-blob \
  --signature bazbom-x86_64-apple-darwin.tar.gz.sig \
  bazbom-x86_64-apple-darwin.tar.gz
```

### Verify Checksums

Download and verify the SHA256 checksum:

```bash
# Download checksum file
curl -LO https://github.com/cboyd0319/BazBOM/releases/download/v1.0.0/bazbom-x86_64-apple-darwin.tar.gz.sha256

# Verify checksum
echo "$(cat bazbom-x86_64-apple-darwin.tar.gz.sha256)  bazbom-x86_64-apple-darwin.tar.gz" | shasum -a 256 --check
```

Expected output: `bazbom-x86_64-apple-darwin.tar.gz: OK`

## SLSA Provenance

BazBOM releases include SLSA Level 3 provenance, providing verifiable information about the build process. This ensures that binaries were built from a specific source and have not been tampered with.

For more information on SLSA provenance verification, see [provenance.md](provenance.md).

## Distribution Channels

### GitHub Releases (Primary)

All releases are published to GitHub Releases:

```
https://github.com/cboyd0319/BazBOM/releases
```

### Homebrew Tap

BazBOM is distributed via a Homebrew tap. See [../getting-started/homebrew-installation.md](../getting-started/homebrew-installation.md) for installation instructions.

### Distribution Channels

BazBOM is available through:

- Homebrew tap (macOS and Linux)
- Direct binary downloads (GitHub Releases)
- Build from source (Cargo)

## Release Checklist

Use this checklist when creating a release:

- [ ] All tests pass locally
- [ ] Version updated in `Cargo.toml` files
- [ ] `CHANGELOG.md` updated with release notes
- [ ] Changes committed and pushed to main
- [ ] Version tag created and pushed
- [ ] Release workflow completed successfully
- [ ] Binaries verified with cosign
- [ ] Checksums verified
- [ ] Release notes published
- [ ] Homebrew formula updated (if applicable)
- [ ] Documentation updated (if needed)

## Troubleshooting

### Release Workflow Fails

If the release workflow fails:

1. Check the workflow run logs in GitHub Actions
2. Verify all required secrets are configured (if any)
3. Ensure the tag follows the correct pattern (`v*`)
4. Verify Rust toolchain compatibility

### Signature Verification Fails

If cosign verification fails:

1. Ensure you have the latest version of cosign
2. Verify you downloaded the correct signature file
3. Check that the binary and signature match the same release
4. Contact maintainers if issues persist

## Security Considerations

- **Keyless Signing**: BazBOM uses Sigstore's keyless signing, which relies on OIDC identity tokens from GitHub Actions. This eliminates the need for long-lived signing keys.
- **Provenance**: SLSA provenance provides verifiable supply chain metadata.
- **No Telemetry**: Release binaries contain zero telemetry and make no network calls except when explicitly running `bazbom db sync`.
- **Offline Operation**: Binaries can operate completely offline after advisory data is synced.

## References

- [Sigstore](https://www.sigstore.dev/) - Keyless signing infrastructure
- [SLSA](https://slsa.dev/) - Supply chain security framework
- [GitHub Actions](https://docs.github.com/en/actions) - CI/CD platform
- [Cosign](https://github.com/sigstore/cosign) - Container and binary signing tool
