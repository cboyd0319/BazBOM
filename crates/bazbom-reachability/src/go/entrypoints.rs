//! Go entrypoint detection

use super::ast_parser::parse_file;
use super::error::Result;
use super::models::{Entrypoint, EntrypointType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Detects Go entrypoints in a project
pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Detect all entrypoints in the project
    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        info!("Detecting Go entrypoints");

        let mut entrypoints = Vec::new();

        // Walk through all Go files
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !self.should_skip(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if self.is_go_file(path) {
                    if let Ok(file_entrypoints) = self.detect_in_file(path) {
                        entrypoints.extend(file_entrypoints);
                    }
                }
            }
        }

        info!("Found {} Go entrypoints", entrypoints.len());

        Ok(entrypoints)
    }

    /// Detect entrypoints in a single Go file
    fn detect_in_file(&self, file_path: &Path) -> Result<Vec<Entrypoint>> {
        let source = std::fs::read_to_string(file_path)?;
        let tree = parse_file(file_path)?;

        let mut entrypoints = Vec::new();
        let root_node = tree.root_node();

        // Check if this is package main
        let is_main_package = source.contains("package main");

        Self::visit_node(
            &root_node,
            &source,
            file_path,
            &mut entrypoints,
            is_main_package,
        );

        Ok(entrypoints)
    }

    fn visit_node(
        node: &tree_sitter::Node,
        source: &str,
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
        is_main_package: bool,
    ) {
        let node_kind = node.kind();

        if node_kind == "function_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let func_name = get_node_text(name_node, source);

                // func main() in package main
                if is_main_package && func_name == "main" {
                    debug!("Found main function in {:?}", file_path);
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: "main".to_string(),
                        entrypoint_type: EntrypointType::Main,
                        metadata: HashMap::new(),
                    });
                }

                // Test functions
                if func_name.starts_with("Test") {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: func_name.clone(),
                        entrypoint_type: EntrypointType::Test,
                        metadata: HashMap::new(),
                    });
                }

                // Benchmark functions
                if func_name.starts_with("Benchmark") {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: func_name,
                        entrypoint_type: EntrypointType::Benchmark,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::visit_node(&child, source, file_path, entrypoints, is_main_package);
        }
    }

    /// Check if directory should be skipped
    fn should_skip(&self, entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["vendor", ".git", "testdata", "node_modules"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    /// Check if file is a Go file
    fn is_go_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "go" && !path.to_str().unwrap_or("").ends_with("_test.go")
        } else {
            false
        }
    }
}

fn get_node_text(node: tree_sitter::Node, source: &str) -> String {
    node.utf8_text(source.as_bytes()).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_main() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.go");

        let code = r#"
package main

func helper() {
    println("helper")
}

func main() {
    helper()
}
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::Main));
    }

    #[test]
    fn test_detect_test_functions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("example.go");

        let code = r#"
package example

func TestAddition(t *testing.T) {
    // test code
}

func TestSubtraction(t *testing.T) {
    // test code
}
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(entrypoints.len() >= 2);
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "TestAddition"));
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "TestSubtraction"));
    }
}
