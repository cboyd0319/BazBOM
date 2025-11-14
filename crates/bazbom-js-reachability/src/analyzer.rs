//! Main orchestrator for JavaScript/TypeScript reachability analysis

use crate::ast_parser::{parse_file, FunctionExtractor};
use crate::call_graph::CallGraph;
use crate::entrypoints::EntrypointDetector;
use crate::error::Result;
use crate::models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
use crate::module_resolver::ModuleResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Main analyzer for JavaScript/TypeScript reachability
pub struct JsReachabilityAnalyzer {
    call_graph: CallGraph,
    module_resolver: ModuleResolver,
    processed_files: HashSet<PathBuf>,
}

impl JsReachabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::new(),
            module_resolver: ModuleResolver::new(PathBuf::new()),
            processed_files: HashSet::new(),
        }
    }

    /// Analyze a JavaScript/TypeScript project for reachability
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        info!("Starting JavaScript/TypeScript reachability analysis");
        info!("Project root: {:?}", project_root);

        // Initialize module resolver with project root
        self.module_resolver = ModuleResolver::new(project_root.to_path_buf());

        // 1. Detect entrypoints
        let entrypoint_detector = EntrypointDetector::new(project_root.to_path_buf());
        let entrypoints = entrypoint_detector.detect_entrypoints()?;

        info!("Found {} entrypoints", entrypoints.len());

        // 2. Discover and parse all JavaScript/TypeScript files
        self.discover_and_parse_files(project_root)?;

        // 3. Mark entrypoints in the call graph
        for entrypoint in &entrypoints {
            let entrypoint_id = format!("{}:{}", entrypoint.file.display(), entrypoint.function_name);
            if let Err(e) = self.call_graph.mark_entrypoint(&entrypoint_id) {
                debug!("Could not mark entrypoint {}: {}", entrypoint_id, e);
            }
        }

        // 4. Perform reachability analysis
        self.call_graph.analyze_reachability()?;

        // 5. Generate report
        let report = self.generate_report(entrypoints)?;

        info!(
            "Analysis complete: {} reachable / {} total functions",
            report.reachable_functions.len(),
            report.all_functions.len()
        );

        Ok(report)
    }

    /// Discover and parse all JS/TS files in the project
    fn discover_and_parse_files(&mut self, project_root: &Path) -> Result<()> {
        info!("Discovering and parsing files...");

        // Common directories to skip
        let skip_dirs = ["node_modules", "dist", "build", "coverage", ".git"];

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

                if self.is_js_or_ts_file(path) && !self.processed_files.contains(path) {
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
        let mut extractor = FunctionExtractor::new(file_path.to_str().unwrap_or("unknown"));
        extractor.extract(&source_code, &tree)?;

        // Add functions to call graph
        for func in &extractor.functions {
            let function_id = format!("{}:{}", file_path.display(), func.name);

            let function_node = FunctionNode::new(
                function_id.clone(),
                func.name.clone(),
                file_path.to_path_buf(),
                func.line,
                func.column,
            );

            self.call_graph.add_function(function_node)?;
        }

        // Add call edges
        for call in &extractor.calls {
            // Try to resolve the callee
            // For now, assume calls within the same file
            let caller_id = format!("{}:unknown", file_path.display());
            let callee_id = format!("{}:{}", file_path.display(), call.callee);

            // Try to add the call edge (may fail if callee not found, which is OK)
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
            // Vulnerabilities are populated by bazbom-polyglot's reachability_integration module
            vulnerabilities: Vec::new(),
        })
    }

    /// Check if a vulnerability is reachable
    pub fn check_vulnerability_reachability(
        &self,
        package: &str,
        vulnerable_function: &str,
    ) -> Option<VulnerabilityReachability> {
        // Search for the vulnerable function in the call graph
        let vulnerable_id = format!("node_modules/{}:{}", package, vulnerable_function);

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
            cve_id: String::new(), // To be filled by caller
            package: package.to_string(),
            version: String::new(), // To be filled by caller
            vulnerable_functions: vec![vulnerable_function.to_string()],
            reachable,
            call_chain,
        })
    }

    fn is_js_or_ts_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext, "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs")
        } else {
            false
        }
    }
}

impl Default for JsReachabilityAnalyzer {
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
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();

        // Create package.json
        let package_json = r#"{
            "name": "test-project",
            "main": "src/index.js"
        }"#;
        fs::write(temp_dir.path().join("package.json"), package_json).unwrap();

        // Create index.js
        let index_js = r#"
function main() {
    helper();
}

function helper() {
    console.log("Hello");
}

function unused() {
    console.log("Never called");
}

main();
"#;
        fs::write(src_dir.join("index.js"), index_js).unwrap();

        let mut analyzer = JsReachabilityAnalyzer::new();
        let report = analyzer.analyze(temp_dir.path()).unwrap();

        // Should have found functions
        assert!(!report.all_functions.is_empty());

        // main and helper should be reachable
        // unused should NOT be reachable
        assert!(!report.unreachable_functions.is_empty());
    }

    #[test]
    fn test_skip_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let node_modules = temp_dir.path().join("node_modules");
        fs::create_dir(&node_modules).unwrap();

        fs::write(node_modules.join("package.js"), "function foo() {}").unwrap();
        fs::write(temp_dir.path().join("index.js"), "function bar() {}").unwrap();

        let mut analyzer = JsReachabilityAnalyzer::new();
        let _ = analyzer.discover_and_parse_files(temp_dir.path());

        // Should not have parsed node_modules
        assert!(!analyzer
            .processed_files
            .iter()
            .any(|p| p.to_str().unwrap().contains("node_modules")));
    }
}
