//! Call graph data structure and reachability analysis for Go code

use crate::error::{GoReachabilityError, Result};
use crate::models::{FunctionId, FunctionNode};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use std::collections::HashMap;
use tracing::{debug, info};

/// Call graph for Go code
pub struct CallGraph {
    /// Directed graph where edges represent function calls
    graph: DiGraph<FunctionId, ()>,
    /// Map from function ID to graph node index
    node_indices: HashMap<FunctionId, NodeIndex>,
    /// Map from function ID to function metadata
    functions: HashMap<FunctionId, FunctionNode>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_indices: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Add a function to the call graph
    pub fn add_function(&mut self, function: FunctionNode) -> Result<()> {
        let id = function.id.clone();

        if self.node_indices.contains_key(&id) {
            return Ok(()); // Already exists
        }

        let node_idx = self.graph.add_node(id.clone());
        self.node_indices.insert(id.clone(), node_idx);
        self.functions.insert(id, function);

        Ok(())
    }

    /// Add a call edge from caller to callee
    pub fn add_call(&mut self, caller_id: &str, callee_id: &str) -> Result<()> {
        let caller_idx = self.node_indices.get(caller_id).ok_or_else(|| {
            GoReachabilityError::CallGraphError(format!("Caller not found: {}", caller_id))
        })?;

        let callee_idx = self.node_indices.get(callee_id).ok_or_else(|| {
            GoReachabilityError::CallGraphError(format!("Callee not found: {}", callee_id))
        })?;

        self.graph.add_edge(*caller_idx, *callee_idx, ());

        // Also update the function's calls list
        if let Some(function) = self.functions.get_mut(caller_id) {
            if !function.calls.contains(&callee_id.to_string()) {
                function.calls.push(callee_id.to_string());
            }
        }

        Ok(())
    }

    /// Mark a function as an entrypoint
    pub fn mark_entrypoint(&mut self, function_id: &str) -> Result<()> {
        self.functions
            .get_mut(function_id)
            .ok_or_else(|| {
                GoReachabilityError::CallGraphError(format!("Function not found: {}", function_id))
            })?
            .is_entrypoint = true;

        Ok(())
    }

    /// Mark all functions as reachable (conservative analysis for dynamic code)
    pub fn mark_all_reachable(&mut self) {
        for function in self.functions.values_mut() {
            function.reachable = true;
        }
    }

    /// Perform reachability analysis via DFS from entrypoints
    pub fn analyze_reachability(&mut self) -> Result<()> {
        info!("Starting Go reachability analysis");

        // Find all entrypoint nodes
        let entrypoint_ids: Vec<FunctionId> = self
            .functions
            .values()
            .filter(|f| f.is_entrypoint)
            .map(|f| f.id.clone())
            .collect();

        debug!("Found {} entrypoints", entrypoint_ids.len());

        // Perform DFS from each entrypoint
        for entrypoint_id in entrypoint_ids {
            if let Some(&start_idx) = self.node_indices.get(&entrypoint_id) {
                self.mark_reachable_from(start_idx)?;
            }
        }

        let reachable_count = self.functions.values().filter(|f| f.reachable).count();

        info!(
            "Reachability analysis complete: {} / {} functions reachable",
            reachable_count,
            self.functions.len()
        );

        Ok(())
    }

    /// Mark all functions reachable from a given node using DFS
    fn mark_reachable_from(&mut self, start_idx: NodeIndex) -> Result<()> {
        let mut dfs = Dfs::new(&self.graph, start_idx);

        while let Some(node_idx) = dfs.next(&self.graph) {
            let function_id = &self.graph[node_idx];

            if let Some(function) = self.functions.get_mut(function_id) {
                function.reachable = true;
            }
        }

        Ok(())
    }

    /// Find the call chain from an entrypoint to a target function
    pub fn find_call_chain(&self, target_id: &str) -> Option<Vec<String>> {
        // Find an entrypoint that can reach the target
        for entrypoint in self.functions.values().filter(|f| f.is_entrypoint) {
            if let Some(chain) = self.find_path(&entrypoint.id, target_id) {
                return Some(chain);
            }
        }

        None
    }

    /// Find a path from start to end using BFS
    fn find_path(&self, start_id: &str, end_id: &str) -> Option<Vec<String>> {
        use std::collections::VecDeque;

        let start_idx = self.node_indices.get(start_id)?;
        let end_idx = self.node_indices.get(end_id)?;

        let mut queue = VecDeque::new();
        let mut visited = HashMap::new();
        let mut parent: HashMap<NodeIndex, NodeIndex> = HashMap::new();

        queue.push_back(*start_idx);
        visited.insert(*start_idx, true);

        while let Some(current_idx) = queue.pop_front() {
            if current_idx == *end_idx {
                // Found! Reconstruct path
                let mut path = Vec::new();
                let mut node = current_idx;

                loop {
                    let id = &self.graph[node];
                    path.push(id.clone());

                    if let Some(&prev) = parent.get(&node) {
                        node = prev;
                    } else {
                        break;
                    }
                }

                path.reverse();
                return Some(path);
            }

            // Visit neighbors
            for neighbor_idx in self.graph.neighbors(current_idx) {
                if let std::collections::hash_map::Entry::Vacant(e) = visited.entry(neighbor_idx) {
                    e.insert(true);
                    parent.insert(neighbor_idx, current_idx);
                    queue.push_back(neighbor_idx);
                }
            }
        }

        None
    }

    /// Get all functions
    pub fn functions(&self) -> &HashMap<FunctionId, FunctionNode> {
        &self.functions
    }

    /// Get reachable functions
    pub fn reachable_functions(&self) -> Vec<&FunctionNode> {
        self.functions.values().filter(|f| f.reachable).collect()
    }

    /// Get unreachable functions
    pub fn unreachable_functions(&self) -> Vec<&FunctionNode> {
        self.functions.values().filter(|f| !f.reachable).collect()
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_call_graph_basic() {
        let mut graph = CallGraph::new();

        let main_fn = FunctionNode::new(
            "main".to_string(),
            "main".to_string(),
            PathBuf::from("test.py"),
            1,
            0,
        );

        let helper_fn = FunctionNode::new(
            "helper".to_string(),
            "helper".to_string(),
            PathBuf::from("test.py"),
            5,
            0,
        );

        graph.add_function(main_fn).unwrap();
        graph.add_function(helper_fn).unwrap();

        graph.add_call("main", "helper").unwrap();
        graph.mark_entrypoint("main").unwrap();

        graph.analyze_reachability().unwrap();

        let reachable = graph.reachable_functions();
        assert_eq!(reachable.len(), 2); // Both main and helper should be reachable
    }

    #[test]
    fn test_unreachable_code() {
        let mut graph = CallGraph::new();

        let main_fn = FunctionNode::new(
            "main".to_string(),
            "main".to_string(),
            PathBuf::from("test.py"),
            1,
            0,
        );

        let used_fn = FunctionNode::new(
            "used".to_string(),
            "used".to_string(),
            PathBuf::from("test.py"),
            5,
            0,
        );

        let unused_fn = FunctionNode::new(
            "unused".to_string(),
            "unused".to_string(),
            PathBuf::from("test.py"),
            10,
            0,
        );

        graph.add_function(main_fn).unwrap();
        graph.add_function(used_fn).unwrap();
        graph.add_function(unused_fn).unwrap();

        graph.add_call("main", "used").unwrap();
        graph.mark_entrypoint("main").unwrap();

        graph.analyze_reachability().unwrap();

        let reachable = graph.reachable_functions();
        let unreachable = graph.unreachable_functions();

        assert_eq!(reachable.len(), 2); // main and used
        assert_eq!(unreachable.len(), 1); // unused
        assert!(unreachable.iter().any(|f| f.name == "unused"));
    }

    #[test]
    fn test_find_call_chain() {
        let mut graph = CallGraph::new();

        graph
            .add_function(FunctionNode::new(
                "main".to_string(),
                "main".to_string(),
                PathBuf::from("test.py"),
                1,
                0,
            ))
            .unwrap();

        graph
            .add_function(FunctionNode::new(
                "a".to_string(),
                "a".to_string(),
                PathBuf::from("test.py"),
                5,
                0,
            ))
            .unwrap();

        graph
            .add_function(FunctionNode::new(
                "b".to_string(),
                "b".to_string(),
                PathBuf::from("test.py"),
                10,
                0,
            ))
            .unwrap();

        graph.add_call("main", "a").unwrap();
        graph.add_call("a", "b").unwrap();
        graph.mark_entrypoint("main").unwrap();

        let chain = graph.find_call_chain("b");
        assert!(chain.is_some());

        let chain = chain.unwrap();
        assert_eq!(chain, vec!["main", "a", "b"]);
    }

    #[test]
    fn test_mark_all_reachable() {
        let mut graph = CallGraph::new();

        graph
            .add_function(FunctionNode::new(
                "func1".to_string(),
                "func1".to_string(),
                PathBuf::from("test.py"),
                1,
                0,
            ))
            .unwrap();

        graph
            .add_function(FunctionNode::new(
                "func2".to_string(),
                "func2".to_string(),
                PathBuf::from("test.py"),
                5,
                0,
            ))
            .unwrap();

        // Before marking all reachable
        assert_eq!(graph.reachable_functions().len(), 0);

        // Mark all as reachable (conservative for dynamic code)
        graph.mark_all_reachable();

        // After marking all reachable
        assert_eq!(graph.reachable_functions().len(), 2);
    }
}
