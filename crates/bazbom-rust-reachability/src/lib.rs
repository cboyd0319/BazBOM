//! Rust reachability analysis for BazBOM vulnerability scanning
//!
//! This crate provides static analysis of Rust codebases to determine which functions
//! are actually reachable from entrypoints, enabling precise vulnerability assessment.
//!
//! # Features
//!
//! - **High Accuracy**: Leverages Rust's `syn` parser for near-perfect AST analysis
//! - **Entrypoint Detection**: Identifies main functions, tests, benchmarks, and async runtimes
//! - **Call Graph Analysis**: Builds complete call graphs using petgraph
//! - **Reachability Analysis**: DFS-based traversal to determine reachable code
//! - **Vulnerability Mapping**: Links CVEs to actual reachable functions
//!
//! # Example
//!
//! ```no_run
//! use bazbom_rust_reachability::{RustReachabilityAnalyzer, analyze_rust_project};
//! use std::path::PathBuf;
//!
//! let project_root = PathBuf::from("/path/to/rust/project");
//! let report = analyze_rust_project(&project_root).expect("Analysis failed");
//!
//! println!("Total functions: {}", report.all_functions.len());
//! println!("Reachable functions: {}", report.reachable_functions.len());
//! println!("Unreachable functions: {}", report.unreachable_functions.len());
//! ```
//!
//! # Architecture
//!
//! The analyzer follows a multi-phase approach:
//!
//! 1. **Entrypoint Detection**: Finds all program entry points
//!    - `fn main()` functions
//!    - `#[test]` functions
//!    - `#[tokio::main]` and other async runtimes
//!    - Benchmarks with `#[bench]`
//!
//! 2. **AST Parsing**: Uses `syn` to parse Rust source files
//!
//! 3. **Call Graph Construction**: Builds directed graph of function calls
//!
//! 4. **Reachability Analysis**: DFS traversal from entrypoints
//!
//! 5. **Report Generation**: Produces detailed reachability report
//!
//! # Limitations
//!
//! - Dynamic dispatch through trait objects has limited precision
//! - Macros are analyzed at their call sites
//! - External crate analysis requires source availability

pub mod analyzer;
pub mod ast_parser;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod models;

pub use analyzer::RustReachabilityAnalyzer;
pub use error::{Result, RustReachabilityError};
pub use models::{
    Entrypoint, EntrypointType, FunctionId, FunctionNode, ReachabilityReport,
    VulnerabilityReachability,
};

use std::path::Path;

/// Convenience function to analyze a Rust project
///
/// # Arguments
///
/// * `project_root` - Path to the root of the Rust project
///
/// # Returns
///
/// A `ReachabilityReport` containing analysis results
///
/// # Example
///
/// ```no_run
/// use bazbom_rust_reachability::analyze_rust_project;
/// use std::path::PathBuf;
///
/// let report = analyze_rust_project(&PathBuf::from("./my-project")).unwrap();
/// println!("Found {} entrypoints", report.entrypoints.len());
/// ```
pub fn analyze_rust_project(project_root: &Path) -> Result<ReachabilityReport> {
    let mut analyzer = RustReachabilityAnalyzer::new(project_root.to_path_buf());
    analyzer.analyze()
}

/// Initialize tracing for debugging
///
/// Call this at the start of your program to enable detailed logging
/// of the analysis process.
///
/// Requires the `tracing-support` feature to be enabled.
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "tracing-support")]
/// bazbom_rust_reachability::init_tracing();
/// ```
#[cfg(feature = "tracing-support")]
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        let main_code = r#"
pub fn public_function() {
    helper();
}

fn helper() {
    println!("helper");
}

fn unused() {
    println!("unused");
}

fn main() {
    public_function();
}
"#;

        fs::write(temp_dir.path().join("main.rs"), main_code).unwrap();

        let lib_code = r#"
#[test]
fn test_example() {
    assert_eq!(2 + 2, 4);
}
"#;

        fs::write(temp_dir.path().join("lib.rs"), lib_code).unwrap();

        temp_dir
    }

    #[test]
    fn test_analyze_rust_project() {
        let temp_dir = create_test_project();
        let report = analyze_rust_project(temp_dir.path()).unwrap();

        assert!(!report.all_functions.is_empty());
        assert!(!report.entrypoints.is_empty());
    }

    #[test]
    fn test_reachability_correctness() {
        let temp_dir = create_test_project();
        let report = analyze_rust_project(temp_dir.path()).unwrap();

        // main and helper should be reachable
        assert!(report
            .reachable_functions
            .iter()
            .any(|id| id.contains("main") || id.contains("helper")));

        // unused should be unreachable
        assert!(report
            .unreachable_functions
            .iter()
            .any(|id| id.contains("unused")));
    }
}
