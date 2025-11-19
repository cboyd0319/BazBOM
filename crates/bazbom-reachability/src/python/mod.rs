//! Python Reachability Analysis for BazBOM
//!
//! This crate provides static analysis capabilities to determine which code paths
//! in Python projects are actually reachable from entrypoints.
//! This is crucial for vulnerability analysis - knowing whether vulnerable code
//! is actually used by your application.
//!
//! ## Features
//!
//! - AST parsing with RustPython (pure Rust implementation)
//! - Call graph generation
//! - Reachability analysis via DFS traversal
//! - Support for multiple Python frameworks (Flask, Django, FastAPI, Click, Celery)
//! - Conservative handling of dynamic code (exec, eval, getattr)
//! - Dynamic code warnings for analysis limitations
//!
//! ## Example
//!
//! ```no_run
//! use bazbom_reachability::python::PythonReachabilityAnalyzer;
//! use std::path::Path;
//!
//! let mut analyzer = PythonReachabilityAnalyzer::new();
//! let report = analyzer.analyze(Path::new("./src")).unwrap();
//!
//! println!("Found {} reachable functions", report.reachable_functions.len());
//!
//! // Check for dynamic code warnings
//! for warning in &report.dynamic_code_warnings {
//!     println!("Warning: {:?} at {}:{}", warning.warning_type, warning.file.display(), warning.line);
//! }
//! ```
//!
//! ## Limitations
//!
//! Python's dynamic nature means perfect reachability analysis is impossible.
//! This analyzer uses conservative strategies:
//!
//! - When `exec()`, `eval()`, or dynamic `getattr()` is detected, all code is marked as reachable
//! - Dynamic imports with variable module names are flagged
//! - Metaclass magic and advanced descriptor protocol usage may be imprecise
//! - C extensions cannot be analyzed (native code)
//!
//! Expected accuracy: >80% for typical Python codebases

pub mod analyzer;
pub mod ast_parser;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod models;
pub mod module_resolver;

pub use analyzer::PythonReachabilityAnalyzer;
pub use call_graph::CallGraph;
pub use error::{PythonReachabilityError, Result};
pub use models::{DynamicCodeWarning, FunctionNode, ReachabilityReport, VulnerabilityReachability};

use std::path::Path;

/// Convenience function to analyze a Python project
pub fn analyze_python_project(project_root: &Path) -> Result<ReachabilityReport> {
    let mut analyzer = PythonReachabilityAnalyzer::new();
    analyzer.analyze(project_root)
}
