# Shaded/Fat JAR Example

This example demonstrates how BazBOM handles shaded (fat) JARs that bundle all dependencies into a single JAR file.

## What is a Shaded JAR?

A shaded JAR (also called a fat JAR or uber JAR) is a JAR file that contains:
- Application classes
- All dependency classes (bundled inside)
- Optionally relocated packages (to avoid conflicts)

### Common Use Cases

1. **Standalone executables**: Distribute a single JAR that "just works"
2. **Lambda functions**: AWS Lambda, Google Cloud Functions, etc.
3. **Command-line tools**: Easy to distribute and run
4. **Plugin systems**: Avoid dependency conflicts with host application

## Building

### Build the application

```bash
cd examples/shaded_jar
bazel build //:app
```

### Run the application

```bash
bazel run //:app
```

Expected output:
```
Processing words:
  - Supply
  - Chain
  - Security

This application can be packaged as a fat/shaded JAR
with all dependencies included for easy deployment.
```

### Create a deploy JAR (fat JAR)

```bash
bazel build //:app_deploy.jar
```

This creates a single JAR containing all dependencies.

## SBOM Challenges with Shaded JARs

Shaded JARs present unique challenges for SBOM generation:

### Problem 1: Hidden Dependencies

Dependencies are embedded, not declared:
```
app.jar
├── com/example/shaded/ShadedApp.class
├── com/google/common/...                    ← Guava embedded
└── org/apache/commons/text/...             ← Commons Text embedded
```

### Problem 2: Relocated Packages

Packages may be renamed to avoid conflicts:
```
Original:  com.google.common.collect.ImmutableList
Relocated: com.myapp.shaded.guava.collect.ImmutableList
```

### Problem 3: Missing Metadata

Embedded JARs may lose:
- Original POM files
- License files
- Version information

## BazBOM's Solution

### Detection Strategy

1. **JAR structure analysis**: Detect nested JAR patterns
2. **Package name patterns**: Identify known library packages
3. **META-INF inspection**: Extract embedded POM files
4. **Manifest analysis**: Read Bundle-License and other headers

### Dependency Reconstruction

```python
# BazBOM analyzes shaded JAR and reconstructs:
{
  "artifact": "app.jar",
  "type": "shaded",
  "embedded_dependencies": [
    {
      "purl": "pkg:maven/com.google.guava/guava@31.1-jre",
      "detected_from": "com.google.common package",
      "original_location": "META-INF/maven/com.google.guava/guava/pom.xml"
    },
    {
      "purl": "pkg:maven/org.apache.commons/commons-text@1.10.0",
      "detected_from": "org.apache.commons.text package",
      "original_location": "META-INF/maven/org.apache.commons/commons-text/pom.xml"
    }
  ]
}
```

### SBOM Generation

The SBOM includes both:

1. **Top-level artifact**: The shaded JAR itself
2. **Embedded components**: All bundled dependencies

```json
{
  "packages": [
    {
      "name": "app",
      "versionInfo": "1.0.0",
      "filesAnalyzed": true,
      "licenseConcluded": "Apache-2.0"
    },
    {
      "name": "guava",
      "versionInfo": "31.1-jre",
      "relationship": "CONTAINS",
      "comment": "Embedded in shaded JAR"
    },
    {
      "name": "commons-text",
      "versionInfo": "1.10.0",
      "relationship": "CONTAINS",
      "comment": "Embedded in shaded JAR"
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-app",
      "relationshipType": "CONTAINS",
      "relatedSpdxElement": "SPDXRef-guava"
    }
  ]
}
```

## Best Practices

### 1. Preserve Metadata

When creating shaded JARs, keep:
- `META-INF/maven/` directories
- `META-INF/LICENSE*` files
- `META-INF/NOTICE*` files
- Manifest headers

### 2. Document Shading

Add manifest entry:
```
Manifest-Version: 1.0
Shaded-Dependencies: com.google.guava:guava:31.1-jre,org.apache.commons:commons-text:1.10.0
```

### 3. Use Consistent Naming

For relocated packages:
```
com.google.common → com.myapp.shaded.guava
```

### 4. Generate SBOM Before Shading

Create SBOM from build definition, not just final JAR:
```bash
# Generate SBOM from build dependencies (more accurate)
bazel build //:app_sbom

# Then create shaded JAR
bazel build //:app_deploy.jar
```

## Validation

### Verify SBOM Completeness

```bash
# Generate SBOM
bazel build //:app_sbom

# Check that all dependencies are captured
cat bazel-bin/app_sbom.spdx.json | jq '.packages[].name'
# Should include: app, guava, commons-text, ...
```

### License Compliance

```bash
# Extract all licenses from shaded JAR
bazel run //tools/supplychain:license_extractor -- \
  --jar bazel-bin/app_deploy.jar \
  --output license_report.json

# Verify all licenses are accounted for
cat license_report.json | jq '.licenses'
```

## Advanced Scenarios

### Relocated Packages

```java
// Original package
import com.google.common.collect.ImmutableList;

// After shading/relocation
import com.myapp.shaded.guava.collect.ImmutableList;
```

BazBOM detects this by:
1. Analyzing class signatures
2. Checking for relocation rules in POM
3. Mapping relocated names back to originals

### Nested JARs

Some tools create JARs with JARs inside:
```
app.jar
├── classes/
│   └── com/example/...
└── lib/
    ├── guava-31.1-jre.jar
    └── commons-text-1.10.0.jar
```

BazBOM handles this by:
1. Unpacking to temporary directory
2. Analyzing each nested JAR
3. Creating CONTAINS relationships

## Known Limitations

1. **Version detection**: May fail if version info is removed
2. **Custom relocations**: Requires relocation rule documentation
3. **Minified code**: Obfuscation makes detection harder
4. **Dynamic loading**: Runtime-loaded classes may be missed

## Future Enhancements

- [ ] Automatic relocation rule detection
- [ ] Support for ProGuard/R8 obfuscation
- [ ] JAR unpacking for deep analysis
- [ ] Comparison with pre-shading dependency list
- [ ] Detect version mismatches between build and runtime

## Related Tools

- **Maven Shade Plugin**: Creates shaded JARs
- **Shadow Plugin** (Gradle): Alternative shading tool
- **Bazel rules_jvm**: Built-in deploy JAR support
- **JarJar Links**: Package relocation tool

## References

- [SPDX Specification](https://spdx.github.io/spdx-spec/) - CONTAINS relationship
- [Maven Shade Plugin](https://maven.apache.org/plugins/maven-shade-plugin/)
- [CycloneDX Component Types](https://cyclonedx.org/docs/1.5/json/#components_items_type)
