# Gradle Kotlin Example

This is an example Spring Boot application written in Kotlin used to test and demonstrate BazBOM's Gradle support.

## Project Details

- **Build System**: Gradle (Kotlin DSL)
- **Language**: Kotlin 1.9.21
- **Framework**: Spring Boot 3.2.0
- **Java Version**: 17
- **Key Dependencies**:
  - Spring Boot Starter Web
  - Spring Boot Starter Security
  - Spring Boot Starter Data JPA
  - Kotlin Standard Library
  - Jackson Kotlin Module
  - H2 Database
  - Guava
  - Apache Commons Lang3
  - Kotlin Logging

## Testing BazBOM with this Project

### Scan for Dependencies

```bash
cd examples/gradle_kotlin
bazbom scan .
```

Expected output:
- Detects Gradle as the build system
- Extracts all dependencies from build.gradle.kts
- Generates SBOM in SPDX 2.3 format
- Scans for vulnerabilities in dependencies

### Generate SBOM

```bash
bazbom scan . --output gradle-sbom.json
```

### Include Test Dependencies

```bash
bazbom scan . --include-test
```

This will include JUnit and other test-scoped dependencies in the SBOM.

### Expected Results

- **Direct Dependencies**: ~15 (defined in build.gradle.kts)
- **Transitive Dependencies**: ~60-90 (pulled by Spring Boot, Kotlin, and other libraries)
- **Total Dependencies**: ~75-105

## Verification Checklist

- [ ] BazBOM detects Gradle correctly
- [ ] All direct dependencies are captured
- [ ] All transitive dependencies are captured
- [ ] Dependency configurations are preserved (implementation, runtimeOnly, testImplementation)
- [ ] SBOM is valid SPDX 2.3 JSON
- [ ] Vulnerability scanning works
- [ ] License information is extracted
- [ ] Version information is accurate
- [ ] Kotlin-specific dependencies are properly handled

## Build the Project (Optional)

```bash
# Compile and package
./gradlew build

# Run the application
./gradlew bootRun

# Or run the JAR
java -jar build/libs/gradle-kotlin-example-1.0.0.jar
```

## Common Gradle Commands for Testing

```bash
# List dependencies
./gradlew dependencies

# List runtime dependencies only
./gradlew dependencies --configuration runtimeClasspath

# List compile dependencies only
./gradlew dependencies --configuration compileClasspath

# Build info
./gradlew properties
```

## Gradle Wrapper

This project uses the Gradle wrapper for reproducible builds:

```bash
# Generate wrapper (if needed)
gradle wrapper --gradle-version 8.5

# Use wrapper
./gradlew build
```
