# BazBOM Gradle Plugin

Gradle plugin for generating authoritative dependency graphs and SBOM data for BazBOM analysis.

## Overview

The BazBOM Gradle Plugin integrates with Gradle's build lifecycle to capture comprehensive dependency information, including:

- Dependency graphs per configuration and variant
- Android build variants and flavors (via Variant API)
- Shadow plugin detection for shaded JARs
- Artifact metadata: coordinates, PURLs, configurations

## Plugin ID

```
io.bazbom.gradle-plugin
```

## Tasks

### `bazbomGraph`

Generates a JSON file containing the complete dependency graph for all configurations.

**Output:** `build/bazbom-graph.json`

### `bazbomSbom`

Generates an SBOM from the dependency graph (placeholder - will invoke BazBOM CLI).

**Depends on:** `bazbomGraph`

### `bazbomFindings`

Generates a security findings report (placeholder - will invoke BazBOM CLI).

**Depends on:** `bazbomGraph`

## Usage

### Apply the Plugin

**Groovy DSL (`build.gradle`):**

```groovy
plugins {
    id 'io.bazbom.gradle-plugin' version '6.5.0'
}
```

**Kotlin DSL (`build.gradle.kts`):**

```kotlin
plugins {
    id("io.bazbom.gradle-plugin") version "6.5.0"
}
```

### Configuration

Configure the plugin (optional):

**Groovy DSL:**

```groovy
bazbom {
    includeTestConfigurations = true
    includeAndroidVariants = true
    analyzeShadow = true
}
```

**Kotlin DSL:**

```kotlin
bazbom {
    includeTestConfigurations.set(true)
    includeAndroidVariants.set(true)
    analyzeShadow.set(true)
}
```

### Run Tasks

Generate the dependency graph:

```bash
./gradlew bazbomGraph
```

Generate SBOM:

```bash
./gradlew bazbomSbom
```

Generate findings:

```bash
./gradlew bazbomFindings
```

## Output Format

The plugin generates a JSON file with the following structure:

```json
{
  "version": "1.0",
  "generator": "bazbom-gradle-plugin",
  "generatedAt": "2025-10-29T00:00:00Z",
  "project": {
    "name": "my-app",
    "group": "com.example",
    "version": "1.0.0",
    "path": ":app"
  },
  "configurations": [
    {
      "name": "runtimeClasspath",
      "description": "Runtime classpath of source set 'main'",
      "dependencies": [
        {
          "group": "com.google.guava",
          "name": "guava",
          "version": "32.1.3-jre",
          "configuration": "runtimeClasspath",
          "purl": "pkg:maven/com.google.guava/guava@32.1.3-jre"
        }
      ]
    }
  ],
  "dependencyCount": 42
}
```

## Android Support

The plugin automatically detects Android projects and captures variant-specific dependencies when `includeAndroidVariants` is enabled.

## Shadow Plugin Support

When the Shadow plugin is detected and `analyzeShadow` is enabled, the plugin captures shading and relocation information.

## Building the Plugin

### Prerequisites

- Java 11 or later
- Gradle 7.0 or later

### Build Commands

```bash
# Build the plugin
./gradlew build

# Run tests
./gradlew test

# Publish to local Maven repository
./gradlew publishToMavenLocal
```

## Development

### Testing Locally

Publish the plugin to your local Maven repository:

```bash
cd plugins/bazbom-gradle-plugin
./gradlew publishToMavenLocal
```

Then use it in a test project:

**settings.gradle.kts:**

```kotlin
pluginManagement {
    repositories {
        mavenLocal()
        gradlePluginPortal()
    }
}
```

**build.gradle.kts:**

```kotlin
plugins {
    id("io.bazbom.gradle-plugin") version "6.5.0"
}
```

### Debugging

Run Gradle with debug output:

```bash
./gradlew bazbomGraph --debug
```

Or with info logging:

```bash
./gradlew bazbomGraph --info
```

## Roadmap

Future enhancements planned:

- [ ] Android Variant API integration for flavor/build type graphs
- [ ] Shadow plugin configuration parsing for relocation maps
- [ ] Dependency insight integration for conflict resolution
- [ ] License extraction from artifacts
- [ ] Checksum generation (SHA-256, SHA-512)
- [ ] Version catalog support
- [ ] Composite build support
- [ ] Configuration caching compatibility

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT License. See [LICENSE](../../LICENSE) for details.

## Support

For issues and questions:
- GitHub Issues: https://github.com/cboyd0319/BazBOM/issues
- Documentation: https://github.com/cboyd0319/BazBOM/tree/main/docs
