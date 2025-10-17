# Multi-Module Monorepo Example

This example demonstrates BazBOM's capabilities in a complex monorepo with multiple Java modules.

## Structure

```
multi_module/
├── common/          # Shared utilities library
│   └── StringUtils.java
├── lib/             # Business logic library
│   └── DataProcessor.java
└── app/             # Application binary
    └── Application.java
```

### Module Dependencies

```
app (binary)
  ├── lib (library)
  │   ├── common (library)
  │   ├── gson (maven)
  │   └── commons-lang3 (maven)
  └── common (library)
      └── guava (maven)
```

## Building

### Build the application

```bash
cd examples/multi_module
bazel build //app:app
```

### Run the application

```bash
bazel run //app:app -- "test input"
```

Expected output:
```json
{"original":"test input","processed":"Test input          "}
```

### Build all modules

```bash
bazel build //...
```

## SBOM Generation

### Generate SBOM for the application

This will capture all dependencies transitively:

```bash
# TODO: Add SBOM generation command once aspect-based approach is implemented
# Expected: bazel build //app:app_sbom
```

The SBOM will include:
- Direct dependencies: lib, common
- Transitive Maven dependencies: guava, gson, commons-lang3
- All transitive dependencies of Maven artifacts

### Generate SBOM for individual modules

```bash
# TODO: Module-specific SBOM generation
# Expected: bazel build //lib:lib_sbom
# Expected: bazel build //common:common_sbom
```

## Dependency Analysis

### View dependency graph

```bash
# Query dependencies of the app
bazel query 'deps(//app:app)' --output graph

# Query what depends on common
bazel query 'rdeps(//..., //common:common)' --output graph
```

### Analyze transitive dependencies

```bash
# Show all Maven dependencies
bazel query 'filter("@maven", deps(//app:app))'
```

## Supply Chain Analysis

### Vulnerability Scanning

Once SBOM is generated, scan for vulnerabilities:

```bash
# TODO: Add SCA scan command
# Expected: bazel build //app:app_sca_scan
```

### License Compliance

Check licenses across all modules and dependencies:

```bash
# TODO: Add license report command
# Expected: bazel build //app:app_license_report
```

## Key Features Demonstrated

1. **Multi-module structure**: Common pattern in large Java projects
2. **Internal dependencies**: Modules depend on each other (app → lib → common)
3. **Maven dependencies**: External dependencies from Maven Central
4. **Transitive dependencies**: Maven artifacts bring their own dependencies
5. **Dependency graph**: Complex dependency relationships

## Use Cases

This example is useful for:

- **Large monorepos**: Multiple teams owning different modules
- **Microservices**: Each module could be a separate service
- **Library ecosystem**: Publishing multiple libraries from one repo
- **Incremental builds**: Only rebuild changed modules
- **Fine-grained SBOMs**: Per-module or per-artifact SBOMs

## Expected SBOM Output

The SBOM for `//app:app` should include:

### Packages
- `app` (internal package)
- `lib` (internal package)
- `common` (internal package)
- `com.google.guava:guava:31.1-jre` (maven)
- `com.google.code.gson:gson:2.10.1` (maven)
- `org.apache.commons:commons-lang3:3.12.0` (maven)
- All transitive dependencies of the above

### Relationships
```
DESCRIBES: app
CONTAINS: lib, common, guava, gson, commons-lang3
DEPENDS_ON: lib → common, lib → gson, lib → commons-lang3
DEPENDS_ON: common → guava
```

## Performance Considerations

For large monorepos:

1. **Incremental analysis**: Only regenerate SBOMs for changed modules
2. **Parallel builds**: Bazel builds modules in parallel
3. **Caching**: Bazel caches unchanged module outputs
4. **Selective SBOMs**: Generate SBOMs only for deployable artifacts (binaries)

## Next Steps

1. Implement aspect-based SBOM generation for multi-module projects
2. Add per-module SBOM targets
3. Add aggregated workspace SBOM
4. Add incremental analysis support
5. Add CI/CD example for monorepos
