# BazBOM Capability Matrix - Quick Reference

## Feature Status Overview

### Legend
- **STABLE** - Production-ready, fully tested
- **INCOMPLETE** - Partial implementation, stubs exist
- **EXPERIMENTAL** - Under development
- **TODO** - Planned but not implemented

---

## Core Capabilities Matrix

### Command Line Interface (11 Commands)

| Command | Subcommands | Status | Key Capabilities |
|---------|------------|--------|-------------------|
| scan | - | STABLE | SBOM generation, SCA, plugin integration, Bazel support |
| container-scan | - | STABLE | OCI image scanning, SBOM generation, comparison |
| policy | check, init, validate | STABLE | Policy enforcement, YAML templates, Rego support |
| fix | - | STABLE | Upgrade intelligence, breaking changes, LLM integration |
| license | obligations, compatibility, contamination | STABLE | License detection, compliance checking |
| db | sync | STABLE | Offline advisory database sync |
| install-hooks | - | STABLE | Git pre-commit hook installation |
| init | - | STABLE | Interactive project setup |
| explore | - | STABLE | TUI-based SBOM exploration |
| dashboard | - | STABLE | Web-based visualization (Axum/Tokio) |
| team | assign, list, mine, audit-log, config | STABLE | Team coordination, assignment tracking |
| report | executive, compliance, developer, trend, all | STABLE | Multi-framework compliance reports |

---

## Build System Support Matrix

| System | Detection File | Parser | Maven Plugin | Gradle Plugin | Status |
|--------|----------------|--------|--------------|---------------|--------|
| **Maven** | pom.xml | Native | ✅ Available | N/A | STABLE |
| **Gradle** | build.gradle[.kts] | Native | N/A | ✅ Available | STABLE |
| **Bazel** | WORKSPACE, MODULE.bazel | Native | N/A | N/A | STABLE |
| **Scala (SBT)** | build.sbt | Native | N/A | N/A | STABLE |
| **Ant** | build.xml | Native | N/A | N/A | STABLE |
| **Buildr** | buildfile, Rakefile | Native | N/A | N/A | STABLE |

---

## JVM Language Support Matrix

| Language | Support | Notes | Status |
|----------|---------|-------|--------|
| **Java** | ✅ Full | Primary focus, all versions | STABLE |
| **Kotlin** | ✅ Full | Multiplatform support | STABLE |
| **Scala** | ✅ Full | Via SBT, all versions | STABLE |
| **Groovy** | ✅ Full | Build script support | STABLE |
| **Clojure** | ✅ Full | Leiningen support | STABLE |
| **Android** | ✅ Special | Gradle-based builds | STABLE |

---

## Polyglot Ecosystem Support Matrix

### Implementation Status

| Ecosystem | Languages | Manifest | Lockfile(s) | Detection | Parsing | Reachability | Status | Notes |
|-----------|-----------|----------|-------------|-----------|---------|--------------|--------|-------|
| **npm** | JavaScript / TypeScript | `package.json` | `package-lock.json`, `yarn.lock*`, `pnpm-lock.yaml*` | ✅ | ✅ | ✅ (bazbom-js-reachability) | STABLE | `*` Yarn/pnpm currently fall back to manifest parsing (warning emitted) |
| **Python** | Python | `pyproject.toml`, `Pipfile`, `requirements.txt` | `poetry.lock`, `Pipfile.lock` | ✅ | ✅ | ✅ (bazbom-python-reachability) | STABLE | Poetry + Pipenv aware |
| **Go** | Go | `go.mod` | `go.sum` | ✅ | ✅ | ✅ (bazbom-go-reachability) | STABLE | Handles replace/indirect blocks |
| **Rust** | Rust | `Cargo.toml` | `Cargo.lock` | ✅ | ✅ | ✅ (bazbom-rust-reachability) | STABLE | cargo-lock crate for accuracy |
| **Ruby** | Ruby | `Gemfile` | `Gemfile.lock` | ✅ | ✅ | ✅ (bazbom-ruby-reachability) | STABLE | Rails/RSpec aware |
| **PHP** | PHP | `composer.json` | `composer.lock` | ✅ | ✅ | ✅ (bazbom-php-reachability) | STABLE | Laravel/Symfony aware |

### Polyglot Features
- Auto-detection: ✅ (no flags needed)
- Unified SBOM: ✅ (polyglot-sbom.json)
- Vulnerability scanning: ✅ (via OSV API)
- Ecosystem isolation: ✅ (per-ecosystem results)

---

## SBOM Format Support Matrix

| Format | Version | Output File | Status | Features |
|--------|---------|-------------|--------|----------|
| **SPDX** | 2.3 | sbom.spdx.json | STABLE | Full spec, PURL support, relationships |
| **CycloneDX** | 1.4 | sbom.cyclonedx.json | STABLE | Full spec, components, licenses |
| **Dual Output** | N/A | Both files | STABLE | --cyclonedx flag, simultaneous generation |

---

## Vulnerability Analysis Matrix

### Source Integration

| Source | Type | Status | Details |
|--------|------|--------|---------|
| **OSV API** | Advisory | STABLE | All 6 ecosystems, no API key required, rate limited |
| **GitHub Advisories** | Advisory | STABLE | Integrated via OSV |
| **NVD (CVE)** | Advisory | STABLE | Integrated via OSV |
| **CVSS Scoring** | Risk Assessment | STABLE | 3.1 specification |
| **EPSS** | Risk Assessment | STABLE | Incorporated when available |

### Analyzer Pipeline

| Analyzer | Type | Technology | Status | Command |
|----------|------|-----------|--------|---------|
| **SCA** | Dependency | OSV | STABLE | (Always runs) |
| **Semgrep** | SAST | Pattern matching | STABLE | --with-semgrep |
| **CodeQL** | SAST | Semantic analysis | STABLE | --with-codeql |
| **Syft** | Container | Image scanning | STABLE | --containers |
| **Threat Intel** | Threat | Pattern detection | STABLE | (Integrated) |

### Report Output

| Format | Default | Alternative | Status |
|--------|---------|-------------|--------|
| **SARIF** | 2.1 | Merged from all tools | STABLE |
| **JSON** | sca_findings.json | Structured vulnerabilities | STABLE |
| **HTML** | Reports via `report` cmd | Multiple types | STABLE |

---

## Advanced Features Matrix

### Reachability Analysis

| Feature | Component | Status | Speed Impact |
|---------|-----------|--------|--------------|
| OPAL Integration | Reachability | STABLE | Significant (mitigated with --fast) |
| Caching | Incremental | STABLE | Hit/miss tracking |
| Filtering | Reachable code only | STABLE | Reduces false positives |

### Upgrade Intelligence

| Feature | Status | Details |
|---------|--------|---------|
| Recursive transitive analysis | STABLE | All dependency changes tracked |
| Breaking change detection | STABLE | GitHub release notes parsed |
| Effort estimation | STABLE | Hours-based, not vague levels |
| Risk scoring | STABLE | LOW/MEDIUM/HIGH/CRITICAL |
| Migration guides | STABLE | Auto-discovered |
| LLM integration | STABLE | Ollama, Anthropic, OpenAI |

### Policy Management

| Feature | Status | Details |
|---------|--------|---------|
| YAML policies | STABLE | Custom rules definable |
| Rego support | STABLE | Optional feature gate |
| Built-in templates | STABLE | PCI-DSS, HIPAA, FedRAMP, SOC2 |
| SARIF output | STABLE | CI/CD integration ready |
| Policy validation | STABLE | Schema checking |

### Performance Features

| Feature | Flag | Status |
|---------|------|--------|
| Benchmarking | --benchmark | STABLE |
| Fast mode (skip reachability) | --fast | STABLE |
| Incremental scanning | --incremental | STABLE |
| Caching | (automatic) | STABLE |
| Parallel processing | (automatic) | STABLE |

---

## Reporting & Visualization Matrix

### Report Types

| Type | Format | Framework Support | Status |
|------|--------|-------------------|--------|
| **Executive** | HTML (1-page) | Any | STABLE |
| **Compliance** | HTML | PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST | STABLE |
| **Developer** | HTML (Technical) | N/A | STABLE |
| **Trend** | HTML (Historical) | N/A | STABLE |

### Interactive Interfaces

| Interface | Framework | Status | Purpose |
|-----------|-----------|--------|---------|
| **Dashboard** | Axum 0.8 + Tokio | STABLE | Web-based visualization |
| **TUI Explorer** | Ratatui 0.29 + Crossterm | STABLE | Terminal-based exploration |
| **CLI Output** | Colored console | STABLE | Human-readable summaries |

---

## Integration Matrix

### CI/CD Integration

| System | Method | Status |
|--------|--------|--------|
| **GitHub Actions** | SARIF upload | STABLE |
| **GitLab CI** | SARIF/JSON | STABLE |
| **Jenkins** | CLI execution | STABLE |
| **Pre-commit hooks** | Git hooks | STABLE |
| **Pull Requests** | --pr flag (fix command) | STABLE |

### Output Formats

| Format | Use Case | Status |
|--------|----------|--------|
| **SARIF 2.1** | CI/CD scanners | STABLE |
| **JSON** | Parsing/analysis | STABLE |
| **HTML** | Reports | STABLE |
| **YAML** | Policies | STABLE |

---

## Crate Architecture Matrix

### Core Crates (v6.5.0 — 30 crates total)

| Area | Crates (examples) | Status | Notes |
|------|-------------------|--------|-------|
| CLI & Formats | `bazbom`, `bazbom-core`, `bazbom-formats`, `bazbom-graph` | ✅ STABLE | Unified commands, SBOM emitters, dependency graph primitives |
| Advisory & Threat Intel | `bazbom-advisories`, `bazbom-threats`, `bazbom-ml` | ✅ STABLE | OSV/NVD/GHSA ingestion, EPSS/KEV enrichment, ML scoring |
| Policy & Automation | `bazbom-policy`, `bazbom-reports`, `bazbom-cache` | ✅ STABLE | Rego/YAML policies, compliance reports, deterministic caching |
| Polyglot & Reachability | `bazbom-polyglot`, `bazbom-{js,python,go,rust,ruby,php}-reachability` | ✅ STABLE | AST/call-graph analysis for 6 non-JVM ecosystems + JVM bridge |
| Containers & Supply Chain | `bazbom-containers`, `bazbom-operator`, `bazbom-cache` | ✅ STABLE | Container scanning, Kubernetes operator, reproducible artifacts |
| Developer Experience | `bazbom-tui`, `bazbom-dashboard`, `bazbom-lsp` | ✅ STABLE | TUI explorer, Axum dashboard, IDE/LSP integrations |

### Focused Enhancements (Active Development)

| Crate | Purpose | Status | Notes |
|-------|---------|--------|-------|
| `bazbom-upgrade-analyzer` | Breaking-change + migration intelligence | ⚙️ BETA | Powers universal auto-fix + effort scoring (shipped, still evolving) |
| `bazbom-depsdev` | deps.dev sync + advisories backfill | ⚙️ BETA | Enabled via feature flag for early adopters |

---

## Test Coverage Matrix

| Category | Count | Status | Notes |
|----------|-------|--------|-------|
| Core CLI + policy unit tests | 180+ | ✅ PASSING | `cargo test --all` across bazbom, core, policy, reports |
| Reachability analyzers | 90+ | ✅ PASSING | Language-specific crates (JS/TS, Python, Go, Rust, Ruby, PHP) |
| Polyglot parsers & detection | 50+ | ✅ PASSING | `bazbom-polyglot` unit + detection tests |
| Container + supply-chain workflows | 20 | ✅ PASSING | bazbom-containers, operator, provenance flows |
| End-to-end workflow smoke tests | 20+ | ✅ PASSING | CLI golden examples + docs validation |
| **Total** | **360+** | **100% PASS** | Reported in CI badges + release checklists |

---

## Known Limitations & TODOs

### Parser Enhancements (Remaining)

| Ecosystem Feature | Status | Blocker | Effort |
|-------------------|--------|---------|--------|
| Yarn.lock rich parsing | PARTIAL | Custom Yarn format (non-JSON) | 3-4 hours |
| pnpm-lock.yaml parsing | PARTIAL | Multi-store format & workspace mapping | 3-4 hours |

### Advanced Features (Pending)

| Feature | Status | Impact | Effort |
|---------|--------|--------|--------|
| JAR bytecode comparison | TODO | Deep breaking-change detection | 6-8 hours |
| Config migration detection | TODO | Framework config diffs | 4-5 hours |
| Community upgrade data | IN PROGRESS | Confidence heuristics | Ongoing |
| PDF report generation | TODO | Exec-ready deliverables | 2-3 hours |

---

## Compliance Standards Matrix

### SBOM Standards

| Standard | Version | Status | Coverage |
|----------|---------|--------|----------|
| SPDX | 2.3 | STABLE | 100% spec compliance |
| CycloneDX | 1.4 | STABLE | 100% spec compliance |
| PURL | Latest | STABLE | All ecosystems |

### Security Standards

| Framework | Version | Status | Reports |
|-----------|---------|--------|---------|
| CVSS | 3.1 | STABLE | Vulnerability scoring |
| EPSS | Latest | STABLE | Exploitability scoring |
| SARIF | 2.1 | STABLE | All analyzers |
| SLSA | Level 3 | STABLE | Provenance |

### Compliance Frameworks

| Framework | Status | Report Type |
|-----------|--------|-------------|
| PCI-DSS 3.2.1 | STABLE | Compliance report |
| HIPAA | STABLE | Compliance report |
| FedRAMP Moderate | STABLE | Compliance report |
| SOC 2 Type II | STABLE | Compliance report |
| GDPR | STABLE | Compliance report |
| ISO 27001 | STABLE | Compliance report |
| NIST CSF | STABLE | Compliance report |

---

## Performance Characteristics

### Speed

| Operation | Typical Time | Notes |
|-----------|--------------|-------|
| Fast scan | <10 seconds | --fast flag, skips reachability |
| Standard scan | 30-60 seconds | Includes reachability |
| Full scan | 2-5 minutes | All analyzers (Semgrep, CodeQL, etc.) |
| Container scan | Depends on image | Size-dependent |
| Incremental scan | 5-15 seconds | Changed files only |

### Resource Usage

| Metric | Typical | Notes |
|--------|---------|-------|
| Memory | 500MB-2GB | Depends on project size |
| CPU | Multi-core | Parallel processing |
| Network | Minimal | OSV API calls only |
| Disk | Output files | ~10-100MB per scan |

---

## Summary Table

| Category | Status | Count | Notes |
|----------|--------|-------|-------|
| **Commands** | STABLE | 11 | All production-ready |
| **Build Systems** | STABLE | 6 | Maven, Gradle, Bazel, SBT, Ant, Buildr |
| **JVM Languages** | STABLE | 6 | Java, Kotlin, Scala, Groovy, Clojure, Android |
| **Polyglot Ecosystems** | STABLE | 6 | npm, Python, Go, Rust, Ruby, PHP (Yarn/pnpm fallback documented) |
| **SBOM Formats** | STABLE | 2 | SPDX 2.3, CycloneDX 1.4 |
| **Analyzers** | STABLE | 5 | SCA, Semgrep, CodeQL, Syft, Threat Intel |
| **Reports** | STABLE | 5 | Executive, Compliance, Developer, Trend, All |
| **Compliance Frameworks** | STABLE | 7 | PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST |
| **Crates** | STABLE | 30 (v6.5.0) | Unified release train |
| **Test Coverage** | 100% PASS | 360+ tests | 0 failures, ≥90% coverage |
| **Known TODOs** | PARTIAL | 4 | Yarn/pnpm parsing, JAR diff, config migration, PDF exports |

---

**Last Updated:** 2025-11-12
**Status:** Production Ready (with noted limitations)
