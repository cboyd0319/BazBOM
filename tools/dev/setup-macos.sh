#!/usr/bin/env bash

# macOS environment bootstrap for BazBOM development.
# Installs language toolchains, linting CLIs, and configures git hooks.

set -euo pipefail

if [[ "${EUID}" -eq 0 ]]; then
  echo "Do not run this script as root; Homebrew and user tooling expect a non-root user." >&2
  exit 1
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "This script is intended for macOS only." >&2
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
if [[ ! -f "${REPO_ROOT}/Cargo.toml" ]]; then
  echo "Unable to locate BazBOM repository root from script path." >&2
  exit 1
fi

log() {
  printf "\n==> %s\n" "$*"
}

warn() {
  printf "Warning: %s\n" "$*" >&2
}

command_exists() {
  command -v "$1" >/dev/null 2>&1
}

require_command() {
  if ! command_exists "$1"; then
    printf "Error: %s is required but not installed.\n" "$1" >&2
    exit 1
  fi
}

append_profile_line() {
  local line="$1"
  local shell_profile="$2"

  mkdir -p "$(dirname "$shell_profile")"
  touch "$shell_profile"
  if ! grep -Fqx "$line" "$shell_profile"; then
    echo "$line" >>"$shell_profile"
    log "Added to ${shell_profile}: ${line}"
  fi
}

ensure_command_line_tools() {
  if ! xcode-select -p >/dev/null 2>&1; then
    warn "Xcode Command Line Tools not detected. Triggering installation dialog..."
    xcode-select --install || true
    warn "Re-run this script after the Command Line Tools finish installing."
    exit 1
  fi
}

ensure_homebrew() {
  local brew_bin=""

  if command_exists brew; then
    return
  fi

  if [[ -x /opt/homebrew/bin/brew ]]; then
    brew_bin="/opt/homebrew/bin/brew"
  elif [[ -x /usr/local/bin/brew ]]; then
    brew_bin="/usr/local/bin/brew"
  else
    warn "Homebrew not detected on PATH. Installing Homebrew..."
    /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    if [[ -x /opt/homebrew/bin/brew ]]; then
      brew_bin="/opt/homebrew/bin/brew"
    elif [[ -x /usr/local/bin/brew ]]; then
      brew_bin="/usr/local/bin/brew"
    else
      echo "Homebrew installation completed but brew binary not found. Please install manually." >&2
      exit 1
    fi
  fi

  # shellcheck disable=SC2016
  local brew_profile_line="eval \"\$(${brew_bin} shellenv)\""
  append_profile_line "$brew_profile_line" "${HOME}/.zprofile"
  append_profile_line "$brew_profile_line" "${HOME}/.bash_profile"

  # shellcheck disable=SC1091
  eval "$(${brew_bin} shellenv)"
}

install_brew_formulae() {
  local formulae=("$@")
  if [[ ${#formulae[@]} -eq 0 ]]; then
    return
  fi

  log "Ensuring Homebrew formulae: ${formulae[*]}"
  brew update

  for formula in "${formulae[@]}"; do
    if brew list --formula "$formula" >/dev/null 2>&1; then
      log "Homebrew formula '${formula}' already installed."
    else
      brew install "$formula"
    fi
  done
}

install_rust_toolchain() {
  log "Installing Rust toolchain (stable) ..."
  if ! command_exists rustup; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck disable=SC1091
    source "${HOME}/.cargo/env"
  else
    rustup update stable
    if [[ -f "${HOME}/.cargo/env" ]]; then
      # shellcheck disable=SC1091
      source "${HOME}/.cargo/env"
    fi
  fi
  rustup component add rustfmt clippy llvm-tools-preview
  cargo install cargo-llvm-cov --locked --force

  append_profile_line 'source "$HOME/.cargo/env"' "${HOME}/.zprofile"
  append_profile_line 'source "$HOME/.cargo/env"' "${HOME}/.bash_profile"
}

configure_java_path() {
  require_command brew
  local openjdk_prefix
  openjdk_prefix="$(brew --prefix openjdk@17)"

  if [[ -d "${openjdk_prefix}/bin" ]]; then
    local export_line="export PATH=\"${openjdk_prefix}/bin:\$PATH\""
    append_profile_line "$export_line" "${HOME}/.zprofile"
    append_profile_line "$export_line" "${HOME}/.bash_profile"
  else
    warn "Could not locate openjdk@17 bin directory at ${openjdk_prefix}/bin."
  fi

  if command_exists /usr/libexec/java_home; then
    if /usr/libexec/java_home -v 17 >/tmp/java_home_path 2>/dev/null; then
      local java_home
      java_home="$(cat /tmp/java_home_path)"
      local java_home_line="export JAVA_HOME=\"${java_home}\""
      append_profile_line "$java_home_line" "${HOME}/.zprofile"
      append_profile_line "$java_home_line" "${HOME}/.bash_profile"
    else
      warn "Unable to determine JAVA_HOME for JDK 17 via /usr/libexec/java_home."
    fi
    rm -f /tmp/java_home_path
  fi
}

install_npm_globals() {
  require_command npm
  log "Installing global npm packages..."
  local packages=(markdownlint-cli typescript @vscode/vsce)
  for package in "${packages[@]}"; do
    if npm list -g --depth=0 "$package" >/dev/null 2>&1; then
      log "npm package '${package}' already installed."
    else
      npm install -g "$package"
    fi
  done
}

install_pipx_tools() {
  log "Installing pipx and Python-based tooling..."

  if ! command_exists pipx; then
    if brew list --formula pipx >/dev/null 2>&1; then
      log "pipx already installed via Homebrew."
    else
      brew install pipx
    fi
  fi

  append_profile_line 'export PATH="$HOME/.local/bin:$PATH"' "${HOME}/.zprofile"
  append_profile_line 'export PATH="$HOME/.local/bin:$PATH"' "${HOME}/.bash_profile"
  pipx ensurepath

  local pipx_tools=(pre-commit)
  for tool in "${pipx_tools[@]}"; do
    if pipx list --short 2>/dev/null | grep -Fxq "$tool"; then
      pipx install "$tool" --force
    else
      pipx install "$tool"
    fi
  done
}

ensure_bazel_alias() {
  if command_exists bazel; then
    return
  fi

  if command_exists bazelisk; then
    append_profile_line 'alias bazel="bazelisk"' "${HOME}/.zprofile"
    append_profile_line 'alias bazel="bazelisk"' "${HOME}/.bash_profile"
    warn "bazel command not found; added shell alias to use bazelisk. Open a new shell to activate."
  else
    warn "Neither bazel nor bazelisk found on PATH. Homebrew installation may have failed."
  fi
}

install_repository_dependencies() {
  if [[ -d "${REPO_ROOT}/crates/bazbom-vscode-extension" ]]; then
    log "Installing VS Code extension dependencies..."
    (cd "${REPO_ROOT}/crates/bazbom-vscode-extension" && npm install --legacy-peer-deps || warn "npm install failed for VS Code extension")
  else
    warn "VS Code extension directory not found; skipping npm dependencies."
  fi

  if [[ -d "${REPO_ROOT}/crates/bazbom-intellij-plugin" ]]; then
    log "Running Gradle wrapper for IntelliJ plugin..."
    (cd "${REPO_ROOT}/crates/bazbom-intellij-plugin" && ./gradlew --version || warn "Gradle wrapper invocation failed for IntelliJ plugin")
  else
    warn "IntelliJ plugin directory not found; skipping Gradle verification."
  fi
}

configure_git_hooks() {
  if [[ -d "${REPO_ROOT}/.git" && -f "${REPO_ROOT}/.pre-commit-config.yaml" ]]; then
    log "Installing pre-commit hooks..."
    (cd "${REPO_ROOT}" && pre-commit install)
  else
    warn "Git repository or .pre-commit-config.yaml not detected; skipping hook installation."
  fi
}

print_summary() {
  log "Environment summary:"
  local bazel_cmd
  if command_exists bazel; then
    bazel_cmd="bazel --version"
  elif command_exists bazelisk; then
    bazel_cmd="bazelisk --version"
  else
    bazel_cmd="echo 'bazel: missing'"
  fi

  local commands=(
    "rustc --version"
    "cargo --version"
    "rustup show active-toolchain"
    "java -version"
    "mvn --version | head -n 1"
    "gradle --version | head -n 1"
    "$bazel_cmd"
    "node --version"
    "npm --version"
    "markdownlint --version"
    "tsc --version"
    "vsce --version"
    "vale --version"
    "trufflehog --version"
    "gitleaks version"
    "pre-commit --version"
    "buildifier --version | head -n 1"
  )

  for cmd in "${commands[@]}"; do
    printf "  - %s: " "$cmd"
    if eval "$cmd" &>/tmp/setup-output; then
      cat /tmp/setup-output | head -n 1
    else
      printf "missing or failed\n"
    fi
  done
  rm -f /tmp/setup-output
}

main() {
  ensure_command_line_tools
  ensure_homebrew
  # shellcheck disable=SC1091
  eval "$(brew shellenv)"

  install_rust_toolchain

  install_brew_formulae \
    openjdk@17 \
    maven \
    gradle \
    bazelisk \
    node \
    vale \
    buildifier \
    trufflehog \
    gitleaks \
    pipx

  configure_java_path
  ensure_bazel_alias
  install_npm_globals
  install_pipx_tools
  install_repository_dependencies
  configure_git_hooks
  print_summary

  log "macOS development environment setup complete."
  log "Open a new terminal session to ensure PATH changes take effect."
}

main "$@"
