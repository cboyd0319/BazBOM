# SARIF Format

BazBOM's SARIF 2.1.0 output for vulnerability reporting.

## Overview

**Format:** SARIF 2.1.0 (JSON)  
**Spec:** <https://docs.oasis-open.org/sarif/sarif/v2.1.0/>  
**Output:** `sca_findings.sarif`  
**Status:** Production-ready

**Why SARIF?**
- GitHub Code Scanning integration
- VS Code, IntelliJ IDEA support
- Standardized security findings format
- Tool-agnostic vulnerability reporting

## Generation

```bash
# SARIF is generated automatically
bazbom scan .

# Output: sca_findings.sarif
```

**Default:** Always generated alongside SBOM and JSON findings.

## Field Mapping

| BazBOM Data | SARIF Field | Notes |
|-------------|-------------|-------|
| CVE ID | `results[].ruleId` | CVE-YYYY-NNNNN |
| Severity | `results[].level` | error/warning/note |
| Description | `results[].message.text` | CVE description |
| Package | `results[].locations[]` | Vulnerable dependency |
| Fixed version | `results[].fixes[]` | Upgrade suggestion |
| CWE | `results[].cweIds[]` | Weakness classification |

## Example Output

```json
{
  "version": "2.1.0",
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "runs": [{
    "tool": {
      "driver": {
        "name": "BazBOM",
        "version": "1.0.0",
        "informationUri": "https://github.com/cboyd0319/BazBOM",
        "rules": [
          {
            "id": "CVE-2024-1234",
            "name": "Log4j Remote Code Execution",
            "shortDescription": {
              "text": "Apache Log4j allows remote code execution"
            },
            "fullDescription": {
              "text": "A vulnerability in Apache Log4j allows attackers to execute arbitrary code..."
            },
            "helpUri": "https://nvd.nist.gov/vuln/detail/CVE-2024-1234",
            "properties": {
              "tags": ["security", "external/cwe/cwe-502"],
              "precision": "high",
              "severity": "critical",
              "cvss": 9.8,
              "epss": 0.85,
              "kev": true
            }
          }
        ]
      }
    },
    "results": [
      {
        "ruleId": "CVE-2024-1234",
        "level": "error",
        "message": {
          "text": "Vulnerability CVE-2024-1234 found in log4j-core 2.17.0"
        },
        "locations": [{
          "physicalLocation": {
            "artifactLocation": {
              "uri": "pom.xml"
            },
            "region": {
              "startLine": 45,
              "snippet": {
                "text": "<artifactId>log4j-core</artifactId>"
              }
            }
          }
        }],
        "fixes": [{
          "description": {
            "text": "Upgrade log4j-core to 2.17.1 or later"
          },
          "artifactChanges": [{
            "artifactLocation": {
              "uri": "pom.xml"
            },
            "replacements": [{
              "deletedRegion": {
                "startLine": 46
              },
              "insertedContent": {
                "text": "<version>2.17.1</version>"
              }
            }]
          }]
        }]
      }
    ]
  }]
}
```

## Severity Mapping

| BazBOM Priority | SARIF Level | Notes |
|-----------------|-------------|-------|
| P0 (CRITICAL + KEV) | error | Actively exploited |
| CRITICAL | error | CVSS ≥ 9.0 |
| HIGH | error | CVSS 7.0-8.9 |
| MEDIUM | warning | CVSS 4.0-6.9 |
| LOW | note | CVSS < 4.0 |

## GitHub Code Scanning Integration

```yaml
# .github/workflows/security.yml
- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v3
  with:
    sarif_file: sca_findings.sarif
```

**Result:** Vulnerabilities appear in GitHub Security tab

**Gotcha:** Requires `security-events: write` permission

## IDE Integration

**VS Code:**
- SARIF Viewer extension auto-detects `*.sarif` files
- Shows findings in Problems panel

**IntelliJ IDEA:**
- SARIF plugin available in marketplace
- Inline annotations in editor

## Validation

```bash
# BazBOM validates by default
bazbom scan . --validate-schemas

# Manual validation
npm install -g @microsoft/sarif-multitool
sarif validate sca_findings.sarif
```

## Orchestrated Scanning

BazBOM merges findings from multiple tools:

```bash
# SCA + Semgrep + CodeQL → Single SARIF
bazbom scan . --with-semgrep --with-codeql=security-extended
```

**Output:** Unified SARIF with all findings

**Why:** Single upload to GitHub Code Scanning

**Details:** [../integrations/orchestrated-scan.md](../integrations/orchestrated-scan.md)

## Filtering Results

```bash
# Only CRITICAL/HIGH
jq '.runs[0].results |= map(select(.level == "error"))' sca_findings.sarif > filtered.sarif

# Only KEV vulnerabilities
jq '.runs[0].results |= map(select(.properties.kev == true))' sca_findings.sarif > kev-only.sarif
```

## Converting to Other Formats

```bash
# SARIF → CSV
jq -r '.runs[0].results[] | [.ruleId, .level, .message.text] | @csv' sca_findings.sarif > findings.csv

# SARIF → Markdown table
# Use external tools like sarif-to-markdown
```

## Next Steps

- [SPDX format](../FORMAT_SPDX.md) - SBOM format
- [CycloneDX format](cyclonedx.md) - Alternative SBOM
- [GitHub integration](../CI.md) - Upload SARIF to Security tab
- [Orchestrated scanning](../integrations/orchestrated-scan.md) - Multi-tool analysis
