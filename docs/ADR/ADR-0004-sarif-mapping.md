# ADR-0004: OSV Severity to SARIF Level Mapping

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Security Team

## Context

OSV and NVD use different severity systems than SARIF. We need consistent mapping to ensure GitHub Code Scanning displays vulnerabilities correctly.

### Severity Systems

<table>
  <thead>
    <tr>
      <th>Source</th>
      <th>Severity Levels</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>CVSS 3.x</td>
      <td>0.0-10.0 (continuous score)</td>
    </tr>
    <tr>
      <td>OSV</td>
      <td>critical, high, moderate, low</td>
    </tr>
    <tr>
      <td>NVD</td>
      <td>CRITICAL, HIGH, MEDIUM, LOW</td>
    </tr>
    <tr>
      <td>SARIF</td>
      <td>error, warning, note, none</td>
    </tr>
  </tbody>
</table>

## Decision

Map vulnerabilities to SARIF levels using this table:

<table>
  <thead>
    <tr>
      <th>OSV/NVD Severity</th>
      <th>CVSS Score Range</th>
      <th>SARIF Level</th>
      <th>GitHub Display</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>CRITICAL</td>
      <td>9.0 - 10.0</td>
      <td>error</td>
      <td>Red, blocks merge (if configured)</td>
    </tr>
    <tr>
      <td>HIGH</td>
      <td>7.0 - 8.9</td>
      <td>error</td>
      <td>Red, alerts required</td>
    </tr>
    <tr>
      <td>MEDIUM/MODERATE</td>
      <td>4.0 - 6.9</td>
      <td>warning</td>
      <td>Yellow, review recommended</td>
    </tr>
    <tr>
      <td>LOW</td>
      <td>0.1 - 3.9</td>
      <td>note</td>
      <td>Gray, informational</td>
    </tr>
    <tr>
      <td>NONE/INFO</td>
      <td>0.0</td>
      <td>none</td>
      <td>Not displayed</td>
    </tr>
  </tbody>
</table>

### Special Cases

1. **No CVSS score available:** Default to `warning` (conservative)
2. **Multiple CVSS scores:** Use highest score
3. **Disputed CVE:** Mark as `note` with justification in message

## Implementation

```python
# tools/supplychain/sarif_adapter.py

def map_severity_to_sarif_level(osv_severity: str, cvss_score: float) -> str:
    """Map OSV/CVSS severity to SARIF level."""

    # Prefer CVSS score (more granular)
    if cvss_score is not None:
        if cvss_score >= 9.0:
            return "error"  # CRITICAL
        elif cvss_score >= 7.0:
            return "error"  # HIGH
        elif cvss_score >= 4.0:
            return "warning"  # MEDIUM
        elif cvss_score > 0.0:
            return "note"  # LOW
        else:
            return "none"  # No risk

    # Fallback to OSV severity
    severity_map = {
        "CRITICAL": "error",
        "HIGH": "error",
        "MODERATE": "warning",
        "MEDIUM": "warning",
        "LOW": "note",
    }
    return severity_map.get(osv_severity.upper(), "warning")  # Default: warning


def create_sarif_result(finding: dict) -> dict:
    """Convert SCA finding to SARIF result."""

    level = map_severity_to_sarif_level(
        finding.get("severity"),
        finding.get("cvss_score")
    )

    return {
        "ruleId": finding["id"],  # OSV-2023-12345
        "level": level,
        "message": {
            "text": f"{finding['summary']} (CVE: {finding['cve']})"
        },
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {
                    "uri": "maven_install.json"
                },
                "region": {
                    "snippet": {
                        "text": finding["package"]["purl"]
                    }
                }
            }
        }]
    }
```

## Consequences

### Positive
- Consistent mapping across all scanners
- CVSS score provides fine-grained control
- GitHub Code Scanning integration works correctly

### Negative
- MEDIUM vulnerabilities may be under-emphasized (warning vs error)
- Subjectivity in borderline cases (e.g., CVSS 6.9 vs 7.0)

### Mitigations
- Allow override via policy: `--error-threshold=medium` promotes warnings to errors
- Document rationale for each mapping

## Alternatives Considered

### Alternative 1: Map MEDIUM → error

**Rationale:** Treat MEDIUM as blocking too.

**Rejected:** Too noisy, blocks too many PRs on low-risk issues.

### Alternative 2: Custom Scoring

**Rationale:** BazBOM-specific risk score considering exploitability, network exposure, etc.

**Rejected:** Complex, not standardized. Stick with CVSS.

## Validation

```bash
# Test mapping
python3 -c "
from tools.supplychain.sarif_adapter import map_severity_to_sarif_level

assert map_severity_to_sarif_level('CRITICAL', 9.8) == 'error'
assert map_severity_to_sarif_level('HIGH', 7.5) == 'error'
assert map_severity_to_sarif_level('MEDIUM', 5.0) == 'warning'
assert map_severity_to_sarif_level('LOW', 2.0) == 'note'
assert map_severity_to_sarif_level(None, 8.5) == 'error'  # Fallback to CVSS
print('✓ All mapping tests passed')
"
```

## References

- [SARIF Specification - Result Level](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html#_Toc34317648)
- [CVSS v3.1 Specification](https://www.first.org/cvss/v3.1/specification-document)
- [OSV Schema](https://ossf.github.io/osv-schema/)
- [GitHub Code Scanning](https://docs.github.com/en/code-security/code-scanning)
