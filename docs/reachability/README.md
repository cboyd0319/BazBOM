# Reachability Analysis

BazBOM provides world-class reachability analysis across all major programming languages. Reachability analysis identifies which code paths are actually reachable from entrypoints, dramatically reducing false-positive vulnerability alerts.

## Overview

**Status**: ✅ Complete polyglot parity (v6.5.0)

Reachability analysis answers the critical question: *"Is this vulnerable code actually reachable in my application?"*

Traditional vulnerability scanners report every vulnerability in every dependency, leading to alert fatigue. BazBOM's reachability analyzers use static analysis to determine if vulnerable functions are:
1. Actually imported/used in your code
2. Reachable from program entrypoints
3. Part of active code paths vs dead code

## Supported Languages

| Language | Status | Accuracy | Analyzer | Version |
|----------|--------|----------|----------|---------|
| JavaScript/TypeScript | ✅ | ~85% | tree-sitter | v6.3.0 |
| Python | ✅ | ~80% | tree-sitter | v6.4.0 |
| Go | ✅ | ~90% | tree-sitter | v6.4.0 |
| Rust | ✅ | >98% | syn | v6.5.0 |
| Ruby | ✅ | ~75% | tree-sitter | v6.5.0 |
| PHP | ✅ | ~70% | tree-sitter | v6.5.0 |
| JVM (Java/Kotlin/Scala) | ✅ | ~85% | bytecode | v6.1.0 |

## Language-Specific Details

### JavaScript/TypeScript

**Entrypoint Detection:**
- Exported functions/classes
- Express.js routes
- Next.js API routes
- Jest/Mocha test cases

**Dynamic Code Handling:**
- `eval()` - Conservative (marks all reachable)
- `new Function()` - Conservative
- Dynamic `require()` - Best-effort resolution

**Module Resolution:**
- npm packages via `node_modules`
- Relative/absolute imports
- Monorepo workspace resolution

### Python

**Entrypoint Detection:**
- `if __name__ == "__main__"`
- Flask routes (`@app.route`)
- Django views
- FastAPI endpoints
- Click commands
- Celery tasks
- pytest test functions

**Dynamic Code Handling:**
- `eval()`, `exec()` - Conservative
- `getattr()` - Conservative
- `__import__()` - Best-effort

**Module Resolution:**
- pip packages via site-packages
- Relative imports
- Virtual environment detection

### Go

**Entrypoint Detection:**
- `func main()`
- `func Test*` (test functions)
- `func Benchmark*` (benchmarks)
- `func Example*` (examples)
- `func init()` (initializers)

**Dynamic Code Handling:**
- `reflect` package - Conservative
- No eval/exec equivalents

**Features:**
- Goroutine detection
- Interface implementation tracking

### Rust

**Entrypoint Detection:**
- `fn main()`
- `#[test]` functions
- `#[tokio::main]`
- `#[actix_web::main]`
- `#[bench]` benchmarks

**Dynamic Code Handling:**
- None - Rust is fully static

**Features:**
- Highest accuracy (>98%)
- Trait implementation tracking
- Async function detection
- Macro expansion at call sites

### Ruby

**Entrypoint Detection:**
- Rails controllers (public methods)
- Rails jobs (`perform` method)
- Rails mailers
- RSpec tests (`it`, `specify`, `example`)
- Minitest (`test_*` methods)
- Sinatra routes (`get`, `post`, etc.)
- Rake tasks

**Dynamic Code Handling:**
- `eval`, `instance_eval`, `class_eval`, `module_eval` - Conservative
- `define_method` - Conservative
- `method_missing` - Conservative
- `send`, `__send__`, `public_send` - Conservative

**Features:**
- Framework-aware detection
- Metaprogramming detection

### PHP

**Entrypoint Detection:**
- Symfony controllers (public methods)
- Laravel controllers
- Laravel jobs
- WordPress action hooks
- WordPress filter hooks
- PHPUnit tests (`test*` methods)

**Dynamic Code Handling:**
- `eval()` - Conservative
- `call_user_func()`, `call_user_func_array()` - Conservative
- Variable functions - Conservative
- `include`/`require` with variables - Conservative

**Features:**
- PSR-4 autoloading support
- Namespace detection
- Framework-aware routing

## Architecture

All reachability analyzers follow a consistent architecture:

```
┌─────────────────┐
│  Entrypoint     │ → Detect program entry points
│  Detection      │   (main, tests, routes, etc.)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  AST Parsing    │ → Parse source code into AST
│  (tree-sitter   │   using language-specific parser
│   or syn)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Call Graph     │ → Build directed graph of
│  Construction   │   function calls (using petgraph)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Reachability   │ → DFS traversal from entrypoints
│  Analysis       │   to determine reachable functions
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Vulnerability  │ → Map CVEs to reachable functions
│  Mapping        │   and generate call chains
└─────────────────┘
```

## Usage

### CLI

```bash
# Analyze a project
bazbom scan /path/to/project

# Generate reachability report
bazbom reachability analyze /path/to/project

# Check specific vulnerability
bazbom reachability check CVE-2024-1234 /path/to/project

# Export results
bazbom scan --format sarif --output results.sarif /path/to/project
```

### Programmatic (Rust)

```rust
use bazbom_reachability::javascript::analyze_js_project;
use std::path::PathBuf;

let project_root = PathBuf::from("/path/to/project");
let report = analyze_js_project(&project_root)?;

println!("Total functions: {}", report.all_functions.len());
println!("Reachable functions: {}", report.reachable_functions.len());
println!("Unreachable functions: {}", report.unreachable_functions.len());

// Check if specific function is reachable
let vuln_function_id = "src/utils.js::vulnerable_function";
if report.reachable_functions.contains(vuln_function_id) {
    if let Some(chain) = report.call_chain_to(vuln_function_id) {
        println!("Call chain: {:?}", chain);
    }
}
```

## Conservative Analysis

When dynamic code patterns are detected, BazBOM uses **conservative analysis**:

1. **Detection**: Scan for `eval()`, reflection, metaprogramming
2. **Warning**: Log that dynamic code was found
3. **Conservative Mode**: Mark all code as potentially reachable
4. **Reason**: Prioritize security (avoid false negatives)

This ensures you never miss a vulnerability due to limitations in static analysis.

## Performance

| Language | Files/sec | Functions/sec |
|----------|-----------|---------------|
| JavaScript | ~500 | ~15,000 |
| Python | ~400 | ~12,000 |
| Go | ~600 | ~20,000 |
| Rust | ~300 | ~10,000 |
| Ruby | ~350 | ~11,000 |
| PHP | ~400 | ~13,000 |

*Benchmarks on Intel i7, 16GB RAM, typical enterprise codebases*

## Output Formats

### SARIF

Reachability information is embedded in SARIF results:

```json
{
  "results": [{
    "ruleId": "CVE-2024-1234",
    "message": {
      "text": "Vulnerable function is reachable"
    },
    "properties": {
      "reachable": true,
      "callChain": ["main", "handler", "vulnerable_fn"]
    }
  }]
}
```

### JSON Report

```json
{
  "all_functions": {...},
  "reachable_functions": [...],
  "unreachable_functions": [...],
  "entrypoints": [...],
  "vulnerabilities": [{
    "cve_id": "CVE-2024-1234",
    "reachable": true,
    "call_chain": ["main", "handler", "vulnerable_fn"]
  }]
}
```

## Limitations

### Static Analysis Boundaries

- **Dynamic dispatch**: Limited precision for runtime polymorphism
- **Reflection**: Conservative fallback
- **Code generation**: Analyzed at call sites
- **External calls**: FFI/C bindings not analyzed

### Language-Specific

- **JavaScript**: Dynamic `require()` may miss some modules
- **Python**: `getattr()` and dynamic imports reduce precision
- **Go**: Interface implementations may have false positives
- **Ruby**: Metaprogramming triggers conservative mode
- **PHP**: Variable functions and dynamic includes are conservative

## Best Practices

1. **Run early and often** - Integrate into CI/CD
2. **Review unreachable code** - Consider removing dead code
3. **Understand limitations** - Know when analysis is conservative
4. **Combine with other tools** - Reachability + fuzzing + manual review
5. **Keep dependencies updated** - Even unreachable vulnerabilities should be patched

## References

- [Call Graph Construction](./call-graph-construction.md)
- [Entrypoint Detection](./entrypoint-detection.md)
- [Dynamic Code Handling](./dynamic-code-handling.md)
- [SARIF Integration](../formats/sarif.md)

## See Also

- [Vulnerability Enrichment](../security/vulnerability-enrichment.md)
- [Policy Integration](../user-guide/policy-integration.md)
- [Report Generation](../user-guide/report-generation.md)
