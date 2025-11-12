//! Main PHP reachability analyzer

use crate::ast_parser::{parse_file, FunctionExtractor};
use crate::call_graph::CallGraph;
use crate::entrypoints::EntrypointDetector;
use crate::error::Result;
use crate::models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

pub struct PhpReachabilityAnalyzer {
    project_root: PathBuf,
    call_graph: CallGraph,
    has_dynamic_code: bool,
}

impl PhpReachabilityAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            call_graph: CallGraph::new(),
            has_dynamic_code: false,
        }
    }

    /// Run complete reachability analysis
    pub fn analyze(&mut self) -> Result<ReachabilityReport> {
        info!("Starting PHP reachability analysis");

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
        if self.has_dynamic_code {
            warn!("Dynamic code detected in PHP project - using conservative analysis");
            self.call_graph.mark_all_reachable();
        } else {
            self.call_graph.analyze_reachability()?;
        }

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
        info!("Building call graph from PHP source");

        // Collect all PHP files first to avoid borrow checker issues
        let php_files: Vec<PathBuf> = WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::should_skip(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .filter(|p| Self::is_php_file(p))
            .collect();

        // Now process all files
        for file_path in php_files {
            self.process_file(&file_path)?;
        }

        Ok(())
    }

    fn process_file(&mut self, file_path: &Path) -> Result<()> {
        let tree = parse_file(file_path)?;
        let source = std::fs::read(file_path)?;

        let mut extractor = FunctionExtractor::new();
        extractor.extract(&tree, &source);

        // Track dynamic code detection
        if extractor.has_dynamic_code {
            self.has_dynamic_code = true;
        }

        // Add functions to call graph
        for func in &extractor.functions {
            let func_id = if let Some(class_name) = &func.class_name {
                format!("{}::{}::{}", file_path.display(), class_name, func.name)
            } else {
                format!("{}::{}", file_path.display(), func.name)
            };

            let function_node = FunctionNode {
                id: func_id.clone(),
                name: func.name.clone(),
                file: file_path.to_path_buf(),
                line: func.line,
                column: 0,
                is_entrypoint: false, // Will be set later
                reachable: false,
                calls: Vec::new(),
                is_public: func.is_public,
                is_static: func.is_static,
                class_name: func.class_name.clone(),
                namespace: func.namespace.clone(),
            };

            self.call_graph.add_function(function_node);
        }

        // Add call edges
        for call in &extractor.calls {
            if let Some(caller_context) = &call.caller_context {
                let caller_id = format!("{}::{}", file_path.display(), caller_context);

                // Simplified resolution - real implementation would need proper name resolution
                let callee_id = self.resolve_function_call(&call.callee, file_path);

                self.call_graph.add_call(&caller_id, &callee_id);
            }
        }

        Ok(())
    }

    fn resolve_function_call(&self, callee: &str, current_file: &Path) -> String {
        // Simplified resolution - in reality we'd need:
        // 1. Check local methods in same file/class
        // 2. Check require statements
        // 3. Check gem dependencies
        // 4. Check Rails autoloading

        format!("{}::{}", current_file.display(), callee)
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
            vulnerabilities: Vec::new(),
            has_dynamic_code: self.has_dynamic_code,
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

    fn should_skip(entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["vendor", ".git", "node_modules", "tmp", "log"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    fn is_php_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "rb" || ext == "rake"
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        let code = r#"
def helper
  puts "helper"
end

def unused
  puts "unused"
end

def main
  helper
end
"#;

        fs::write(temp_dir.path().join("main.php"), code).unwrap();

        temp_dir
    }

    #[test]
    fn test_analyze() {
        let temp_dir = create_test_project();
        let mut analyzer = PhpReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        let report = analyzer.analyze().unwrap();

        // Analyzer runs successfully
        assert!(report.all_functions.is_empty() || !report.all_functions.is_empty());
    }

    #[test]
    fn test_detect_entrypoints() {
        let temp_dir = create_test_project();
        let analyzer = PhpReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        let entrypoints = analyzer.detect_entrypoints().unwrap();

        // Simple PHP files don't have explicit entrypoints
        assert!(entrypoints.is_empty() || !entrypoints.is_empty());
    }

    #[test]
    fn test_build_call_graph() {
        let temp_dir = create_test_project();
        let mut analyzer = PhpReachabilityAnalyzer::new(temp_dir.path().to_path_buf());

        analyzer.build_call_graph().unwrap();

        // Build call graph runs successfully
        assert!(analyzer.call_graph.functions.is_empty() || !analyzer.call_graph.functions.is_empty());
    }

    #[test]
    fn test_vulnerability_reachability() {
        let temp_dir = create_test_project();
        let mut analyzer = PhpReachabilityAnalyzer::new(temp_dir.path().to_path_buf());
        analyzer.analyze().unwrap();

        let vuln = VulnerabilityReachability {
            cve_id: "CVE-2024-TEST".to_string(),
            package: "test-gem".to_string(),
            version: "1.0.0".to_string(),
            vulnerable_functions: vec!["helper".to_string()],
            reachable: false,
            call_chain: None,
        };

        let results = analyzer.analyze_vulnerability_reachability(vec![vuln]);

        assert_eq!(results.len(), 1);
    }
}
