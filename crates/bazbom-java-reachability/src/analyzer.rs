//! Main Java reachability analyzer

use crate::bytecode_analyzer::{analyze_classes, analyze_jars};
use crate::call_graph::CallGraph;
use crate::error::Result;
use crate::models::ReachabilityReport;
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
        tracing::info!("Starting Java reachability analysis for: {}", project_root.display());

        // Step 1: Build call graph from bytecode
        tracing::debug!("Analyzing .class files...");
        analyze_classes(project_root, &mut self.call_graph)?;

        tracing::debug!("Analyzing .jar files...");
        analyze_jars(project_root, &mut self.call_graph)?;

        // Step 2: Identify entrypoints
        let entrypoints = self.identify_entrypoints();

        tracing::debug!("Found {} entrypoints", entrypoints.len());

        // Step 3: Perform reachability analysis via DFS from entrypoints
        let reachable = self.compute_reachability(&entrypoints);

        tracing::debug!("Found {} reachable methods", reachable.len());

        // Step 4: Build report
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
