use std::collections::{HashMap, VecDeque};

/// Dependency graph for analyzing transitive dependencies
pub struct DependencyGraph {
    /// Map of parent ID -> list of child IDs (for forward traversal/future use)
    #[allow(dead_code)]
    adjacency: HashMap<String, Vec<String>>,
    /// Map of child ID -> list of parent IDs (for finding roots)
    parents: HashMap<String, Vec<String>>,
    /// Map of artifact ID -> artifact name
    artifacts: HashMap<String, String>,
    /// Map of artifact name -> list of artifact IDs (for reverse lookup)
    name_to_ids: HashMap<String, Vec<String>>,
}

impl DependencyGraph {
    /// Build dependency graph from Syft SBOM
    pub fn new(sbom: &serde_json::Value) -> Self {
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
        let mut parents: HashMap<String, Vec<String>> = HashMap::new();
        let mut artifacts: HashMap<String, String> = HashMap::new();
        let mut name_to_ids: HashMap<String, Vec<String>> = HashMap::new();

        // Parse artifacts
        if let Some(artifact_list) = sbom["artifacts"].as_array() {
            for artifact in artifact_list {
                if let (Some(id), Some(name)) = (artifact["id"].as_str(), artifact["name"].as_str())
                {
                    artifacts.insert(id.to_string(), name.to_string());
                    name_to_ids
                        .entry(name.to_string())
                        .or_default()
                        .push(id.to_string());
                }
            }
        }

        // Parse relationships
        if let Some(relationships) = sbom["relationships"].as_array() {
            for rel in relationships {
                let type_ = rel["type"].as_str().unwrap_or("");

                if let (Some(parent), Some(child)) = (rel["parent"].as_str(), rel["child"].as_str())
                {
                    if type_ == "dependency-of" {
                        // child depends on parent? No, "dependency-of" means 'child' is a dependency OF 'parent'
                        // So parent is the consumer, child is the dependency.
                        // We want to traverse from Root -> ... -> Vulnerable Package
                        // So we want adjacency: Parent -> Child
                        adjacency
                            .entry(parent.to_string())
                            .or_default()
                            .push(child.to_string());

                        parents
                            .entry(child.to_string())
                            .or_default()
                            .push(parent.to_string());
                    } else if type_ == "contains" {
                        // parent contains child
                        adjacency
                            .entry(parent.to_string())
                            .or_default()
                            .push(child.to_string());

                        parents
                            .entry(child.to_string())
                            .or_default()
                            .push(parent.to_string());
                    }
                }
            }
        }

        Self {
            adjacency,
            parents,
            artifacts,
            name_to_ids,
        }
    }

    /// Find shortest path from any root to the target package
    pub fn find_path(&self, target_package: &str) -> Option<Vec<String>> {
        // Get all IDs for this package name
        let target_ids = self.name_to_ids.get(target_package)?;

        let mut shortest_path: Option<Vec<String>> = None;

        for target_id in target_ids {
            if let Some(path) = self.bfs_to_root(target_id) {
                if shortest_path.is_none() || path.len() < shortest_path.as_ref().unwrap().len() {
                    shortest_path = Some(path);
                }
            }
        }

        shortest_path
    }

    /// BFS backwards from node to a root
    fn bfs_to_root(&self, start_id: &str) -> Option<Vec<String>> {
        let mut queue = VecDeque::new();
        queue.push_back(vec![start_id.to_string()]);
        let mut visited = std::collections::HashSet::new();
        visited.insert(start_id.to_string());

        while let Some(path) = queue.pop_front() {
            let current = path.last().unwrap();

            // Check if this is a root (no parents)
            if !self.parents.contains_key(current) || self.parents[current].is_empty() {
                // Found a path to root!
                // Convert IDs to names
                let named_path: Vec<String> = path
                    .iter()
                    .rev() // Reverse because we went child -> parent
                    .filter_map(|id| self.artifacts.get(id).cloned())
                    .collect();
                return Some(named_path);
            }

            if let Some(parents) = self.parents.get(current) {
                for parent in parents {
                    if !visited.contains(parent) {
                        visited.insert(parent.to_string());
                        let mut new_path = path.clone();
                        new_path.push(parent.to_string());
                        queue.push_back(new_path);
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_dependency_graph() {
        let sbom = json!({
            "artifacts": [
                {"id": "root", "name": "my-app"},
                {"id": "lib-a", "name": "library-a"},
                {"id": "lib-b", "name": "library-b"},
                {"id": "vuln-pkg", "name": "vulnerable-package"}
            ],
            "relationships": [
                // my-app -> lib-a
                {"parent": "root", "child": "lib-a", "type": "dependency-of"},
                // lib-a -> lib-b
                {"parent": "lib-a", "child": "lib-b", "type": "dependency-of"},
                // lib-b -> vuln-pkg
                {"parent": "lib-b", "child": "vuln-pkg", "type": "dependency-of"}
            ]
        });

        let graph = DependencyGraph::new(&sbom);

        // Test path to vulnerable package
        let path = graph.find_path("vulnerable-package");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(
            path,
            vec!["my-app", "library-a", "library-b", "vulnerable-package"]
        );

        // Test path to direct dependency
        let path = graph.find_path("library-a");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path, vec!["my-app", "library-a"]);

        // Test non-existent package
        let path = graph.find_path("non-existent");
        assert!(path.is_none());
    }
}
