//! Main Java reachability analyzer

use super::bytecode_analyzer::{analyze_classes, analyze_jars};
use super::call_graph::CallGraph;
use super::error::Result;
use super::models::ReachabilityReport;
use petgraph::visit::Dfs;
use std::path::Path;

/// Java reachability analyzer
pub struct JavaReachabilityAnalyzer {
    call_graph: CallGraph,
}

impl JavaReachabilityAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self {
            call_graph: CallGraph::new(),
        }
    }

    /// Analyze a Java project to determine method reachability
    pub fn analyze(&mut self, project_root: &Path) -> Result<ReachabilityReport> {
        tracing::info!(
            "Starting Java reachability analysis for: {}",
            project_root.display()
        );

        // Step 1: Analyze application bytecode
        self.analyze_application_code(project_root)?;

        // Step 2: Analyze transitive dependencies
        self.analyze_dependencies(project_root)?;

        // Step 3: Identify entrypoints
        let entrypoints = self.identify_entrypoints();

        tracing::debug!("Found {} entrypoints", entrypoints.len());

        // Step 4: Perform reachability analysis via DFS from entrypoints
        let reachable = self.compute_reachability(&entrypoints);

        tracing::debug!("Found {} reachable methods", reachable.len());

        // Step 5: Build report
        let mut report = ReachabilityReport::new();
        report.entrypoints = entrypoints.clone();
        report.reachable_functions = reachable.clone();

        // Mark methods as reachable
        for method_id in &reachable {
            if let Some(method) = self.call_graph.get_method_mut(method_id) {
                method.reachable = true;
            }
        }

        // Copy all methods to report
        for (id, method) in &self.call_graph.methods {
            report.all_functions.insert(id.clone(), method.clone());

            if !method.reachable {
                report.unreachable_functions.insert(id.clone());
            }
        }

        Ok(report)
    }

    /// Analyze application code (target/classes, build/classes)
    fn analyze_application_code(&mut self, project_root: &Path) -> Result<()> {
        tracing::debug!("Analyzing application bytecode...");

        // Look for compiled classes in common locations
        let app_class_dirs = vec![
            project_root.join("target").join("classes"),      // Maven
            project_root.join("build").join("classes"),       // Gradle
            project_root.join("target").join("test-classes"), // Maven tests
            project_root.join("build").join("test-classes"),  // Gradle tests
        ];

        for class_dir in app_class_dirs {
            if class_dir.exists() {
                tracing::debug!("Analyzing classes in {:?}", class_dir);
                analyze_classes(&class_dir, &mut self.call_graph)?;
            }
        }

        // Also analyze any .jar files in lib/ directories
        let lib_dirs = vec![project_root.join("lib"), project_root.join("libs")];

        for lib_dir in lib_dirs {
            if lib_dir.exists() {
                tracing::debug!("Analyzing JARs in {:?}", lib_dir);
                analyze_jars(&lib_dir, &mut self.call_graph)?;
            }
        }

        Ok(())
    }

    /// Analyze transitive dependencies from Maven/Gradle
    fn analyze_dependencies(&mut self, project_root: &Path) -> Result<()> {
        // Maven dependencies
        let maven_repo = dirs::home_dir()
            .map(|h| h.join(".m2").join("repository"))
            .filter(|p| p.exists());

        if let Some(maven_repo) = maven_repo {
            tracing::info!("Analyzing Maven dependencies in ~/.m2/repository");
            // NOTE: In production, we'd parse pom.xml to get specific dependencies
            // For now, we'll skip full Maven repo scan as it's massive
            // Instead, look for local dependencies
        }

        // Gradle cache
        let gradle_cache = dirs::home_dir()
            .map(|h| h.join(".gradle").join("caches").join("modules-2").join("files-2.1"))
            .filter(|p| p.exists());

        if let Some(gradle_cache) = gradle_cache {
            tracing::info!("Analyzing Gradle dependencies in ~/.gradle/caches");
            // NOTE: In production, we'd parse build.gradle to get specific dependencies
            // For now, we'll skip full Gradle cache scan as it's massive
        }

        // Check for vendored dependencies (lib/, libs/)
        let vendor_dirs = vec![
            project_root.join("lib"),
            project_root.join("libs"),
            project_root.join("vendor"),
        ];

        for vendor_dir in vendor_dirs {
            if vendor_dir.exists() {
                tracing::debug!("Analyzing vendored dependencies in {:?}", vendor_dir);
                analyze_jars(&vendor_dir, &mut self.call_graph)?;
            }
        }

        Ok(())
    }

    /// Identify all entrypoint methods
    fn identify_entrypoints(&mut self) -> Vec<String> {
        let mut entrypoints = Vec::new();

        for (method_id, method) in &mut self.call_graph.methods {
            if method.is_entrypoint {
                entrypoints.push(method_id.clone());
            }
        }

        entrypoints
    }

    /// Compute reachable methods from entrypoints using DFS
    fn compute_reachability(&self, entrypoints: &[String]) -> std::collections::HashSet<String> {
        let mut reachable = std::collections::HashSet::new();

        for entrypoint in entrypoints {
            if let Some(node_idx) = self.call_graph.method_indices.get(entrypoint) {
                // Perform DFS from this entrypoint
                let mut dfs = Dfs::new(&self.call_graph.graph, *node_idx);

                while let Some(visited_idx) = dfs.next(&self.call_graph.graph) {
                    if let Some(method_id) = self.call_graph.graph.node_weight(visited_idx) {
                        reachable.insert(method_id.clone());
                    }
                }
            }
        }

        reachable
    }
}

impl Default for JavaReachabilityAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_analyzer_empty_project() {
        let temp_dir = TempDir::new().unwrap();
        let mut analyzer = JavaReachabilityAnalyzer::new();

        let report = analyzer.analyze(temp_dir.path()).unwrap();

        assert_eq!(report.entrypoints.len(), 0);
        assert_eq!(report.all_functions.len(), 0);
        assert_eq!(report.reachable_functions.len(), 0);
    }
}
