# bazbom-js-reachability

JavaScript/TypeScript Reachability Analysis for BazBOM (v6.3.0)

## Overview

This crate provides static analysis capabilities to determine which code paths in JavaScript and TypeScript projects are actually reachable from entrypoints. This is crucial for vulnerability analysis - knowing whether vulnerable code is actually used by your application.

## Status: In Progress (v6.3.0)

**✅ Completed:**
- Core architecture designed
- Data models and error types defined
- Call graph structure with DFS reachability
- Entrypoint detection logic
- Module resolution algorithm (Node.js-style)
- Main analyzer orchestration

**🚧 In Progress:**
- Resolving SWC dependency conflicts with existing workspace dependencies
- The crate structure is complete but has compilation issues due to SWC/serde version incompatibility

**📋 TODO:**
- Finalize SWC integration or implement alternative parser
- Add comprehensive test coverage
- Integrate with existing SCA pipeline
- SARIF output with reachability information

## Architecture

### Components

1. **AST Parser** (`ast_parser.rs`)
   - Parses JavaScript/TypeScript files using SWC
   - Extracts function definitions and call sites
   - Supports .js, .ts, .jsx, .tsx files

2. **Call Graph** (`call_graph.rs`)
   - Directed graph structure using `petgraph`
   - DFS-based reachability analysis
   - Call chain tracking for vulnerability reports

3. **Entrypoint Detector** (`entrypoints.rs`)
   - Detects main entry (package.json)
   - HTTP handlers (Express, Fastify)
   - Test files
   - Exported functions

4. **Module Resolver** (`module_resolver.rs`)
   - Node.js-style module resolution
   - Handles relative imports (./foo, ../bar)
   - node_modules resolution
   - Extension resolution (.js, .ts, etc.)
   - Index file support

5. **Analyzer** (`analyzer.rs`)
   - Main orchestration
   - File discovery
   - Graph building
   - Report generation

## Usage (Planned)

```rust
use bazbom_js_reachability::JsReachabilityAnalyzer;
use std::path::Path;

let mut analyzer = JsReachabilityAnalyzer::new();
let report = analyzer.analyze(Path::new("./myproject"))?;

println!("Reachable functions: {}", report.reachable_functions.len());
println!("Unreachable functions: {}", report.unreachable_functions.len());

// Check if a vulnerability is reachable
let vuln = analyzer.check_vulnerability_reachability(
    "express",
    "Router.use"
);

if let Some(v) = vuln {
    if v.reachable {
        println!("⚠️  Vulnerability is REACHABLE!");
        if let Some(chain) = v.call_chain {
            println!("Call chain: {}", chain.join(" → "));
        }
    } else {
        println!("✅ Vulnerability is NOT reachable");
    }
}
```

## Output Example

```
🔍 Analyzing JavaScript project at /path/to/project

📂 Discovered:
  - 45 source files
  - 123 functions
  - 8 entrypoints

🕸️  Building call graph...
  - 98 reachable functions
  - 25 unreachable functions

🔴 REACHABLE VULNERABILITY
  CVE-2024-1234 in express@4.17.0

  Vulnerable function: express.Router.use()

  Call chain:
    src/index.js:main() [line 10]
      → src/routes/api.js:setupRoutes() [line 5]
        → express.Router.use() [line 15]  ← VULNERABLE!

  EPSS: 78.5%
  Fix: Upgrade to express@4.18.0

🟢 UNREACHABLE VULNERABILITY
  CVE-2024-5678 in lodash@4.17.0

  Vulnerable function: lodash.template()

  Status: NOT CALLED by your application ✓
```

## Integration with BazBOM

Once completed, this crate will integrate with:

1. **bazbom-polyglot** - JavaScript/TypeScript scanning
2. **bazbom-advisories** - Vulnerability database
3. **bazbom-formats** - SARIF output with reachability
4. **bazbom-reports** - Enhanced vulnerability reports

## Technical Details

### Reachability Algorithm

1. **File Discovery:** Scan project for .js/.ts files (exclude node_modules)
2. **AST Parsing:** Parse each file to extract functions and calls
3. **Graph Construction:** Build directed call graph
4. **Entrypoint Identification:** Detect entrypoints automatically
5. **DFS Traversal:** Mark all functions reachable from entrypoints
6. **Vulnerability Mapping:** Check if vulnerable functions are reachable
7. **Report Generation:** Produce actionable results

### Handling Dynamic Code

JavaScript is highly dynamic, so we take a conservative approach:

- **Dynamic imports:** `import(variable)` - Mark as potentially reachable
- **eval():** Mark entire module as reachable
- **Computed property access:** `obj[variable]()` - Conservative tracking
- **Callbacks:** Track callback registrations

## Dependencies

- `swc_common`, `swc_ecma_ast`, `swc_ecma_parser`, `swc_ecma_visit` - AST parsing
- `petgraph` - Call graph data structure
- `walkdir` - File system traversal
- `serde`, `serde_json` - Serialization

## Known Limitations (by Design)

Per the v6.3.0 roadmap, these are intentionally out of scope:

- ❌ Webpack/Vite dynamic imports with variables
- ❌ eval() and new Function() analysis
- ❌ Monkey-patching detection
- ❌ Browser-only code
- ❌ Minified/bundled code analysis

These may be addressed in future versions.

## Development Status

This is part of the BazBOM v6.2-v6.5 Polyglot Parity Initiative:

- **v6.2.0** ✅ - Polyglot Upgrade Intelligence (COMPLETED)
- **v6.3.0** 🚧 - JavaScript/TypeScript Reachability (IN PROGRESS)
- **v6.4.0** 📋 - Python + Go Reachability
- **v6.5.0** 📋 - Rust + Ruby + PHP Reachability

## Contributing

This crate is under active development. The architecture is complete but needs:

1. Resolution of SWC dependency conflicts
2. Comprehensive test coverage
3. Performance optimization
4. Integration testing with real-world projects

## License

MIT
