# VEX Statements

This directory contains VEX (Vulnerability Exploitability eXchange) statements for managing false positives and accepted risks in vulnerability scanning.

## Purpose

VEX statements allow you to document why certain vulnerabilities are not applicable to your application, helping to:
- Reduce alert fatigue from false positives
- Document security decisions
- Provide audit trails for compliance
- Suppress known non-issues in CI/CD

## Format

VEX statements use a simplified JSON format:

```json
{
  "cve": "CVE-2023-12345",
  "vulnerability_id": "CVE-2023-12345",
  "package": "pkg:maven/com.example/vulnerable-lib@1.2.3",
  "status": "not_affected",
  "justification": "Explanation of why this vulnerability does not affect us",
  "created": "2023-10-17T00:00:00Z",
  "author": "Security Team"
}
```

### Required Fields

- **cve** or **vulnerability_id**: The CVE or vulnerability identifier
- **status**: One of:
  - `not_affected` - Vulnerability does not affect this package/version
  - `false_positive` - Scanner incorrectly flagged this
  - `mitigated` - Vulnerability is mitigated by other controls
  - `accepted_risk` - Risk is known and accepted
- **justification**: Explanation for the status

### Optional Fields

- **package**: PURL of the specific package (for package-specific suppressions)
- **created**: ISO 8601 timestamp
- **author**: Who created this statement
- **metadata**: Additional metadata for tracking and auditing

## Usage

VEX statements are automatically applied when running the SCA scan:

```bash
bazel run //:sca_scan_with_vex
```

Or manually apply VEX filtering:

```bash
bazel run //tools/supplychain:vex_processor -- \
  --vex-dir=vex/statements \
  --sca-findings=bazel-bin/sca_findings.json \
  --output=bazel-bin/sca_findings_filtered.json
```

## Best Practices

1. **Be Specific**: Include package PURLs when the suppression only applies to specific versions
2. **Document Thoroughly**: Provide clear justifications for audit purposes
3. **Regular Review**: Set review dates and revisit statements periodically
4. **Version Control**: Keep VEX statements in git for audit trails
5. **Team Review**: Have security team review and approve VEX statements

## Example: Creating a VEX Statement

```bash
# Example: False positive from scanner
cat > vex/statements/CVE-2023-99999.json <<EOF
{
  "cve": "CVE-2023-99999",
  "package": "pkg:maven/org.example/mylib@2.0.0",
  "status": "false_positive",
  "justification": "This CVE affects version 1.x only. Version 2.0.0 has a different codebase.",
  "created": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "author": "$(git config user.name)"
}
EOF
```

## CSAF VEX Support

BazBOM also supports the CSAF VEX format for enterprise compliance. See [docs/VEX.md](../docs/VEX.md) for details.
