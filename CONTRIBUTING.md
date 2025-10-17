# Contributing to BazBOM

Thank you for your interest in contributing to BazBOM! This document provides guidelines and instructions for contributing.

## Development Environment

### Prerequisites

- [Bazelisk](https://github.com/bazelbuild/bazelisk) (recommended) or Bazel 7.0.0
- Java 11 or later (for examples)
- Python 3.9 or later (for supply chain tools)

### Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/cboyd0319/BazBOM.git
   cd BazBOM
   ```

2. Build the project:
   ```bash
   bazel build //...
   ```

3. Run tests:
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
```
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

## Code Review

All submissions require review. We use GitHub pull requests for this purpose. Consult
[GitHub Help](https://help.github.com/articles/about-pull-requests/) for more information on using pull requests.

### Review Guidelines

- Code must pass all CI checks
- Documentation must be updated for user-facing changes
- Tests must be included for new functionality
- Security implications must be considered

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
