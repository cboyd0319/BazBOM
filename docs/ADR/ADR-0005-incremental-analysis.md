# ADR-0005: Git-Based Incremental Analysis Strategy

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Build Engineering, DevOps

## Context

Full workspace SBOM generation on every PR is slow (15-30 min for large repos). Most PRs change < 10% of targets. We need incremental analysis to maintain fast CI feedback.

## Decision

Use git diff to detect changed files, then Bazel query to find affected targets.

### Algorithm

```bash
# 1. Detect changed files
CHANGED_FILES=$(git diff --name-only origin/main...HEAD)

# 2. Convert to Bazel labels
CHANGED_LABELS=$(echo "$CHANGED_FILES" | sed 's|^|//|g')

# 3. Find reverse dependencies (what depends on changed files)
AFFECTED_TARGETS=$(bazel query "rdeps(//..., set($CHANGED_LABELS))")

# 4. Build SBOMs only for affected targets
bazel build $AFFECTED_TARGETS --aspects=//tools/supplychain:aspects.bzl%sbom_aspect
```

### Scope

**Include in incremental:**
- Modified source files
- Modified BUILD files
- Modified WORKSPACE/lockfiles
- Direct and transitive dependents

**Force full analysis:**
- Changes to aspect implementation
- Changes to SBOM generation scripts
- Weekly scheduled scans
- Release tags

## Implementation

`tools/supplychain/incremental_analyzer.py`:

```python
import subprocess

def get_affected_targets(base_ref: str = "origin/main") -> list[str]:
    """Get Bazel targets affected by changes since base_ref."""

    # Get changed files
    result = subprocess.run(
        ["git", "diff", "--name-only", f"{base_ref}...HEAD"],
        capture_output=True,
        text=True
    )
    changed_files = result.stdout.strip().split("\n")

    # Force full analysis if critical files changed
    critical_files = [
        "tools/supplychain/aspects.bzl",
        "tools/supplychain/write_sbom.py",
        "WORKSPACE",
    ]
    if any(f in changed_files for f in critical_files):
        return ["//..."]  # Full analysis

    # Query affected targets
    query = f"rdeps(//..., set({' '.join(changed_files)}))"
    result = subprocess.run(
        ["bazel", "query", query],
        capture_output=True,
        text=True
    )

    return result.stdout.strip().split("\n")
```

## Consequences

### Positive
- 5-10x faster PRanalyses
- Same security coverage (affected targets analyzed)
- Lower CI costs

### Negative
- Risk of missing indirect effects
- Added complexity in CI

### Mitigations
- Weekly full scans catch anything missed
- Force full analysis on critical file changes
- Document limitations clearly

## Alternatives Considered

**Alternative 1:** Bazel's built-in change detection
- **Rejected:** Doesn't work well with aspects

**Alternative 2:** Timestamp-based detection
- **Rejected:** Not reliable in CI (fresh clones)

## Validation

```bash
# Test: Change single file, verify only affected targets analyzed
echo "// comment" >> app/Main.java
git add app/Main.java
TARGETS=$(bazel run //tools/supplychain:incremental_analyzer)
# Assert: Only //app:* targets in output
```

## References
- [Bazel Query Documentation](https://bazel.build/query/guide)
- [Git Diff Documentation](https://git-scm.com/docs/git-diff)
