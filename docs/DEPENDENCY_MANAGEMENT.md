# Dependency Management Guide

This document describes how BazBOM manages Python dependencies following PYSEC_OMEGA security standards.

## Overview

BazBOM uses **pip-tools** with hash-based verification to ensure:
- ✅ **Reproducible builds** - Same dependencies every time
- ✅ **Supply chain security** - SHA256 hash verification prevents tampering
- ✅ **Transitive dependency tracking** - All dependencies explicitly listed
- ✅ **Security auditing** - Easy to scan for vulnerabilities

## Dependency Files

### Source Requirements (.in files)

These define your **desired** dependencies with version constraints:

- `requirements.in` - Core runtime dependencies
- `requirements-test.in` - Testing dependencies
- `requirements-security.in` - Security tooling

**Example `requirements.in`:**
```text
# Core runtime dependencies
requests>=2.32.0
jsonschema>=4.23.0
defusedxml>=0.7.1
```

### Locked Requirements (.txt files)

These contain **pinned** versions with SHA256 hashes:

- `requirements.txt` - Locked runtime dependencies
- `requirements-test.txt` - Locked testing dependencies
- `requirements-security.txt` - Locked security tools

**Example `requirements.txt`:**
```text
requests==2.32.3 \
    --hash=sha256:55365417734eb18255590a9ff9eb97e9e1da868d4ccd6402399eaf68af20a760 \
    --hash=sha256:70761cfe03c773ceb22aa2f671b4757976145175cdfca038c02654d061d6dcc6
certifi==2025.10.5 \
    --hash=sha256:0f212c2744a9bb6de0c56639a6f68afe01ecd92d91f14ae897c4fe7bbeeef0de \
    --hash=sha256:47c09d31ccf2acf0be3f701ea53595ee7e0b8fa08801c6624be771df09ae7b43
```

## Installation

### Install pip-tools

```bash
pip install pip-tools
```

### Install dependencies

```bash
# Install runtime dependencies with hash verification
pip install -r requirements.txt --require-hashes

# Install test dependencies
pip install -r requirements-test.txt --require-hashes

# Install security tools
pip install -r requirements-security.txt --require-hashes
```

## Adding Dependencies

### Step 1: Update .in file

Edit the appropriate `.in` file:

```bash
# Add to requirements.in
echo "requests>=2.32.0" >> requirements.in
```

### Step 2: Compile with hashes

```bash
# Generate locked requirements with SHA256 hashes
pip-compile --generate-hashes requirements.in
```

This creates/updates `requirements.txt` with:
- Exact versions
- SHA256 hashes for all packages
- Transitive dependencies resolved

### Step 3: Sync environment

```bash
# Install the newly locked dependencies
pip-sync requirements.txt
```

### Step 4: Test

```bash
# Run tests to ensure compatibility
pytest

# Run security scans
bandit -r tools/supplychain
pip-audit -r requirements.txt
```

### Step 5: Commit

```bash
git add requirements.in requirements.txt
git commit -m "deps: Add requests>=2.32.0"
```

## Updating Dependencies

### Update all dependencies

```bash
# Recompile to get latest compatible versions
pip-compile --generate-hashes --upgrade requirements.in
pip-compile --generate-hashes --upgrade requirements-test.in
pip-compile --generate-hashes --upgrade requirements-security.in

# Sync environment
pip-sync requirements.txt requirements-test.txt requirements-security.txt

# Test everything
pytest
```

### Update specific dependency

```bash
# Update only requests
pip-compile --generate-hashes --upgrade-package requests requirements.in

# Sync and test
pip-sync requirements.txt
pytest
```

### Update for security patch

```bash
# 1. Identify vulnerable package from security scan
# 2. Update version constraint in .in file
echo "requests>=2.32.3" > requirements.in  # Fix CVE

# 3. Recompile
pip-compile --generate-hashes requirements.in

# 4. Verify fix
pip-audit -r requirements.txt

# 5. Commit
git add requirements.in requirements.txt
git commit -m "security: Update requests to 2.32.3 (CVE-XXXX-YYYY)"
```

## Best Practices

### 1. Always use hashes

**Required:** All requirements files MUST include SHA256 hashes.

```bash
# ✅ CORRECT
pip-compile --generate-hashes requirements.in

# ❌ WRONG - No hashes
pip-compile requirements.in
```

### 2. Pin to exact versions in applications

For **applications** (not libraries), pin exact versions:

```text
# requirements.in for applications
requests==2.32.3  # Exact version
jsonschema==4.23.0
```

For **libraries**, use ranges:

```text
# setup.py or pyproject.toml for libraries
requests>=2.32.0,<3.0.0
```

### 3. Separate concerns

Keep different dependency types separate:

- **Runtime** (`requirements.txt`) - Minimal dependencies for production
- **Testing** (`requirements-test.txt`) - Test framework and tools
- **Security** (`requirements-security.txt`) - Security scanners
- **Development** (`requirements-dev.txt`) - Optional dev tools

### 4. Review transitive dependencies

When compiling, review what gets pulled in:

```bash
pip-compile --generate-hashes requirements.in | tee compile.log

# Check for unexpected packages
grep "via" compile.log
```

### 5. Regular updates

**Schedule:**
- **Weekly:** Check for security updates (automated by Dependabot)
- **Monthly:** Update all dependencies to latest compatible versions
- **Quarterly:** Review and update version constraints in .in files

### 6. Audit before committing

Always audit before committing updated requirements:

```bash
# Security audit
pip-audit -r requirements.txt

# Check for known vulnerabilities
safety check -r requirements.txt

# Verify hashes present
grep -c "hash=sha256" requirements.txt
```

## Security Considerations

### Hash Verification Prevents

1. **Package tampering** - Modified packages detected by hash mismatch
2. **Dependency confusion** - Wrong package version can't be installed
3. **Supply chain attacks** - Malicious package updates blocked
4. **Registry compromises** - Even if PyPI is compromised, hashes protect you

### When Hashes Are Verified

```bash
# ✅ Hash verification ENABLED
pip install -r requirements.txt --require-hashes

# ⚠️ Hash verification DISABLED (dangerous!)
pip install -r requirements.txt
```

**Always use `--require-hashes` in production!**

### Handling Hash Errors

If you see:
```
THESE PACKAGES DO NOT MATCH THE HASHES FROM THE REQUIREMENTS FILE
```

**DO NOT** ignore this error! This indicates:
- Package on PyPI has changed
- Possible supply chain attack
- Corrupted download

**Action:**
1. Verify package legitimacy
2. Check package changelog for unexpected changes
3. Recompile with `pip-compile --generate-hashes`
4. Audit the new hashes

## CI/CD Integration

### GitHub Actions

```yaml
- name: Set up Python
  uses: actions/setup-python@SHA
  with:
    python-version: '3.12'
    cache: 'pip'
    cache-dependency-path: |
      requirements.txt
      requirements-test.txt

- name: Install dependencies
  run: |
    pip install --upgrade pip
    pip install -r requirements.txt --require-hashes
    pip install -r requirements-test.txt --require-hashes

- name: Verify dependencies
  run: |
    pip-audit -r requirements.txt
    pip-audit -r requirements-test.txt
```

### Pre-commit Hook

Add to `.pre-commit-config.yaml`:

```yaml
- repo: local
  hooks:
    - id: check-requirements-hashes
      name: Verify requirements have hashes
      entry: bash -c 'grep -q "hash=sha256" requirements.txt || (echo "ERROR: requirements.txt missing hashes" && exit 1)'
      language: system
      pass_filenames: false
```

## Troubleshooting

### Problem: pip-compile is slow

**Solution:** Use a local cache and parallel resolution:

```bash
pip-compile --generate-hashes --resolver=backtracking requirements.in
```

### Problem: Conflicting dependencies

**Solution:** Adjust version constraints in .in file:

```text
# If package-a requires requests<3.0 and package-b requires requests>=3.0
# Choose compatible versions:
package-a>=1.0,<2.0  # Requires requests<3.0
package-b>=2.5,<3.0  # Also works with requests<3.0
```

### Problem: Missing package hash

**Solution:** Recompile with `--generate-hashes`:

```bash
pip-compile --generate-hashes requirements.in
```

### Problem: Package has no wheels

**Solution:** You may need to build from source or find alternative:

```bash
# Allow building from source (security risk!)
pip-compile --generate-hashes --allow-unsafe requirements.in

# Better: Find package with wheels or alternatives
```

## References

- [pip-tools Documentation](https://pip-tools.readthedocs.io/)
- [PEP 665: Specifying Installation Requirements](https://peps.python.org/pep-0665/)
- [Hash-checking mode](https://pip.pypa.io/en/stable/topics/secure-installs/)
- [PYSEC_OMEGA Standards](copilot/PYSEC.md)
- [Supply Chain Security Guide](docs/security/WORKFLOW_SECURITY_POLICY.md)

## Summary

**Key Commands:**

```bash
# Add dependency
echo "package>=1.0.0" >> requirements.in
pip-compile --generate-hashes requirements.in

# Update all
pip-compile --generate-hashes --upgrade requirements.in

# Update one package
pip-compile --generate-hashes --upgrade-package package requirements.in

# Install with verification
pip install -r requirements.txt --require-hashes

# Audit for vulnerabilities
pip-audit -r requirements.txt
```

**Remember:**
- ✅ Always use `--generate-hashes`
- ✅ Always use `--require-hashes` when installing
- ✅ Commit both .in and .txt files
- ✅ Audit before every commit
- ✅ Review transitive dependencies
- ✅ Keep dependencies up to date
