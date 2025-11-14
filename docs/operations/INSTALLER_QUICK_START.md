# Quick Start: Testing BazBOM Installers

This is a quick reference for testing installers before a release.

## TL;DR - Fastest Method

```bash
# 1. Trigger the GitHub Actions workflow
#    Go to: Actions → Build Installers (Manual) → Run workflow

# 2. Download the signed-dist artifact when complete

# 3. Extract and test
unzip signed-dist.zip
tar -xzf dist/bazbom-*/bazbom-<your-platform>.tar.gz
./bazbom --version
```

## Quick Local Test

```bash
# Build
cargo build --release -p bazbom

# Package
./scripts/package-local-build.sh

# Test
tar -xzf dist/bazbom-*.tar.gz
./bazbom --version
```

## Quick Homebrew Test (After Release)

```bash
# Generate formula
./scripts/generate-homebrew-formula.sh 6.5.0

# Copy formula to homebrew-bazbom repo
cp /tmp/bazbom-checksums/Formula-bazbom.rb ~/homebrew-bazbom/Formula/bazbom.rb

# Test
brew tap cboyd0319/bazbom
brew install bazbom
```

## Full Documentation

See [testing-installers.md](testing-installers.md) for complete instructions.

## Files Created

- **Workflow**: `.github/workflows/build-installers.yml` - Manual build workflow
- **Scripts**:
  - `scripts/generate-homebrew-formula.sh` - Generate Homebrew formula
  - `scripts/package-local-build.sh` - Package local builds
- **Docs**:
  - `docs/operations/testing-installers.md` - Full testing guide
  - `docs/operations/homebrew-tap-creation.md` - Homebrew tap setup

## Quick Checklist

Before release:
- [ ] Build succeeds on all platforms
- [ ] `bazbom --version` shows correct version
- [ ] Install script works
- [ ] Homebrew formula (if used) passes `brew audit --strict`

## Need Help?

See [testing-installers.md](testing-installers.md) for troubleshooting and detailed instructions.
