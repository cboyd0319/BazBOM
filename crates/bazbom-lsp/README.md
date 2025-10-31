# BazBOM Language Server (LSP)

Language Server Protocol implementation for BazBOM vulnerability scanning. Provides real-time security feedback in IDEs and editors that support LSP.

## Features

- **Real-time vulnerability scanning** - Scans on file save for `pom.xml`, `build.gradle`, and `BUILD.bazel` files
- **Fast mode** - Uses `bazbom scan --fast` for sub-10-second scans
- **Diagnostic publishing** - Shows vulnerabilities as editor diagnostics (errors, warnings, info)
- **Cross-editor support** - Works with any LSP-compatible editor (VS Code, Vim, Emacs, Sublime Text, etc.)

## Installation

Build the LSP server:

```bash
cargo build --release --package bazbom-lsp
```

The binary will be available at `target/release/bazbom-lsp`.

## Editor Integration

### VS Code

Install the [BazBOM extension](../bazbom-vscode-extension/) from the VS Code marketplace, or configure manually:

```json
{
  "bazbom.lspPath": "/path/to/bazbom-lsp",
  "bazbom.enableRealTimeScanning": true
}
```

### Vim/Neovim

Using [coc.nvim](https://github.com/neoclide/coc.nvim):

```json
{
  "languageserver": {
    "bazbom": {
      "command": "/path/to/bazbom-lsp",
      "filetypes": ["xml", "groovy", "kotlin", "bzl"],
      "rootPatterns": ["pom.xml", "build.gradle", "WORKSPACE", "MODULE.bazel"]
    }
  }
}
```

### Emacs

Using [lsp-mode](https://github.com/emacs-lsp/lsp-mode):

```elisp
(with-eval-after-load 'lsp-mode
  (add-to-list 'lsp-language-id-configuration '(xml-mode . "xml"))
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "/path/to/bazbom-lsp")
    :major-modes '(xml-mode groovy-mode kotlin-mode)
    :server-id 'bazbom)))
```

## How It Works

1. **File Watch** - Monitors `pom.xml`, `build.gradle`, `build.gradle.kts`, `BUILD`, and `BUILD.bazel` files
2. **On Save/Open** - Triggers `bazbom scan --fast` when files are saved or opened
3. **Parse Results** - Reads `sca_findings.json` from scan output
4. **Publish Diagnostics** - Converts vulnerabilities to LSP diagnostics with severity levels
5. **Display** - Editor shows inline warnings/errors at relevant locations

## Diagnostic Format

Diagnostics include:
- **Severity**: ERROR (Critical/High), WARNING (Medium), INFORMATION (Low)
- **Code**: CVE identifier (e.g., CVE-2021-44228)
- **Message**: Full description including package name, current version, and fixed version
- **Source**: "BazBOM"

Example:
```
CVE-2021-44228 (Critical): Remote code execution via JNDI in org.apache.logging.log4j:log4j-core - Fixed in version 2.21.1
```

## Performance

- **Fast mode**: <10 seconds for typical projects (skips reachability analysis)
- **Caching**: Scan results are cached per file
- **Async**: All operations are asynchronous to avoid blocking the editor

## Requirements

- `bazbom` CLI must be installed and in PATH
- Advisory cache must be synced: `bazbom db sync`

## Troubleshooting

**No diagnostics appear:**
- Ensure `bazbom` is installed: `bazbom --version`
- Check advisory cache exists: `ls .bazbom/cache`
- View LSP logs in your editor's LSP log output

**Scans are slow:**
- LSP uses `--fast` mode by default (skips reachability)
- For full scans, use CLI: `bazbom scan --reachability`

**Diagnostics not updating:**
- Save the file to trigger a scan
- Check that file is a supported build file (pom.xml, build.gradle, BUILD)

## Development

Run tests:
```bash
cargo test --package bazbom-lsp
```

Run with debug logging:
```bash
RUST_LOG=debug bazbom-lsp
```

## License

MIT - See LICENSE file for details
