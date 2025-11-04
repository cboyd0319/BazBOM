# Apache Buildr Project Example

This is an example Java project that uses Apache Buildr as its build system.

## Build System Detection

BazBOM automatically detects this as a Buildr project by finding the `buildfile`:

```bash
bazbom scan .
# Output: Detected build system: Buildr
```

## Project Structure

```
buildr_project/
├── buildfile          # Buildr build configuration (Ruby DSL)
├── src/
│   └── main/
│       └── java/
│           └── com/
│               └── example/
│                   └── Main.java
└── README.md
```

## Building with Buildr

```bash
# Compile the project
buildr compile

# Run tests
buildr test

# Create JAR package
buildr package

# Clean build artifacts
buildr clean

# Build and install to local repository
buildr install
```

## About Buildr

Apache Buildr is a Ruby-based build system for Java projects that emphasizes:
- Ruby DSL for build scripts (instead of XML)
- Maven-compatible dependency management
- Convention over configuration
- Fast incremental builds

## Dependency Management

Buildr uses Maven coordinates for dependency management:

```ruby
compile.with(
  'org.slf4j:slf4j-api:jar:1.7.36',
  'ch.qos.logback:logback-classic:jar:1.2.11'
)
```

Dependencies are automatically downloaded from Maven Central and other configured repositories.

## BazBOM Support

BazBOM supports Buildr projects with:

- ✅ Build system detection via `buildfile`
- ✅ Detection via `Rakefile` with Buildr markers
- ✅ Maven coordinate dependency tracking
- ✅ SBOM generation for Buildr-based applications
- 🔄 Buildr dependency extraction (planned)

## Notes

Buildr provides a more expressive alternative to Ant while maintaining compatibility with Maven repositories. It's particularly popular in projects where Ruby expertise is available.
