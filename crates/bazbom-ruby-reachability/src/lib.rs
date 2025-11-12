//! Ruby reachability analysis for BazBOM vulnerability scanning
//!
//! This crate provides static analysis of Ruby codebases to determine which functions
//! are actually reachable from entrypoints, enabling precise vulnerability assessment.
//!
//! # Features
//!
//! - **Framework-Aware**: Detects Rails, Sinatra, RSpec, and Rake entrypoints
//! - **Dynamic Code Detection**: Identifies eval, define_method, method_missing patterns
//! - **Conservative Analysis**: Falls back to marking all code reachable when dynamic patterns detected
//! - **Call Graph Analysis**: Builds complete call graphs using petgraph
//! - **Vulnerability Mapping**: Links CVEs to actual reachable methods
//!
//! # Example
//!
//! ```no_run
//! use bazbom_ruby_reachability::{RubyReachabilityAnalyzer, analyze_ruby_project};
//! use std::path::PathBuf;
//!
//! let project_root = PathBuf::from("/path/to/ruby/project");
//! let report = analyze_ruby_project(&project_root).expect("Analysis failed");
//!
//! println!("Total functions: {}", report.all_functions.len());
//! println!("Reachable functions: {}", report.reachable_functions.len());
//!
//! if report.has_dynamic_code {
//!     println!("Warning: Dynamic code detected, using conservative analysis");
//! }
//! ```
//!
//! # Supported Frameworks
//!
//! - **Rails**: Controllers, Jobs, Mailers
//! - **RSpec**: Test examples (it, specify, example)
//! - **Minitest**: test_* methods
//! - **Sinatra**: HTTP routes (get, post, put, delete)
//! - **Rake**: Task definitions
//!
//! # Dynamic Code Limitations
//!
//! Ruby's metaprogramming features limit static analysis precision:
//! - `eval`, `instance_eval`, `class_eval`, `module_eval`
//! - `define_method`, `method_missing`
//! - `send`, `__send__`, `public_send`
//!
//! When these patterns are detected, the analyzer uses conservative analysis
//! (marking all code as potentially reachable) to avoid false negatives.

pub mod analyzer;
pub mod ast_parser;
pub mod call_graph;
pub mod entrypoints;
pub mod error;
pub mod models;

pub use analyzer::RubyReachabilityAnalyzer;
pub use error::{Result, RubyReachabilityError};
pub use models::{
    Entrypoint, EntrypointType, FunctionId, FunctionNode, ReachabilityReport,
    VulnerabilityReachability,
};

use std::path::Path;

/// Convenience function to analyze a Ruby project
///
/// # Arguments
///
/// * `project_root` - Path to the root of the Ruby project
///
/// # Returns
///
/// A `ReachabilityReport` containing analysis results
///
/// # Example
///
/// ```no_run
/// use bazbom_ruby_reachability::analyze_ruby_project;
/// use std::path::PathBuf;
///
/// let report = analyze_ruby_project(&PathBuf::from("./my-project")).unwrap();
/// println!("Found {} entrypoints", report.entrypoints.len());
/// ```
pub fn analyze_ruby_project(project_root: &Path) -> Result<ReachabilityReport> {
    let mut analyzer = RubyReachabilityAnalyzer::new(project_root.to_path_buf());
    analyzer.analyze()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        let main_code = r#"
def public_function
  helper
end

def helper
  puts "helper"
end

def unused
  puts "unused"
end

def main
  public_function
end
"#;

        fs::write(temp_dir.path().join("main.rb"), main_code).unwrap();

        let spec_code = r#"
RSpec.describe "Example" do
  it "tests something" do
    expect(1 + 1).to eq(2)
  end
end
"#;

        fs::write(temp_dir.path().join("example_spec.rb"), spec_code).unwrap();

        temp_dir
    }

    #[test]
    fn test_analyze_ruby_project() {
        let temp_dir = create_test_project();
        let report = analyze_ruby_project(temp_dir.path()).unwrap();

        assert!(!report.all_functions.is_empty());
    }

    #[test]
    fn test_dynamic_code_detection() {
        let temp_dir = TempDir::new().unwrap();

        let code = r#"
def dynamic_method
  eval("some_code")
end
"#;

        fs::write(temp_dir.path().join("dynamic.rb"), code).unwrap();

        let report = analyze_ruby_project(temp_dir.path()).unwrap();

        assert!(report.has_dynamic_code);
    }
}
