# IDE Setup Guide

This guide covers detailed setup instructions for BazBOM IDE integrations.

## Overview

BazBOM provides native IDE integration for:
- **IntelliJ IDEA** (Community & Ultimate)
- **VS Code** (via Language Server Protocol)
- **Other LSP-compatible editors** (Vim, Emacs, Sublime Text)

## Prerequisites

Before installing IDE plugins, ensure BazBOM CLI is installed and working.

See [USAGE.md](../USAGE.md#installation) for CLI installation instructions.

## IntelliJ IDEA Plugin

### Installation

Build and install the plugin from source:

```bash
cd crates/bazbom-intellij-plugin
./gradlew buildPlugin
```

The plugin ZIP will be created at `build/distributions/bazbom-intellij-plugin-1.0.0.zip`.

Install in IntelliJ IDEA:
1. **Settings** → **Plugins** → **⚙️** → **Install Plugin from Disk...**
2. Select the ZIP file
3. Restart IDE

### Configuration

After installation:

1. **Settings** → **Tools** → **BazBOM**
2. Set **BazBOM CLI Path** if not in system PATH
3. Configure scanning options and severity thresholds
4. Optional: Set custom policy file path

### Features

- Dependency tree visualization with security status
- Real-time vulnerability highlighting in build files
- One-click quick fixes with automatic testing
- Auto-scan on project open and file save

See [USAGE.md](../USAGE.md#intellij-idea-plugin) for complete documentation.

## VS Code Extension

### Installation

Build and install the extension from source:

```bash
# Build LSP server
cd crates/bazbom-lsp
cargo build --release

# Build extension
cd ../bazbom-vscode-extension
npm install
npm run compile
npx vsce package
```

Install in VS Code:
1. **Extensions** → **⋯** → **Install from VSIX...**
2. Select `bazbom-*.vsix`
3. Reload VS Code

### Configuration

Configure in VS Code settings:

```json
{
  "bazbom.lspPath": "/path/to/bazbom-lsp",
  "bazbom.enableRealTimeScanning": true,
  "bazbom.minimumSeverity": "medium"
}
```

### Features

- Real-time diagnostics in Problems panel
- Inline vulnerability warnings
- Quick fix code actions
- Commands for manual scanning

See [USAGE.md](../USAGE.md#vs-code-extension) for complete documentation.

## Troubleshooting

For troubleshooting help, see [USAGE.md](../USAGE.md#troubleshooting).

## Additional Resources

- [USAGE.md](../USAGE.md) - Complete usage guide
- [Phase 4 Specification](../copilot/PHASE_4_DEVELOPER_EXPERIENCE.md)
- [Phase 4 Progress](../copilot/PHASE_4_PROGRESS.md)
