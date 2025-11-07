# Quick Start

Get BazBOM running in minutes with the essential steps below. Need an even faster path? Check the [90‑Second Quickstart](./quickstart-90-seconds.md).

## Prerequisites

- macOS or Linux shell with Homebrew _or_ access to the latest BazBOM release binaries
- Rust 1.70+ (only if you plan to build from source)
- Java 11 or newer when you enable reachability or bytecode analysis features

## 1. Install BazBOM

> **Recommendation:** Use Homebrew for signed binaries and automatic updates.

```bash
brew tap cboyd0319/bazbom
brew install bazbom
bazbom --version
```

Alternative installation routes:

- **Pre-built binaries:** Download from the [latest GitHub release](https://github.com/cboyd0319/BazBOM/releases) and place `bazbom` on your `PATH`.
- **Build from source:**
  ```bash
  git clone https://github.com/cboyd0319/BazBOM.git
  cd BazBOM
  cargo build --release
  export PATH="$PWD/target/release:$PATH"
  ```
- **Detailed install notes:** See [Homebrew Installation](./homebrew-installation.md).

## 2. Initialise a workspace

Run the interactive initializer from the root of a JVM project:

```bash
bazbom init
```

The wizard detects your build system, proposes policy templates, and triggers the first scan. It creates a `.bazbom/` directory containing configuration, cached advisories, and the initial reports.

## 3. Run your first scan

```bash
bazbom scan .
```

Key options you may add as you grow:

| Option | Purpose |
| --- | --- |
| `--format cyclonedx` | Emit CycloneDX instead of SPDX |
| `--out-dir reports` | Redirect artefacts to a dedicated directory |
| `--reachability` | Enrich findings with bytecode reachability (Java required) |
| `--with-semgrep` / `--with-codeql` | Orchestrate additional static analysis engines |

Default outputs include `sbom/spdx.json`, `findings/sca.sarif`, and `findings/merged.sarif` for GitHub Code Scanning. Full command coverage lives in the [Usage Guide](../user-guide/usage.md).

## 4. Review and remediate

```bash
bazbom dashboard --open   # Web dashboard
bazbom explore             # Terminal dependency explorer
bazbom report executive --output executive.html
bazbom fix --suggest       # Prioritised remediation advice
bazbom fix --apply         # Automated upgrade workflow
```

These commands surface the same results created during the scan. Policies, remediation, and reporting workflows are covered in [Report Generation](../user-guide/report-generation.md) and [Policy Integration](../user-guide/policy-integration.md).

## 5. Wire BazBOM into CI/CD

Embed the scan in GitHub Actions (excerpt):

```yaml
- name: BazBOM Security Scan
  uses: cboyd0319/BazBOM@v1
  with:
    scan-path: '.'
    generate-reports: 'true'
```

More examples are available under [`examples/github-actions`](../../examples) and the [Validation playbook](../operations/validation.md).

## Next steps

- Browse the [Usage Guide](../user-guide/usage.md) for command-by-command detail
- Explore [IDE integration](../integrations/ide/ide-integration.md) or the [LLM workflow](../integrations/llm-integration.md)
- Review your organisation’s exposure with the [Threat Model](../security/threat-model.md) and [Threat Detection guide](../security/threat-detection.md)
- Prepare for release with [Release Process](../operations/release-process.md) and [Versioning](../operations/versioning.md)
- Keep strategy artefacts in view through the [Product Roadmap](../strategy/roadmap.md)

BazBOM is opinionated but flexible—start with the defaults, then layer on reachability, policy enforcement, or orchestrated scans as your maturity grows.
