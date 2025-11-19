//! AST parsing using tree-sitter for PHP code analysis

use super::error::{PhpReachabilityError, Result};
use std::path::Path;
use tree_sitter::{Parser, Tree};

/// Parse a PHP file into an AST
pub fn parse_file(file_path: &Path) -> Result<Tree> {
    let source_code = std::fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    let language = tree_sitter_php::LANGUAGE_PHP.into();
    parser.set_language(&language)?;

    parser.parse(&source_code, None).ok_or_else(|| {
        PhpReachabilityError::ParseError(format!(
            "Failed to parse PHP file: {}",
            file_path.display()
        ))
    })
}

#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub is_public: bool,
    pub is_static: bool,
    pub class_name: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ExtractedCall {
    pub callee: String,
    pub line: usize,
    pub caller_context: Option<String>,
}

pub struct FunctionExtractor {
    pub functions: Vec<ExtractedFunction>,
    pub calls: Vec<ExtractedCall>,
    pub has_dynamic_code: bool,
    current_function: Option<String>,
    current_class: Option<String>,
    current_namespace: Option<String>,
}

impl FunctionExtractor {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            has_dynamic_code: false,
            current_function: None,
            current_class: None,
            current_namespace: None,
        }
    }

    pub fn extract(&mut self, tree: &Tree, source: &[u8]) {
        let root_node = tree.root_node();
        self.visit_node(&root_node, source);
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        let node_kind = node.kind();

        match node_kind {
            "namespace_definition" => self.extract_namespace(node, source),
            "class_declaration" => self.extract_class(node, source),
            "function_definition" | "method_declaration" => self.extract_function(node, source),
            "function_call_expression" => self.extract_call(node, source),
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source);
        }
    }

    fn extract_namespace(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let namespace_name = get_node_text(&name_node, source);
            let prev_namespace = self.current_namespace.clone();
            self.current_namespace = Some(namespace_name);

            // Visit namespace body
            for child in node.named_children(&mut node.walk()) {
                if child.kind() != "name" {
                    self.visit_node(&child, source);
                }
            }

            self.current_namespace = prev_namespace;
        }
    }

    fn extract_class(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = get_node_text(&name_node, source);
            let prev_class = self.current_class.clone();
            self.current_class = Some(class_name);

            // Visit class body
            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_class = prev_class;
        }
    }

    fn extract_function(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let func_name = get_node_text(&name_node, source);
            let line = node.start_position().row + 1;

            // Check visibility
            let is_public = self.check_visibility(node, source);
            let is_static = self.check_static(node, source);

            let func = ExtractedFunction {
                name: func_name.clone(),
                line,
                is_public,
                is_static,
                class_name: self.current_class.clone(),
                namespace: self.current_namespace.clone(),
            };

            self.functions.push(func);

            // Visit function body
            let prev_function = self.current_function.clone();
            self.current_function = Some(func_name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_function = prev_function;
        }
    }

    fn extract_call(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(function_node) = node.child_by_field_name("function") {
            let callee = get_node_text(&function_node, source);
            let line = node.start_position().row + 1;

            // Check for dynamic code patterns
            if matches!(
                callee.as_str(),
                "eval"
                    | "assert"
                    | "create_function"
                    | "call_user_func"
                    | "call_user_func_array"
                    | "preg_replace"
                    | "include"
                    | "require"
            ) {
                self.has_dynamic_code = true;
            }

            self.calls.push(ExtractedCall {
                callee,
                line,
                caller_context: self.current_function.clone(),
            });
        }
    }

    fn check_visibility(&self, node: &tree_sitter::Node, source: &[u8]) -> bool {
        // Check for visibility modifiers (public/private/protected)
        for child in node.children(&mut node.walk()) {
            let text = get_node_text(&child, source);
            if text == "private" || text == "protected" {
                return false;
            }
        }
        true // Default to public
    }

    fn check_static(&self, node: &tree_sitter::Node, source: &[u8]) -> bool {
        for child in node.children(&mut node.walk()) {
            if get_node_text(&child, source) == "static" {
                return true;
            }
        }
        false
    }
}

impl Default for FunctionExtractor {
    fn default() -> Self {
        Self::new()
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
    fn test_parse_simple_php() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.php");

        let code = r#"<?php
function hello() {
    echo "Hello";
}

function main() {
    hello();
}
?>"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }

    #[test]
    fn test_extract_functions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.php");

        let code = r#"<?php
class MyClass {
    public function publicMethod() {
        echo "public";
    }

    private function privateMethod() {
        echo "private";
    }
}
?>"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        let source = fs::read(&file_path).unwrap();

        let mut extractor = FunctionExtractor::new();
        extractor.extract(&tree, &source);

        assert!(extractor.functions.len() >= 2);
        assert!(extractor.functions.iter().any(|f| f.name == "publicMethod"));
    }

    #[test]
    fn test_detect_dynamic_code() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.php");

        let code = r#"<?php
function dynamic() {
    eval($_GET['code']);
}
?>"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        let source = fs::read(&file_path).unwrap();

        let mut extractor = FunctionExtractor::new();
        extractor.extract(&tree, &source);

        assert!(extractor.has_dynamic_code);
    }
}
