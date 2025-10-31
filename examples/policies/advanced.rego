# BazBOM Advanced Policy (Rego/OPA)
# This example demonstrates complex policy rules using Rego
# that go beyond what YAML policies can express.

package bazbom

# Block if CRITICAL vulnerability is reachable
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "CRITICAL"
    vuln.reachable == true
    msg := sprintf("CRITICAL vulnerability %s is reachable in %s", [vuln.id, vuln.package])
}

# Block if CISA KEV regardless of severity
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.cisa_kev == true
    msg := sprintf("CISA KEV vulnerability %s must be fixed immediately (BOD 22-01)", [vuln.id])
}

# Block if HIGH severity with EPSS > 0.7
deny[msg] {
    vuln := input.vulnerabilities[_]
    vuln.severity == "HIGH"
    vuln.epss_score > 0.7
    msg := sprintf("HIGH severity vulnerability %s has high exploit probability (EPSS: %v)", [vuln.id, vuln.epss_score])
}

# Warn if license is copyleft and used in commercial product
warn[msg] {
    dep := input.dependencies[_]
    copyleft_licenses := {"GPL-2.0", "GPL-3.0", "AGPL-3.0", "GPL-2.0-only", "GPL-3.0-only", "AGPL-3.0-only"}
    dep.license in copyleft_licenses
    input.metadata.commercial == true
    msg := sprintf("Copyleft license %s in dependency %s may conflict with commercial use", [dep.license, dep.name])
}

# Warn if dependency has no license information
warn[msg] {
    dep := input.dependencies[_]
    unlicensed := {"NOASSERTION", "Unknown", "NONE", ""}
    dep.license in unlicensed
    msg := sprintf("Dependency %s has no license information", [dep.name])
}

# Warn if vulnerability is MEDIUM+ but not reachable
warn[msg] {
    vuln := input.vulnerabilities[_]
    severity_levels := {"MEDIUM", "HIGH", "CRITICAL"}
    vuln.severity in severity_levels
    vuln.reachable == false
    msg := sprintf("Vulnerability %s (%s) is not reachable but should be monitored", [vuln.id, vuln.severity])
}

# Allow exceptions for approved CVEs with valid expiration
allow[msg] {
    vuln := input.vulnerabilities[_]
    exception := data.exceptions[_]
    vuln.id == exception.cve
    
    # Check if exception is not expired
    now := time.now_ns()
    expiration := time.parse_rfc3339_ns(exception.expires)
    now < expiration
    
    msg := sprintf("Exception approved for %s until %s (approved by: %s)", [vuln.id, exception.expires, exception.approved_by])
}

# Allow if vulnerability has compensating control
allow[msg] {
    vuln := input.vulnerabilities[_]
    control := data.compensating_controls[_]
    vuln.id == control.cve
    control.effective == true
    msg := sprintf("Compensating control in place for %s: %s", [vuln.id, control.description])
}

# Complex rule: Block if multiple HIGH vulnerabilities in same package
deny[msg] {
    package_name := input.vulnerabilities[_].package
    high_vulns := [v | v := input.vulnerabilities[_]; v.package == package_name; v.severity == "HIGH"]
    count(high_vulns) >= 3
    msg := sprintf("Package %s has %d HIGH vulnerabilities - consider replacing", [package_name, count(high_vulns)])
}

# Complex rule: Warn about outdated dependencies (>2 years old)
warn[msg] {
    dep := input.dependencies[_]
    dep.published_date
    now := time.now_ns()
    published := time.parse_rfc3339_ns(dep.published_date)
    age_seconds := (now - published) / 1000000000
    age_days := age_seconds / 86400
    age_days > 730  # More than 2 years
    msg := sprintf("Dependency %s is %d days old - consider upgrading", [dep.name, age_days])
}

# License compatibility check
deny[msg] {
    project_license := input.metadata.project_license
    project_license == "MIT"
    
    dep := input.dependencies[_]
    incompatible_licenses := {"GPL-3.0", "AGPL-3.0", "GPL-3.0-only", "AGPL-3.0-only"}
    dep.license in incompatible_licenses
    
    msg := sprintf("License incompatibility: MIT project cannot use %s dependency (%s)", [dep.license, dep.name])
}

# Supply chain security: Flag dependencies without provenance
warn[msg] {
    dep := input.dependencies[_]
    not dep.provenance
    dep.direct == true  # Only flag direct dependencies
    msg := sprintf("Direct dependency %s lacks SLSA provenance - supply chain risk", [dep.name])
}
