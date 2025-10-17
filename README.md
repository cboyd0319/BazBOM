# BazBOM

**Bazel-native SBOM and SCA for the JVM ecosystem**

BazBOM generates SBOMs (Software Bill of Materials) and performs supply chain security analysis for Bazel-built Java/JVM projects. Zero configuration, automatic dependency discovery via Bazel aspects, SPDX/SARIF output for GitHub Code Scanning.

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/cboyd0319/BazBOM/actions)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![SLSA Level 3](https://img.shields.io/badge/SLSA-Level%203-green)](docs/PROVENANCE.md)

## TL;DR

```bash
# Generate SBOM for all targets
bazel build //:sbom_all

# Run vulnerability scan
bazel run //:sca_scan

# View results
ls bazel-bin/**/*.spdx.json bazel-bin/sca_findings.sarif
```

That's it. SBOMs generated, vulnerabilities scanned, SARIF ready for GitHub Code Scanning.

## What is This?

BazBOM solves the problem of generating accurate, complete SBOMs for Java/JVM projects built with Bazel. Unlike post-build scanners that guess at dependencies, BazBOM uses Bazel's build graph as the source of truth.

**Pain:** Manual SBOM creation is error-prone. Post-build scanners miss transitive dependencies or include test artifacts.

**Solution:** Bazel aspects traverse the dependency graph automatically. Every build produces an SBOM. Maven lockfile provides accurate versions and licenses.

**Target users:** Security teams, DevSecOps, organizations with Bazel+Java monorepos requiring supply chain security.

## Features

| Feature | Description | Status |
|---------|-------------|--------|
| **SBOM Generation** | SPDX 2.3 (JSON), optional CycloneDX | ✅ |
| **Dependency Graph** | JSON + GraphML for visualization | ✅ |
| **Vulnerability Scanning** | OSV, NVD, GitHub Security Advisories | ✅ |
| **SARIF Output** | GitHub Code Scanning integration | ✅ |
| **SLSA Provenance** | Level 3, Sigstore signed | ✅ |
| **VEX Support** | False positive suppression (CSAF 2.0) | ✅ |
| **License Compliance** | Extract licenses, detect conflicts | ✅ |
| **Incremental Analysis** | Git-based changed target detection | ✅ |
| **Large Monorepo Support** | 5000+ targets, <30 min analysis | ✅ |

## Quickstart

### Prerequisites

| Requirement | Version | Why |
|-------------|---------|-----|
| Bazel | ≥ 6.0 | Build system |
| Java | ≥ 11 | JVM runtime |
| Python | ≥ 3.9 | Tooling scripts |
| Git | ≥ 2.30 | Incremental analysis |

### Installation

1. **Add to WORKSPACE:**

```python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "bazbom",
    urls = ["https://github.com/cboyd0319/BazBOM/archive/v1.0.0.tar.gz"],
    sha256 = "...",  # Replace with actual SHA256
    strip_prefix = "BazBOM-1.0.0",
)
```

2. **Run setup:**

```bash
bazel run @bazbom//setup:install
```

3. **Generate first SBOM:**

```bash
bazel build //app:app_sbom
cat bazel-bin/app/app_sbom.spdx.json
```

**Expected output:** Valid SPDX 2.3 JSON with all dependencies.

## Usage

### Basic: Generate SBOM for Single Target

```bash
bazel build //app:deployable_sbom
```

Output: `bazel-bin/app/deployable_sbom.spdx.json`

### Generate SBOMs for All Targets

```bash
bazel build //:sbom_all
```

Output: SBOMs for every `java_binary` and `java_library` in workspace.

### Run Vulnerability Scan

```bash
bazel run //:sca_scan
```

Output:
- `bazel-bin/sca_findings.json` - Machine-readable findings
- `bazel-bin/sca_findings.sarif` - GitHub Code Scanning format

### Generate Dependency Graph

```bash
bazel build //:dep_graph_all
```

Output:
- `bazel-bin/dep_graph.json` - Queryable graph
- `bazel-bin/dep_graph.graphml` - For Gephi/yEd visualization

### Apply VEX Statements (Filter False Positives)

```bash
bazel run //:apply_vex -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

See [VEX Guide](docs/VEX.md) for creating VEX statements.

## Configuration

BazBOM works with zero configuration for most projects. Configuration options:

| Flag | Default | Purpose |
|------|---------|---------|
| `--define=include_test_deps=true` | `false` | Include test dependencies in SBOM |
| `--define=cyclonedx=true` | `false` | Generate CycloneDX in addition to SPDX |
| `--define=max_depth=N` | `unlimited` | Limit transitive dependency depth |
| `--define=offline_mode=true` | `false` | Use local CVE database (no network) |

Example:

```bash
bazel build //:sbom_all --define=include_test_deps=true --define=cyclonedx=true
```

See [Usage Guide](docs/USAGE.md) for full configuration reference.

## Architecture

```
┌─────────────┐
│  Developer  │
└──────┬──────┘
       │ bazel build
       ▼
┌─────────────────┐
│ Bazel Aspect    │ ──► Traverse dependency graph
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ write_sbom.py   │ ──► Generate SPDX 2.3 JSON
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ osv_query.py    │ ──► Query OSV for vulnerabilities
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ sarif_adapter   │ ──► Convert to SARIF 2.1.0
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ GitHub Security │ ──► Display in Code Scanning
└─────────────────┘
```

See [Architecture Doc](docs/ARCHITECTURE.md) for detailed design.

## Security

- **Secrets:** No secrets required. OSV API is public.
- **Least privilege:** Read-only access to source code and dependencies.
- **Supply chain:** Releases signed with Sigstore. SBOMs published at `/releases/`.
- **Disclosure:** Report vulnerabilities via [SECURITY.md](SECURITY.md).
- **SLSA Level:** Level 3 (signed provenance, hardened build platform).

## Performance

Expected analysis times (with remote cache):

| Repo Size | Targets | Dependencies | Time |
|-----------|---------|--------------|------|
| Small | < 50 | < 100 | < 2 min |
| Medium | 50-500 | 100-500 | < 5 min |
| Large | 500-5K | 500-2K | < 15 min |
| Massive | 5K+ | 2K+ | < 30 min (incremental) |

See [Performance Guide](docs/PERFORMANCE.md) for optimization.

## Troubleshooting

### Issue: "No such package: @maven"

**Cause:** `rules_jvm_external` not configured.

**Fix:** Add to WORKSPACE:

```python
load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    artifacts = ["com.google.guava:guava:31.1-jre"],
    repositories = ["https://repo1.maven.org/maven2"],
    maven_install_json = "//:maven_install.json",
)
```

### Issue: SBOM missing dependencies

**Cause:** Aspect not applied to all targets.

**Fix:** Rebuild with `--nocache_test_results`:

```bash
bazel build //:sbom_all --nocache_test_results
```

See [Troubleshooting Guide](docs/TROUBLESHOOTING.md) for more.

## Roadmap

- [ ] Gradle support (in addition to Maven)
- [ ] Kotlin Multiplatform support
- [ ] Container image SBOM (rules_oci integration)
- [ ] Dependency conflict auto-resolution
- [ ] Visual dependency graph UI

See [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues) for details.

## Documentation

**Getting Started:**
- [Quickstart](docs/QUICKSTART.md) - 5-minute setup
- [Usage Guide](docs/USAGE.md) - Commands and workflows

**Architecture:**
- [Architecture](docs/ARCHITECTURE.md) - System design
- [Supply Chain](docs/SUPPLY_CHAIN.md) - SBOM/SCA details
- [Threat Model](docs/THREAT_MODEL.md) - Security analysis

**Advanced:**
- [Performance](docs/PERFORMANCE.md) - Large monorepo optimization
- [Provenance](docs/PROVENANCE.md) - SLSA attestation
- [VEX](docs/VEX.md) - False positive management
- [Dependency Graphs](docs/GRAPH_ANALYSIS.md) - Visualization and queries

**Operations:**
- [Validation](docs/VALIDATION.md) - Schema validation
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues

**Decisions:**
- [ADRs](docs/ADR/) - Architecture Decision Records

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Local development setup
- Running tests
- Code style (lint with `bazel run //:lint`)
- Commit message format
- PR review process

Code of Conduct: [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)

## License

Apache License 2.0. See [LICENSE](LICENSE).

**What you can do:**
- Use commercially
- Modify and distribute
- Patent grant included

**What you must do:**
- Include license and copyright notice
- State changes made

**What you cannot do:**
- Hold authors liable
- Use trademarks without permission

Choose a License: https://choosealicense.com/licenses/apache-2.0/

## Support

- **Bug reports:** [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
- **Feature requests:** [GitHub Discussions](https://github.com/cboyd0319/BazBOM/discussions)
- **Security issues:** [SECURITY.md](SECURITY.md)
- **Maintainers:** [MAINTAINERS.md](MAINTAINERS.md)

## Status

**Active Development** - Production-ready, security-first implementation following Bazel best practices.

Last updated: 2025-10-17
