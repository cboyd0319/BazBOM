# BazBOM Policy Templates

This directory contains pre-built policy templates for common regulatory frameworks and compliance standards. These templates provide a starting point for organizations implementing security and license compliance policies.

## Available Templates

### Regulatory Compliance

#### `pci-dss.yml` - PCI-DSS v4.0 Compliance
Payment Card Industry Data Security Standard for applications handling cardholder data.

**Key Features:**
- Blocks CRITICAL and HIGH vulnerabilities
- CISA KEV enforcement enabled
- Restricts copyleft licenses (GPL, AGPL)
- Suitable for payment processing systems

**Use When:**
- Building payment processing applications
- Handling credit card data
- Required for PCI-DSS certification

---

#### `hipaa.yml` - HIPAA Security Rule
Health Insurance Portability and Accountability Act for applications handling ePHI.

**Key Features:**
- Enforces security risk analysis requirements
- Blocks unknown/unspecified licenses
- Comprehensive vulnerability thresholds
- VEX documentation for audit trails

**Use When:**
- Building healthcare applications
- Handling protected health information (PHI/ePHI)
- Required for HIPAA compliance

---

#### `fedramp-moderate.yml` - FedRAMP Moderate
Federal Risk and Authorization Management Program for cloud services serving federal agencies.

**Key Features:**
- NIST SP 800-53 Rev 5 aligned
- CISA BOD 22-01 KEV enforcement
- 30-day remediation requirements
- Restricts strong copyleft licenses

**Use When:**
- Providing cloud services to federal agencies
- Seeking FedRAMP authorization
- Government contractor requirements

---

#### `soc2.yml` - SOC 2 Type II
Service Organization Control 2 for trust services criteria (Security, Availability).

**Key Features:**
- Continuous monitoring support
- Security incident response requirements
- 365-day audit trail support
- Component inventory documentation

**Use When:**
- Seeking SOC 2 certification
- B2B SaaS applications
- Customer compliance requirements

---

### Development Policies

#### `corporate-permissive.yml` - Corporate Standard (Development)
Permissive policy for development and testing environments.

**Key Features:**
- Warning-only mode (non-blocking)
- All licenses allowed
- Focus on awareness over enforcement
- Suitable for early development

**Use When:**
- Local development environments
- Feature branches
- Internal tools and prototypes
- Early-stage projects

---

## Usage

### Initializing a Template

Use the `bazbom policy init` command to initialize a template in your project:

```bash
# List available templates
bazbom policy init --list

# Initialize a specific template
bazbom policy init --template pci-dss

# This creates bazbom.yml in your project root
```

### Customizing Templates

After initialization, customize the policy file to match your organization's requirements:

1. Copy the template to your project as `bazbom.yml`
2. Adjust severity thresholds based on your risk appetite
3. Modify license allow/deny lists for your specific needs
4. Add exceptions for approved vulnerabilities
5. Configure reachability and VEX settings

### Policy Inheritance

For organizations with multiple teams and projects, use policy inheritance:

```
.bazbom/
├── policies/
│   ├── organization.yml      # Baseline (strictest)
│   ├── team-backend.yml       # Team overrides
│   └── project-api.yml        # Project-specific
└── config.yml                  # Inheritance configuration
```

See the main documentation for policy inheritance details.

---

## Best Practices

### Development → Staging → Production

Use progressively stricter policies:

1. **Development**: `corporate-permissive.yml` - Non-blocking warnings
2. **Staging**: Custom policy with moderate enforcement
3. **Production**: Regulatory template (PCI-DSS, HIPAA, etc.) - Strict enforcement

### Policy Selection Guide

| Your Requirement | Recommended Template |
|-----------------|---------------------|
| Payment processing | `pci-dss.yml` |
| Healthcare/medical data | `hipaa.yml` |
| Federal government cloud | `fedramp-moderate.yml` |
| B2B SaaS (general) | `soc2.yml` |
| Development/testing | `corporate-permissive.yml` |
| Custom requirements | Start with closest template, customize |

### Compliance Notes

- These templates provide baseline security controls
- Consult with your compliance officer or QSA for complete requirements
- Additional organizational policies may be required
- Regular policy reviews and updates recommended
- Maintain audit trails of all policy decisions
- Document exceptions and compensating controls

---

## Policy Structure

All templates follow this structure:

```yaml
name: "Policy Name"
description: "Policy Description"
version: "1.0"

# Vulnerability thresholds
severity_threshold: HIGH
kev_gate: true
epss_threshold: 0.5
reachability_required: false

# License controls
license_denylist: [...]
license_allowlist: [...]

# Documentation
vex_auto_apply: true
```

---

## Advanced Policy Features

### Rego/OPA Support

For complex rules beyond YAML capabilities, use Rego policies:

```bash
# Create advanced.rego with custom logic
bazbom policy check --rego advanced.rego
```

See the Phase 5 documentation and `examples/policies/advanced.rego` for Rego policy examples.

### Policy Validation

Validate your policy file before deployment:

```bash
bazbom policy validate bazbom.yml
```

---

## Contributing

Have a policy template for another regulatory framework? We welcome contributions:

1. Create the template following the existing structure
2. Add comprehensive comments explaining requirements
3. Include compliance officer notes
4. Update this README
5. Submit a pull request

**Requested templates:**
- ISO 27001
- GDPR compliance
- CCPA compliance
- FISMA
- PCI-DSS v3.2.1 (legacy)

---

## Support

For questions about policy templates:
- Open an issue at https://github.com/cboyd0319/BazBOM/issues
- Tag with `policy` label
- Consult `docs/` for detailed documentation

**Disclaimer:** These templates provide baseline security controls and should be reviewed by qualified compliance professionals. BazBOM policy templates are not a substitute for comprehensive compliance programs.
