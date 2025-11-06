# Creating the Homebrew Tap Repository

This guide provides step-by-step instructions for creating and maintaining the BazBOM Homebrew tap repository.

## Overview

A Homebrew tap is a third-party repository that contains formulae (package definitions) for Homebrew. Before submitting to the official homebrew-core, BazBOM uses a user-owned tap for initial distribution and testing.

## Repository Structure

The Homebrew tap repository should have the following structure:

```
homebrew-bazbom/
├── README.md
├── Formula/
│   └── bazbom.rb
└── .github/
    └── workflows/
        └── tests.yml
```

## Step 1: Create the Repository

1. Create a new public repository named `homebrew-bazbom` in your GitHub account:

   ```bash
   # Via GitHub CLI
   gh repo create homebrew-bazbom --public --description "Homebrew tap for BazBOM"
   
   # Or via GitHub web interface
   # Navigate to https://github.com/new
   # Repository name: homebrew-bazbom
   # Description: Homebrew tap for BazBOM
   # Public: Yes
   ```

2. Clone the repository locally:

   ```bash
   git clone https://github.com/cboyd0319/homebrew-bazbom.git
   cd homebrew-bazbom
   ```

## Step 2: Create the Formula

1. Create the `Formula` directory:

   ```bash
   mkdir -p Formula
   ```

2. Create `Formula/bazbom.rb` with the following template:

   ```ruby
   class Bazbom < Formula
     desc "Build-time SBOM, SCA, and dependency graph for JVM projects"
     homepage "https://github.com/cboyd0319/BazBOM"
     version "0.1.0"
     license "MIT"

     on_macos do
       if Hardware::CPU.arm?
         url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-aarch64-apple-darwin.tar.gz"
         sha256 "INSERT_AARCH64_MACOS_SHA256_HERE"
       else
         url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-apple-darwin.tar.gz"
         sha256 "INSERT_X86_64_MACOS_SHA256_HERE"
       end
     end

     on_linux do
       if Hardware::CPU.arm?
         url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-aarch64-unknown-linux-gnu.tar.gz"
         sha256 "INSERT_AARCH64_LINUX_SHA256_HERE"
       else
         url "https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-unknown-linux-gnu.tar.gz"
         sha256 "INSERT_X86_64_LINUX_SHA256_HERE"
       end
     end

     def install
       bin.install "bazbom"
       
       # Generate and install shell completions
       generate_completions_from_executable(bin/"bazbom", "completions")
     end

     test do
       assert_match "bazbom", shell_output("#{bin}/bazbom --version")
       
       # Basic functionality test
       system bin/"bazbom", "--help"
     end
   end
   ```

## Step 3: Generate SHA256 Checksums

After creating a release in the main BazBOM repository, generate checksums for each platform:

```bash
# Download release artifacts
VERSION="0.1.0"
for platform in aarch64-apple-darwin x86_64-apple-darwin aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu; do
  curl -LO "https://github.com/cboyd0319/BazBOM/releases/download/v${VERSION}/bazbom-${platform}.tar.gz"
  sha256sum "bazbom-${platform}.tar.gz" | awk '{print $1}'
done
```

Update the SHA256 values in `Formula/bazbom.rb` with the output.

## Step 4: Create README

Create `README.md` in the repository root:

```markdown
# Homebrew BazBOM

Official Homebrew tap for [BazBOM](https://github.com/cboyd0319/BazBOM) - Enterprise-grade build-time SBOM, SCA, and dependency graph for JVM projects.

## Installation

```bash
brew tap cboyd0319/bazbom
brew install bazbom
```

## Usage

```bash
bazbom --version
bazbom scan .
```

## Documentation

For full documentation, see the [BazBOM repository](https://github.com/cboyd0319/BazBOM).

## Supported Platforms

- macOS (Intel and Apple Silicon)
- Linux (x86_64 and aarch64)

## Issues

Report issues at [BazBOM Issues](https://github.com/cboyd0319/BazBOM/issues).
```

## Step 5: Add CI Testing

Create `.github/workflows/tests.yml`:

```yaml
name: Formula Tests

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up Homebrew
        uses: Homebrew/actions/setup-homebrew@master
      
      - name: Tap repository
        run: |
          mkdir -p $(brew --repo)/Library/Taps/cboyd0319
          ln -s $PWD $(brew --repo)/Library/Taps/cboyd0319/homebrew-bazbom
      
      - name: Test formula
        run: |
          brew test-bot --only-setup
          brew test-bot --only-tap-syntax
          brew audit --strict Formula/bazbom.rb
      
      - name: Install formula
        run: brew install bazbom
      
      - name: Test installation
        run: |
          bazbom --version
          bazbom --help
```

## Step 6: Commit and Push

```bash
git add Formula/bazbom.rb README.md .github/workflows/tests.yml
git commit -m "Initial formula for BazBOM"
git push origin main
```

## Step 7: Test Locally

Before announcing the tap, test it locally:

```bash
# Untap if already tapped
brew untap cboyd0319/bazbom 2>/dev/null || true

# Tap the repository
brew tap cboyd0319/bazbom

# Install from tap
brew install bazbom

# Verify installation
bazbom --version

# Test basic functionality
cd /tmp
mkdir test-project
cd test-project
echo '<project></project>' > pom.xml
bazbom scan . || echo "Expected to fail without valid project"
```

## Updating the Formula

When releasing a new version:

1. Update version in BazBOM repository
2. Create and push a new git tag
3. Wait for release workflow to complete
4. Download new release artifacts and generate checksums
5. Update `Formula/bazbom.rb`:
   - Change `version` field
   - Update all `url` fields with new version
   - Update all `sha256` fields with new checksums
6. Commit and push changes:

   ```bash
   git add Formula/bazbom.rb
   git commit -m "Update bazbom to version X.Y.Z"
   git push origin main
   ```

7. Test the update:

   ```bash
   brew update
   brew upgrade bazbom
   bazbom --version
   ```

## Formula Best Practices

### Version Strings

- Use semantic versioning (e.g., "0.1.0", "1.2.3")
- Match the version from BazBOM releases
- Do not include "v" prefix in version string

### SHA256 Checksums

- Always generate checksums from official release artifacts
- Never manually edit or guess checksum values
- Verify checksums match release artifacts

### Testing

- Include meaningful tests in the `test` block
- Test version output at minimum
- Test help command to verify binary works
- Consider adding integration tests for core functionality

### Documentation

- Keep formula description concise
- Link to main repository for full documentation
- Include clear installation instructions in README
- Document any platform-specific considerations

## Troubleshooting

### Formula Audit Fails

If `brew audit` reports issues:

```bash
# Run audit locally
brew audit --strict Formula/bazbom.rb

# Common issues:
# - Missing license field
# - Invalid URL format
# - Checksum mismatch
# - Missing test block
```

### Installation Fails

If installation fails:

```bash
# Check formula syntax
brew install --verbose bazbom

# Verify checksums
brew fetch bazbom --force

# Check download URLs
curl -I https://github.com/cboyd0319/BazBOM/releases/download/v0.1.0/bazbom-x86_64-apple-darwin.tar.gz
```

### Binary Not Found After Install

If the binary is not in PATH:

```bash
# Check installation location
brew list bazbom

# Verify symlink
ls -la $(brew --prefix)/bin/bazbom

# Reinstall if needed
brew reinstall bazbom
```

## Submitting to homebrew-core

After several stable releases in the user tap, consider submitting to homebrew-core:

1. Ensure formula follows all homebrew-core guidelines
2. Add bottle blocks for faster installation
3. Open PR to homebrew-core repository
4. Address review feedback
5. Wait for CI to pass and PR to be merged

See [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook) for detailed guidelines.

## References

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Tap Documentation](https://docs.brew.sh/Taps)
- [Formula API Reference](https://rubydoc.brew.sh/Formula)
- [Acceptable Formulae Guidelines](https://docs.brew.sh/Acceptable-Formulae)
- [BazBOM Release Process](release-process.md)
