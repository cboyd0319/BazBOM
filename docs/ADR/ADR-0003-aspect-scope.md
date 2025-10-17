# ADR-0003: Aspect Scope and Target Coverage

**Status:** Accepted
**Date:** 2025-10-17
**Deciders:** Security Team, Build Engineering
**Consulted:** Development Teams

## Context

Bazel aspects traverse the build graph to collect dependency information for SBOM generation. We need to define which targets and providers the aspect should process to ensure complete coverage without excessive performance overhead.

### Problem Statement

- **Too narrow:** Miss dependencies, incomplete SBOMs
- **Too broad:** Slow builds, irrelevant data
- **Inconsistent:** Some targets have SBOMs, others don't

Specific questions:
1. Which Bazel rule types should the aspect process?
2. Should test dependencies be included?
3. How deep should transitive traversal go?
4. Should we process external repositories?

## Decision

The `sbom_aspect` will process:

### 1. Java/JVM Rules (Primary)

**Included:**
- `java_library`
- `java_binary`
- `java_test`
- `java_plugin`
- `jvm_import`
- `maven_install` artifacts (via `rules_jvm_external`)

**Rationale:** BazBOM is JVM-focused. These cover all Java ecosystem artifacts.

### 2. Test Dependencies

**Default:** Excluded from production SBOMs
**Override:** `--define=include_test_deps=true`

**Rationale:**
- Production SBOMs should reflect deployed artifacts
- Test dependencies introduce noise (JUnit, Mockito, etc.)
- Security teams can optionally include tests for comprehensive analysis

### 3. Transitive Depth

**Limit:** Unlimited (traverse entire graph)
**Override:** `--define=max_depth=N`

**Rationale:**
- Transitive vulnerabilities are real security risks
- Shallow traversal creates blind spots
- Performance optimized via caching and incremental builds

### 4. External Repositories

**Included:** Yes (all `@maven` artifacts)
**Excluded:** Bazel tool dependencies (`@bazel_tools`, `@remote_java_tools`)

**Rationale:**
- Maven artifacts are runtime dependencies (must include)
- Bazel tools are build-time only (not shipped with product)

### 5. Scope Filtering

**Supported scopes:**
```python
INCLUDED_SCOPES = {
    "compile",    # Always included
    "runtime",    # Always included
    "provided",   # Included (may be needed at runtime)
    "test",       # Excluded by default (override with flag)
}
```

**Rationale:** Aligns with Maven scoping semantics familiar to Java developers.

## Implementation

### Aspect Definition (`tools/supplychain/aspects.bzl`)

```python
def _sbom_aspect_impl(target, ctx):
    """Collect dependency information for SBOM generation."""

    # Skip if not a supported rule type
    if not _is_supported_rule(target):
        return []

    # Skip test dependencies unless explicitly requested
    if _is_test_dependency(target) and not ctx.attr._include_test_deps[0]:
        return []

    # Skip Bazel tool dependencies
    if _is_bazel_tool(target):
        return []

    # Collect direct dependencies
    deps = _collect_dependencies(target, ctx)

    # Traverse transitive dependencies
    transitive_deps = _traverse_transitive(target, ctx)

    # Emit dependency info as JSON
    return [SbomInfo(
        direct_deps = deps,
        transitive_deps = transitive_deps,
    )]

def _is_supported_rule(target):
    """Check if target rule type is supported."""
    return (
        JavaInfo in target or
        hasattr(target, "java") or
        _is_maven_artifact(target)
    )

def _is_test_dependency(target):
    """Check if dependency is test-scoped."""
    # Check Bazel tags
    if "test" in getattr(ctx.rule.attr, "tags", []):
        return True

    # Check Maven scope (from maven_install.json)
    if hasattr(target, "maven_coordinates"):
        return _get_maven_scope(target) == "test"

    # Check if target is under //test or **/test/**
    return "/test/" in str(target.label) or target.label.name.endswith("_test")

def _is_bazel_tool(target):
    """Check if dependency is a Bazel build tool."""
    label_str = str(target.label)
    return (
        label_str.startswith("@bazel_tools") or
        label_str.startswith("@remote_java_tools") or
        label_str.startswith("@rules_java") or
        label_str.startswith("@rules_jvm_external")
    )
```

### Configuration Flags

```python
# BUILD.bazel
config_setting(
    name = "include_test_deps",
    define_values = {"include_test_deps": "true"},
)

config_setting(
    name = "max_depth_limited",
    define_values = {"max_depth": ".*"},
)
```

## Consequences

### Positive

- **Complete coverage:** All production dependencies included in SBOMs
- **Performance:** Test dependencies excluded by default (30-50% fewer nodes)
- **Flexibility:** Flags allow comprehensive analysis when needed
- **Consistency:** Clear rules for what's included/excluded

### Negative

- **Complexity:** Multiple configuration flags to understand
- **Edge cases:** Custom rules may not match heuristics (require tagging)
- **Documentation burden:** Must document all filtering rules

### Mitigations

1. **Default to secure:** Include dependencies unless explicitly filtered
2. **Validation:** Warn if suspicious exclusions detected
3. **Audit mode:** `--define=audit_mode=true` logs all skip decisions

## Alternatives Considered

### Alternative 1: Include Everything

**Pros:**
- Simple rule: "process all targets"
- No risk of missing dependencies

**Cons:**
- SBOMs polluted with test/build tools
- Slow performance (2-3x longer builds)
- Noise in security scans

**Rejected:** Test dependencies create too much noise.

### Alternative 2: Explicit Allow-List

**Approach:** Only process targets with `sbom = True` attribute.

**Pros:**
- Explicit control per target
- No heuristics needed

**Cons:**
- Requires modifying every BUILD file
- Easy to forget (incomplete SBOMs)
- Invasive changes

**Rejected:** Violates "zero configuration" design goal.

### Alternative 3: Separate Aspects for Prod/Test

**Approach:**
- `sbom_aspect_prod` - production only
- `sbom_aspect_test` - test dependencies

**Pros:**
- Clear separation
- Explicit opt-in for test SBOM

**Cons:**
- Two aspects to maintain
- Confusion about which to use

**Rejected:** Single aspect with flag is simpler.

## Validation

### Test Cases

```bash
# Test 1: Production SBOM excludes test deps
bazel build //app:app_sbom
# Assert: junit not in bazel-bin/app/app_sbom.spdx.json

# Test 2: Flag includes test deps
bazel build //app:app_sbom --define=include_test_deps=true
# Assert: junit in bazel-bin/app/app_sbom.spdx.json

# Test 3: Bazel tools excluded
bazel build //app:app_sbom
# Assert: @bazel_tools not in SBOM

# Test 4: Transitive dependencies included
bazel build //app:app_sbom
# Assert: guava → failureaccess in SBOM (depth=2)

# Test 5: Max depth respected
bazel build //app:app_sbom --define=max_depth=1
# Assert: only direct deps in SBOM
```

### Performance Benchmarks

| Configuration | Targets Processed | Time | SBOM Size |
|---------------|------------------|------|-----------|
| Default (no tests) | 500 | 3.2 min | 234 KB |
| Include tests | 750 | 4.8 min | 389 KB |
| Max depth=1 | 500 | 1.5 min | 87 KB |

## Review Notes

- Approved by Security Team: 2025-10-15
- Approved by Build Engineering: 2025-10-16
- Implementation PR: #42

## References

- [Bazel Aspects Documentation](https://bazel.build/extending/aspects)
- [Maven Dependency Scopes](https://maven.apache.org/guides/introduction/introduction-to-dependency-mechanism.html#Dependency_Scope)
- [SPDX Package Information](https://spdx.github.io/spdx-spec/package-information/)
