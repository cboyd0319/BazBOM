# BazBOM Validation Test Suite

This directory contains test projects for validating BazBOM's features across all ecosystems.

## Validation Goals

Each ecosystem test validates:
1. **SBOM Generation** - Correct dependency detection
2. **Vulnerability Scanning** - Known CVEs detected
3. **Reachability Analysis** - Unreachable vulns filtered
4. **Auto-Remediation** - Version updates applied correctly

## Test Projects

| Directory | Ecosystem | Build System | Status |
|-----------|-----------|--------------|--------|
| `java-maven` | Java | Maven | ⚠️ Needs testing |
| `java-gradle` | Java | Gradle | ⚠️ Needs testing |
| `bazel` | Java | Bazel | ⚠️ Needs testing |
| `typescript` | TypeScript | npm | ⚠️ Needs testing |
| `python` | Python | pip | ⚠️ Needs testing |
| `go` | Go | go modules | ⚠️ Needs testing |
| `rust` | Rust | Cargo | ⚠️ Needs testing |
| `ruby` | Ruby | Bundler | ⚠️ Needs testing |
| `php` | PHP | Composer | ⚠️ Needs testing |

## Running Validation

```bash
# Run all validations
./run-validation.sh

# Run specific ecosystem
./run-validation.sh java-maven
```

## Expected Results

Each test should:
- Detect the intentionally vulnerable dependency
- Show reachability status (reachable/unreachable)
- Suggest correct remediation version
- Apply fix without breaking the project
