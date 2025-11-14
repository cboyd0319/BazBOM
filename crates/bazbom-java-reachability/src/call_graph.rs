//! Call graph data structure for Java methods

use crate::models::{MethodId, MethodNode};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Call graph representing method invocations
pub struct CallGraph {
    /// The actual graph structure
    pub graph: DiGraph<MethodId, ()>,
    /// Maps method IDs to graph node indices
    pub method_indices: HashMap<MethodId, NodeIndex>,
    /// Maps method IDs to method details
    pub methods: HashMap<MethodId, MethodNode>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            method_indices: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    /// Add a method to the call graph
    pub fn add_method(&mut self, method: MethodNode) {
        let method_id = method.id.clone();

        // Add to graph if not already present
        if !self.method_indices.contains_key(&method_id) {
            let node_idx = self.graph.add_node(method_id.clone());
            self.method_indices.insert(method_id.clone(), node_idx);
        }

        // Store method details
        self.methods.insert(method_id, method);
    }

    /// Add a call edge from caller to callee
    pub fn add_call(&mut self, caller_id: &str, callee_id: &str) {
        // Ensure both methods exist
        let caller_idx = self.method_indices.get(caller_id).copied();
        let callee_idx = self.method_indices.get(callee_id).copied();

        if let (Some(from), Some(to)) = (caller_idx, callee_idx) {
            self.graph.add_edge(from, to, ());

            // Update calls list in method node
            if let Some(method) = self.methods.get_mut(caller_id) {
                if !method.calls.contains(&callee_id.to_string()) {
                    method.calls.push(callee_id.to_string());
                }
            }
        }
    }

    /// Get all method IDs
    pub fn method_ids(&self) -> Vec<&MethodId> {
        self.methods.keys().collect()
    }

    /// Get a method by ID
    pub fn get_method(&self, id: &str) -> Option<&MethodNode> {
        self.methods.get(id)
    }

    /// Get a mutable method by ID
    pub fn get_method_mut(&mut self, id: &str) -> Option<&mut MethodNode> {
        self.methods.get_mut(id)
    }
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}
