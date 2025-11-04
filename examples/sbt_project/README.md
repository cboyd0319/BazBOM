# sbt (Scala Build Tool) Project Example

This is an example Scala project that uses sbt as its build system.

## Build System Detection

BazBOM automatically detects this as an sbt project by finding `build.sbt` or `project/build.properties`:

```bash
bazbom scan .
# Output: Detected build system: Sbt
```

## Project Structure

```
sbt_project/
├── build.sbt              # sbt build configuration
├── project/
│   └── build.properties   # sbt version configuration
├── src/
│   ├── main/
│   │   └── scala/
│   │       └── com/
│   │           └── example/
│   │               └── Main.scala
│   └── test/
│       └── scala/
└── README.md
```

## Building with sbt

```bash
# Compile the project
sbt compile

# Run the application
sbt run

# Run tests
sbt test

# Create JAR file
sbt package

# Create fat JAR (assembly)
sbt assembly

# Clean build artifacts
sbt clean

# Interactive REPL
sbt console
```

## About sbt

sbt (Scala Build Tool) is the de facto standard build tool for Scala projects. Key features:

- **Incremental Compilation**: Only recompiles changed files and dependencies
- **Interactive Shell**: Run commands and tests interactively
- **Dependency Management**: Ivy-based (compatible with Maven repositories)
- **Scala-Based DSL**: Build definitions written in Scala
- **Multi-Project Support**: Manage multiple subprojects in one build
- **Plugin Ecosystem**: Rich plugin system for extending functionality

## Dependency Management

sbt uses Ivy for dependency resolution, but is fully compatible with Maven repositories:

```scala
libraryDependencies ++= Seq(
  "org.slf4j" % "slf4j-api" % "1.7.36",
  "ch.qos.logback" % "logback-classic" % "1.2.11",
  
  // Scala libraries use %% for automatic Scala version matching
  "org.scalatest" %% "scalatest" % "3.2.15" % Test
)
```

The `%%` operator automatically appends the Scala binary version to the artifact ID, ensuring compatibility.

## BazBOM Support

BazBOM supports sbt projects with:

- ✅ Build system detection via `build.sbt`
- ✅ Detection via `project/build.properties`
- ✅ Ivy/Maven coordinate dependency tracking
- ✅ SBOM generation for Scala applications
- 🔄 sbt dependency extraction (planned)
- 🔄 Multi-project build support (planned)

## Common sbt Commands

- `sbt compile` - Compile main sources
- `sbt test:compile` - Compile test sources
- `sbt test` - Run all tests
- `sbt run` - Run the main class
- `sbt package` - Create JAR file
- `sbt assembly` - Create fat JAR with dependencies
- `sbt clean` - Remove generated files
- `sbt console` - Start Scala REPL with project classpath
- `sbt ~compile` - Continuously compile on file changes (watch mode)

## Notes

sbt is the most popular build tool for Scala projects and provides excellent support for mixed Java/Scala codebases. It's particularly well-suited for large-scale Scala applications and is used by companies like Twitter, LinkedIn, and Apple.
