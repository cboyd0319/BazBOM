# Orchestrated SCA and Static Analysis

BazBOM can orchestrate multiple analysis tools (SCA, Semgrep, CodeQL) and merge their findings into a single SARIF report for GitHub Code Scanning.

## Quick Start

Run a scan with Semgrep integration:

```bash
bazbom scan . --with-semgrep --no-upload
```

Run with both Semgrep and CodeQL:

```bash
bazbom scan . --with-semgrep --with-codeql=security-extended
```

## CLI Flags

### Analysis Flags

- `--with-semgrep` - Run Semgrep pattern analysis (requires semgrep installed)
- `--with-codeql[=SUITE]` - Run CodeQL dataflow analysis
  - `default` - Basic security and quality queries
  - `security-extended` - Extended security query suite
- `--cyclonedx` - Also emit CycloneDX SBOM format

### Control Flags

- `--no-upload` - Skip GitHub Code Scanning upload (local dev)
- `--target-module <MODULE>` - Limit analysis to specific module
- `--out-dir <DIR>` - Output directory (default: current directory)

## Output Structure

Orchestrated scans create this directory structure:

```
out/
├── sbom/
│   ├── spdx.json              # SPDX 2.3 SBOM (always)
│   └── cyclonedx.json         # CycloneDX (if --cyclonedx)
├── findings/
│   ├── sca.sarif              # SCA vulnerability findings
│   ├── semgrep.sarif          # Semgrep findings (if --with-semgrep)
│   ├── codeql.sarif           # CodeQL findings (if --with-codeql)
│   └── merged.sarif           # Merged report for upload
├── enrich/
│   └── depsdev.json           # Enrichment data (future)
└── fixes/
    └── openrewrite/           # Fix suggestions (future)
```

## Configuration File

Create `bazbom.toml` in your project root:

```toml
[analysis]
cyclonedx = true

[analysis.semgrep]
enabled = true
ruleset = "curated-jvm@sha256:abc123"

[analysis.codeql]
enabled = false
suite = "default"

[enrich]
depsdev = true

[publish]
github_code_scanning = true
artifact = true
```

CLI flags override configuration file settings.

## Tool Requirements

### Semgrep

Install via pipx (recommended):

```bash
pipx install semgrep
```

Or use the standalone binary from [semgrep.dev](https://semgrep.dev/docs/getting-started/).

### CodeQL

Download from [GitHub CodeQL CLI releases](https://github.com/github/codeql-cli-binaries/releases):

```bash
# Example for Linux
CODEQL_VERSION=2.19.4
curl -sL https://github.com/github/codeql-cli-binaries/releases/download/v${CODEQL_VERSION}/codeql-linux64.zip -o codeql.zip
unzip codeql.zip -d $HOME
echo 'export PATH="$HOME/codeql:$PATH"' >> ~/.bashrc
```

## GitHub Actions Integration

### PR Workflow (Fast)

```yaml
name: Security Scan
on:
  pull_request:

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up JDK
        uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: '21'
      
      - name: Install Semgrep
        run: pipx install semgrep
      
      - name: BazBOM scan
        run: bazbom scan . --with-semgrep --cyclonedx
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: findings/merged.sarif
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: bazbom-results
          path: |
            sbom/**
            findings/**
```

### Main Branch Workflow (Deep Analysis)

```yaml
name: Deep Security Scan
on:
  push:
    branches: [main]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Set up JDK
        uses: actions/setup-java@v4
        with:
          distribution: temurin
          java-version: '21'
      
      - name: Install tools
        run: |
          pipx install semgrep
          CODEQL_VERSION=2.19.4
          curl -sL https://github.com/github/codeql-cli-binaries/releases/download/v${CODEQL_VERSION}/codeql-linux64.zip -o codeql.zip
          unzip -q codeql.zip -d $HOME
          echo "$HOME/codeql" >> $GITHUB_PATH
      
      - name: BazBOM deep scan
        run: |
          bazbom scan . \
            --with-semgrep \
            --with-codeql=security-extended \
            --cyclonedx
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: findings/merged.sarif
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: bazbom-deep-scan
          path: |
            sbom/**
            findings/**
            enrich/**
```

## Backward Compatibility

The orchestrated scanning mode activates only when you use:
- `--with-semgrep`
- `--with-codeql`
- `--cyclonedx` (with analysis flags)

Without these flags, BazBOM uses the original scan logic, ensuring backward compatibility.

## Current Status

### Implemented
- Core pipeline infrastructure
- SARIF merge and deduplication
- Semgrep integration (requires user-installed CLI)
- CodeQL integration (placeholder, requires user-installed CLI)
- Configuration file support
- GitHub Actions examples

### In Progress
- Full CodeQL database creation and analysis
- SCA migration to pipeline
- Enrichment via deps.dev

### Planned
- OpenRewrite autofix (dry-run and PR modes)
- Container SBOM integration (Syft fallback)
- GUAC/Dependency-Track publishers
- Performance optimizations (scope by changed files)

## See Also

- [Architecture](ARCHITECTURE.md)
- [Capabilities Reference](reference/capabilities-reference.md)
- [GitHub Actions](examples/bazel-monorepo-workflows.md)
