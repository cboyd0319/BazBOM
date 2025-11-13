# Manual Source Installation

> **Status:** Package manager distributions (Homebrew, Chocolatey, winget, etc.) are not yet published. The only supported way to run BazBOM today is to clone this repository and build the CLI from source.

## Prerequisites

- macOS or Linux workstation (Apple Silicon, x86_64, or compatible)
- `git`
- Rust toolchain (`rustup`, `cargo`, `rustc`). Install via <https://rustup.rs/>.
- Build dependencies:
  - macOS: Xcode Command Line Tools (`xcode-select --install`)
  - Linux: `build-essential`, `pkg-config`, `libssl-dev`, `curl`, `tar`
- Optional: `sudo`/administrator rights if you plan to copy binaries into `/usr/local/bin`

## 1. Clone the Repository

```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
```

## 2. Build the CLI

```bash
cargo build --release -p bazbom
```

The compiled binary is created at `target/release/bazbom`.

## 3. Add BazBOM to Your PATH

### Option A: Install system-wide

```bash
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

### Option B: Use a workspace-local PATH entry

```bash
export PATH="$PWD/target/release:$PATH"
```

Add the export to your shell profile if you want it to persist:

```bash
echo 'export PATH="$HOME/src/BazBOM/target/release:$PATH"' >> ~/.zshrc  # adjust path + shell as needed
```

## 4. Verify Installation

```bash
bazbom --version
bazbom --help | head -n 20
```

If the command is not found, confirm that the binary exists and that your PATH changes were applied:

```bash
ls -lh target/release/bazbom
which bazbom
```

## Updating BazBOM

```bash
cd BazBOM
git pull
cargo build --release -p bazbom
sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom
```

## Uninstalling

```bash
sudo rm -f /usr/local/bin/bazbom
# or remove the directory you added to PATH
```

## Troubleshooting

| Issue | Fix |
|-------|-----|
| `bazbom: command not found` | Ensure `target/release` is on your PATH or copy the binary into a directory that already is (e.g., `/usr/local/bin`). |
| Rust compiler errors | Update toolchain: `rustup update stable && rustup component add rustfmt clippy`. |
| Missing OpenSSL / pkg-config | Install `libssl-dev` and `pkg-config` packages (Linux) or Xcode Command Line Tools (macOS). |
| Java reachability errors | Install Java 11+ and set `JAVA_HOME`. |

## Future Package Managers

The `homebrew/`, `windows/`, and related directories remain in the repo for future distribution work, but no public taps or installers exist yet. When those channels are ready, this guide will be updated with the appropriate instructions.
