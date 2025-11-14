# Installation Guide

Get BazBOM installed quickly with one of the methods below. Choose the easiest option for your workflow.

## Quick Install (Recommended)

### 🚀 One-Line Install Script

The fastest way to get BazBOM on macOS or Linux:

```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

Or using wget:

```bash
wget -qO- https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

This script:
- Auto-detects your OS and architecture
- Downloads the latest pre-built binary from GitHub releases
- Installs to `/usr/local/bin` (or prompts for sudo if needed)
- Verifies the installation

**Supported platforms:** macOS (x86_64, ARM64), Linux (x86_64, ARM64)

---

### 🍺 Homebrew (macOS & Linux)

> **Status:** Homebrew tap is ready but not yet published. Coming soon!

Once published, installation will be:

```bash
brew tap cboyd0319/bazbom
brew install bazbom
```

**Update to latest version:**
```bash
brew upgrade bazbom
```

---

### 📦 Download Pre-Built Binaries

Download the latest release directly from GitHub:

1. Go to [BazBOM Releases](https://github.com/cboyd0319/BazBOM/releases/latest)
2. Download the appropriate archive for your platform:
   - `bazbom-x86_64-apple-darwin.tar.gz` - macOS Intel
   - `bazbom-aarch64-apple-darwin.tar.gz` - macOS Apple Silicon
   - `bazbom-x86_64-unknown-linux-gnu.tar.gz` - Linux x86_64
   - `bazbom-aarch64-unknown-linux-gnu.tar.gz` - Linux ARM64
   - `bazbom-x86_64-pc-windows-msvc.tar.gz` - Windows x86_64

3. Extract and install:

**macOS/Linux:**
```bash
tar -xzf bazbom-*.tar.gz
sudo install -m 755 bazbom /usr/local/bin/bazbom
bazbom --version
```

**Windows:**
```powershell
tar -xzf bazbom-x86_64-pc-windows-msvc.tar.gz
# Move bazbom.exe to a directory in your PATH
```

Each release includes:
- Pre-compiled binaries for all platforms
- SHA256 checksums (`.sha256` files)
- Cosign signatures (`.sig` files) for verification

**Verify with cosign (optional):**
```bash
cosign verify-blob \
  --signature bazbom-x86_64-apple-darwin.tar.gz.sig \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  bazbom-x86_64-apple-darwin.tar.gz
```

---

### 🦀 Cargo Install (Rust Developers)

If you have Rust installed, you can install directly from source:

```bash
cargo install --git https://github.com/cboyd0319/BazBOM bazbom
```

**Once published to crates.io:**
```bash
cargo install bazbom
```

This compiles from source, so it takes a few minutes but ensures you get a binary optimized for your exact system.

---

## Manual Source Installation

Want to build from source or contribute to development? Follow these steps.

### Prerequisites

- macOS or Linux workstation (Apple Silicon, x86_64, or compatible)
- `git`
- Rust toolchain (`rustup`, `cargo`, `rustc`). Install via <https://rustup.rs/>.
- Build dependencies:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: `build-essential`, `pkg-config`, `libssl-dev`, `curl`, `tar`
- Optional: `sudo`/administrator rights if you plan to copy binaries into `/usr/local/bin`

### 1. Clone the Repository

```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
```

### 2. Build the CLI

```bash
cargo build --release -p bazbom
```

The compiled binary is created at `target/release/bazbom`.

### 3. Add BazBOM to Your PATH

#### Option A: Install system-wide

```bash
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

#### Option B: Use a workspace-local PATH entry

```bash
export PATH="$PWD/target/release:$PATH"
```

Add the export to your shell profile if you want it to persist:

```bash
echo 'export PATH="$HOME/src/BazBOM/target/release:$PATH"' >> ~/.zshrc  # adjust path + shell as needed
```

### 4. Verify Installation

```bash
bazbom --version
bazbom --help | head -n 20
```

If the command is not found, confirm that the binary exists and that your PATH changes were applied:

```bash
ls -lh target/release/bazbom
which bazbom
```

---

## Updating BazBOM

### With Install Script
```bash
curl -sSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | sh
```

### With Homebrew (when available)
```bash
brew upgrade bazbom
```

### With Cargo
```bash
cargo install --git https://github.com/cboyd0319/BazBOM bazbom --force
```

### From Source
```bash
cd BazBOM
git pull
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

---

## Uninstalling

### If Installed via Script or Binary Download
```bash
sudo rm -f /usr/local/bin/bazbom
```

### If Installed via Homebrew
```bash
brew uninstall bazbom
brew untap cboyd0319/bazbom
```

### If Installed via Cargo
```bash
cargo uninstall bazbom
```

### If Installed from Source
```bash
sudo rm -f /usr/local/bin/bazbom
# or remove the directory you added to PATH
```

---

## Platform Support

| Platform | x86_64 | ARM64 | Installation Method |
|----------|--------|-------|---------------------|
| **macOS** | ✅ | ✅ | Install script, Homebrew (soon), Binary download, Cargo, Source |
| **Linux** | ✅ | ✅ | Install script, Binary download, Cargo, Source |
| **Windows** | ✅ | ⏳ | Binary download, Cargo, Source |

- ✅ Fully supported
- ⏳ Planned

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `bazbom: command not found` | Ensure `/usr/local/bin` or `target/release` is on your PATH. Run: `export PATH="/usr/local/bin:$PATH"` |
| Rust compiler errors | Update toolchain: `rustup update stable && rustup component add rustfmt clippy` |
| Missing OpenSSL / pkg-config | Install `libssl-dev` and `pkg-config` packages (Linux) or Xcode Command Line Tools (macOS) |
| Java reachability errors | Install Java 11+ and set `JAVA_HOME` |
| Permission denied on install | Use `sudo` or install to a user-writable directory like `~/.local/bin` |
| Binary won't run (macOS) | Allow unsigned binary: `xattr -d com.apple.quarantine /usr/local/bin/bazbom` |

---

## Verifying Releases

All BazBOM releases are:
- **Signed with cosign** using keyless signing (SLSA Level 3)
- **Checksummed** with SHA256
- **Built with GitHub Actions** with full provenance

To verify a download:

```bash
# Verify checksum
sha256sum -c bazbom-x86_64-apple-darwin.tar.gz.sha256

# Verify signature (requires cosign)
cosign verify-blob \
  --signature bazbom-x86_64-apple-darwin.tar.gz.sig \
  --certificate-identity-regexp ".*" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  bazbom-x86_64-apple-darwin.tar.gz
```

---

## Next Steps

After installation:

1. **Quick scan:** `bazbom check`
2. **Full scan with reachability:** `bazbom scan --reachability`
3. **View help:** `bazbom --help`
4. **Read the [Quick Start Guide](quickstart.md)**

---

## Package Managers Roadmap

We're working on making BazBOM available through more package managers:

- ✅ **GitHub Releases** - Available now
- ✅ **Install Script** - Available now
- 🚧 **Homebrew Tap** - Ready to publish
- ⏳ **Homebrew Core** - After tap is stable
- ⏳ **Chocolatey** (Windows) - Planned
- ⏳ **Scoop** (Windows) - Planned
- ⏳ **winget** (Windows) - Planned
- ⏳ **Docker Hub** - Planned
- ⏳ **crates.io** - Planned

Stay tuned for updates!
