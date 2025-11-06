# JVM Language Support

**Version:** 1.0  
**Last Updated:** 2025-11-04  
**Status:** Complete

---

## Overview

BazBOM provides comprehensive support for all JVM-based languages across Maven, Gradle, and Bazel build systems. This document details the support for Java, Kotlin, and Scala projects.

**Supported JVM Languages:**
-  Java (all versions)
-  Kotlin (1.x, 2.x)
-  Scala (2.x, 3.x)

**Supported Build Systems:**
-  Maven (3.x, 4.x)
-  Gradle (7.x, 8.x)
-  Bazel (6.x, 7.x)

---

## Java Support

### Maven

BazBOM automatically detects and analyzes Java Maven projects:

```bash
# Standard Java project
bazbom scan .

# With specific Java version
bazbom scan . --java-version 17
```

**Supported Maven plugins:**
- `maven-compiler-plugin` (all versions)
- `maven-jar-plugin`
- `maven-war-plugin`
- `maven-shade-plugin` (shaded JARs detected)
- `spring-boot-maven-plugin`

**Example pom.xml:**
```xml
<project>
  <properties>
    <maven.compiler.source>17</maven.compiler.source>
    <maven.compiler.target>17</maven.compiler.target>
  </properties>
  
  <dependencies>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-web</artifactId>
      <version>3.2.0</version>
    </dependency>
  </dependencies>
</project>
```

### Gradle

BazBOM supports Java projects with both Groovy and Kotlin DSL:

```bash
# Gradle project (Groovy or Kotlin DSL)
bazbom scan .

# Include test dependencies
bazbom scan . --include-test
```

**Example build.gradle.kts:**
```kotlin
plugins {
    java
    id("org.springframework.boot") version "3.2.0"
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

dependencies {
    implementation("org.springframework.boot:spring-boot-starter-web")
    testImplementation("junit:junit:4.13.2")
}
```

### Bazel

BazBOM automatically detects Java rules in Bazel projects:

```bash
# Scan all Java targets
bazbom scan . --bazel-targets "kind(java_library, //...)"

# Scan specific target
bazbom scan . --bazel-targets "//java/com/example:app"
```

**Supported Java rules:**
- `java_library`
- `java_binary`
- `java_test`
- `java_plugin`
- `java_import`

**Example BUILD.bazel:**
```python
java_library(
    name = "lib",
    srcs = glob(["src/**/*.java"]),
    deps = [
        "@maven//:com_google_guava_guava",
        "@maven//:org_springframework_boot_spring_boot",
    ],
)

java_binary(
    name = "app",
    main_class = "com.example.Main",
    runtime_deps = [":lib"],
)
```

---

## Kotlin Support

### Maven (Kotlin)

BazBOM supports Kotlin Maven projects with the `kotlin-maven-plugin`:

```bash
# Kotlin Maven project
bazbom scan .
```

**Supported plugins:**
- `kotlin-maven-plugin`
- `kotlin-maven-allopen`
- `kotlin-maven-noarg`

**Example pom.xml:**
```xml
<project>
  <properties>
    <kotlin.version>1.9.21</kotlin.version>
  </properties>
  
  <dependencies>
    <dependency>
      <groupId>org.jetbrains.kotlin</groupId>
      <artifactId>kotlin-stdlib</artifactId>
      <version>${kotlin.version}</version>
    </dependency>
    <dependency>
      <groupId>org.springframework.boot</groupId>
      <artifactId>spring-boot-starter-web</artifactId>
    </dependency>
  </dependencies>
  
  <build>
    <plugins>
      <plugin>
        <groupId>org.jetbrains.kotlin</groupId>
        <artifactId>kotlin-maven-plugin</artifactId>
        <version>${kotlin.version}</version>
      </plugin>
    </plugins>
  </build>
</project>
```

### Gradle (Kotlin)

BazBOM fully supports Kotlin projects with both Groovy and Kotlin DSL:

```bash
# Kotlin Gradle project
bazbom scan .

# Scan example: docs/examples/gradle_kotlin
bazbom scan docs/examples/gradle_kotlin
```

**Supported plugins:**
- `kotlin("jvm")`
- `kotlin("plugin.spring")`
- `kotlin("plugin.jpa")`
- `kotlin("kapt")` (annotation processing)

**Example build.gradle.kts:**
```kotlin
plugins {
    kotlin("jvm") version "1.9.21"
    kotlin("plugin.spring") version "1.9.21"
    id("org.springframework.boot") version "3.2.0"
}

dependencies {
    implementation("org.jetbrains.kotlin:kotlin-stdlib")
    implementation("org.jetbrains.kotlin:kotlin-reflect")
    implementation("org.springframework.boot:spring-boot-starter-web")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin")
}
```

### Bazel (Kotlin)

BazBOM supports Kotlin Bazel projects using `rules_kotlin`:

```bash
# Query all Kotlin targets
bazbom scan . --bazel-query-kotlin

# Scan specific Kotlin target
bazbom scan . --bazel-targets "//kotlin/com/example:app"
```

**Supported Kotlin rules:**
- `kotlin_library`
- `kt_jvm_library`
- `kt_jvm_binary`
- `kt_jvm_test`
- `kt_jvm_import`

**Example BUILD.bazel:**
```python
load("@rules_kotlin//kotlin:jvm.bzl", "kt_jvm_library", "kt_jvm_binary")

kt_jvm_library(
    name = "lib",
    srcs = glob(["src/**/*.kt"]),
    deps = [
        "@maven//:org_jetbrains_kotlin_kotlin_stdlib",
        "@maven//:com_google_guava_guava",
    ],
)

kt_jvm_binary(
    name = "app",
    main_class = "com.example.MainKt",
    runtime_deps = [":lib"],
)
```

**WORKSPACE configuration:**
```python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# rules_kotlin
http_archive(
    name = "rules_kotlin",
    sha256 = "...",
    url = "https://github.com/bazelbuild/rules_kotlin/releases/download/v1.9.1/rules_kotlin-v1.9.1.tar.gz",
)

load("@rules_kotlin//kotlin:repositories.bzl", "kotlin_repositories")
kotlin_repositories()

load("@rules_kotlin//kotlin:core.bzl", "kt_register_toolchains")
kt_register_toolchains()
```

---

## Scala Support

### Maven (Scala)

BazBOM supports Scala Maven projects with the `scala-maven-plugin`:

```bash
# Scala Maven project
bazbom scan .
```

**Supported plugins:**
- `scala-maven-plugin`
- `maven-scala-plugin`

**Example pom.xml:**
```xml
<project>
  <properties>
    <scala.version>2.13.12</scala.version>
  </properties>
  
  <dependencies>
    <dependency>
      <groupId>org.scala-lang</groupId>
      <artifactId>scala-library</artifactId>
      <version>${scala.version}</version>
    </dependency>
    <dependency>
      <groupId>com.typesafe.akka</groupId>
      <artifactId>akka-actor_2.13</artifactId>
      <version>2.8.5</version>
    </dependency>
  </dependencies>
  
  <build>
    <plugins>
      <plugin>
        <groupId>net.alchim31.maven</groupId>
        <artifactId>scala-maven-plugin</artifactId>
        <version>4.8.1</version>
      </plugin>
    </plugins>
  </build>
</project>
```

### Gradle (Scala)

BazBOM supports Scala Gradle projects:

```bash
# Scala Gradle project
bazbom scan .
```

**Supported plugins:**
- `scala`
- `java-library` (with Scala)

**Example build.gradle.kts:**
```kotlin
plugins {
    scala
}

scala {
    zincVersion.set("1.9.5")
}

dependencies {
    implementation("org.scala-lang:scala-library:2.13.12")
    implementation("com.typesafe.akka:akka-actor_2.13:2.8.5")
}
```

### Bazel (Scala)

BazBOM supports Scala Bazel projects using `rules_scala`:

```bash
# Query all Scala targets
bazbom scan . --bazel-query-scala

# Scan specific Scala target
bazbom scan . --bazel-targets "//scala/com/example:app"
```

**Supported Scala rules:**
- `scala_library`
- `scala_binary`
- `scala_test`
- `scala_import`
- `scala_macro_library`

**Example BUILD.bazel:**
```python
load("@io_bazel_rules_scala//scala:scala.bzl", "scala_library", "scala_binary")

scala_library(
    name = "lib",
    srcs = glob(["src/**/*.scala"]),
    deps = [
        "@maven//:org_scala_lang_scala_library",
        "@maven//:com_typesafe_akka_akka_actor_2_13",
    ],
)

scala_binary(
    name = "app",
    main_class = "com.example.Main",
    runtime_deps = [":lib"],
)
```

**WORKSPACE configuration:**
```python
load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

# rules_scala
http_archive(
    name = "io_bazel_rules_scala",
    sha256 = "...",
    url = "https://github.com/bazelbuild/rules_scala/releases/download/v6.5.0/rules_scala-v6.5.0.tar.gz",
)

load("@io_bazel_rules_scala//:scala_config.bzl", "scala_config")
scala_config(scala_version = "2.13.12")

load("@io_bazel_rules_scala//scala:scala.bzl", "scala_repositories")
scala_repositories()

load("@io_bazel_rules_scala//scala:toolchains.bzl", "scala_register_toolchains")
scala_register_toolchains()
```

---

## Mixed-Language Projects

BazBOM seamlessly handles projects that mix Java, Kotlin, and Scala:

### Maven Multi-Module

```bash
# Scan entire multi-module project
bazbom scan .

# Scan specific module
bazbom scan java-module/
```

**Example:**
```
project/
├── pom.xml (parent)
├── java-module/
│   ├── pom.xml (java)
│   └── src/main/java/
├── kotlin-module/
│   ├── pom.xml (kotlin)
│   └── src/main/kotlin/
└── scala-module/
    ├── pom.xml (scala)
    └── src/main/scala/
```

### Gradle Multi-Project

```bash
# Scan entire multi-project build
bazbom scan .
```

**Example settings.gradle.kts:**
```kotlin
rootProject.name = "multi-lang-project"

include("java-lib")
include("kotlin-lib")
include("scala-lib")
```

### Bazel Monorepo

```bash
# Scan all JVM targets (Java, Kotlin, Scala)
bazbom scan . --bazel-query-jvm

# Or query each language separately
bazbom scan . --bazel-targets "kind(java_library, //...)"
bazbom scan . --bazel-targets "kind(kt_jvm_library, //...)"
bazbom scan . --bazel-targets "kind(scala_library, //...)"
```

---

## Bazel Query Helpers

BazBOM provides specialized Bazel query functions for JVM languages:

### Query All JVM Targets

Queries all Java, Kotlin, and Scala targets in the workspace:

```rust
use bazbom::bazel::query_all_jvm_targets;

let targets = query_all_jvm_targets(Path::new("/workspace"))?;
// Returns: ["//java/app:lib", "//kotlin/api:service", "//scala/core:domain"]
```

### Query Kotlin-Specific Targets

```rust
use bazbom::bazel::query_kotlin_targets;

let targets = query_kotlin_targets(Path::new("/workspace"))?;
// Returns: ["//kotlin/api:service", "//kotlin/client:app"]
```

### Query Scala-Specific Targets

```rust
use bazbom::bazel::query_scala_targets;

let targets = query_scala_targets(Path::new("/workspace"))?;
// Returns: ["//scala/core:domain", "//scala/utils:helpers"]
```

### Check Rule Type

```rust
use bazbom::bazel::is_jvm_rule;

assert!(is_jvm_rule("java_library"));
assert!(is_jvm_rule("kt_jvm_library"));
assert!(is_jvm_rule("scala_library"));
assert!(!is_jvm_rule("py_binary"));  // Not JVM
```

---

## Dependency Resolution

### Maven

Uses Maven's built-in dependency resolution:
- Follows Maven Central repository conventions
- Supports `dependencyManagement` sections
- Handles transitive dependencies
- Respects dependency scopes (compile, runtime, test, provided)

### Gradle

Uses Gradle's dependency resolution engine:
- Supports Maven Central and custom repositories
- Handles configuration inheritance
- Resolves conflicts using Gradle's resolution rules
- Supports dependency constraints and platforms

### Bazel

Uses `rules_jvm_external` for Maven dependency management:
- Requires `maven_install.json` (generated by `@maven//:pin`)
- Fetches dependencies from Maven Central or custom repositories
- Deterministic dependency resolution
- Supports artifact exclusions and version overrides

---

## Language-Specific Features

### Kotlin

**Coroutines:**
```kotlin
dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")
}
```

**Multiplatform (JVM target only):**
```kotlin
kotlin {
    jvm {
        withJava()
    }
}
```

**Spring Boot + Kotlin:**
```kotlin
plugins {
    kotlin("jvm")
    kotlin("plugin.spring")
}

dependencies {
    implementation("org.springframework.boot:spring-boot-starter")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin")
}
```

### Scala

**Scala 2.x vs 3.x:**
BazBOM automatically detects Scala version from dependencies:
- Scala 2.13: `org.scala-lang:scala-library:2.13.12`
- Scala 3: `org.scala-lang:scala3-library_3:3.3.1`

**Binary-compatible artifacts:**
Scala artifacts include binary version in artifact ID:
- `akka-actor_2.13` (Scala 2.13)
- `akka-actor_3` (Scala 3)

---

## Troubleshooting

### Maven

**Issue:** Kotlin dependencies not found
```bash
# Ensure kotlin-maven-plugin is configured
mvn dependency:tree | grep kotlin
```

**Issue:** Scala dependencies not resolved
```bash
# Verify scala-maven-plugin version
mvn help:effective-pom | grep scala-maven-plugin
```

### Gradle

**Issue:** Kotlin plugin not applied
```kotlin
// Apply Kotlin plugin explicitly
plugins {
    kotlin("jvm") version "1.9.21"
}
```

**Issue:** Scala plugin missing
```groovy
plugins {
    id 'scala'
}
```

### Bazel

**Issue:** `rules_kotlin` not loaded
```python
# Verify WORKSPACE has rules_kotlin
load("@rules_kotlin//kotlin:repositories.bzl", "kotlin_repositories")
```

**Issue:** `rules_scala` not loaded
```python
# Verify WORKSPACE has rules_scala
load("@io_bazel_rules_scala//scala:scala.bzl", "scala_repositories")
```

**Issue:** `maven_install.json` not found
```bash
# Generate it first
bazel run @maven//:pin
```

---

## Examples

BazBOM includes working examples for all JVM languages and build systems:

- **Maven Java:** `examples/maven-spring-boot/`
- **Maven Kotlin:** `examples/maven-kotlin/` (if exists)
- **Gradle Kotlin:** `docs/examples/gradle_kotlin/`
- **Bazel Java:** `examples/bazel-java/` (if exists)
- **Bazel Multi-language:** See `docs/examples/bazel-monorepo-workflows.md`

---

## API Reference

### Rust API

```rust
// Bazel JVM support
use bazbom::bazel::{
    is_jvm_rule,
    query_all_jvm_targets,
    query_kotlin_targets,
    query_scala_targets,
    get_jvm_rule_query,
};

// Check if a rule is JVM-based
if is_jvm_rule("kt_jvm_library") {
    println!("This is a Kotlin JVM rule");
}

// Query all JVM targets
let all_jvm = query_all_jvm_targets(workspace_path)?;

// Query language-specific targets
let kotlin_targets = query_kotlin_targets(workspace_path)?;
let scala_targets = query_scala_targets(workspace_path)?;

// Generate custom query
let query = get_jvm_rule_query("//java/...");
```

---

## Best Practices

### Maven

1. Always specify plugin versions explicitly
2. Use `dependencyManagement` for multi-module projects
3. Keep Kotlin/Scala versions consistent across modules
4. Run `mvn dependency:tree` to verify resolution

### Gradle

1. Use Kotlin DSL for Kotlin projects (type-safe)
2. Enable version catalogs for consistent dependencies
3. Use `implementation` over `compile` (deprecated)
4. Run `./gradlew dependencies` to debug issues

### Bazel

1. Pin dependencies with `@maven//:pin`
2. Use `rules_jvm_external` for Maven dependencies
3. Keep `maven_install.json` in version control
4. Query targets before scanning: `bazel query 'kind(java_library, //...)'`

---

## Performance Considerations

### Large Kotlin Projects

- Kotlin compilation is slower than Java
- Consider using `kapt` alternatives like KSP
- Enable Gradle build cache
- Use incremental compilation

### Large Scala Projects

- Scala compilation is significantly slower
- Use Zinc incremental compiler
- Enable sbt-like compilation in Maven
- Consider parallel subproject builds

### Bazel Performance

- Bazel caching speeds up repeated scans
- Use `--bazel-targets` to scan specific targets
- Remote caching improves CI performance
- Query optimization reduces scan time

---

## Related Documentation

- [../user-guide/usage.md](../user-guide/usage.md) - General BazBOM usage guide
- [Bazel Monorepo Workflows](examples/bazel-monorepo-workflows.md)
- [Gradle Kotlin Example](examples/gradle_kotlin.md)
- [Maven Spring Boot Example](examples/maven_spring_boot.md)
- [Build System Detection](../architecture/architecture.md#build-system-detection)

---

## Support Matrix

| Language | Maven | Gradle | Bazel | Version Support |
|----------|-------|--------|-------|-----------------|
| Java     |  Full |  Full |  Full | 8, 11, 17, 21+ |
| Kotlin   |  Full |  Full |  Full | 1.x, 2.x |
| Scala    |  Full |  Full |  Full | 2.11, 2.12, 2.13, 3.x |

---

**Last Verified:** 2025-11-04  
**BazBOM Version:** 0.5.1+
