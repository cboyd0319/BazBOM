//! Main Rust reachability analyzer

use crate::ast_parser::{parse_file, FunctionExtractor};
use crate::call_graph::CallGraph;
use crate::entrypoints::EntrypointDetector;
use crate::error::Result;
use crate::models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::info;
use walkdir::WalkDir;

pub struct RustReachabilityAnalyzer {
    project_root: PathBuf,
    call_graph: CallGraph,
}

impl RustReachabilityAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            call_graph: CallGraph::new(),
        }
    }

    /// Run complete reachability analysis
    pub fn analyze(&mut self) -> Result<ReachabilityReport> {
        info!("Starting Rust reachability analysis");

        // Step 1: Detect entrypoints
        let entrypoints = self.detect_entrypoints()?;
        info!("Found {} entrypoints", entrypoints.len());

        // Step 2: Build call graph
        self.build_call_graph()?;
        info!("Built call graph with {} functions", self.call_graph.functions.len());

        // Step 3: Mark entrypoints
        for entrypoint in &entrypoints {
            let func_id = format!("{}::{}",
                entrypoint.file.display(),
                entrypoint.function_name
            );
            if let Some(func) = self.call_graph.functions.get_mut(&func_id) {
                func.is_entrypoint = true;
            }
        }

        // Step 4: Analyze reachability
        self.call_graph.analyze_reachability()?;

        // Step 5: Generate report
        let report = self.generate_report()?;

        info!("Analysis complete: {}/{} functions reachable",
            report.reachable_functions.len(),
            report.all_functions.len()
        );

        Ok(report)
    }

    fn detect_entrypoints(&self) -> Result<Vec<crate::models::Entrypoint>> {
        let detector = EntrypointDetector::new(self.project_root.clone());
        detector.detect_entrypoints()
    }

    fn build_call_graph(&mut self) -> Result<()> {
        info!("Building call graph from Rust source");

        // Collect all Rust files first to avoid borrow checker issues
        let rust_files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::should_skip_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .filter(|p| Self::is_rust_file(p))
            .collect();

        // Now process all files
        for file_path in rust_files {
            self.process_file(&file_path)?;
        }

        Ok(())
    }

    fn process_file(&mut self, file_path: &Path) -> Result<()> {
        let ast = parse_file(file_path)?;
        let mut extractor = FunctionExtractor::new();
        extractor.extract(&ast);

        // Add functions to call graph
        for func in &extractor.functions {
            let func_id = format!("{}::{}", file_path.display(), func.name);

            let function_node = FunctionNode {
                id: func_id.clone(),
                name: func.name.clone(),
                file: file_path.to_path_buf(),
                line: func.line,
                column: 0,
                is_entrypoint: false, // Will be set later
                reachable: false,
                calls: Vec::new(),
                is_pub: func.is_pub,
                is_async: func.is_async,
                is_test: func.is_test,
            };

            self.call_graph.add_function(function_node);
        }

        // Add call edges
        for call in &extractor.calls {
            if let Some(caller_context) = &call.caller_context {
                let caller_id = format!("{}::{}", file_path.display(), caller_context);

                // Try to resolve callee
                // This is simplified - real implementation would need proper name resolution
                let callee_id = self.resolve_function_call(&call.callee, file_path);

                self.call_graph.add_call(&caller_id, &callee_id);
            }
        }

        Ok(())
    }

    fn resolve_function_call(&self, callee: &str, current_file: &Path) -> String {
        // Simplified resolution - in reality we'd need:
        // 1. Check local functions in same file
        // 2. Check imports/use statements
        // 3. Check standard library
        // 4. Check external crates

        // For now, assume same file if simple name
        if !callee.contains("::") {
            format!("{}::{}", current_file.display(), callee)
        } else {
            callee.to_string()
        }
    }

    fn generate_report(&self) -> Result<ReachabilityReport> {
        let all_functions = self.call_graph.functions.clone();

        let reachable_functions: HashSet<_> = all_functions.values()
            .filter(|f| f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let unreachable_functions: HashSet<_> = all_functions.values()
            .filter(|f| !f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let entrypoints: Vec<_> = all_functions.values()
            .filter(|f| f.is_entrypoint)
            .map(|f| f.id.clone())
            .collect();

        Ok(ReachabilityReport {
            all_functions,
            reachable_functions,
            unreachable_functions,
            entrypoints,
            vulnerabilities: Vec::new(), // Would be populated with actual vulnerability data
        })
    }

    /// Analyze if vulnerable functions are reachable
    pub fn analyze_vulnerability_reachability(
        &self,
        vulnerabilities: Vec<VulnerabilityReachability>,
    ) -> Vec<VulnerabilityReachability> {
        vulnerabilities.into_iter().map(|mut vuln| {
            // Check if any vulnerable function is reachable
            let is_reachable = vuln.vulnerable_functions.iter().any(|func_name| {
                self.call_graph.functions.values().any(|f| {
                    f.name.contains(func_name) && f.reachable
                })
            });

            vuln.reachable = is_reachable;

            // Try to find call chain if reachable
            if is_reachable {
                for func_name in &vuln.vulnerable_functions {
                    if let Some(func) = self.call_graph.functions.values()
                        .find(|f| f.name.contains(func_name))
                    {
                        if let Some(chain) = self.call_graph.find_call_chain(&func.id) {
                            vuln.call_chain = Some(chain);
                            break;
                        }
                    }
                }
            }

            vuln
        }).collect()
    }

    fn should_skip_entry(entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["target", ".git", "node_modules", "vendor"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    fn is_rust_file(path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("rs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        let main_code = r#"
fn helper() {
    println!("helper");
}

fn unused() {
    println!("unused");
}

fn main() {
    helper();
}
"#;

        fs::write(temp_dir.path().join("main.rs"), main_code).unwrap();

        temp_dir
    }

    #[test]
    fn test_analyze() {
        let temp_dir = create_test_project();
        let mut analyzer = RustReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        let report = analyzer.analyze().unwrap();

        assert!(!report.all_functions.is_empty());
        assert!(!report.entrypoints.is_empty());
        assert!(!report.reachable_functions.is_empty());
    }

    #[test]
    fn test_detect_entrypoints() {
        let temp_dir = create_test_project();
        let analyzer = RustReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        let entrypoints = analyzer.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints.iter().any(|e| e.function_name == "main"));
    }

    #[test]
    fn test_build_call_graph() {
        let temp_dir = create_test_project();
        let mut analyzer = RustReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        analyzer.build_call_graph().unwrap();

        assert!(!analyzer.call_graph.functions.is_empty());
    }

    #[test]
    fn test_vulnerability_reachability() {
        let temp_dir = create_test_project();
        let mut analyzer = RustReachabilityAnalyzer::new(temp_dir.path().to_path_buf());
        analyzer.analyze().unwrap();

        let vuln = VulnerabilityReachability {
            cve_id: "CVE-2024-TEST".to_string(),
            package: "test-package".to_string(),
            version: "1.0.0".to_string(),
            vulnerable_functions: vec!["helper".to_string()],
            reachable: false,
            call_chain: None,
        };

        let results = analyzer.analyze_vulnerability_reachability(vec![vuln]);

        assert_eq!(results.len(), 1);
        assert!(results[0].reachable);
    }
}
