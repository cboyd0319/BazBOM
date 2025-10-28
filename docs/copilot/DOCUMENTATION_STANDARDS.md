# Documentation Standards

Policy: All documentation resides under `docs/`.

Exceptions
- Root `README.md` remains as the project landing page and links into `docs/`.
- If GitHub requires conventional files (e.g., `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `SECURITY.md`), create minimal root stubs that link to the canonical copies in `docs/`. Treat stubs as pointers, not documentation.

Structure
- H1 title on first line; clear summary paragraph next
- Order: Context → How-to → Reference → Troubleshooting
- Use relative links; avoid duplicate documents
- ADRs in `docs/ADR/` with sequential IDs and status
- Images in `docs/images/`; diagrams in `docs/diagrams/`

Naming
- Concept/reference: `UPPER_SNAKE_CASE.md` (e.g., `TEST_PLAN.md`)
- Guides/how-to: `lower-kebab-case.md` (e.g., `gradle-plugin-guide.md`)

Style
- Markdownlint + Vale enforced in CI
- Active voice; short sentences; avoid weasel words; consistent terminology
- Commands in fenced blocks; file paths and code in backticks

Prohibited
- Zero emojis in code, ever. Do not include emojis in source files, generated code, code comments, or code examples intended for copy/paste.

Doc Creation Criteria
- Prefer updating existing canonical docs over creating new ones.
- Create a new doc only when there is a clear, reusable gap (new concept, feature, or reference area).
- New docs must be linked from `docs/README.md` and organized in the appropriate subfolder (e.g., `docs/examples/`, `docs/reference/`, `docs/security/`).
- Avoid one‑off documents tied to a single PR or task; integrate content into existing docs.

Quality Gates
- New or changed docs must pass link checks and style checks
- Cross-links must be valid; no orphaned pages

Migration Plan
- Consolidate all Markdown currently outside `docs/` into `docs/`
- Keep root stubs where necessary for GitHub conventions only
