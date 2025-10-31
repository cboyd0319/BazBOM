# PCI-DSS v4.0 Compliance Checklist

This checklist helps ensure your project meets PCI-DSS requirements using BazBOM's policy engine.

## Overview

**Standard:** Payment Card Industry Data Security Standard (PCI-DSS) v4.0  
**Applies To:** Applications handling credit card data  
**BazBOM Policy:** `examples/policies/pci-dss.yml`

---

## Implementation Checklist

### 1. Policy Configuration

- [ ] Copy `examples/policies/pci-dss.yml` to your project root as `bazbom.yml`
- [ ] Customize policy for your organization's requirements
- [ ] Review and update license restrictions
- [ ] Configure audit logging for 12-month retention

### 2. Requirement 6.2.4: Public-Facing Web Applications Protected

**Requirement:** Applications must be protected from known attacks.

- [ ] Block all CRITICAL vulnerabilities
- [ ] Block HIGH vulnerabilities if in CISA KEV (Known Exploited Vulnerabilities)
- [ ] Warn on MEDIUM vulnerabilities before release
- [ ] Configure automated scans in CI/CD pipeline

**BazBOM Configuration:**
```yaml
rules:
  - name: block-critical-vulns
    severity_threshold: CRITICAL
    action: BLOCK

  - name: block-high-vulns-if-kev
    conditions:
      - severity: HIGH
        cisa_kev: true
    action: BLOCK
```

**Validation:**
```bash
bazbom scan --path . --policy-check
bazbom policy check
```

### 3. Requirement 6.3.2: Software Development Security Training

**Requirement:** Development teams must be trained on secure coding.

- [ ] Review MEDIUM and LOW vulnerabilities during development
- [ ] Document review process in team procedures
- [ ] Track exceptions with approval and expiration dates

**BazBOM Configuration:**
```yaml
rules:
  - name: warn-medium-vulns
    severity_threshold: MEDIUM
    action: WARN
```

### 4. Requirement 6.5: Security Vulnerabilities Addressed

**Requirement:** Custom code must be reviewed for vulnerabilities.

- [ ] Enable reachability analysis to prioritize exploitable vulnerabilities
- [ ] Use EPSS scores to assess exploit probability
- [ ] Block vulnerabilities with EPSS > 0.5 (>50% exploit probability)

**BazBOM Configuration:**
```yaml
rules:
  - name: block-reachable-critical
    conditions:
      - severity: CRITICAL
        reachable: true
    action: BLOCK

  - name: block-high-epss
    conditions:
      - epss: ">= 0.5"
    action: BLOCK
```

### 5. Requirement 6.6: Change Control Process

**Requirement:** Changes must be documented and tested.

- [ ] Integrate BazBOM scans into pull request workflows
- [ ] Require policy checks to pass before merge
- [ ] Maintain audit trail of all scans
- [ ] Document exceptions with reason and approval

**CI/CD Integration:**
```yaml
# .github/workflows/pci-compliance.yml
- name: PCI-DSS Security Scan
  run: |
    bazbom scan --path . --policy-check
    bazbom policy check
```

### 6. License Compliance

**Requirement:** Software licenses must not conflict with PCI requirements.

- [ ] Deny copyleft licenses (GPL, AGPL) that may complicate compliance
- [ ] Allow permissive licenses (MIT, Apache-2.0, BSD)
- [ ] Maintain SBOM for all dependencies

**BazBOM Configuration:**
```yaml
licenses:
  deny:
    - "GPL-*"
    - "AGPL-*"
  allow:
    - "MIT"
    - "Apache-2.0"
    - "BSD-*"
```

### 7. Audit and Logging (Requirement 10.2.1)

**Requirement:** All access to cardholder data must be logged.

- [ ] Enable audit logging in policy configuration
- [ ] Set retention period to 365 days (12 months)
- [ ] Log all policy violations
- [ ] Review logs monthly

**BazBOM Configuration:**
```yaml
audit:
  enabled: true
  log_file: ".bazbom/audit.jsonl"
  log_violations: true
  retention_days: 365
```

**Review Logs:**
```bash
# View all violations
cat .bazbom/audit.jsonl | jq 'select(.result == "fail")'

# Count violations by month
cat .bazbom/audit.jsonl | jq -r '.timestamp[:7]' | sort | uniq -c
```

---

## Quarterly Review Checklist

Perform these checks every 90 days:

- [ ] Review all active exceptions and renew/revoke as needed
- [ ] Update policy thresholds based on new vulnerabilities
- [ ] Verify audit logs are being retained (365 days minimum)
- [ ] Test policy enforcement in CI/CD pipelines
- [ ] Review license compliance for new dependencies
- [ ] Update policy file with latest PCI-DSS guidance

---

## Common Exceptions

### Temporary Exception for Specific CVE

```yaml
exceptions:
  - cve: CVE-2023-12345
    reason: "WAF blocks exploit vector (PCI requirement 6.6 compensating control)"
    approved_by: "security-team@company.com"
    expires: "2026-01-31"
```

### Development Environment Exception

```yaml
# Use permissive policy for development
# Switch to pci-dss.yml for staging/production

# dev-policy.yml
rules:
  - name: warn-critical-only
    severity_threshold: CRITICAL
    action: WARN
```

---

## Pre-Production Validation

Before deploying to production:

- [ ] Run full security scan: `bazbom scan --path . --reachability`
- [ ] Verify no policy violations: `bazbom policy check`
- [ ] Review SBOM for compliance: Check `sbom.spdx.json`
- [ ] Verify SLSA provenance: Check build attestations
- [ ] Archive audit logs: Save `.bazbom/audit.jsonl`

---

## QSA Audit Preparation

For Qualified Security Assessor (QSA) audits, prepare:

1. **Policy Documentation**
   - [ ] Copy of `bazbom.yml` with all rules documented
   - [ ] Change history for policy file (via git)
   - [ ] Approval records for exceptions

2. **Scan Results**
   - [ ] SBOM files for all production systems
   - [ ] Policy check results (last 12 months)
   - [ ] Audit logs (`.bazbom/audit.jsonl`)

3. **Process Documentation**
   - [ ] CI/CD pipeline configuration with BazBOM integration
   - [ ] Vulnerability remediation procedures
   - [ ] Exception approval workflow
   - [ ] Incident response plan

4. **Evidence of Remediation**
   - [ ] Before/after scan results showing fixed vulnerabilities
   - [ ] Git commits with vulnerability fixes
   - [ ] Testing evidence for patched dependencies

---

## Remediation SLAs

PCI-DSS does not mandate specific SLAs, but best practices:

| Severity | KEV Status | Recommended SLA |
|----------|------------|-----------------|
| CRITICAL | Yes (KEV)  | 7 days          |
| CRITICAL | No         | 30 days         |
| HIGH     | Yes (KEV)  | 14 days         |
| HIGH     | No         | 60 days         |
| MEDIUM   | Any        | 90 days         |
| LOW      | Any        | Best effort     |

Configure automated tracking:

```bash
# Find vulnerabilities by age
bazbom scan --path . --policy-check --show-age

# Export for tracking
bazbom scan --format csv > vulnerabilities.csv
```

---

## Resources

- [PCI-DSS v4.0 Standard](https://www.pcisecuritystandards.org/)
- [PCI Software Security Framework](https://www.pcisecuritystandards.org/document_library)
- [CISA KEV Catalog](https://www.cisa.gov/known-exploited-vulnerabilities-catalog)
- [BazBOM Policy Integration Guide](../../docs/guides/POLICY_INTEGRATION.md)

## Support

For questions about PCI-DSS compliance:
- Contact your QSA (Qualified Security Assessor)
- [PCI Security Standards Council](https://www.pcisecuritystandards.org/contact_us/)
- [BazBOM GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
