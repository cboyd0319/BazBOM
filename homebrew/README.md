# Homebrew Tap Infrastructure

This directory contains the infrastructure for creating and maintaining the BazBOM Homebrew tap.

## Files

- **bazbom.rb.template**: Template for the Homebrew formula with placeholders for version and SHA256 hashes
- **generate-formula.sh**: Script to generate the formula with actual SHA256 hashes from GitHub releases
- **README.md**: This file

## Setup Process

### 1. Create the Tap Repository

Create a new public repository named `homebrew-bazbom` under the `cboyd0319` GitHub account:

```bash
# The repository should be at: https://github.com/cboyd0319/homebrew-bazbom
```

### 2. Initialize the Tap Repository

```bash
# Clone the new tap repository
git clone https://github.com/cboyd0319/homebrew-bazbom.git
cd homebrew-bazbom

# Create Formula directory
mkdir -p Formula

# Add README
cat > README.md << 'EOF'
# Homebrew Tap for BazBOM

Official Homebrew tap for [BazBOM](https://github.com/cboyd0319/BazBOM) - Enterprise-grade build-time SBOM, SCA, and dependency graph for JVM projects.

## Installation

```bash
brew tap cboyd0319/bazbom
brew install bazbom
```

## Usage

See the [main BazBOM repository](https://github.com/cboyd0319/BazBOM) for usage instructions.

## About

BazBOM provides:
- Build-time SBOM generation (SPDX 2.3, CycloneDX 1.5)
- Software Composition Analysis with OSV/NVD/GHSA integration
- Dependency graphs for Maven, Gradle, and Bazel
- Zero telemetry, offline-first operation
- Memory-safe Rust implementation

## License

MIT
EOF

git add .
git commit -m "Initial tap setup"
git push
```

### 3. Generate Formula for a Release

After creating a GitHub release with signed binaries:

```bash
# From the BazBOM repository root
./homebrew/generate-formula.sh 0.1.0

# This will:
# - Fetch SHA256 hashes from the release assets
# - Generate homebrew/bazbom.rb with the correct hashes
# - Print next steps
```

### 4. Update the Tap

Copy the generated formula to the tap repository:

```bash
# Copy formula to tap repository
cp homebrew/bazbom.rb /path/to/homebrew-bazbom/Formula/

# Commit and push
cd /path/to/homebrew-bazbom
git add Formula/bazbom.rb
git commit -m "Update formula for v0.1.0"
git push
```

### 5. Test Installation

```bash
# Test the tap installation
brew tap cboyd0319/bazbom
brew install bazbom

# Verify it works
bazbom --version
```

## Updating for New Releases

For each new release:

1. Create a GitHub release with signed binaries (handled by `.github/workflows/release.yml`)
2. Run `./homebrew/generate-formula.sh <version>` to generate the formula
3. Copy to the tap repository and push
4. Users will automatically get the update via `brew upgrade bazbom`

## Automation Opportunities

Future improvements:
- Automate formula generation in the release workflow
- Automatically open a PR to the tap repository
- Generate bottles for faster installation

## References

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Creating and Maintaining a Tap](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)
- [BazBOM Homebrew Tap Guide](../docs/operations/homebrew-tap-creation.md)
