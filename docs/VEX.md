# VEX (Vulnerability Exploitability eXchange) Guide

**Audience:** Security engineers, security analysts, developers
**Purpose:** Manage false positives and accepted risks in vulnerability findings
**Last Reviewed:** 2025-10-17

## TL;DR

VEX statements document why a vulnerability does not affect your application. Use VEX to suppress false positives without ignoring real security issues.

```bash
# Create VEX statement
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-12345 \
  --package="pkg:maven/org.example/foo@1.2.3" \
  --status="not_affected" \
  --justification="Vulnerable code path not used" \
  --output=vex/statements/CVE-2023-12345.json

# Apply VEX to filter findings
bazel run //:apply_vex -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

## What is VEX?

VEX (Vulnerability Exploitability eXchange) is a standard format for communicating whether a vulnerability affects a specific product. It addresses the problem of "known false positives."

**Example scenario:**
- Your SBOM lists `log4j-core:2.14.1`
- Scanner flags CVE-2021-44228 (Log4Shell)
- Your code only uses Log4j for compile-time annotation processing
- **Solution:** Issue a VEX statement: "not_affected" with justification

## VEX Status Types

<table>
  <thead>
    <tr>
      <th>Status</th>
      <th>Meaning</th>
      <th>Use When</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>not_affected</code></td>
      <td>Vulnerability does not impact this product</td>
      <td>Vulnerable code path not used</td>
    </tr>
    <tr>
      <td><code>affected</code></td>
      <td>Vulnerability impacts this product</td>
      <td>Confirming scanner is correct</td>
    </tr>
    <tr>
      <td><code>fixed</code></td>
      <td>Vulnerability was fixed in this version</td>
      <td>Patched or upgraded</td>
    </tr>
    <tr>
      <td><code>under_investigation</code></td>
      <td>Status unknown, investigating</td>
      <td>Need time to analyze</td>
    </tr>
  </tbody>
</table>

## VEX Justification Types

For `not_affected` status, provide a justification:

<table>
  <thead>
    <tr>
      <th>Justification</th>
      <th>Description</th>
      <th>Example</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><code>component_not_present</code></td>
      <td>Vulnerable component not included</td>
      <td>Compile-time only dependency</td>
    </tr>
    <tr>
      <td><code>vulnerable_code_not_present</code></td>
      <td>Vulnerable code removed</td>
      <td>Stripped down build</td>
    </tr>
    <tr>
      <td><code>vulnerable_code_not_in_execute_path</code></td>
      <td>Code exists but never called</td>
      <td>Dead code, test-only path</td>
    </tr>
    <tr>
      <td><code>vulnerable_code_cannot_be_controlled_by_adversary</code></td>
      <td>Input sanitized</td>
      <td>Validated before reaching vuln</td>
    </tr>
    <tr>
      <td><code>inline_mitigations_already_exist</code></td>
      <td>Mitigated via other controls</td>
      <td>WAF, network segmentation</td>
    </tr>
  </tbody>
</table>

## VEX Document Structure (CSAF 2.0)

```json
{
  "document": {
    "category": "csaf_vex",
    "csaf_version": "2.0",
    "publisher": {
      "category": "vendor",
      "name": "Example Org",
      "namespace": "https://example.com"
    },
    "title": "VEX for CVE-2023-12345 in foo-1.2.3",
    "tracking": {
      "id": "VEX-2023-001",
      "status": "final",
      "version": "1",
      "revision_history": [
        {
          "number": "1",
          "date": "2025-10-17T12:00:00Z",
          "summary": "Initial VEX statement"
        }
      ],
      "initial_release_date": "2025-10-17T12:00:00Z",
      "current_release_date": "2025-10-17T12:00:00Z"
    }
  },
  "product_tree": {
    "branches": [
      {
        "category": "product_name",
        "name": "BazBOM App",
        "branches": [
          {
            "category": "product_version",
            "name": "1.0.0",
            "product": {
              "product_id": "BazBOM-App-1.0.0",
              "name": "BazBOM App 1.0.0"
            }
          }
        ]
      }
    ]
  },
  "vulnerabilities": [
    {
      "cve": "CVE-2023-12345",
      "product_status": {
        "known_not_affected": ["BazBOM-App-1.0.0"]
      },
      "threats": [
        {
          "category": "impact",
          "details": "The vulnerable code path in org.example.foo:1.2.3 is not reachable in our application. We only use the Bar class which does not contain the vulnerability.",
          "product_ids": ["BazBOM-App-1.0.0"]
        }
      ],
      "flags": [
        {
          "label": "vulnerable_code_not_in_execute_path",
          "product_ids": ["BazBOM-App-1.0.0"]
        }
      ]
    }
  ]
}
```

## Creating VEX Statements

### Using BazBOM CLI

```bash
# Basic VEX statement
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-12345 \
  --package="pkg:maven/org.example/foo@1.2.3" \
  --product="BazBOM App" \
  --product-version="1.0.0" \
  --status="not_affected" \
  --justification="vulnerable_code_not_in_execute_path" \
  --details="The FooService class containing the vulnerability is never instantiated in our codebase. Static analysis confirms no call path reaches the vulnerable method." \
  --output=vex/statements/CVE-2023-12345.json
```

### Manual VEX Creation

1. **Investigate the vulnerability:**

```bash
# Review scanner findings
jq '.findings[] | select(.cve == "CVE-2023-12345")' bazel-bin/sca_findings.json

# Check where package is used
bazel query 'deps(//...)' --output=graph | grep 'org.example.foo'

# Static analysis: Is vulnerable code reachable?
# (Use CodeQL, Semgrep, or manual code review)
```

2. **Document analysis:**

```markdown
## CVE-2023-12345 Analysis

**Package:** org.example:foo:1.2.3
**Vulnerability:** Remote code execution in FooService.parse()
**Affected versions:** < 1.2.4

**Our usage:**
- Imported via //lib:common (transitive from guava)
- Only Bar.validate() method used
- FooService class never instantiated
- parse() method never called

**Conclusion:** Not affected
**Justification:** vulnerable_code_not_in_execute_path
```

3. **Create VEX statement:**

```bash
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-12345 \
  --package="pkg:maven/org.example/foo@1.2.3" \
  --status="not_affected" \
  --justification="vulnerable_code_not_in_execute_path" \
  --details="FooService.parse() is never called. Only Bar.validate() is used." \
  --analysis-doc="docs/security/CVE-2023-12345-analysis.md" \
  --output=vex/statements/CVE-2023-12345.json
```

### VEX for Multiple Products

```bash
# Same vulnerability, multiple product versions
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-12345 \
  --package="pkg:maven/org.example/foo@1.2.3" \
  --products="App-1.0.0,App-1.1.0,App-2.0.0" \
  --status="not_affected" \
  --justification="component_not_present" \
  --details="Dependency removed in all versions since 1.0.0" \
  --output=vex/statements/CVE-2023-12345-multi.json
```

## Applying VEX Statements

### Filter SCA Findings

```bash
# Apply all VEX statements in directory
bazel run //:apply_vex -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json

# Apply single VEX statement
bazel run //:apply_vex -- \
  --vex-file=vex/statements/CVE-2023-12345.json \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

### Verify Filtering

```bash
# Before: 10 findings
jq '.summary.total_findings' bazel-bin/sca_findings.json
# Output: 10

# After: 8 findings (2 suppressed by VEX)
jq '.summary.total_findings' bazel-bin/sca_findings_filtered.json
# Output: 8

# Show suppressed findings
jq '.suppressed[] | {cve, reason}' bazel-bin/sca_findings_filtered.json
```

## VEX Workflow in CI

### GitHub Actions Integration

```yaml
- name: Run SCA scan
  run: bazel run //:sca_scan

- name: Apply VEX statements
  run: |
    bazel run //:apply_vex -- \
      --vex-dir=vex/statements \
      --sca-findings=bazel-bin/sca_findings.json \
      --output=bazel-bin/sca_findings_filtered.json

- name: Check for critical vulnerabilities
  run: |
    CRITICAL=$(jq '.summary.by_severity.critical' bazel-bin/sca_findings_filtered.json)
    if [ "$CRITICAL" -gt 0 ]; then
      echo "❌ $CRITICAL critical vulnerabilities found after VEX filtering"
      exit 1
    fi

- name: Upload filtered SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: bazel-bin/sca_findings_filtered.sarif
```

### Require Justification Review

```yaml
- name: Validate VEX statements
  run: |
    # Ensure all VEX statements have proper justification
    for vex in vex/statements/*.json; do
      STATUS=$(jq -r '.vulnerabilities[0].product_status | keys[0]' "$vex")
      if [ "$STATUS" = "known_not_affected" ]; then
        JUSTIFICATION=$(jq -r '.vulnerabilities[0].flags[0].label' "$vex")
        if [ -z "$JUSTIFICATION" ]; then
          echo "❌ $vex missing justification"
          exit 1
        fi
      fi
    done
```

## VEX Statement Management

### Directory Structure

```
vex/
├── statements/
│   ├── CVE-2023-12345.json          # Individual VEX per CVE
│   ├── CVE-2023-54321.json
│   └── log4j-false-positives.json   # Grouped VEX (multiple CVEs)
├── archive/
│   └── CVE-2022-11111.json          # Expired/obsolete VEX
└── templates/
    └── vex-template.json            # Template for new statements
```

### Version Control

**Commit VEX statements to git:**

```bash
git add vex/statements/CVE-2023-12345.json
git commit -m "security: add VEX for CVE-2023-12345 (not affected)"
```

**Require review:**

Add to `CODEOWNERS`:

```
# VEX statements require security team approval
vex/statements/*.json @security-team
```

### Expiration Policy

VEX statements should expire when:
- Dependency is upgraded (vuln no longer present)
- Product version changes significantly
- Vulnerability is re-analyzed

```bash
# Archive expired VEX
mv vex/statements/CVE-2022-11111.json vex/archive/

# Update VEX with new version
bazel run //tools/supplychain:update_vex -- \
  --vex=vex/statements/CVE-2023-12345.json \
  --product-version="2.0.0" \
  --revalidate
```

## Auditing VEX Usage

### Generate VEX Report

```bash
bazel run //tools/supplychain:vex_report -- \
  --vex-dir=vex/statements \
  --output=vex-audit.json
```

Output:

```json
{
  "total_vex_statements": 12,
  "by_status": {
    "not_affected": 10,
    "under_investigation": 2
  },
  "by_justification": {
    "vulnerable_code_not_in_execute_path": 7,
    "component_not_present": 3
  },
  "oldest_statement": {
    "cve": "CVE-2022-11111",
    "date": "2023-05-10",
    "age_days": 525
  },
  "pending_review": [
    {
      "cve": "CVE-2023-99999",
      "status": "under_investigation",
      "age_days": 14
    }
  ]
}
```

### VEX Coverage Metrics

```bash
# Calculate VEX coverage
TOTAL_FINDINGS=$(jq '.summary.total_findings' bazel-bin/sca_findings.json)
VEX_SUPPRESSED=$(jq '.suppressed | length' bazel-bin/sca_findings_filtered.json)
COVERAGE=$(echo "scale=2; $VEX_SUPPRESSED / $TOTAL_FINDINGS * 100" | bc)
echo "VEX coverage: ${COVERAGE}%"
```

## VEX Best Practices

### 1. Document Analysis Process

Every VEX statement should link to analysis documentation:

```bash
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-12345 \
  --package="pkg:maven/org.example/foo@1.2.3" \
  --status="not_affected" \
  --justification="vulnerable_code_not_in_execute_path" \
  --analysis-doc="docs/security/CVE-2023-12345.md" \
  --analyzer="security-team@example.com" \
  --analysis-date="2025-10-17" \
  --output=vex/statements/CVE-2023-12345.json
```

### 2. Use Specific Justifications

**Bad:**
```json
{
  "details": "Not affected"
}
```

**Good:**
```json
{
  "details": "The vulnerability exists in FooService.parseXML() which parses untrusted XML input. Our application only uses FooService.validateSchema() with static, trusted schemas. The parseXML() method is never called (confirmed via static analysis with CodeQL query: 'calls to FooService.parseXML()').",
  "flags": [
    {
      "label": "vulnerable_code_not_in_execute_path"
    }
  ]
}
```

### 3. Review VEX Statements Quarterly

Set calendar reminder to review all VEX statements every 3 months:

```bash
# List VEX statements older than 90 days
find vex/statements -name "*.json" -mtime +90 -exec basename {} \;
```

### 4. Version VEX with Products

Update VEX statements when product version changes:

```bash
# Update all VEX for new product version
bazel run //tools/supplychain:bulk_update_vex -- \
  --vex-dir=vex/statements \
  --old-version="1.0.0" \
  --new-version="2.0.0" \
  --revalidate-all
```

## Common VEX Scenarios

### Scenario 1: Test-Only Dependency

```bash
# log4j used only in tests
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2021-44228 \
  --package="pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1" \
  --status="not_affected" \
  --justification="component_not_present" \
  --details="log4j-core is a test-scoped dependency (maven scope: test). It is not included in production builds or runtime classpath." \
  --output=vex/statements/CVE-2021-44228.json
```

### Scenario 2: Mitigated by WAF

```bash
# Vulnerability mitigated by Web Application Firewall
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-88888 \
  --package="pkg:maven/org.example/web-parser@1.0.0" \
  --status="not_affected" \
  --justification="inline_mitigations_already_exist" \
  --details="The SQL injection vulnerability is mitigated by CloudFlare WAF rules 100001 and 100002, which block all SQL metacharacters before requests reach the application. Tested on 2025-10-15 with OWASP SQLi test vectors." \
  --mitigation-doc="docs/security/waf-rules.md" \
  --output=vex/statements/CVE-2023-88888.json
```

### Scenario 3: Fixed in Custom Patch

```bash
# Applied custom patch while awaiting official release
bazel run //tools/supplychain:create_vex -- \
  --cve=CVE-2023-77777 \
  --package="pkg:maven/com.vendor/lib@3.2.1" \
  --status="fixed" \
  --details="Applied vendor-provided patch from security advisory SA-2023-07. Patch file: patches/lib-3.2.1-CVE-2023-77777.patch" \
  --patch-file="patches/lib-3.2.1-CVE-2023-77777.patch" \
  --output=vex/statements/CVE-2023-77777.json
```

## Troubleshooting

### VEX Not Filtering Finding

**Check VEX matches finding:**

```bash
# Extract package PURL from finding
FINDING_PURL=$(jq -r '.findings[0].package.purl' bazel-bin/sca_findings.json)

# Check VEX references same PURL
VEX_PURL=$(jq -r '.vulnerabilities[0].product_tree.purl' vex/statements/CVE-2023-12345.json)

# Compare (must match exactly)
if [ "$FINDING_PURL" = "$VEX_PURL" ]; then
  echo "✓ PURL matches"
else
  echo "✗ PURL mismatch: $FINDING_PURL != $VEX_PURL"
fi
```

### VEX Schema Validation Failure

```bash
# Validate VEX against CSAF schema
bazel run //tools/supplychain/validators:validate_vex -- \
  vex/statements/CVE-2023-12345.json

# Common issues:
# - Missing required fields (publisher, tracking)
# - Invalid status value (typo: "not_effected" vs "not_affected")
# - Malformed date (use ISO 8601: 2025-10-17T12:00:00Z)
```

## References

- [CSAF VEX Specification](https://docs.oasis-open.org/csaf/csaf/v2.0/csaf-v2.0.html)
- [VEX Justification Guidance](https://www.cisa.gov/sites/default/files/publications/VEX_Use_Cases_April2022.pdf)
- [CISA VEX Status Codes](https://www.cisa.gov/sites/default/files/publications/VEX_Status_Justification_Jun22.pdf)
- [OpenVEX](https://github.com/openvex/spec) (Simplified VEX format)
