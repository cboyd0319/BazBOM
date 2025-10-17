# BazBOM

> **Bazel-native SBOM (Software Bill of Materials) and SCA (Software Composition Analysis) for the JVM ecosystem**

BazBOM is a production-grade, security-forward toolkit for generating SBOMs and performing supply chain security analysis on Bazel-built JVM projects. It uses Bazel aspects to discover dependencies, generates standards-compliant SPDX documents, and integrates with vulnerability databases like OSV.

## Features

- 🔍 **Dependency Discovery**: Bazel aspects automatically traverse your build graph
- 📋 **SPDX Compliance**: Generate SPDX 2.3+ compliant SBOM documents
- 🛡️ **Vulnerability Scanning**: Integration with OSV (Open Source Vulnerabilities) database
- 📊 **SARIF Output**: GitHub Code Scanning compatible vulnerability reports
- 🔒 **Security First**: Minimal permissions, reproducible builds, pinned dependencies
- 🚀 **CI Ready**: GitHub Actions workflows included

## Quick Start

```bash
# Clone the repository
git clone https://github.com/cboyd0319/BazBOM.git
cd BazBOM

# Generate SBOMs for all targets
bazel build //:sbom_all

# Run SCA from generated SBOMs
bazel run //:sca_from_sbom

# View generated SPDX files
ls bazel-bin/**/*.spdx.json
```

## Documentation

- [Quick Start Guide](docs/QUICKSTART.md) - Get up and running in 5 minutes
- [Usage Guide](docs/USAGE.md) - Day-to-day commands and workflows
- [Architecture](docs/ARCHITECTURE.md) - System design and architecture diagrams
- [Supply Chain Security](docs/SUPPLY_CHAIN.md) - SBOM/SCA architecture and usage
- [Validation](docs/VALIDATION.md) - SPDX and SARIF validation steps
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions
- [Threat Model](docs/THREAT_MODEL.md) - Security assets, risks, and controls

## Project Governance

- [Contributing Guide](CONTRIBUTING.md) - How to contribute
- [Code of Conduct](CODE_OF_CONDUCT.md) - Community standards
- [Security Policy](SECURITY.md) - Security reporting and disclosure
- [Maintainers](MAINTAINERS.md) - Project maintainers and review policy

## License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## Status

🚧 **Project Status**: Active Development

This is a production-grade implementation following security-first principles and Bazel best practices.
