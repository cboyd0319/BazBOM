# Contributing to BazBOM

Thank you for your interest in contributing to BazBOM! This document provides guidelines and instructions for contributing.

## Development Environment

### Prerequisites

- [Bazelisk](https://github.com/bazelbuild/bazelisk) (recommended) or Bazel 7.0.0
- Java 11 or later (for examples)
- Python 3.12 or later (for supply chain tools)
- pip-tools (for dependency management)

### Setup

1. Clone the repository:

   ```bash
   git clone https://github.com/cboyd0319/BazBOM.git
   cd BazBOM
   ```

2. Install Python dependencies with hash verification:

   ```bash
   pip install pip-tools
   pip install -r requirements.txt --require-hashes
   pip install -r requirements-test.txt --require-hashes
   pip install -r requirements-security.txt --require-hashes
   ```

   See [Dependency Management Guide](docs/DEPENDENCY_MANAGEMENT.md) for details.

3. Install pre-commit hooks:

   ```bash
   pre-commit install
   ```

4. Build the project:

   ```bash
   bazel build //...
   ```

5. Run tests:

   ```bash
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

- Implement SPDX 3.0 schema
- Update write_sbom.py to support both 2.3 and 3.0
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
   bandit -r tools/supplychain
   pip-audit -r requirements.txt
   ```

2. **Dependency management** - Use pip-tools with hashes:
   ```bash
   # Add new dependency to .in file
   echo "package>=1.0.0" >> requirements.in
   
   # Generate locked requirements with hashes
   pip-compile --generate-hashes requirements.in
   
   # Verify no vulnerabilities
   pip-audit -r requirements.txt
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
- **Security scans must pass** (Bandit, Semgrep, CodeQL, pip-audit)
- **Dependencies must have SHA256 hashes** (use pip-tools)
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
