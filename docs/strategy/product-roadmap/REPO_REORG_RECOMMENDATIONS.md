# Repository Reorganization Recommendations

Goal: Ensure all documentation lives under `docs/`, reduce duplication, and simplify navigation without breaking developer workflows.

Proposed Moves (Docs Consolidation)
- Root → docs/
  - Quickstart lives at `docs/getting-started/quickstart.md`; keep the root README pointing at the docs version.
  - `TESTING.md` → `docs/development/test-plan.md` (consolidate content).
  - `COMPREHENSIVE_VERIFICATION.md` → merge into `docs/operations/validation.md` as a verification appendix.
  - `VERIFICATION_REPORT.md` → append to `docs/development/test-plan.md` as an appendix.
  - `SECURITY.md` → `docs/security/SECURITY.md` (canonical). Keep a minimal root stub linking to canonical copy to preserve GitHub UI integration.
  - `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `MAINTAINERS.md` → canonical copies in `docs/` with root stubs that link into docs.

- `security/` folder → `docs/security/`
  - Move: `security/*.md` into `docs/security/` keeping structure; adjust links
  - Keep non-doc assets (policies YAML, scanner outputs) in a non-doc path (e.g., `security/` or `tools/security/`) and reference from docs

- `examples/*/README.md` → `docs/examples/`
  - Create `docs/examples/` pages for each example (`maven_spring_boot`, `gradle_kotlin`, `shaded_jar`, `multi_module`, etc.)
  - Replace example folder READMEs with a one-line pointer (optional), or remove to comply strictly with docs-under-docs policy

- `tools/supplychain/sbom_schemas/README.md` → `docs/reference/schemas/README.md`
- `tools/supplychain/tests/README.md` and `tools/supplychain/tests/fixtures/README.md` → `docs/development/` (fixtures section)
- `vex/statements/README.md` → `docs/vex/README.md`
- `benchmarks/README.md` → `docs/benchmarks/README.md`

Indexing and Navigation
- Keep `docs/README.md` as the canonical docs index
- Ensure `README.md` (root) focuses on overview and links into docs

CI Enforcement (follow-up)
- Add a docs location checker in CI: fail if `**/*.md` exists outside `docs/` except for allowed root stubs
- Allowlist:
  - Root stubs: `README.md`, `CHANGELOG.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`, `MAINTAINERS.md`
  - Operational exception: `.github/copilot-instructions.md`

Risks
- GitHub UI behavior for root files; mitigated by keeping stubs linking to canonical `docs/` files
- Link rot during migration; mitigated by a link-checking workflow and mass update script

Next Steps (upon approval)
1) Move/consolidate the listed files
2) Update internal links and README references
3) Add CI guard to prevent regressions
4) Remove tracked OS cruft (e.g., `.DS_Store`) and rely on `.gitignore`
