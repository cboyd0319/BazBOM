//! AST parsing using tree-sitter for Ruby code analysis

use super::error::{Result, RubyReachabilityError};
use std::path::Path;
use tree_sitter::{Parser, Tree};

/// Parse a Ruby file into an AST
pub fn parse_file(file_path: &Path) -> Result<Tree> {
    let source_code = std::fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    let language = tree_sitter_ruby::LANGUAGE.into();
    parser.set_language(&language)?;

    parser.parse(&source_code, None).ok_or_else(|| {
        RubyReachabilityError::ParseError(format!(
            "Failed to parse Ruby file: {}",
            file_path.display()
        ))
    })
}

#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub is_class_method: bool,
    pub is_instance_method: bool,
    pub is_public: bool,
    pub class_name: Option<String>,
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
}

impl FunctionExtractor {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            has_dynamic_code: false,
            current_function: None,
            current_class: None,
        }
    }

    pub fn extract(&mut self, tree: &Tree, source: &[u8]) {
        let root_node = tree.root_node();
        self.visit_node(&root_node, source);
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        let node_kind = node.kind();

        match node_kind {
            "class" => self.extract_class(node, source),
            "method" => self.extract_method(node, source),
            "call" => self.extract_call(node, source),
            "singleton_method" => self.extract_singleton_method(node, source),
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source);
        }
    }

    fn extract_class(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = get_node_text(&name_node, source);
            let prev_class = self.current_class.clone();
            self.current_class = Some(class_name.clone());

            // Visit class body
            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_class = prev_class;
        }
    }

    fn extract_method(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let method_name = get_node_text(&name_node, source);
            let line = node.start_position().row + 1;

            // Check if method is private or protected
            let is_public = !self.is_private_or_protected(&method_name);

            let func = ExtractedFunction {
                name: method_name.clone(),
                line,
                is_class_method: false,
                is_instance_method: true,
                is_public,
                class_name: self.current_class.clone(),
            };

            self.functions.push(func);

            // Visit method body to extract calls
            let prev_function = self.current_function.clone();
            self.current_function = Some(method_name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_function = prev_function;
        }
    }

    fn extract_singleton_method(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let method_name = format!("self.{}", get_node_text(&name_node, source));
            let line = node.start_position().row + 1;

            let func = ExtractedFunction {
                name: method_name.clone(),
                line,
                is_class_method: true,
                is_instance_method: false,
                is_public: true,
                class_name: self.current_class.clone(),
            };

            self.functions.push(func);

            // Visit method body
            let prev_function = self.current_function.clone();
            self.current_function = Some(method_name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_function = prev_function;
        }
    }

    fn extract_call(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(method_node) = node.child_by_field_name("method") {
            let callee = get_node_text(&method_node, source);
            let line = node.start_position().row + 1;

            // Check for dynamic code patterns
            if matches!(
                callee.as_str(),
                "eval"
                    | "instance_eval"
                    | "class_eval"
                    | "module_eval"
                    | "define_method"
                    | "method_missing"
                    | "send"
                    | "__send__"
                    | "public_send"
                    | "const_get"
                    | "class_variable_get"
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

    fn is_private_or_protected(&self, _method_name: &str) -> bool {
        // Simplified - real implementation would track visibility modifiers
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
    fn test_parse_simple_ruby() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");

        let code = r#"
def hello
  puts "Hello"
end

def main
  hello
end
"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }

    #[test]
    fn test_extract_functions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");

        let code = r#"
def public_method
  helper
end

def helper
  puts "helper"
end

class MyClass
  def instance_method
    puts "instance"
  end

  def self.class_method
    puts "class method"
  end
end
"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        let source = fs::read(&file_path).unwrap();

        let mut extractor = FunctionExtractor::new();
        extractor.extract(&tree, &source);

        assert!(extractor.functions.len() >= 4);
        assert!(extractor
            .functions
            .iter()
            .any(|f| f.name == "public_method"));
        assert!(extractor
            .functions
            .iter()
            .any(|f| f.name == "instance_method"));
    }

    #[test]
    fn test_detect_dynamic_code() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rb");

        let code = r#"
def dynamic_code
  eval("some code")
end
"#;

        fs::write(&file_path, code).unwrap();
        let tree = parse_file(&file_path).unwrap();
        let source = fs::read(&file_path).unwrap();

        let mut extractor = FunctionExtractor::new();
        extractor.extract(&tree, &source);

        assert!(extractor.has_dynamic_code);
    }
}
