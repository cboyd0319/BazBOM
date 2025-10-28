# GitHub Actions Workflow Security Policy

**Last Updated:** 2025-10-20  
**Policy Version:** 1.0.0  
**Authority:** PYSEC_OMEGA Security Standards

## Purpose

This document defines mandatory security requirements for all GitHub Actions workflows in the BazBOM repository. These policies align with industry best practices and PYSEC_OMEGA security engineering standards.

## Mandatory Requirements

### 1. Action Pinning (CRITICAL)

**Requirement:** ALL GitHub Actions MUST be pinned to full 40-character SHA hashes with version comments.

**Format:**
```yaml
- uses: actions/checkout@692973e3d937129bcbf40652eb9f2f61becf3332 # v4.1.7
```

**Rationale:**
- Tag-based references can be moved by attackers who compromise action repositories
- SHA pinning ensures immutability and prevents supply chain attacks
- Version comments provide human readability

**Enforcement:**
- Pre-commit hooks check for unpinned actions
- CI validation fails on unpinned actions
- Dependabot automatically updates pinned SHAs weekly

**Exceptions:**
- None. This rule has no exceptions.

### 2. Minimum Permissions (CRITICAL)

**Requirement:** ALL workflows and jobs MUST use minimum required permissions.

**Default workflow permissions:**
```yaml
permissions:
  contents: read
```

**Per-job permissions:**
```yaml
jobs:
  security-scan:
    permissions:
      contents: read
      security-events: write  # Only for SARIF upload
```

**Prohibited:**
- `permissions: write-all`
- `permissions: {}` with inherited write access
- Overly broad permissions

**Rationale:**
- Limits blast radius of compromised workflows
- Prevents unauthorized repository modifications
- Reduces risk of token theft/abuse

### 3. Credential Handling

**Requirements:**

**a) Never persist credentials:**
```yaml
- uses: actions/checkout@SHA
  with:
    persist-credentials: false
```

**b) Use OIDC for cloud providers:**
```yaml
permissions:
  id-token: write  # For OIDC federation
  contents: read

- uses: aws-actions/configure-aws-credentials@SHA
  with:
    role-to-assume: arn:aws:iam::ACCOUNT:role/ROLE
    aws-region: us-east-1
```

**c) Never log secrets:**
- Use GitHub's automatic secret masking
- Avoid echoing variables that might contain secrets
- Use intermediate variables for untrusted input

**d) Rotate secrets regularly:**
- Long-lived tokens: Rotate every 90 days
- CI/CD tokens: Rotate after security incidents
- Use environment protection for production

### 4. Input Validation & Injection Prevention

**Requirement:** NEVER interpolate untrusted input directly in `run:` blocks.

**VULNERABLE:**
```yaml
# ❌ WRONG - Command injection risk
- run: echo "Title: ${{ github.event.issue.title }}"
```

**SECURE:**
```yaml
# ✅ CORRECT - Use environment variables
- name: Safe example
  env:
    ISSUE_TITLE: ${{ github.event.issue.title }}
  run: echo "Title: $ISSUE_TITLE"
```

**Rationale:**
- Prevents workflow injection attacks
- Protects against command injection
- Mitigates arbitrary code execution

**Untrusted inputs include:**
- `github.event.issue.*`
- `github.event.pull_request.*`
- `github.event.comment.*`
- `github.head_ref` (on `pull_request_target`)

### 5. Third-Party Action Vetting

**Requirement:** ALL third-party actions MUST be reviewed before use.

**Review checklist:**
- [ ] Action source code reviewed for security issues
- [ ] Action maintainer reputation verified
- [ ] Action has security policy (SECURITY.md)
- [ ] Action actively maintained (commits within 6 months)
- [ ] Action has no known CVEs
- [ ] Action permissions documented and justified

**Trusted action sources:**
- `actions/*` (GitHub official)
- `github/*` (GitHub official)
- Organizations with strong security track record

**Approval process:**
1. Security team review required for new third-party actions
2. Document justification in PR description
3. Add to approved actions list after review

### 6. Artifact Security

**Requirements:**

**a) Validate artifact checksums:**
```yaml
- name: Verify artifact
  run: |
    sha256sum -c artifact.sha256
```

**b) Sign artifacts with Sigstore:**
```yaml
- uses: sigstore/gh-action-sigstore-python@SHA
  with:
    inputs: ./dist/*.whl
```

**c) Set retention limits:**
```yaml
- uses: actions/upload-artifact@SHA
  with:
    retention-days: 30  # Maximum for non-releases
```

**d) Check for file existence:**
```yaml
- uses: actions/upload-artifact@SHA
  if: always()
  with:
    if-no-files-found: warn  # Don't fail silently
```

### 7. Job Isolation & Timeouts

**Requirement:** ALL jobs MUST have appropriate timeouts and isolation.

**Timeout configuration:**
```yaml
jobs:
  test:
    timeout-minutes: 30  # Prevent hung jobs
```

**Concurrency control:**
```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true  # For PR builds
```

**Rationale:**
- Prevents resource exhaustion
- Limits cost of runaway jobs
- Ensures predictable execution

### 8. SARIF Upload Best Practices

**Requirement:** SARIF uploads MUST handle missing files gracefully.

**Correct pattern:**
```yaml
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@SHA
  if: always() && hashFiles('results.sarif') != ''
  with:
    sarif_file: results.sarif
    category: tool-name
  continue-on-error: true  # Don't block on upload failures
```

**Rationale:**
- Prevents workflow failures when scan produces no findings
- Ensures SARIF only uploaded when file exists
- Allows workflow to complete even if upload fails

### 9. Cache Security

**Requirements:**

**a) Key caches by content hash:**
```yaml
- uses: actions/cache@SHA
  with:
    key: ${{ runner.os }}-pip-${{ hashFiles('requirements.txt') }}
```

**b) Never cache secrets or credentials**

**c) Use read-only mode for untrusted PRs:**
```yaml
- uses: actions/cache@SHA
  with:
    lookup-only: true  # Read-only mode
```

### 10. Branch Protection & Required Checks

**Requirements:**

**a) Main branch protection:**
- Require status checks before merge
- Require code review (minimum 1 approver)
- Require signed commits
- Restrict push access
- Dismiss stale approvals

**b) Required status checks:**
- All tests passing
- Security scans clean
- Documentation lint passing
- SBOM generation successful

## Workflow-Specific Policies

### CodeQL Workflows

**Requirements:**
- Run on push to main and all PRs
- Use `security-extended` and `security-and-quality` queries
- Schedule weekly scans
- Upload SARIF to GitHub Security tab

### Dependency Review

**Requirements:**
- Run on all PRs
- Block high/critical vulnerabilities
- Check license compatibility
- Use `actions/dependency-review-action`

### SBOM Generation

**Requirements:**
- Generate SPDX 2.3 format
- Include all transitive dependencies
- Sign SBOMs with Sigstore
- Upload as release artifacts
- Retain for 90 days

### Supply Chain Security

**Requirements:**
- Daily scheduled scans
- OSV vulnerability database queries
- SLSA provenance generation
- VEX statement processing
- Metrics aggregation

## Incident Response

### Compromised Workflow Detection

**Indicators:**
- Unexpected permissions elevation
- New third-party actions without approval
- Secrets accessed by unauthorized jobs
- Unusual artifact uploads
- Modified workflow files from external contributors

**Response procedure:**
1. Immediately revoke compromised tokens
2. Review audit logs for scope of compromise
3. Disable affected workflows
4. Roll back to last known good state
5. Conduct security review of all changes
6. Document incident in security log

### Workflow Failure Handling

**For security scan failures:**
- Never disable security checks
- Investigate root cause
- Fix issues, don't suppress warnings
- Document false positives in VEX statements

## Compliance & Auditing

### Automated Checks

- **Pre-commit:** Action pinning validation
- **CI:** Workflow policy enforcement
- **Weekly:** Dependabot security updates
- **Monthly:** Security policy review

### Manual Reviews

- **Quarterly:** Third-party action security review
- **Annually:** Comprehensive workflow security audit
- **On-demand:** New workflow approval process

### Documentation Requirements

- All workflows MUST have header comments explaining purpose
- Security-sensitive workflows MUST document threat model
- Non-standard permissions MUST be justified in comments
- Exceptions to policy MUST be documented and approved

## Policy Enforcement

### Violations

**Critical violations (block merge):**
- Unpinned actions
- Overly broad permissions
- Persisted credentials
- Direct input interpolation

**Major violations (require review):**
- Missing timeouts
- Unvetted third-party actions
- Improper artifact handling
- Missing SARIF conditionals

**Minor violations (warning):**
- Missing version comments
- Suboptimal cache keys
- Redundant permissions

### Exception Process

1. Document exception justification
2. Security team review required
3. Time-bound approval (max 90 days)
4. Add compensating controls
5. Plan remediation timeline

## Resources

- [GitHub Actions Security Guides](https://docs.github.com/en/actions/security-guides)
- [PYSEC_OMEGA Documentation](../docs/copilot/PYSEC.md)
- [SLSA Framework](https://slsa.dev/)
- [Sigstore Project](https://www.sigstore.dev/)
- [SARIF Specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/)

## Change History

| Version | Date | Changes | Author |
|---------|------|---------|--------|
| 1.0.0 | 2025-10-20 | Initial policy creation | PYSEC_OMEGA |

## Approval

This policy is approved for use and enforcement across all BazBOM workflows.

**Policy Owner:** Security Team  
**Review Frequency:** Quarterly  
**Next Review:** 2026-01-20
