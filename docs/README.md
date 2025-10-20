# Documentation Index

Welcome to the BazBOM documentation! This directory contains comprehensive guides for using and understanding BazBOM.

## Getting Started

- **[Quick Start](QUICKSTART.md)** - Get up and running in 5 minutes
- **[Usage Guide](USAGE.md)** - Day-to-day commands and workflows

## Architecture & Design

- **[Architecture](ARCHITECTURE.md)** - System design and architecture diagrams
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

- **[Vulnerability Enrichment](VULNERABILITY_ENRICHMENT.md)** - KEV, EPSS, GHSA, and risk scoring
- **[Performance Optimization](PERFORMANCE.md)** - Tuning for large monorepos
- **[Provenance Generation](PROVENANCE.md)** - SLSA provenance and signing
- **[VEX Statements](VEX.md)** - Managing false positives and accepted risks
- **[Dependency Graph Analysis](GRAPH_ANALYSIS.md)** - Querying and visualizing dependencies

## Operations

- **[Validation](VALIDATION.md)** - SPDX and SARIF validation steps
- **[Troubleshooting](TROUBLESHOOTING.md)** - Common issues and solutions

## Testing & Coverage

- **[Test Plan](TEST_PLAN.md)** - Strategy and scope
- **[Testing Reports](testing/)** - Coverage, summaries, and optimization reports

## Diagrams

Architecture diagrams are available in the [diagrams](diagrams/) directory in Mermaid format.

## Documentation Standards

All BazBOM documentation follows:
- **[Google Developer Documentation Style Guide](https://developers.google.com/style)**
- **[Microsoft Writing Style Guide](https://learn.microsoft.com/en-us/style-guide/welcome/)**
- **Diátaxis Framework** (Tutorial, How-to, Explanation, Reference)

Documentation is validated in CI using:
- `markdownlint` for structure
- `vale` for style and tone
- Link validation
- Example code execution tests
