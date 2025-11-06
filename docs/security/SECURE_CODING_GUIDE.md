# BazBOM Secure Coding Guide

This guide provides security best practices for contributors to BazBOM, following PYSEC_OMEGA standards.

## General Principles

1. **Never trust user input** - Validate and sanitize all external data
2. **Fail securely** - Default deny, explicit allow
3. **Defense in depth** - Multiple layers of security
4. **Least privilege** - Minimal permissions required
5. **Secure defaults** - Safe by default, opt-in to risky behavior

## Python Security Patterns

### Input Validation

```python
#  GOOD: Validate all inputs
def process_package_name(name: str) -> bool:
    if not isinstance(name, str):
        raise TypeError("Package name must be a string")
    
    # Whitelist allowed characters
    if not re.match(r'^[a-zA-Z0-9._-]+$', name):
        raise ValueError(f"Invalid package name: {name}")
    
    # Prevent path traversal
    if '..' in name or '/' in name:
        raise ValueError("Package name cannot contain path separators")
    
    return True

#  BAD: No validation
def process_package_name(name):
    # Directly use untrusted input
    return name
```

### XML Parsing

```python
#  GOOD: Use defusedxml
from defusedxml import ElementTree as ET

def parse_pom(path: str):
    tree = ET.parse(path)  # Protected against XXE
    return tree

#  BAD: Vulnerable to XXE
import xml.etree.ElementTree as ET

def parse_pom(path: str):
    tree = ET.parse(path)  # VULNERABLE!
    return tree
```

### Subprocess Execution

```python
#  GOOD: Use list arguments, no shell=True
import subprocess
import shlex

def run_bazel_build(target: str):
    # Validate target first
    if not re.match(r'^//[a-zA-Z0-9/_:-]+$', target):
        raise ValueError(f"Invalid Bazel target: {target}")
    
    # Use list arguments
    result = subprocess.run(
        ['bazel', 'build', target],
        check=True,
        capture_output=True,
        timeout=300,
        text=True
    )
    return result.stdout

#  BAD: Shell injection vulnerability
def run_bazel_build(target: str):
    # NEVER do this!
    cmd = f"bazel build {target}"  # Shell injection!
    subprocess.run(cmd, shell=True)  # VULNERABLE!
```

### File Operations

```python
#  GOOD: Safe file operations
from pathlib import Path

def read_config(config_path: str) -> dict:
    # Validate path is within allowed directory
    config_path = Path(config_path).resolve()
    base_dir = Path('/allowed/config/dir').resolve()
    
    if not config_path.is_relative_to(base_dir):
        raise ValueError("Config path outside allowed directory")
    
    with open(config_path, 'r', encoding='utf-8') as f:
        return json.load(f)

#  BAD: Path traversal vulnerability
def read_config(config_path: str) -> dict:
    # No validation - path traversal!
    with open(config_path, 'r') as f:
        return json.load(f)
```

### URL Handling

```python
#  GOOD: Validate URL schemes
from urllib.parse import urlparse

def fetch_url(url: str) -> bytes:
    parsed = urlparse(url)
    
    # Whitelist allowed schemes
    if parsed.scheme not in ('http', 'https'):
        raise ValueError(f"Invalid URL scheme: {parsed.scheme}")
    
    # Prevent SSRF to internal networks
    if parsed.hostname in ('localhost', '127.0.0.1', '0.0.0.0'):
        raise ValueError("Cannot access localhost")
    
    with urllib.request.urlopen(url, timeout=5) as response:
        return response.read()

#  BAD: SSRF vulnerability
def fetch_url(url: str) -> bytes:
    # Allows file:/, ftp:/, any scheme!
    with urllib.request.urlopen(url) as response:
        return response.read()
```

### Secrets Management

```python
#  GOOD: Never log secrets
import logging

def authenticate(token: str) -> bool:
    logger = logging.getLogger(__name__)
    logger.info("Attempting authentication")  # No token logged
    
    # Use token securely
    result = verify_token(token)
    
    if result:
        logger.info("Authentication successful")
    else:
        logger.warning("Authentication failed")  # No token in error
    
    return result

#  BAD: Secrets in logs
def authenticate(token: str) -> bool:
    logger = logging.getLogger(__name__)
    logger.info(f"Authenticating with token: {token}")  # LEAKED!
    return verify_token(token)
```

### SQL/NoSQL Queries

```python
#  GOOD: Parameterized queries
import sqlite3

def get_package_info(package_name: str) -> dict:
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    # Use parameterized query
    cursor.execute(
        "SELECT * FROM packages WHERE name = ?",
        (package_name,)
    )
    
    return cursor.fetchone()

#  BAD: SQL injection
def get_package_info(package_name: str) -> dict:
    conn = sqlite3.connect('packages.db')
    cursor = conn.cursor()
    
    # String concatenation - SQL injection!
    query = f"SELECT * FROM packages WHERE name = '{package_name}'"
    cursor.execute(query)  # VULNERABLE!
    
    return cursor.fetchone()
```

## GitHub Actions Security

### Action Pinning

```yaml
#  GOOD: Pin to full SHA with version comment
- name: Checkout code
  uses: actions/checkout@08c6903cd8c0fde910a37f88322edcfb5dd907a8 # v5.0.0

#  BAD: Using version tags (mutable)
- name: Checkout code
  uses: actions/checkout@v5
```

### Workflow Permissions

```yaml
#  GOOD: Minimal permissions
permissions:
  contents: read

jobs:
  build:
    permissions:
      contents: read
      packages: write  # Only what's needed

#  BAD: Excessive permissions
permissions:
  contents: write
  packages: write
  security-events: write
```

### Preventing Workflow Injection

```yaml
#  GOOD: Use environment variables
- name: Process issue title
  env:
    ISSUE_TITLE: ${{ github.event.issue.title }}
  run: echo "Title: $ISSUE_TITLE"

#  BAD: Direct interpolation
- name: Process issue title
  run: echo "Title: ${{ github.event.issue.title }}"
```

## Testing Security

### Test Isolation

```python
#  GOOD: Clean state for each test
import pytest

@pytest.fixture(autouse=True)
def reset_state():
    # Setup
    yield
    # Teardown - clean up state
    
def test_something():
    # Test runs with clean state
    pass
```

### Coverage for Security Code

```python
#  GOOD: Test security validations
def test_input_validation_rejects_path_traversal():
    with pytest.raises(ValueError, match="path separator"):
        process_package_name("../etc/passwd")

def test_url_validation_rejects_file_scheme():
    with pytest.raises(ValueError, match="Invalid URL scheme"):
        fetch_url("file:///etc/passwd")
```

## Pre-commit Checklist

Before committing code, ensure:

-  No secrets or credentials in code
-  All inputs are validated
-  No shell=True in subprocess calls
-  No string concatenation in SQL/commands
-  Secrets not logged
-  Tests cover security validations
-  Bandit scan passes
-  Pre-commit hooks pass

## Tools

### Required Tools

- **Bandit** - Python security scanner
- **Semgrep** - Static analysis
- **truffleHog** - Secret detection
- **gitleaks** - Secret scanning
- **pre-commit** - Hook framework

### Running Security Scans

```bash
# Run all security scans
bandit -r tools/supplychain -c .bandit
semgrep --config auto tools/supplychain
trufflehog filesystem . --only-verified
gitleaks detect --source . --verbose

# Pre-commit hooks
pre-commit install
pre-commit run --all-files
```

## Resources

- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [PYSEC_OMEGA Guide](../docs/strategy/product-roadmap/PYSEC.md)
- [Python Security Best Practices](https://python.readthedocs.io/en/stable/library/security_warnings.html)

## Reporting Security Issues

See [SECURITY.md](../SECURITY.md) for how to report security vulnerabilities.

---

**Last Updated:** 2025-10-19
**Maintained By:** Security Team
