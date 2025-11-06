# Local Development Environment

This guide consolidates the tooling requirements for hacking on BazBOM locally. It is based on the canonical copilot instructions in `.github/copilot-instructions.md` and the automation steps in `.github/copilot-setup-steps.yml`. Follow the platform-specific sections below, then run the validation commands to confirm your workstation is ready for development and linting.

## Required Tools (All Platforms)

- Rust (stable) via `rustup`; install the components `rustfmt`, `clippy`, and `llvm-tools-preview`
- `cargo-llvm-cov` for coverage analysis
- OpenJDK 17, Maven, Gradle, and Bazel (Bazelisk wrapper recommended)
- Node.js (LTS) with global npm packages: `markdownlint-cli`, `typescript`, `@vscode/vsce`
- Python 3 with `pip`/`pipx` (optional utilities and scripts)
- Security and formatting CLIs used by pre-commit: `trufflehog`, `gitleaks`, `markdownlint`, `buildifier`
- Vale for documentation linting
- Git hooks managed through `pre-commit`

Refer to `tool-versions.toml` for pinned third-party tool releases, checksums, and download locations when manual installation is required.

## macOS Setup

Run the automated script for a fully managed setup (Homebrew, Rust, npm, pipx, git hooks):

```bash
./tools/dev/setup-macos.sh
```

> The script requires the Xcode Command Line Tools. If they are missing, the installer prompts you to add them before continuing.

If you prefer to execute the steps manually, follow the commands below.

```bash
# 1. Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"
rustup update stable
rustup component add rustfmt clippy llvm-tools-preview
cargo install cargo-llvm-cov --locked

# 2. Core build tooling (requires Homebrew)
brew update
brew install openjdk@17 maven gradle bazelisk node vale buildifier trufflehog gitleaks
echo 'export PATH="/usr/local/opt/openjdk@17/bin:$PATH"' >> ~/.zshrc

# 3. Node-based linters and extension tooling
npm install -g markdownlint-cli typescript @vscode/vsce

# 4. Optional Python helpers
python3 -m pip install --upgrade pip
python3 -m pip install --user pipx
pipx install pre-commit
```

Verify versions:

```bash
rustc --version
cargo --version
java -version
mvn --version
gradle --version
bazel --version
node --version
markdownlint --version
vale --version
trufflehog --version
gitleaks version
```

## Linux Setup

```bash
# 1. Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
source "$HOME/.cargo/env"
rustup update stable
rustup component add rustfmt clippy llvm-tools-preview
cargo install cargo-llvm-cov --locked

# 2. Core build tooling (Debian/Ubuntu shown)
sudo apt-get update
sudo apt-get install -y openjdk-17-jdk maven curl unzip wget python3 python3-pip npm build-essential pkg-config libssl-dev
curl -fsSL https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64 -o /tmp/bazelisk
sudo install -m 0755 /tmp/bazelisk /usr/local/bin/bazel
wget https://services.gradle.org/distributions/gradle-8.5-bin.zip -P /tmp
sudo unzip -d /opt/gradle /tmp/gradle-8.5-bin.zip
sudo ln -sf /opt/gradle/gradle-8.5/bin/gradle /usr/local/bin/gradle

# 3. Security and lint tooling
sudo npm install -g markdownlint-cli typescript @vscode/vsce
python3 -m pip install --user pipx
pipx install pre-commit || python3 -m pip install --user pre-commit
pipx install trufflehog || python3 -m pip install --user trufflehog
pipx install gitleaks || python3 -m pip install --user gitleaks
TMP_DIR=$(mktemp -d)
curl -sSL -o "$TMP_DIR/vale.tar.gz" https://github.com/errata-ai/vale/releases/latest/download/vale_Linux_64-bit.tar.gz
tar -xzf "$TMP_DIR/vale.tar.gz" -C "$TMP_DIR"
sudo install -m 0755 "$TMP_DIR/vale" /usr/local/bin/vale
curl -fsSL https://github.com/bazelbuild/buildtools/releases/latest/download/buildifier-linux-amd64 -o /tmp/buildifier
sudo install -m 0755 /tmp/buildifier /usr/local/bin/buildifier
```

If your distribution does not provide packages for these tools, fetch the archives listed in `tool-versions.toml`, verify the SHA-256 checksums, and place the binaries on your `PATH`. For Fedora/RHEL or other distributions, adapt the package manager commands accordingly. On Windows, use WSL2 with Ubuntu and follow the Linux instructions.

## Post-Install Repository Setup

```bash
# Configure git hooks
pre-commit install

# Pull npm/Gradle dependencies needed for IDE integrations
(cd crates/bazbom-vscode-extension && npm install)
(cd crates/bazbom-intellij-plugin && ./gradlew --version)
```

Run the confirmation script from `.github/copilot-setup-steps.yml` or execute the following spot checks:

```bash
rustup show active-toolchain
cargo --list | head -n 5
java -version
gradle --version
bazel --version
node --version
vale --version
markdownlint --version
trufflehog --version
gitleaks version
```

## Daily Development Commands

- Formatting: `cargo fmt --all -- --check`
- Rust linting: `cargo clippy --all-targets --all-features -- -D warnings`
- Rust tests: `cargo test --all --locked`
- Coverage (optional): `cargo llvm-cov --workspace --lcov --output-path coverage.lcov`
- Bazel: `bazel build //...` and `bazel test //...`
- VS Code extension: `(cd crates/bazbom-vscode-extension && npm test)`
- IntelliJ plugin: `(cd crates/bazbom-intellij-plugin && ./gradlew test)`
- Documentation linting: `markdownlint --config .markdownlint.json **/*.md` and `vale docs/`
- Git hooks: `pre-commit run --all-files`

These commands mirror the CI pipeline and the expectations captured in `.github/copilot-instructions.md`. Running them locally before opening a pull request helps prevent regressions and ensures the BazBOM security posture is preserved.

## Troubleshooting

- Ensure your shell loads `~/.cargo/env` so the Rust toolchain is on `PATH`.
- If `pre-commit` cannot find `trufflehog` or `gitleaks`, install them with `pipx` or expose their binaries via `brew`/`apt`.
- When Bazel reports an unsupported Java runtime, confirm that JDK 17 is active (`java -version`) and adjust `JAVA_HOME`.
- For Vale or markdownlint failures, sync the configuration files (`.vale.ini`, `.markdownlint.json`) and rerun `npm install -g markdownlint-cli`.
- Re-run `pre-commit autoupdate` after upgrading tool versions to keep hook revisions aligned with `tool-versions.toml`.
- If `pipx` binaries are not on your `PATH`, add `~/.local/bin` (Linux) or `~/Library/Python/<version>/bin` (macOS) to your shell profile.
