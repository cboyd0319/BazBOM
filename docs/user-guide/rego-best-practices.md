# Rego Policy Authoring Best Practices

This guide explains how to write effective Rego policies for BazBOM's policy engine using Open Policy Agent (OPA) language.

## Table of Contents

1. [Why Rego?](#why-rego)
2. [Getting Started](#getting-started)
3. [Policy Structure](#policy-structure)
4. [Common Patterns](#common-patterns)
5. [Testing Policies](#testing-policies)
6. [Performance Tips](#performance-tips)
7. [Debugging](#debugging)
8. [Examples](#examples)

---

## Why Rego?

**Rego** (pronounced "ray-go") is the policy language for Open Policy Agent (OPA). Use Rego when:

 **Complex Logic**: AND/OR/NOT combinations that are hard to express in YAML  
 **Dynamic Rules**: Policies that depend on context or metadata  
 **Reusable Logic**: Functions and rules that can be shared  
 **Advanced Queries**: Filtering, aggregation, or transformation  

Use YAML when:
- Simple threshold-based rules
- Static allow/deny lists
- Straightforward compliance checks

---

## Getting Started

### Enable Rego Support

BazBOM's Rego support is optional. Build with the feature flag:

```bash
cargo build --features rego
```

Or in `Cargo.toml`:

```toml
[dependencies]
bazbom-policy = { version = "0.2", features = ["rego"] }
```

### Your First Rego Policy

Create `policy.rego`:

```rego
package bazbom

# Block CRITICAL vulnerabilities
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("CRITICAL vulnerability %s in %s", [vuln.id, vuln.package])
}
```

Run with BazBOM:

```bash
bazbom scan --policy policy.rego
```

---

## Policy Structure

### Package Declaration

All BazBOM policies must use the `bazbom` package:

```rego
package bazbom
```

### Rule Types

BazBOM recognizes three rule types:

1. **`deny`**: Violations that block deployment
2. **`warn`**: Issues that require review but don't block
3. **`allow`**: Exceptions to deny/warn rules

```rego
package bazbom

# Block rule
deny[msg] {
    # conditions
    msg := "Violation message"
}

# Warning rule
warn[msg] {
    # conditions
    msg := "Warning message"
}

# Exception rule
allow[msg] {
    # conditions
    msg := "Exception reason"
}
```

### Input Schema

BazBOM provides this input structure:

```json
{
  "vulnerabilities": [
    {
      "id": "CVE-2024-1234",
      "severity": "CRITICAL",
      "priority": "P0",
      "component": "org.example:lib:1.0.0",
      "description": "Remote code execution",
      "kev": true,
      "epss_score": 0.85,
      "reachable": true,
      "fixed_version": "1.0.1"
    }
  ],
  "dependencies": [
    {
      "name": "org.example:lib",
      "version": "1.0.0",
      "license": "MIT",
      "scope": "compile"
    }
  ],
  "metadata": {
    "project": "my-api",
    "commercial": true,
    "environment": "production"
  }
}
```

Access with:
- `input.vulnerabilities[_]` - iterate vulnerabilities
- `input.dependencies[_]` - iterate dependencies
- `input.metadata.project` - access metadata

---

## Common Patterns

### Pattern 1: Severity-Based Blocking

```rego
package bazbom

# Block CRITICAL and HIGH
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity in ["CRITICAL", "HIGH"]
    msg := sprintf("Severity %s not allowed: %s", [vuln.severity, vuln.id])
}
```

### Pattern 2: CISA KEV Enforcement

```rego
package bazbom

# Block all CISA Known Exploited Vulnerabilities
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.kev == true
    msg := sprintf("CISA KEV vulnerability detected: %s", [vuln.id])
}
```

### Pattern 3: EPSS Threshold

```rego
package bazbom

# Block vulnerabilities with high exploit probability
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.epss_score >= 0.5  # >50% exploit probability
    msg := sprintf("High EPSS score (%f) for %s", [vuln.epss_score, vuln.id])
}
```

### Pattern 4: Reachability Analysis

```rego
package bazbom

# Block CRITICAL if reachable
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    vuln.reachable == true
    msg := sprintf("Reachable CRITICAL vulnerability: %s", [vuln.id])
}

# Warn on CRITICAL if not reachable
warn[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    vuln.reachable == false
    msg := sprintf("Unreachable CRITICAL (review): %s", [vuln.id])
}
```

### Pattern 5: License Compatibility

```rego
package bazbom

# Copyleft licenses prohibited
copyleft_licenses := ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]

deny[msg] {
    dep := input.dependencies[_]
    dep.license in copyleft_licenses
    msg := sprintf("Copyleft license %s in %s", [dep.license, dep.name])
}
```

### Pattern 6: Conditional Rules

```rego
package bazbom

# Block HIGH in production, warn in dev
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "HIGH"
    input.metadata.environment == "production"
    msg := sprintf("HIGH severity not allowed in production: %s", [vuln.id])
}

warn[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "HIGH"
    input.metadata.environment == "development"
    msg := sprintf("HIGH severity detected in dev: %s", [vuln.id])
}
```

### Pattern 7: Time-Based Exceptions

```rego
package bazbom

import future.keywords.in

# Approved exceptions
exceptions := [
    {
        "cve": "CVE-2023-12345",
        "expires": "2025-12-31T23:59:59Z",
        "reason": "Mitigated by WAF"
    }
]

# Allow if exception is valid and not expired
allow[msg] {
    vuln := input.vulnerabilities[_]
    exception := exceptions[_]
    vuln.id == exception.cve
    time.now_ns() < time.parse_rfc3339_ns(exception.expires)
    msg := sprintf("Exception approved for %s: %s", [vuln.id, exception.reason])
}
```

### Pattern 8: Aggregation

```rego
package bazbom

# Deny if total CRITICAL count exceeds threshold
deny[msg] {
    critical_count := count([v | v := input.vulnerabilities[_]; v.severity == "CRITICAL"])
    critical_count > 5
    msg := sprintf("Too many CRITICAL vulnerabilities: %d (max: 5)", [critical_count])
}
```

### Pattern 9: External Data

```rego
package bazbom

# Load approved packages from external file
approved_packages := data.approved_list

deny[msg] {
    dep := input.dependencies[_]
    not dep.name in approved_packages
    msg := sprintf("Package not approved: %s", [dep.name])
}
```

To use external data:

```json
// approved_list.json
{
    "approved_list": [
        "org.springframework:spring-core",
        "org.slf4j:slf4j-api",
        "com.google.guava:guava"
    ]
}
```

```bash
bazbom scan --policy policy.rego --data approved_list.json
```

---

## Testing Policies

### Unit Testing with OPA

Create `policy_test.rego`:

```rego
package bazbom

test_deny_critical {
    deny["CRITICAL vulnerability detected: CVE-2024-1234"] with input as {
        "vulnerabilities": [{
            "id": "CVE-2024-1234",
            "severity": "CRITICAL",
            "package": "example:lib:1.0"
        }]
    }
}

test_allow_low_severity {
    not deny[_] with input as {
        "vulnerabilities": [{
            "id": "CVE-2024-5678",
            "severity": "LOW",
            "package": "example:lib:1.0"
        }]
    }
}

test_kev_blocked {
    count(deny) > 0 with input as {
        "vulnerabilities": [{
            "id": "CVE-2024-9999",
            "severity": "MEDIUM",
            "kev": true
        }]
    }
}
```

Run tests:

```bash
opa test policy.rego policy_test.rego
```

### Integration Testing

```bash
# Create test SBOM
cat > test-sbom.json <<EOF
{
  "vulnerabilities": [
    {
      "id": "CVE-2024-1234",
      "severity": "CRITICAL",
      "package": "test:lib:1.0"
    }
  ]
}
EOF

# Test policy
bazbom scan --policy policy.rego --input test-sbom.json
```

---

## Performance Tips

### 1. Use Indexed Lookups

 **Slow:**
```rego
deny[msg] {
    vuln := input.vulnerabilities[_]
    some i
    input.dependencies[i].name == vuln.package
}
```

 **Fast:**
```rego
deny[msg] {
    vuln := input.vulnerabilities[_]
    dep_names := {d.name | d := input.dependencies[_]}
    vuln.package in dep_names
}
```

### 2. Avoid Redundant Iterations

 **Slow:**
```rego
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("CRITICAL: %s", [vuln.id])
}

deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.kev == true
    msg := sprintf("KEV: %s", [vuln.id])
}
```

 **Fast:**
```rego
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("CRITICAL: %s", [vuln.id])
}

deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.kev == true
    not vuln.severity == "CRITICAL"  # Avoid duplicate
    msg := sprintf("KEV: %s", [vuln.id])
}
```

### 3. Use Comprehensions

 **Slow:**
```rego
critical_vulns[vuln.id] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
}

count_critical := count(critical_vulns)
```

 **Fast:**
```rego
count_critical := count([v | v := input.vulnerabilities[_]; v.severity == "CRITICAL"])
```

### 4. Cache Expensive Operations

```rego
# Cache external lookups
approved_packages := data.approved_list

# Reuse across rules
deny[msg] {
    dep := input.dependencies[_]
    not dep.name in approved_packages
    msg := sprintf("Not approved: %s", [dep.name])
}
```

---

## Debugging

### 1. Print Debugging

Use `trace()` to debug:

```rego
package bazbom

deny[msg] {
    vuln := input.vulnerabilities[_]
    trace(sprintf("Checking vuln: %v", [vuln]))
    vuln.severity == "CRITICAL"
    msg := sprintf("CRITICAL: %s", [vuln.id])
}
```

### 2. OPA REPL

Interactive testing:

```bash
opa run policy.rego

# In REPL:
> data.bazbom.deny
> input := {"vulnerabilities": [{"id": "CVE-2024-1234", "severity": "CRITICAL"}]}
> data.bazbom.deny
```

### 3. Policy Playground

Use [OPA Playground](https://play.openpolicyagent.org/) for online testing.

### 4. Validate Syntax

```bash
opa check policy.rego
```

---

## Examples

### Example 1: PCI-DSS Compliance

```rego
package bazbom

# PCI-DSS Requirement 6.2.4: Public-facing applications protected
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("PCI-DSS 6.2.4: CRITICAL vulnerability %s must be fixed", [vuln.id])
}

deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "HIGH"
    vuln.kev == true
    msg := sprintf("PCI-DSS 6.2.4: Known exploited HIGH vulnerability %s", [vuln.id])
}

# License restrictions
copyleft_licenses := ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]

deny[msg] {
    dep := input.dependencies[_]
    dep.license in copyleft_licenses
    msg := sprintf("PCI-DSS: Copyleft license %s may conflict", [dep.license])
}
```

### Example 2: HIPAA Compliance

```rego
package bazbom

# HIPAA §164.308(a)(1)(ii)(A): Security risk analysis
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    vuln.reachable == true
    msg := sprintf("HIPAA §164.308: Reachable CRITICAL %s poses ePHI risk", [vuln.id])
}

# §164.308(a)(5)(ii)(B): Protection from malicious software
deny[msg] {
    dep := input.dependencies[_]
    dep.malicious == true
    msg := sprintf("HIPAA §164.308: Malicious package %s prohibited", [dep.name])
}

# License requirements
deny[msg] {
    dep := input.dependencies[_]
    dep.license in ["Unknown", "NOASSERTION"]
    msg := sprintf("HIPAA: License info required for %s", [dep.name])
}
```

### Example 3: FedRAMP Moderate

```rego
package bazbom

import future.keywords.in

# NIST SP 800-53 RA-5: Vulnerability Scanning
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.kev == true
    msg := sprintf("FedRAMP/RA-5: CISA KEV %s per BOD 22-01", [vuln.id])
}

# NIST SP 800-53 SI-2: Flaw Remediation (30-day window)
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity in ["CRITICAL", "HIGH"]
    vuln.age_days > 30
    msg := sprintf("FedRAMP/SI-2: %s vulnerability %s exceeds 30-day window", 
                   [vuln.severity, vuln.id])
}

# SLSA provenance required
warn[msg] {
    not input.metadata.slsa_provenance
    msg := "FedRAMP: SLSA provenance recommended for supply chain security"
}
```

### Example 4: Multi-Environment

```rego
package bazbom

import future.keywords.in

# Production: Block CRITICAL and HIGH
deny[msg] {
    input.metadata.environment == "production"
    vuln := input.vulnerabilities[_]
    vuln.severity in ["CRITICAL", "HIGH"]
    msg := sprintf("Production: %s severity %s not allowed", [vuln.severity, vuln.id])
}

# Staging: Warn on CRITICAL, allow HIGH
warn[msg] {
    input.metadata.environment == "staging"
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    msg := sprintf("Staging: CRITICAL %s should be fixed", [vuln.id])
}

# Development: Info only
# (No deny rules for development)
```

---

## Best Practices Summary

 **DO:**
- Use descriptive rule names and messages
- Include compliance requirement references (e.g., "PCI-DSS 6.2.4")
- Test policies with unit tests
- Cache expensive operations
- Use comprehensions for filtering
- Document complex logic with comments

 **DON'T:**
- Use generic error messages
- Iterate multiple times over same data
- Forget to handle missing fields
- Use unbounded recursion
- Mix concerns in single rule

---

## Resources

- [OPA Documentation](https://www.openpolicyagent.org/docs/latest/)
- [Rego Language Reference](https://www.openpolicyagent.org/docs/latest/policy-language/)
- [OPA Playground](https://play.openpolicyagent.org/)
- [BazBOM Examples](../../examples/policies/)

## Next Steps

- Review [Policy Integration Guide](policy-integration.md)
- Browse [Policy Templates](../../examples/policies/)
- Check [Compliance Checklists](../../examples/policies/checklists/)

## Support

For questions or issues:
- [GitHub Issues](https://github.com/cboyd0319/BazBOM/issues)
- [Documentation](https://github.com/cboyd0319/BazBOM/tree/main/docs)
