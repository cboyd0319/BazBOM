# BazBOM Integrations

Connect BazBOM with external tools and platforms for end-to-end supply chain security.

## Overview

BazBOM integrates with scanning tools, platforms, IDEs, and CI systems to provide comprehensive supply chain security. This directory contains detailed integration guides for various tools and workflows.

## Quick Links

- **[Complete Integration Guide](../INTEGRATIONS.md)** - Comprehensive guide covering all integrations with examples

## Integration Guides by Category

### 🔍 Scanning & Analysis

- **[Orchestrated Scan](orchestrated-scan.md)** - Combine BazBOM with Semgrep, CodeQL, and other SAST tools for comprehensive analysis
- **[RipGrep Integration](ripgrep-integration.md)** - Fast code search integration for vulnerability pattern detection
- **[Container Scanning](../features/container-scanning.md)** - Docker/OCI image scanning with layer attribution

### 🤖 AI & ML Features

- **[LLM Integration Guide](llm-guide.md)** - Privacy-first AI-powered fix generation with Ollama, OpenAI, and Anthropic Claude

### 💻 IDE Integration

- **[IDE Integration Overview](ide/ide-integration.md)** - IntelliJ IDEA and VS Code plugins
  - [Marketplace Assets](ide/marketplace-assets.md) - Publishing requirements and assets
  - [Submission Checklist](ide/marketplace-submission.md) - Pre-publication checklist
  - [Plugin Testing](ide/plugin-testing.md) - Testing guide for IDE plugins
  - [Marketplace Publishing](ide/marketplace-publishing.md) - Publishing process

### 🔧 External Tools

Detailed integration guides for external tools are available in the [Complete Integration Guide](../INTEGRATIONS.md):

- **Syft** - Container SBOM generation for non-JVM dependencies
- **OSV-Scanner** - Cross-check vulnerability findings
- **Dependency-Track** - Centralized SBOM management and risk tracking
- **GUAC** - Supply chain graph analysis
- **GitHub Security** - Code Scanning API and Dependabot integration
- **Semgrep** - Custom rule integration
- **CodeQL** - Advanced static analysis

### ⚙️ CI/CD Integration

For CI/CD integration examples, see:
- **[CI Guide](../CI.md)** - GitHub Actions, GitLab CI, Jenkins, CircleCI
- **[Quick Reference](../QUICKREF.md)** - Quick command examples for CI workflows

## Getting Started

Each integration guide assumes familiarity with base BazBOM workflows. If you're new to BazBOM:

1. Start with the [Quick Start Guide](../getting-started/quickstart.md)
2. Review the [Usage Guide](../user-guide/usage.md) for core commands
3. Then explore the specific integration guides above

## Integration Patterns

### Pattern 1: BazBOM as Primary Scanner

```bash
# BazBOM handles everything
bazbom scan . --with-semgrep --ml-risk
```

**Best for:** JVM projects with optional SAST analysis

### Pattern 2: BazBOM + External Tools

```bash
# BazBOM for SCA
bazbom scan . --format spdx

# Syft for container layers
syft myapp:latest -o spdx-json > container-sbom.json

# Upload both to Dependency-Track
```

**Best for:** Multi-language projects, containers

### Pattern 3: Orchestrated Scanning

```bash
# Use orchestrated-scan workflow
bazbom orchestrate --config scan-config.yml
```

**Best for:** Enterprise environments with multiple scanning tools

See [Orchestrated Scan Guide](orchestrated-scan.md) for details.

## Need Help?

- **Troubleshooting:** See [Troubleshooting Guide](../TROUBLESHOOTING.md)
- **Usage Questions:** See [Usage Guide](../user-guide/usage.md)
- **Issues:** [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)

---

**Note:** All integration guides assume you have BazBOM installed and configured. See the [Installation Guide](../getting-started/README.md) if you haven't installed BazBOM yet.
