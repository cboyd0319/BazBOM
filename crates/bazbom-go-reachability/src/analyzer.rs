//! Main orchestrator for Go reachability analysis

use crate::ast_parser::{parse_file, FunctionExtractor};
use crate::call_graph::CallGraph;
use crate::entrypoints::EntrypointDetector;
use crate::error::Result;
use crate::models::{
    FunctionNode, ReachabilityReport, ReflectionWarning, VulnerabilityReachability,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Main analyzer for Go reachability
pub struct GoReachabilityAnalyzer {
    call_graph: CallGraph,
    processed_files: HashSet<PathBuf>,
    reflection_warnings: Vec<ReflectionWarning>,
    has_reflection: bool,
}

impl GoReachabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::new(),
            processed_files: HashSet::new(),
            reflection_warnings: Vec::new(),
            has_reflection: false,
        }
    }

    /// Analyze a Go project for reachability
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        info!("Starting Go reachability analysis");
        info!("Project root: {:?}", project_root);

        // 1. Detect entrypoints
        let entrypoint_detector = EntrypointDetector::new(project_root.to_path_buf());
        let entrypoints = entrypoint_detector.detect_entrypoints()?;

        info!("Found {} entrypoints", entrypoints.len());

        // 2. Discover and parse all Go files
        self.discover_and_parse_files(project_root)?;

        // 3. Mark entrypoints in the call graph
        for entrypoint in &entrypoints {
            let entrypoint_id =
                format!("{}:{}", entrypoint.file.display(), entrypoint.function_name);

            if let Err(e) = self.call_graph.mark_entrypoint(&entrypoint_id) {
                debug!("Could not mark entrypoint {}: {}", entrypoint_id, e);
            }
        }

        // 4. Perform reachability analysis
        if self.has_reflection {
            warn!("Reflection detected - using conservative analysis");
            self.call_graph.mark_all_reachable();
        } else {
            self.call_graph.analyze_reachability()?;
        }

        // 5. Generate report
        let report = self.generate_report(entrypoints)?;

        info!(
            "Analysis complete: {} reachable / {} total functions",
            report.reachable_functions.len(),
            report.all_functions.len()
        );

        Ok(report)
    }

    /// Discover and parse all Go files in the project
    fn discover_and_parse_files(&mut self, project_root: &Path) -> Result<()> {
        info!("Discovering and parsing Go files...");

        let skip_dirs = ["vendor", ".git", "testdata", "node_modules"];

        for entry in WalkDir::new(project_root)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let dir_name = e.file_name().to_str().unwrap_or("");
                    !skip_dirs.contains(&dir_name)
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if self.is_go_file(path) && !self.processed_files.contains(path) {
                    if let Err(e) = self.parse_and_build_graph(path) {
                        debug!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse a file and add functions/calls to the call graph
    fn parse_and_build_graph(&mut self, file_path: &Path) -> Result<()> {
        debug!("Parsing file: {:?}", file_path);

        let source_code = std::fs::read_to_string(file_path)?;
        let tree = parse_file(file_path)?;
        let mut extractor = FunctionExtractor::new();
        extractor.extract(&source_code, &tree)?;

        // Check for reflection
        if !extractor.reflections.is_empty() {
            self.has_reflection = true;
            for detection in &extractor.reflections {
                self.reflection_warnings.push(ReflectionWarning {
                    file: file_path.to_path_buf(),
                    line: detection.line,
                    warning_type: detection.reflection_type.clone(),
                    description: detection.description.clone(),
                });
            }
        }

        // Add functions to call graph
        for func in &extractor.functions {
            let function_id = format!("{}:{}", file_path.display(), func.name);

            let mut function_node = FunctionNode::new(
                function_id.clone(),
                func.name.clone(),
                file_path.to_path_buf(),
                func.line,
                func.column,
            );

            function_node.receiver_type = func.receiver_type.clone();
            function_node.is_exported = func.is_exported;

            self.call_graph.add_function(function_node)?;
        }

        // Add call edges
        for call in &extractor.calls {
            let caller_id = if let Some(caller_context) = &call.caller_context {
                format!("{}:{}", file_path.display(), caller_context)
            } else {
                format!("{}:__init__", file_path.display())
            };

            let callee_id = format!("{}:{}", file_path.display(), call.callee);

            let _ = self.call_graph.add_call(&caller_id, &callee_id);
        }

        self.processed_files.insert(file_path.to_path_buf());
        Ok(())
    }

    /// Generate the final reachability report
    fn generate_report(
        &self,
        entrypoints: Vec<crate::models::Entrypoint>,
    ) -> Result<ReachabilityReport> {
        let all_functions = self.call_graph.functions().clone();

        let reachable_functions: HashSet<String> = all_functions
            .values()
            .filter(|f| f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let unreachable_functions: HashSet<String> = all_functions
            .values()
            .filter(|f| !f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let entrypoint_ids: Vec<String> = entrypoints
            .iter()
            .map(|e| format!("{}:{}", e.file.display(), e.function_name))
            .collect();

        Ok(ReachabilityReport {
            all_functions,
            reachable_functions,
            unreachable_functions,
            entrypoints: entrypoint_ids,
            vulnerabilities: Vec::new(),
            reflection_warnings: self.reflection_warnings.clone(),
        })
    }

    /// Check if a vulnerability is reachable
    pub fn check_vulnerability_reachability(
        &self,
        package: &str,
        vulnerable_function: &str,
    ) -> Option<VulnerabilityReachability> {
        let vulnerable_id = format!("{}:{}", package, vulnerable_function);

        let reachable = self
            .call_graph
            .functions()
            .get(&vulnerable_id)
            .map(|f| f.reachable)
            .unwrap_or(false);

        let call_chain = if reachable {
            self.call_graph.find_call_chain(&vulnerable_id)
        } else {
            None
        };

        Some(VulnerabilityReachability {
            cve_id: String::new(),
            package: package.to_string(),
            version: String::new(),
            vulnerable_functions: vec![vulnerable_function.to_string()],
            reachable,
            call_chain,
        })
    }

    fn is_go_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            ext == "go" && !path.to_str().unwrap_or("").ends_with("_test.go")
        } else {
            false
        }
    }
}

impl Default for GoReachabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_analyze_simple_project() {
        let temp_dir = TempDir::new().unwrap();

        let main_go = r#"
package main

func helper() {
    println("helper")
}

func unused() {
    println("unused")
}

func main() {
    helper()
}
"#;
        fs::write(temp_dir.path().join("main.go"), main_go).unwrap();

        let mut analyzer = GoReachabilityAnalyzer::new();
        let report = analyzer.analyze(temp_dir.path()).unwrap();

        assert!(
            report.all_functions.len() >= 2,
            "Should have found at least 2 functions"
        );
        assert!(
            !report.entrypoints.is_empty(),
            "Should have found main entrypoint"
        );
    }
}
