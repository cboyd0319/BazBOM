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

BazBOM is built with Rust for memory safety and follows industry-leading security practices. Our comprehensive security program includes:

### Code Security

- **Static Analysis (SAST)**: cargo-clippy, cargo-audit, CodeQL, Semgrep
- **Dependency Scanning**: cargo-audit (RustSec Advisory Database), OSV Scanner
- **Secret Detection**: TruffleHog, GitLeaks
- **Vulnerability Management**: Automated SARIF uploads to GitHub Security
- **Zero Unsafe Code**: 100% memory-safe Rust across all crates (except where required with justification)

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
| **cargo-clippy** | Rust Lints | Push, PR | CI logs (enforced with -D warnings) |
| **cargo-audit** | Dependencies | Daily, Push | JSON report (RustSec advisories) |
| **Semgrep** | Custom Rules | Push, PR | SARIF uploaded |
| **TruffleHog** | Secrets | Pre-commit | Blocks commit |
| **GitLeaks** | Secrets | Pre-commit, CI | Blocks commit |

### Manual Security Reviews

- Code review required for all changes
- Security team review for sensitive changes
- Regular security audits (monthly)
- Penetration testing (quarterly)

## Security Features

### Authentication & Authorization (v7.0+)

**Enterprise-Ready Access Control**:
- ✅ **JWT Authentication** - RFC 7519 compliant tokens with 24h expiration
- ✅ **RBAC** - Role-Based Access Control with 5 roles (Admin, SecurityLead, Developer, User, CI)
- ✅ **API Keys** - Long-lived keys for CI/CD with scoped permissions
- ✅ **Rate Limiting** - 100 requests/minute per endpoint to prevent DoS
- ✅ **Audit Logging** - Comprehensive security event logging with HMAC signatures
- ✅ **OS Keychain** - Secure credential storage on macOS/Windows/Linux

### Data Protection (v7.0+)

**Multi-Layer Data Security**:
- ✅ **Encryption at Rest** - ChaCha20-Poly1305 AEAD for sensitive data
- ✅ **Encryption in Transit** - TLS 1.3 (1.2 minimum) for all communications
- ✅ **Secure Memory** - Automatic cleanup with zeroize on key material
- ✅ **Constant-Time Operations** - Timing attack prevention in crypto operations

### Input Validation

All external input is validated and sanitized:

- ✅ Path traversal prevention (path canonicalization)
- ✅ Command injection prevention (safe APIs only)
- ✅ Memory safety (guaranteed by Rust)
- ✅ Buffer overflow prevention (guaranteed by Rust)
- ✅ SSRF prevention (URL scheme validation)
- ✅ File size limits (10MB max for SBOMs - DoS prevention)
- ✅ XSS prevention (strict CSP without unsafe-inline)

### Secure Defaults

- Command execution through safe APIs only
- Read-only file system permissions
- Least privilege principle throughout (RBAC)
- Fail-secure error handling
- No secrets in logs or error messages
- Zero unsafe code without explicit justification
- Authentication required by default for dashboard (v7.0+)
- TLS enforced for production deployments

### Cryptography

**Enterprise-Grade Cryptography (v7.0+)**:
- **ChaCha20-Poly1305** AEAD encryption for sensitive data (256-bit keys)
- **JWT Authentication** (RFC 7519) with HMAC-SHA256
- **bcrypt** password hashing for API keys (cost factor 12)
- **SHA256** for integrity verification and checksums
- **HMAC-SHA256** for audit log tamper-evidence
- **TLS 1.3** for network communications (1.2 minimum)
- Keyless signing with Sigstore/Cosign
- Cryptographic attestations for all builds
- OS keychain integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)

## Compliance & Standards

BazBOM adheres to:

- ✅ **OWASP Top 10 2025** - All categories addressed including new A03 (Software Supply Chain Failures) and A10 (Mishandling of Exceptional Conditions)
- ✅ **CWE Top 25 2024** - Critical weaknesses mitigated (released Nov 2024)
- ✅ **SLSA v1.1** - Level 3 supply chain security (upgrading to Level 4)
- ✅ **NIST Cybersecurity Framework 2.0** - 6-function framework (Identify, Protect, Detect, Respond, Recover, Govern)
- ✅ **NIST SSDF** - Secure software development framework
- ✅ **Memory Safety** - 100% Rust with zero unsafe code (except where required with justification)
- 🚧 **SOC 2 Type II** - In preparation (target: Q2 2026) - See [SOC2_PREPARATION.md](docs/compliance/SOC2_PREPARATION.md)
- 🚧 **GDPR** - Implementation in progress (target: Q2 2026) - See [GDPR_COMPLIANCE.md](docs/compliance/GDPR_COMPLIANCE.md)
- 🚧 **ISO 27001** - Planned (target: Q3 2026)

## Security Documentation

Comprehensive security documentation:

- [Security Overview](docs/security/SECURITY_OVERVIEW.md) - **NEW v7.0** - Complete security architecture
- [Secure Coding Guide](docs/security/SECURE_CODING_GUIDE.md) - Security best practices
- [Risk Ledger](docs/security/RISK_LEDGER.md) - Current security posture and findings
- [Threat Model](docs/security/threat-model.md) - Attack vectors and mitigations
- [Workflow Security Policy](docs/security/WORKFLOW_SECURITY_POLICY.md) - CI/CD security
- [JWT Authentication](docs/security/JWT_AUTHENTICATION.md) - Authentication architecture

### Compliance Documentation

- [SOC 2 Preparation](docs/compliance/SOC2_PREPARATION.md) - **NEW v7.0** - SOC 2 Type II certification path
- [GDPR Compliance](docs/compliance/GDPR_COMPLIANCE.md) - **NEW v7.0** - GDPR implementation guide

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

- [OWASP Top 10 2025](https://owasp.org/Top10/) - Latest web application security risks
- [OWASP Secure Coding Practices](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/)
- [CWE Top 25 2024](https://cwe.mitre.org/top25/) - Most dangerous software weaknesses (Nov 2024)
- [SLSA Framework v1.1](https://slsa.dev/) - Supply chain security levels
- [NIST Cybersecurity Framework 2.0](https://www.nist.gov/cyberframework) - Released Feb 2024
- [Sigstore](https://www.sigstore.dev/) - Keyless signing and transparency
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [RustSec Advisory Database](https://rustsec.org/)
