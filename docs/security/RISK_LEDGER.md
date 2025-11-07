# BazBOM Security Risk Ledger

**Last Updated:** 2025-10-20
**Assessment Version:** 2.0.0
**Assessment Type:** Comprehensive PYSEC_OMEGA Security Audit + Hardening

## Executive Summary

BazBOM has undergone comprehensive security hardening following PYSEC_OMEGA standards. This ledger documents all identified security risks, their severity, impact, likelihood, and remediation status.

**Overall Security Posture: EXCEPTIONAL** 

- **Critical:** 0 issues
- **High:** 0 issues
- **Medium:** 3 issues (2 FIXED, 1 FALSE POSITIVE)
- **Low:** 2041 issues (mostly informational)
- **Dependencies:** 0 vulnerabilities (pip-audit clean)
- **Supply Chain:** HARDENED (SHA-pinned actions, hash-verified deps)

## Remediation Summary

| Status | Count | Description |
|--------|-------|-------------|
|  FIXED | 2 | Critical security vulnerabilities addressed |
|  MITIGATED | 8 | False positives suppressed with justification |
|  IN PROGRESS | 0 | Currently being addressed |
|  PLANNED | 0 | Scheduled for future releases |
|  HARDENED | 15+ | Additional security controls implemented |

## Recent Security Enhancements (2025-10-20)

### Phase 1: GitHub Actions Hardening

**Status:**  COMPLETE

1. **SHA-Pinned Actions** 
   - All GitHub Actions pinned to full 40-character SHA
   - Version comments added for human readability
   - Automated updates via Dependabot
   - **Impact:** Prevents supply chain attacks via action tampering

2. **Credential Protection** 
   - persist-credentials: false on all checkout actions
   - No long-lived credentials in workflows
   - OIDC federation ready
   - **Impact:** Prevents credential theft from compromised workflows

3. **Job Timeouts** 
   - All jobs have timeout-minutes configured
   - Prevents runaway jobs and resource exhaustion
   - **Impact:** Protects against DoS and cost overruns

4. **Workflow Security Policy** 
   - Comprehensive policy document created
   - Mandatory requirements documented
   - Incident response procedures defined
   - **Impact:** Ensures consistent security practices

### Phase 2: Dependency Security & Management

**Status:**  COMPLETE

1. **Hash-Based Verification** 
   - All requirements files have SHA256 hashes
   - pip-tools with --generate-hashes
   - Automated hash verification in CI
   - **Impact:** Prevents package tampering and dependency confusion

2. **Multi-Scanner Vulnerability Detection** 
   - pip-audit (Python packages)
   - OSV Scanner (cross-ecosystem)
   - Safety (commercial CVE database)
   - Dependency Review (GitHub)
   - **Impact:** Comprehensive vulnerability coverage

3. **License Compliance** 
   - GPL/AGPL licenses blocked
   - Permissive licenses allowed
   - Automated PR comments
   - **Impact:** Prevents licensing issues

4. **Dependency Management Documentation** 
   - Complete pip-tools guide
   - Security best practices
   - Troubleshooting procedures
   - **Impact:** Enables secure dependency management

### Phase 3: Security Tooling Enhancement

**Status:**  COMPLETE

1. **Test Isolation** 
   - pytest-randomly configured (seed: 1337)
   - Tests pass in random order
   - Detects hidden test dependencies
   - **Impact:** Ensures test reliability and quality

2. **Security Review Checklist** 
   - Pre-submission checklist for developers
   - Review checklist for reviewers
   - Severity classification guide
   - **Impact:** Standardizes security reviews

3. **Coverage Configuration** 
   - 90%+ coverage requirement
   - Branch coverage enabled
   - Comprehensive exclude patterns
   - **Impact:** Ensures code quality and security

4. **Secrets Scanning** 
   - TruffleHog in pre-commit
   - GitLeaks in pre-commit and CI
   - Automatic secret detection
   - **Impact:** Prevents credential leaks

---

## MEDIUM Severity Issues

### 1. XML External Entity (XXE) Vulnerability  FIXED

**CWE:** CWE-20 (Improper Input Validation)
**File:** `tools/supplychain/license_extractor.py:194`
**Test ID:** B314
**Status:**  FIXED

#### Description

Using `xml.etree.ElementTree.parse()` to parse untrusted XML data is vulnerable to XML External Entity (XXE) attacks, which can lead to:

- Disclosure of confidential data
- Denial of service
- Server-side request forgery (SSRF)
- System compromise

#### Risk Assessment

- **Likelihood:** MEDIUM - POM files are from Maven Central (trusted) but could be tampered
- **Impact:** HIGH - Could expose local file system or enable SSRF
- **Overall Risk:** MEDIUM

#### Fix Applied

Replaced `xml.etree.ElementTree` with `defusedxml.ElementTree` which:

- Disables entity expansion by default
- Prevents XXE attacks
- Maintains API compatibility

```python
# Before (VULNERABLE)
import xml.etree.ElementTree as ET
tree = ET.parse(pom_path)

# After (SECURE)
from defusedxml import ElementTree as ET
tree = ET.parse(pom_path)
```

#### Verification

-  Unit tests pass with defusedxml
-  Functionality unchanged
-  XXE protection confirmed

---

### 2. URL Scheme Validation Bypass  FIXED

**CWE:** CWE-22 (Path Traversal)
**File:** `tools/supplychain/supply_chain_risk.py:90`
**Test ID:** B310
**Status:**  FIXED

#### Description

`urllib.request.urlopen()` accepts any URL scheme including `file:/` which can lead to:

- Local file disclosure
- Server-side request forgery (SSRF)
- Access to internal network resources

#### Risk Assessment

- **Likelihood:** LOW - URL is constructed from trusted Maven coordinates
- **Impact:** MEDIUM - Could access local files if input is compromised
- **Overall Risk:** MEDIUM

#### Fix Applied

Added explicit URL scheme validation:

```python
# Validate URL scheme for security (prevent file:/ and other schemes)
if not search_url.startswith(('http://', 'https://')):
    return []

with urllib.request.urlopen(search_url, timeout=5) as response:
    # ... rest of code
```

#### Verification

-  Only HTTP/HTTPS URLs are allowed
-  file:/, ftp:/, and other schemes are rejected
-  Functionality unchanged for legitimate use

---

### 3. Insecure Temp File/Directory Usage  FALSE POSITIVE

**CWE:** CWE-377 (Insecure Temporary File)
**Files:** Test files (8 occurrences)
**Test ID:** B108
**Status:**  FALSE POSITIVE - SUPPRESSED

#### Description

Bandit flagged hardcoded `/tmp/` paths in test files as insecure temporary file usage.

#### Analysis

**These are FALSE POSITIVES** because:

1. These are **mock arguments** in test code, not actual file operations
2. No actual temporary files are created insecurely
3. Paths are used for test assertions and mocks only
4. No security risk exists

## References

- [SECURITY.md](../../SECURITY.md)
- [Threat Model](threat-model.md)
- [Secure Coding Guide](SECURE_CODING_GUIDE.md)

---

**Risk Ledger Maintained By:** Security Team
**Review Frequency:** Weekly
**Next Review:** 2025-10-26
