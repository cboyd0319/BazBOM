//! JavaScript/TypeScript Reachability Analysis for BazBOM
//!
//! This crate provides static analysis capabilities to determine which code paths
//! in JavaScript and TypeScript projects are actually reachable from entrypoints.
//! This is crucial for vulnerability analysis - knowing whether vulnerable code
//! is actually used by your application.
//!
//! ## Features
//!
//! - AST parsing with SWC (Rust-native, fast)
//! - Call graph generation
//! - Reachability analysis via DFS traversal
//! - Support for both CommonJS and ESM modules
//! - Conservative handling of dynamic imports
//!
//! ## Example
//!
//! ```no_run
//! use bazbom_js_reachability::JsReachabilityAnalyzer;
//! use std::path::Path;
//!
//! let analyzer = JsReachabilityAnalyzer::new();
//! let report = analyzer.analyze(Path::new("./src")).unwrap();
//!
//! println!("Found {} reachable functions", report.reachable_functions.len());
//! ```

pub mod analyzer;
pub mod ast_parser;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod module_resolver;
pub mod models;

pub use analyzer::JsReachabilityAnalyzer;
pub use call_graph::CallGraph;
pub use error::{JsReachabilityError, Result};
pub use models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
