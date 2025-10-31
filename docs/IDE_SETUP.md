# IDE Integration Setup Guide

This guide covers installation and configuration of BazBOM IDE plugins for real-time vulnerability scanning.

## Overview

BazBOM provides native IDE integration for:
- **IntelliJ IDEA** (Community & Ultimate)
- **VS Code** (and compatible editors)
- **Any LSP-compatible editor** (Vim, Emacs, Sublime Text, etc.)

## Prerequisites

All IDE integrations require:
1. **BazBOM CLI** installed and in PATH
2. **Advisory database synced** at least once

```bash
# Install BazBOM CLI (choose one method)
# Via Homebrew (recommended)
brew install cboyd0319/tap/bazbom

# Or build from source
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release
sudo cp target/release/bazbom /usr/local/bin/

# Verify installation
bazbom --version

# Sync advisory database
bazbom db sync
```

---

## IntelliJ IDEA Plugin

Real-time vulnerability scanning with dependency tree visualization, inline warnings, and one-click fixes.

### Features

- ✅ **Dependency Tree Visualization** - Side panel showing all dependencies with security status
- ✅ **Real-time Vulnerability Highlighting** - Inline warnings in `pom.xml`, `build.gradle`, `BUILD.bazel`
- ✅ **One-Click Quick Fixes** - Automatically upgrade vulnerable dependencies with `Alt+Enter`
- ✅ **Automated Testing** - Runs tests after applying fixes to ensure safety
- ✅ **Build System Auto-Detection** - Supports Maven, Gradle, and Bazel
- ✅ **Privacy-First** - All scanning happens locally, no data sent to external servers

### Requirements

- **IntelliJ IDEA**: 2023.3 or later (Community or Ultimate Edition)
- **Java**: 17+ (bundled with IntelliJ)

### Installation

#### From JetBrains Marketplace (Coming Soon)

1. Open IntelliJ IDEA
2. Go to **Settings/Preferences** → **Plugins** → **Marketplace**
3. Search for "BazBOM Security Scanner"
4. Click **Install**
5. Restart IntelliJ IDEA

#### Manual Installation (Development/Beta)

1. Build the plugin:
   ```bash
   cd crates/bazbom-intellij-plugin
   ./gradlew buildPlugin
   ```

2. Install in IntelliJ IDEA:
   - Go to **Settings/Preferences** → **Plugins** → **⚙️** → **Install Plugin from Disk...**
   - Select `build/distributions/bazbom-intellij-plugin-1.0.0.zip`
   - Restart IntelliJ IDEA

### Usage

#### Initial Setup

1. Verify BazBOM CLI is accessible:
   ```bash
   bazbom --version
   ```

2. Open a Java project in IntelliJ IDEA (Maven, Gradle, or Bazel)

3. Plugin will automatically detect build system and start scanning

#### Viewing Vulnerabilities

**Tool Window:**
- Click **BazBOM** in the right sidebar
- View dependency tree with security indicators:
  - ✅ Green: No vulnerabilities
  - ⚠️ Yellow: Medium severity
  - 🔴 Red: High/Critical severity

**Inline Warnings:**
- Vulnerabilities appear as editor warnings in build files
- Hover for full CVE details
- Click on warning for quick actions

#### Quick Fixes

1. Position cursor on vulnerable dependency (red/yellow underline)
2. Press `Alt+Enter` (Windows/Linux) or `⌥+Return` (macOS)
3. Select "Upgrade to safe version X.X.X"
4. Plugin automatically:
   - Updates version in build file
   - Reloads build system
   - Runs tests to verify safety
   - Shows notification when complete

#### Manual Scanning

- **Tools** → **Scan with BazBOM**
- **Tools** → **Sync BazBOM Advisory Database**
- Or use keyboard shortcuts (configurable in Settings → Keymap)

### Configuration

Go to **Settings/Preferences** → **Tools** → **BazBOM**

Available settings:
- **Enable real-time scanning**: Scan on file save (default: on)
- **Auto-scan on project open**: Scan when opening project (default: off)
- **Severity thresholds**: Which severity levels to display
- **BazBOM CLI path**: Custom path to bazbom binary
- **Policy file**: Path to custom policy file

### Troubleshooting

**Plugin not loading:**
- Check IntelliJ version (requires 2023.3+)
- Verify plugin is enabled: **Settings** → **Plugins** → **Installed** → Check "BazBOM Security Scanner"
- Check logs: **Help** → **Show Log in Finder/Explorer**

**No scan results:**
- Ensure BazBOM CLI is installed: `bazbom --version`
- Sync advisory database: `bazbom db sync`
- Check project has supported build files (`pom.xml`, `build.gradle`, etc.)
- Check plugin logs for errors

**Scans are slow:**
- First scan takes 30-60 seconds (builds dependency graph)
- Subsequent scans use cache (<10 seconds)
- Disable reachability analysis in settings for faster scans

**Quick fixes not working:**
- Ensure project has tests configured
- Check network connectivity (for downloading new dependency versions)
- View test output in **Run** tool window

---

## VS Code Extension

Lightweight real-time vulnerability scanning powered by the BazBOM Language Server Protocol (LSP) implementation.

### Features

- ✅ **Real-time Scanning** - Automatically scans on file save
- ✅ **Inline Diagnostics** - Shows vulnerabilities as editor problems
- ✅ **Fast Mode** - Sub-10-second scans (skips reachability analysis)
- ✅ **Privacy-First** - All scanning happens locally
- ✅ **Multi-Platform** - Works on macOS, Linux, Windows

### Requirements

- **VS Code**: 1.85 or later
- **BazBOM CLI & LSP**: Both must be installed

### Installation

#### From VS Code Marketplace (Coming Soon)

1. Open VS Code
2. Go to **Extensions** (Ctrl+Shift+X / Cmd+Shift+X)
3. Search for "BazBOM Security Scanner"
4. Click **Install**

#### Manual Installation (Development/Beta)

1. Build the LSP server:
   ```bash
   cd crates/bazbom-lsp
   cargo build --release
   sudo cp ../../target/release/bazbom-lsp /usr/local/bin/
   ```

2. Build the extension:
   ```bash
   cd crates/bazbom-vscode-extension
   npm install
   npm run compile
   ```

3. Package the extension:
   ```bash
   npx vsce package
   ```

4. Install in VS Code:
   - **Extensions** → **...** → **Install from VSIX...**
   - Select `bazbom-1.0.0.vsix`

### Usage

#### Initial Setup

1. Verify BazBOM CLI is accessible:
   ```bash
   bazbom --version
   bazbom-lsp --version
   ```

2. Open a Java project (Maven, Gradle, or Bazel)

3. Open `pom.xml`, `build.gradle`, or `BUILD.bazel`

4. Extension will automatically scan on file save

#### Viewing Vulnerabilities

Vulnerabilities appear in:
- **Editor**: Inline squiggles (red/yellow/blue)
- **Problems Panel**: View → Problems (Ctrl+Shift+M / Cmd+Shift+M)

Diagnostic format:
```
CVE-2021-44228 (Critical): Remote code execution in org.apache.logging.log4j:log4j-core - Fixed in version 2.21.1
```

Severity levels:
- **ERROR (red)**: Critical and High severity
- **WARNING (yellow)**: Medium severity
- **INFO (blue)**: Low severity

#### Commands

Access via Command Palette (Ctrl+Shift+P / Cmd+Shift+P):
- **BazBOM: Scan Project** - Manually trigger scan
- **BazBOM: Sync Advisory Database** - Update vulnerability database

### Configuration

Add to **Settings** (JSON):
```json
{
  "bazbom.lspPath": "/usr/local/bin/bazbom-lsp",
  "bazbom.enableRealTimeScanning": true,
  "bazbom.scanOnOpen": false,
  "bazbom.severityThreshold": "medium"
}
```

Or via UI: **Settings** → **Extensions** → **BazBOM**

### Troubleshooting

**Extension not activating:**
- Check LSP server is installed: `bazbom-lsp --version`
- Check VS Code Output panel: **View** → **Output** → **BazBOM Language Server**
- Check file is a supported build file

**No diagnostics appearing:**
- Ensure BazBOM CLI is installed: `bazbom --version`
- Sync advisory database: `bazbom db sync`
- Save the file to trigger scan (diagnostics appear on save)
- Check Output panel for errors

**Scans are slow:**
- Extension uses fast mode by default (skips reachability)
- For full analysis, use CLI: `bazbom scan --reachability`

---

## Other LSP-Compatible Editors

BazBOM's LSP server works with any editor that supports the Language Server Protocol.

### Vim/Neovim (coc.nvim)

Add to `:CocConfig`:
```json
{
  "languageserver": {
    "bazbom": {
      "command": "/usr/local/bin/bazbom-lsp",
      "filetypes": ["xml", "groovy", "kotlin", "bzl"],
      "rootPatterns": ["pom.xml", "build.gradle", "WORKSPACE", "MODULE.bazel"]
    }
  }
}
```

### Vim/Neovim (native LSP)

Add to `init.lua`:
```lua
vim.lsp.start({
  name = 'bazbom',
  cmd = {'/usr/local/bin/bazbom-lsp'},
  root_dir = vim.fs.dirname(vim.fs.find({'pom.xml', 'build.gradle', 'WORKSPACE'}, { upward = true })[1]),
})
```

### Emacs (lsp-mode)

Add to `init.el`:
```elisp
(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection "/usr/local/bin/bazbom-lsp")
  :major-modes '(xml-mode groovy-mode kotlin-mode)
  :server-id 'bazbom))
```

### Sublime Text (LSP package)

Add to LSP settings:
```json
{
  "clients": {
    "bazbom": {
      "enabled": true,
      "command": ["/usr/local/bin/bazbom-lsp"],
      "selector": "source.xml | source.groovy | source.kotlin | source.bzl"
    }
  }
}
```

---

## Performance Expectations

| Project Size | First Scan | Subsequent Scans | Notes |
|--------------|-----------|------------------|-------|
| Small (<50 deps) | 5-10s | <1s | Instant feedback |
| Medium (50-200 deps) | 15-30s | <5s | Suitable for real-time |
| Large (200+ deps) | 30-60s | <10s | Use caching |
| Monorepos (1000+ deps) | 60-120s | 10-30s | Consider selective scanning |

**Performance Tips:**
- Fast mode (default) skips reachability analysis (10x faster)
- Results are cached until build file changes
- All operations run asynchronously (non-blocking)
- Use pre-commit hooks for gating rather than continuous scanning

---

## Privacy & Security

All BazBOM IDE integrations follow these principles:

- ✅ **100% Local** - All scanning happens on your machine
- ✅ **Zero Telemetry** - No data sent to external servers
- ✅ **Offline-Capable** - Works without internet (after initial `bazbom db sync`)
- ✅ **Open Source** - Auditable code (MIT license)
- ✅ **No Account Required** - No registration or authentication

Advisory database sources (public):
- OSV (Open Source Vulnerabilities)
- NVD (National Vulnerability Database)
- GHSA (GitHub Security Advisories)
- CISA KEV (Known Exploited Vulnerabilities)

---

## Next Steps

- **Configure Policy**: Create `bazbom.yml` to customize severity thresholds
- **Set Up Pre-Commit Hooks**: `bazbom install-hooks` to gate commits
- **Explore Remediation**: `bazbom fix --suggest` for fix guidance
- **Read Full Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs

## Support

- **GitHub Issues**: https://github.com/cboyd0319/BazBOM/issues
- **Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs
- **Email**: support@bazbom.io

---

**Last Updated**: 2025-10-31  
**Plugin Versions**: IntelliJ 1.0.0, VS Code 1.0.0, LSP 0.2.1
