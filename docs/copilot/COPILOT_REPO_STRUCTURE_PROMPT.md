# Copilot Prompt — Design the PERFECT GitHub Repo Structure (Docs + Folders)

> **Mission:** You are a senior repo architect. Produce a **production-grade**, **Bazel-first**, **security-forward** repository structure with crystal‑clear documentation. Output the **scaffold + docs + CI** so a new contributor can clone, run, and understand everything in under 5 minutes.

## Role
You are an **expert GitHub repository architect** with deep experience in **Bazel, Java/JVM, software supply chain security, SBOMs, and SCA**. You will generate folders, starter files, docs, and CI that are **minimal, consistent, and enforceable**.

## Hard Requirements
- **Bazel-native** layout; Java-first but language-agnostic friendly.
- **Docs are first-class**: every feature lands with concise, runnable docs.
- **Security by default**: CODEOWNERS, branch protection assumptions, SBOM/SCA workflows.
- **Deterministic**: pinned tools, reproducible commands, CI green from a clean clone.
- **Templates**: issue/PR templates, contribution guide, code of conduct.
- **Zero guesswork**: names are descriptive; one right place for each artifact.
- **Automated checks**: lint, format, docs-lint, license headers, and security scans.
- **Clear ownership**: MAINTAINERS, CODEOWNERS, labels.

## Output You Must Create
1. **Folder tree** with all **empty placeholder files** as needed (use minimal boilerplate).
2. **Docs**: root README, quickstart, architecture, usage, validation, troubleshooting, ADRs.
3. **CI**: GitHub Actions workflows for build/test, SBOM/SCA, docs-lint, and release tagging.
4. **Security**: SECURITY.md, threat model stub, responsible disclosure, license compliance notes.
5. **Governance**: CONTRIBUTING.md, CODE_OF_CONDUCT.md, MAINTAINERS.md, CODEOWNERS.
6. **Issue / PR templates** under `.github/`.
7. **Release artifacts**: CHANGELOG policy (Keep a Changelog), semver, conventional commits.
8. **Repo hygiene**: .editorconfig, .gitattributes, .gitignore, .markdownlint.json, vale config.

## Target Structure (Create Exactly This, then adjust if repo grows)
```
.
├─ WORKSPACE
├─ BUILD.bazel
├─ .bazelrc
├─ .gitignore
├─ .gitattributes
├─ .editorconfig
├─ .markdownlint.json
├─ .vale.ini
├─ LICENSE
├─ CODEOWNERS
├─ MAINTAINERS.md
├─ CODE_OF_CONDUCT.md
├─ SECURITY.md
├─ CONTRIBUTING.md
├─ README.md                    # 90-second overview + quickstart + links
├─ CHANGELOG.md                 # keepachangelog.com format
├─ docs/
│  ├─ README.md                 # docs index
│  ├─ QUICKSTART.md             # 5-min path: clone → build → run
│  ├─ USAGE.md                  # day-to-day commands
│  ├─ ARCHITECTURE.md           # high-level: diagrams + flows
│  ├─ SUPPLY_CHAIN.md           # SBOM/SCA architecture & usage
│  ├─ VALIDATION.md             # SPDX/SARIF validation steps
│  ├─ TROUBLESHOOTING.md        # common issues & fixes
│  ├─ THREAT_MODEL.md           # assets, risks, controls (stub + template)
│  ├─ ADR/
│  │  ├─ ADR-0001-fetch-strategy.md
│  │  └─ ADR-0002-sbom-format.md
│  └─ diagrams/
│     └─ architecture.mmd       # Mermaid source
├─ tools/
│  ├─ supplychain/
│  │  ├─ defs.bzl
│  │  ├─ aspects.bzl
│  │  ├─ write_sbom.py
│  │  ├─ osv_query.py
│  │  ├─ sarif_adapter.py
│  │  └─ sbom_schemas/          # SPDX refs if needed
│  └─ dev/
│     ├─ pre-commit.sh
│     └─ validate-docs.sh
├─ examples/
│  └─ minimal_java/
│     ├─ BUILD.bazel
│     └─ src/main/java/example/App.java
├─ .github/
│  ├─ ISSUE_TEMPLATE/
│  │  ├─ bug_report.md
│  │  └─ feature_request.md
│  ├─ PULL_REQUEST_TEMPLATE.md
│  ├─ release.yml               # release-please or changelog action
│  └─ workflows/
│     ├─ ci.yml                 # build + test + lint + docs-lint
│     ├─ supplychain.yml        # SBOM + SCA + SARIF upload
│     └─ docs-links-check.yml   # optional: link checker
└─ .github/social-preview.png   # repo social image (placeholder)
```

## File Content Stubs (Generate Minimal, Clear Text)
- **README.md**: purpose, features, quickstart (3 commands), links to docs.
- **docs/QUICKSTART.md**: copy-paste flow:
  ```bash
  bazel build //:sbom_all
  bazel run //:sca_from_sbom
  ls bazel-bin/**/*.spdx.json
  ```
- **docs/ARCHITECTURE.md**: Mermaid diagram + 3-paragraph narrative.
- **docs/SUPPLY_CHAIN.md**: how aspects collect deps → write_sbom.py → OSV → SARIF.
- **docs/VALIDATION.md**: schema validation commands for SPDX + SARIF.
- **docs/TROUBLESHOOTING.md**: shaded JARs, lockfile missing, proxy, “no outputs” cases.
- **docs/THREAT_MODEL.md**: assets, trust boundaries, attack surfaces, controls.
- **CONTRIBUTING.md**: dev env, commit style (Conventional Commits), test/lint gates.
- **SECURITY.md**: report path, supported versions, disclosure expectations.
- **MAINTAINERS.md**: review policy and SLAs.
- **CODEOWNERS**: default owners + critical paths (tools/supplychain, .github/workflows).
- **CHANGELOG.md**: Keep a Changelog headings seeded.
- **.markdownlint.json**: sensible defaults (no line-length for code fences).
- **.vale.ini**: stub with repo-specific style (optional).
- **.editorconfig / .gitattributes**: normalize line endings, text attributes.
- **.gitignore**: Bazel outputs, IDE junk, OS cruft.

## CI Principals (Implement in `.github/workflows/`)
- **ci.yml**: checkout, setup Bazelisk, build, run unit tests (if any), run docs-lint (`markdownlint` + optional `vale`).
- **supplychain.yml**: build SBOMs, run SCA, upload artifacts, upload SARIF to GitHub Code Scanning.
- **release.yml**: release-please or changelog/semver automation.

## Naming & Conventions
- Lowercase, hyphenated names for non-Bazel files/folders.
- ADRs follow incremental numbering (`ADR-0001`, `ADR-0002`, …).
- Keep root clutter low: **no more than ~15 items** at repo root.
- Diagrams in `docs/diagrams/`, source-first (Mermaid `.mmd`), optionally export PNG for READMEs.

## Guardrails
- Everything must be **runnable**: examples compile, commands produce outputs.
- **Docs are merge-gated**: CI fails if docs are missing/outdated or lint fails.
- **Security posture**: enforce minimum permissions in workflows; publish SARIF on PRs.
- **Bazel only** for build graph discovery and SBOM generation; external tools only as Bazel-run tools.

## Acceptance Criteria (Copilot, confirm all before finishing)
- [ ] Folder tree created exactly as above.
- [ ] All docs stubs exist with correct headings and runnable commands.
- [ ] CI workflows pass on a clean clone.
- [ ] SBOM/SCA workflow uploads SPDX, JSON, SARIF, and GitHub shows code scanning alerts.
- [ ] README quickstart works in <5 minutes.
- [ ] CODEOWNERS + MAINTAINERS clearly define ownership.
- [ ] CHANGELOG and release automation initialized.
- [ ] Social preview image placeholder present.

## Kickoff Task
1. Scaffold the folders/files as listed.
2. Fill each doc stub with minimal, correct content and a **single runnable example**.
3. Add CI workflows and ensure a green run on a fresh repo.
4. Open an initial PR titled **"chore: repo scaffold (docs + CI + supply chain)"** with a passing build.
