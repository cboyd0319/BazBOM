# Policy Integration Guide

This guide explains how to integrate BazBOM's policy engine into your development workflow, CI/CD pipelines, and compliance processes.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Policy File Structure](#policy-file-structure)
3. [CI/CD Integration](#cicd-integration)
4. [Policy Inheritance](#policy-inheritance)
5. [Audit Trail Setup](#audit-trail-setup)
6. [Compliance Workflows](#compliance-workflows)
7. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Step 1: Initialize a Policy Template

Choose a regulatory template that matches your compliance requirements:

```bash
# List available templates
bazbom policy init --list

# Initialize PCI-DSS template
bazbom policy init --template pci-dss

# Or start with a permissive policy for development
bazbom policy init --template corporate-permissive
```

This creates a `bazbom.yml` file in your project root.

### Step 2: Customize the Policy

Edit `bazbom.yml` to match your organization's requirements:

```yaml
name: "MyOrg Policy"
version: "1.0"

rules:
  - name: block-critical-vulns
    severity_threshold: CRITICAL
    action: BLOCK
    message: "CRITICAL vulnerabilities must be fixed before deployment"

licenses:
  deny:
    - "GPL-*"
    - "AGPL-*"
  allow:
    - "MIT"
    - "Apache-2.0"
    - "BSD-*"
```

### Step 3: Run Policy Check

```bash
# Scan and check policy in one command
bazbom scan --path . --policy-check

# Or run policy check separately
bazbom policy check
```

---

## Policy File Structure

### Basic YAML Policy

```yaml
name: "Project Policy"
description: "Security and license policy for MyProject"
version: "1.0"

# Vulnerability rules
rules:
  - name: block-critical
    severity_threshold: CRITICAL
    action: BLOCK
    message: "CRITICAL vulnerabilities block deployment"

  - name: warn-high-reachable
    conditions:
      - severity: HIGH
        reachable: true
    action: WARN
    message: "HIGH severity + reachable should be reviewed"

  - name: block-kev
    conditions:
      - cisa_kev: true
    action: BLOCK
    message: "CISA KEV vulnerabilities must be fixed immediately"

  - name: block-high-epss
    conditions:
      - epss: ">= 0.5"  # >50% exploit probability
    action: BLOCK
    message: "High exploitability vulnerabilities must be addressed"

# License rules
licenses:
  # Allowed licenses (allowlist mode)
  allow:
    - "MIT"
    - "Apache-2.0"
    - "BSD-2-Clause"
    - "BSD-3-Clause"
    - "ISC"

  # Denied licenses (denylist mode)
  deny:
    - "GPL-*"       # Wildcard matches GPL-2.0, GPL-3.0, etc.
    - "AGPL-*"      # Network copyleft
    - "Unknown"     # Unknown licenses are risky
    - "NOASSERTION" # No license info

  # Project license for compatibility checking
  project_license: "MIT"

# Exceptions (time-limited)
exceptions:
  - cve: CVE-2023-12345
    reason: "Mitigated by WAF rule #42"
    approved_by: "security@example.com"
    expires: "2025-12-31"

# Audit configuration
audit:
  enabled: true
  log_file: ".bazbom/audit.jsonl"
  log_all_scans: false
  log_violations: true
  retention_days: 365
```

### Advanced Rego Policy

For complex rules, use Rego (Open Policy Agent language):

```rego
# policy.rego
package bazbom

# Block CRITICAL vulnerabilities
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("CRITICAL vulnerability %s in %s", [vuln.id, vuln.package])
}

# Block CISA KEV regardless of severity
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.cisa_kev == true
    msg := sprintf("CISA KEV %s must be fixed per BOD 22-01", [vuln.id])
}

# Warn on copyleft licenses in commercial projects
warn[msg] {
    dep := input.dependencies[_]
    copyleft := ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]
    dep.license in copyleft
    input.metadata.commercial == true
    msg := sprintf("Copyleft license %s in %s", [dep.license, dep.name])
}

# Allow exceptions for approved CVEs
allow[msg] {
    vuln := input.vulnerabilities[_]
    exception := data.exceptions[_]
    vuln.id == exception.cve
    time.now_ns() < time.parse_rfc3339_ns(exception.expires)
    msg := sprintf("Exception approved for %s until %s", [vuln.id, exception.expires])
}
```

To use Rego policies:

```bash
# Enable Rego feature flag when building
cargo build --features rego

# Use Rego policy
bazbom scan --policy policy.rego
```

---

## CI/CD Integration

### GitHub Actions

```yaml
# .github/workflows/security-scan.yml
name: Security Scan

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  bazbom-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run BazBOM Security Scan
        uses: cboyd0319/bazbom-action@v1
        with:
          policy-file: bazbom.yml
          fail-on-violation: true
          upload-sarif: true

      - name: Upload SARIF results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: policy_violations.sarif
```

### GitLab CI

```yaml
# .gitlab-ci.yml
security_scan:
  stage: test
  image: bazbom/bazbom:latest
  script:
    - bazbom scan --path . --policy-check
    - bazbom policy check
  artifacts:
    reports:
      sast: policy_violations.sarif
    paths:
      - sbom.spdx.json
      - policy_violations.json
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

### Jenkins

```groovy
// Jenkinsfile
pipeline {
    agent any

    stages {
        stage('Security Scan') {
            steps {
                sh 'bazbom scan --path . --out-dir ./reports'
                sh 'bazbom policy check'
            }
        }

        stage('Policy Enforcement') {
            steps {
                script {
                    def policyResult = readJSON file: 'policy_result.json'
                    if (!policyResult.passed) {
                        error("Policy violations detected: ${policyResult.violations.size()}")
                    }
                }
            }
        }
    }

    post {
        always {
            archiveArtifacts artifacts: 'reports/**', fingerprint: true
            publishHTML([
                reportDir: 'reports',
                reportFiles: 'sbom.spdx.json',
                reportName: 'SBOM Report'
            ])
        }
    }
}
```

### CircleCI

```yaml
# .circleci/config.yml
version: 2.1

jobs:
  security-scan:
    docker:
      - image: bazbom/bazbom:latest
    steps:
      - checkout
      - run:
          name: BazBOM Scan
          command: |
            bazbom scan --path . --policy-check
            bazbom policy check
      - store_artifacts:
          path: sbom.spdx.json
      - store_artifacts:
          path: policy_violations.sarif

workflows:
  version: 2
  build-and-scan:
    jobs:
      - security-scan
```

---

## Policy Inheritance

### Use Case: Organization → Team → Project

Create a 3-level policy hierarchy:

```
.bazbom/
├── policies/
│   ├── organization.yml      # Baseline (strictest)
│   ├── team-backend.yml       # Backend team overrides
│   └── project-api.yml        # Project-specific exceptions
└── config.yml                  # Policy inheritance config
```

### Organization Policy (Baseline)

```yaml
# .bazbom/policies/organization.yml
name: "Acme Corp Security Baseline"
version: "1.0"

rules:
  - name: block-all-critical
    severity_threshold: CRITICAL
    action: BLOCK

  - name: block-cisa-kev
    conditions:
      - cisa_kev: true
    action: BLOCK

licenses:
  deny:
    - "AGPL-*"      # Network copyleft prohibited company-wide
    - "Unknown"
    - "NOASSERTION"

audit:
  enabled: true
  log_all_scans: true
  retention_days: 730  # 2 years for compliance
```

### Team Policy (Override)

```yaml
# .bazbom/policies/team-backend.yml
name: "Backend Team Policy"
version: "1.0"

# Inherit organization policy, add team-specific rules
rules:
  - name: warn-high-reachable
    conditions:
      - severity: HIGH
        reachable: true
    action: WARN  # Backend can manage HIGH vulnerabilities

licenses:
  allow:
    - "MIT"
    - "Apache-2.0"
    - "BSD-*"
```

### Project Policy (Final Overrides)

```yaml
# bazbom.yml
name: "API Project Policy"
version: "1.0"

exceptions:
  # Temporary exception for specific CVE
  - cve: CVE-2024-1234
    reason: "Library only used in test code, not deployed"
    approved_by: "john@example.com"
    expires: "2025-06-30"

licenses:
  # Project uses LGPL library with legal approval
  allow:
    - "LGPL-2.1-only"
```

### Configuration File

```yaml
# .bazbom/config.yml
policy_inheritance:
  - .bazbom/policies/organization.yml  # Loaded first (baseline)
  - .bazbom/policies/team-backend.yml  # Overrides organization
  - bazbom.yml                          # Project-specific (final overrides)

merge_strategy: "strict"  # Options: strict, permissive, override

# strict:     Use most restrictive rule
# permissive: Use least restrictive rule
# override:   Last policy wins
```

---

## Audit Trail Setup

### Enable Audit Logging

Add to your policy file:

```yaml
# bazbom.yml
audit:
  enabled: true
  log_file: ".bazbom/audit.jsonl"
  log_all_scans: false        # Only log violations
  log_violations: true
  log_warnings: true
  max_size_bytes: 104857600   # 100 MB
  retention_days: 365         # 1 year
```

### Audit Log Format (JSONL)

Each line is a JSON object:

```json
{
  "timestamp": "2025-10-31T22:00:00Z",
  "timestamp_unix": 1730412000,
  "action": "policy_check",
  "result": "fail",
  "violation_count": 3,
  "warning_count": 0,
  "policy_source": "bazbom.yml",
  "context": {
    "project": "my-api",
    "user": "ci-bot",
    "ci_job_id": "12345",
    "commit_sha": "abc123def456"
  }
}
```

### Query Audit Logs

```bash
# View all logs
cat .bazbom/audit.jsonl | jq .

# Filter by date (Unix timestamp)
cat .bazbom/audit.jsonl | jq 'select(.timestamp_unix > 1730000000)'

# Filter by result
cat .bazbom/audit.jsonl | jq 'select(.result == "fail")'

# Count violations by day
cat .bazbom/audit.jsonl | jq -r '.timestamp[:10]' | sort | uniq -c
```

### Audit Trail in CI

```yaml
# GitHub Actions
- name: Run policy check with audit context
  run: |
    export BAZBOM_AUDIT_CONTEXT='{
      "project": "${{ github.repository }}",
      "user": "${{ github.actor }}",
      "ci_job_id": "${{ github.run_id }}",
      "commit_sha": "${{ github.sha }}"
    }'
    bazbom policy check
```

---

## Compliance Workflows

### PCI-DSS Compliance

```yaml
# bazbom.yml
name: "PCI-DSS v4.0 Compliance"

rules:
  # Requirement 6.2.4: Public-facing web applications protected
  - name: block-critical-vulns
    severity_threshold: CRITICAL
    action: BLOCK
    message: "PCI-DSS 6.2.4: CRITICAL vulnerabilities must be fixed"

  - name: block-high-kev
    conditions:
      - severity: HIGH
        cisa_kev: true
    action: BLOCK
    message: "PCI-DSS 6.2.4: Known exploited vulnerabilities must be fixed"

licenses:
  deny:
    - "GPL-*"  # Copyleft may conflict with PCI compliance

audit:
  enabled: true
  retention_days: 365  # PCI requires 12-month retention
```

### HIPAA Compliance

```yaml
# bazbom.yml
name: "HIPAA Security Rule Compliance"

rules:
  # §164.308(a)(1)(ii)(A): Security risk analysis
  - name: block-reachable-critical
    conditions:
      - severity: CRITICAL
        reachable: true
    action: BLOCK
    message: "HIPAA §164.308: Reachable CRITICAL vulnerabilities pose ePHI risk"

  # §164.312(e)(1): Transmission security
  - name: require-encryption
    required_packages:
      - "org.bouncycastle:*"
    action: WARN
    message: "HIPAA §164.312: Encryption libraries required"

licenses:
  require_license_info: true
  deny:
    - "Unknown"
    - "NOASSERTION"
```

### SOC 2 Type II

```yaml
# bazbom.yml
name: "SOC 2 Type II Compliance"

rules:
  # CC7.1: Detect Security Threats
  - name: daily-scans
    max_scan_age_hours: 24
    action: WARN
    message: "SOC 2 CC7.1: Daily scans required"

  # CC7.2: Respond to Security Incidents
  - name: block-high-epss
    conditions:
      - epss: ">= 0.5"
    action: BLOCK
    message: "SOC 2 CC7.2: High exploitability requires response"

audit:
  enabled: true
  log_all_scans: true
  retention_days: 365  # SOC 2 requires 12-month audit trail
```

---

## Troubleshooting

### Policy Validation Errors

**Problem:** `Invalid policy syntax`

```bash
# Validate policy file
bazbom policy validate bazbom.yml

# Check YAML syntax
yamllint bazbom.yml
```

**Problem:** `Unknown license in denylist`

```bash
# List all recognized licenses
bazbom license list

# Use SPDX identifiers (e.g., "MIT" not "MIT License")
```

### Policy Not Applied

**Problem:** Policy file not being used

```bash
# Ensure file is named correctly
ls bazbom.yml

# Specify policy file explicitly
bazbom scan --policy-file my-policy.yml

# Check file permissions
chmod 644 bazbom.yml
```

### Audit Logs Not Created

**Problem:** No audit.jsonl file

```yaml
# Ensure audit is enabled in policy
audit:
  enabled: true
  log_file: ".bazbom/audit.jsonl"
```

```bash
# Create directory if needed
mkdir -p .bazbom

# Check permissions
ls -la .bazbom/
```

### CI/CD Failures

**Problem:** Policy check fails in CI but passes locally

```bash
# Use same policy file
git add bazbom.yml
git commit -m "Add policy file"

# Check CI environment
echo $BAZBOM_POLICY_FILE
```

**Problem:** Audit logs in CI not persisted

```yaml
# GitHub Actions: Archive audit logs
- uses: actions/upload-artifact@v4
  with:
    name: audit-logs
    path: .bazbom/audit.jsonl
```

---

## Next Steps

- **Advanced Policies**: See [Rego Best Practices](REGO_BEST_PRACTICES.md)
- **License Management**: See [USAGE.md](../USAGE.md#license-commands)
- **Templates**: Browse [examples/policies/](../../examples/policies/)
- **Compliance Checklists**: See [examples/policies/checklists/](../../examples/policies/checklists/)

## Support

For questions or issues:
- [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
- [Documentation](https://github.com/cboyd0319/BazBOM/tree/main/docs)
- [Contributing Guide](../../CONTRIBUTING.md)
