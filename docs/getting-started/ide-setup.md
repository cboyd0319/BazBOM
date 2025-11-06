# IDE Setup Quick Start

Get BazBOM running in your IDE in 5 minutes.

---

## Prerequisites

### 1. Install BazBOM CLI

**macOS/Linux:**
```bash
curl -fsSL https://bazbom.io/install.sh | bash
```

**Or with Homebrew:**
```bash
brew install bazbom
```

**Or build from source:**
```bash
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM
cargo build --release
sudo cp target/release/bazbom /usr/local/bin/
```

**Verify installation:**
```bash
bazbom --version
```

### 2. Sync Advisory Database

```bash
bazbom db sync
```

This downloads the latest vulnerability data (OSV, NVD, GHSA, KEV).

---

## IntelliJ IDEA

### Quick Install (From Source)

```bash
# 1. Clone repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM/crates/bazbom-intellij-plugin

# 2. Build plugin
./gradlew buildPlugin

# 3. Locate plugin file
ls build/distributions/bazbom-intellij-plugin-*.zip
```

**Install in IntelliJ:**
1. Open IntelliJ IDEA
2. **Settings/Preferences** → **Plugins**
3. Click  → **Install Plugin from Disk...**
4. Select `build/distributions/bazbom-intellij-plugin-*.zip`
5. Click **OK** and **Restart IDE**

### First Scan

1. Open a Java project with `pom.xml`, `build.gradle`, or `BUILD.bazel`
2. **Tools** → **Scan with BazBOM**
3. Wait for scan to complete (usually 10-30 seconds)
4. Check the **BazBOM** tool window (right sidebar)

### Try a Quick Fix

1. Open `pom.xml` (or `build.gradle`)
2. Look for a red/yellow underline on a dependency version
3. Place cursor on it
4. Press `Alt+Enter` (or `⌥⏎` on Mac)
5. Select "Upgrade to safe version X"
6. Wait for notification

---

## VS Code

### Quick Install (From Source)

```bash
# 1. Clone repository
git clone https://github.com/cboyd0319/BazBOM.git

# 2. Build LSP server (required)
cd BazBOM
cargo build --release -p bazbom-lsp
sudo cp target/release/bazbom-lsp /usr/local/bin/

# 3. Build VS Code extension
cd crates/bazbom-vscode-extension
npm install
npm run compile
npx vsce package

# 4. Install extension
code --install-extension bazbom-*.vsix
```

### First Scan

1. Open a workspace with Java project
2. Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac)
3. Type "BazBOM: Scan Project"
4. Press Enter
5. Check **Problems** panel (`Ctrl+Shift+M`)

### Try a Quick Fix

1. Open `pom.xml` (or `build.gradle`)
2. Look for squiggly underline on a dependency
3. Place cursor on it
4. Press `Ctrl+.` (or `Cmd+.` on Mac)
5. Select "Upgrade to safe version X"

---

## Vim/Neovim

### Using nvim-lspconfig

```bash
# 1. Install LSP server
cargo install --git https://github.com/cboyd0319/BazBOM.git bazbom-lsp

# 2. Configure Neovim
# Add to ~/.config/nvim/init.lua
```

```lua
require'lspconfig'.bazbom.setup{
  cmd = { "bazbom-lsp" },
  filetypes = { "xml", "groovy", "kotlin" },
}
```

### First Scan

1. Open a `pom.xml` file
2. LSP server starts automatically
3. Diagnostics appear in a few seconds

---

## Troubleshooting

### "bazbom command not found"

```bash
# Check if in PATH
which bazbom

# If not, add to PATH (add to ~/.bashrc or ~/.zshrc)
export PATH="$HOME/.bazbom/bin:$PATH"
```

### "No vulnerabilities found" but there should be

```bash
# 1. Sync database
bazbom db sync

# 2. Run manual scan to debug
bazbom scan --out-dir .bazbom/scan-output .

# 3. Check output
cat .bazbom/scan-output/sca_findings.json
```

### IntelliJ plugin doesn't load

1. Check IDE version: Must be 2023.3+
2. Check plugin is enabled: **Settings** → **Plugins**
3. Check logs: **Help** → **Show Log in Finder/Explorer**

### VS Code extension doesn't work

1. Check LSP server is installed:
   ```bash
   which bazbom-lsp
   ```

2. Check output panel:
   - **View** → **Output**
   - Select "BazBOM Language Server" from dropdown

3. Restart extension:
   - Press `Ctrl+Shift+P`
   - Type "Developer: Reload Window"

---

## Next Steps

- Read [../integrations/ide/ide-integration.md](../integrations/ide/ide-integration.md) for detailed features
- Configure policy in `bazbom.yml`
- Set up [pre-commit hooks](../../user-guide/usage.md#pre-commit-hooks)
- Try `bazbom fix --apply` for automated remediation

---

**Need help?** Open an issue at https://github.com/cboyd0319/BazBOM/issues
