# FedRAMP Moderate Impact Level Compliance Checklist

This checklist helps ensure your project meets FedRAMP Moderate requirements using BazBOM's policy engine.

## Overview

**Standard:** Federal Risk and Authorization Management Program (FedRAMP) Moderate Impact Level  
**Framework:** NIST SP 800-53 Rev. 5  
**Applies To:** Cloud services used by U.S. federal agencies  
**BazBOM Policy:** `examples/policies/fedramp-moderate.yml`

---

## Implementation Checklist

### 1. Policy Configuration

- [ ] Copy `examples/policies/fedramp-moderate.yml` to your project root as `bazbom.yml`
- [ ] Customize for your FedRAMP authorization boundary
- [ ] Enable SLSA provenance verification
- [ ] Configure VEX statement generation

### 2. RA-5: Vulnerability Monitoring and Scanning

**NIST Control:** Organizations must scan for vulnerabilities and remediate legitimate vulnerabilities.

**FedRAMP Enhancement:** Scanning must occur at least monthly, with high-risk vulnerabilities scanned at least weekly.

- [ ] Configure weekly automated scans in CI/CD
- [ ] Block all CISA KEV vulnerabilities (BOD 22-01 requirement)
- [ ] Track vulnerability discovery and remediation dates
- [ ] Generate SARIF reports for vulnerability tracking

**BazBOM Configuration:**
```yaml
rules:
  - name: block-cisa-kev
    conditions:
      - cisa_kev: true
    action: BLOCK
    message: "FedRAMP/RA-5: CISA KEV vulnerabilities must be remediated per BOD 22-01"
```

**Validation:**
```bash
# Weekly scan (automated)
bazbom scan --path . --reachability --policy-check

# Verify CISA KEV enforcement
bazbom policy check
```

### 3. SI-2: Flaw Remediation

**NIST Control:** Identify, report, and correct system flaws.

**FedRAMP Requirements:**
- High vulnerabilities: 30 days
- Moderate vulnerabilities: 90 days
- Low vulnerabilities: 180 days

- [ ] Track vulnerability age from discovery date
- [ ] Block CRITICAL/HIGH vulnerabilities older than 30 days
- [ ] Document remediation efforts
- [ ] Maintain POA&M (Plan of Action and Milestones) for exceptions

**BazBOM Configuration:**
```yaml
rules:
  - name: block-critical-high-30-days
    conditions:
      - severity: ["CRITICAL", "HIGH"]
        age_days: 30  # Older than 30 days
    action: BLOCK
    message: "FedRAMP/SI-2: CRITICAL/HIGH vulnerabilities must be fixed within 30 days"
```

**Track Remediation:**
```bash
# Show vulnerability age
bazbom scan --path . --show-age --format csv > fedramp-remediation.csv

# Filter overdue vulnerabilities
cat fedramp-remediation.csv | awk -F, '$4 > 30 && ($3 == "CRITICAL" || $3 == "HIGH")'
```

### 4. SA-11: Developer Testing and Evaluation

**NIST Control:** Require developers to create security test plans and perform security testing.

- [ ] Integrate BazBOM scans into CI/CD pipeline
- [ ] Require scans to pass before deployment
- [ ] Document test results in authorization package
- [ ] Include SBOM in System Security Plan (SSP)

**CI/CD Integration:**
```yaml
# .github/workflows/fedramp-scan.yml
name: FedRAMP Security Scan

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: FedRAMP Security Scan
        run: |
          bazbom scan --path . --reachability --policy-check
          bazbom policy check
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: policy_violations.sarif
      
      - name: Archive SBOM
        uses: actions/upload-artifact@v4
        with:
          name: fedramp-sbom
          path: sbom.spdx.json
```

### 5. SR-3: Supply Chain Controls

**NIST Control:** Employ supply chain controls to protect against supply chain risks.

**FedRAMP Requirements:**
- Maintain software bill of materials (SBOM)
- Verify software integrity
- Document supply chain risk management

- [ ] Generate SPDX 2.3 SBOM for all releases
- [ ] Enable SLSA provenance attestation
- [ ] Verify package checksums
- [ ] Document trusted package sources

**BazBOM Configuration:**
```yaml
sbom:
  require_slsa_provenance: true  # FedRAMP emerging requirement
  require_vex: true               # Vulnerability Exploitability eXchange statements
  format: "spdx-2.3"
  verify_checksums: true
```

**Generate FedRAMP-Compliant SBOM:**
```bash
# Generate SBOM with provenance
bazbom scan --path . --format spdx --provenance

# Verify SBOM schema compliance
bazbom validate-sbom sbom.spdx.json
```

### 6. SI-5: Security Alerts, Advisories, and Directives

**NIST Control:** Receive security alerts and advisories and implement directives.

**BOD 22-01 Compliance:**
CISA Binding Operational Directive 22-01 requires federal agencies to remediate KEV vulnerabilities.

- [ ] Enable CISA KEV checking
- [ ] Block deployment with any KEV vulnerabilities
- [ ] Monitor CISA KEV catalog updates
- [ ] Document KEV remediation in POA&M

**BazBOM Configuration:**
```yaml
rules:
  - name: block-cisa-kev
    conditions:
      - cisa_kev: true
    action: BLOCK
    message: "BOD 22-01: CISA KEV must be remediated immediately"
```

**Update KEV Database:**
```bash
# Sync with CISA KEV catalog (requires network access)
bazbom db sync --source cisa-kev

# Verify KEV database is current
bazbom db status
```

---

## License Compliance for FedRAMP

### Government Rights Considerations

Some federal agencies restrict copyleft licenses:

- [ ] Review agency-specific license requirements
- [ ] Document all open source licenses in SBOM
- [ ] Verify no GPL-3.0 if restricted by agency
- [ ] Maintain license compliance documentation

**BazBOM Configuration:**
```yaml
licenses:
  deny:
    - "GPL-3.0-only"  # Some federal agencies restrict copyleft
  warn:
    - "GPL-*"
    - "AGPL-*"
  require_license_info: true
```

---

## Continuous Monitoring Requirements

### FedRAMP ConMon (Continuous Monitoring)

**Monthly Requirements:**
- [ ] Vulnerability scan reports
- [ ] POA&M updates
- [ ] Change request documentation
- [ ] Incident reports

**BazBOM Monthly Report:**
```bash
# Generate monthly vulnerability report
bazbom scan --path . --reachability --format sarif > fedramp-monthly-$(date +%Y-%m).sarif

# Generate POA&M data
bazbom scan --format csv --show-age > fedramp-poam-$(date +%Y-%m).csv

# Archive SBOM
cp sbom.spdx.json fedramp-sbom-$(date +%Y-%m).spdx.json
```

---

## POA&M (Plan of Action and Milestones)

### Tracking Unresolved Vulnerabilities

For vulnerabilities that cannot be immediately remediated:

```yaml
exceptions:
  - cve: CVE-2023-12345
    severity: HIGH
    discovery_date: "2025-01-15"
    remediation_deadline: "2025-02-14"  # 30 days for HIGH
    reason: "Waiting for vendor patch. Compensating controls in place."
    compensating_controls:
      - "WAF rule blocks exploit vector"
      - "Network segmentation isolates vulnerable component"
      - "Enhanced monitoring via SIEM"
    risk_assessment: "Residual risk: LOW (exploit requires authenticated access)"
    approved_by: "isso@agency.gov"
    tracking_id: "POAM-2025-001"
```

**Generate POA&M Report:**
```bash
# Export vulnerabilities with age tracking
bazbom scan --format csv --show-age > poam-tracking.csv

# Add to agency POA&M tool
# Import poam-tracking.csv into agency system
```

---

## Annual Assessment and Authorization (A&A)

### Security Assessment Report (SAR)

Prepare for annual assessment:

1. **Vulnerability Scanning Evidence**
   - [ ] 12 months of scan results
   - [ ] Trend analysis showing remediation progress
   - [ ] False positive documentation

2. **Supply Chain Security**
   - [ ] SBOM for all system components
   - [ ] Software provenance attestations
   - [ ] Vendor security documentation

3. **Continuous Monitoring**
   - [ ] Monthly ConMon reports
   - [ ] POA&M updates
   - [ ] Change management records

**Generate A&A Package:**
```bash
# Archive 12 months of scans
for month in {1..12}; do
    bazbom scan --path . --format sarif > aar-scans/fedramp-scan-2025-$(printf "%02d" $month).sarif
done

# Generate annual SBOM archive
bazbom scan --format spdx > aar-sboms/fedramp-sbom-2025-annual.spdx.json
```

---

## Remediation SLAs (FedRAMP Requirements)

| Severity | FedRAMP SLA | BOD 22-01 (KEV) |
|----------|-------------|-----------------|
| CRITICAL | 30 days     | 15 days (if KEV)|
| HIGH     | 30 days     | 15 days (if KEV)|
| MODERATE | 90 days     | N/A             |
| LOW      | 180 days    | N/A             |

**Important:** KEV vulnerabilities supersede standard SLAs per BOD 22-01.

---

## Pre-Authorization Testing (PAT)

Before FedRAMP authorization:

- [ ] Full security scan with reachability analysis
- [ ] Zero HIGH or CRITICAL violations
- [ ] Complete SBOM with SLSA provenance
- [ ] Documented remediation for all exceptions
- [ ] Security test results in SAR format

**PAT Validation:**
```bash
# Pre-authorization security scan
bazbom scan --path . --reachability --policy-check --strict

# Generate SAR-compatible report
bazbom scan --format sarif --output fedramp-pat-results.sarif

# Verify no critical findings
if [ $(jq '.runs[0].results | length' fedramp-pat-results.sarif) -eq 0 ]; then
    echo "PAT PASSED: No security findings"
else
    echo "PAT FAILED: Security findings detected"
    exit 1
fi
```

---

## Integration with Agency Tools

### CSAM (Cyber Security Assessment Management)

Export data for CSAM:

```bash
# Generate CSAM-compatible CSV
bazbom scan --format csv --show-age > csam-export.csv

# Map to CSAM fields:
# - Plugin ID → CVE ID
# - Severity → Severity Level
# - First Discovered → Discovery Date
# - Status → Remediation Status
```

### eMASS (Enterprise Mission Assurance Support Service)

Prepare data for eMASS POA&M:

```bash
# Generate vulnerability data
bazbom scan --format json > emass-vulnerabilities.json

# Process for eMASS import
jq '[.vulnerabilities[] | {
    control: "SI-2",
    vulnerability_id: .id,
    description: .description,
    severity: .severity,
    status: "Open",
    mitigation: "Patch to version \(.fixed_version // "pending")"
}]' emass-vulnerabilities.json > emass-poam.json
```

---

## Audit Preparation

### Documentation Checklist

- [ ] System Security Plan (SSP) with SBOM appendix
- [ ] Security Assessment Report (SAR) with scan results
- [ ] POA&M with all open vulnerabilities
- [ ] Continuous Monitoring Plan including BazBOM scans
- [ ] Incident Response Plan with vulnerability escalation
- [ ] Configuration Management Plan with change control

### Evidence Artifacts

- [ ] 12 months of scan results (SARIF format)
- [ ] Monthly POA&M updates
- [ ] SBOM for each major release
- [ ] Audit logs (`.bazbom/audit.jsonl`)
- [ ] Policy configuration history (via git)
- [ ] Remediation evidence (git commits, test results)

---

## Resources

- [FedRAMP.gov](https://www.fedramp.gov/)
- [NIST SP 800-53 Rev. 5](https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final)
- [CISA BOD 22-01](https://www.cisa.gov/binding-operational-directive-22-01)
- [CISA KEV Catalog](https://www.cisa.gov/known-exploited-vulnerabilities-catalog)
- [SLSA Framework](https://slsa.dev/)
- [BazBOM Policy Integration Guide](../../docs/guides/POLICY_INTEGRATION.md)

## Support

For FedRAMP compliance questions:
- Consult with your agency ISSO (Information System Security Officer)
- [FedRAMP PMO](https://www.fedramp.gov/provide-public-comment/)
- [BazBOM GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
