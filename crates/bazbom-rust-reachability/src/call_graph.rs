//! Call graph construction and reachability analysis

use crate::error::Result;
use crate::models::{FunctionId, FunctionNode};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tracing::{debug, info};

pub struct CallGraph {
    graph: DiGraph<FunctionId, ()>,
    pub functions: HashMap<FunctionId, FunctionNode>,
    node_indices: HashMap<FunctionId, NodeIndex>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            functions: HashMap::new(),
            node_indices: HashMap::new(),
        }
    }

    /// Add a function to the call graph
    pub fn add_function(&mut self, function: FunctionNode) {
        let func_id = function.id.clone();

        if !self.node_indices.contains_key(&func_id) {
            let idx = self.graph.add_node(func_id.clone());
            self.node_indices.insert(func_id.clone(), idx);
        }

        self.functions.insert(func_id, function);
    }

    /// Add a call edge from caller to callee
    pub fn add_call(&mut self, caller_id: &FunctionId, callee_id: &FunctionId) {
        let caller_idx = self.get_or_create_node(caller_id);
        let callee_idx = self.get_or_create_node(callee_id);

        self.graph.add_edge(caller_idx, callee_idx, ());
    }

    fn get_or_create_node(&mut self, func_id: &FunctionId) -> NodeIndex {
        if let Some(&idx) = self.node_indices.get(func_id) {
            idx
        } else {
            let idx = self.graph.add_node(func_id.clone());
            self.node_indices.insert(func_id.clone(), idx);
            idx
        }
    }

    /// Analyze reachability from all entrypoints
    pub fn analyze_reachability(&mut self) -> Result<()> {
        info!("Analyzing Rust code reachability");

        let entrypoints: Vec<_> = self.functions.values()
            .filter(|f| f.is_entrypoint)
            .map(|f| f.id.clone())
            .collect();

        debug!("Found {} Rust entrypoints", entrypoints.len());

        for entrypoint_id in &entrypoints {
            if let Some(&idx) = self.node_indices.get(entrypoint_id) {
                self.mark_reachable_from(idx);
            }
        }

        Ok(())
    }

    fn mark_reachable_from(&mut self, start: NodeIndex) {
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![start];

        while let Some(node_idx) = stack.pop() {
            if !visited.insert(node_idx) {
                continue;
            }

            if let Some(func_id) = self.graph.node_weight(node_idx) {
                if let Some(func) = self.functions.get_mut(func_id) {
                    func.reachable = true;
                }
            }

            // Add all callees to stack
            for neighbor in self.graph.neighbors(node_idx) {
                if !visited.contains(&neighbor) {
                    stack.push(neighbor);
                }
            }
        }
    }

    /// Mark all functions as reachable (conservative analysis)
    pub fn mark_all_reachable(&mut self) {
        for function in self.functions.values_mut() {
            function.reachable = true;
        }
    }

    /// Find shortest call chain from entrypoint to target
    pub fn find_call_chain(&self, target_id: &FunctionId) -> Option<Vec<FunctionId>> {
        use petgraph::algo::dijkstra;
        use petgraph::Direction;

        let target_idx = self.node_indices.get(target_id)?;

        // Find all entrypoints
        let entrypoints: Vec<_> = self.functions.values()
            .filter(|f| f.is_entrypoint)
            .filter_map(|f| self.node_indices.get(&f.id))
            .copied()
            .collect();

        // Try each entrypoint and find shortest path
        for &entry_idx in &entrypoints {
            let distances = dijkstra(&self.graph, entry_idx, Some(*target_idx), |_| 1);

            if distances.contains_key(target_idx) {
                // Reconstruct path
                let mut path = vec![*target_idx];
                let mut current = *target_idx;

                while current != entry_idx {
                    let predecessors: Vec<_> = self.graph
                        .neighbors_directed(current, Direction::Incoming)
                        .collect();

                    if let Some(&pred) = predecessors.iter()
                        .find(|&&pred| distances.contains_key(&pred)
                            && distances[&pred] == distances[&current] - 1)
                    {
                        path.push(pred);
                        current = pred;
                    } else {
                        break;
                    }
                }

                path.reverse();

                // Convert indices to function IDs
                let func_chain: Vec<_> = path.iter()
                    .filter_map(|&idx| self.graph.node_weight(idx))
                    .cloned()
                    .collect();

                return Some(func_chain);
            }
        }

        None
    }

    /// Get reachability statistics
    pub fn get_stats(&self) -> CallGraphStats {
        let total = self.functions.len();
        let reachable = self.functions.values().filter(|f| f.reachable).count();
        let unreachable = total - reachable;

        CallGraphStats {
            total_functions: total,
            reachable_functions: reachable,
            unreachable_functions: unreachable,
            entrypoints: self.functions.values().filter(|f| f.is_entrypoint).count(),
        }
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct CallGraphStats {
    pub total_functions: usize,
    pub reachable_functions: usize,
    pub unreachable_functions: usize,
    pub entrypoints: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn create_test_function(id: &str, name: &str) -> FunctionNode {
        FunctionNode {
            id: id.to_string(),
            name: name.to_string(),
            file: PathBuf::from("test.rs"),
            line: 1,
            column: 0,
            is_entrypoint: false,
            reachable: false,
            calls: Vec::new(),
            is_pub: false,
            is_async: false,
            is_test: false,
        }
    }

    #[test]
    fn test_add_function() {
        let mut graph = CallGraph::new();
        let func = create_test_function("main", "main");
        graph.add_function(func);

        assert_eq!(graph.functions.len(), 1);
        assert!(graph.functions.contains_key("main"));
    }

    #[test]
    fn test_add_call() {
        let mut graph = CallGraph::new();

        let mut main_func = create_test_function("main", "main");
        main_func.is_entrypoint = true;
        graph.add_function(main_func);

        let helper_func = create_test_function("helper", "helper");
        graph.add_function(helper_func);

        graph.add_call(&"main".to_string(), &"helper".to_string());

        assert_eq!(graph.graph.edge_count(), 1);
    }

    #[test]
    fn test_reachability_simple() {
        let mut graph = CallGraph::new();

        let mut main_func = create_test_function("main", "main");
        main_func.is_entrypoint = true;
        graph.add_function(main_func);

        let helper_func = create_test_function("helper", "helper");
        graph.add_function(helper_func);

        let unused_func = create_test_function("unused", "unused");
        graph.add_function(unused_func);

        graph.add_call(&"main".to_string(), &"helper".to_string());

        graph.analyze_reachability().unwrap();

        assert!(graph.functions.get("main").unwrap().reachable);
        assert!(graph.functions.get("helper").unwrap().reachable);
        assert!(!graph.functions.get("unused").unwrap().reachable);
    }

    #[test]
    fn test_find_call_chain() {
        let mut graph = CallGraph::new();

        let mut main_func = create_test_function("main", "main");
        main_func.is_entrypoint = true;
        graph.add_function(main_func);

        let func_a = create_test_function("func_a", "func_a");
        graph.add_function(func_a);

        let func_b = create_test_function("func_b", "func_b");
        graph.add_function(func_b);

        graph.add_call(&"main".to_string(), &"func_a".to_string());
        graph.add_call(&"func_a".to_string(), &"func_b".to_string());

        let chain = graph.find_call_chain(&"func_b".to_string()).unwrap();
        assert_eq!(chain, vec!["main", "func_a", "func_b"]);
    }

    #[test]
    fn test_mark_all_reachable() {
        let mut graph = CallGraph::new();

        let func1 = create_test_function("func1", "func1");
        graph.add_function(func1);

        let func2 = create_test_function("func2", "func2");
        graph.add_function(func2);

        graph.mark_all_reachable();

        assert!(graph.functions.get("func1").unwrap().reachable);
        assert!(graph.functions.get("func2").unwrap().reachable);
    }

    #[test]
    fn test_stats() {
        let mut graph = CallGraph::new();

        let mut main_func = create_test_function("main", "main");
        main_func.is_entrypoint = true;
        graph.add_function(main_func);

        let reachable_func = create_test_function("reachable", "reachable");
        graph.add_function(reachable_func);

        let unreachable_func = create_test_function("unreachable", "unreachable");
        graph.add_function(unreachable_func);

        graph.add_call(&"main".to_string(), &"reachable".to_string());
        graph.analyze_reachability().unwrap();

        let stats = graph.get_stats();
        assert_eq!(stats.total_functions, 3);
        assert_eq!(stats.reachable_functions, 2);
        assert_eq!(stats.unreachable_functions, 1);
        assert_eq!(stats.entrypoints, 1);
    }
}
