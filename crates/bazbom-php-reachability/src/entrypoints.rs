//! PHP entrypoint detection

use crate::ast_parser::parse_file;
use crate::error::Result;
use crate::models::{Entrypoint, EntrypointType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        info!("Detecting PHP entrypoints");

        let mut entrypoints = Vec::new();

        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !Self::should_skip(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if Self::is_php_file(path) {
                    if let Ok(file_entrypoints) = self.detect_in_file(path) {
                        entrypoints.extend(file_entrypoints);
                    }
                }
            }
        }

        info!("Found {} PHP entrypoints", entrypoints.len());

        Ok(entrypoints)
    }

    fn detect_in_file(&self, file_path: &Path) -> Result<Vec<Entrypoint>> {
        let source = std::fs::read(file_path)?;
        let source_str = String::from_utf8_lossy(&source);
        let tree = parse_file(file_path)?;

        let mut entrypoints = Vec::new();
        let root_node = tree.root_node();

        let path_str = file_path.to_string_lossy();

        // Symfony controllers
        if path_str.contains("Controller") && source_str.contains("Route") {
            Self::extract_methods(
                &root_node,
                &source,
                file_path,
                EntrypointType::SymfonyController,
                &mut entrypoints,
            );
        }

        // Laravel controllers
        if path_str.contains("app/Http/Controllers") {
            Self::extract_methods(
                &root_node,
                &source,
                file_path,
                EntrypointType::LaravelController,
                &mut entrypoints,
            );
        }

        // WordPress actions/filters
        if source_str.contains("add_action") || source_str.contains("add_filter") {
            debug!("Found WordPress hooks in {:?}", file_path);
            entrypoints.push(Entrypoint {
                file: file_path.to_path_buf(),
                function_name: "wordpress_hooks".to_string(),
                entrypoint_type: EntrypointType::WordPressAction,
                metadata: HashMap::new(),
            });
        }

        // PHPUnit tests
        if path_str.ends_with("Test.php") || source_str.contains("PHPUnit") {
            Self::extract_test_methods(&root_node, &source, file_path, &mut entrypoints);
        }

        Ok(entrypoints)
    }

    fn extract_methods(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entry_type: EntrypointType,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "method_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let method_name = get_node_text(&name_node, source);
                entrypoints.push(Entrypoint {
                    file: file_path.to_path_buf(),
                    function_name: method_name,
                    entrypoint_type: entry_type.clone(),
                    metadata: HashMap::new(),
                });
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::extract_methods(&child, source, file_path, entry_type.clone(), entrypoints);
        }
    }

    fn extract_test_methods(
        node: &tree_sitter::Node,
        source: &[u8],
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        if node.kind() == "method_declaration" {
            if let Some(name_node) = node.child_by_field_name("name") {
                let method_name = get_node_text(&name_node, source);
                if method_name.starts_with("test") {
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: method_name,
                        entrypoint_type: EntrypointType::PHPUnitTest,
                        metadata: HashMap::new(),
                    });
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            Self::extract_test_methods(&child, source, file_path, entrypoints);
        }
    }

    fn should_skip(entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = ["vendor", ".git", "node_modules", "cache", "storage"];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    fn is_php_file(path: &Path) -> bool {
        path.extension().and_then(|s| s.to_str()) == Some("php")
    }
}

fn get_node_text(node: &tree_sitter::Node, source: &[u8]) -> String {
    node.utf8_text(source).unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_laravel_controller() {
        let temp_dir = TempDir::new().unwrap();
        let controllers_dir = temp_dir.path().join("app/Http/Controllers");
        fs::create_dir_all(&controllers_dir).unwrap();

        let file_path = controllers_dir.join("UserController.php");

        let code = r#"<?php
class UserController {
    public function index() {
        return view('users.index');
    }
}
?>"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::LaravelController));
    }

    #[test]
    fn test_detect_phpunit_test() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("ExampleTest.php");

        let code = r#"<?php
use PHPUnit\Framework\TestCase;

class ExampleTest extends TestCase {
    public function testAddition() {
        $this->assertEquals(4, 2 + 2);
    }
}
?>"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "testAddition"));
    }
}
