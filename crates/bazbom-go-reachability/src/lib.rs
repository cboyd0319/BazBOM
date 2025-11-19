//! Go reachability analysis for BazBOM vulnerability scanning

pub mod analyzer;
pub mod error;
pub mod models;

pub use analyzer::GoReachabilityAnalyzer;
pub use error::{GoReachabilityError, Result};
pub use models::{FunctionNode, ReachabilityReport};

use std::path::Path;

/// Convenience function to analyze a Go project
pub fn analyze_go_project(project_root: &Path) -> Result<ReachabilityReport> {
    let mut analyzer = GoReachabilityAnalyzer::new();
    analyzer.analyze(project_root)
}
