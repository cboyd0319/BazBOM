//! Go Reachability Analysis for BazBOM
//!
//! This crate provides static analysis capabilities to determine which code paths
//! in Go projects are actually reachable from entrypoints.
//!
//! ## Features
//!
//! - AST parsing with tree-sitter-go
//! - Call graph generation
//! - Reachability analysis via DFS traversal
//! - Goroutine tracking
//! - Reflection detection and conservative analysis
//!
//! ## Example
//!
//! ```no_run
//! use bazbom_go_reachability::GoReachabilityAnalyzer;
//! use std::path::Path;
//!
//! let mut analyzer = GoReachabilityAnalyzer::new();
//! let report = analyzer.analyze(Path::new("./src")).unwrap();
//!
//! println!("Found {} reachable functions", report.reachable_functions.len());
//! ```

pub mod analyzer;
pub mod ast_parser;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod models;

pub use analyzer::GoReachabilityAnalyzer;
pub use call_graph::CallGraph;
pub use error::{GoReachabilityError, Result};
pub use models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
