//! Main orchestrator for Python reachability analysis

use super::ast_parser::{parse_file, FunctionExtractor};
use super::call_graph::CallGraph;
use super::entrypoints::EntrypointDetector;
use super::error::Result;
use super::models::{
    DynamicCodeWarning, FunctionNode, ReachabilityReport, VulnerabilityReachability,
};
use super::module_resolver::ModuleResolver;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Main analyzer for Python reachability
pub struct PythonReachabilityAnalyzer {
    call_graph: CallGraph,
    module_resolver: ModuleResolver,
    processed_files: HashSet<PathBuf>,
    dynamic_warnings: Vec<DynamicCodeWarning>,
    has_dynamic_code: bool,
}

impl PythonReachabilityAnalyzer {
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::new(),
            module_resolver: ModuleResolver::new(PathBuf::new()),
            processed_files: HashSet::new(),
            dynamic_warnings: Vec::new(),
            has_dynamic_code: false,
        }
    }

    /// Analyze a Python project for reachability
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        info!("Starting Python reachability analysis");
        info!("Project root: {:?}", project_root);

        // Initialize module resolver with project root
        self.module_resolver = ModuleResolver::new(project_root.to_path_buf());

        // 1. Detect entrypoints
        let entrypoint_detector = EntrypointDetector::new(project_root.to_path_buf());
        let entrypoints = entrypoint_detector.detect_entrypoints()?;

        info!("Found {} entrypoints", entrypoints.len());

        // 2. Discover and parse all Python files
        self.discover_and_parse_files(project_root)?;

        // 3. Mark entrypoints in the call graph
        for entrypoint in &entrypoints {
            // Create entrypoint ID
            let entrypoint_id = if entrypoint.function_name == "__main__" {
                format!("{}:__main__", entrypoint.file.display())
            } else {
                format!("{}:{}", entrypoint.file.display(), entrypoint.function_name)
            };

            if let Err(e) = self.call_graph.mark_entrypoint(&entrypoint_id) {
                debug!("Could not mark entrypoint {}: {}", entrypoint_id, e);
            }
        }

        // 4. Perform reachability analysis
        // If dynamic code detected, mark all as reachable (conservative)
        if self.has_dynamic_code {
            warn!(
                "Dynamic code detected - using conservative analysis (all code marked reachable)"
            );
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

        if !self.dynamic_warnings.is_empty() {
            warn!(
                "Found {} dynamic code warnings",
                self.dynamic_warnings.len()
            );
        }

        Ok(report)
    }

    /// Discover and parse all Python files in the project
    fn discover_and_parse_files(&mut self, project_root: &Path) -> Result<()> {
        info!("Discovering and parsing Python files...");

        // Check if this is inside a virtual environment
        let is_in_venv = project_root
            .to_str()
            .map(|s| s.contains("site-packages") || s.contains("dist-packages"))
            .unwrap_or(false);

        // First, parse application code
        self.discover_and_parse_application_files(project_root)?;

        // Then, parse transitive dependencies if not already in venv
        if !is_in_venv {
            self.discover_and_parse_dependency_files(project_root)?;
        }

        Ok(())
    }

    /// Parse application source files (skip venv, __pycache__, etc.)
    fn discover_and_parse_application_files(&mut self, project_root: &Path) -> Result<()> {
        info!("Parsing application files...");

        // Directories to skip for application code
        let skip_dirs = [
            "venv",
            ".venv",
            "env",
            "__pycache__",
            ".git",
            ".tox",
            "node_modules",
            "dist",
            "build",
            ".pytest_cache",
            ".mypy_cache",
            ".eggs",
            "*.egg-info",
        ];

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

                if self.is_python_file(path) && !self.processed_files.contains(path) {
                    if let Err(e) = self.parse_and_build_graph(path) {
                        debug!("Failed to parse {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Parse transitive dependency files in venv/site-packages
    fn discover_and_parse_dependency_files(&mut self, project_root: &Path) -> Result<()> {
        // Look for common venv locations
        let venv_candidates = vec![
            project_root.join("venv"),
            project_root.join(".venv"),
            project_root.join("env"),
        ];

        let mut site_packages: Option<PathBuf> = None;

        for venv_path in &venv_candidates {
            if !venv_path.exists() {
                continue;
            }

            // Find site-packages within venv
            // Could be: venv/lib/python3.X/site-packages (Unix)
            // Or: venv/Lib/site-packages (Windows)
            let lib_dir = venv_path.join("lib");
            if lib_dir.exists() {
                for entry in WalkDir::new(&lib_dir)
                    .max_depth(2)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    if entry.file_type().is_dir()
                        && entry.file_name().to_str() == Some("site-packages")
                    {
                        site_packages = Some(entry.path().to_path_buf());
                        break;
                    }
                }
            }

            // Windows path
            let lib_dir_win = venv_path.join("Lib").join("site-packages");
            if lib_dir_win.exists() {
                site_packages = Some(lib_dir_win);
            }

            if site_packages.is_some() {
                break;
            }
        }

        let site_packages = match site_packages {
            Some(sp) => sp,
            None => {
                info!("No virtual environment found, skipping dependency analysis");
                return Ok(());
            }
        };

        info!("Parsing transitive dependencies in {:?}...", site_packages);

        // Skip directories even within site-packages
        let skip_dirs = ["__pycache__", "tests", "test", ".dist-info", ".egg-info"];

        for entry in WalkDir::new(&site_packages)
            .into_iter()
            .filter_entry(|e| {
                if e.file_type().is_dir() {
                    let dir_name = e.file_name().to_str().unwrap_or("");
                    !skip_dirs.iter().any(|&skip| dir_name.contains(skip))
                } else {
                    true
                }
            })
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                // Skip test files
                let path_str = path.to_str().unwrap_or("");
                if path_str.contains("/test_") || path_str.contains("/tests/") {
                    continue;
                }

                if self.is_python_file(path) && !self.processed_files.contains(path) {
                    if let Err(e) = self.parse_and_build_graph(path) {
                        debug!("Failed to parse dependency {}: {}", path.display(), e);
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

        // Check for dynamic code
        if !extractor.dynamic_code.is_empty() {
            self.has_dynamic_code = true;
            for detection in &extractor.dynamic_code {
                self.dynamic_warnings.push(DynamicCodeWarning {
                    file: file_path.to_path_buf(),
                    line: detection.line,
                    warning_type: detection.dynamic_type.clone(),
                    description: detection.description.clone(),
                });
            }
        }

        // Add functions to call graph
        for func in &extractor.functions {
            // Build function ID
            let function_id = if let Some(class_name) = &func.class_name {
                format!("{}:{}.{}", file_path.display(), class_name, func.name)
            } else {
                format!("{}:{}", file_path.display(), func.name)
            };

            let mut function_node = FunctionNode::new(
                function_id.clone(),
                func.name.clone(),
                file_path.to_path_buf(),
                func.line,
                func.column,
            );

            function_node.class_name = func.class_name.clone();
            function_node.is_async = func.is_async;
            function_node.decorators = func.decorators.clone();

            self.call_graph.add_function(function_node)?;
        }

        // Add call edges
        for call in &extractor.calls {
            // Determine caller ID
            let caller_id = if let Some(caller_context) = &call.caller_context {
                format!("{}:{}", file_path.display(), caller_context)
            } else {
                // Top-level call - use a synthetic module-level caller
                format!("{}:__module__", file_path.display())
            };

            // Resolve the callee - try both within-file and cross-module
            let mut resolved = false;

            // First try within-file call
            let within_file_id = format!("{}:{}", file_path.display(), call.callee);
            if self.call_graph.functions().contains_key(&within_file_id) {
                let _ = self.call_graph.add_call(&caller_id, &within_file_id);
                resolved = true;
            }

            // If not found, try cross-module resolution
            if !resolved {
                // Extract module name from call if it's a qualified call (e.g., module.function)
                if let Some(dot_pos) = call.callee.rfind('.') {
                    let module_part = &call.callee[..dot_pos];
                    let func_part = &call.callee[dot_pos + 1..];

                    // Try to resolve the module
                    if let Ok(module_files) =
                        self.module_resolver.resolve_import(module_part, file_path)
                    {
                        for module_file in module_files {
                            let cross_module_id =
                                format!("{}:{}", module_file.display(), func_part);
                            if self.call_graph.functions().contains_key(&cross_module_id) {
                                let _ = self.call_graph.add_call(&caller_id, &cross_module_id);
                                resolved = true;
                                break;
                            }
                        }
                    }
                }
            }

            // If still not resolved, try as-is (might be stdlib or external)
            if !resolved {
                let _ = self.call_graph.add_call(&caller_id, &call.callee);
            }
        }

        self.processed_files.insert(file_path.to_path_buf());
        Ok(())
    }

    /// Generate the final reachability report
    fn generate_report(
        &self,
        entrypoints: Vec<super::models::Entrypoint>,
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
            .map(|e| {
                if e.function_name == "__main__" {
                    format!("{}:__main__", e.file.display())
                } else {
                    format!("{}:{}", e.file.display(), e.function_name)
                }
            })
            .collect();

        Ok(ReachabilityReport {
            all_functions,
            reachable_functions,
            unreachable_functions,
            entrypoints: entrypoint_ids,
            // Vulnerabilities are populated by bazbom-polyglot's reachability_integration module
            vulnerabilities: Vec::new(),
            dynamic_code_warnings: self.dynamic_warnings.clone(),
        })
    }

    /// Check if a vulnerability is reachable
    pub fn check_vulnerability_reachability(
        &self,
        package: &str,
        vulnerable_function: &str,
    ) -> Option<VulnerabilityReachability> {
        // Search for the vulnerable function in the call graph
        // Look in site-packages or venv paths
        let vulnerable_id = format!("site-packages/{}:{}", package, vulnerable_function);

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

    fn is_python_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            ext == "py"
        } else {
            false
        }
    }
}

impl Default for PythonReachabilityAnalyzer {
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

        // Create main.py
        let main_py = r#"
def helper():
    print("helper")

def unused():
    print("unused")

if __name__ == "__main__":
    helper()
"#;
        fs::write(temp_dir.path().join("main.py"), main_py).unwrap();

        let mut analyzer = PythonReachabilityAnalyzer::new();
        let report = analyzer.analyze(temp_dir.path()).unwrap();

        // Should have found functions
        assert!(
            report.all_functions.len() >= 2,
            "Should have found at least 2 functions"
        );

        // Should have found entrypoints
        assert!(
            !report.entrypoints.is_empty(),
            "Should have found __main__ entrypoint"
        );
    }

    #[test]
    fn test_skip_venv() {
        let temp_dir = TempDir::new().unwrap();
        let venv = temp_dir.path().join("venv");
        fs::create_dir(&venv).unwrap();

        fs::write(venv.join("package.py"), "def foo(): pass").unwrap();
        fs::write(
            temp_dir.path().join("main.py"),
            "def bar(): pass\nif __name__ == '__main__': bar()",
        )
        .unwrap();

        let mut analyzer = PythonReachabilityAnalyzer::new();
        let _ = analyzer.discover_and_parse_files(temp_dir.path());

        // Should not have parsed venv
        assert!(!analyzer
            .processed_files
            .iter()
            .any(|p| p.to_str().unwrap().contains("venv")));
    }

    #[test]
    fn test_dynamic_code_warning() {
        let temp_dir = TempDir::new().unwrap();

        let code = r#"
def dangerous():
    exec("print('danger')")

if __name__ == "__main__":
    dangerous()
"#;
        fs::write(temp_dir.path().join("main.py"), code).unwrap();

        let mut analyzer = PythonReachabilityAnalyzer::new();
        let report = analyzer.analyze(temp_dir.path()).unwrap();

        // Should have dynamic code warning
        assert!(!report.dynamic_code_warnings.is_empty());

        // Should have marked all as reachable due to dynamic code
        assert_eq!(report.unreachable_functions.len(), 0);
    }
}
