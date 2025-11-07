# Security Review Checklist

This checklist ensures all security requirements are met before merging code changes.

## Pre-Submission Checklist (Developer)

Before submitting a PR, verify:

### Code Security

- [ ] **No hardcoded secrets** - All credentials use environment variables or secrets management
- [ ] **Input validation** - All external input is validated and sanitized
- [ ] **No SQL injection** - All database queries use parameterized statements
- [ ] **No command injection** - subprocess calls use list arguments, no shell=True
- [ ] **No path traversal** - File paths are validated and constrained
- [ ] **XXE protection** - XML parsing uses defusedxml
- [ ] **SSRF prevention** - URL schemes validated (only http/https allowed)
- [ ] **Safe deserialization** - No pickle/yaml.load on untrusted data
- [ ] **Error handling** - No sensitive data in error messages or logs
- [ ] **Type hints** - All function signatures have type hints

### Testing

- [ ] **Security tests added** - New security-sensitive code has tests
- [ ] **Test coverage ≥90%** - All new code covered by tests
- [ ] **Edge cases tested** - Boundary conditions and error paths tested
- [ ] **Test isolation** - Tests pass with pytest-randomly
- [ ] **No test secrets** - Tests use mocks/fixtures, not real credentials

### Dependencies

- [ ] **Dependencies updated** - Locked with pip-compile --generate-hashes
- [ ] **Hash verification** - All requirements have SHA256 hashes
- [ ] **No vulnerabilities** - pip-audit passes (0 vulnerabilities)
- [ ] **License compliance** - No GPL or copyleft licenses added
- [ ] **Transitive deps reviewed** - Reviewed "via" dependencies in compile log

### Code Quality

- [ ] **Linters pass** - Ruff, Black, Bandit, Semgrep all clean
- [ ] **Type checking** - mypy passes (if configured)
- [ ] **Documentation updated** - User-facing changes documented
- [ ] **Commit messages** - Follow Conventional Commits format

### GitHub Actions (if workflows changed)

- [ ] **Actions SHA-pinned** - All actions use full 40-char SHA with version comment
- [ ] **Minimal permissions** - Only required permissions granted
- [ ] **No credential persistence** - persist-credentials: false
- [ ] **Job timeouts** - All jobs have timeout-minutes
- [ ] **Input validation** - No direct interpolation of untrusted input
- [ ] **SARIF conditionals** - Upload only when file exists

## Security Review Checklist (Reviewer)

### Code Review

- [ ] **Security implications understood** - Reviewer understands attack vectors
- [ ] **Input validation sufficient** - All attack vectors considered
- [ ] **Error handling secure** - Failures don't leak sensitive data
- [ ] **Least privilege** - Code uses minimum required permissions
- [ ] **Secure defaults** - Safe behavior by default
- [ ] **Defense in depth** - Multiple layers of security

### Vulnerability Assessment

- [ ] **CWE mapping** - Relevant CWEs identified and mitigated
- [ ] **OWASP Top 10** - No new OWASP vulnerabilities introduced
- [ ] **Supply chain risks** - New dependencies vetted and justified
- [ ] **Privilege escalation** - No unintended privilege elevation
- [ ] **Data exposure** - No unintended data leakage

### Testing Validation

- [ ] **Security tests adequate** - Vulnerability scenarios tested
- [ ] **Coverage acceptable** - Critical paths have ≥95% coverage
- [ ] **Test quality** - Tests actually validate security properties
- [ ] **Negative tests** - Attack scenarios tested and blocked
- [ ] **Performance impact** - Security controls don't cause DoS

### Documentation Review

- [ ] **Security considerations documented** - Risks and mitigations noted
- [ ] **Examples secure** - Sample code follows secure patterns
- [ ] **Warnings present** - Dangerous operations clearly marked
- [ ] **CHANGELOG updated** - Security fixes noted as [security]

## Automated Checks (CI/CD)

These are automatically verified by CI:

- [x] **Bandit** - No new high/medium severity issues
- [x] **Semgrep** - No security rule violations
- [x] **CodeQL** - No new security alerts
- [x] **pip-audit** - No vulnerable dependencies
- [x] **Safety** - No known CVEs in dependencies
- [x] **OSV Scanner** - No vulnerabilities across ecosystems
- [x] **Dependency Review** - No risky dependency changes
- [x] **TruffleHog** - No secrets detected
- [x] **GitLeaks** - No credentials leaked
- [x] **Hash verification** - All requirements have hashes
- [x] **Test coverage** - ≥90% coverage maintained
- [x] **pytest-randomly** - Tests pass in random order

## Security-Sensitive Areas

Extra scrutiny required for changes in:

### Critical Components

- [ ] **Authentication/Authorization** - Privilege checks and access control
- [ ] **Cryptography** - Key management, algorithm selection
- [ ] **File operations** - Path traversal, race conditions
- [ ] **Network operations** - SSRF, TLS validation, DNS rebinding
- [ ] **Subprocess execution** - Command injection, privilege escalation
- [ ] **Data serialization** - Deserialization attacks
- [ ] **XML/JSON parsing** - XXE, injection attacks
- [ ] **Template rendering** - Template injection, XSS

### Supply Chain

- [ ] **Dependency changes** - New packages justified and vetted
- [ ] **Workflow changes** - Action pinning, permissions reviewed
- [ ] **SBOM generation** - Accuracy and completeness verified
- [ ] **Provenance** - Build integrity maintained

## Severity Classification

When security issues are found:

### Critical (P0)
- Remote code execution
- Authentication bypass
- Privilege escalation to admin
- Data breach (PII/credentials exposed)
- Supply chain compromise

**Action:** Block merge, fix immediately, security advisory

### High (P1)
- Local privilege escalation
- Denial of service (DoS)
- Information disclosure (sensitive data)
- CSRF/SSRF vulnerabilities
- Insecure deserialization

**Action:** Block merge, fix before merge, document in RISK_LEDGER

### Medium (P2)
- Input validation missing
- Path traversal (limited scope)
- Weak cryptography
- Information disclosure (low sensitivity)
- Missing rate limiting

**Action:** Fix before merge or create issue, add to RISK_LEDGER

### Low (P3)
- Security hardening opportunities
- Defense-in-depth improvements
- Security misconfigurations
- Outdated dependencies (no known exploit)

**Action:** Create issue for later, optional for current PR

### Informational
- Best practice violations
- Style/convention issues
- Documentation improvements

**Action:** Optional fix, suggest for future improvement

## Exception Process

If security requirements cannot be met:

1. **Document justification** - Why requirement cannot be met
2. **Risk assessment** - What is the risk and likelihood
3. **Compensating controls** - Alternative mitigations in place
4. **Security team approval** - Required for P0-P2 exceptions
5. **Temporary exception** - Time-bound (max 90 days)
6. **Remediation plan** - Clear path to full compliance

## Post-Merge Verification

After merge to main:

- [ ] **Production scans pass** - No new vulnerabilities in production
- [ ] **Monitoring alerts** - No security alerts triggered
- [ ] **RISK_LEDGER updated** - New risks documented
- [ ] **Security metrics** - Overall posture maintained or improved

## References

- [Secure Coding Guide](SECURE_CODING_GUIDE.md)
- [Workflow Security Policy](WORKFLOW_SECURITY_POLICY.md)
- [Risk Ledger](RISK_LEDGER.md)
- [Threat Model](threat-model.md)

## Checklist Usage

### For Contributors

1. Complete "Pre-Submission Checklist" before creating PR
2. Document any exceptions in PR description
3. Respond to reviewer security questions
4. Fix identified issues promptly

### For Reviewers

1. Complete "Security Review Checklist" during review
2. Use "Severity Classification" for any findings
3. Block merge for P0-P1 issues
4. Document decision in review comments

### For Security Team

1. Review all changes to security-sensitive areas
2. Approve/deny security exceptions
3. Update RISK_LEDGER after review
4. Monitor security metrics post-merge

---

**Last Updated:** 2025-10-20  
**Version:** 1.0.0  
**Owner:** Security Team
