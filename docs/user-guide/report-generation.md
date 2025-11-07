# BazBOM Report Generation Guide

**Version:** 1.0  
**Last Updated:** 2025-11-04  
**Status:** Production Ready  

---

## Overview

BazBOM provides comprehensive report generation capabilities for security teams, compliance officers, executives, and developers. Reports can be generated in HTML format, suitable for viewing in browsers, converting to PDF, or sharing via email.

---

## Report Types

### 1. Executive Summary Report

**Purpose:** High-level security overview for executives and CISOs

**Features:**
- Security score (0-100) with color-coded status
- Vulnerability breakdown by severity
- Top 5 critical risks
- Policy compliance status
- Actionable recommendations

**Command:**
```bash
bazbom report executive --output executive-report.html
```

**Optional Parameters:**
```bash
# Use specific SBOM file
bazbom report executive --sbom path/to/sbom.json --output executive-report.html

# Use specific findings file
bazbom report executive --findings path/to/findings.json --output executive-report.html

# Use both
bazbom report executive \
  --sbom path/to/sbom.json \
  --findings path/to/findings.json \
  --output executive-report.html
```

**Use Cases:**
- Board presentations
- Executive briefings
- Quarterly security reviews
- Stakeholder communications

---

### 2. Compliance Reports

**Purpose:** Framework-specific compliance documentation

**Supported Frameworks:**
1. **PCI-DSS v4.0** - Payment Card Industry Data Security Standard
2. **HIPAA** - Health Insurance Portability and Accountability Act
3. **FedRAMP Moderate** - Federal Risk and Authorization Management Program
4. **SOC 2 Type II** - Service Organization Control 2
5. **GDPR** - General Data Protection Regulation
6. **ISO 27001** - Information Security Management
7. **NIST CSF** - NIST Cybersecurity Framework

**Command:**
```bash
# PCI-DSS
bazbom report compliance pci-dss --output compliance-pci-dss.html

# HIPAA
bazbom report compliance hipaa --output compliance-hipaa.html

# FedRAMP
bazbom report compliance fedramp-moderate --output compliance-fedramp.html

# SOC 2
bazbom report compliance soc2 --output compliance-soc2.html

# GDPR
bazbom report compliance gdpr --output compliance-gdpr.html

# ISO 27001
bazbom report compliance iso27001 --output compliance-iso27001.html

# NIST CSF
bazbom report compliance nist-csf --output compliance-nist-csf.html
```

**Optional Parameters:**
```bash
bazbom report compliance pci-dss \
  --sbom path/to/sbom.json \
  --findings path/to/findings.json \
  --output compliance-pci-dss.html
```

**Features:**
- Framework-specific requirements mapping
- Pass/fail status per requirement
- Evidence documentation
- Remediation recommendations
- Audit-ready formatting

**Use Cases:**
- Compliance audits
- Certification processes
- Vendor assessments
- Regulatory reporting

---

### 3. Developer Report

**Purpose:** Detailed technical report for development teams

**Features:**
- Complete vulnerability list with CVE details
- CVSS scores and severity classifications
- Fixed versions and upgrade paths
- Remediation instructions with code examples
- Reachability analysis results
- CISA KEV (Known Exploited Vulnerabilities) indicators
- EPSS (Exploit Prediction Scoring System) scores

**Command:**
```bash
bazbom report developer --output developer-report.html
```

**Optional Parameters:**
```bash
bazbom report developer \
  --sbom path/to/sbom.json \
  --findings path/to/findings.json \
  --output developer-report.html
```

**Use Cases:**
- Sprint planning
- Vulnerability remediation
- Security testing
- Code reviews

---

### 4. Trend Report

**Purpose:** Historical security metrics and trends

**Features:**
- Security score trend over time
- Vulnerability introduction rate
- Remediation velocity metrics
- Mean Time To Fix (MTTF)
- Dependency growth analysis
- Team performance metrics

**Command:**
```bash
bazbom report trend --output trend-report.html
```

**Optional Parameters:**
```bash
bazbom report trend \
  --sbom path/to/sbom.json \
  --findings path/to/findings.json \
  --output trend-report.html
```

**Use Cases:**
- Quarterly reviews
- Security metrics tracking
- Process improvement
- Team KPI monitoring

---

## Batch Report Generation

### Generate All Reports

Generate all report types in a single command:

```bash
bazbom report all --output-dir reports/
```

This creates:
- `reports/executive-report.html`
- `reports/developer-report.html`
- `reports/trend-report.html`
- `reports/compliance-pci-dss.html`
- `reports/compliance-hipaa.html`
- `reports/compliance-soc2.html`

**With Data Sources:**
```bash
bazbom report all \
  --sbom path/to/sbom.json \
  --findings path/to/findings.json \
  --output-dir reports/
```

---

## Integration Workflows

### 1. CI/CD Pipeline Integration

**GitHub Actions Example:**
```yaml
name: Security Reports

on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly on Monday
  workflow_dispatch:

jobs:
  generate-reports:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install BazBOM
        run: |
          curl -fsSL https://raw.githubusercontent.com/cboyd0319/BazBOM/main/install.sh | bash
      
      - name: Scan Project
        run: bazbom scan .
      
      - name: Generate Reports
        run: bazbom report all --output-dir reports/
      
      - name: Upload Reports
        uses: actions/upload-artifact@v4
        with:
          name: security-reports
          path: reports/
      
      - name: Send Executive Report
        run: |
          # Email or Slack notification logic here
          echo "Reports generated successfully"
```

**GitLab CI Example:**
```yaml
security-reports:
  stage: report
  script:
    - bazbom scan .
    - bazbom report all --output-dir reports/
  artifacts:
    paths:
      - reports/
    expire_in: 30 days
  only:
    - schedules
```

### 2. Scheduled Reporting

**Generate Weekly Executive Reports:**
```bash
#!/bin/bash
# weekly-security-report.sh

# Scan project
bazbom scan /path/to/project

# Generate executive report
bazbom report executive \
  --output /var/reports/executive-$(date +%Y-%m-%d).html

# Email report
mail -s "Weekly Security Report" ciso@company.com < /var/reports/executive-$(date +%Y-%m-%d).html
```

**Cron Job:**
```cron
# Every Monday at 9 AM
0 9 * * 1 /usr/local/bin/weekly-security-report.sh
```

### 3. Pre-Release Compliance Check

```bash
#!/bin/bash
# pre-release-compliance.sh

# Scan release branch
git checkout release

# Generate all compliance reports
bazbom report all --output-dir release-reports/

# Check for critical vulnerabilities
if grep -q "CRITICAL" release-reports/developer-report.html; then
    echo " CRITICAL vulnerabilities found - release blocked"
    exit 1
else
    echo " Compliance checks passed"
    exit 0
fi
```

---

## Report Customization

### Data Sources

Reports can use data from:

1. **Default Cache** (`.bazbom/cache/`)
   ```bash
   bazbom scan .
   bazbom report executive --output report.html
   ```

2. **Custom SBOM File**
   ```bash
   bazbom report executive \
     --sbom path/to/custom-sbom.json \
     --output report.html
   ```

3. **Custom Findings File**
   ```bash
   bazbom report executive \
     --findings path/to/custom-findings.json \
     --output report.html
   ```

4. **Both Custom Files**
   ```bash
   bazbom report executive \
     --sbom path/to/sbom.json \
     --findings path/to/findings.json \
     --output report.html
   ```

### Output Formats

**HTML (Primary)**
- Suitable for browser viewing
- Can be converted to PDF using browser print or external tools
- Includes embedded CSS for styling
- Responsive design for mobile/tablet

**Converting to PDF:**

Using browser:
```bash
# macOS
open report.html
# File > Print > Save as PDF

# Linux with Chrome
google-chrome --headless --print-to-pdf=report.pdf report.html

# Using wkhtmltopdf
wkhtmltopdf report.html report.pdf
```

Using tools:
```bash
# puppeteer (Node.js)
npm install -g puppeteer

# weasyprint (Python)
pip install weasyprint
weasyprint report.html report.pdf
```

---

## Best Practices

### 1. Regular Reporting Schedule

**Recommended Frequencies:**
- **Executive Reports:** Weekly or monthly
- **Compliance Reports:** Quarterly or before audits
- **Developer Reports:** Daily or per commit/PR
- **Trend Reports:** Monthly or quarterly

### 2. Report Storage and Retention

```bash
# Organize by date
reports/
├── 2025-11/
│   ├── 2025-11-01-executive.html
│   ├── 2025-11-01-developer.html
│   └── 2025-11-01-compliance-pci-dss.html
└── 2025-10/
    └── ...
```

### 3. Access Control

- Store reports in secure locations
- Implement role-based access:
  - **Executives:** Executive summaries
  - **Compliance:** Framework-specific reports
  - **Developers:** Developer reports
  - **Security Team:** All reports

### 4. Version Control Integration

```bash
# Store reports in git with LFS
git lfs track "*.html"
git add reports/*.html
git commit -m "Add weekly security reports"
```

### 5. Notification Integration

**Slack Notification:**
```bash
#!/bin/bash
bazbom report executive --output report.html

SCORE=$(grep -oP 'security_score":\K[0-9]+' report.html)

curl -X POST $SLACK_WEBHOOK_URL \
  -H 'Content-Type: application/json' \
  -d "{\"text\":\"Security Report Ready: Score $SCORE/100\"}"
```

**Email Notification:**
```bash
#!/bin/bash
bazbom report executive --output report.html

mail -s "Weekly Security Report" -a report.html security-team@company.com < /dev/null
```

---

## Troubleshooting

### Issue: Empty or Incomplete Reports

**Cause:** No scan data available

**Solution:**
```bash
# Run scan first
bazbom scan .

# Then generate report
bazbom report executive --output report.html
```

### Issue: Outdated Data in Reports

**Cause:** Using cached data

**Solution:**
```bash
# Clear cache
rm -rf .bazbom/cache/

# Rescan
bazbom scan .

# Generate report
bazbom report executive --output report.html
```

### Issue: Report Shows "Unknown Project"

**Cause:** No SBOM data provided

**Solution:**
```bash
# Provide SBOM file
bazbom report executive \
  --sbom sbom.spdx.json \
  --output report.html
```

---

## Advanced Usage

### Custom Report Filtering

```bash
# Generate report for specific findings file
bazbom report developer \
  --findings findings-critical-only.json \
  --output critical-vulnerabilities.html
```

### Multi-Project Reports

```bash
#!/bin/bash
# Generate reports for multiple projects

for project in projects/*; do
    cd "$project"
    bazbom scan .
    bazbom report all --output-dir "../reports/$(basename $project)"
    cd ..
done
```

### Automated Compliance Tracking

```bash
#!/bin/bash
# Track compliance over time

date=$(date +%Y-%m-%d)
bazbom report compliance pci-dss \
  --output "compliance-history/pci-dss-${date}.html"

# Compare with previous
diff <(grep -o "PASS\|FAIL" "compliance-history/pci-dss-${date}.html") \
     <(grep -o "PASS\|FAIL" "compliance-history/pci-dss-previous.html")
```

---

## Examples

### Example 1: Sprint Security Review

```bash
# 1. Scan current sprint branch
git checkout sprint-45
bazbom scan .

# 2. Generate developer report
bazbom report developer --output sprint-45-security.html

# 3. Open in browser
open sprint-45-security.html
```

### Example 2: Pre-Production Compliance

```bash
# 1. Scan production-ready code
git checkout release/v2.0
bazbom scan .

# 2. Generate all compliance reports
bazbom report all --output-dir prod-compliance-v2.0/

# 3. Archive for audit trail
tar czf prod-compliance-v2.0.tar.gz prod-compliance-v2.0/
```

### Example 3: Continuous Monitoring

```bash
# Add to cron (daily at 2 AM)
0 2 * * * cd /opt/myapp && bazbom scan . && bazbom report trend --output /var/reports/trend-$(date +\%Y-\%m-\%d).html
```

---

## Related Documentation

- **[Usage Guide](usage.md)** - Core command reference
- **[Policy Integration](policy-integration.md)** - Policy configuration
- **[Capabilities Reference](../reference/capabilities-reference.md)** - Complete feature catalog

---

## Support

For issues, questions, or feature requests:
- **GitHub Issues:** [github.com/cboyd0319/BazBOM/issues](https://github.com/cboyd0319/BazBOM/issues)
- **Discussions:** [github.com/cboyd0319/BazBOM/discussions](https://github.com/cboyd0319/BazBOM/discussions)
- **Documentation:** [github.com/cboyd0319/BazBOM/tree/main/docs](https://github.com/cboyd0319/BazBOM/tree/main/docs)

---

**Document Version:** 1.0  
**Last Updated:** 2025-11-04  
**Maintained By:** BazBOM Maintainers
