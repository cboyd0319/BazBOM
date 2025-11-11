# Third-Party Notices

This document provides information about third-party tools and software that BazBOM may interact with or invoke during operation. BazBOM does not redistribute, embed, or bundle these tools—they are external dependencies that users may optionally install.

## External Tools (Optional)

### Semgrep

- **License**: LGPL v2.1 (Semgrep CE engine)
- **Usage**: BazBOM can optionally invoke Semgrep for static application security testing (SAST)
- **Installation**: User-installed via `pip install semgrep` or system package manager
- **Website**: <https://semgrep.dev/>
- **Note**: BazBOM includes custom security rules written by the BazBOM project, not rules from Semgrep's registry

### CodeQL

- **License**: GitHub CodeQL Terms & Conditions (free for open-source analysis)
- **Usage**: BazBOM can optionally invoke CodeQL CLI for code analysis
- **Installation**: User-installed or available via GitHub Actions
- **Website**: <https://codeql.github.com/>
- **Note**: BazBOM does not redistribute CodeQL binaries or queries

### Syft

- **License**: Apache License 2.0
- **Copyright**: Anchore, Inc.
- **Usage**: BazBOM can optionally invoke Syft for container SBOM generation
- **Installation**: User-installed via `brew install syft` or from <https://github.com/anchore/syft>
- **Website**: <https://github.com/anchore/syft>
- **Note**: BazBOM does not redistribute Syft binaries

## Rust Dependencies

BazBOM is built with Rust and uses various open-source crates. The complete list of dependencies with their licenses is available in:
- `Cargo.lock` - Complete dependency tree with versions
- `Cargo.toml` - Direct dependencies

To generate a complete license report:
```bash
cargo install cargo-license
cargo license --all-features
```

## Build-Time Tools

The following tools are used during development and CI/CD but are not distributed with BazBOM:

- **Rust toolchain** (MIT/Apache-2.0)
- **Bazel** (Apache-2.0)
- **Buildifier** (Apache-2.0)
- **pre-commit** (MIT)
- **markdownlint** (MIT)
- **Vale** (MIT)

## Security Scanning

BazBOM uses the following tools for security scanning in CI/CD:
- **Bandit** (Apache-2.0) - Python SAST
- **Safety** (MIT) - Python dependency vulnerability scanning
- **pip-audit** (Apache-2.0) - Python package auditing
- **TruffleHog** (AGPL-3.0) - Secret scanning
- **GitLeaks** (MIT) - Secret detection

## Compliance Statement

BazBOM does not redistribute, embed, bundle, or statically link any of the external tools listed above. Users who wish to use BazBOM's optional integration with these tools must install them separately according to each tool's license terms.

The custom Semgrep rules and CodeQL configurations included in this repository are original works created by the BazBOM project and are licensed under the same MIT license as BazBOM itself.

## Questions?

If you have questions about BazBOM's use of third-party tools or licensing compliance, please open an issue at:
<https://github.com/cboyd0319/BazBOM/issues>
