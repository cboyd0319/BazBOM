# Advanced Security Features

BazBOM includes advanced security features for threat intelligence, anomaly detection, notifications, authentication, and IDE integration.

## Threat Intelligence

Detect supply chain threats including typosquatting, dependency confusion, and maintainer takeover risks.

### Scan for Threats

```bash
# Full threat scan
bazbom threats scan

# Specific threat types
bazbom threats scan --typosquatting
bazbom threats scan --dep-confusion
bazbom threats scan --maintainer-takeover
bazbom threats scan --scorecard

# JSON output for CI/CD
bazbom threats scan --json
```

### Configure Threat Detection

```bash
bazbom threats configure
```

### Threat Types

| Threat | Description |
|--------|-------------|
| Typosquatting | Detects packages with similar names to popular packages |
| Dependency Confusion | Identifies internal package names that conflict with public registries |
| Maintainer Takeover | Flags packages with recent maintainer changes |
| Scorecard | Uses OpenSSF Scorecard to assess project health |

## ML-Based Anomaly Detection

Detect unusual patterns in dependencies using machine learning.

### Scan for Anomalies

```bash
# Run anomaly detection
bazbom anomaly scan

# JSON output
bazbom anomaly scan --json

# Save to file
bazbom anomaly scan -o anomalies.json
```

### Train Custom Model

Train a model on your project's historical data:

```bash
# Train from historical scan data
bazbom anomaly train --from-dir ./historical-scans

# Specify output model location
bazbom anomaly train --from-dir ./data -o ./models/custom-model.json
```

### Generate Reports

```bash
# Generate anomaly report
bazbom anomaly report

# Save to file
bazbom anomaly report -o anomaly-report.html
```

## Notifications

Configure alerts for security findings.

### Supported Channels

- **Slack**: Webhook integration
- **Microsoft Teams**: Webhook integration
- **Email**: SMTP configuration
- **GitHub Issues**: Automatic issue creation

### Configure Notifications

```bash
# Configure Slack
bazbom notify configure --channel slack --webhook https://hooks.slack.com/...

# Configure Teams
bazbom notify configure --channel teams --webhook https://outlook.office.com/webhook/...

# Configure Email
bazbom notify configure --channel email --smtp smtp.example.com --port 587

# Configure GitHub Issues
bazbom notify configure --channel github --repo owner/repo --token ghp_...
```

### Test Notifications

```bash
# Send test notification
bazbom notify test --channel slack
```

### View History

```bash
# View notification history
bazbom notify history

# Limit results
bazbom notify history --limit 10
```

## Authentication & RBAC

Manage users, API tokens, and access control.

### Initialize Auth System

```bash
bazbom auth init
```

This creates:
- `~/.config/bazbom/auth/config.json` - Configuration
- `~/.config/bazbom/auth/users.json` - User database
- `~/.config/bazbom/auth/tokens.json` - API tokens
- `~/.config/bazbom/auth/audit.json` - Audit log

### User Management

```bash
# Add user with role
bazbom auth user add admin@example.com --role admin
bazbom auth user add dev@example.com --role developer
bazbom auth user add viewer@example.com --role viewer

# List users
bazbom auth user list

# Change role
bazbom auth user set-role dev@example.com --role admin

# Remove user
bazbom auth user remove user@example.com
```

### Available Roles

| Role | Permissions |
|------|-------------|
| admin | Full access, user management |
| developer | Read/write scans, apply fixes |
| viewer | Read-only access |

### API Token Management

```bash
# Create token for CI
bazbom auth token create --name ci-token --scope read

# Create admin token
bazbom auth token create --name admin-token --scope admin --expires 90

# List tokens
bazbom auth token list

# Revoke token
bazbom auth token revoke tk_abc123
```

### Token Scopes

| Scope | Permissions |
|-------|-------------|
| read | Read SBOMs, findings |
| write | Create scans, apply fixes |
| admin | Full access |

### Audit Log

```bash
# View recent events
bazbom auth audit-log

# Limit to specific events
bazbom auth audit-log --limit 50 --event-type auth_failure
```

## LSP Server (IDE Integration)

Get real-time vulnerability scanning in your IDE.

### Installation

```bash
# Install LSP server
cargo install --path crates/bazbom-lsp
# OR
cargo install bazbom-lsp
```

### VS Code Setup

1. Install the 'Custom Language Server' extension
2. Add to settings.json:
```json
{
  "customLanguageServerExtension.commands": [{
    "id": "bazbom",
    "name": "BazBOM Security",
    "command": "bazbom-lsp",
    "languages": ["xml", "groovy", "kotlin", "starlark"]
  }]
}
```

### IntelliJ Setup

1. Install the 'LSP4IJ' plugin
2. Go to Settings > Languages > LSP
3. Add server: `bazbom-lsp`
4. Associate with: `*.xml`, `*.gradle`, `BUILD*`

### Neovim Setup

```lua
require('lspconfig').bazbom.setup{
  cmd = { 'bazbom-lsp' },
  filetypes = { 'xml', 'groovy', 'kotlin', 'bzl' },
}
```

### Features

- Real-time vulnerability diagnostics in build files
- Quick fixes to upgrade vulnerable dependencies
- Hover information with CVE details
- Supports: `pom.xml`, `build.gradle`, `build.gradle.kts`, `BUILD`, `BUILD.bazel`

## Integration Examples

### CI/CD Pipeline with Threat Detection

```yaml
# GitHub Actions
- name: Security Scan
  run: |
    bazbom scan . --out-dir ./results
    bazbom threats scan --json > threats.json
    bazbom anomaly scan --json > anomalies.json

- name: Notify on Findings
  if: failure()
  run: bazbom notify test --channel slack
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

bazbom threats scan --json | jq -e '.threats | length == 0' || {
  echo "Threats detected! Commit blocked."
  exit 1
}
```

### API Authentication

```bash
# Set token in environment
export BAZBOM_TOKEN=bzb_live_abc123

# Use in requests
curl -H "Authorization: Bearer $BAZBOM_TOKEN" \
  http://localhost:8080/api/scan
```

## Cross-Ecosystem Support

All advanced features work across supported ecosystems:

| Feature | Java | JavaScript | Python | Go | Rust | Ruby | PHP |
|---------|------|------------|--------|-----|------|------|-----|
| Threats | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Anomaly | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Notify | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| Auth | Yes | Yes | Yes | Yes | Yes | Yes | Yes |
| LSP | Yes | Yes | Planned | Planned | Planned | Planned | Planned |
