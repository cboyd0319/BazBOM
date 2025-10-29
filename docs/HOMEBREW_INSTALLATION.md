# Homebrew Installation

This guide explains how to install BazBOM using Homebrew on macOS and Linux.

## Prerequisites

- Homebrew installed on your system
- macOS or Linux operating system
- Java 11 or later (optional, required only for reachability analysis)

### Install Homebrew

If you do not have Homebrew installed:

**macOS:**

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Linux:**

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Follow the post-installation instructions to add Homebrew to your PATH.

## Installation

### Step 1: Add the BazBOM Tap

Add the BazBOM Homebrew tap to access the formula:

```bash
brew tap cboyd0319/bazbom
```

### Step 2: Install BazBOM

Install BazBOM using the formula:

```bash
brew install bazbom
```

### Step 3: Verify Installation

Check that BazBOM is installed correctly:

```bash
bazbom --version
```

Expected output:

```
bazbom 0.1.0
```

## Quick Start

After installation, you can immediately start using BazBOM:

```bash
# Scan a project directory
cd /path/to/your/jvm/project
bazbom scan .

# Output SPDX format
bazbom scan . --format spdx

# Output SARIF format for GitHub
bazbom scan . --format sarif
```

## Shell Completions

Homebrew automatically installs shell completions for Bash and Zsh during installation. To use them:

**Bash:**

Completions are installed to `$(brew --prefix)/etc/bash_completion.d/`. Ensure your `.bashrc` or `.bash_profile` sources Homebrew's bash completion:

```bash
if type brew &>/dev/null; then
  HOMEBREW_PREFIX="$(brew --prefix)"
  if [[ -r "${HOMEBREW_PREFIX}/etc/profile.d/bash_completion.sh" ]]; then
    source "${HOMEBREW_PREFIX}/etc/profile.d/bash_completion.sh"
  fi
fi
```

**Zsh:**

Completions are installed to `$(brew --prefix)/share/zsh/site-functions/`. Ensure your `.zshrc` includes the Homebrew function directory:

```bash
if type brew &>/dev/null; then
  FPATH="$(brew --prefix)/share/zsh/site-functions:${FPATH}"
  autoload -Uz compinit
  compinit
fi
```

## Updating BazBOM

Update BazBOM to the latest version:

```bash
brew update
brew upgrade bazbom
```

## Uninstallation

Remove BazBOM from your system:

```bash
brew uninstall bazbom
brew untap cboyd0319/bazbom
```

## Homebrew Tap Repository

The Homebrew tap is maintained at:

```
https://github.com/cboyd0319/homebrew-bazbom
```

The formula is located at:

```
Formula/bazbom.rb
```

## Platform Support

The Homebrew formula supports:

- **macOS**
  - Intel (x86_64)
  - Apple Silicon (aarch64)
- **Linux**
  - x86_64
  - aarch64

The formula automatically detects your platform and installs the correct binary.

## Verifying Binaries

BazBOM binaries distributed via Homebrew are signed using Sigstore cosign. While Homebrew handles binary integrity through checksums, you can manually verify signatures if desired.

See [RELEASE_PROCESS.md](RELEASE_PROCESS.md) for details on signature verification with cosign.

## Advanced Configuration

### Custom Cache Location

By default, BazBOM stores advisory data in `~/.bazbom/cache`. To use a custom location:

```bash
export BAZBOM_CACHE_DIR=/path/to/cache
bazbom db sync
```

### Offline Mode

BazBOM operates offline by default. To fetch advisory data:

```bash
bazbom db sync
```

This command downloads vulnerability data from OSV, NVD, GHSA, and CISA KEV, storing it locally for offline use.

## Troubleshooting

### Command Not Found After Installation

If `bazbom` is not found after installation:

1. Verify installation completed successfully:

   ```bash
   brew list bazbom
   ```

2. Check Homebrew's bin directory is in your PATH:

   ```bash
   echo $PATH | grep $(brew --prefix)/bin
   ```

3. If not present, add to your shell configuration:

   ```bash
   export PATH="$(brew --prefix)/bin:$PATH"
   ```

### Java Not Found (Reachability)

If you encounter Java-related errors when using reachability features:

1. Install Java 11 or later:

   ```bash
   brew install openjdk@11
   ```

2. Add Java to your PATH as instructed by Homebrew.

3. Reachability analysis is optional and can be disabled by omitting the `--reachability` flag.

### Permission Errors

If you encounter permission errors:

```bash
brew doctor
```

Follow any recommendations to fix Homebrew permissions.

### Formula Not Found

If the tap or formula is not found:

1. Update Homebrew:

   ```bash
   brew update
   ```

2. Re-add the tap:

   ```bash
   brew untap cboyd0319/bazbom
   brew tap cboyd0319/bazbom
   ```

3. Verify the tap is present:

   ```bash
   brew tap
   ```

## Alternative Installation Methods

If Homebrew is not suitable for your environment, see alternative installation methods:

- [Manual Installation](QUICKSTART.md#option-0-rust-cli-preview) - Build from source
- [GitHub Releases](RELEASE_PROCESS.md) - Download pre-built binaries
- [Installer Script](../README.md#option-1-one-line-install-recommended) - Python-based legacy installer

## Support

For issues specific to Homebrew installation:

1. Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
3. Open a new issue with the `homebrew` label

For Homebrew formula issues, open an issue in the tap repository:

```
https://github.com/cboyd0319/homebrew-bazbom/issues
```

## Contributing

To contribute improvements to the Homebrew formula:

1. Fork the tap repository
2. Make your changes to `Formula/bazbom.rb`
3. Test locally with `brew install --build-from-source ./Formula/bazbom.rb`
4. Submit a pull request

See [CONTRIBUTING.md](../CONTRIBUTING.md) for general contribution guidelines.
