//! Main Rust reachability analyzer

use super::ast_parser::{parse_file, FunctionExtractor};
use super::call_graph::CallGraph;
use super::dependency_resolver::{Dependency, DependencyResolver};
use super::entrypoints::EntrypointDetector;
use super::error::Result;
use super::models::{FunctionNode, ReachabilityReport, VulnerabilityReachability};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tracing::info;
use walkdir::WalkDir;

pub struct RustReachabilityAnalyzer {
    project_root: PathBuf,
    call_graph: CallGraph,
    dependencies: Vec<Dependency>,
    // Map crate name to source path
    crate_sources: HashMap<String, PathBuf>,
}

impl RustReachabilityAnalyzer {
    pub fn new(project_root: PathBuf) -> Self {
        Self {
            project_root,
            call_graph: CallGraph::new(),
            dependencies: Vec::new(),
            crate_sources: HashMap::new(),
        }
    }

    /// Run complete reachability analysis (including transitive dependencies)
    pub fn analyze(&mut self) -> Result<ReachabilityReport> {
        let cargo_lock_path = self.project_root.join("Cargo.lock");
        let has_cargo_lock = cargo_lock_path.exists();

        if has_cargo_lock {
            info!("Starting Rust transitive reachability analysis (root project)");
        } else {
            info!("Starting Rust reachability analysis (dependency crate, no transitive analysis)");
        }

        // Step 1: Resolve dependencies from Cargo.lock (only for root projects)
        if has_cargo_lock {
            info!("Resolving dependencies from Cargo.lock");
            self.resolve_dependencies()?;
            info!("Found {} dependencies", self.dependencies.len());
        }

        // Step 2: Detect entrypoints
        let entrypoints = self.detect_entrypoints()?;
        info!("Found {} entrypoints", entrypoints.len());

        // Step 3: Build call graph for this crate's source
        info!("Building call graph for application");
        self.build_call_graph()?;

        // Step 4: Build call graphs for dependencies (only for root projects)
        if has_cargo_lock && !self.dependencies.is_empty() {
            info!("Building call graphs for dependencies");
            self.build_dependency_call_graphs()?;
        }

        info!(
            "Built unified call graph with {} functions",
            self.call_graph.functions.len()
        );

        // Step 5: Mark entrypoints
        for entrypoint in &entrypoints {
            let func_id = format!(
                "{}::{}",
                entrypoint.file.display(),
                entrypoint.function_name
            );
            if let Some(func) = self.call_graph.functions.get_mut(&func_id) {
                func.is_entrypoint = true;
            }
        }

        // Step 6: Analyze reachability
        self.call_graph.analyze_reachability()?;

        // Step 7: Generate report
        let report = self.generate_report()?;

        if has_cargo_lock {
            info!(
                "Analysis complete: {}/{} functions reachable (including transitive deps)",
                report.reachable_functions.len(),
                report.all_functions.len()
            );
        } else {
            info!(
                "Analysis complete: {}/{} functions reachable",
                report.reachable_functions.len(),
                report.all_functions.len()
            );
        }

        Ok(report)
    }

    fn detect_entrypoints(&self) -> Result<Vec<super::models::Entrypoint>> {
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
        // Try to parse the file, but handle parse errors gracefully
        let ast = match parse_file(file_path) {
            Ok(ast) => ast,
            Err(e) => {
                // Log parse error but continue processing other files
                tracing::warn!(
                    "Skipping {} due to parse error: {}",
                    file_path.display(),
                    e
                );
                return Ok(()); // Return Ok to continue processing other files
            }
        };

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
        // Enhanced resolution with crate support
        // 1. Check if it's a qualified path (contains ::)
        if callee.contains("::") {
            // Extract crate name from qualified path
            // e.g., "chrono::Utc::now" -> check if "chrono" is a known crate
            let parts: Vec<&str> = callee.split("::").collect();
            if !parts.is_empty() {
                let potential_crate = parts[0];
                // If it's a known dependency crate, keep the full path
                if self.crate_sources.contains_key(potential_crate) {
                    return callee.to_string();
                }
            }
            return callee.to_string();
        }

        // Simple name - assume same file
        format!("{}::{}", current_file.display(), callee)
    }

    /// Resolve dependencies from Cargo.lock
    fn resolve_dependencies(&mut self) -> Result<()> {
        let resolver = DependencyResolver::new(self.project_root.clone());
        self.dependencies = resolver.resolve_dependencies()?;

        // Build map of crate name -> source path
        for dep in &self.dependencies {
            if let Some(ref source_path) = dep.source_path {
                self.crate_sources.insert(dep.name.clone(), source_path.clone());
                tracing::debug!(
                    "Mapped crate {} to source {:?}",
                    dep.name,
                    source_path
                );
            }
        }

        Ok(())
    }

    /// Build call graphs for all dependencies
    fn build_dependency_call_graphs(&mut self) -> Result<()> {
        // Clone to avoid borrow checker issues
        let deps = self.dependencies.clone();

        for dep in &deps {
            if let Some(ref source_path) = dep.source_path {
                tracing::info!("Analyzing dependency: {} @ {}", dep.name, dep.version);
                self.build_call_graph_for_crate(&dep.name, source_path)?;
            } else {
                tracing::warn!(
                    "Skipping {} @ {} - source not available",
                    dep.name,
                    dep.version
                );
            }
        }
        Ok(())
    }

    /// Build call graph for a specific crate (dependency)
    fn build_call_graph_for_crate(&mut self, crate_name: &str, crate_path: &Path) -> Result<()> {
        // Collect all Rust files in the crate
        let rust_files: Vec<PathBuf> = WalkDir::new(crate_path)
            .into_iter()
            .filter_entry(|e| !Self::should_skip_entry(e))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .filter(|p| Self::is_rust_file(p))
            .collect();

        tracing::debug!(
            "Found {} source files for crate {}",
            rust_files.len(),
            crate_name
        );

        // Process all files in the crate
        for file_path in rust_files {
            // Parse the file and extract functions, handling parse errors gracefully
            let ast = match parse_file(&file_path) {
                Ok(ast) => ast,
                Err(e) => {
                    // Log parse error but continue processing other files
                    tracing::debug!(
                        "Skipping {} in crate {} due to parse error: {}",
                        file_path.display(),
                        crate_name,
                        e
                    );
                    continue; // Skip this file and move to the next
                }
            };

            let mut extractor = FunctionExtractor::new();
            extractor.extract(&ast);

            // Add functions to call graph with crate-qualified names
            for func in &extractor.functions {
                // Create fully qualified function ID: crate::module::function
                let func_id = if func.is_pub {
                    // Public functions get crate-qualified names
                    format!("{}::{}", crate_name, func.name)
                } else {
                    // Private functions include file path
                    format!("{}::{}::{}", crate_name, file_path.display(), func.name)
                };

                let function_node = FunctionNode {
                    id: func_id.clone(),
                    name: func.name.clone(),
                    file: file_path.clone(),
                    line: func.line,
                    column: 0,
                    is_entrypoint: false,
                    reachable: false,
                    calls: Vec::new(),
                    is_pub: func.is_pub,
                    is_async: func.is_async,
                    is_test: func.is_test,
                };

                self.call_graph.add_function(function_node);
            }

            // Add call edges within this crate
            for call in &extractor.calls {
                if let Some(caller_context) = &call.caller_context {
                    let caller_id = format!("{}::{}", crate_name, caller_context);
                    let callee_id = self.resolve_function_call(&call.callee, &file_path);
                    self.call_graph.add_call(&caller_id, &callee_id);
                }
            }
        }

        Ok(())
    }

    fn generate_report(&self) -> Result<ReachabilityReport> {
        let all_functions = self.call_graph.functions.clone();

        let reachable_functions: HashSet<_> = all_functions
            .values()
            .filter(|f| f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let unreachable_functions: HashSet<_> = all_functions
            .values()
            .filter(|f| !f.reachable)
            .map(|f| f.id.clone())
            .collect();

        let entrypoints: Vec<_> = all_functions
            .values()
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
        vulnerabilities
            .into_iter()
            .map(|mut vuln| {
                // Check if any vulnerable function is reachable
                let is_reachable = vuln.vulnerable_functions.iter().any(|func_name| {
                    self.call_graph
                        .functions
                        .values()
                        .any(|f| f.name.contains(func_name) && f.reachable)
                });

                vuln.reachable = is_reachable;

                // Try to find call chain if reachable
                if is_reachable {
                    for func_name in &vuln.vulnerable_functions {
                        if let Some(func) = self
                            .call_graph
                            .functions
                            .values()
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
            })
            .collect()
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
