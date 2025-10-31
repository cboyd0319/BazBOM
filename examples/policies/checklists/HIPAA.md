# HIPAA Security Rule Compliance Checklist

This checklist helps ensure your project meets HIPAA requirements using BazBOM's policy engine.

## Overview

**Standard:** Health Insurance Portability and Accountability Act (HIPAA) Security Rule  
**Applies To:** Applications handling Protected Health Information (PHI) or Electronic PHI (ePHI)  
**BazBOM Policy:** `examples/policies/hipaa.yml`

---

## Implementation Checklist

### 1. Policy Configuration

- [ ] Copy `examples/policies/hipaa.yml` to your project root as `bazbom.yml`
- [ ] Customize policy for your organization's HIPAA requirements
- [ ] Configure audit logging for security events
- [ ] Ensure all dependencies have license information

### 2. §164.308(a)(1)(ii)(A): Security Risk Analysis

**Requirement:** Conduct regular security risk analysis to identify vulnerabilities.

- [ ] Block all reachable CRITICAL vulnerabilities
- [ ] Enable reachability analysis for accurate risk assessment
- [ ] Conduct scans at least weekly (daily recommended)
- [ ] Document risk assessment findings

**BazBOM Configuration:**
```yaml
rules:
  - name: block-all-reachable-criticals
    conditions:
      - severity: CRITICAL
        reachable: true
    action: BLOCK
    message: "HIPAA §164.308: Reachable CRITICAL vulnerabilities pose ePHI risk"
```

**Validation:**
```bash
# Full scan with reachability analysis
bazbom scan --path . --reachability --policy-check

# Generate risk assessment report
bazbom scan --format sarif > hipaa-risk-assessment.sarif
```

### 3. §164.308(a)(5)(ii)(B): Protection from Malicious Software

**Requirement:** Implement procedures to guard against malicious software.

- [ ] Block packages detected as malicious
- [ ] Verify package integrity with checksums
- [ ] Use trusted package repositories only
- [ ] Enable SLSA provenance verification

**BazBOM Configuration:**
```yaml
rules:
  - name: malicious-package-detection
    detect_malicious: true
    action: BLOCK
    message: "HIPAA §164.308(a)(5): Malicious packages prohibited"

sbom:
  require_slsa_provenance: true
  verify_checksums: true
```

### 4. §164.312(a)(2)(iv): Encryption and Decryption

**Requirement:** Implement mechanism to encrypt and decrypt ePHI.

- [ ] Verify cryptography libraries are present
- [ ] Use approved encryption libraries (e.g., BouncyCastle for JVM)
- [ ] Ensure encryption libraries are up-to-date
- [ ] Document encryption implementation

**BazBOM Configuration:**
```yaml
rules:
  - name: require-encryption-libs
    required_packages:
      - "org.bouncycastle:*"  # Cryptography libraries must be present
    action: WARN
    message: "HIPAA §164.312: Encryption libraries required for ePHI protection"
```

**Verify Encryption Libraries:**
```bash
# Check for cryptography dependencies
bazbom scan --path . | jq '.packages[] | select(.name | contains("bouncycastle"))'
```

### 5. §164.312(e)(1): Transmission Security

**Requirement:** Implement technical security measures to guard against unauthorized access to ePHI during transmission.

- [ ] Verify TLS/SSL libraries are present and current
- [ ] Ensure no vulnerabilities in transmission libraries
- [ ] Document secure transmission mechanisms

**Required Libraries:**
- TLS/SSL libraries (OpenSSL, BoringSSL, or Java SSL)
- Secure HTTP clients (Apache HttpClient, OkHttp)

### 6. §164.308(a)(8): Evaluation

**Requirement:** Perform periodic technical and non-technical evaluation to ensure security measures are implemented.

- [ ] Schedule regular security scans (weekly minimum)
- [ ] Enable audit logging for all scans
- [ ] Review audit logs monthly
- [ ] Document evaluation findings

**BazBOM Configuration:**
```yaml
audit:
  enabled: true
  log_all_scans: true
  log_violations: true
  retention_days: 2555  # 7 years for HIPAA compliance
```

**Audit Review:**
```bash
# Review monthly scan history
cat .bazbom/audit.jsonl | jq 'select(.timestamp | startswith("2025-10"))'

# Generate compliance report
cat .bazbom/audit.jsonl | jq -r '[.timestamp, .result, .violation_count] | @csv' > hipaa-audit-report.csv
```

---

## License Compliance for HIPAA

### Required: Complete License Information

HIPAA requires maintaining accurate software inventory (§164.308(a)(8)).

- [ ] Ensure all dependencies have valid license information
- [ ] Block packages with "Unknown" or "NOASSERTION" licenses
- [ ] Maintain SBOM with complete license data

**BazBOM Configuration:**
```yaml
licenses:
  require_license_info: true
  deny:
    - "Unknown"
    - "NOASSERTION"
```

**Verify License Completeness:**
```bash
# Check for missing licenses
bazbom scan --path . | jq '.packages[] | select(.license == "Unknown" or .license == "NOASSERTION")'
```

---

## Business Associate Agreements (BAA)

If using third-party services or dependencies:

- [ ] Verify vendor has signed BAA if handling PHI
- [ ] Document all third-party dependencies
- [ ] Review licenses for data handling restrictions
- [ ] Ensure cloud services are HIPAA-compliant

---

## Breach Notification Requirements (§164.404)

In case of security incident:

1. **Immediate Actions:**
   - [ ] Document the vulnerability discovery date
   - [ ] Archive current SBOM and scan results
   - [ ] Identify affected systems and ePHI

2. **Assessment:**
   - [ ] Determine if ePHI was accessed/disclosed
   - [ ] Evaluate risk to ePHI
   - [ ] Document assessment findings

3. **Notification Timeline:**
   - [ ] Notify affected individuals within 60 days (if breach affects 500+ individuals)
   - [ ] Notify HHS within 60 days
   - [ ] Notify media if affecting 500+ individuals in a state

---

## Annual Review Checklist

Perform annually to maintain compliance:

- [ ] Review and update security policy
- [ ] Verify all critical vulnerabilities are addressed
- [ ] Test incident response procedures
- [ ] Update risk assessment documentation
- [ ] Review audit logs for anomalies
- [ ] Verify encryption is functioning correctly
- [ ] Update Business Associate Agreements

---

## Pre-Production Validation

Before deploying systems handling ePHI:

- [ ] Full security scan: `bazbom scan --path . --reachability --policy-check`
- [ ] Verify no policy violations
- [ ] Review SBOM for unapproved packages
- [ ] Confirm encryption libraries present and current
- [ ] Archive baseline scan for future reference
- [ ] Document deployment in change log

---

## Audit Preparation

For HIPAA audits, prepare:

### 1. Security Documentation

- [ ] Current `bazbom.yml` policy file
- [ ] Change history for security policies
- [ ] Risk assessment reports (SARIF files)
- [ ] SBOM files for all production systems

### 2. Audit Logs

- [ ] Complete audit trail (`.bazbom/audit.jsonl`)
- [ ] Scan history for past 7 years
- [ ] Policy violation records
- [ ] Remediation evidence

### 3. Process Documentation

- [ ] CI/CD pipeline with security scanning
- [ ] Vulnerability remediation procedures
- [ ] Incident response plan
- [ ] Encryption key management procedures

### 4. Evidence of Compliance

- [ ] Regular scan results showing no violations
- [ ] Documentation of fixed vulnerabilities
- [ ] Training records for development team
- [ ] Business Associate Agreements with vendors

---

## Remediation SLAs

Recommended timeframes for HIPAA compliance:

| Severity | ePHI Risk | Recommended SLA |
|----------|-----------|-----------------|
| CRITICAL | High      | 3 days          |
| CRITICAL | Medium    | 7 days          |
| CRITICAL | Low       | 14 days         |
| HIGH     | High      | 7 days          |
| HIGH     | Medium    | 30 days         |
| MEDIUM   | Any       | 60 days         |
| LOW      | Any       | 90 days         |

**Track Remediation:**
```bash
# Generate remediation tracking report
bazbom scan --format csv --show-age > hipaa-remediation-tracking.csv
```

---

## Integration with Risk Analysis

### Required Risk Factors

Document these for each vulnerability:

1. **Likelihood of Threat Occurrence**
   - EPSS Score (exploit probability)
   - CISA KEV status
   - Attack vector (network, local, physical)

2. **Magnitude of Impact**
   - Severity level (CRITICAL, HIGH, MEDIUM, LOW)
   - Reachability status
   - Potential ePHI exposure

3. **Adequacy of Safeguards**
   - WAF rules
   - Network segmentation
   - Access controls

**Generate Risk Report:**
```bash
bazbom scan --path . --reachability --policy-check --format sarif > hipaa-risk-report.sarif
```

---

## Common Exceptions

### Temporary Exception with Compensating Control

```yaml
exceptions:
  - cve: CVE-2023-12345
    reason: "WAF rule blocks exploit. Network segmentation isolates vulnerable component."
    approved_by: "hipaa-security-officer@example.com"
    approved_date: "2025-01-15"
    expires: "2025-06-30"
    compensating_controls:
      - "WAF rule #142 blocks all CVE-2023-12345 exploit attempts"
      - "Vulnerable component isolated in DMZ with no ePHI access"
      - "Additional monitoring via SIEM alert rule #89"
```

---

## Continuous Monitoring

### Daily Monitoring Tasks

```bash
# Automated daily scan
0 2 * * * cd /path/to/project && bazbom scan --path . --policy-check

# Alert on new violations
if [ $(bazbom policy check | grep -c "violations detected") -gt 0 ]; then
    # Send alert to security team
    mail -s "HIPAA Policy Violations" security@example.com < policy_violations.json
fi
```

### Weekly Review Tasks

- [ ] Review new vulnerabilities
- [ ] Assess ePHI exposure risk
- [ ] Plan remediation for violations
- [ ] Update exception requests

---

## Resources

- [HIPAA Security Rule](https://www.hhs.gov/hipaa/for-professionals/security/index.html)
- [HHS Breach Notification Rule](https://www.hhs.gov/hipaa/for-professionals/breach-notification/index.html)
- [NIST HIPAA Security Rule Toolkit](https://csrc.nist.gov/projects/hipaa-security-rule-toolkit)
- [BazBOM Policy Integration Guide](../../docs/guides/POLICY_INTEGRATION.md)

## Support

For HIPAA compliance questions:
- Consult with your Privacy Officer or Security Officer
- [HHS HIPAA Resources](https://www.hhs.gov/hipaa/index.html)
- [BazBOM GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
