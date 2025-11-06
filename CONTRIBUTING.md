# Contributing to BazBOM

Thank you for your interest in contributing to BazBOM! This document provides guidelines and instructions for contributing.

## Development Environment

### Prerequisites

- Follow [Local Development Environment](docs/development/local-environment-setup.md) for complete setup instructions
- Rust (stable) via `rustup`, including `rustfmt`, `clippy`, and `llvm-tools-preview`
- [Bazelisk](https://github.com/bazelbuild/bazelisk) (recommended Bazel wrapper) with Bazel-compatible JDK 17
- Java build tooling: OpenJDK 17, Maven, and Gradle
- Node.js (LTS) and npm for IDE integrations
- Pre-commit dependencies: `pre-commit`, `trufflehog`, `gitleaks`, `markdownlint`, `buildifier`, and `vale`

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/cboyd0319/BazBOM.git
   cd BazBOM
   ```

2. Build the Rust workspace to pull dependencies:

   ```bash
   cargo build --all --locked
   ```

3. Install git hooks:

   ```bash
   pre-commit install
   ```

4. Run linters and formatters:

   ```bash
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features -- -D warnings
   pre-commit run --all-files
   ```

5. Run tests:

   ```bash
   cargo test --all --locked
   ```

6. Build with Bazel (for Bazel integrations and examples):

   ```bash
   bazel build //...
   bazel test //...
   ```

## Commit Style

We follow [Conventional Commits](https://www.conventionalcommits.org/) for commit messages:

- `feat:` for new features
- `fix:` for bug fixes
- `docs:` for documentation changes
- `chore:` for maintenance tasks
- `test:` for test changes
- `refactor:` for code refactoring

Example:

```text
feat: add SPDX 3.0 support to SBOM generation

- Implement SPDX 3.0 schema in bazbom-formats crate
- Update SBOM generation to support both 2.3 and 3.0
- Add validation for SPDX 3.0 format
```

## Pull Request Process

1. Fork the repository and create your branch from `main`
2. Make your changes and ensure they follow the project style
3. Add or update tests as needed
4. Run linters and formatters:

   ```bash
   bazel run //tools/dev:lint
   ```

5. Ensure all tests pass:

   ```bash
   bazel test //...
   ```
6. Update documentation if needed
7. Submit a pull request with a clear description

## Security Requirements

BazBOM follows **PYSEC_OMEGA** security standards. All contributions must meet these requirements:

### Mandatory Security Checks

1. **No security vulnerabilities** - Run security scans before submitting:
   ```bash
   cargo audit
   cargo clippy -- -D warnings
   ```

2. **Dependency management** - Use Cargo.lock for reproducible builds:
   ```bash
   # Add new dependency to Cargo.toml
   cargo add package@1.0.0
   
   # Update and audit dependencies
   cargo update
   cargo audit
   ```

3. **Input validation** - Validate ALL external input
4. **Secure defaults** - Safe by default, opt-in to risky behavior
5. **No secrets** - Never commit secrets, keys, or credentials
6. **Code review** - Security-sensitive changes require security team review

See [Secure Coding Guide](docs/security/SECURE_CODING_GUIDE.md) for detailed guidelines.

## Project Rules (Critical)

- Zero emojis in code, ever. Do not add emojis to source files, generated code, or code comments. Code examples intended for copy/paste must be emoji‑free.
- Avoid documentation sprawl. Do not create a new document for each small change. Update canonical docs under `docs/` whenever possible. Create new docs only when a clear gap exists, and add them to `docs/README.md`.
- All canonical documentation must live under `docs/` (root files like `README.md`, `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `CHANGELOG.md`, `MAINTAINERS.md` act as stubs/entry points).

## Code Review

All submissions require review. We use GitHub pull requests for this purpose. Consult
[GitHub Help](https://help.github.com/articles/about-pull-requests/) for more information on using pull requests.

### Review Guidelines

- Code must pass all CI checks
- Documentation must be updated for user-facing changes
- Tests must be included for new functionality
- **Security scans must pass** (cargo-audit, clippy, CodeQL)
- **Dependencies must be audited** (cargo audit)
- Security implications must be documented

## Testing

- Write unit tests for new code
- Ensure existing tests pass
- Add integration tests for complex features
- Document test scenarios

## Documentation

- Update relevant documentation in `/docs` for user-facing changes
- Use clear, concise language
- Include runnable examples
- Follow the existing documentation style

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
