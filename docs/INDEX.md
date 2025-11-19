# BazBOM Documentation Index

**Version:** 6.5.0
**Last Updated:** 2025-11-18

This is the master index for all BazBOM documentation. Start here!

---

## 🚀 Quick Start

**New users start here:**

1. [90-Second Quickstart](getting-started/quickstart-90-seconds.md) - Get running FAST
2. [macOS Quick Start](getting-started/MACOS_QUICK_START.md) - macOS-specific setup
3. [Usage Guide](user-guide/usage.md) - Command reference
4. [Quick Reference](QUICKREF.md) - Command cheat sheet

---

## 📊 Current Status & Achievements

**Latest documentation on what's working:**

- **[CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md)** - ⭐ **Complete feature status** (single source of truth)
- **[BENCHMARKS_AND_METRICS.md](BENCHMARKS_AND_METRICS.md)** - Performance metrics & benchmarks
- **[Transitive Reachability Status](archive/status/FINAL_STATUS_V6_5_0.md)** - v6.5.0 completion announcement (archived)

---

## 🎯 Transitive Dependency Reachability (v6.5.0)

**Our killer feature - industry-leading reachability analysis:**

### Overview
- **[Reachability Analysis Guide](reachability/README.md)** - ⭐ Complete guide (8/8 ecosystems)

### Per-Ecosystem Deep Dives
- [Rust/Cargo](reachability/rust.md) - ✅ Production ready (>98% accuracy)
- [JavaScript/npm](reachability/javascript.md) - ✅ Production ready
- [Python/pip](reachability/python.md) - ✅ Production ready
- [Go/Go Modules](reachability/go.md) - ✅ Production ready
- [Java/Maven/Gradle](reachability/java.md) - ✅ Production ready (full bytecode!)

*Ruby, PHP, and Bazel reachability are covered in the [main guide](reachability/README.md).*

---

## 📖 Feature Documentation

### Core Features
- [Bazel Support](BAZEL.md) - Bazel/Bazelisk integration
- [Integrations](INTEGRATIONS.md) - IDE, CI/CD, and tool integrations
- [Memory Management](MEMORY_GUIDE.md) - Memory usage optimization

### Advanced Features
- [Container Scanning](features/container-scanning.md)
- [Upgrade Intelligence](features/upgrade-intelligence.md)
- [Threat Detection](security/threat-detection.md)
- [VEX Support](security/vex.md)

---

## 🔧 Development

### Getting Started
- [Local Environment Setup](development/local-environment-setup.md)
- [Testing Guide](development/testing-guide.md)
- [Test Plan](development/test-plan.md)
- [Debugging](getting-started/debug-logging.md)

### Architecture & Design
- [Architecture Overview](ARCHITECTURE.md)
- [Architecture Details](architecture/architecture.md)
- [Graph Analysis](architecture/graph-analysis.md)
- [ADRs (Architecture Decision Records)](ADR/) - Design decisions

### Implementation Details
- [Dependency Management](development/dependency-management.md)
- [OSV API Integration](development/osv-api-integration.md)
- [Execution Order](development/execution-order.md)

---

## 🛠️ Operations

### Installation & Setup
- [Homebrew Installation](getting-started/homebrew-installation.md)
- [Shell Completions](getting-started/shell-completions.md)
- [IDE Setup](getting-started/ide-setup.md)

### Release Management
- [Release Process](operations/release-process.md)
- [Release Checklist](operations/release-checklist.md)
- [Versioning](operations/versioning.md)
- [Validation](operations/validation.md)

### Security & Compliance
- [GPG Signing](security/GPG_SIGNING.md)
- [Reproducible Builds](operations/REPRODUCIBLE_BUILDS.md)
- [Threat Model](security/threat-model.md)
- [Supply Chain Security](security/supply-chain.md)
- [Secure Coding Guide](security/SECURE_CODING_GUIDE.md)

---

## 📋 Reference

### Format Documentation
- [CycloneDX](formats/cyclonedx.md)
- [SPDX](FORMAT_SPDX.md)
- [SARIF](formats/sarif.md)
- [GitHub Snapshot](formats/github-snapshot.md)

### Capability Reference
- [Capability Matrix](CAPABILITY_MATRIX.md)
- [JVM Build Systems](reference/jvm-build-systems.md)
- [JVM Language Support](reference/jvm-language-support.md)
- [ML Features](reference/ml-features.md)

---

## 📚 Examples

### Practical Guides
- [CLI Examples](examples/CLI_EXAMPLES.md)
- [Bazel Monorepo Workflows](examples/bazel-monorepo-workflows.md)
- [Maven Spring Boot](examples/maven_spring_boot.md)
- [Gradle Kotlin](examples/gradle_kotlin.md)
- [Multi-Module Projects](examples/multi_module.md)
- [Shaded JARs](examples/shaded_jar.md)

---

## 🗂️ Integrations

### IDE Integration
- [IDE Integration Guide](integrations/ide/ide-integration.md)
- [Plugin Testing](integrations/ide/plugin-testing.md)
- [Marketplace Publishing](integrations/ide/marketplace-publishing.md)

### CI/CD Integration
- [CI Guide](CI.md)
- [Orchestrated Scans](integrations/orchestrated-scan.md)
- [Ripgrep Integration](integrations/ripgrep-integration.md)

### External Tools
- [LLM Guide](integrations/llm-guide.md)
- [Policy Integration](user-guide/policy-integration.md)
- [Rego Best Practices](user-guide/rego-best-practices.md)

---

## 🔍 Troubleshooting

- [Troubleshooting Guide](TROUBLESHOOTING.md) - Top 20 failures with exact error text
- [Comprehensive Testing Plan](COMPREHENSIVE_TESTING_PLAN.md)

---

## 📜 Legal & Compliance

- [Third-Party Notices](THIRD_PARTY_NOTICES.md)
- [GDPR Compliance](compliance/GDPR_COMPLIANCE.md)
- [SOC2 Preparation](compliance/SOC2_PREPARATION.md)

---

## 🗃️ Archives

**Historical documentation (kept for reference):**

- [Archived Phase Docs](archive/phases/) - Development phase documentation
- [Old Roadmaps](archive/roadmaps-old/) - Completed roadmaps
- [Audits](archive/audits/) - Historical security and quality audits
- [Feature Integration](archive/feature-integration/) - Old integration docs

---

## 🚧 Work in Progress

### Future Roadmaps
- [v6.8 Roadmap](roadmaps/6.8/)
- [v7 Roadmap](roadmaps/v7/)

---

## 📞 Getting Help

1. Check the [Troubleshooting Guide](TROUBLESHOOTING.md)
2. Review relevant feature documentation above
3. Check [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
4. See [Usage Guide](user-guide/usage.md) for command reference

---

## 📌 Most Important Documents

**If you only read 5 docs, read these:**

1. [CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md) - Complete feature status (single source of truth)
2. [Reachability Analysis](reachability/README.md) - Our killer feature (8/8 ecosystems)
3. [Usage Guide](user-guide/usage.md) - How to use BazBOM
4. [BENCHMARKS_AND_METRICS.md](BENCHMARKS_AND_METRICS.md) - Performance data
5. [90-Second Quickstart](getting-started/quickstart-90-seconds.md) - Get started FAST

---

*Documentation index generated: 2025-11-18*
*BazBOM Version: 6.5.0*
