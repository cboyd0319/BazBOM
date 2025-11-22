# Repository Guidelines

## Project Structure & Module Organization
- Rust workspace in `crates/`: CLI (`bazbom`), shared model (`bazbom-core`), graph normalization (`bazbom-graph`), vulnerability intelligence (`bazbom-vulnerabilities`), policy engine (`bazbom-policy`), formats/exporters (`bazbom-formats`), reachability (`bazbom-reachability`), container scanning (`bazbom-containers`), upgrade intelligence (`bazbom-upgrade-analyzer`), UI surfaces (`bazbom-dashboard`, `bazbom-tui`, IDE/LSP crates).
- Build system glue: Bazel files (`BUILD.bazel`, `MODULE.bazel`, `WORKSPACE`, `rules/`), Maven/Gradle plugins in `plugins/`, shared tooling in `tools/` and `scripts/`.
- Feature docs and matrices live in `docs/ARCHITECTURE.md`, `docs/CAPABILITY_MATRIX.md`, and `docs/features/` (container scanning, upgrade intelligence, advanced security, JAR identity); keep new docs under `docs/` and link from `docs/README.md`.
- Integration assets: `examples/`, fuzzing in `fuzz/`, platform packaging in `windows/`, `homebrew/`; tests live in `tests/` plus per-crate suites.

## Build, Test, and Development Commands
- `make build` / `make dev` — release or debug CLI build (`target/release|debug/bazbom`).
- `make test` — all Rust tests; `make quick` targets the CLI crate for fast cycles.
- `make check` — fmt + clippy + tests; required before PRs.
- Bazel integration: `bazel build //...` and `bazel test //...` when touching Bazel rules/aspects.
- Linters: `cargo fmt --all -- --check`, `cargo clippy --all --all-targets --all-features -- -D warnings`.
- Coverage/insights: `cargo llvm-cov --all-features --workspace`; `cargo audit` for deps; reachability JAR tool in `tools/reachability/`.

## Coding Style & Naming Conventions
- Rust: `rustfmt` defaults, `snake_case` for functions/modules, `CamelCase` types, `SCREAMING_SNAKE_CASE` constants; keep public APIs documented when surfaced across crates (graph, reachability, containers, upgrade intelligence).
- Bazel/Starlark: run `buildifier`; descriptive labels (`//service/api:server`), keep aspects/policies consistent with `tools/supplychain/aspects.bzl`.
- Config (TOML/JSON/YAML): lowercase keys; mirror upstream schema names. Keep security policies and capability matrices in sync with `docs/CAPABILITY_MATRIX.md`.
- No emojis in code/comments; docs can use minimal icons already present in feature docs.

## Testing Guidelines
- Add unit/integration tests next to crates; shared fixtures in `tests/`. Mirror ecosystem coverage from `docs/CAPABILITY_MATRIX.md` when adding parsers or exporters.
- Prefer explicit names (`handles_missing_lockfile`, `prioritizes_reachable_cves`); cover reachability, container multi-tool merges, and upgrade intelligence edge cases when touching those areas.
- For Bazel changes, include `bazel test //...`; for plugins, add sample projects under `examples/`.
- Generate coverage with `cargo llvm-cov` for critical paths; run `cargo audit` after dependency changes.

## Commit & Pull Request Guidelines
- Use Conventional Commits (e.g., `feat: add reachability cache metrics`, `fix: container layer attribution sorting`); keep subjects ~72 chars.
- Before PRs: `make check`, `bazel test //...` if Bazel/Gradle/Maven integrations changed, and update relevant feature docs in `docs/features/` plus `docs/ARCHITECTURE.md` or `docs/CAPABILITY_MATRIX.md` when altering capabilities/status.
- PRs should link issues, summarize behavior changes, include CLI/TUI screenshots or sample command output when UX shifts, and note reachability/container/upgrade impacts.
- Security: run `cargo audit`; ensure pre-commit hooks stay green and no secrets leak.
- When networked tools or external access are needed (e.g., crates.io, container registries), request approval before running; explicitly note the target and purpose so access can be granted.
