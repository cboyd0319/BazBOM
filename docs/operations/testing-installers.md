# Testing Installers Before Release

This guide explains how to build and test BazBOM installers before creating an official release.

## Overview

Before creating a release with installers, you should:

1. Build binaries locally or via GitHub Actions
2. Test the install script
3. Test the Homebrew formula (if applicable)
4. Verify all packages work correctly

## Method 1: Test with GitHub Actions (Recommended)

The `build-installers.yml` workflow allows you to build all platform binaries without creating a release.

### Trigger the Workflow

1. Go to **Actions** → **Build Installers (Manual)** in the GitHub UI
2. Click **Run workflow**
3. Optionally specify a version (default: 6.5.0)
4. Click **Run workflow**

### Download and Test

Once the workflow completes:

1. Download the `signed-dist` artifact from the workflow run
2. Extract the artifacts:
   ```bash
   unzip signed-dist.zip
   ```

3. Test on your platform:
   ```bash
   # Extract the archive for your platform
   tar -xzf dist/bazbom-*/bazbom-x86_64-apple-darwin.tar.gz

   # Test the binary
   ./bazbom --version
   ./bazbom --help
   ```

4. Verify checksums:
   ```bash
   sha256sum -c dist/bazbom-*/bazbom-x86_64-apple-darwin.tar.gz.sha256
   ```

5. Verify signatures (optional, requires cosign):
   ```bash
   cosign verify-blob \
     --signature dist/bazbom-*/bazbom-x86_64-apple-darwin.tar.gz.sig \
     --certificate-identity-regexp ".*" \
     --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
     dist/bazbom-*/bazbom-x86_64-apple-darwin.tar.gz
   ```

## Method 2: Test Locally

### Build the Binary

```bash
cargo build --release -p bazbom
```

### Package the Binary

Use the packaging script:

```bash
./scripts/package-local-build.sh
```

This creates:
- `dist/bazbom-<target>.tar.gz` - The binary archive
- `dist/bazbom-<target>.tar.gz.sha256` - SHA256 checksum

### Test the Install Script

The install script downloads from GitHub releases, so you need to either:

**Option A: Create a test release**

1. Create a pre-release on GitHub with your test artifacts
2. Test the install script:
   ```bash
   VERSION=6.5.0-test curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
   ```

**Option B: Mock a local server**

1. Start a local HTTP server:
   ```bash
   python3 -m http.server 8000
   ```

2. Modify the install script temporarily to use localhost
3. Test the installation

**Option C: Manual installation (quickest for testing)**

```bash
# Extract the binary
tar -xzf dist/bazbom-*.tar.gz

# Install to a test location
mkdir -p ~/.local/bin
install -m 755 bazbom ~/.local/bin/bazbom

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"

# Test
bazbom --version
bazbom --help
```

## Testing the Homebrew Formula

### Prerequisites

- Homebrew installed (`/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"`)
- A GitHub release with artifacts OR local test artifacts

### Option 1: Test with a Real Release

Once you have a GitHub release with artifacts:

1. Generate the formula with correct checksums:
   ```bash
   ./scripts/generate-homebrew-formula.sh 6.5.0
   ```

2. Create the homebrew-bazbom repository (first time only):
   ```bash
   gh repo create homebrew-bazbom --public --description "Homebrew tap for BazBOM"
   git clone https://github.com/cboyd0319/homebrew-bazbom.git
   cd homebrew-bazbom
   ```

3. Add the formula:
   ```bash
   mkdir -p Formula
   cp /tmp/bazbom-checksums/Formula-bazbom.rb Formula/bazbom.rb
   git add Formula/bazbom.rb README.md
   git commit -m "Add bazbom formula v6.5.0"
   git push origin main
   ```

4. Test the tap:
   ```bash
   # Tap the repository
   brew tap cboyd0319/bazbom

   # Install
   brew install bazbom

   # Verify
   bazbom --version
   which bazbom
   ```

5. Test upgrade (after updating the formula):
   ```bash
   brew update
   brew upgrade bazbom
   ```

### Option 2: Test Formula Locally (Without Publishing)

Test the formula syntax without publishing:

```bash
# Audit the formula
brew audit --strict Formula/bazbom.rb

# Test formula (dry run)
brew install --dry-run Formula/bazbom.rb

# Install from local formula
brew install Formula/bazbom.rb
```

## Testing Checklist

Before creating an official release, verify:

### Binary Functionality
- [ ] Binary builds successfully for all platforms
- [ ] `bazbom --version` returns correct version
- [ ] `bazbom --help` displays help information
- [ ] Basic scan works: `bazbom check` (in a test project)
- [ ] Binary size is reasonable (< 50MB)

### Install Script
- [ ] Script downloads correct binary for platform
- [ ] Script installs to correct location
- [ ] Binary is executable after installation
- [ ] Binary is in PATH or instructions are clear
- [ ] Error messages are helpful

### Homebrew Formula (if applicable)
- [ ] Formula passes `brew audit --strict`
- [ ] Formula installs successfully
- [ ] Correct binary is selected for platform (ARM vs x86_64)
- [ ] Checksums are correct
- [ ] Formula test block passes
- [ ] Binary works after installation
- [ ] Uninstall works cleanly

### Security
- [ ] Binaries are signed with cosign
- [ ] SHA256 checksums are provided
- [ ] Signatures can be verified
- [ ] No secrets or credentials in artifacts

### Documentation
- [ ] README reflects current installation methods
- [ ] CHANGELOG is updated
- [ ] Version numbers are consistent across files

## Platform-Specific Testing

### macOS

```bash
# Test on both architectures if possible
# Apple Silicon (ARM64)
uname -m  # Should show: arm64
bazbom --version

# Intel (x86_64)
uname -m  # Should show: x86_64
bazbom --version

# Test Homebrew
brew install bazbom
bazbom --version
```

### Linux

```bash
# Test on both architectures if possible
# x86_64
uname -m  # Should show: x86_64
bazbom --version

# ARM64 (aarch64)
uname -m  # Should show: aarch64
bazbom --version

# Test in different distros if possible
# - Ubuntu/Debian
# - CentOS/RHEL
# - Alpine
```

### Windows

```bash
# Extract and test
tar -xzf bazbom-x86_64-pc-windows-msvc.tar.gz
./bazbom.exe --version
./bazbom.exe --help
```

## Common Issues

### Binary Not Executable

```bash
chmod +x bazbom
```

### Wrong Architecture Downloaded

Check platform detection:
```bash
uname -s  # OS
uname -m  # Architecture
```

### Homebrew Checksum Mismatch

Regenerate checksums:
```bash
./scripts/generate-homebrew-formula.sh <version>
```

### Install Script Fails

Enable debug mode:
```bash
bash -x install.sh
```

## Automation

For continuous testing, consider:

1. **GitHub Actions Matrix Testing**: Test on all platforms automatically
2. **Pre-release Tags**: Create `v6.5.0-rc1` tags for testing
3. **Draft Releases**: Create draft releases for testing before publishing

## Next Steps

After successful testing:

1. Create an official release tag: `git tag -a v6.5.0 -m "Release v6.5.0"`
2. Push the tag: `git push origin v6.5.0`
3. Wait for the release workflow to complete
4. Update the Homebrew formula (if needed)
5. Announce the release

## Resources

- [Release Workflow](.github/workflows/release.yml)
- [Build Installers Workflow](.github/workflows/build-installers.yml)
- [Install Script](install.sh)
- [Homebrew Tap Guide](homebrew-tap-creation.md)
