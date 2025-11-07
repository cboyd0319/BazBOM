# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to the maintainers. You can find the maintainer contact information in [MAINTAINERS.md](MAINTAINERS.md).

You should receive a response within 24 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information:

- Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, XXE, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit the issue
- CWE or CVE identifiers (if applicable)

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine the affected versions
2. Audit code to find any similar problems
3. Prepare fixes for all supported versions
4. Release patches as soon as possible
5. Update the [Risk Ledger](docs/security/RISK_LEDGER.md) with findings and remediation
6. Publish security advisories for critical/high severity issues

## Security Architecture

BazBOM follows **PYSEC_OMEGA** security standards, making it one of the most secure GitHub projects. Our comprehensive security program includes:

### Code Security

- **Static Analysis (SAST)**: Bandit, Semgrep, CodeQL
- **Dependency Scanning**: pip-audit, Safety, OSV Scanner
- **Secret Detection**: TruffleHog, GitLeaks
- **Vulnerability Management**: Automated SARIF uploads to GitHub Security
- **Pre-commit Hooks**: Automated security checks before every commit

### Supply Chain Security

- **SBOM Generation**: SPDX 2.3 format for all builds
- **Provenance**: SLSA Level 2+ with cryptographic attestations
- **Signing**: Sigstore/Cosign for keyless signing with transparency logs
- **Dependency Pinning**: All dependencies pinned with version control
- **Automated Updates**: Dependabot for security patches

### GitHub Actions Security

- **SHA-Pinned Actions**: All actions pinned to full SHA256 hashes
- **Minimal Permissions**: Principle of least privilege (read-only by default)
- **OIDC Federation**: No long-lived credentials in workflows
- **Workflow Isolation**: Concurrency controls prevent race conditions
- **Artifact Security**: Signed artifacts with retention policies

### Testing & Validation

- **Test Coverage**: 90%+ requirement with branch coverage
- **Security Tests**: Dedicated tests for vulnerability fixes
- **Continuous Testing**: Automated on every PR and push
- **Mutation Testing**: Ensures test quality

## Security Tools & Scans

### Automated Security Scans

All code is automatically scanned on every commit:

| Tool | Type | Frequency | Results |
|------|------|-----------|---------|
| **CodeQL** | SAST | Push, PR, Weekly | GitHub Security Tab |
| **Bandit** | Python SAST | Push, PR | SARIF uploaded |
| **Semgrep** | Custom Rules | Push, PR | SARIF uploaded |
| **pip-audit** | Dependencies | Daily | JSON report |
| **Safety** | Dependencies | Daily | JSON report |
| **TruffleHog** | Secrets | Pre-commit | Blocks commit |
| **GitLeaks** | Secrets | Pre-commit, CI | Blocks commit |

### Manual Security Reviews

- Code review required for all changes
- Security team review for sensitive changes
- Regular security audits (monthly)
- Penetration testing (quarterly)

## Security Features

### Input Validation

All external input is validated and sanitized:

- ✅ Path traversal prevention
- ✅ Command injection prevention
- ✅ SQL injection prevention (parameterized queries)
- ✅ XXE attack prevention (defusedxml)
- ✅ SSRF prevention (URL scheme validation)

### Secure Defaults

- No shell=True in subprocess calls
- Read-only file system permissions
- Least privilege principle throughout
- Fail-secure error handling
- No secrets in logs or error messages

### Cryptography

- SHA256+ for hashing (no MD5/SHA1)
- TLS 1.2+ for all network communications
- Keyless signing with Sigstore/Cosign
- Cryptographic attestations for all builds

## Compliance & Standards

BazBOM adheres to:

- ✅ **OWASP Top 10** - All vulnerabilities addressed
- ✅ **CWE Top 25** - Critical weaknesses mitigated
- ✅ **SLSA** - Level 2+ supply chain security
- ✅ **NIST SSDF** - Secure software development framework
- ✅ **PYSEC_OMEGA** - Supreme Python security standards

## Security Documentation

Comprehensive security documentation:

- [Secure Coding Guide](docs/security/SECURE_CODING_GUIDE.md) - Security best practices
- [Risk Ledger](docs/security/RISK_LEDGER.md) - Current security posture and findings
- [Threat Model](docs/security/threat-model.md) - Attack vectors and mitigations
- [Workflow Security Policy](docs/security/WORKFLOW_SECURITY_POLICY.md) - CI/CD security

## Security Metrics

Current security posture (updated weekly):

- **Critical Vulnerabilities**: 0
- **High Vulnerabilities**: 0
- **Medium Vulnerabilities**: 0 (2 fixed, 1 false positive)
- **Dependency Vulnerabilities**: 0
- **Test Coverage**: 90%+
- **Security Scan Pass Rate**: 100%

See [Risk Ledger](docs/security/RISK_LEDGER.md) for detailed metrics and history.

## Security Contact

For security-related questions or concerns:

- **Email**: See [MAINTAINERS.md](MAINTAINERS.md)
- **Response Time**: Within 24 hours
- **Severity Classification**: Critical, High, Medium, Low
- **PGP Key**: Available upon request

## Recognition

We appreciate security researchers who responsibly disclose vulnerabilities. Contributors will be acknowledged in:

- Security advisories
- CHANGELOG.md
- Risk Ledger
- GitHub Security Hall of Fame (if applicable)

## Additional Resources

- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [CWE/SANS Top 25](https://cwe.mitre.org/top25/)
- [SLSA Framework](https://slsa.dev/)
- [Sigstore](https://www.sigstore.dev/)
- [Python Security](https://python.readthedocs.io/en/stable/library/security_warnings.html)
