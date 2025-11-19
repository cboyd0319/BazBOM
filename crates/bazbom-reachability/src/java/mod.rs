//! Java Reachability Analysis
//!
//! Analyzes Java bytecode to determine which methods are reachable from entrypoints.
//! This helps identify whether vulnerable code in dependencies is actually used.
//!
//! # Architecture
//!
//! 1. **Bytecode Analysis** - Parse .class and .jar files to build call graph
//! 2. **Entrypoint Detection** - Identify main(), Servlet, Spring, JAX-RS endpoints
//! 3. **Reachability Computation** - DFS traversal from entrypoints
//! 4. **Report Generation** - List reachable/unreachable methods
//!
//! # Usage
//!
//! ```rust,no_run
//! use bazbom_reachability::java::analyze_java_project;
//! use std::path::Path;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let report = analyze_java_project(Path::new("target/classes"))?;
//!
//! println!("Found {} entrypoints", report.entrypoints.len());
//! println!("Reachable: {}/{}",
//!     report.reachable_functions.len(),
//!     report.all_functions.len()
//! );
//! # Ok(())
//! # }
//! ```

pub mod analyzer;
pub mod bytecode_analyzer;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod models;

pub use analyzer::JavaReachabilityAnalyzer;
pub use call_graph::CallGraph;
pub use error::{JavaReachabilityError, Result};
pub use models::{
    MethodId, MethodNode, ReachabilityReport, ReflectionWarning, VulnerabilityReachability,
};

use std::path::Path;

/// Convenience function to analyze a Java project
///
/// # Arguments
///
/// * `project_root` - Path to the project root (containing .class files or .jar files)
///
/// # Returns
///
/// A `ReachabilityReport` containing entrypoints, reachable methods, and unreachable methods.
///
/// # Example
///
/// ```rust,no_run
/// use bazbom_reachability::java::analyze_java_project;
/// use std::path::Path;
///
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let report = analyze_java_project(Path::new("target/classes"))?;
/// println!("Analyzed {} methods", report.all_functions.len());
/// # Ok(())
/// # }
/// ```
pub fn analyze_java_project(project_root: &Path) -> Result<ReachabilityReport> {
    let mut analyzer = JavaReachabilityAnalyzer::new();
    analyzer.analyze(project_root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let report = analyze_java_project(temp_dir.path()).unwrap();

        assert_eq!(report.entrypoints.len(), 0);
        assert_eq!(report.all_functions.len(), 0);
    }
}
