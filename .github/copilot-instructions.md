# Copilot Instructions — BazBOM

**Purpose:** Help maintainers and Copilot keep BazBOM docs, capabilities, and workflows consistent and up to date.

## Project Overview

BazBOM is a JVM supply chain security toolkit with:
- **99 Python files**, 45,000+ lines of code, 49 test files, 90%+ coverage target
- **Universal build system support** (Maven, Gradle, Bazel)
- **SBOM generation** (SPDX 2.3, CycloneDX 1.5)
- **Vulnerability scanning** (OSV, NVD, GHSA, CISA KEV, EPSS)
- **SLSA Level 3** provenance and Sigstore signing
- **GitHub Action** for automated CI/CD integration
- **VEX support** for false positive management

## Single Source of Truth

- **Capabilities Reference:** `docs/reference/capabilities-reference.md` — Complete feature catalog
- **README:** Must link to Capabilities Reference and reflect current features
- **All user docs:** Live under `docs/` (reference, guides, ADRs, testing, copilot)

## Key Documentation

Essential guides to keep updated:
- `docs/USAGE.md` — All commands and workflows
- `docs/PROVENANCE.md` — SLSA Level 3 attestation
- `docs/VEX.md` — Vulnerability Exploitability eXchange
- `docs/PERFORMANCE.md` — Large monorepo optimization
- `docs/ARCHITECTURE.md` — System design and data flow
- `docs/copilot/PYSEC.md` — Python security engineering guide

## When Adding or Changing Features

1. Update `docs/reference/capabilities-reference.md` with:
   - New build system support
   - SBOM format additions
   - Vulnerability data sources
   - CI/CD integration changes
2. Update README.md:
   - Feature bullets in "Features" section
   - Quickstart examples if CLI changes
   - Performance benchmarks if applicable
3. Update relevant guides:
   - `docs/USAGE.md` for command changes
   - `docs/PROVENANCE.md` for SLSA changes
   - `docs/VEX.md` for VEX workflow updates
4. If GitHub Action changes:
   - Update `action.yml` inputs/outputs documentation
   - Update examples in README and docs
5. Run validation:
   ```bash
   pre-commit run --all-files  # Markdown, links, security
   pytest                      # All tests must pass
   ```

## Current Statistics (Keep Updated)

Track these in `docs/reference/capabilities-reference.md` and README.md:
- **Python Files:** 99 (verify with `find . -name "*.py" | wc -l`)
- **Lines of Code:** 45,000+ (verify with `find tools/supplychain -name "*.py" | xargs wc -l`)
- **Test Files:** 49 (verify with `find . -name "test_*.py" | wc -l`)
- **Test Coverage:** 90%+ target (from pytest-cov output)
- **Build Systems:** Maven, Gradle, Bazel (3 supported)
- **SBOM Formats:** SPDX 2.3, CycloneDX 1.5
- **Vulnerability Sources:** OSV, NVD, GHSA, CISA KEV, EPSS
- **SLSA Level:** 3 (provenance + signing)

## Documentation Style Guidelines

- **Format:** Concise, scannable bullets; line length ≤ 120 chars
- **Examples:** Runnable code snippets with expected output
- **Headings:** Use Title Case consistently
- **Links:** Prefer relative paths within repo; validate all links
- **Commands:** Include full command syntax with flags

## Build Systems & Examples Checklist

**Must cover all three build systems:**
- ✅ Maven examples (pom.xml)
- ✅ Gradle examples (build.gradle / build.gradle.kts)
- ✅ Bazel examples (WORKSPACE / MODULE.bazel)

**Additional examples:**
- Container SBOM scanning (Docker/Podman)
- VEX statements for false positive management
- GitHub Action CI/CD integration
- Offline/air-gapped mode

## Security & Supply Chain

**Keep these features documented:**
- SLSA Level 3 provenance generation
- Sigstore keyless signing
- VEX (Vulnerability Exploitability eXchange)
- Policy enforcement thresholds
- Dependency pinning strategies
- License compliance checking

**Security standards:**
- CWE mapping for vulnerabilities
- SARIF 2.1.0 output format
- SPDX 2.3 / CycloneDX 1.5 compliance
- CISA KEV integration
- EPSS risk scoring

## Sanity Checks Before Merge

- [ ] Capabilities Reference updated with new features
- [ ] README "Features" section matches Capabilities Reference
- [ ] Build system examples updated (Maven, Gradle, Bazel)
- [ ] `docs/README.md` includes link to Capabilities Reference
- [ ] No duplicate "Capabilities" docs elsewhere
- [ ] All cross-references valid (no broken links)
- [ ] GitHub Action examples tested
- [ ] Pre-commit hooks pass
- [ ] All tests pass (pytest)

## For Comprehensive Development Guide

See `docs/copilot/PYSEC.md` for:
- Python security engineering standards
- Supply chain security best practices
- GitHub Actions workflow security
- Vulnerability detection patterns
- Testing and quality requirements

Questions? Open a docs issue and tag `@cboyd0319`.
