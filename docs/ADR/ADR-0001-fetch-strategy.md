# ADR-0001: Maven Dependency Fetch Strategy

**Status**: Accepted

**Date**: 2025-10-17

**Context**: We need a reliable and secure way to fetch Maven dependencies for SBOM generation and analysis.

## Decision

We will use `rules_jvm_external` with pinned versions and lockfiles (`maven_install.json`).

## Rationale

### Options Considered

1. **Direct HTTP downloads**
   - ❌ No caching
   - ❌ No checksum verification
   - ❌ Manual dependency resolution

2. **Maven CLI (mvn)**
   - ❌ Non-deterministic
   - ❌ Requires Maven installation
   - ❌ Not Bazel-native

3. **rules_jvm_external** (Chosen)
   - ✅ Bazel-native integration
   - ✅ Automatic checksum verification
   - ✅ Lockfile support for reproducibility
   - ✅ Built-in caching
   - ✅ Transitive dependency resolution

## Implementation

```python
# In WORKSPACE
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "rules_jvm_external",
    strip_prefix = "rules_jvm_external-6.0",
    sha256 = "85fd6bad58ac76cc3a27c8e051e4255ff9ccd8c92ba879670d195622e7c0a9b7",
    url = "https://github.com/bazelbuild/rules_jvm_external/releases/download/6.0/rules_jvm_external-6.0.tar.gz",
)

load("@rules_jvm_external//:defs.bzl", "maven_install")

maven_install(
    artifacts = [
        "com.google.guava:guava:31.1-jre",
    ],
    repositories = [
        "https://repo1.maven.org/maven2",
    ],
    maven_install_json = "@//:maven_install.json",
    fail_on_missing_checksum = True,
)
```

## Consequences

### Positive

- **Reproducible Builds**: Lockfile ensures same dependencies across environments
- **Security**: Checksum verification prevents tampering
- **Performance**: Bazel caching speeds up repeated builds
- **Simplicity**: Single source of truth for dependencies

### Negative

- **Lockfile Maintenance**: Must regenerate lockfile when dependencies change
- **Learning Curve**: Bazel-specific approach may be unfamiliar
- **Initial Setup**: Requires proper WORKSPACE configuration

### Neutral

- **Bazel Dependency**: Ties project to Bazel ecosystem (acceptable given project goals)

## Alternatives Considered

### Coursier

Coursier is a fast dependency resolver for JVM.

**Pros**:
- Very fast resolution
- Used by Scala community

**Cons**:
- Additional tooling required
- Not Bazel-native
- Would need custom integration

**Decision**: Not chosen; rules_jvm_external provides better Bazel integration.

### Gradle with Dependency Locking

Gradle has built-in dependency locking.

**Pros**:
- Familiar to many developers
- Good IDE integration

**Cons**:
- Not Bazel-native
- Would require running Gradle alongside Bazel
- Adds complexity

**Decision**: Not chosen; stay Bazel-native.

## Implementation Notes

### Updating Dependencies

```bash
# Modify artifacts in WORKSPACE
# Then regenerate lockfile
bazel run @maven//:pin
git add maven_install.json
git commit -m "chore: update dependencies"
```

### Security Scanning

Dependencies fetched via rules_jvm_external are automatically included in SBOM generation via aspects.

### Future Enhancements

- Consider supporting private Maven repositories with authentication
- Explore mirroring strategy for air-gapped environments
- Evaluate dependency update automation (e.g., Renovate, Dependabot)

## References

- [rules_jvm_external Documentation](https://github.com/bazelbuild/rules_jvm_external)
- [Bazel External Dependencies Guide](https://bazel.build/external/overview)
- [SLSA Build Level 2 Requirements](https://slsa.dev/spec/v1.0/levels#build-l2)
