# bazbom-ruby-reachability

Ruby reachability analysis for BazBOM vulnerability scanning.

## Overview

This crate provides static analysis of Ruby codebases to determine which functions are actually reachable from entrypoints, enabling precise vulnerability assessment with ~75% accuracy.

## Features

- **Framework-Aware**: Detects Rails, Sinatra, RSpec, Rake entrypoints
- **Dynamic Code Detection**: Identifies `eval`, `define_method`, `method_missing`, `send`
- **Conservative Analysis**: Falls back to marking all code reachable when metaprogramming detected
- **Call Graph Analysis**: Builds complete call graphs using petgraph
- **Vulnerability Mapping**: Links CVEs to actual reachable methods with call chains

## Usage

```rust
use bazbom_ruby_reachability::analyze_ruby_project;
use std::path::PathBuf;

let project_root = PathBuf::from("/path/to/ruby/project");
let report = analyze_ruby_project(&project_root)?;

println!("Total functions: {}", report.all_functions.len());
println!("Reachable: {}", report.reachable_functions.len());

if report.has_dynamic_code {
    println!("Warning: Dynamic code detected, using conservative analysis");
}
```

## Entrypoint Detection

### Rails
- Controller actions (public methods)
- ActiveJob `perform` methods
- Mailer methods

### Testing
- RSpec: `it`, `specify`, `example` blocks
- Minitest: `test_*` methods

### Other Frameworks
- Sinatra: HTTP routes (`get`, `post`, `put`, `delete`)
- Rake: Task definitions

## Dynamic Code Handling

Ruby's metaprogramming triggers **conservative analysis** mode:
- `eval`, `instance_eval`, `class_eval`, `module_eval`
- `define_method` - dynamic method definition
- `method_missing` - dynamic dispatch
- `send`, `__send__`, `public_send` - dynamic invocation

**Strategy**: When detected → mark all code as potentially reachable (prioritize security over precision)

## Architecture

Uses tree-sitter-ruby for parsing:

1. **AST Parsing**: Parse Ruby files using tree-sitter
2. **Function Extraction**: Identify methods and their calls
3. **Dynamic Detection**: Scan for metaprogramming patterns
4. **Call Graph**: Build directed graph with petgraph
5. **Reachability**: DFS or conservative analysis based on dynamic code
6. **Reporting**: Generate detailed reachability report

## Accuracy

**~75% precision** - Limited by Ruby's dynamic nature:
- Metaprogramming reduces precision
- `method_missing` makes analysis conservative
- Runtime method dispatch is unpredictable

## See Also

- [Reachability Documentation](../../docs/reachability/README.md)
- [BazBOM Architecture](../../docs/ARCHITECTURE.md)
- Part of BazBOM v6.5.0
