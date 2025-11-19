//! Python entrypoint detection

use crate::ast_parser::parse_file;
use crate::error::Result;
use crate::models::{Entrypoint, EntrypointType};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};
use walkdir::WalkDir;

/// Detects Python entrypoints in a project
pub struct EntrypointDetector {
    project_root: PathBuf,
}

impl EntrypointDetector {
    pub fn new(project_root: PathBuf) -> Self {
        Self { project_root }
    }

    /// Detect all entrypoints in the project
    pub fn detect_entrypoints(&self) -> Result<Vec<Entrypoint>> {
        info!("Detecting Python entrypoints");

        let mut entrypoints = Vec::new();

        // Walk through all Python files
        for entry in WalkDir::new(&self.project_root)
            .into_iter()
            .filter_entry(|e| !self.should_skip(e))
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                let path = entry.path();

                if self.is_python_file(path) {
                    if let Ok(file_entrypoints) = self.detect_in_file(path) {
                        entrypoints.extend(file_entrypoints);
                    }
                }
            }
        }

        info!("Found {} Python entrypoints", entrypoints.len());

        Ok(entrypoints)
    }

    /// Detect entrypoints in a single Python file
    fn detect_in_file(&self, file_path: &Path) -> Result<Vec<Entrypoint>> {
        let source = std::fs::read_to_string(file_path)?;
        let tree = parse_file(file_path)?;

        let mut entrypoints = Vec::new();
        let root_node = tree.root_node();

        self.visit_node(&root_node, &source, file_path, &mut entrypoints);

        Ok(entrypoints)
    }

    fn visit_node(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        let node_kind = node.kind();

        match node_kind {
            // Decorated functions (Flask, FastAPI, etc.)
            "decorated_definition" => {
                self.check_decorated_function(node, source, file_path, entrypoints);
            }
            // if __name__ == "__main__":
            "if_statement" => {
                if self.is_main_guard(node, source) {
                    debug!("Found __main__ guard in {:?}", file_path);
                    entrypoints.push(Entrypoint {
                        file: file_path.to_path_buf(),
                        function_name: "__main__".to_string(),
                        entrypoint_type: EntrypointType::Main,
                        metadata: HashMap::new(),
                    });
                }
            }
            // Test functions (test_*)
            "function_definition" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    let func_name = get_node_text(name_node, source);
                    if func_name.starts_with("test_") {
                        entrypoints.push(Entrypoint {
                            file: file_path.to_path_buf(),
                            function_name: func_name,
                            entrypoint_type: EntrypointType::PytestTest,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source, file_path, entrypoints);
        }
    }

    fn check_decorated_function(
        &self,
        node: &tree_sitter::Node,
        source: &str,
        file_path: &Path,
        entrypoints: &mut Vec<Entrypoint>,
    ) {
        // Extract decorator names
        let mut decorators = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                let dec_text = get_node_text(child, source);
                if let Some(stripped) = dec_text.strip_prefix('@') {
                    decorators.push(stripped.trim().to_string());
                }
            }
        }

        // Find function definition
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_definition" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let func_name = get_node_text(name_node, source);

                    // Check decorators for framework patterns
                    for decorator in &decorators {
                        if let Some(entrypoint) =
                            self.analyze_decorator(decorator, &func_name, file_path)
                        {
                            entrypoints.push(entrypoint);
                        }
                    }

                    // Check for test functions
                    if func_name.starts_with("test_") {
                        entrypoints.push(Entrypoint {
                            file: file_path.to_path_buf(),
                            function_name: func_name,
                            entrypoint_type: EntrypointType::PytestTest,
                            metadata: HashMap::new(),
                        });
                    }
                }
            }
        }
    }

    fn analyze_decorator(
        &self,
        decorator_str: &str,
        func_name: &str,
        file_path: &Path,
    ) -> Option<Entrypoint> {
        // Flask: @app.route("/path")
        if decorator_str.contains("route")
            && (decorator_str.starts_with("app.") || decorator_str.starts_with("blueprint."))
        {
            let mut metadata = HashMap::new();
            // Try to extract path from decorator string
            if let Some(start) = decorator_str.find('(') {
                if let Some(end) = decorator_str.find(')') {
                    let args = &decorator_str[start + 1..end];
                    if let Some(path) = args.split(',').next() {
                        metadata.insert(
                            "path".to_string(),
                            path.trim_matches(|c| c == '"' || c == '\'').to_string(),
                        );
                    }
                }
            }

            return Some(Entrypoint {
                file: file_path.to_path_buf(),
                function_name: func_name.to_string(),
                entrypoint_type: EntrypointType::FlaskRoute,
                metadata,
            });
        }

        // FastAPI: @app.get("/path"), @app.post("/path")
        if decorator_str.starts_with("app.get")
            || decorator_str.starts_with("app.post")
            || decorator_str.starts_with("app.put")
            || decorator_str.starts_with("app.delete")
            || decorator_str.starts_with("router.get")
            || decorator_str.starts_with("router.post")
        {
            let mut metadata = HashMap::new();
            if let Some(start) = decorator_str.find('(') {
                if let Some(end) = decorator_str.find(')') {
                    let args = &decorator_str[start + 1..end];
                    if let Some(path) = args.split(',').next() {
                        metadata.insert(
                            "path".to_string(),
                            path.trim_matches(|c| c == '"' || c == '\'').to_string(),
                        );
                    }
                }
            }

            return Some(Entrypoint {
                file: file_path.to_path_buf(),
                function_name: func_name.to_string(),
                entrypoint_type: EntrypointType::FastApiRoute,
                metadata,
            });
        }

        // Click: @click.command()
        if decorator_str.starts_with("click.") || decorator_str == "command" {
            return Some(Entrypoint {
                file: file_path.to_path_buf(),
                function_name: func_name.to_string(),
                entrypoint_type: EntrypointType::ClickCommand,
                metadata: HashMap::new(),
            });
        }

        // Celery: @app.task or @celery.task
        if decorator_str.contains("task")
            && (decorator_str.starts_with("app.") || decorator_str.starts_with("celery."))
        {
            return Some(Entrypoint {
                file: file_path.to_path_buf(),
                function_name: func_name.to_string(),
                entrypoint_type: EntrypointType::CeleryTask,
                metadata: HashMap::new(),
            });
        }

        None
    }

    fn is_main_guard(&self, node: &tree_sitter::Node, source: &str) -> bool {
        // Look for: if __name__ == "__main__":
        let condition = node.child_by_field_name("condition");
        if let Some(cond_node) = condition {
            let cond_text = get_node_text(cond_node, source);
            cond_text.contains("__name__") && cond_text.contains("__main__")
        } else {
            false
        }
    }

    /// Check if directory should be skipped
    fn should_skip(&self, entry: &walkdir::DirEntry) -> bool {
        let skip_dirs = [
            "venv",
            ".venv",
            "env",
            "__pycache__",
            ".git",
            ".tox",
            "node_modules",
            "dist",
            "build",
            ".pytest_cache",
        ];

        if entry.file_type().is_dir() {
            let dir_name = entry.file_name().to_str().unwrap_or("");
            skip_dirs.contains(&dir_name)
        } else {
            false
        }
    }

    /// Check if file is a Python file
    fn is_python_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            ext == "py"
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
    fn test_detect_main_guard() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("main.py");

        let code = r#"
def helper():
    print("helper")

if __name__ == "__main__":
    helper()
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
    fn test_detect_flask_route() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("app.py");

        let code = r#"
from flask import Flask
app = Flask(__name__)

@app.route("/users")
def get_users():
    return []
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        let flask_route = entrypoints
            .iter()
            .find(|e| e.entrypoint_type == EntrypointType::FlaskRoute);
        assert!(flask_route.is_some());
        assert_eq!(flask_route.unwrap().function_name, "get_users");
    }

    #[test]
    fn test_detect_fastapi_route() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("api.py");

        let code = r#"
from fastapi import FastAPI
app = FastAPI()

@app.get("/items")
async def get_items():
    return []
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(!entrypoints.is_empty());
        assert!(entrypoints
            .iter()
            .any(|e| e.entrypoint_type == EntrypointType::FastApiRoute));
    }

    #[test]
    fn test_detect_pytest_test() {
        let temp_dir = TempDir::new().unwrap();
        let test_dir = temp_dir.path().join("tests");
        fs::create_dir(&test_dir).unwrap();
        let file_path = test_dir.join("test_foo.py");

        let code = r#"
def test_addition():
    assert 1 + 1 == 2

def test_subtraction():
    assert 2 - 1 == 1
"#;

        fs::write(&file_path, code).unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        assert!(entrypoints.len() >= 2);
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "test_addition"));
        assert!(entrypoints
            .iter()
            .any(|e| e.function_name == "test_subtraction"));
    }

    #[test]
    fn test_skip_venv() {
        let temp_dir = TempDir::new().unwrap();
        let venv_dir = temp_dir.path().join("venv");
        fs::create_dir(&venv_dir).unwrap();

        fs::write(venv_dir.join("module.py"), "def foo(): pass").unwrap();
        fs::write(
            temp_dir.path().join("main.py"),
            "if __name__ == '__main__': pass",
        )
        .unwrap();

        let detector = EntrypointDetector::new(temp_dir.path().to_path_buf());
        let entrypoints = detector.detect_entrypoints().unwrap();

        // Should have found main.py but not venv/module.py
        assert_eq!(entrypoints.len(), 1);
    }
}
