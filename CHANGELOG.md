# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.1] - 2025-11-03

### Added

- **Master Roadmap (docs/ROADMAP.md)** - Comprehensive feature tracking checklist consolidating all phases
- Complete phase-by-phase tracking for Phases 0-11
- Distribution & marketplace readiness tracking (Homebrew, GitHub Marketplace, IDE plugins, Windows)
- Detailed status indicators for all features and capabilities
- Timeline overview and priority guidance for development
- Success metrics and KPIs for each development phase

### Changed

- **Version bump to 0.5.1** - Updated all Rust crates from 0.2.1 to 0.5.1
- **README roadmap section** - Consolidated and linked to master roadmap
- **Documentation index (docs/README.md)** - Enhanced roadmap navigation and organization
- Improved distribution channel visibility and planning
- Better tracking of marketplace publishing readiness

### Documentation

- Reorganized roadmap documentation for better discoverability
- Added emphasis on GitHub Marketplace, Homebrew, and IDE plugin distribution
- Consolidated multiple roadmap documents into single source of truth
- Enhanced cross-references between planning documents

## [0.2.1] - 2025-10-30

### Added

- Initial repository structure
- Bazel-native SBOM generation framework
- SPDX 2.3 support
- OSV vulnerability scanning integration
- SARIF report generation
- GitHub Actions CI/CD workflows
- Comprehensive documentation
- Example Java project
- Security-first development practices
- **PYSEC_OMEGA security hardening** - Comprehensive security improvements following supreme Python security standards
- Pre-commit hooks configuration with TruffleHog, GitLeaks, Bandit, Semgrep, Ruff, and Black
- Dependabot configuration for automated dependency updates (GitHub Actions, Python, npm, Maven)
- CodeQL workflow for comprehensive Python security analysis
- Custom Semgrep security policies (14 rules covering OWASP Top 10 and CWE Top 25)
- Security documentation directory with Risk Ledger and Secure Coding Guide
- `requirements-security.txt` for easy security tool installation
- Concurrency controls in GitHub Actions to prevent race conditions
- Artifact retention policies in workflows
- Job timeouts for all workflow jobs

### Changed

- **GitHub Actions hardening**: All actions pinned to SHA256 with version comments
- **Permissions hardening**: Per-job permissions following principle of least privilege
- Updated Python version to 3.12 in workflows for latest security patches
- Enhanced workflow permissions with read-only defaults
- Added SHA256 verification for buildifier installation
- Updated `SECURITY.md` with comprehensive security architecture documentation
- Improved documentation linting to include security directory

### Deprecated

- N/A

### Removed

- N/A

### Fixed

- **[SECURITY]** XXE vulnerability in `license_extractor.py` - Replaced `xml.etree.ElementTree` with `defusedxml.ElementTree` (CWE-20)
- **[SECURITY]** URL scheme validation bypass in `supply_chain_risk.py` - Added explicit HTTP/HTTPS scheme validation to prevent SSRF and file disclosure (CWE-22)

### Security

- **Critical security milestone**: 0 High/Critical vulnerabilities, 0 dependency vulnerabilities
- Implemented comprehensive SAST with Bandit, Semgrep, and CodeQL
- Added automated secret detection in pre-commit hooks and CI
- SHA-pinned all GitHub Actions to prevent supply chain attacks
- Configured dependency vulnerability scanning (pip-audit, Safety)
- Established security documentation and policies following PYSEC_OMEGA standards
- 100% of security-critical code paths covered by tests

## [0.1.0] - TBD

### Added

- Initial release
- Core SBOM generation capability
- Basic SCA functionality
