# Usage Guide

> **Note:** This file has been consolidated. For the complete usage guide, see [user-guide/usage.md](user-guide/usage.md).

## Quick Links

- **[Complete Usage Guide](user-guide/usage.md)** - Comprehensive command reference and workflows
- **[Quick Reference](QUICKREF.md)** - One-page cheat sheet for common operations
- **[Troubleshooting](TROUBLESHOOTING.md)** - Top 20 failures with exact error text and fixes

## Quick Start

```bash
# Scan any project (auto-detects build system)
bazbom scan .

# Quick scan with short flags
bazbom scan -r -s -f spdx -o ./reports

# Use named profile
bazbom scan -p strict

# JSON output for CI/CD
bazbom scan --json | jq '.vulnerabilities[] | select(.severity == "CRITICAL")'
```

For complete documentation, examples, and advanced usage, see the [full usage guide](user-guide/usage.md).
