# Apache Ant Project Example

This is an example Java project that uses Apache Ant as its build system.

## Build System Detection

BazBOM automatically detects this as an Ant project by finding the `build.xml` file:

```bash
bazbom scan .
# Output: Detected build system: Ant
```

## Project Structure

```
ant_project/
├── build.xml          # Ant build configuration
├── src/
│   └── main/
│       └── java/
│           └── com/
│               └── example/
│                   └── Main.java
├── lib/              # Dependencies (JARs)
└── README.md
```

## Building with Ant

```bash
# Compile the project
ant compile

# Create JAR file
ant jar

# Run the application
ant run

# Clean build artifacts
ant clean

# Run all targets
ant all
```

## Dependency Management

Ant projects typically manage dependencies in one of several ways:

1. **Manual JAR Management**: Place JARs in `lib/` directory
2. **Apache Ivy**: Ivy integration for dependency resolution (recommended)
3. **Maven Ant Tasks**: Use Maven dependency management from Ant

## BazBOM Support

BazBOM supports Ant projects with:

- ✅ Build system detection via `build.xml`
- ✅ JAR dependency scanning in `lib/` directory
- ✅ SBOM generation for Ant-based applications
- 🔄 Ivy integration (planned)
- 🔄 Maven Ant Tasks integration (planned)

## Notes

Ant is one of the oldest JVM build tools, widely used in legacy enterprise Java projects. While newer build tools like Maven and Gradle have largely superseded Ant, many production systems still rely on Ant builds.
