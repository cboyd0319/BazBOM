# BazBOM Reachability Analyzer

ASM-based bytecode reachability analysis tool for BazBOM.

## Overview

This tool analyzes Java bytecode to determine which methods and classes are reachable from application entrypoints. This enables more accurate vulnerability assessment by identifying which vulnerable code paths are actually reachable from the application.

## Features

- **Bytecode Analysis**: Uses ASM library to parse and analyze Java bytecode
- **Call Graph Generation**: Builds a call graph from entrypoints to identify reachable methods
- **Auto-Detection**: Automatically detects main methods and public constructors as entrypoints
- **JAR Support**: Analyzes classes from JAR files, directories, and individual class files
- **JSON Output**: Produces structured JSON output with reachable methods, classes, and packages

## Building

Build the fat JAR using Maven:

```bash
cd tools/reachability
mvn clean package
```

This produces `target/bazbom-reachability-0.1.0-SNAPSHOT.jar` (~690KB).

## Usage

Run the JAR with the following arguments:

```bash
java -jar bazbom-reachability-0.1.0-SNAPSHOT.jar \
  --classpath "/path/to/app.jar:/path/to/lib1.jar:/path/to/lib2.jar" \
  --entrypoints "com.example.Main.main" \
  --output reachability.json
```

### Arguments

- `--classpath`: Colon-separated (or semicolon on Windows) list of JAR files, directories, or class files to analyze
- `--entrypoints`: Comma-separated list of entrypoints (optional; auto-detects if empty)
- `--output`: Path to output JSON file (default: `reachability.json`)

### Entrypoint Format

Entrypoints can be specified as:
- `com.example.Main.main` - specific method
- `com.example.Main` - all methods in class

If no entrypoints are specified, the tool auto-detects:
- `public static void main(String[])` methods
- Public constructors

## Output Format

The tool produces JSON output with the following structure:

```json
{
  "tool": "bazbom-reachability",
  "version": "0.1.0",
  "classpath": "/path/to/app.jar",
  "entrypoints": "",
  "detectedEntrypoints": [
    "com.example.Main.main([Ljava/lang/String;)V"
  ],
  "reachableMethods": [
    "com.example.Main.main([Ljava/lang/String;)V",
    "com.example.Utils.helper()V",
    "java.lang.System.out()Ljava/io/PrintStream;"
  ],
  "reachableClasses": [
    "com.example.Main",
    "com.example.Utils",
    "java.lang.System",
    "java.io.PrintStream"
  ],
  "reachablePackages": [
    "com.example",
    "java.lang",
    "java.io"
  ]
}
```

## Example

Analyze a simple JAR file:

```bash
java -jar bazbom-reachability-0.1.0-SNAPSHOT.jar \
  --classpath "myapp.jar" \
  --output reachability.json
```

Output:
```
[reachability] Starting analysis
[reachability] Classpath: myapp.jar
[reachability] Entrypoints: (auto-detect)
[reachability] Loading 1 classpath entries
[reachability] Loaded 31 classes
[reachability] Found 29 entrypoints
[reachability] Analysis complete
[reachability] Reachable methods: 267
[reachability] Reachable classes: 91
[reachability] Output: reachability.json
```

## Testing

Run the test suite:

```bash
mvn test
```

## Integration with BazBOM

The BazBOM CLI invokes this tool when the `--reachability` flag is used:

```bash
bazbom scan . --reachability --format spdx
```

The CLI:
1. Detects the build system (Maven, Gradle, or Bazel)
2. Extracts the runtime classpath
3. Invokes the reachability analyzer
4. Tags vulnerabilities as reachable/unreachable based on the analysis

## Performance

- **Small projects** (<100 classes): <1 second
- **Medium projects** (100-1000 classes): 1-5 seconds
- **Large projects** (1000+ classes): 5-30 seconds

Performance can be improved through:
- Caching call graph results
- Limiting analysis depth
- Filtering by package prefixes

## Limitations

- **Dynamic dispatch**: Conservative analysis includes all potential targets
- **Reflection**: Methods invoked via reflection are not tracked
- **Native code**: JNI calls are not analyzed
- **Class loading**: Dynamically loaded classes are not included

## Future Enhancements

- [ ] OPAL integration for more precise analysis
- [ ] Reflection detection and tracking
- [ ] Persistent cache for call graph results
- [ ] Configurable depth limits
- [ ] Package filtering options
- [ ] Method-level trace output

## Dependencies

- ASM 9.7 - Bytecode analysis
- Gson 2.11.0 - JSON serialization
- JUnit 4.13.2 - Testing

## License

MIT License - See main repository LICENSE file
