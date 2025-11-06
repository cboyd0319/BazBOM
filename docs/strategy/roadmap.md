# BazBOM Product Roadmap

**Last Updated:** 2025-11-06  
**Status:** Preparing v1.0.0 GA

This roadmap focuses on the work remaining to ship BazBOM 1.0.0 and the priorities that immediately follow. Historical phase breakdowns are now archived under `docs/archive/phases/`.

---

## Current Status

- ✅ Rust single-binary CLI with signed releases and SLSA provenance
- ✅ Build-system integrations for Maven, Gradle, and Bazel (aspects)
- ✅ Advisory intelligence across OSV, NVD, GHSA with KEV/EPSS enrichment
- ✅ Policy-as-code engine, enterprise license compliance, and VEX workflows
- ✅ Reachability analysis, shading detection, SPDX/CycloneDX/SARIF exporters
- ✅ Homebrew tap, GitHub Action, and release automation
- 🟡 Marketplace publishing for IntelliJ & VS Code extensions (assets + submissions in flight)
- 🟡 Dashboard polish and documentation updates for 1.0 GA
- 🟡 Large-repo validation with design partners

---

## Near-Term Priorities

1. **Marketplace Launch**
   - Finalize visual assets (icons, screenshots, demos)
   - Publish VS Code extension and IntelliJ plugin
   - Document installation/update flows in the Getting Started guides

2. **Dashboard & Reporting Polish**
   - Align dashboard styling with design system
   - Capture end-to-end walkthroughs for executive reporting
   - Harden API endpoints ahead of shared deployments

3. **Field Validation**
   - Exercise `bazbom scan` and remediation workflows on >50K target monorepos
   - Capture telemetry-free performance baselines and update the performance guide
   - Validate policy templates against target compliance frameworks

---

## Distribution & Marketplaces

| Channel | Status | Documentation | Security | Priority |
|---------|--------|---------------|----------|----------|
| **Homebrew** | Live | [Homebrew Installation](../getting-started/homebrew-installation.md) | Signed | P0 |
| **GitHub Releases** | Live | [Release Process](../operations/release-process.md) | Signed | P0 |
| **VS Code Marketplace** | Ready for submission | [IDE Integration](../integrations/ide/ide-integration.md) | Built-in | P0 |
| **JetBrains Marketplace** | Ready for submission | [IDE Integration](../integrations/ide/ide-integration.md) | Built-in | P0 |
| **GitHub Marketplace** | Planned | [action.yml](../action.yml) | Actions | P1 |
| **Windows (Chocolatey)** | Planned | [Implementation Roadmap](product-roadmap/IMPLEMENTATION_ROADMAP.md) | Pending | P1 |
| **Windows (winget)** | Planned | [Implementation Roadmap](product-roadmap/IMPLEMENTATION_ROADMAP.md) | Pending | P1 |
| **Container Images** | Planned | [Container Scanning](../integrations/container-scanning.md) | Pending | P1 |
| **Kubernetes Operator** | Planned | [Strategic Vision](product-roadmap/STRATEGIC_ROADMAP.md) | Pending | P1 |

---

## Backlog Themes (Post v1.0)

- **Scale & Performance:** distributed scanning, caching, and incremental analysis for very large monorepos
- **Threat Intelligence:** continuous malicious package detection, notification workflows, and enrichment data feeds
- **Container & JVM Ecosystem:** OCI image scanning, additional JVM build tools (Ant, Buildr, sbt, Kotlin Multiplatform)
- **AI & Automation:** ML-driven prioritisation, LLM-assisted remediation, smarter policy suggestions
- **Enterprise Distribution:** Windows installers, Kubernetes operator, air-gapped deployment bundles

---

## Supporting Documents

- [Implementation Status](implementation-status.md) – definitive matrix of shipped vs. planned capabilities
- [Implementation Roadmap](product-roadmap/IMPLEMENTATION_ROADMAP.md) – current delivery sprint tracking
- [Strategic Vision](product-roadmap/STRATEGIC_ROADMAP.md) – 12–18 month outlook and investment themes
- [Market Analysis](market-analysis/) – comparative research and positioning
- [Archived Phase Plans](../archive/phases/) – historic documentation retained for reference

