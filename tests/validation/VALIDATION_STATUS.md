# BazBOM Validation Status

**Date:** 2025-11-19
**Version:** 6.6.0

## Summary

Started validation testing. Found several issues that need fixing before production use.

## Test Results

### Java/Maven

| Feature | Status | Notes |
|---------|--------|-------|
| SBOM Generation | ✅ Pass | Detected 4 packages from pom.xml |
| Vulnerability Scanning | ✅ Pass | Found 59 CVEs (log4j, jackson, commons-collections) |
| Reachability Analysis | ❌ Fail | "Found 0 JARs" - can't find deps in ~/.m2/repository |
| Auto-Remediation | ⏸️ Blocked | Needs reachability working first |

### TypeScript/npm

| Feature | Status | Notes |
|---------|--------|-------|
| SBOM Generation | ⏸️ Pending | Test project created |
| Vulnerability Scanning | ⏸️ Pending | |
| Reachability Analysis | ⏸️ Pending | |

### Python/pip

| Feature | Status | Notes |
|---------|--------|-------|
| SBOM Generation | ⏸️ Pending | Test project created |
| Vulnerability Scanning | ⏸️ Pending | |
| Reachability Analysis | ⏸️ Pending | |

## Bugs Found

### Critical

1. **Maven reachability can't find JARs**
   - Location: `bazbom-reachability` or `scan_orchestrator.rs`
   - Symptom: "Analyzing Maven dependencies in ~/.m2/repository" then "Found 0 JARs"
   - JARs exist at `~/.m2/repository/org/apache/logging/log4j/log4j-core/2.14.1/`

### High

2. **`--fast` mode skips Maven entirely**
   - Location: `scan_orchestrator.rs:1169`
   - Maven scanner is in polyglot path, which fast mode skips
   - JVM-only projects get zero packages with `--fast`

3. **Small repo auto-enables fast mode**
   - "Small repo detected - enabling reachability (fast)"
   - This overrides user intent and causes Maven to not be scanned

## Target Repo Info

- **Bazel:** 8.3.1 with bzlmod
- **Dependencies:** rules_jvm_external 6.7
- **Format:** Multiple `maven_install.json` files
- **Size:** 9GB, 175K items
- **Languages:** Java 74%, TypeScript 8%, Clojure 7%, Python 3%

Parser confirmed compatible with their `maven_install.json` format.

## Next Steps

1. Fix Maven JAR discovery in reachability
2. Fix fast mode to include Maven scanning for JVM projects
3. Complete TypeScript/Python validation
4. Test on actual target repo

## Test Projects

Created validation test projects with intentionally vulnerable dependencies:

- `java-maven/` - log4j 2.14.1, jackson 2.9.8, commons-collections 3.2.1
- `typescript/` - lodash 4.17.20, axios 0.21.1
- `python/` - requests 2.25.0, pyyaml 5.3, pillow 8.0.0

Each has both reachable and unreachable vulnerable deps for testing.
