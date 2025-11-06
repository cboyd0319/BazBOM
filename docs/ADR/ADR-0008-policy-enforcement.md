# ADR-0008: Policy Enforcement Framework

**Status**: Accepted

**Date**: 2025-10-17

**Context**: Supply chain security requires not just detection of vulnerabilities and compliance issues, but also enforcement of security policies in CI/CD pipelines. We need a systematic way to define, check, and enforce security policies that can fail builds when violations occur.

## Decision

We will implement a comprehensive **policy enforcement framework** that validates security and compliance policies against:

1. Vulnerability findings (with severity thresholds)
2. License compliance (blocked licenses, conflicts)
3. Dependency conflicts (version mismatches)
4. Supply chain risks (typosquatting, unmaintained packages)
5. VEX statement requirements (documentation of accepted risks)

## Implementation

### Policy Check Tool

Location: `tools/supplychain/policy_check.py`

Core functionality:
- Load multiple report types (SCA, license, conflicts, risks)
- Apply configurable policy rules
- Generate violation reports
- Exit with appropriate code for CI/CD integration

### Policy Configuration

Policies are defined via command-line flags or configuration files:

```bash
python tools/supplychain/policy_check.py \
  --findings sca_findings_filtered.json \
  --license-report license_report.json \
  --conflicts conflicts.json \
  --risk-report supply_chain_risks.json \
  --max-critical 0 \
  --max-high 5 \
  --blocked-licenses GPL-2.0 GPL-3.0 AGPL-3.0 \
  --block-license-conflicts \
  --require-vex-for-accepted \
  --block-typosquatting
```

### Exit Codes

- **0**: All policy checks passed
- **1**: Policy violations found (build should fail)

This allows CI/CD pipelines to enforce policies automatically:

```yaml
- name: Enforce Security Policies
  run: |
    python tools/supplychain/policy_check.py \
      --findings bazel-bin/sca_findings_filtered.json \
      --max-critical 0 --max-high 5
```

## Rationale

### Why Policy Enforcement is Critical

1. **Automation**: Manual review of security reports is error-prone and doesn't scale
2. **Consistency**: Same policies applied across all projects and teams
3. **Prevention**: Catch violations before deployment, not after
4. **Compliance**: Many regulations require documented security policies
5. **Developer Guidance**: Clear feedback on what needs to be fixed

### Why Command-Line Tool vs. Bazel Rule

**Chosen**: Command-line tool with Bazel wrapper

**Rationale**:
- **Flexibility**: Easy to use in any CI system, not just Bazel
- **Debugging**: Simple to run manually with different configurations
- **Composability**: Can pipe to other tools, integrate with custom workflows
- **Transparency**: Clear what's being checked and why
- **Testing**: Easy to unit test policy logic

**Alternative considered**: Bazel rule with aspects
-  Harder to debug and customize
-  Tight coupling to Bazel
-  Less transparent to developers

## Policy Categories

### 1. Vulnerability Thresholds

Control maximum allowed vulnerabilities by severity:

```python
PolicyChecker({
  'max_critical': 0,      # Strict: no critical vulns
  'max_high': 5,          # Allow some high vulns
  'max_medium': float('inf'),  # Unlimited
  'max_low': float('inf')
})
```

**Rationale**:
- Different severity levels have different risk profiles
- Production may require zero critical/high
- Development may allow more flexibility
- Provides clear remediation target

### 2. License Compliance

Enforce license policies:

```python
PolicyChecker({
  'blocked_licenses': ['GPL-2.0', 'GPL-3.0', 'AGPL-3.0'],
  'block_license_conflicts': True,
  'flag_copyleft': True
})
```

**Use cases**:
- **Proprietary software**: Block copyleft licenses
- **Open source projects**: Block incompatible licenses
- **Corporate policies**: Flag licenses requiring legal review

### 3. VEX Requirements

Require documentation for accepted risks:

```python
PolicyChecker({
  'require_vex_for_accepted': True
})
```

**Rationale**:
- Forces documentation of why vulnerabilities are accepted
- Provides audit trail for security decisions
- Prevents "silent" acceptance of risks

### 4. Supply Chain Risks

Block suspicious dependencies:

```python
PolicyChecker({
  'block_typosquatting': True,
  'unmaintained_threshold': 0
})
```

**Rationale**:
- Typosquatting is a common attack vector
- Unmaintained packages may contain unfixed vulnerabilities
- Early detection prevents supply chain attacks

### 5. Dependency Conflicts

Enforce consistent dependency versions:

```python
PolicyChecker({
  'block_dependency_conflicts': True
})
```

**Rationale**:
- Version conflicts can cause runtime issues
- Makes dependency tree more predictable
- Simplifies troubleshooting

## Policy Violation Reporting

### Console Output

Human-readable violation summary:

```text
 Policy violations found: 3

CRITICAL (1):
  • max_critical_vulnerabilities: Found 1 critical vulnerabilities (max allowed: 0)

HIGH (2):
  • blocked_license: Package foo uses blocked license: GPL-3.0
  • typosquatting_risk: Potential typosquatting detected: googl

Total violations: 3
```

### JSON Output

Machine-readable for automation:

```json
{
  "total_violations": 3,
  "violations": [
    {
      "severity": "CRITICAL",
      "rule": "max_critical_vulnerabilities",
      "message": "Found 1 critical vulnerabilities (max allowed: 0)",
      "details": {
        "count": 1,
        "threshold": 0
      }
    }
  ],
  "policy_config": { ... }
}
```

## Environment-Specific Policies

Different environments have different risk tolerances:

### Production Policy (Strict)

```bash
--max-critical 0 \
--max-high 0 \
--max-medium 5 \
--blocked-licenses GPL-2.0 GPL-3.0 AGPL-3.0 \
--block-license-conflicts \
--require-vex-for-accepted \
--block-typosquatting \
--block-dependency-conflicts
```

**Rationale**: Zero tolerance for high-risk issues in production

### Staging Policy (Moderate)

```bash
--max-critical 0 \
--max-high 3 \
--max-medium 20 \
--flag-copyleft \
--block-typosquatting
```

**Rationale**: Allow some issues for testing, but block critical

### Development Policy (Flexible)

```bash
--max-critical 0 \
--max-high 10 \
--flag-copyleft
```

**Rationale**: Focus on critical issues, allow flexibility for experimentation

### Audit Policy (Reporting Only)

```bash
--max-critical 999 \
--max-high 999 \
--max-medium 999 \
--output audit-report.json
```

**Rationale**: Generate reports without blocking builds

## Integration Points

### 1. Bazel BUILD Target

```python
genrule(
    name = "policy_check_report",
    srcs = [
        ":sca_findings_filtered.json",
        ":license_report",
        ":conflict_report",
        ":supply_chain_risk_report",
    ],
    outs = ["policy_check.json"],
    cmd = "$(location //tools/supplychain:policy_check) " +
          "--findings $(location :sca_findings_filtered.json) " +
          "--license-report $(location :license_report) " +
          "--conflicts $(location :conflict_report) " +
          "--risk-report $(location :supply_chain_risk_report) " +
          "--max-critical 0 --max-high 10 " +
          "--output $@ || echo '{\"violations\": []}' > $@",
    tools = ["//tools/supplychain:policy_check"],
)
```

### 2. GitHub Actions

```yaml
- name: Enforce Security Policies
  run: |
    python tools/supplychain/policy_check.py \
      --findings bazel-bin/sca_findings_filtered.json \
      --license-report bazel-bin/license_report.json \
      --max-critical 0 \
      --max-high 5 \
      --block-license-conflicts \
      --output policy-report.json
    
  # Upload report even if policy check fails
  - name: Upload Policy Report
    if: always()
    uses: actions/upload-artifact@v4
    with:
      name: policy-report
      path: policy-report.json
```

### 3. Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

bazel build //:policy_check_report
if [ $? -ne 0 ]; then
  echo " Policy violations found. Please fix before committing."
  cat bazel-bin/policy_check.json | jq '.violations'
  exit 1
fi
```

### 4. Release Gates

```yaml
# .github/workflows/release.yml
- name: Enforce Production Policies
  run: |
    python tools/supplychain/policy_check.py \
      --findings bazel-bin/sca_findings_filtered.json \
      --max-critical 0 \
      --max-high 0 \
      --require-vex-for-accepted \
      --block-license-conflicts
```

## Consequences

### Positive

- **Automated Enforcement**: Policies applied consistently without manual review
- **Clear Feedback**: Developers know exactly what needs to be fixed
- **Flexible Configuration**: Different policies for different environments
- **Audit Trail**: JSON reports provide evidence of compliance
- **Prevention**: Issues caught before deployment
- **Extensible**: Easy to add new policy types

### Negative

- **Build Friction**: Strict policies may slow down development initially
- **Configuration Overhead**: Need to maintain policy configs per environment
- **False Positives**: May need VEX statements for legitimate edge cases
- **Learning Curve**: Developers need to understand policy framework

### Mitigation

- **Documentation**: Clear docs on policies and how to resolve violations
- **VEX Workflow**: Easy process to document accepted risks
- **Staged Rollout**: Start with reporting, gradually enforce
- **Developer Training**: Educate on supply chain security practices

## Future Enhancements

### 1. Policy-as-Code

YAML/JSON configuration files for policies:

```yaml
# policy.yaml
policies:
  vulnerabilities:
    critical: 0
    high: 5
  licenses:
    blocked:
      - GPL-2.0
      - GPL-3.0
    conflicts: block
  supply_chain:
    typosquatting: block
```

### 2. Policy Inheritance

```yaml
base: production-policy.yaml
overrides:
  vulnerabilities:
    high: 10  # More relaxed for staging
```

### 3. Custom Policy Plugins

```python
class CustomPolicy(PolicyChecker):
    def check_custom_rule(self, data):
        # Custom validation logic
        pass
```

### 4. Policy Exceptions

Per-package exceptions with expiration:

```yaml
exceptions:
  - package: "pkg:maven/org.example/foo@1.0.0"
    rule: max_high_vulnerabilities
    justification: "Mitigated by network isolation"
    expires: "2026-01-01"
```

### 5. Trend Analysis

Track policy violations over time:

```bash
python tools/supplychain/policy_check.py \
  --baseline previous-report.json \
  --current current-report.json \
  --output trend-analysis.json
```

## Testing Strategy

### Unit Tests

```python
def test_vulnerability_threshold_check():
    checker = PolicyChecker({'max_critical': 0})
    findings = {'vulnerabilities': [
        {'severity': 'CRITICAL', 'id': 'CVE-2023-1234'}
    ]}
    checker.check_vulnerability_thresholds(findings)
    assert len(checker.violations) == 1
    assert checker.get_exit_code() == 1
```

### Integration Tests

```bash
# Test full policy check pipeline
bazel build //:policy_check_report
[ $? -eq 0 ] || echo "Policy check failed as expected"
```

### Test Data

Maintain test fixtures for different scenarios:
- No violations (should pass)
- Critical vulnerabilities (should fail)
- License conflicts (should fail with license flag)
- Supply chain risks (should fail with risk flag)

## References

- [NIST SSDF: Secure Software Development Framework](https://csrc.nist.gov/Projects/ssdf)
- [SLSA: Supply Chain Levels for Software Artifacts](https://slsa.dev/)
- [OpenSSF Scorecards](https://github.com/ossf/scorecard)
- [CISA Known Exploited Vulnerabilities Catalog](https://www.cisa.gov/known-exploited-vulnerabilities-catalog)

## Review Date

**Next Review**: 2026-04-01 (6 months)

Review triggers:
- New compliance requirements
- Change in organizational risk tolerance
- Feedback from development teams on policy friction
- New attack vectors or supply chain threats
