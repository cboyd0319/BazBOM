# Threat Model

This document identifies security assets, trust boundaries, attack surfaces, and controls for BazBOM.

## Purpose

This threat model helps:
- Identify potential security risks
- Evaluate the impact of threats
- Implement appropriate controls
- Guide security reviews and testing

## Assets

### Primary Assets

1. **Source Code**
   - Value: Intellectual property, business logic
   - Confidentiality: High
   - Integrity: Critical
   - Availability: Medium

2. **SBOM Documents**
   - Value: Dependency inventory, license info
   - Confidentiality: Low (typically public)
   - Integrity: Critical
   - Availability: Medium

3. **Vulnerability Reports**
   - Value: Security findings, risk assessment
   - Confidentiality: Medium (may reveal attack vectors)
   - Integrity: Critical
   - Availability: Medium

4. **Build Artifacts**
   - Value: Compiled outputs, dependencies
   - Confidentiality: Low
   - Integrity: Critical
   - Availability: High

### Supporting Assets

5. **Build Environment**
   - Bazel workspace
   - Local cache
   - CI/CD runners

6. **Dependencies**
   - Maven/npm packages
   - Python libraries
   - Bazel rules

7. **Credentials**
   - GitHub tokens
   - OSV API keys (if any)
   - Code signing keys

## Trust Boundaries

### 1. Build Environment → External Networks

**Boundary**: Network requests from build tools

**Trust Level**: Low

**Threats**:
- Malicious dependency injection
- Man-in-the-middle attacks
- DNS poisoning

**Controls**:
- HTTPS-only connections
- Checksum verification (SHA-256)
- Pinned dependency versions
- Lockfiles (maven_install.json)

### 2. CI/CD → GitHub

**Boundary**: GitHub Actions to GitHub API

**Trust Level**: Medium

**Threats**:
- Credential theft
- Unauthorized code modifications
- SARIF injection attacks

**Controls**:
- Minimal GITHUB_TOKEN permissions
- Read-only tokens where possible
- Branch protection rules
- Required status checks

### 3. Build Tools → OSV Database

**Boundary**: SCA queries to OSV API

**Trust Level**: Medium

**Threats**:
- API abuse
- Response tampering
- Privacy leakage (dependency fingerprinting)

**Controls**:
- HTTPS-only connections
- Rate limiting
- Result validation
- Local caching

### 4. Developer → Build Environment

**Boundary**: Developer machine to Bazel

**Trust Level**: High (but verify)

**Threats**:
- Malicious code injection
- Local privilege escalation
- Cache poisoning

**Controls**:
- Code review requirements
- Sandboxed builds
- Hermetic builds
- Signed commits (recommended)

## Attack Surfaces

### 1. Dependency Resolution

**Surface**: Maven/npm dependency downloads

**Attack Vectors**:
- Typosquatting (wrong package name)
- Dependency confusion
- Compromised package repositories
- Version rollback attacks

**Mitigations**:
```python
# In WORKSPACE - pin exact versions and checksums
maven_install(
    artifacts = [
        "com.google.guava:guava:31.1-jre",  # Exact version
    ],
    repositories = [
        "https://repo1.maven.org/maven2",  # Trusted repository
    ],
    maven_install_json = "@//:maven_install.json",  # Lockfile
    fail_on_missing_checksum = True,  # Require checksums
)
```

### 2. Aspect Execution

**Surface**: Bazel aspects traversing build graph

**Attack Vectors**:
- Malicious BUILD files
- Aspect code injection
- Information disclosure via aspects

**Mitigations**:
- Aspects run in Bazel sandbox
- No network access from aspects
- Aspects are reviewed code (not user input)
- Minimal privileges

### 3. SBOM Generation

**Surface**: Python scripts processing build data

**Attack Vectors**:
- Code injection in package names
- Path traversal in output paths
- XML/JSON injection in SPDX documents

**Mitigations**:
```python
# In write_sbom.py
import re
from pathlib import Path

# Sanitize package names
def sanitize_spdx_id(name):
    # Only allow safe characters
    return re.sub(r'[^A-Za-z0-9.-]', '-', name)

# Validate output paths
def safe_output_path(path):
    resolved = Path(path).resolve()
    # Ensure within workspace
    if not str(resolved).startswith(str(workspace_root)):
        raise ValueError("Path outside workspace")
    return resolved
```

### 4. OSV Queries

**Surface**: HTTP requests to OSV API

**Attack Vectors**:
- Response injection
- Malicious SARIF generation
- False positive/negative manipulation

**Mitigations**:
```python
# In osv_query.py
import requests
from jsonschema import validate

# Validate OSV response
OSV_RESPONSE_SCHEMA = { ... }

def query_osv(package, version):
    response = requests.post(
        "https://api.osv.dev/v1/query",
        json={"package": {"name": package}, "version": version},
        timeout=30,  # Prevent hanging
        verify=True,  # Verify SSL
    )
    response.raise_for_status()
    
    data = response.json()
    validate(instance=data, schema=OSV_RESPONSE_SCHEMA)  # Validate schema
    return data
```

### 5. SARIF Upload

**Surface**: GitHub Code Scanning upload

**Attack Vectors**:
- Malicious SARIF injection
- Information disclosure
- Privilege escalation via alerts

**Mitigations**:
- SARIF schema validation before upload
- Minimal workflow permissions
- Review alerts before taking action
- Rate limiting on uploads

### 6. CI/CD Pipeline

**Surface**: GitHub Actions workflows

**Attack Vectors**:
- Workflow injection via PR titles/descriptions
- Secret exfiltration
- Malicious workflow modifications

**Mitigations**:
```yaml
# .github/workflows/supplychain.yml
name: Supply Chain Security

permissions:
  contents: read        # Minimal read access
  security-events: write  # Only what's needed

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          persist-credentials: false  # Don't persist token
      
      # Don't use user input in commands
      - name: Run SCA
        run: bazel run //:sca_from_sbom  # No ${{ }} interpolation
```

## Threat Scenarios

### Scenario 1: Malicious Dependency Injection

**Attacker Goal**: Inject malicious code via compromised dependency

**Attack Path**:
1. Compromise upstream package repository
2. Replace legitimate package with malicious version
3. Wait for BazBOM to download and include in SBOM
4. Malicious code executes during build

**Likelihood**: Low (requires compromising Maven Central or similar)

**Impact**: Critical (arbitrary code execution)

**Controls**:
- Pinned versions in WORKSPACE
- Checksum verification
- Lockfile (maven_install.json)
- Dependency review in PRs

### Scenario 2: False Vulnerability Reports

**Attacker Goal**: Generate false positive vulnerabilities to cause panic or distrust

**Attack Path**:
1. Man-in-the-middle OSV API responses
2. Inject fake CVE data
3. Generate SARIF with false vulnerabilities
4. Upload to GitHub Code Scanning

**Likelihood**: Very Low (requires MITM on HTTPS)

**Impact**: Medium (noise, wasted time)

**Controls**:
- HTTPS certificate validation
- OSV response schema validation
- Manual review of high-severity alerts
- Multiple vulnerability sources (OSV, NVD, GHSA, KEV, EPSS)

### Scenario 3: SBOM Tampering

**Attacker Goal**: Modify SBOM to hide vulnerable dependencies

**Attack Path**:
1. Modify write_sbom.py to exclude certain packages
2. Submit PR with changes
3. SBOM appears clean but vulnerabilities hidden

**Likelihood**: Low (requires code review bypass)

**Impact**: High (undetected vulnerabilities)

**Controls**:
- Required code review
- CODEOWNERS for critical paths
- Automated SBOM validation
- Comparison with external SBOM tools

### Scenario 4: CI/CD Compromise

**Attacker Goal**: Steal secrets or inject malicious code via CI

**Attack Path**:
1. Submit PR with malicious workflow changes
2. Workflow runs with elevated permissions
3. Exfiltrate GITHUB_TOKEN or inject code

**Likelihood**: Low (requires PR approval)

**Impact**: Critical (full repository access)

**Controls**:
- Branch protection on workflow files
- Minimal workflow permissions
- No secrets in workflows
- Review workflow changes carefully

### Scenario 5: Supply Chain Attack via Build Tools

**Attacker Goal**: Compromise build process via malicious Bazel rule

**Attack Path**:
1. Introduce malicious Bazel rule dependency
2. Rule executes during build
3. Modifies outputs or steals data

**Likelihood**: Very Low (rules_jvm_external is well-maintained)

**Impact**: Critical (build compromise)

**Controls**:
- Pin Bazel rule versions
- Verify checksums of rule downloads
- Review rule source code
- Use only trusted rule repositories

## Security Controls

### Preventive Controls

1. **Dependency Pinning**: All dependencies pinned to exact versions
2. **Checksum Verification**: SHA-256 checksums verified for all downloads
3. **Hermetic Builds**: Bazel sandboxing prevents external access
4. **Code Review**: All changes require maintainer approval
5. **Branch Protection**: Main branch requires status checks and reviews
6. **Minimal Permissions**: CI workflows use least-privilege tokens

### Detective Controls

1. **SBOM Generation**: Automatic inventory of all dependencies
2. **Vulnerability Scanning**: OSV queries for known CVEs
3. **Code Scanning**: GitHub Code Scanning displays security alerts
4. **Audit Logs**: GitHub audit logs track all repository actions
5. **Dependency Review**: GitHub Dependency Review on PRs
6. **Validation**: Automated SPDX/SARIF schema validation

### Corrective Controls

1. **Incident Response**: Security policy defines reporting process
2. **Patch Process**: Fast-track security updates
3. **Rollback**: Git revert for malicious changes
4. **Alert Dismissal**: Process for handling false positives

## Compliance & Standards

BazBOM aligns with:

- **NIST SSDF**: Secure Software Development Framework
- **SLSA**: Supply Chain Levels for Software Artifacts (Level 2+)
- **NTIA SBOM**: Minimum elements for SBOM
- **EO 14028**: U.S. Executive Order on Cybersecurity
- **OWASP Top 10**: Software supply chain security

## Review Schedule

This threat model should be reviewed:

- **Quarterly**: Regular review for new threats
- **On significant changes**: Architecture changes, new features
- **After incidents**: Post-mortem analysis
- **Before releases**: Pre-release security review

## Contributing to This Model

To update this threat model:

1. Identify new assets, boundaries, or attack surfaces
2. Assess likelihood and impact
3. Propose appropriate controls
4. Submit PR for review by maintainers
5. Update after security incidents or audits

## References

- [OWASP Threat Modeling](https://owasp.org/www-community/Threat_Modeling)
- [STRIDE Methodology](https://en.wikipedia.org/wiki/STRIDE_(security))
- [SLSA Framework](https://slsa.dev/)
- [NIST SSDF](https://csrc.nist.gov/Projects/ssdf)
