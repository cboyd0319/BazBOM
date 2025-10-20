# Security Incident Response Plan

**Document Version:** 1.0.0  
**Last Updated:** 2025-10-20  
**Owner:** Security Team

## Purpose

This document defines procedures for responding to security incidents in the BazBOM project.

## Scope

This plan covers:
- Security vulnerabilities in BazBOM code
- Compromised credentials or secrets
- Supply chain attacks
- Unauthorized access
- Data breaches
- Denial of service
- Workflow compromises

## Incident Classification

### Severity Levels

#### P0 - Critical
- **Impact:** Immediate threat to production systems or data
- **Examples:** Remote code execution, authentication bypass, active data breach
- **Response Time:** Immediate (< 1 hour)
- **Escalation:** Security team + maintainers notified immediately

#### P1 - High
- **Impact:** Significant security risk, not immediately exploited
- **Examples:** Privilege escalation, SSRF, insecure deserialization
- **Response Time:** Within 4 hours
- **Escalation:** Security team notified, maintainers informed

#### P2 - Medium
- **Impact:** Limited security risk with mitigating factors
- **Examples:** Path traversal (limited scope), information disclosure
- **Response Time:** Within 24 hours
- **Escalation:** Security team handles, maintainers informed

#### P3 - Low
- **Impact:** Minor security concern, no immediate risk
- **Examples:** Missing rate limiting, weak cryptography (non-critical)
- **Response Time:** Within 1 week
- **Escalation:** Security team handles, no immediate escalation

## Incident Response Phases

### 1. Detection

#### Automated Detection

**Monitoring Systems:**
- GitHub Security Advisories
- Dependabot alerts
- CodeQL alerts
- SARIF upload failures
- Workflow failures
- pip-audit reports
- OSV Scanner alerts

**Alert Channels:**
- GitHub notifications
- Email to security team
- CI/CD failure notifications

#### Manual Detection

**Reporting Channels:**
- Security email (see SECURITY.md)
- Private vulnerability reports (GitHub)
- Direct messages to maintainers

**Reporter Information Needed:**
- Vulnerability type (CWE/CVE)
- Affected components
- Reproduction steps
- Proof of concept (if safe)
- Impact assessment

### 2. Triage

**Immediate Actions (< 1 hour):**

1. **Acknowledge receipt** - Confirm we received the report
2. **Initial assessment** - Determine severity (P0-P3)
3. **Containment decision** - Can we contain it now?
4. **Team assembly** - Notify appropriate responders

**Assessment Questions:**

- [ ] Is this actively exploited?
- [ ] What is the blast radius?
- [ ] Are credentials compromised?
- [ ] Is data exposed?
- [ ] Can we contain it quickly?
- [ ] Do we need to notify users?

**Triage Decision Tree:**

```
Is it actively exploited? 
  ├─ YES → P0, immediate response
  └─ NO
      ├─ RCE/Auth bypass/Data breach possible? → P1
      ├─ Privilege escalation/SSRF possible? → P1
      ├─ Information disclosure/Path traversal? → P2
      └─ Hardening opportunity? → P3
```

### 3. Containment

**Immediate Containment (P0/P1):**

1. **Disable affected functionality** (if safe)
   ```bash
   # Disable vulnerable workflow
   mv .github/workflows/vulnerable.yml .github/workflows/vulnerable.yml.disabled
   git commit -m "security: Disable vulnerable workflow"
   git push
   ```

2. **Revoke compromised credentials**
   - GitHub tokens
   - Cloud credentials
   - API keys
   - Service account keys

3. **Block malicious actors**
   - IP blocks (if applicable)
   - Account suspensions
   - Rate limiting

4. **Document actions taken**
   ```markdown
   ## Containment Actions
   - [x] Disabled workflow at 2025-10-20 14:30 UTC
   - [x] Revoked GitHub token at 14:31 UTC
   - [x] Notified security team at 14:32 UTC
   ```

**Short-term Containment (P2/P3):**

1. **Risk mitigation** - Add compensating controls
2. **Monitoring enhancement** - Increase logging/alerting
3. **User notification** - If data exposure risk

### 4. Investigation

**Evidence Collection:**

1. **Audit logs**
   ```bash
   # Export GitHub audit log
   gh api /orgs/ORG/audit-log --paginate > audit-log.json
   ```

2. **Git history**
   ```bash
   # Find suspicious commits
   git log --all --oneline --author="suspicious@email.com"
   ```

3. **Workflow runs**
   ```bash
   # Check recent workflow runs
   gh run list --limit 100
   ```

4. **Dependency history**
   ```bash
   # Check when dependency was added
   git log --all -p requirements.txt | grep "package"
   ```

**Root Cause Analysis:**

- [ ] How was the vulnerability introduced?
- [ ] Why didn't existing controls catch it?
- [ ] What was the attack vector?
- [ ] What data was accessed?
- [ ] How long was it exploitable?
- [ ] Are there similar vulnerabilities?

### 5. Remediation

**Fix Development:**

1. **Create private fork** (for P0/P1)
   ```bash
   # Work in private repo to avoid tipping off attackers
   git clone git@github.com:org/bazbom-security-fix.git
   cd bazbom-security-fix
   git checkout -b fix/CVE-YYYY-NNNNN
   ```

2. **Develop fix with tests**
   ```python
   def test_vulnerability_fixed():
       """Ensure CVE-YYYY-NNNNN is fixed"""
       # Test that attack no longer works
       with pytest.raises(ValueError):
           vulnerable_function(malicious_input)
   ```

3. **Security review**
   - Code review by security team
   - Test coverage ≥95% for fix
   - Verify no regression

4. **Backport to supported versions**
   ```bash
   git cherry-pick FIX_COMMIT_SHA
   ```

**Deployment:**

1. **Coordinated release**
   - Release fix to all supported versions
   - Update CHANGELOG.md with [security] tag
   - Create GitHub Security Advisory

2. **User notification**
   - Email security mailing list (if exists)
   - Post to discussion forum
   - Update SECURITY.md

### 6. Recovery

**System Recovery:**

1. **Verify fix effectiveness**
   - Run security scans (Bandit, Semgrep, CodeQL)
   - Manual security testing
   - Verify exploit no longer works

2. **Restore normal operations**
   - Re-enable disabled functionality
   - Remove workarounds
   - Update monitoring

3. **Validation testing**
   - Full test suite passes
   - Integration tests pass
   - Security scans clean

**Communication:**

1. **Internal** - Brief team on lessons learned
2. **Public** - Security advisory (for P0/P1)
3. **Reporters** - Thank and update on fix

### 7. Post-Incident Review

**Within 1 Week:**

1. **Incident report** - Document what happened
2. **Lessons learned** - What could we do better?
3. **Action items** - Preventive measures

**Post-Incident Report Template:**

```markdown
# Security Incident Report: CVE-YYYY-NNNNN

## Summary
- **Date Detected:** YYYY-MM-DD
- **Severity:** P1
- **Status:** Resolved
- **Reporter:** [Name/Anonymous]

## Timeline
- 14:00 UTC - Vulnerability reported
- 14:30 UTC - Triage complete (P1)
- 14:45 UTC - Containment (workflow disabled)
- 16:00 UTC - Fix developed
- 17:00 UTC - Fix deployed
- 18:00 UTC - Verified resolved

## Technical Details
- **Vulnerability:** [Description]
- **Root Cause:** [Analysis]
- **Affected Versions:** v1.0.0 - v1.2.3
- **Attack Vector:** [How it could be exploited]

## Impact Assessment
- **Users Affected:** None (caught before exploitation)
- **Data Exposed:** None
- **Systems Compromised:** None

## Remediation
- **Fix:** [Description of solution]
- **Verification:** [How we confirmed it's fixed]
- **Prevention:** [How we prevent recurrence]

## Lessons Learned
1. [What went well]
2. [What could be improved]
3. [Process gaps identified]

## Action Items
- [ ] Update SAST rules to detect this pattern
- [ ] Add test case for this vulnerability type
- [ ] Document secure pattern in coding guide
- [ ] Add check to pre-commit hooks
```

## Special Scenarios

### Compromised GitHub Token

1. **Revoke immediately** in GitHub settings
2. **Audit token usage** - Check what it accessed
3. **Review recent commits** - Verify no malicious changes
4. **Check workflow runs** - Look for unauthorized runs
5. **Generate new token** with minimal required permissions
6. **Update documentation** - Record incident

### Supply Chain Attack (Dependency)

1. **Identify affected dependency**
   ```bash
   pip-audit -r requirements.txt
   ```

2. **Check for malicious changes**
   ```bash
   # Compare package hash
   pip download package==version --no-deps
   sha256sum package-version.whl
   ```

3. **Remove or downgrade**
   ```bash
   # Downgrade to safe version
   echo "package==SAFE_VERSION" >> requirements.in
   pip-compile --generate-hashes requirements.in
   ```

4. **Scan codebase** - Did we use affected functionality?
5. **Update RISK_LEDGER** - Document mitigation

### Compromised Workflow

1. **Disable workflow immediately**
   ```bash
   gh workflow disable WORKFLOW_NAME
   ```

2. **Review workflow changes** - Git log and diff
3. **Check workflow runs** - Look for suspicious runs
4. **Audit artifacts** - Were malicious artifacts uploaded?
5. **Fix and re-enable** after thorough review

### Secret Leaked to Git History

1. **Revoke secret immediately**
2. **Remove from history** (if public repo)
   ```bash
   # Use git-filter-repo or BFG Repo-Cleaner
   # WARNING: Rewrites history
   git filter-repo --path-glob 'SECRET_FILE' --invert-paths
   git push --force
   ```

3. **Force update all clones**
4. **Scan codebase** - Find all references
5. **Notify affected services**

## Communication Templates

### Initial Acknowledgment

```
Subject: Security Report Received - [REFERENCE-ID]

Thank you for reporting this security issue. We have received your report
and assigned it reference ID [REFERENCE-ID].

Our security team is reviewing the report and will provide an initial 
assessment within 24 hours. We will keep you updated on our progress.

Security Team
```

### Status Update

```
Subject: Security Report Update - [REFERENCE-ID]

Update on security report [REFERENCE-ID]:

Status: [In Progress / Fixed / Verified]
Severity: [P0 / P1 / P2 / P3]
Expected Resolution: [Date]

[Brief description of progress]

We will provide another update by [Date].

Security Team
```

### Resolution Notice

```
Subject: Security Issue Resolved - [REFERENCE-ID]

We have resolved the security issue reported in [REFERENCE-ID].

Fixed Version: v1.2.4
CVE ID: CVE-YYYY-NNNNN (if assigned)
Security Advisory: [URL]

Thank you for your responsible disclosure. We have credited you in:
- CHANGELOG.md
- Security Advisory
- Risk Ledger

Security Team
```

## Security Team Contacts

- **Primary:** See SECURITY.md for contact email
- **Backup:** See MAINTAINERS.md for maintainer contacts
- **Escalation:** Repository owner

## Tools & Resources

### Incident Response Tools

- **GitHub CLI:** `gh` for API access and automation
- **git-filter-repo:** For removing secrets from history
- **BFG Repo-Cleaner:** Alternative secret removal tool
- **pip-audit:** Dependency vulnerability scanning
- **Bandit:** Python SAST for quick checks

### Reference Documentation

- [PYSEC_OMEGA Standards](../docs/copilot/PYSEC.md)
- [Security Review Checklist](SECURITY_REVIEW_CHECKLIST.md)
- [Workflow Security Policy](WORKFLOW_SECURITY_POLICY.md)
- [Risk Ledger](RISK_LEDGER.md)
- [Secure Coding Guide](SECURE_CODING_GUIDE.md)

### External Resources

- [GitHub Security Best Practices](https://docs.github.com/en/code-security)
- [OWASP Incident Response](https://owasp.org/www-community/Incident_Response)
- [NIST Incident Response Guide](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-61r2.pdf)

## Training & Drills

### Quarterly Security Drills

1. **Tabletop exercise** - Walk through incident response
2. **Simulated incident** - Practice containment and remediation
3. **Communication drill** - Practice internal/external communication
4. **Tool validation** - Verify incident response tools work

### Annual Review

- Review and update this document
- Validate contact information
- Test backup communication channels
- Update team training

---

**Document Control:**
- **Version:** 1.0.0
- **Next Review:** 2026-01-20
- **Owner:** Security Team
- **Approval:** [Maintainers]
