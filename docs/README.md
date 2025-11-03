# Documentation Index

Welcome to the BazBOM documentation! This directory contains comprehensive guides for using and understanding BazBOM.

## Getting Started

- **[90-Second Quickstart](90-SECOND-QUICKSTART.md)** ⚡ Zero to first scan in 90 seconds
- **[Quick Start](QUICKSTART.md)** - Get up and running in 5 minutes
- **[Usage Guide](USAGE.md)** - Day-to-day commands and workflows
- **[IDE Setup](quickstart/IDE_SETUP.md)** - Quick start guide for IDE integration
- **[Homebrew Installation](HOMEBREW_INSTALLATION.md)** - Install via Homebrew tap
- **[Migration Guide](MIGRATION_GUIDE.md)** - Transition from Python to Rust CLI

## Architecture & Design

- **[Architecture (Current State)](ARCHITECTURE_CURRENT.md)** ⭐ - **Current Python/Rust dual architecture with data flows**
- **[Architecture (Planned)](ARCHITECTURE.md)** - Long-term system design and architecture diagrams
- **[Supply Chain Security](SUPPLY_CHAIN.md)** - SBOM/SCA architecture and usage
- **[Threat Model](THREAT_MODEL.md)** - Security assets, risks, and controls
- **[ADRs](ADR/)** - Architecture Decision Records
  - [ADR-0001: Fetch Strategy](ADR/ADR-0001-fetch-strategy.md) - Why http_archive over BCR
  - [ADR-0002: SBOM Format](ADR/ADR-0002-sbom-format.md) - SPDX vs CycloneDX selection
  - [ADR-0003: Aspect Scope](ADR/ADR-0003-aspect-scope.md) - Target coverage and filtering
  - [ADR-0004: SARIF Mapping](ADR/ADR-0004-sarif-mapping.md) - Severity level mapping
  - [ADR-0005: Incremental Analysis](ADR/ADR-0005-incremental-analysis.md) - Git-based optimization
  - [ADR-0006: Graph Storage](ADR/ADR-0006-graph-storage.md) - Dependency graph data structure
  - [ADR-0007: SLSA Level](ADR/ADR-0007-slsa-level.md) - Provenance target level

## Advanced Features

- **[Orchestrated Scanning](ORCHESTRATED_SCAN.md)** - Integrate Semgrep and CodeQL with merged SARIF output
- **[IDE Integration](IDE_INTEGRATION.md)** - Real-time vulnerability scanning in your IDE
- **[Vulnerability Enrichment](VULNERABILITY_ENRICHMENT.md)** - KEV, EPSS, GHSA, and risk scoring
- **[Performance Optimization](PERFORMANCE.md)** - Tuning for large monorepos
- **[Provenance Generation](PROVENANCE.md)** - SLSA provenance and signing
- **[VEX Statements](VEX.md)** - Managing false positives and accepted risks
- **[Dependency Graph Analysis](GRAPH_ANALYSIS.md)** - Querying and visualizing dependencies
- **[RipGrep Integration](RIPGREP_INTEGRATION.md)** - Fast file scanning for large monorepos
- **[Capabilities Reference](reference/capabilities-reference.md)** - Complete feature catalog

## Operations

- **[Validation](VALIDATION.md)** - SPDX and SARIF validation steps
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions
- **[Versioning](VERSIONING.md)** - Release process and semantic versioning guidelines

## Developer & Maintainer Guides

- **[Release Process](RELEASE_PROCESS.md)** - Binary releases, signing, and distribution
- **[Homebrew Tap Creation](HOMEBREW_TAP_CREATION.md)** - Creating and maintaining the Homebrew tap
- **[Release Checklist](RELEASE_CHECKLIST.md)** - Pre-release verification steps

## Testing & Coverage

- **[Test Plan](TEST_PLAN.md)** - Strategy and scope
- **[Testing Guide](testing/TESTING_GUIDE.md)** - Comprehensive testing documentation (Rust)
- **[Test Fixtures](testing/TEST_FIXTURES_README.md)** - Sample data for testing

## Planning & Roadmap Documentation

Essential resources for tracking BazBOM development:

- **[Master Roadmap (ROADMAP.md)](ROADMAP.md)** 📋 - **Complete feature tracking checklist with all phases and distribution channels**
- **[Implementation Roadmap](copilot/IMPLEMENTATION_ROADMAP.md)** 🚀 - **8-week UX sprint: Interactive init, TUI explorer, web dashboard, team features**
- **[Implementation Status](copilot/IMPLEMENTATION_STATUS.md)** ⭐ - **Comprehensive audit of actual vs. documented capabilities**
- **[Strategic Roadmap](copilot/STRATEGIC_ROADMAP.md)** 🎯 - **12-18 month vision and market leadership plan**
- **[Copilot Directory](copilot/)** - All phase specifications and planning documents
- **[Developer Guides](developer/)** - Internal development documentation

## Diagrams

Architecture diagrams are available in the [diagrams](diagrams/) directory in Mermaid format.

## Documentation Standards

All canonical documentation lives under `docs/` only. Root-level files are minimal stubs when required by GitHub UX.

Standards:
- **[Documentation Standards](copilot/DOCUMENTATION_STANDARDS.md)** (canonical)
- **[Google Developer Documentation Style Guide](https://developers.google.com/style)**
- **[Microsoft Writing Style Guide](https://learn.microsoft.com/en-us/style-guide/welcome/)**
- **Diátaxis Framework** (Tutorial, How-to, Explanation, Reference)

Validation (CI):
- `markdownlint` for structure
- `vale` for style and tone
- Link validation
- Examples verified where applicable
