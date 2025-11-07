# Threat Detection & Supply Chain Security

**Status:**  Available in BazBOM v0.5.1+  
**Phase:** 7 - Threat Intelligence (95% Complete)

---

## Overview

BazBOM includes advanced threat detection capabilities to identify supply chain attacks, malicious packages, and suspicious behavior before they compromise your application.

### Features

- **Malicious Package Detection** - Identify packages with suspicious behavior
- **Typosquatting Detection** - Find packages with names similar to popular libraries
- **Dependency Confusion** - Detect potential internal namespace hijacking
- **Supply Chain Attack Indicators** - Identify compromised dependencies
- **Threat Intelligence Feeds** - Integration with OSV, GHSA, and curated threat databases
- **Team Notifications** - Alert your team via Slack, email, Teams, or GitHub Issues

---

## Quick Start

### Basic Threat Detection

```bash
# Run scan with standard threat detection
bazbom scan --threat-detection standard

# Run scan with aggressive threat detection (more thorough, slower)
bazbom scan --threat-detection aggressive

# Disable threat detection
bazbom scan --threat-detection off
```

### Threat Detection Levels

| Level | Description | Performance Impact |
|-------|-------------|-------------------|
| `off` | No threat detection | None |
| `standard` | Balanced detection (default) | Low |
| `aggressive` | Maximum detection, more false positives | Medium |

---

## Detection Capabilities

### 1. Typosquatting Detection

Identifies packages with names suspiciously similar to popular libraries:

```bash
# Example detections:
 commons-io-typo  → Similar to: commons-io (popular library)
 log4jj          → Similar to: log4j (popular library)
 springboot      → Similar to: spring-boot (popular library)
```

**Algorithm:**
- Levenshtein distance calculation
- Edit distance threshold: ≤ 2 for packages with >1000 downloads
- Common character substitutions (0→o, 1→l, etc.)

### 2. Dependency Confusion

Detects potential namespace hijacking:

```bash
# Example:
  Package: com.internal.mycompany:common-utils:1.0.0
    Found public package with same name: com.internal.mycompany:common-utils
    Risk: Internal namespace may be hijacked by public registry
```

### 3. Malicious Package Indicators

Identifies packages with suspicious characteristics:

```bash
# Indicators detected:
- Obfuscated code (high ratio of single-letter variables)
- Unusual network calls (external HTTP/HTTPS during build)
- Cryptocurrency mining patterns
- Code execution in static initializers
- Excessive use of reflection
- Native library loading from unusual sources
```

### 4. Supply Chain Attack Indicators

Monitors for signs of compromised packages:

```bash
# Examples:
  Package: popular-library:1.2.3
    - Sudden version jump (1.2.2 → 1.2.3 released within 1 hour)
    - Maintainer change detected
    - Binary size increased by >300%
    - New network dependencies added
```

---

## Threat Intelligence Integration

### OSV (Open Source Vulnerabilities)

BazBOM automatically queries OSV database for known malicious packages:

```bash
# OSV integration is automatic during scan
bazbom scan --threat-detection standard

# Output includes OSV malicious package alerts:
[bazbom] checking OSV database for malicious packages...
[bazbom] found 0 known malicious packages
```

### GHSA (GitHub Security Advisory)

Integrates with GitHub's security advisory database:

```bash
# GHSA integration queries GitHub's GraphQL API
# Requires GITHUB_TOKEN for authenticated requests (higher rate limit)

export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
bazbom scan --threat-detection aggressive
```

### Curated Threat Database

BazBOM maintains a curated database of known threats:

```bash
# Database includes:
- Known malicious packages from security researchers
- Typosquatting campaigns
- Compromised maintainer accounts
- Historical supply chain attacks

# Database synced with:
bazbom db sync
```

---

## Team Notifications

### Configure Notifications

Alert your security team when threats are detected:

```bash
# Slack webhook
bazbom team-config --slack-webhook https://hooks.slack.com/services/XXX/YYY/ZZZ

# Email (SMTP)
bazbom team-config --email smtp://mail.company.com:587

# Microsoft Teams webhook
bazbom team-config --teams-webhook https://outlook.office.com/webhook/XXX

# GitHub Issues (auto-create)
bazbom team-config --github-issues --github-token ghp_xxxx --github-repo owner/repo
```

### Notification Format

**Slack/Teams Example:**
```
 BazBOM Threat Detected

Severity: HIGH
Package: suspicious-package:1.0.0
Threat: Typosquatting (similar to: popular-package)

Details:
- Edit distance: 2
- Downloads: 10 (suspicious for popular name)
- First seen: 2024-11-04

Action Required: Review and investigate this dependency

Project: myapp
Scan Date: 2024-11-04 19:00:00 UTC
```

**GitHub Issue Example:**
```markdown
##  Threat Detection Alert

**Severity:** HIGH  
**Package:** suspicious-package:1.0.0  
**Threat Type:** Typosquatting

### Details

This package has a name suspiciously similar to `popular-package`, which has over 1 million downloads per month.

**Indicators:**
- Edit distance: 2
- Package age: 1 day
- Downloads: 10
- Maintainer: new-account-2024

### Recommended Actions

1. [ ] Review package source code
2. [ ] Check if this is an intentional dependency
3. [ ] If unintentional, remove from project
4. [ ] Report to Maven Central if confirmed malicious

### Project Information

- **Project:** myapp
- **Scan Date:** 2024-11-04 19:00:00 UTC
- **BazBOM Version:** 0.5.1

---
*This issue was automatically created by BazBOM*
```

---

## Configuration

### Policy-Based Threat Detection

Configure threat detection in `bazbom.yml`:

```yaml
threat_detection:
  enabled: true
  level: standard  # off | standard | aggressive
  
  # Typosquatting detection
  typosquatting:
    enabled: true
    edit_distance_threshold: 2
    check_popular_packages: true
    
  # Dependency confusion
  dependency_confusion:
    enabled: true
    internal_namespaces:
      - com.mycompany
      - com.internal
      
  # Malicious package detection
  malicious_detection:
    enabled: true
    check_osv: true
    check_ghsa: true
    check_curated_db: true
    
  # Notifications
  notifications:
    slack_webhook: ${SLACK_WEBHOOK_URL}
    email_smtp: ${SMTP_SERVER}
    teams_webhook: ${TEAMS_WEBHOOK_URL}
    github_token: ${GITHUB_TOKEN}
    github_repo: ${GITHUB_REPOSITORY}
    
    # Notification filters
    min_severity: MEDIUM  # Only notify for MEDIUM and above
    enabled_channels:
      - slack
      - github_issues
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: BazBOM Threat Detection

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  threat-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run BazBOM with Threat Detection
        run: |
          bazbom scan --threat-detection aggressive
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
      
      - name: Upload Findings
        uses: actions/upload-artifact@v4
        with:
          name: threat-findings
          path: .bazbom/findings/
```

### GitLab CI

```yaml
bazbom:threat-scan:
  stage: security
  script:
    - bazbom scan --threat-detection aggressive
  artifacts:
    paths:
      - .bazbom/findings/
  allow_failure: false  # Fail build on threats
```

---

## Understanding Threat Reports

### Threat Finding Format

```json
{
  "threat_id": "THREAT-2024-001234",
  "severity": "HIGH",
  "threat_type": "typosquatting",
  "package": {
    "name": "commons-io-typo",
    "version": "1.0.0",
    "purl": "pkg:maven/org.apache.commons/commons-io-typo@1.0.0"
  },
  "indicators": {
    "similar_to": "commons-io",
    "edit_distance": 2,
    "package_age_days": 1,
    "download_count": 10,
    "maintainer_reputation": "unknown"
  },
  "recommendation": "Remove this dependency immediately",
  "references": [
    "https://ossf.org/typosquatting-guide",
    "https://security.snyk.io/typosquatting"
  ],
  "detected_at": "2024-11-04T19:00:00Z",
  "detection_method": "typosquatting_analysis"
}
```

### Severity Levels

| Severity | Description | Action |
|----------|-------------|--------|
| **CRITICAL** | Confirmed malicious package | Block immediately, remove from all systems |
| **HIGH** | Strong indicators of malicious intent | Investigate urgently, consider blocking |
| **MEDIUM** | Suspicious behavior detected | Review carefully, monitor |
| **LOW** | Minor concerns, likely false positive | Review when convenient |

---

## Best Practices

### 1. Enable Threat Detection in CI/CD

Always run threat detection in your CI/CD pipeline:

```bash
# Fail build on HIGH or CRITICAL threats
bazbom scan --threat-detection aggressive --fail-on-threat high
```

### 2. Configure Team Notifications

Ensure your security team is notified immediately:

```bash
# Setup notifications
bazbom team-config \
  --slack-webhook https://hooks.slack.com/... \
  --min-severity medium
```

### 3. Review Threat Reports Regularly

```bash
# Weekly security review
bazbom report threat-summary --last-7-days

# Monthly trend analysis
bazbom report threat-trends --last-30-days
```

### 4. Maintain Internal Namespace List

Configure your internal namespaces to detect dependency confusion:

```yaml
# bazbom.yml
threat_detection:
  dependency_confusion:
    internal_namespaces:
      - com.yourcompany
      - com.internal
      - net.yourorg
```

### 5. Keep Threat Database Updated

```bash
# Sync threat intelligence database weekly
bazbom db sync
```

---

## Troubleshooting

### High False Positive Rate

If you're seeing too many false positives:

```bash
# Use standard detection level instead of aggressive
bazbom scan --threat-detection standard

# Or adjust sensitivity in bazbom.yml
threat_detection:
  typosquatting:
    edit_distance_threshold: 1  # Stricter (fewer false positives)
```

### OSV/GHSA API Rate Limiting

If you hit API rate limits:

```bash
# Set GITHUB_TOKEN for higher rate limits
export GITHUB_TOKEN=ghp_xxxxxxxxxxxx
bazbom scan --threat-detection aggressive

# Or use offline mode with cached database
export BAZBOM_OFFLINE=1
bazbom scan --threat-detection standard
```

### Notifications Not Working

Check your webhook configuration:

```bash
# Test Slack webhook
curl -X POST -H 'Content-type: application/json' \
  --data '{"text":"Test from BazBOM"}' \
  https://hooks.slack.com/services/XXX/YYY/ZZZ

# Check bazbom logs
bazbom scan --threat-detection standard --verbose
```

---

## Performance Impact

### Detection Level Performance

| Level | Scan Time Overhead | Memory Overhead |
|-------|-------------------|-----------------|
| Off | 0% | 0 MB |
| Standard | +10-15% | +50 MB |
| Aggressive | +25-30% | +100 MB |

### Optimization Tips

```bash
# For faster builds, use standard detection
bazbom scan --threat-detection standard

# Use cache to speed up repeated scans
# (threat checks are cached for 1 hour by default)

# For CI/CD, use aggressive only on main branch
if [ "$BRANCH" = "main" ]; then
  bazbom scan --threat-detection aggressive
else
  bazbom scan --threat-detection standard
fi
```

---

## Related Documentation

- [ML Features](../reference/ml-features.md) - Machine learning capabilities
- [Threat Model](threat-model.md) - Security threat model
- [Vulnerability Enrichment](vulnerability-enrichment.md) - Advisory database details
- [Supply Chain Security](supply-chain.md) - Supply chain security overview

---

## Contributing

Help improve threat detection:

1. **Report False Positives** - Open an issue with package details
2. **Submit Threat Indicators** - Share new detection patterns
3. **Contribute to Curated Database** - Submit known malicious packages
4. **Improve Detection Algorithms** - Submit PRs with better heuristics

See [CONTRIBUTING.md](../CONTRIBUTING.md) for more information.

---

**Last Updated:** 2024-11-04  
**Version:** 0.5.1  
**Status:** Production Ready (Phase 7: 95% Complete)
