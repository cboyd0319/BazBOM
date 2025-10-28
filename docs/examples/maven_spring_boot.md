# Maven Spring Boot Example

This is an example Spring Boot application used to test and demonstrate BazBOM's Maven support.

## Project Details

- **Build System**: Maven
- **Framework**: Spring Boot 3.2.0
- **Java Version**: 17
- **Key Dependencies**:
  - Spring Boot Starter Web
  - Spring Boot Starter Security
  - Spring Boot Starter Data JPA
  - H2 Database
  - Jackson (JSON processing)
  - Guava
  - Apache Commons Lang3

## Testing BazBOM with this Project

### Scan for Dependencies

```bash
cd examples/maven_spring_boot
bazbom scan .
```

Expected output:
- Detects Maven as the build system
- Extracts all dependencies from pom.xml
- Generates SBOM in SPDX 2.3 format
- Scans for vulnerabilities in dependencies

### Generate SBOM

```bash
bazbom scan . --output maven-sbom.json
```

### Include Test Dependencies

```bash
bazbom scan . --include-test
```

This will include JUnit and other test-scoped dependencies in the SBOM.

### Expected Results

- **Direct Dependencies**: ~10 (defined in pom.xml)
- **Transitive Dependencies**: ~50-80 (pulled by Spring Boot and other libraries)
- **Total Dependencies**: ~60-90

## Verification Checklist

- [ ] BazBOM detects Maven correctly
- [ ] All direct dependencies are captured
- [ ] All transitive dependencies are captured
- [ ] Dependency scopes are preserved (compile, runtime, test)
- [ ] SBOM is valid SPDX 2.3 JSON
- [ ] Vulnerability scanning works
- [ ] License information is extracted
- [ ] Version information is accurate

## Build the Project (Optional)

```bash
# Compile and package
mvn clean package

# Run the application
java -jar target/maven-spring-boot-example-1.0.0.jar
```

## Common Maven Commands for Testing

```bash
# List dependencies
mvn dependency:tree

# Download sources
mvn dependency:sources

# Resolve all dependencies
mvn dependency:resolve
```

