# Quick Start: Testing BazBOM Installers

This is a quick reference for testing installers before a release.

## 🚀 TL;DR - ONE COMMAND!

```bash
# Test everything locally in one command:
make installer-test
```

**That's it!** This single command:
- ✅ Builds the binary
- ✅ Packages it
- ✅ Sets up a mock server
- ✅ Tests the install script end-to-end
- ✅ Verifies it works
- ✅ Cleans up automatically

## Quick Command Reference

```bash
# Local testing (⭐ RECOMMENDED)
make installer-test              # Full test - ONE COMMAND!
make installer-test-quick        # Skip build step
make package                     # Just package the binary

# GitHub Actions
make installer-build             # Trigger workflow from CLI

# Multi-platform
./scripts/test-installer-docker.sh all      # Test on 5+ platforms

# Homebrew
make homebrew-formula            # Generate formula

# Release readiness
make release-check               # Comprehensive pre-release check
```

## Traditional Methods (If Needed)

### GitHub Actions Build

```bash
# Via CLI (easier):
make installer-build

# Or manually:
# 1. Go to: Actions → Build Installers (Manual) → Run workflow
# 2. Download the signed-dist artifact when complete
# 3. Extract and test:
unzip signed-dist.zip
tar -xzf dist/bazbom-*/bazbom-<your-platform>.tar.gz
./bazbom --version
```

### Manual Local Test

```bash
# Build
cargo build --release -p bazbom

# Package
./scripts/package-local-build.sh

# Test
tar -xzf dist/bazbom-*.tar.gz
./bazbom --version
```

### Homebrew Test (After Release)

```bash
# Generate formula
make homebrew-formula

# Note: Homebrew tap updates AUTOMATICALLY on release!
# Or manually copy:
cp /tmp/bazbom-checksums/Formula-bazbom.rb ~/homebrew-bazbom/Formula/bazbom.rb

# Test
brew tap cboyd0319/bazbom
brew install bazbom
```

## Full Documentation

See [testing-installers.md](testing-installers.md) for complete instructions.

## 📦 What's Included

**Workflows:**
- `.github/workflows/build-installers.yml` - Manual build workflow
- `.github/workflows/update-homebrew-tap.yml` - Auto-update Homebrew (⭐ NEW!)
- `.github/workflows/test-installers.yml` - Integration tests (⭐ NEW!)

**Scripts:**
- `scripts/test-installer-local.sh` - One-command local testing (⭐ NEW!)
- `scripts/trigger-installer-build.sh` - CLI workflow trigger (⭐ NEW!)
- `scripts/test-installer-docker.sh` - Multi-platform testing (⭐ NEW!)
- `scripts/generate-homebrew-formula.sh` - Generate Homebrew formula
- `scripts/package-local-build.sh` - Package local builds

**Makefile Targets:** (⭐ NEW!)
- `make installer-test` - One-command testing
- `make installer-build` - Trigger GitHub Actions
- `make homebrew-formula` - Generate formula
- `make release-check` - Pre-release verification

**Docs:**
- `docs/operations/INSTALLER_IMPROVEMENTS.md` - All improvements explained (⭐ NEW!)
- `docs/operations/INSTALLER_QUICK_START.md` - This file (updated)
- `docs/operations/testing-installers.md` - Comprehensive testing guide
- `docs/operations/homebrew-tap-creation.md` - Homebrew tap setup

## Quick Checklist

Before release:
- [ ] Build succeeds on all platforms
- [ ] `bazbom --version` shows correct version
- [ ] Install script works
- [ ] Homebrew formula (if used) passes `brew audit --strict`

## Need Help?

See [testing-installers.md](testing-installers.md) for troubleshooting and detailed instructions.
