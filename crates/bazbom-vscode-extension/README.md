# BazBOM VS Code Extension

Real-time vulnerability scanning for Java projects using Maven, Gradle, or Bazel. Get instant security feedback as you code.

## Features

- **Real-time Scanning**: Automatically scans `pom.xml`, `build.gradle`, and `BUILD.bazel` files on save
- **Inline Diagnostics**: Shows vulnerabilities as editor problems with severity indicators
- **Fast Mode**: Uses optimized scanning for sub-10-second feedback
- **Privacy-First**: All scanning happens locally, no data sent to external servers
- **Build System Support**: Works with Maven, Gradle, and Bazel projects

## Requirements

- **BazBOM CLI**: Must be installed and available in PATH (build from source)
  ```bash
  git clone https://github.com/cboyd0319/BazBOM.git ~/src/BazBOM
  cd ~/src/BazBOM
  cargo build --release -p bazbom
  sudo install -m 0755 target/release/bazbom /usr/local/bin/bazbom  # or add target/release to PATH
  ```

- **Advisory Database**: Sync the advisory database before first use
  ```bash
  bazbom db sync
  ```

## Extension Settings

This extension contributes the following settings:

- `bazbom.lspPath`: Path to bazbom-lsp binary (default: looks in PATH)
- `bazbom.enableRealTimeScanning`: Enable/disable real-time scanning (default: true)
- `bazbom.scanOnOpen`: Automatically scan when opening a project (default: false)
- `bazbom.severityThreshold`: Minimum severity to display (default: medium)

## Commands

- **BazBOM: Scan Project** - Manually trigger a scan of the current file
- **BazBOM: Sync Advisory Database** - Update the local vulnerability database

## Usage

1. Open a Java project (Maven, Gradle, or Bazel)
2. Open `pom.xml`, `build.gradle`, or `BUILD.bazel`
3. The extension will automatically scan on file save
4. Vulnerabilities will appear as problems in the editor

## Diagnostic Format

Vulnerabilities are shown with the following format:

```
CVE-2021-44228 (Critical): Remote code execution in org.apache.logging.log4j:log4j-core - Fixed in version 2.21.1
```

- **Severity Levels**:
  - ERROR (red): Critical and High severity
  - WARNING (yellow): Medium severity
  - INFO (blue): Low severity

## Performance

- **Scan Time**: <10 seconds for typical projects (fast mode)
- **Caching**: Results are cached to minimize scan frequency
- **Async**: All operations run in the background to avoid blocking the editor

## Troubleshooting

**Extension not activating:**
- Check that bazbom-lsp is installed: `bazbom-lsp --version`
- Check VS Code output panel: View → Output → BazBOM Language Server

**No diagnostics appearing:**
- Ensure bazbom CLI is installed: `bazbom --version`
- Sync advisory database: `bazbom db sync`
- Check file is a supported build file (pom.xml, build.gradle, BUILD)
- Save the file to trigger scan

**Scans are slow:**
- Extension uses fast mode by default (skips reachability analysis)
- For full scans with reachability, use CLI: `bazbom scan --reachability`

## Privacy & Security

- All scanning happens **locally** on your machine
- No data is sent to external servers
- No telemetry or tracking
- Advisory database synced from public sources (OSV, NVD, GHSA)

## License

MIT - See LICENSE file for details

## Support

- GitHub Issues: https://github.com/cboyd0319/BazBOM/issues
- Documentation: https://github.com/cboyd0319/BazBOM/tree/main/docs
