# BazBOM IntelliJ IDEA Plugin

Real-time vulnerability scanning for Java projects using Maven, Gradle, or Bazel. Provides dependency tree visualization, inline warnings, and one-click fixes.

## Features

- **Dependency Tree Visualization**: Side panel showing all dependencies with security status
- **Real-time Vulnerability Highlighting**: Inline warnings in `pom.xml`, `build.gradle`, `BUILD.bazel`
- **One-Click Quick Fixes**: Automatically upgrade vulnerable dependencies
- **Automated Testing**: Runs tests after applying fixes to ensure safety
- **Build System Auto-Detection**: Supports Maven, Gradle, and Bazel
- **Privacy-First**: All scanning happens locally, no data sent to external servers

## Requirements

- **IntelliJ IDEA**: 2023.3 or later (Community or Ultimate Edition)
- **BazBOM CLI**: Must be installed and available in PATH
- **Advisory Database**: Sync before first use with `bazbom db sync`

## Installation

### From JetBrains Marketplace (Coming Soon)

1. Open IntelliJ IDEA
2. Go to **Settings/Preferences** → **Plugins** → **Marketplace**
3. Search for "BazBOM Security Scanner"
4. Click **Install**
5. Restart IntelliJ IDEA

### Manual Installation (Development)

1. Build the plugin:
   ```bash
   cd crates/bazbom-intellij-plugin
   ./gradlew buildPlugin
   ```

2. Install in IntelliJ IDEA:
   - Go to **Settings/Preferences** → **Plugins** → **** → **Install Plugin from Disk...**
   - Select `build/distributions/bazbom-intellij-plugin-1.0.0.zip`
   - Restart IntelliJ IDEA

## Usage

### Initial Setup

1. Ensure BazBOM CLI is installed:
   ```bash
   bazbom --version
   ```

2. Sync advisory database:
   ```bash
   bazbom db sync
   ```

3. Open a Java project in IntelliJ IDEA

### Scanning Projects

**Automatic Scanning:**
- Plugin scans when you open build files (`pom.xml`, `build.gradle`, `BUILD.bazel`)
- Scans on file save (configurable)

**Manual Scanning:**
- **Tools** → **Scan with BazBOM**
- Or use keyboard shortcut (configurable)

### Viewing Results

**Tool Window:**
1. Click **BazBOM** in the right sidebar
2. View dependency tree with security indicators:
   -  Green: No vulnerabilities
   -  Yellow: Medium severity
   -  Red: High/Critical severity

**Inline Warnings:**
- Vulnerabilities appear as editor warnings in build files
- Hover for details
- Click on warning for quick actions

### Quick Fixes

1. Position cursor on vulnerable dependency
2. Press `Alt+Enter` (Windows/Linux) or `⌥+Return` (macOS)
3. Select "Upgrade to safe version X.X.X"
4. Plugin updates version and runs tests
5. Get notification when complete

## Configuration

Go to **Settings/Preferences** → **Tools** → **BazBOM**

Available settings:
- **Enable real-time scanning**: Scan on file save
- **Auto-scan on project open**: Scan when opening project
- **Severity thresholds**: Which severity levels to display
- **BazBOM CLI path**: Custom path to bazbom binary
- **Policy file**: Path to custom policy file

## Build System Support

### Maven (`pom.xml`)
- Full dependency graph
- Scope tracking (compile, test, runtime)
- Property resolution
- Parent POM support

### Gradle (`build.gradle`, `build.gradle.kts`)
- Multi-configuration support
- Version catalog support
- Android variant support
- Shadow plugin detection

### Bazel (`BUILD.bazel`)
- `java_*` rules support
- `maven_install` support
- Workspace dependencies
- Aspect-based analysis

## Performance

- **First Scan**: 30-60 seconds (full analysis)
- **Subsequent Scans**: <10 seconds (cached results)
- **Background Operation**: All scans run asynchronously
- **No UI Blocking**: Editor remains responsive during scans

## Troubleshooting

**Plugin not loading:**
- Check IntelliJ version (2023.3+)
- Check plugin is enabled: **Settings** → **Plugins** → **Installed**
- Check logs: **Help** → **Show Log in Finder/Explorer**

**No scan results:**
- Ensure BazBOM CLI is installed: `bazbom --version`
- Sync advisory database: `bazbom db sync`
- Check project has supported build files
- View plugin logs for errors

**Scans are slow:**
- First scan is slower (builds dependency graph)
- Subsequent scans use cache
- Disable reachability analysis for faster scans

**Quick fixes not working:**
- Ensure tests exist in project
- Check network connectivity (for downloading dependencies)
- View test output in **Run** tool window

## Development

### Building

```bash
cd crates/bazbom-intellij-plugin
./gradlew build
```

### Testing

```bash
./gradlew test
```

### Running in Development

```bash
./gradlew runIde
```

This launches a new IntelliJ IDEA instance with the plugin installed.

## Architecture

```
bazbom-intellij-plugin/
├── src/main/kotlin/io/bazbom/intellij/
│   ├── BazBomPlugin.kt              # Main entry point
│   ├── actions/                     # Menu actions
│   │   ├── ScanProjectAction.kt     # Manual scan trigger
│   │   └── SyncDatabaseAction.kt    # Sync advisory DB
│   ├── settings/                    # Settings UI
│   │   └── BazBomConfigurable.kt    # Settings panel
│   ├── toolwindow/                  # Dependency tree UI
│   │   └── BazBomToolWindowFactory.kt
│   ├── services/                    # Plugin services
│   │   └── BazBomProjectService.kt  # Caching service
│   ├── listeners/                   # Event listeners
│   │   └── BazBomProjectListener.kt # Project lifecycle
│   └── util/                        # Utilities
│       └── BazBomCliRunner.kt       # CLI execution
└── src/main/resources/
    └── META-INF/
        └── plugin.xml               # Plugin descriptor
```

## License

MIT - See LICENSE file for details

## Support

- **GitHub Issues**: https://github.com/cboyd0319/BazBOM/issues
- **Documentation**: https://github.com/cboyd0319/BazBOM/tree/main/docs
- **Email**: support@bazbom.io
