//! AST parsing using tree-sitter for Go code analysis

use crate::error::{GoReachabilityError, Result};
use crate::models::ReflectionType;
use std::path::Path;
use tree_sitter::{Parser, Tree};

/// Parse a Go file into an AST
pub fn parse_file(file_path: &Path) -> Result<Tree> {
    let source_code = std::fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    let language = tree_sitter_go::language();

    parser
        .set_language(&language)
        .map_err(|e| GoReachabilityError::ParseError(format!("Failed to set language: {}", e)))?;

    parser
        .parse(&source_code, None)
        .ok_or_else(|| {
            GoReachabilityError::ParseError(format!(
                "Failed to parse file: {}",
                file_path.display()
            ))
        })
}

/// Extracted function information from Go AST
#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub is_method: bool,
    pub receiver_type: Option<String>,
    pub is_exported: bool,
}

/// Extracted function call information
#[derive(Debug, Clone)]
pub struct ExtractedCall {
    pub callee: String,
    pub line: usize,
    pub column: usize,
    /// The function that contains this call
    pub caller_context: Option<String>,
    /// Whether this is a goroutine launch (go func())
    pub is_goroutine: bool,
}

/// Detected reflection or dynamic code pattern
#[derive(Debug, Clone)]
pub struct ReflectionDetection {
    pub line: usize,
    pub reflection_type: ReflectionType,
    pub description: String,
}

/// Function extractor that walks the Go AST
pub struct FunctionExtractor {
    pub functions: Vec<ExtractedFunction>,
    pub calls: Vec<ExtractedCall>,
    pub reflections: Vec<ReflectionDetection>,
    current_function: Option<String>,
}

impl FunctionExtractor {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            reflections: Vec::new(),
            current_function: None,
        }
    }

    pub fn extract(&mut self, source_code: &str, tree: &Tree) -> Result<()> {
        let root_node = tree.root_node();
        self.visit_node(&root_node, source_code.as_bytes());
        Ok(())
    }

    fn visit_node(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        let node_kind = node.kind();

        match node_kind {
            "function_declaration" => {
                self.extract_function(node, source);
            }
            "method_declaration" => {
                self.extract_method(node, source);
            }
            "call_expression" => {
                self.extract_call(node, source);
            }
            "go_statement" => {
                self.extract_goroutine(node, source);
            }
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source);
        }
    }

    fn extract_function(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(name_node, source);
            let pos = node.start_position();

            // Check if exported (starts with uppercase)
            let is_exported = name.chars().next().is_some_and(|c| c.is_uppercase());

            let function = ExtractedFunction {
                name: name.clone(),
                line: pos.row + 1,
                column: pos.column,
                is_method: false,
                receiver_type: None,
                is_exported,
            };

            self.functions.push(function);

            // Visit function body with context
            let prev_function = self.current_function.clone();
            self.current_function = Some(name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_function = prev_function;
        }
    }

    fn extract_method(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        let receiver_type = node.child_by_field_name("receiver")
            .and_then(|receiver_node| {
                // Extract receiver type from (r *ReceiverType)
                let receiver_text = get_node_text(receiver_node, source);
                // Simple extraction - just get the type name
                receiver_text.split_whitespace().last().map(|s| s.trim_end_matches(')').to_string())
            });

        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(name_node, source);
            let pos = node.start_position();

            let is_exported = name.chars().next().is_some_and(|c| c.is_uppercase());

            let function = ExtractedFunction {
                name: name.clone(),
                line: pos.row + 1,
                column: pos.column,
                is_method: true,
                receiver_type: receiver_type.clone(),
                is_exported,
            };

            self.functions.push(function);

            // Visit method body with context
            let prev_function = self.current_function.clone();
            self.current_function = Some(name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_function = prev_function;
        }
    }

    fn extract_call(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(function_node) = node.child_by_field_name("function") {
            let callee = get_node_text(function_node, source);
            let pos = node.start_position();

            // Check for reflection patterns
            if callee.contains("reflect.Value.Call") || callee.contains(".Call(") {
                self.reflections.push(ReflectionDetection {
                    line: pos.row + 1,
                    reflection_type: ReflectionType::ReflectCall,
                    description: "reflect.Value.Call() detected - conservative analysis".to_string(),
                });
            } else if callee.contains("MethodByName") {
                self.reflections.push(ReflectionDetection {
                    line: pos.row + 1,
                    reflection_type: ReflectionType::MethodByName,
                    description: "reflect.Value.MethodByName() detected".to_string(),
                });
            } else if callee.contains("FieldByName") {
                self.reflections.push(ReflectionDetection {
                    line: pos.row + 1,
                    reflection_type: ReflectionType::FieldByName,
                    description: "reflect.Value.FieldByName() detected".to_string(),
                });
            }

            self.calls.push(ExtractedCall {
                callee,
                line: pos.row + 1,
                column: pos.column,
                caller_context: self.current_function.clone(),
                is_goroutine: false,
            });
        }
    }

    fn extract_goroutine(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        // go statement contains a call_expression
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "call_expression" {
                if let Some(function_node) = child.child_by_field_name("function") {
                    let callee = get_node_text(function_node, source);
                    let pos = child.start_position();

                    self.calls.push(ExtractedCall {
                        callee,
                        line: pos.row + 1,
                        column: pos.column,
                        caller_context: self.current_function.clone(),
                        is_goroutine: true,
                    });
                }
            }
        }
    }
}

fn get_node_text(node: tree_sitter::Node, source: &[u8]) -> String {
    node.utf8_text(source).unwrap_or("").to_string()
}

impl Default for FunctionExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_go() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");

        let code = r#"
package main

func hello() {
    println("Hello")
}

func main() {
    hello()
}
"#;

        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }

    #[test]
    fn test_extract_functions() {
        let code = r#"
package main

func foo() {
    bar()
}

func bar() {
    println("bar")
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        assert_eq!(extractor.functions.len(), 2);
        assert!(extractor.functions.iter().any(|f| f.name == "foo"));
        assert!(extractor.functions.iter().any(|f| f.name == "bar"));
    }

    #[test]
    fn test_extract_methods() {
        let code = r#"
package main

type MyStruct struct{}

func (m *MyStruct) Method() {
    println("method")
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        let method = extractor.functions.iter().find(|f| f.name == "Method").unwrap();
        assert!(method.is_method);
        assert!(method.receiver_type.is_some());
    }

    #[test]
    fn test_goroutine_detection() {
        let code = r#"
package main

func worker() {
    println("working")
}

func main() {
    go worker()
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        assert!(extractor.calls.iter().any(|c| c.is_goroutine));
    }

    #[test]
    fn test_exported_functions() {
        let code = r#"
package mypackage

func PublicFunc() {}
func privateFunc() {}
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.go");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        let public = extractor.functions.iter().find(|f| f.name == "PublicFunc").unwrap();
        let private = extractor.functions.iter().find(|f| f.name == "privateFunc").unwrap();

        assert!(public.is_exported);
        assert!(!private.is_exported);
    }
}
