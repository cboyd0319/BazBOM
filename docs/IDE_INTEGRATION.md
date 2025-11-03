# BazBOM IDE Integration

**Status:** Beta (95% Complete)  
**Last Updated:** 2025-11-03

BazBOM now integrates directly into your IDE, providing real-time vulnerability scanning and one-click fixes without leaving your development environment.

---

## Overview

### What's New

BazBOM Phase 4 brings the security scanning experience into your IDE:

- **Real-time vulnerability warnings** as you edit build files
- **One-click fixes** to upgrade vulnerable dependencies
- **Automated testing** after upgrades to ensure nothing breaks
- **Multi-build-system support**: Maven, Gradle, and Bazel
- **Privacy-first**: All scanning happens locally

### Supported IDEs

- **IntelliJ IDEA** (Community & Ultimate) - 95% complete
- **VS Code** - 95% complete
- **Any LSP-compatible editor** (via `bazbom-lsp`) - 95% complete

---

## IntelliJ IDEA Plugin

### Features

#### 1. Dependency Tree Visualization

A side panel shows all project dependencies with security status:

- ✅ **Secure** dependencies (green)
- ⚠️ **Vulnerable** dependencies (yellow/red)
- Vulnerability counts per dependency
- Grouped by scope (compile, test, runtime)
- Scan and refresh buttons

#### 2. Real-Time Vulnerability Highlighting

Inline warnings appear directly in your build files:

**Maven (`pom.xml`):**
```xml
<dependency>
    <groupId>org.apache.logging.log4j</groupId>
    <artifactId>log4j-core</artifactId>
    <version>2.17.0</version>  ⚠️ CVE-2021-44832 (MEDIUM)
</dependency>
```

**Gradle (`build.gradle` or `build.gradle.kts`):**
```groovy
implementation 'log4j:log4j-core:2.17.0'  ⚠️ CVE-2021-44832 (MEDIUM)
```

**Bazel (`BUILD.bazel`, `WORKSPACE`, `MODULE.bazel`):**
```python
"org.apache.logging.log4j:log4j-core:2.17.0"  ⚠️ CVE-2021-44832 (MEDIUM)
```

Severity levels:
- 🔴 **CRITICAL** - Red error underline
- 🟠 **HIGH** - Orange warning underline
- 🟡 **MEDIUM** - Yellow weak warning
- 🔵 **LOW** - Blue info

Special indicators:
- **(CISA KEV)** - Known Exploited Vulnerability
- **(Reachable)** - Code path reaches vulnerable code

#### 3. One-Click Quick Fixes

Press `Alt+Enter` (or `⌥⏎` on Mac) on a highlighted vulnerability:

```
⚠️ log4j-core 2.17.0 has CVE-2021-44832
💡 Upgrade to safe version 2.21.1
```

When you select the fix:
1. ✅ Version is updated in your build file
2. ✅ Maven/Gradle project reloads automatically
3. ✅ Tests run in the background
4. ✅ Notification shows success or failure

**Notifications:**
- ✅ Success: "Upgraded log4j-core to 2.21.1. All tests passed."
- ⚠️ Warning: "Upgraded log4j-core to 2.21.1 but tests failed. Please review and fix."
- ❌ Error: "Failed to upgrade log4j-core: [reason]"

### Installation

#### Prerequisites

1. **BazBOM CLI** must be installed and in your PATH:
   ```bash
   curl -fsSL https://bazbom.io/install.sh | bash
   ```

2. **Advisory database** must be synced:
   ```bash
   bazbom db sync
   ```

#### Install Plugin

**Option 1: From Marketplace (Coming Soon)**
1. Open IntelliJ IDEA
2. Go to **Settings/Preferences** → **Plugins**
3. Search for "BazBOM Security Scanner"
4. Click **Install**
5. Restart IDE

**Option 2: From Source (Current)**
```bash
# Clone repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM/crates/bazbom-intellij-plugin

# Build plugin
./gradlew buildPlugin

# Install manually
# Go to Settings → Plugins → ⚙️ → Install Plugin from Disk
# Select: build/distributions/bazbom-intellij-plugin-*.zip
```

### Configuration

Go to **Settings/Preferences** → **Tools** → **BazBOM**:

- **Enable real-time scanning** - Scan on file save (default: on)
- **Show inline warnings** - Display vulnerabilities in editor (default: on)
- **Severity thresholds** - Which severities to show (default: CRITICAL, HIGH, MEDIUM)
- **Policy file** - Path to `bazbom.yml` policy (default: `bazbom.yml`)
- **BazBOM CLI path** - Path to `bazbom` binary (default: auto-detect)

### Usage

#### Scan Your Project

**Automatic:**
- Open a project with `pom.xml`, `build.gradle`, or `BUILD.bazel`
- Plugin scans automatically on project open (if configured)

**Manual:**
- **Tools** → **Scan with BazBOM**
- Or click the **Scan** button in the BazBOM tool window

#### View Dependencies

1. Open the **BazBOM** tool window (right sidebar)
2. See dependency tree with security status
3. Click on dependencies to see details

#### Fix Vulnerabilities

1. Open a build file with a highlighted vulnerability
2. Place cursor on the highlighted line
3. Press `Alt+Enter` (or `⌥⏎` on Mac)
4. Select "Upgrade to safe version X"
5. Wait for tests to complete
6. Review notification

#### Update Advisory Database

**Tools** → **Sync BazBOM Advisory Database**

Or run manually:
```bash
bazbom db sync
```

---

## VS Code Extension

### Features

- Real-time vulnerability diagnostics via LSP
- Quick fix code actions (Ctrl+. or Cmd+.)
- File watching for build files
- Commands:
  - **BazBOM: Scan Project**
  - **BazBOM: Sync Advisory Database**

### Installation

#### Prerequisites

Same as IntelliJ (BazBOM CLI + advisory database).

#### Install Extension

**Option 1: From Marketplace (Coming Soon)**
1. Open VS Code
2. Go to **Extensions** (Ctrl+Shift+X)
3. Search for "BazBOM Security Scanner"
4. Click **Install**

**Option 2: From Source (Current)**
```bash
# Clone repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM/crates/bazbom-vscode-extension

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package extension
npx vsce package

# Install manually
code --install-extension bazbom-*.vsix
```

### Configuration

Open **Settings** (Ctrl+,) and search for "BazBOM":

```json
{
  "bazbom.lspPath": "bazbom-lsp",
  "bazbom.enableRealTimeScanning": true,
  "bazbom.severityThreshold": "medium",
  "bazbom.policyFile": "bazbom.yml"
}
```

### Usage

#### Scan Your Project

**Automatic:**
- Open a workspace with build files
- Extension activates and scans on file save

**Manual:**
- Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
- Type "BazBOM: Scan Project"
- Press Enter

#### View Vulnerabilities

- Vulnerabilities appear in the **Problems** panel (Ctrl+Shift+M)
- Inline squiggles in build files
- Hover for details

#### Fix Vulnerabilities

1. Place cursor on a vulnerability
2. Press `Ctrl+.` (or `Cmd+.` on Mac)
3. Select "Upgrade to safe version X"

---

## Language Server Protocol (LSP)

The `bazbom-lsp` server can be used with any LSP-compatible editor:

- **Vim/Neovim** (via coc.nvim, nvim-lsp)
- **Emacs** (via lsp-mode, eglot)
- **Sublime Text** (via LSP package)
- **Atom** (via atom-ide-ui)

### Installation

```bash
# Build LSP server
cargo build --release -p bazbom-lsp

# Copy to PATH
sudo cp target/release/bazbom-lsp /usr/local/bin/
```

### Configuration

#### Neovim (nvim-lspconfig)

```lua
require'lspconfig'.bazbom.setup{
  cmd = { "bazbom-lsp" },
  filetypes = { "xml", "groovy", "kotlin", "starlark" },
  root_dir = function(fname)
    return vim.fn.getcwd()
  end,
}
```

#### Emacs (lsp-mode)

```elisp
(require 'lsp-mode)

(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection "bazbom-lsp")
  :major-modes '(nxml-mode groovy-mode kotlin-mode)
  :server-id 'bazbom))

(add-hook 'nxml-mode-hook #'lsp)
(add-hook 'groovy-mode-hook #'lsp)
```

---

## Troubleshooting

### Plugin doesn't show vulnerabilities

1. **Check BazBOM CLI is installed:**
   ```bash
   bazbom --version
   ```

2. **Check advisory database is synced:**
   ```bash
   ls ~/.bazbom/advisories/
   ```

3. **Run manual scan:**
   ```bash
   bazbom scan --out-dir .bazbom/scan-output .
   ```

4. **Check scan output:**
   ```bash
   cat .bazbom/scan-output/sca_findings.json
   ```

5. **Check plugin logs:**
   - IntelliJ: **Help** → **Show Log in Finder/Explorer**
   - VS Code: **View** → **Output** → Select "BazBOM Language Server"

### Quick fix doesn't work

1. **Check Maven/Gradle is working:**
   ```bash
   mvn --version  # or
   gradle --version
   ```

2. **Try manual upgrade:**
   - Update version in build file
   - Run tests: `mvn test` or `gradle test`

3. **Check notification messages** for specific error details

### Tests fail after upgrade

This is intentional behavior! BazBOM detected that the upgrade breaks your application.

**What to do:**
1. Review test failures
2. Fix compatibility issues
3. Re-run tests manually
4. Commit when passing

### Performance is slow

1. **Use fast mode** (skips reachability analysis):
   ```bash
   bazbom scan --fast .
   ```

2. **Exclude directories** in `.gitignore` or `.bazbomignore`

3. **Increase cache size** in plugin settings

---

## Development

### Building from Source

```bash
# IntelliJ Plugin
cd crates/bazbom-intellij-plugin
./gradlew build

# VS Code Extension
cd crates/bazbom-vscode-extension
npm install && npm run compile

# LSP Server
cd crates/bazbom-lsp
cargo build --release
```

### Running Tests

```bash
# IntelliJ Plugin
./gradlew test

# LSP Server
cargo test -p bazbom-lsp
```

### Debugging

#### IntelliJ Plugin
```bash
./gradlew runIde
```

#### VS Code Extension
1. Open `crates/bazbom-vscode-extension` in VS Code
2. Press F5 to launch Extension Development Host

---

## Roadmap

### Completed ✅
- [x] IntelliJ plugin with Maven/Gradle/Bazel support
- [x] VS Code extension with LSP
- [x] Real-time vulnerability highlighting
- [x] One-click quick fixes
- [x] Automated testing after upgrades
- [x] Notification system

### In Progress 🔄
- [ ] Manual testing with real projects
- [ ] Performance optimization
- [ ] Marketplace publishing

### Planned 📅
- [ ] User analytics (privacy-preserving, opt-in)
- [ ] Enhanced settings panels
- [ ] Vulnerability details panel with links
- [ ] Status bar integration
- [ ] Eclipse plugin
- [ ] Android Studio optimizations

---

## Support

- **Documentation:** [docs/](../docs/)
- **Issues:** [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
- **Discussions:** [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)
- **Security:** [SECURITY.md](../SECURITY.md)

---

**Ready to try it?** Install the plugin and start scanning! 🚀
