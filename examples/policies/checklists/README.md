# Compliance Checklists

This directory contains detailed compliance checklists for each BazBOM policy template.

## Available Checklists

### [PCI-DSS v4.0](PCI-DSS.md)
**Use For:** Applications handling credit card data  
**Key Requirements:**
- Block CRITICAL vulnerabilities
- Block HIGH vulnerabilities in CISA KEV
- 12-month audit retention
- License compliance

**Policy File:** [`../pci-dss.yml`](../pci-dss.yml)

---

### [HIPAA Security Rule](HIPAA.md)
**Use For:** Applications handling Protected Health Information (PHI/ePHI)  
**Key Requirements:**
- Block reachable CRITICAL vulnerabilities
- Malicious package detection
- Encryption library verification
- 7-year audit retention

**Policy File:** [`../hipaa.yml`](../hipaa.yml)

---

### [FedRAMP Moderate](FedRAMP.md)
**Use For:** Cloud services for U.S. federal agencies  
**Key Requirements:**
- CISA KEV enforcement (BOD 22-01)
- 30-day remediation for CRITICAL/HIGH
- SLSA provenance requirement
- Continuous monitoring

**Policy File:** [`../fedramp-moderate.yml`](../fedramp-moderate.yml)

---

### [SOC 2 Type II](SOC2.md) *(Coming Soon)*
**Use For:** B2B SaaS and service providers  
**Key Requirements:**
- Daily vulnerability scans
- High EPSS blocking
- 12-month audit trail
- Continuous monitoring

**Policy File:** [`../soc2.yml`](../soc2.yml)

---

### [Corporate Standard](Corporate.md) *(Coming Soon)*
**Use For:** Development environments and permissive policies  
**Key Requirements:**
- Warn on CRITICAL (no blocking)
- Info-level for HIGH/MEDIUM
- License warnings only

**Policy File:** [`../corporate-permissive.yml`](../corporate-permissive.yml)

---

## How to Use

1. **Choose Your Compliance Framework**
   - Review the requirements for your industry/regulation
   - Select the appropriate checklist

2. **Initialize Policy Template**
   ```bash
   bazbom policy init --template pci-dss
   # or hipaa, fedramp-moderate, soc2, corporate-permissive
   ```

3. **Follow the Checklist**
   - Work through each section systematically
   - Check off items as you complete them
   - Document exceptions and compensating controls

4. **Customize for Your Organization**
   - Adjust severity thresholds as needed
   - Add organization-specific rules
   - Configure audit retention per your requirements

5. **Integrate into CI/CD**
   - Add security scanning to your pipeline
   - Enforce policy checks before deployment
   - Archive audit logs and SBOM files

6. **Maintain Compliance**
   - Review policy quarterly
   - Update exceptions as needed
   - Track remediation progress
   - Prepare for audits

---

## Quick Reference

### Common Commands

```bash
# Initialize policy template
bazbom policy init --template <template-id>

# Validate policy syntax
bazbom policy validate bazbom.yml

# Run security scan with policy check
bazbom scan --path . --policy-check

# Check policy compliance
bazbom policy check

# Generate compliance report
bazbom scan --format sarif > compliance-report.sarif
```

### Audit Log Review

```bash
# View all violations
cat .bazbom/audit.jsonl | jq 'select(.result == "fail")'

# Monthly summary
cat .bazbom/audit.jsonl | jq -r '.timestamp[:7]' | sort | uniq -c

# Export for compliance reporting
cat .bazbom/audit.jsonl | jq -r '[.timestamp, .result, .violation_count] | @csv'
```

---

## Comparison Matrix

| Requirement | PCI-DSS | HIPAA | FedRAMP | SOC 2 |
|-------------|---------|-------|---------|-------|
| **Scan Frequency** | Monthly (min) | Weekly | Weekly | Daily |
| **CRITICAL SLA** | 30 days | 3-14 days | 30 days (15 if KEV) | Immediate |
| **HIGH SLA** | 60 days | 7-30 days | 30 days (15 if KEV) | 7 days |
| **Audit Retention** | 12 months | 7 years | 6 years | 12 months |
| **SBOM Required** | Recommended | Yes | Yes | Recommended |
| **KEV Enforcement** | Recommended | No | Yes (BOD 22-01) | Recommended |
| **License Compliance** | Some restrictions | Info required | Agency-specific | Info required |
| **SLSA Provenance** | No | No | Emerging | No |

---

## Regulatory References

### PCI-DSS
- **Standard:** Payment Card Industry Data Security Standard v4.0
- **Website:** https://www.pcisecuritystandards.org/
- **Key Requirements:** 6.2.4, 6.3.2, 6.5, 6.6, 10.2.1

### HIPAA
- **Standard:** Health Insurance Portability and Accountability Act
- **Regulation:** 45 CFR Parts 160, 162, and 164
- **Website:** https://www.hhs.gov/hipaa/
- **Key Requirements:** §164.308(a)(1)(ii)(A), §164.308(a)(5)(ii)(B), §164.312(a)(2)(iv), §164.312(e)(1), §164.308(a)(8)

### FedRAMP
- **Program:** Federal Risk and Authorization Management Program
- **Framework:** NIST SP 800-53 Rev. 5
- **Website:** https://www.fedramp.gov/
- **Key Controls:** RA-5, SI-2, SA-11, SR-3, SI-5

### SOC 2
- **Standard:** Service Organization Control 2 (AICPA)
- **Framework:** Trust Services Criteria
- **Website:** https://us.aicpa.org/interestareas/frc/assuranceadvisoryservices/aicpasoc2report
- **Key Criteria:** CC7.1, CC7.2, CC8.1

---

## Contributing

Found an error or have suggestions for improving these checklists?

1. Open an issue: https://github.com/cboyd0319/BazBOM/issues
2. Submit a pull request
3. Contact the maintainers

---

## Additional Resources

- **Policy Integration Guide:** [docs/guides/POLICY_INTEGRATION.md](../../../docs/guides/POLICY_INTEGRATION.md)
- **Rego Best Practices:** [docs/guides/REGO_BEST_PRACTICES.md](../../../docs/guides/REGO_BEST_PRACTICES.md)
- **Usage Documentation:** [docs/USAGE.md](../../../docs/USAGE.md)
- **Policy Templates:** [examples/policies/](../)

---

## Support

For compliance-related questions:
- **GitHub Issues:** https://github.com/cboyd0319/BazBOM/issues
- **Documentation:** https://github.com/cboyd0319/BazBOM/tree/main/docs
- **Contributing:** https://github.com/cboyd0319/BazBOM/blob/main/CONTRIBUTING.md

For regulatory/legal compliance advice, consult with:
- Your organization's compliance officer
- Qualified Security Assessor (QSA) for PCI-DSS
- Privacy Officer or Security Officer for HIPAA
- Information System Security Officer (ISSO) for FedRAMP
- SOC 2 auditor for SOC 2

---

**Last Updated:** 2025-10-31  
**Maintained By:** BazBOM Project
