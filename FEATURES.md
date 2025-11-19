# BazBOM Features & Capabilities

**Version 6.6.0** | Complete Feature Matrix

BazBOM is a developer-friendly security scanner with reachability analysis that cuts vulnerability noise by 70-90%.

---

## Table of Contents

- [Validation Status](#validation-status)
- [CLI Commands](#cli-commands)
- [Ecosystem Support](#ecosystem-support)
- [Reachability Analysis](#reachability-analysis)
- [Threat Detection](#threat-detection)
- [Output Formats](#output-formats)
- [Integrations](#integrations)
- [Crate Architecture](#crate-architecture)

---

## Validation Status

> **Legend:** ✅ Tested | ⚠️ Implemented (needs validation) | ❌ Not implemented

### Core Feature Parity Matrix

| Feature | npm | pip | cargo | bundler | composer | go | maven | gradle | bazel |
|---------|-----|-----|-------|---------|----------|----|----|--------|-------|
| **SBOM Generation** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Vulnerability Scanning** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **Reachability Analysis** | ⚠️ | ⚠️ | ✅ | ⚠️ | ⚠️ | ⚠️ | ✅ | ✅ | ⚠️ |
| **Auto-Remediation** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | ⚠️ |
| **Typosquatting DB** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | N/A |
| **License Scanning** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

### Validation Notes

**✅ Fully Tested:**
- Unit tests pass
- Integration tests pass
- Tested against real-world projects

**⚠️ Needs Validation:**
- Code implemented with unit tests
- Awaiting integration testing on real projects
- May have edge cases not covered

**Recent Additions (v6.6.0):**
- Maven/Gradle/Bazel remediation updaters added - need real-world validation
- Go typosquatting database added
- Reachability analyzers for all languages implemented but accuracy estimates need validation

---

## CLI Commands

### Quick Commands (Zero-Config)

| Command | Description | Use Case |
|---------|-------------|----------|
| `check` | Fast local scan (<10s) | Daily development |
| `quick` | 5-second smoke test | Pre-commit check |
| `ci` | CI-optimized (JSON + SARIF) | Automated pipelines |
| `pr` | PR-optimized (incremental + diff) | Pull request validation |
| `full` | All features enabled | Comprehensive audit |

### Core Scanning

| Command | Description | Key Flags |
|---------|-------------|-----------|
| `scan` | Full SBOM + vulnerability scan | `-r` (reachability), `--include-cicd` |
| `container-scan` | Container security analysis | `--layers`, `--fix` |
| `threats` | Threat intelligence scanning | Typosquatting, dependency confusion |
| `anomaly` | ML anomaly detection | Risk scoring |

### Remediation

| Command | Description | Output |
|---------|-------------|--------|
| `fix` | Generate/apply remediation | PRs, patches, Jira tickets |
| `explain` | Vulnerability details + call chain | Deep CVE analysis |
| `license` | License compliance checking | SPDX identifiers |

### DevOps & Integration

| Command | Description | Platforms |
|---------|-------------|-----------|
| `install` | CI/CD templates | GitHub, GitLab, Jenkins, CircleCI, Azure |
| `install-hooks` | Git pre-commit hooks | Pre-commit validation |
| `jira` | Jira integration | Ticket creation |
| `git-hub` | GitHub integration | PR comments, Actions |
| `notify` | Notifications | Slack, Teams, Email |

### Analysis & Reporting

| Command | Description | Output |
|---------|-------------|--------|
| `status` | Quick security overview | Score, summary |
| `compare` | Compare branches/commits | Diff analysis |
| `report` | Security/compliance reports | HTML, PDF |
| `policy` | Policy checks | SARIF, JSON |
| `vex` | VEX management | False positive suppression |

### Developer Tools

| Command | Description | Features |
|---------|-------------|----------|
| `explore` | TUI dependency graph explorer | ASCII tree, interactive |
| `dashboard` | Web dashboard server | Real-time monitoring |
| `watch` | Continuous monitoring | Auto-rescan on changes |
| `lsp` | IDE integration server | Real-time warnings |
| `init` | Interactive setup wizard | Configuration generation |

### Administration

| Command | Description | Features |
|---------|-------------|----------|
| `db` | Advisory database operations | Update, sync, offline |
| `auth` | Authentication/RBAC | Team permissions |
| `team` | Team coordination | CVE ownership |

---

## Ecosystem Support

### Build Systems (13 Total)

#### JVM Ecosystem

| System | Detection | Lock File | Workspaces |
|--------|-----------|-----------|------------|
| Maven | `pom.xml` | `dependency:tree` | Multi-module |
| Gradle | `build.gradle`, `build.gradle.kts` | `gradle.lockfile` | Multi-project |
| Bazel | `BUILD`, `WORKSPACE` | `maven_install.json` | Monorepo native |
| SBT | `build.sbt` | `build.sbt.lock` | Multi-project |
| Ant | `build.xml` + Ivy | `ivy.xml` | - |
| Buildr | `Buildfile` | - | - |
| Android | `build.gradle` | - | - |

#### JavaScript/TypeScript

| System | Detection | Lock File | Workspaces |
|--------|-----------|-----------|------------|
| npm | `package.json` | `package-lock.json` | Supported |
| Yarn | `package.json` | `yarn.lock` | Supported |
| pnpm | `package.json` | `pnpm-lock.yaml` | Supported |

#### Python

| System | Detection | Lock File |
|--------|-----------|-----------|
| pip | `requirements.txt` | `requirements.txt` |
| Poetry | `pyproject.toml` | `poetry.lock` |
| Pipenv | `Pipfile` | `Pipfile.lock` |
| PDM | `pyproject.toml` | `pdm.lock` |

#### Other Languages

| Language | System | Detection | Lock File |
|----------|--------|-----------|-----------|
| Go | go modules | `go.mod` | `go.sum` |
| Rust | Cargo | `Cargo.toml` | `Cargo.lock` |
| Ruby | Bundler | `Gemfile` | `Gemfile.lock` |
| PHP | Composer | `composer.json` | `composer.lock` |

---

## Reachability Analysis

Determines which vulnerabilities are actually exploitable through call graph analysis.

### Language Support

| Language | Accuracy | Parser | Special Features |
|----------|----------|--------|------------------|
| Rust | >98% | Native syn | Trait tracking, macro expansion |
| JVM (Java/Kotlin/Scala) | >95% | OPAL Framework | Bytecode analysis, reflection detection |
| Go | ~90% | tree-sitter | Goroutine tracking, interface resolution |
| JavaScript/TypeScript | ~85% | SWC | Framework detection (React, Vue, Express) |
| Python | ~80% | RustPython | Dynamic code warnings, Django/Flask |
| Ruby | ~75% | - | Rails, RSpec, metaprogramming support |
| PHP | ~70% | - | Laravel, Symfony, WordPress |

### Analysis Features

- **Call graph construction** - Maps function calls across the codebase
- **Entry point detection** - main(), routes, test functions, event handlers
- **Framework awareness** - Understands routing, middleware, dependency injection
- **Dynamic code warnings** - Flags reflection, eval, metaprogramming
- **Container support** - Full call graph analysis for containerized apps

---

## Threat Detection

### Supply Chain Attack Detection

| Threat Type | Detection Method |
|-------------|------------------|
| **Typosquatting** | Keyboard proximity, homoglyphs, character substitution |
| **Dependency confusion** | Public/private namespace collision |
| **Maintainer takeover** | Ownership transfer detection |
| **Protestware** | Known malicious package database (colors, faker, node-ipc) |

### Package Risk Assessment

| Risk Factor | Analysis |
|-------------|----------|
| **Abandonment risk** | Last update, issue response time |
| **Bus factor** | Contributor distribution |
| **License risk** | Compliance issues, license changes |
| **Obfuscation** | Minified/obfuscated install scripts |
| **OpenSSF Scorecard** | Community security practices |

### Install Script Analysis

- **Network patterns** - Outbound connections, data exfiltration
- **Filesystem patterns** - Sensitive file access, persistence
- **Execution patterns** - Shell spawning, privilege escalation
- **Crypto miner detection** - Mining software indicators

---

## Output Formats

### SBOM Formats

| Format | Versions | Variants |
|--------|----------|----------|
| SPDX | 2.3 | JSON (default), Tag-Value |
| CycloneDX | 1.5 | JSON, XML |

### Report Formats

| Format | Use Case | Features |
|--------|----------|----------|
| JSON | CI/CD integration | Machine-readable, complete data |
| SARIF | GitHub/IDE integration | Code scanning alerts |
| HTML | Human review | Interactive, styled |
| PDF | Compliance | PCI-DSS, HIPAA, FedRAMP, SOC2 |

### Graph Export

| Format | Tool |
|--------|------|
| GraphML | Cytoscape, yEd |
| DOT | Graphviz |
| JSON | Custom visualization |

---

## Integrations

### CI/CD Platforms

| Platform | Install Command | Features |
|----------|-----------------|----------|
| GitHub Actions | `bazbom install github` | SARIF upload, PR comments, quality gates |
| GitLab CI | `bazbom install gitlab` | Pipeline integration, MR comments |
| Jenkins | `bazbom install jenkins` | Pipeline steps, report archiving |
| CircleCI | `bazbom install circleci` | Orb support, artifact upload |
| Azure DevOps | `bazbom install azure` | Pipeline tasks, board integration |

### Issue Tracking

| Platform | Features |
|----------|----------|
| Jira | Ticket creation, CVE linking, bulk operations |
| GitHub Issues | Auto-linking, label management |

### Communication

| Platform | Features |
|----------|----------|
| Slack | Webhooks, interactive messages |
| Microsoft Teams | Adaptive cards, channel notifications |
| Email | SMTP, templated alerts |

### IDE Integration

| IDE | Type | Features |
|-----|------|----------|
| VS Code | Extension | Real-time scanning, inline warnings |
| IntelliJ | Plugin | Gutter icons, quick fixes |
| Any (LSP) | LSP Server | Language-agnostic support |

### Container & Orchestration

| Platform | Features |
|----------|----------|
| Docker/OCI | Layer attribution, base image detection |
| Kubernetes | Operator (CRD-based), admission control |
| Syft | Integration for container SBOM |

---

## Crate Architecture

BazBOM is built as a modular workspace with 28 specialized crates.

### Core Crates

| Crate | Purpose |
|-------|---------|
| `bazbom` | Main CLI binary |
| `bazbom-core` | Core types and traits |
| `bazbom-scanner` | Build system scanning |
| `bazbom-vulnerabilities` | Vulnerability database and matching |

### Analysis Crates

| Crate | Purpose |
|-------|---------|
| `bazbom-reachability` | Call graph and reachability analysis |
| `bazbom-graph` | Dependency graph operations |
| `bazbom-threats` | Threat intelligence and detection |
| `bazbom-ml` | ML-enhanced risk scoring |
| `bazbom-upgrade-analyzer` | Breaking change analysis |

### Output Crates

| Crate | Purpose |
|-------|---------|
| `bazbom-formats` | SPDX, CycloneDX, SARIF generation |
| `bazbom-reports` | HTML/PDF report generation |
| `bazbom-policy` | Policy evaluation (Rego/YAML/CUE) |

### Integration Crates

| Crate | Purpose |
|-------|---------|
| `bazbom-github` | GitHub API integration |
| `bazbom-jira` | Jira API integration |
| `bazbom-depsdev` | deps.dev API client |
| `bazbom-containers` | Container scanning (Syft) |
| `bazbom-operator` | Kubernetes operator |

### UI Crates

| Crate | Purpose |
|-------|---------|
| `bazbom-tui` | Terminal UI (graph explorer) |
| `bazbom-dashboard` | Web dashboard server |
| `bazbom-lsp` | Language Server Protocol |
| `bazbom-vscode-extension` | VS Code extension |
| `bazbom-intellij-plugin` | IntelliJ plugin |

### Infrastructure Crates

| Crate | Purpose |
|-------|---------|
| `bazbom-cache` | Caching layer |
| `bazbom-auth` | Authentication and RBAC |
| `bazbom-crypto` | Cryptographic operations |
| `bazbom-orchestrator` | Parallel scan orchestration |
| `bazbom-verify` | SBOM verification |
| `bazbom-tool-verify` | Tool verification |

---

## Key Capabilities Summary

### Noise Reduction

- **70-90% fewer alerts** through reachability analysis
- **P0-P4 priority scoring** for focused remediation
- **EPSS integration** for exploit probability
- **CISA KEV** for actively exploited vulnerabilities

### Developer Experience

- **Zero-config scanning** with smart defaults
- **Plain English output** instead of CVE jargon
- **One-command CI setup** for 5 platforms
- **Universal auto-fix** for 9 package managers
- **Watch mode** for continuous monitoring

### Enterprise Features

- **SLSA v1.1 Level 3** provenance
- **Offline/air-gapped** operation
- **SBOM signing** with Cosign/Sigstore
- **Policy-as-code** (Rego, YAML, CUE)
- **Team RBAC** and CVE ownership
- **Compliance reports** (PCI-DSS, HIPAA, FedRAMP, SOC2)

### Scale

- **5K+ Bazel targets** tested
- **Incremental scanning** for 10x faster PRs
- **Parallel orchestration** for multi-ecosystem scans
- **OSV batch API** for 97% fewer HTTP requests

---

## Related Documentation

- [Quick Reference](docs/QUICKREF.md) - Command examples
- [Usage Guide](docs/user-guide/usage.md) - Workflows
- [Bazel Integration](docs/BAZEL.md) - Monorepo features
- [Reachability Analysis](docs/reachability/README.md) - How it works
- [Container Scanning](docs/features/container-scanning.md) - Docker/OCI
- [CI/CD Integration](docs/CI.md) - Pipeline setup

---

**BazBOM** - Find vulnerabilities that actually matter.
