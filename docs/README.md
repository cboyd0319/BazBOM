# BazBOM Documentation

Curated entry point for every BazBOM guide. Files are grouped by the job they help you complete—start with "Getting Started" and branch out to deeper references as needed.

## Quick Navigation

**Essential docs** (from tech writer persona):
- **[QUICKREF.md](QUICKREF.md)** - One-page cheat sheet for all common BazBOM operations
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Components, data flow, boundaries, decisions (ADR-style summaries)
- **[USAGE.md](USAGE.md)** - Quick start guide with links to comprehensive usage documentation
- **[BAZEL.md](BAZEL.md)** - Aspects, rules, targets, macros, minimal reproducible examples
- **[CI.md](CI.md)** - Bazel + CI recipes (GitHub Actions baseline), caching, artifacts, SARIF/attestations
- **[FORMAT_SPDX.md](FORMAT_SPDX.md)** - SPDX 2.3 mapping, fields, known gaps
- **[INTEGRATIONS.md](INTEGRATIONS.md)** - Syft/OSV/Dependency-Track/GUAC (stubs or recipes)
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Top 20 failures with exact error text + fix

## Getting Started

- **[QUICKREF.md](QUICKREF.md)** – One-page cheat sheet (installation, commands, workflows)
- [Quickstart in 90 Seconds](getting-started/quickstart-90-seconds.md) – zero to first scan with defaults
- [Quick Start Guide](getting-started/quickstart.md) – five-minute walkthrough with CI tips
- [Manual Source Installation](getting-started/homebrew-installation.md) – clone, build, and add BazBOM to PATH
- [Shell Completions](getting-started/shell-completions.md) – Bash, Zsh, Fish completion setup
- [IDE Setup Checklist](getting-started/ide-setup.md) – configure IntelliJ / VS Code extensions

## User Guide & How-tos

- [Usage Guide](user-guide/usage.md) – comprehensive guide with core commands, SBOM outputs, policy workflows
- [Report Generation](user-guide/report-generation.md) – executive, compliance, and SARIF reporting
- [Advanced Bazel Usage](user-guide/advanced-bazel-features.md) – query tuning and large monorepos (also see [BAZEL.md](BAZEL.md))
- [Policy Integration](user-guide/policy-integration.md) & [Rego Best Practices](user-guide/rego-best-practices.md)
- [Troubleshooting Playbook](user-guide/troubleshooting.md) (also see top-level [TROUBLESHOOTING.md](TROUBLESHOOTING.md))

## Integrations

- [IDE Integration Overview](integrations/ide/ide-integration.md)
  - [Marketplace Assets](integrations/ide/marketplace-assets.md)
  - [Submission Checklist](integrations/ide/marketplace-submission.md)
  - [Plugin Testing Guide](integrations/ide/plugin-testing.md)
  - [Marketplace Publishing](integrations/ide/marketplace-publishing.md)
- [LLM Integration Guide](integrations/llm-guide.md) - Privacy-first AI-powered fix generation
- [Orchestrated Scan (Semgrep/CodeQL)](integrations/orchestrated-scan.md)
- [Container Scanning](features/container-scanning.md)
- [RipGrep Integration](integrations/ripgrep-integration.md)
- **[Complete Integration Guide](INTEGRATIONS.md)** - All integrations in one place

## Polyglot & Reachability Analysis

**Status**: ✅ Complete (v6.5.0)

- **[Reachability Analysis Overview](reachability/README.md)** - World-class reachability across 7 languages
- [Polyglot Overview](polyglot/README.md) - Multi-language monorepo support

### Supported Languages (7/7)
- ✅ JavaScript/TypeScript (v6.3.0)
- ✅ Python (v6.4.0)
- ✅ Go (v6.4.0)
- ✅ Rust (v6.5.0)
- ✅ Ruby (v6.5.0)
- ✅ PHP (v6.5.0)
- ✅ JVM: Java/Kotlin/Scala (v6.1.0)

## Security & Assurance

- [Supply Chain Architecture](security/supply-chain.md)
- [Threat Detection Guide](security/threat-detection.md)
- [Threat Model](security/threat-model.md)
- [Vulnerability Enrichment](security/vulnerability-enrichment.md)
- [VEX Guidance](security/vex.md) and [VEX Statements](vex/README.md)
- Security policy references: [Secure Coding](security/SECURE_CODING_GUIDE.md), [Risk Ledger](security/RISK_LEDGER.md), [Workflow Security Policy](security/WORKFLOW_SECURITY_POLICY.md)

## Architecture & Internals

- **[Architecture Overview](ARCHITECTURE.md)** - High-level components, data flow, mini-ADRs
- [Detailed Architecture](architecture/architecture.md) - Complete implementation details
- [Dependency Graph Analysis](architecture/graph-analysis.md)
- [ADR Index](ADR/) - Architecture Decision Records
- [Reference: JVM Build Systems](reference/jvm-build-systems.md), [JVM Language Support](reference/jvm-language-support.md), [ML Features](reference/ml-features.md)
- [Diagrams](diagrams/) – Mermaid diagrams rendered in CI

## Operations & Release Management

- **[CI/CD Integration](CI.md)** - Complete CI recipes for GitHub Actions, GitLab, Jenkins, CircleCI
- [Deep Analysis Report](operations/deep-analysis-report.md) - Comprehensive audit results (705 tests, zero issues)
- [Release Process](operations/release-process.md)
- [Release Checklist](operations/release-checklist.md)
- [Homebrew Tap Creation](operations/homebrew-tap-creation.md)
- [Provenance & SLSA](operations/provenance.md)
- [Performance Tuning](operations/performance.md)
- [Validation Workflows](operations/validation.md)
- [Versioning Strategy](operations/versioning.md)

## Development & Testing

- [Local Environment Setup](development/local-environment-setup.md)
- [Dependency Management](development/dependency-management.md)
- [Test Plan](development/test-plan.md) and [Testing Guide](development/testing-guide.md)
- [Test Fixtures](development/test-fixtures.md)

## Examples, Benchmarks & Reference Material

- [Examples](examples/) – sample workflows and CI automation
- [CLI Examples](examples/CLI_EXAMPLES.md)
- [Bazel Monorepo Workflows](examples/bazel-monorepo-workflows.md)
- [Maven Spring Boot](examples/maven_spring_boot.md)
- [Gradle Kotlin](examples/gradle_kotlin.md)
- [Shaded JAR](examples/shaded_jar.md)
- [Benchmarks](benchmarks/README.md)
- [Capabilities Reference](reference/capabilities-reference.md)

## Format Specifications

- **[SPDX 2.3 Format](FORMAT_SPDX.md)** - Field mapping, examples, validation, known gaps
- [CycloneDX Format](formats/cyclonedx.md) - CycloneDX SBOM generation
- [SARIF Format](formats/sarif.md) - Security analysis results
- [Schemas Reference](reference/schemas/README.md)

## Version History & Roadmaps

### Future Roadmap
- 🚀 [v7 Roadmap](roadmaps/V7_ROADMAP.md) - GitHub Marketplace Edition
- 📖 [v7 Quick Start](roadmaps/V7_QUICK_START.md) - 30-day plan

### Historical Milestones
- ✅ v6.5.0 - Complete Polyglot Reachability Parity (archived)
- ✅ v6.4.0 - Python + Go Reachability (archived)
- ✅ v6.3.0 - JavaScript/TypeScript Reachability (archived)
- ✅ v6.2.0 - Polyglot Upgrade Intelligence (archived)

## Quality & Style Standards

- [Third-Party Notices](THIRD_PARTY_NOTICES.md) - Attribution for optional external tools (Semgrep, CodeQL, etc.)
- Style references: Google Developer Style, Microsoft Writing Style, Diátaxis
- CI checks: `markdownlint`, `vale`, broken-link checks, example validation

> All canonical documentation resides under `docs/`. Root-level files exist only where GitHub's UI requires them (README.md, CONTRIBUTING.md, etc.).

## Documentation Principles

Following the **tech writer persona** guidelines:

1. **Signal > Noise** - No paragraphs that can be a table. No tables that should be a checklist.
2. **Progressive disclosure** - Start with the 20% everyone needs; link out for the 80%.
3. **Docs as product** - Each page has a clear job, audience, and success metric.
4. **Truth from code** - When in doubt, infer from `BUILD`, Bazel aspects, and Rust modules.
5. **Maintainability** - Short pages, shared partials/snippets, versioned examples, generated references.
6. **One link of truth** - Canonicalize duplicated guidance. Cross-link instead of repeating.
7. **Mermaid diagrams** - Picture the pipeline. Keep simple (≤7 nodes); annotate edges; title every diagram.
