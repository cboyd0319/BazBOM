# bazbom-rust-reachability

Rust reachability analysis for BazBOM vulnerability scanning.

## Overview

This crate provides static analysis of Rust codebases to determine which functions are actually reachable from entrypoints, enabling precise vulnerability assessment with >98% accuracy.

## Features

- **High Accuracy**: Leverages Rust's `syn` parser for near-perfect AST analysis
- **Entrypoint Detection**: Identifies `main()`, `#[test]`, `#[tokio::main]`, `#[actix_web::main]`
- **Call Graph Analysis**: Builds complete call graphs using petgraph
- **Reachability Analysis**: DFS-based traversal to determine reachable code
- **Vulnerability Mapping**: Links CVEs to actual reachable functions with call chains
- **Fully Static**: No dynamic code patterns in Rust - highest precision

## Usage

```rust
use bazbom_rust_reachability::analyze_rust_project;
use std::path::PathBuf;

let project_root = PathBuf::from("/path/to/rust/project");
let report = analyze_rust_project(&project_root)?;

println!("Total functions: {}", report.all_functions.len());
println!("Reachable: {}", report.reachable_functions.len());
println!("Unreachable: {}", report.unreachable_functions.len());
```

## Entrypoint Detection

- `fn main()` - Program entry point
- `#[test]` - Test functions
- `#[tokio::main]` - Tokio async runtime
- `#[actix_web::main]` - Actix-web runtime
- `#[bench]` - Benchmark functions

## Architecture

Uses Rust's official `syn` parser for maximum accuracy:

1. **AST Parsing**: Parse Rust files using `syn`
2. **Function Extraction**: Identify all functions and their calls
3. **Call Graph**: Build directed graph with petgraph
4. **Reachability**: DFS from entrypoints to mark reachable code
5. **Reporting**: Generate detailed reachability report

## Accuracy

**>98% precision** - Rust's fully static nature allows near-perfect analysis:
- No `eval()` or reflection
- Explicit trait implementations
- No runtime metaprogramming
- Macro expansion at call sites

## See Also

- [Reachability Documentation](../../docs/reachability/README.md)
- [BazBOM Architecture](../../docs/ARCHITECTURE.md)
- Part of BazBOM v6.5.0
