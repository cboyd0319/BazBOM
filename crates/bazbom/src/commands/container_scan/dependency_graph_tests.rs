
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
        assert_eq!(path, vec!["my-app", "library-a", "library-b"]);

        // Test path to direct dependency
        let path = graph.find_path("library-a");
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path, vec!["my-app"]);

        // Test path to root (should be empty or contain itself? Logic says path *to* root, but bfs_to_root returns path *from* root)
        // bfs_to_root returns [root, ..., parent]
        // find_path returns that.
        
        // Test non-existent package
        let path = graph.find_path("non-existent");
        assert!(path.is_none());
    }
}
