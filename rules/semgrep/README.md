# BazBOM Curated JVM Semgrep Ruleset

This directory contains BazBOM's curated security ruleset for JVM languages (Java, Kotlin, Scala). The rules focus on high-impact vulnerabilities commonly found in JVM applications.

## Overview

The `semgrep-jvm.yml` ruleset is automatically used when running:

```bash
bazbom scan . --with-semgrep
```

## Coverage

The ruleset includes rules for:

### Critical Vulnerabilities (ERROR severity)
1. **Deserialization** (CWE-502) - Unsafe ObjectInputStream usage
2. **SQL Injection** (CWE-89) - Direct string concatenation in queries
3. **XML External Entity (XXE)** (CWE-611) - Unsafe XML parser configuration
4. **Command Injection** (CWE-78) - Runtime.exec with string concatenation
5. **Path Traversal** (CWE-22) - Unsanitized file path construction
6. **LDAP Injection** (CWE-90) - String concatenation in LDAP filters
7. **Server-Side Request Forgery (SSRF)** (CWE-918) - Unvalidated URL requests

### Security Warnings (WARNING severity)
8. **Weak Cryptography** (CWE-327) - DES, RC4, MD5, SHA1 usage
9. **Hardcoded Secrets** (CWE-798) - Passwords, API keys, tokens in code
10. **Insecure Random** (CWE-330) - java.util.Random for security-sensitive operations

## Rule Selection Criteria

Rules are included based on:
- **High impact**: Direct exploitability or significant security consequences
- **Low false positives**: Patterns that reliably indicate real issues
- **OWASP Top 10 2021 alignment**: Maps to current threat landscape
- **CWE coverage**: Well-defined Common Weakness Enumeration categories

## Updating the Ruleset

The ruleset is version-controlled and reviewed before updates. To propose a new rule:

1. Ensure the rule covers a high-impact vulnerability
2. Test against false positive rates on real codebases
3. Include CWE and OWASP mappings in metadata
4. Add to `semgrep-jvm.yml` with clear message and fix guidance

## Pinned Version

BazBOM uses a pinned, audited version of this ruleset. Updates are:
- Reviewed for quality and false positive rates
- Tested against benchmark projects
- Versioned with SHA-256 hashes (when external rules are incorporated)

## Integration with BazBOM

When `--with-semgrep` is enabled:
1. BazBOM checks for system-installed `semgrep` in PATH
2. Falls back to downloading and caching Semgrep via tool manifest
3. Runs Semgrep with this curated ruleset
4. Converts findings to SARIF 2.1.0 format
5. Merges with other analyzer results (SCA, CodeQL)
6. Uploads to GitHub Code Scanning

## Performance

Target: < 120 seconds on typical JVM projects (50-100K LOC)
- Rules are optimized for speed without sacrificing accuracy
- Timeout configured at 120 seconds per run
- Scope can be limited to changed modules with `--target`

## References

- [Semgrep Rules Registry](https://semgrep.dev/explore)
- [OWASP Top 10 2021](https://owasp.org/Top10/)
- [CWE Top 25](https://cwe.mitre.org/top25/)
- [GitHub Code Scanning](https://docs.github.com/en/code-security/code-scanning)

## License

These rules are part of BazBOM and distributed under the MIT License.
