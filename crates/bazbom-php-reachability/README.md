# bazbom-php-reachability

PHP reachability analysis for BazBOM vulnerability scanning.

## Overview

This crate provides static analysis of PHP codebases to determine which functions are actually reachable from entrypoints, enabling precise vulnerability assessment with ~70% accuracy.

## Features

- **Framework-Aware**: Detects Symfony, Laravel, WordPress, PHPUnit entrypoints
- **Namespace Support**: PSR-4 autoloading resolution
- **Dynamic Code Detection**: Identifies `eval`, `call_user_func`, variable functions
- **Conservative Analysis**: Falls back to marking all code reachable when dynamic patterns detected
- **Call Graph Analysis**: Builds complete call graphs using petgraph
- **Vulnerability Mapping**: Links CVEs to actual reachable functions with call chains

## Usage

```rust
use bazbom_php_reachability::analyze_php_project;
use std::path::PathBuf;

let project_root = PathBuf::from("/path/to/php/project");
let report = analyze_php_project(&project_root)?;

println!("Total functions: {}", report.all_functions.len());
println!("Reachable: {}", report.reachable_functions.len());

if report.has_dynamic_code {
    println!("Warning: Dynamic code detected, using conservative analysis");
}
```

## Entrypoint Detection

### Symfony
- Controller methods (public methods with `Route` annotations)
- Console commands

### Laravel
- Controller methods (in `app/Http/Controllers`)
- Jobs, Commands, Artisan commands

### WordPress
- Action hooks (`add_action`)
- Filter hooks (`add_filter`)

### Testing
- PHPUnit: `test*` methods

## Dynamic Code Handling

PHP's dynamic features trigger **conservative analysis** mode:
- `eval()` - code execution
- `call_user_func()`, `call_user_func_array()` - dynamic dispatch
- Variable functions - `$func_name()`
- Dynamic includes - `include`/`require` with variables

**Strategy**: When detected → mark all code as potentially reachable (prioritize security over precision)

## Architecture

Uses tree-sitter-php for parsing:

1. **AST Parsing**: Parse PHP files using tree-sitter
2. **Namespace Resolution**: Track namespaces and PSR-4 structure
3. **Function Extraction**: Identify functions/methods and their calls
4. **Dynamic Detection**: Scan for dynamic code patterns
5. **Call Graph**: Build directed graph with petgraph
6. **Reachability**: DFS or conservative analysis based on dynamic code
7. **Reporting**: Generate detailed reachability report

## Accuracy

**~70% precision** - Limited by PHP's dynamic nature:
- Variable functions reduce precision
- Dynamic includes are unpredictable
- `eval()` makes analysis conservative
- Reflection is conservative

## See Also

- [Reachability Documentation](../../docs/reachability/README.md)
- [BazBOM Architecture](../../docs/ARCHITECTURE.md)
- Part of BazBOM v6.5.0
