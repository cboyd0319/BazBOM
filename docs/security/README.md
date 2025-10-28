# BazBOM Security

This directory contains security-related documentation, policies, and scan results for BazBOM.

## Directory Structure

```
security/
├── README.md                          # This file
├── RISK_LEDGER.md                     # Security risk documentation
├── SECURE_CODING_GUIDE.md             # Coding best practices
├── POLICIES/                          # Security policies
│   └── semgrep/                       # Semgrep security rules
│       └── python-security.yaml       # Custom Python security rules
└── SCANNER_RESULTS/                   # Security scan outputs (gitignored)
    ├── bandit-*.sarif
    ├── semgrep-*.sarif
    ├── pip-audit-*.json
    └── safety-*.json
```

## Quick Links

### Documentation

- **[Risk Ledger](RISK_LEDGER.md)** - Current security posture, vulnerabilities, and remediation status
- **[Secure Coding Guide](SECURE_CODING_GUIDE.md)** - Python security best practices with examples
- **[Security Policy](../../SECURITY.md)** - How to report vulnerabilities
- **[Threat Model](../THREAT_MODEL.md)** - Attack vectors and mitigations

### Security Standards

- **[PYSEC_OMEGA](../copilot/PYSEC.md)** - Supreme Python security standards
- **[OWASP Top 10](https://owasp.org/www-project-top-ten/)** - Web application security
- **[CWE Top 25](https://cwe.mitre.org/top25/)** - Most dangerous software weaknesses
- **[SLSA](https://slsa.dev/)** - Supply chain security framework

## Security Tools

### Static Analysis (SAST)

```bash
# Run Bandit security scanner
bandit -r tools/supplychain -c .bandit -f sarif -o security/SCANNER_RESULTS/bandit.sarif

# Run Semgrep with custom policies
semgrep scan --config auto --config security/POLICIES/semgrep/ \
  --sarif --output security/SCANNER_RESULTS/semgrep.sarif tools/supplychain

# Run CodeQL (requires GitHub CLI)
gh codeql database create codeql-db --language=python
gh codeql database analyze codeql-db --format=sarif-latest --output=security/SCANNER_RESULTS/codeql.sarif
```

### Dependency Scanning

```bash
# Audit Python dependencies
pip-audit --format json --output security/SCANNER_RESULTS/pip-audit.json

# Check with Safety
safety check --json --output security/SCANNER_RESULTS/safety.json

# OSV Scanner
osv-scanner --format json --output security/SCANNER_RESULTS/osv.json .
```

### Secret Detection

```bash
# TruffleHog secret scanner
trufflehog filesystem . --json --no-update > security/SCANNER_RESULTS/trufflehog.json

# GitLeaks secret scanner
gitleaks detect --source . --report-path security/SCANNER_RESULTS/gitleaks.json --verbose
```

### Pre-commit Hooks

```bash
# Install pre-commit hooks
pre-commit install

# Run all hooks manually
pre-commit run --all-files

# Update hook versions
pre-commit autoupdate
```

## Security Policies

### Custom Semgrep Rules

We maintain custom security rules in `POLICIES/semgrep/python-security.yaml`:

- **dangerous-xml-parsing** - Detect XXE vulnerabilities
- **subprocess-shell-true** - Prevent command injection
- **yaml-unsafe-load** - Unsafe YAML deserialization
- **pickle-unsafe-load** - Pickle deserialization risks
- **exec-eval-usage** - Dangerous code execution
- **sql-string-concatenation** - SQL injection prevention
- **hardcoded-secret** - Secret detection
- **insecure-random** - Weak randomness
- **unvalidated-redirect** - Open redirect prevention
- **path-traversal-risk** - Directory traversal
- **logging-sensitive-data** - Secret leakage in logs
- **weak-cryptography** - MD5/SHA1 usage
- **unsafe-deserialization** - JSON deserialization
- **missing-timeout** - HTTP request timeouts

### Running Custom Rules

```bash
# Run only custom policies
semgrep scan --config security/POLICIES/semgrep/ tools/supplychain

# Run with auto rules + custom
semgrep scan --config auto --config security/POLICIES/semgrep/ tools/supplychain
```

## Security Workflows

### GitHub Actions

BazBOM uses multiple security workflows:

1. **CodeQL** (`.github/workflows/codeql.yml`)
   - Comprehensive Python security analysis
   - Runs on: push, PR, weekly schedule
   - Results: GitHub Security tab

2. **Supply Chain** (`.github/workflows/supplychain.yml`)
   - SBOM generation and signing
   - Vulnerability scanning
   - Provenance attestations
   - Runs on: push, PR, daily schedule

3. **CI** (`.github/workflows/ci.yml`)
   - Build and test validation
   - SHA-pinned actions
   - Minimal permissions
   - Runs on: push, PR

### Viewing Results

- **GitHub Security Tab**: Navigate to repository → Security → Code scanning alerts
- **Workflow Artifacts**: Download SARIF/JSON reports from workflow runs
- **Local Scans**: Run tools locally and view results in `security/SCANNER_RESULTS/`

## Security Checklist

### For Contributors

Before submitting a PR, ensure:

- [ ] Pre-commit hooks pass
- [ ] No secrets in code or commits
- [ ] Input validation for all external data
- [ ] No shell=True in subprocess calls
- [ ] Parameterized SQL queries
- [ ] defusedxml for XML parsing
- [ ] URL scheme validation
- [ ] Error messages don't leak sensitive data
- [ ] Tests cover security-critical paths
- [ ] Documentation updated

### For Reviewers

When reviewing PRs, verify:

- [ ] Security tools pass (Bandit, Semgrep, CodeQL)
- [ ] No new vulnerabilities introduced
- [ ] Security best practices followed
- [ ] Input validation present
- [ ] Error handling appropriate
- [ ] No hardcoded secrets
- [ ] Tests validate security controls
- [ ] Documentation accurate

### For Maintainers

Regular security tasks:

- [ ] Review Risk Ledger weekly
- [ ] Update security documentation
- [ ] Review Dependabot PRs
- [ ] Monitor GitHub Security alerts
- [ ] Update security tools
- [ ] Run comprehensive security audit monthly
- [ ] Update threat model quarterly

## Security Metrics

Current security posture is tracked in the [Risk Ledger](RISK_LEDGER.md):

- **Vulnerability Count**: Critical, High, Medium, Low
- **Dependency Status**: Known CVEs in dependencies
- **Test Coverage**: Security-critical code paths
- **Scan Results**: SAST, dependency, secret detection

