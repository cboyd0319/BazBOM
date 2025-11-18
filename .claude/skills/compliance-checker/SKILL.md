---
name: compliance-checker
description: Validates compliance against specific frameworks (PCI-DSS, HIPAA, FedRAMP, SOC2, GDPR, ISO27001, NIST), generates compliance reports, and checks policy enforcement. Activates when user asks about compliance status, policy validation, or framework requirements.
---

# Compliance Checker Skill

Validates compliance against security and regulatory frameworks.

## When to Use

Activate this skill when you hear:
- "Check PCI-DSS compliance"
- "Generate HIPAA report"
- "Validate policy"
- "Are we SOC2 compliant?"
- "FedRAMP compliance status"
- "Check GDPR requirements"
- "ISO 27001 audit prep"

## Supported Frameworks

### 1. PCI-DSS 3.2.1 (Payment Card Industry Data Security Standard)
**Requirements relevant to BazBOM:**

| Req | Description | BazBOM Check |
|-----|-------------|--------------|
| 6.1 | Process to identify security vulnerabilities | ✅ Scan executed |
| 6.2 | Ensure all components protected from known vulnerabilities | ✅ 0 critical, <30 day high |
| 6.3 | Develop secure applications | ✅ SAST analysis |
| 6.4 | Follow change control processes | ✅ Git workflow |
| 6.5 | Address common coding vulnerabilities | ✅ Vuln types checked |

### 2. HIPAA Security Rule (Health Insurance Portability and Accountability Act)
**Security standards relevant to BazBOM:**

| Standard | Description | BazBOM Check |
|----------|-------------|--------------|
| 164.308(a)(1)(ii)(A) | Risk Analysis | ✅ Vulnerability scan |
| 164.308(a)(5)(ii)(B) | Protection from malicious software | ✅ Threat intel |
| 164.312(a)(2)(iv) | Encryption | ✅ Crypto library validation |
| 164.312(b) | Audit controls | ✅ SBOM generation |

### 3. FedRAMP Moderate (Federal Risk and Authorization Management Program)
**Controls relevant to BazBOM (NIST 800-53):**

| Control | Description | BazBOM Check |
|---------|-------------|--------------|
| RA-5 | Vulnerability Scanning | ✅ Automated scanning |
| SI-2 | Flaw Remediation | ✅ Patch timeline tracking |
| SA-11 | Developer Security Testing | ✅ SAST integration |
| SA-15 | Development Process | ✅ Secure SDLC |

### 4. SOC 2 Type II (Service Organization Control)
**Trust Services Criteria relevant to BazBOM:**

| Criteria | Description | BazBOM Check |
|----------|-------------|--------------|
| CC6.1 | Logical and physical access | ✅ Auth/authz review |
| CC7.1 | Security monitoring | ✅ Continuous scanning |
| CC7.2 | Vulnerability management | ✅ Remediation tracking |
| CC8.1 | Change control | ✅ SBOM change tracking |

### 5. GDPR Article 32 (General Data Protection Regulation)
**Security requirements:**

| Requirement | Description | BazBOM Check |
|-------------|-------------|--------------|
| 32(1)(b) | Ability to ensure ongoing confidentiality | ✅ Encryption validation |
| 32(1)(d) | Process for testing security measures | ✅ Automated testing |
| 32(2) | Risk assessment | ✅ Vulnerability analysis |

### 6. ISO 27001:2013 (Information Security Management)
**Controls relevant to BazBOM:**

| Control | Description | BazBOM Check |
|---------|-------------|--------------|
| A.12.6.1 | Management of technical vulnerabilities | ✅ Vuln scanning |
| A.14.2.1 | Secure development policy | ✅ Policy enforcement |
| A.14.2.5 | Secure system engineering | ✅ SAST/SCA |

### 7. NIST Cybersecurity Framework
**Functions relevant to BazBOM:**

| Function | Category | BazBOM Check |
|----------|----------|--------------|
| IDENTIFY | Asset Management (ID.AM) | ✅ SBOM generation |
| PROTECT | Information Protection (PR.IP) | ✅ Vuln remediation |
| DETECT | Security Continuous Monitoring (DE.CM) | ✅ Continuous scanning |
| RESPOND | Response Planning (RS.RP) | ✅ Incident tracking |

## Compliance Check Commands

```bash
# Check specific framework
bazbom policy check --framework pci-dss

# Generate compliance report
bazbom report compliance --framework hipaa -o hipaa-compliance.html

# Check all frameworks
bazbom report compliance --framework all -o compliance-report.html

# Validate policy file
bazbom policy validate .bazbom/policy.yaml

# Initialize framework template
bazbom policy init --template pci-dss
```

## Compliance Report Format

```
PCI-DSS 3.2.1 Compliance Report
===============================

Generated: 2024-11-18 14:30 UTC
Scanned: myapp (v2.1.0)
Scan Type: Full (reachability enabled)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

OVERALL STATUS: NON-COMPLIANT ⚠️
Compliance Score: 73/100

Issues: 3 requirements failing
  • Requirement 6.2 (critical vulnerabilities)
  • Requirement 6.5 (coding vulnerabilities)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REQUIREMENT 6.1: Identify Security Vulnerabilities
Status: ✅ COMPLIANT

Evidence:
  • Vulnerability scan executed: 2024-11-18 14:30 UTC
  • Scan frequency: Daily (automated CI/CD)
  • Vulnerability sources: OSV, NVD, GHSA, CISA KEV
  • Coverage: 100% of dependencies scanned

Process Documentation:
  • Scan policy: .bazbom/policy.yaml
  • Scan logs: Available in CI/CD artifacts
  • Responsibility: Security team + DevOps

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REQUIREMENT 6.2: Protect from Known Vulnerabilities
Status: ⚠️ NON-COMPLIANT

PCI-DSS 6.2 Requirements:
  1. All system components must be protected
  2. High-risk vulnerabilities must be addressed within 30 days
  3. Critical vulnerabilities require immediate attention

Compliance Status:
  ❌ Critical vulnerabilities found: 3 (must be 0)
  ⚠️  High vulnerabilities >30 days old: 2
  ✅ Medium/Low vulnerabilities: Acceptable

Details:

  Critical Vulnerabilities (MUST FIX):
    1. CVE-2024-1234 - log4j-core 2.14.1
       Severity: 9.8 (CRITICAL)
       Age: 47 days
       CISA KEV: YES
       Status: OVERDUE (>30 days)
       Fix: Upgrade to 2.20.0 (0.75 hours)

    2. CVE-2024-5678 - openssl 1.1.1f
       Severity: 9.0 (CRITICAL)
       Age: 23 days
       EPSS: 0.87
       Fix: Upgrade to 3.0.2 (1.5 hours)

    3. CVE-2024-9999 - nginx 1.18.0
       Severity: 8.5 (CRITICAL)
       Age: 15 days
       CISA KEV: YES
       Fix: Upgrade to 1.24.0 (0.5 hours)

  High Vulnerabilities >30 Days:
    1. CVE-2024-1111 - Django 3.2.0 (47 days old)
    2. CVE-2024-2222 - requests 2.25.1 (38 days old)

Action Required:
  1. Fix all 3 critical vulnerabilities IMMEDIATELY
  2. Fix 2 high vulnerabilities within 7 days
  3. Document remediation timeline

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REQUIREMENT 6.3: Develop Secure Applications
Status: ✅ COMPLIANT

Evidence:
  • SAST tools integrated: Semgrep, CodeQL
  • Code review process: Mandatory PR reviews
  • Security training: Annual for all developers
  • Secure coding standards: Documented

SAST Scan Results:
  • High severity issues: 0
  • Medium severity issues: 3 (in review)
  • False positives suppressed: 12 (documented)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REQUIREMENT 6.4: Change Control Processes
Status: ✅ COMPLIANT

Evidence:
  • Version control: Git (GitHub)
  • Change approval: PR review required (2 approvers)
  • Testing: CI/CD automated tests
  • Rollback capability: Git history + deployment automation

Recent Changes:
  • Last 30 days: 47 changes
  • Failed builds: 2 (blocked from production)
  • Rollbacks: 1 (documented incident)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REQUIREMENT 6.5: Address Common Coding Vulnerabilities
Status: ⚠️ PARTIALLY COMPLIANT

PCI-DSS 6.5 Common Vulnerabilities:

  6.5.1 - Injection flaws (SQL, OS command, LDAP):
    Status: ⚠️ 1 SQL injection risk found
    Location: api/payment_processor.java:234
    Severity: HIGH
    Action: Fix parameterized query

  6.5.2 - Buffer overflows:
    Status: ✅ COMPLIANT (Rust + Java = memory safe)

  6.5.3 - Insecure cryptographic storage:
    Status: ✅ COMPLIANT
    Evidence: Using industry-standard libraries (OpenSSL 3.x)

  6.5.4 - Insecure communications:
    Status: ✅ COMPLIANT
    Evidence: TLS 1.3 enforced, no plaintext protocols

  6.5.5 - Improper error handling:
    Status: ✅ COMPLIANT
    Evidence: No stack traces in production

  6.5.6 - All high-risk vulnerabilities:
    Status: ❌ NON-COMPLIANT (see Requirement 6.2)

  6.5.7 - Cross-site scripting (XSS):
    Status: ⚠️ 1 XSS risk found
    Location: frontend/user_profile.jsx:89
    Severity: MEDIUM
    Action: Sanitize user input

  6.5.8 - Improper access control:
    Status: ✅ COMPLIANT
    Evidence: RBAC implemented, tested

  6.5.9 - Cross-site request forgery (CSRF):
    Status: ✅ COMPLIANT
    Evidence: CSRF tokens on all forms

  6.5.10 - Broken authentication:
    Status: ✅ COMPLIANT
    Evidence: OAuth 2.0 + MFA

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

REMEDIATION PLAN

Immediate Actions (< 24 hours):
  □ Fix CVE-2024-1234 (log4j-core)
  □ Fix CVE-2024-5678 (openssl)
  □ Fix CVE-2024-9999 (nginx)

Short-term (< 7 days):
  □ Fix SQL injection in payment_processor.java:234
  □ Fix XSS in user_profile.jsx:89
  □ Fix high vulnerabilities >30 days old

Documentation:
  □ Document all remediation activities
  □ Update vulnerability baseline
  □ Schedule follow-up audit

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

CERTIFICATION

This report provides evidence for PCI-DSS Requirement 6 (Secure Systems
and Applications). Compliance requires addressing all NON-COMPLIANT
findings above.

Auditor Guidance:
  • Review this report with technical details
  • Verify vulnerability remediation
  • Confirm change control processes
  • Validate SAST integration

Next Audit: 2024-12-18 (30 days)
```

## Policy Validation

### YAML Policy Example
```yaml
# .bazbom/policy.yaml
version: "1.0"
name: "PCI-DSS Compliance Policy"

rules:
  - id: no-critical-vulns
    description: "PCI-DSS 6.2: No critical vulnerabilities allowed"
    severity: error
    condition: vulnerabilities.filter(v => v.severity == "CRITICAL").length == 0

  - id: high-vuln-age
    description: "PCI-DSS 6.2: High vulnerabilities must be <30 days old"
    severity: error
    condition: vulnerabilities.filter(v => v.severity == "HIGH" && v.age_days > 30).length == 0

  - id: vulnerability-scan-required
    description: "PCI-DSS 6.1: Vulnerability scanning required"
    severity: error
    condition: scan_executed == true

  - id: sast-required
    description: "PCI-DSS 6.3: SAST analysis required"
    severity: warning
    condition: sast_results != null
```

### Validation Commands
```bash
# Validate policy syntax
bazbom policy validate .bazbom/policy.yaml

# Check compliance
bazbom policy check --verbose

# Dry-run (show what would fail without failing build)
bazbom policy check --dry-run

# Generate policy violation report
bazbom policy check --report policy-violations.html
```

## Quick Compliance Checks

```bash
# Check critical vulnerabilities (PCI-DSS 6.2)
bazbom scan . -o /tmp/results
CRITICAL_COUNT=$(jq '.vulnerabilities[] | select(.severity == "CRITICAL") | .id' /tmp/results/sca_findings.json | wc -l)
if [ $CRITICAL_COUNT -gt 0 ]; then
    echo "PCI-DSS 6.2 FAIL: $CRITICAL_COUNT critical vulnerabilities found"
else
    echo "PCI-DSS 6.2 PASS: 0 critical vulnerabilities"
fi

# Check high vulnerability age (PCI-DSS 6.2)
jq '.vulnerabilities[] | select(.severity == "HIGH" and .age_days > 30)' /tmp/results/sca_findings.json

# Check SBOM generation (NIST CSF ID.AM)
test -f sbom.spdx.json && echo "NIST CSF PASS: SBOM exists" || echo "NIST CSF FAIL: No SBOM"

# Check encryption libraries (HIPAA 164.312)
jq '.packages[] | select(.name | contains("openssl") or contains("crypto"))' /tmp/results/sbom.spdx.json
```

## Success Criteria

Compliance checking is effective when:
- ✅ All framework requirements mapped to BazBOM checks
- ✅ Clear pass/fail criteria
- ✅ Actionable remediation steps provided
- ✅ Evidence documentation included
- ✅ Audit-ready reports generated
- ✅ Policy violations caught before production

Remember: **Compliance is about documentation and evidence** - having the controls is not enough, you must demonstrate them.
