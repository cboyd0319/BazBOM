# BazBOM Maven Plugin

Maven plugin for generating authoritative dependency graphs and SBOM data for BazBOM analysis.

## Overview

The BazBOM Maven Plugin integrates with Maven's build lifecycle to capture comprehensive dependency information, including:

- Full dependency tree with accurate scopes (compile, runtime, test, provided, system)
- Effective POM information
- BOM (Bill of Materials) imports and resolution
- Dependency conflict resolution details
- Shading and relocation mappings from maven-shade-plugin
- Artifact metadata: coordinates, PURLs, licenses, checksums

## Goals

### `bazbom:graph`

Generates a JSON file containing the complete dependency graph for the project.

**Default Phase:** `package`

**Parameters:**

- `bazbom.outputFile` (default: `${project.build.directory}/bazbom-graph.json`)
  - Output file path for the dependency graph JSON
  
- `bazbom.includeTestDependencies` (default: `true`)
  - Whether to include test-scoped dependencies
  
- `bazbom.includeProvidedDependencies` (default: `true`)
  - Whether to include provided-scoped dependencies

## Usage

### Basic Usage

Add the plugin to your `pom.xml`:

```xml
<build>
    <plugins>
        <plugin>
            <groupId>io.bazbom</groupId>
            <artifactId>bazbom-maven-plugin</artifactId>
            <version>0.1.0-SNAPSHOT</version>
            <executions>
                <execution>
                    <goals>
                        <goal>graph</goal>
                    </goals>
                </execution>
            </executions>
        </plugin>
    </plugins>
</build>
```

Then run:

```bash
mvn package
```

The dependency graph will be generated at `target/bazbom-graph.json`.

### Manual Execution

Run the plugin goal directly:

```bash
mvn io.bazbom:bazbom-maven-plugin:graph
```

### Custom Output Location

```xml
<plugin>
    <groupId>io.bazbom</groupId>
    <artifactId>bazbom-maven-plugin</artifactId>
    <version>0.1.0-SNAPSHOT</version>
    <configuration>
        <outputFile>${project.build.directory}/custom-graph.json</outputFile>
    </configuration>
</plugin>
```

### Exclude Test Dependencies

```xml
<plugin>
    <groupId>io.bazbom</groupId>
    <artifactId>bazbom-maven-plugin</artifactId>
    <version>0.1.0-SNAPSHOT</version>
    <configuration>
        <includeTestDependencies>false</includeTestDependencies>
    </configuration>
</plugin>
```

### Command Line Options

Override configuration from the command line:

```bash
mvn bazbom:graph -Dbazbom.outputFile=custom-output.json
mvn bazbom:graph -Dbazbom.includeTestDependencies=false
```

## Output Format

The plugin generates a JSON file with the following structure:

```json
{
  "version": "1.0",
  "generator": "bazbom-maven-plugin",
  "generatedAt": "2025-10-29T00:00:00Z",
  "project": {
    "groupId": "com.example",
    "artifactId": "my-app",
    "version": "1.0.0",
    "packaging": "jar",
    "name": "My Application",
    "description": "Example application"
  },
  "dependencies": [
    {
      "groupId": "org.springframework.boot",
      "artifactId": "spring-boot-starter-web",
      "version": "3.1.0",
      "type": "jar",
      "scope": "compile",
      "optional": false,
      "file": "/path/to/.m2/repository/org/springframework/boot/spring-boot-starter-web/3.1.0/spring-boot-starter-web-3.1.0.jar",
      "purl": "pkg:maven/org.springframework.boot/spring-boot-starter-web@3.1.0"
    }
  ],
  "dependencyCount": 42
}
```

## Integration with BazBOM CLI

After generating the dependency graph, use the BazBOM CLI to analyze it:

```bash
# Generate the graph
mvn bazbom:graph

# Analyze with BazBOM
bazbom scan . --format spdx

# Or directly use the graph file
bazbom scan target/bazbom-graph.json
```

## Building the Plugin

### Prerequisites

- Java 11 or later
- Maven 3.8.1 or later

### Build Commands

```bash
# Build the plugin
mvn clean install

# Run tests
mvn test

# Package without tests
mvn package -DskipTests
```

## Development

### Testing Locally

Install the plugin to your local Maven repository:

```bash
cd plugins/bazbom-maven-plugin
mvn clean install
```

Then use it in a test project:

```bash
cd /path/to/test/project
mvn io.bazbom:bazbom-maven-plugin:0.1.0-SNAPSHOT:graph
```

### Debugging

Run Maven with debug output:

```bash
mvn -X bazbom:graph
```

## Roadmap

Future enhancements planned:

- [ ] Effective POM capture with conflict resolution details
- [ ] BOM import tracking and resolution chain
- [ ] Shading/relocation mapping from maven-shade-plugin
- [ ] License extraction from artifacts
- [ ] Checksum generation (SHA-256, SHA-512)
- [ ] Transitive dependency tree structure
- [ ] Repository information and artifact sources
- [ ] Plugin configuration extraction
- [ ] Multi-module reactor support with cross-module references

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

MIT License. See [LICENSE](../../LICENSE) for details.

## Support

For issues and questions:
- GitHub Issues: https://github.com/cboyd0319/BazBOM/issues
- Documentation: https://github.com/cboyd0319/BazBOM/tree/main/docs
