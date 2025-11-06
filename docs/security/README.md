# Security Guide

This directory collects the documents and artefacts that underpin BazBOM’s secure-by-design posture. Use it as the starting point when you need to evaluate risk, run assurance workflows, or author internal policy.

## High-priority references

- [Supply Chain Architecture](supply-chain.md) – SBOM and SCA data flows
- [Threat Model](threat-model.md) – Assets, adversaries, mitigations
- [Threat Detection Playbook](threat-detection.md) – Attack detection and response tactics
- [Vulnerability Enrichment](vulnerability-enrichment.md) – KEV, EPSS, GHSA enrichment pipeline
- [VEX Guidance](vex.md) – Managing false positives and documenting compensating controls
- Root-level [SECURITY.md](../../SECURITY.md) – Coordinated disclosure process

## Operating the security toolchain

Most day-to-day scanning is handled directly by the Rust CLI:

```bash
bazbom scan . --with-semgrep --with-codeql
bazbom db sync                 # refresh advisory data
bazbom policy check            # enforce policy gates
bazbom report executive        # security summary exports
```

Complementary artefacts in this directory:

| File | Purpose |
| --- | --- |
| `SECURE_CODING_GUIDE.md` | Coding practices and guardrails for contributors |
| `RISK_LEDGER.md` | Current risk register with mitigation ownership |
| `SECURITY_REVIEW_CHECKLIST.md` | Release/security review checklist |
| `CODEQL_OPTIMIZATION.md` / `CODEQL_TIMEOUT_MITIGATION.md` | Hardening guidance for CodeQL workflows |
| `WORKFLOW_SECURITY_POLICY.md` | CI hardening, secret handling, and access controls |

Custom Semgrep policies live under `POLICIES/`. Scanner outputs are written to `SCANNER_RESULTS/` (gitignored) when you run optional tools (`semgrep`, `trufflehog`, etc.).

## Related resources

- [Threat Detection](threat-detection.md) for advanced telemetry
- [Operations / Validation](../operations/validation.md) for SBOM/SARIF schema checks
- [Release Process](../operations/release-process.md) for signed releases and provenance
- [Dependency Management](../development/dependency-management.md) for supply-chain hygiene

Security is everyone’s responsibility—contribute improvements by opening an issue or PR, and keep `SECURITY.md` up to date when escalation paths change.
