// Example: Parse Go and Rust dependencies using BazBOM ecosystem parsers
use bazbom::ecosystems::{CargoParser, DependencyScope, GoModulesParser};
use std::path::Path;

fn main() {
    println!("=== BazBOM Ecosystem Parser Example ===\n");

    // Test Go parser
    println!("📦 Testing Go Modules Parser");
    println!("────────────────────────────");
    let go_parser = GoModulesParser::new();
    match go_parser.parse_go_mod(Path::new("examples/go-example/go.mod")) {
        Ok(deps) => {
            println!("Found {} Go dependencies:\n", deps.len());
            for dep in deps {
                let scope_label = if dep.scope == DependencyScope::Indirect {
                    "indirect"
                } else {
                    "direct"
                };
                println!("  • {} v{}", dep.name, dep.version);
                println!("    Scope: {}", scope_label);
                println!("    PURL:  {}\n", dep.purl);
            }
        }
        Err(e) => eprintln!("❌ Error parsing go.mod: {}", e),
    }

    // Test Rust parser
    println!("\n🦀 Testing Rust/Cargo Parser");
    println!("────────────────────────────");
    let rust_parser = CargoParser::new();
    match rust_parser.parse_cargo_toml(Path::new("examples/rust-example/Cargo.toml")) {
        Ok(deps) => {
            println!("Found {} Rust dependencies:\n", deps.len());
            for dep in deps {
                let scope_label = match dep.scope {
                    DependencyScope::Direct => "runtime",
                    DependencyScope::Dev => "dev",
                    DependencyScope::Build => "build",
                    _ => "unknown",
                };
                println!("  • {} v{}", dep.name, dep.version);
                println!("    Scope: {}", scope_label);
                println!("    PURL:  {}\n", dep.purl);
            }
        }
        Err(e) => eprintln!("❌ Error parsing Cargo.toml: {}", e),
    }

    println!("✅ Example complete!");
}
