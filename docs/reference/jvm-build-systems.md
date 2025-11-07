# JVM Build Systems Support

**BazBOM Version:** 0.5.1+  
**Last Updated:** 2025-11-04  
**Status:** Production Ready

---

## Overview

BazBOM provides world-class SBOM, SCA, and dependency graph analysis for **JVM ecosystems only**. This document covers the three supported build systems and their capabilities.

### Supported Languages
-  **Java** (all versions)
-  **Kotlin** (JVM targets only)
-  **Scala** (JVM targets only)

### Supported Build Systems
-  **Maven** (100% production ready)
-  **Gradle** (100% production ready)
-  **Bazel** (100% production ready with JVM rules)

---

## Maven Support

### Status:  Production Ready (100%)

Maven is fully supported with comprehensive dependency resolution and SBOM generation.

### Supported Files
- `pom.xml` - Project Object Model
- Parent POMs with inheritance
- Multi-module projects
- Maven Shade plugin configurations
- `maven_install.json` (Bazel rules_jvm_external)

### Features

#### Dependency Resolution
- Complete transitive dependency tree
- Dependency scope detection:
  - `compile` - Runtime and compile-time
  - `runtime` - Runtime only  
  - `test` - Test dependencies
  - `provided` - Provided by container
  - `system` - System path
- Property resolution (`${project.version}`, etc.)
- Parent POM inheritance
- Dependency management sections
- Exclusions handling

#### Maven Shade Plugin
- Detection of shaded (fat) JARs
- Class relocation tracking
- Minimization detection
- Transformer configurations

#### Build Integration
- `bazbom-maven-plugin` for authoritative JSON output
- Direct pom.xml parsing as fallback
- Maven repository resolution

### Usage

```bash
# Scan Maven project
bazbom scan /path/to/maven/project

# Scan specific module in multi-module project
bazbom scan /path/to/maven/project --target my-module

# Generate CycloneDX format
bazbom scan /path/to/maven/project --cyclonedx
```

### Example Output

```json
{
  "name": "org.springframework:spring-core",
  "version": "5.3.31",
  "purl": "pkg:maven/org.springframework/spring-core@5.3.31",
  "scope": "compile",
  "sha256": "abc123...",
  "vulnerabilities": [
    {
      "id": "CVE-2024-xxxx",
      "severity": "HIGH",
      "cvss": 7.5
    }
  ]
}
```

### Performance
- Small projects (<100 deps): <1 second
- Medium projects (100-1000 deps): 2-5 seconds
- Large projects (1000+ deps): 10-30 seconds
- Multi-module (10+ modules): 15-45 seconds

---

## Gradle Support

### Status:  Production Ready (100%)

Gradle projects are fully supported with comprehensive configuration-based analysis.

### Supported Files
- `build.gradle` - Groovy DSL
- `build.gradle.kts` - Kotlin DSL
- `settings.gradle` / `settings.gradle.kts`
- `gradle.lockfile` - Dependency locking
- `gradle/libs.versions.toml` - Version catalogs

### Features

#### Dependency Resolution
- Configuration-based scoping:
  - `implementation` - Runtime dependencies
  - `api` - Public API dependencies
  - `compileOnly` - Compile-time only
  - `runtimeOnly` - Runtime only
  - `testImplementation` - Test dependencies
- Composite builds support
- Multi-project builds
- Dependency constraints
- Platform/BOM imports

#### Gradle Shadow Plugin
- Detection of shadow (fat) JARs
- Class relocation tracking
- Dependency merging
- Minimization detection

#### Build Integration
- `io.bazbom.gradle-plugin` for authoritative output
- Direct build file parsing as fallback
- Gradle dependency resolution API

### Usage

```bash
# Scan Gradle project
bazbom scan /path/to/gradle/project

# Scan specific subproject
bazbom scan /path/to/gradle/project --target :my-subproject

# Scan Android project
bazbom scan /path/to/android/project
```

### Kotlin DSL Example

```kotlin
// build.gradle.kts
dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib:1.9.21")
    implementation("io.ktor:ktor-server-core:2.3.7")
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
}
```

### Performance
- Small projects (<100 deps): <1 second
- Medium projects (100-1000 deps): 3-6 seconds
- Large projects (1000+ deps): 15-35 seconds
- Android projects: 20-60 seconds

---

## Bazel Support

### Status:  Production Ready (100%)

Bazel is fully supported with advanced monorepo optimizations.

### Supported Files
- `BUILD.bazel` - Build definitions
- `MODULE.bazel` - Bzlmod module definitions
- `WORKSPACE` - Legacy workspace file
- `maven_install.json` - rules_jvm_external lock file

### Supported Rules

#### JVM Rules (Priority)
- `java_library` - Java libraries
- `java_binary` - Java executables
- `java_test` - Java tests
- `java_plugin` - Annotation processors

#### Kotlin Rules
- `kt_jvm_library` - Kotlin libraries  
- `kt_jvm_binary` - Kotlin executables
- `kt_jvm_test` - Kotlin tests

#### Scala Rules
- `scala_library` - Scala libraries
- `scala_binary` - Scala executables
- `scala_test` - Scala tests

### Features

#### Dependency Resolution
- Complete dependency graph via `bazel query`
- Transitive dependency analysis
- Target filtering (`java_*`, `kt_*`, `scala_*`)
- Label resolution
- rules_jvm_external integration
- Maven coordinate extraction

#### Query Optimization
- Query result caching
- Batch query execution
- Incremental analysis
- Performance metrics tracking
- 50K+ target monorepo support

#### Build Integration
- Bazel aspects for dependency extraction
- bzlmod module system support
- rules_jvm_external direct integration
- Custom rule support

### Usage

```bash
# Scan entire Bazel workspace
bazbom scan /path/to/bazel/workspace

# Scan specific package
bazbom scan /path/to/bazel/workspace --target //my/package:target

# Scan with incremental analysis
bazbom scan /path/to/bazel/workspace --incremental

# Scan large monorepo with caching
bazbom scan /path/to/monorepo
```

### Bazel Query Examples

```bash
# Query all JVM targets
bazel query 'kind("java_.*|kt_jvm_.*|scala_.*", //...)'

# Query dependencies of a target
bazel query 'deps(//my/app:main)'

# Query transitive dependencies
bazel query 'rdeps(//..., @maven//:com_google_guava_guava)'
```

### Performance
- Small workspaces (<100 targets): <2 seconds
- Medium workspaces (100-1000 targets): 5-10 seconds
- Large workspaces (1000-10K targets): 15-60 seconds
- Monorepos (10K-50K targets): 60-300 seconds
- With caching (2nd run): 1-10 seconds

---

## Multi-Module Projects

BazBOM handles complex multi-module projects across all build systems.

### Maven Multi-Module

```xml
<!-- parent pom.xml -->
<modules>
    <module>core</module>
    <module>api</module>
    <module>app</module>
</modules>
```

```bash
# Scan all modules
bazbom scan /path/to/parent

# Scan specific module
bazbom scan /path/to/parent --target api
```

### Gradle Multi-Project

```kotlin
// settings.gradle.kts
include(":core", ":api", ":app")
```

```bash
# Scan all projects
bazbom scan /path/to/root

# Scan specific project
bazbom scan /path/to/root --target :api
```

### Bazel Monorepo

```python
# WORKSPACE or MODULE.bazel
# Large monorepo with 1000s of targets
```

```bash
# Scan entire monorepo (with optimizations)
bazbom scan /path/to/monorepo --incremental

# Scan specific package
bazbom scan /path/to/monorepo --target //services/api:all
```

---

## Shaded/Fat JARs

BazBOM detects and analyzes shaded (fat) JARs created by Maven Shade or Gradle Shadow plugins.

### Maven Shade Plugin

```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-shade-plugin</artifactId>
    <configuration>
        <relocations>
            <relocation>
                <pattern>com.google</pattern>
                <shadedPattern>my.app.shaded.google</shadedPattern>
            </relocation>
        </relocations>
    </configuration>
</plugin>
```

### Gradle Shadow Plugin

```kotlin
plugins {
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

shadowJar {
    relocate("com.google", "my.app.shaded.google")
    minimize()
}
```

### Detection Features
- Class fingerprinting with Blake3 hashing
- Relocation pattern detection
- Original artifact identification
- Minimization detection
- Transitive dependency tracking in shaded JARs

---

## SBOM Formats

BazBOM generates industry-standard SBOMs for all JVM projects.

### SPDX 2.3 (Primary)

```bash
# Generate SPDX SBOM
bazbom scan /path/to/project
# Output: .bazbom/sbom/spdx.json
```

### CycloneDX 1.5 (Optional)

```bash
# Generate CycloneDX SBOM
bazbom scan /path/to/project --cyclonedx
# Output: .bazbom/sbom/cyclonedx.json
```

### Format Features
- Complete dependency tree
- License information
- Vulnerability findings
- Build system metadata
- Package URLs (PURLs)
- SHA-256 checksums
- Relationship mappings

---

## Advisory Database Integration

BazBOM enriches JVM dependency data with multiple intelligence sources.

### Supported Sources
- **OSV.dev** - Open Source Vulnerabilities
- **NVD** - National Vulnerability Database  
- **GHSA** - GitHub Security Advisories
- **KEV** - CISA Known Exploited Vulnerabilities
- **EPSS** - Exploit Prediction Scoring System

### Usage

```bash
# Sync advisory database (offline-first)
bazbom db sync

# Scan with advisory lookup
bazbom scan /path/to/project

# Generate findings report
cat .bazbom/findings/sca_findings.json
```

### Output

```json
{
  "vulnerability": {
    "id": "CVE-2021-44228",
    "severity": "CRITICAL",
    "cvss": 10.0,
    "kev": true,
    "epss": 0.975,
    "description": "Log4Shell RCE vulnerability"
  },
  "package": {
    "name": "log4j-core",
    "version": "2.14.1",
    "purl": "pkg:maven/org.apache.logging.log4j/log4j-core@2.14.1"
  },
  "fix": {
    "available": true,
    "version": "2.21.1"
  }
}
```

---

## Performance Optimizations

### Caching System

BazBOM includes intelligent caching for faster repeated scans.

```bash
# First scan (no cache)
bazbom scan /path/to/project  # 30 seconds

# Second scan (cache hit)
bazbom scan /path/to/project  # 1 second

# Disable cache for testing
BAZBOM_DISABLE_CACHE=1 bazbom scan /path/to/project
```

### Incremental Analysis

For large monorepos, use incremental analysis:

```bash
# Only scan changed dependencies
bazbom scan /path/to/monorepo --incremental
```

### Parallel Processing

BazBOM automatically uses multi-threading:
- Rayon work-stealing parallelism
- Automatic CPU detection
- Configurable thread pool
- Progress-aware batching

---

## Best Practices

### 1. Use Dependency Locking

**Maven:**
```xml
<plugin>
    <groupId>org.apache.maven.plugins</groupId>
    <artifactId>maven-dependency-plugin</artifactId>
    <executions>
        <execution>
            <goals><goal>resolve</goal></goals>
        </execution>
    </executions>
</plugin>
```

**Gradle:**
```kotlin
dependencyLocking {
    lockAllConfigurations()
}
```

**Bazel:**
```python
# Use maven_install.json (rules_jvm_external)
# Always commit maven_install.json
```

### 2. Pin Versions

Avoid version ranges - always use exact versions:

```xml
<!-- Good -->
<dependency>
    <groupId>org.springframework</groupId>
    <artifactId>spring-core</artifactId>
    <version>5.3.31</version>
</dependency>

<!-- Bad -->
<dependency>
    <groupId>org.springframework</groupId>
    <artifactId>spring-core</artifactId>
    <version>[5.0,6.0)</version>
</dependency>
```

### 3. Regular Scans

Integrate BazBOM into CI/CD:

```yaml
# GitHub Actions example
- name: Scan dependencies
  run: bazbom scan . --policy=production.yml
```

### 4. Policy Enforcement

Use policies to gate builds:

```yaml
# bazbom.toml
[policy]
severity_threshold = "HIGH"
kev_policy = "block"
epss_threshold = 0.5

[policy.licenses]
allowed = ["MIT", "Apache-2.0", "BSD-3-Clause"]
denied = ["GPL-3.0", "AGPL-3.0"]
```

---

## Troubleshooting

### Maven Issues

**Problem:** Dependencies not resolved
```bash
# Ensure Maven installed and pom.xml valid
mvn validate

# Check effective POM
mvn help:effective-pom
```

**Problem:** Parent POM not found
- Ensure parent POM is in local repository
- Run `mvn install` on parent first

### Gradle Issues

**Problem:** Configuration not resolved
```bash
# List configurations
gradle dependencies

# Resolve specific configuration
gradle dependencies --configuration implementation
```

**Problem:** Kotlin DSL not parsed
- Ensure Gradle 6.0+ for full Kotlin DSL support
- Check `build.gradle.kts` syntax

### Bazel Issues

**Problem:** Query timeouts
```bash
# Increase query timeout
bazel query --query_timeout=300 'kind("java_.*", //...)'
```

**Problem:** Large monorepo slow
```bash
# Use incremental analysis
bazbom scan . --incremental

# Use target filtering
bazbom scan . --target //my/package:*
```

---

## Future Enhancements

### Scale & Performance
- [ ] Memory optimization for 50K+ target Bazel monorepos
- [ ] Profile-guided optimization (PGO)
- [ ] Advanced query caching

### Distribution
- [ ] Windows binary optimization
- [ ] GitHub Marketplace listing
- [ ] Homebrew bottles

---

## References

- [Maven Documentation](https://maven.apache.org/guides/)
- [Gradle User Manual](https://docs.gradle.org/)
- [Bazel Build Encyclopedia](https://bazel.build/reference/be/overview)
- [rules_jvm_external](https://github.com/bazelbuild/rules_jvm_external)
- [SPDX Specification 2.3](https://spdx.github.io/spdx-spec/v2.3/)
- [CycloneDX Specification 1.5](https://cyclonedx.org/docs/1.5/)

---

**Last Updated:** 2025-11-04  
**Version:** 0.5.1  
**Focus:** JVM Ecosystems Only (Java, Kotlin, Scala)
