//! AST parsing using tree-sitter for JavaScript and TypeScript

use crate::error::{JsReachabilityError, Result};
use std::path::Path;
use tree_sitter::{Language, Parser, Tree};

/// Parse a JavaScript or TypeScript file into an AST
pub fn parse_file(file_path: &Path) -> Result<Tree> {
    let source_code = std::fs::read_to_string(file_path)?;

    let language = determine_language(file_path)?;
    let mut parser = Parser::new();

    parser
        .set_language(&language)
        .map_err(|e| JsReachabilityError::ParseError(format!("Failed to set language: {}", e)))?;

    parser
        .parse(&source_code, None)
        .ok_or_else(|| {
            JsReachabilityError::ParseError(format!(
                "Failed to parse file: {}",
                file_path.display()
            ))
        })
}

/// Determine the language (JS vs TS) based on file extension
fn determine_language(file_path: &Path) -> Result<Language> {
    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match extension {
        "ts" | "tsx" => Ok(tree_sitter_typescript::language_typescript()),
        _ => Ok(tree_sitter_javascript::language()),
    }
}

/// Extract function definitions and calls from source code
pub fn extract_functions_and_calls(
    source_code: &str,
    tree: &Tree,
) -> Result<(Vec<ExtractedFunction>, Vec<ExtractedCall>)> {
    let mut functions = Vec::new();
    let mut calls = Vec::new();

    let root_node = tree.root_node();

    // Traverse the tree to find function declarations and calls
    visit_node(
        &root_node,
        source_code.as_bytes(),
        &mut functions,
        &mut calls,
    );

    Ok((functions, calls))
}

fn visit_node(
    node: &tree_sitter::Node,
    source: &[u8],
    functions: &mut Vec<ExtractedFunction>,
    calls: &mut Vec<ExtractedCall>,
) {
    let node_kind = node.kind();

    // Extract function declarations
    match node_kind {
        "function_declaration" | "function" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = get_node_text(name_node, source);
                let pos = node.start_position();

                functions.push(ExtractedFunction {
                    name,
                    line: pos.row + 1,
                    column: pos.column,
                    is_export: is_exported(node),
                });
            }
        }
        "arrow_function" => {
            // Arrow functions are typically assigned to variables
            if let Some(parent) = node.parent() {
                if parent.kind() == "variable_declarator" {
                    if let Some(name_node) = parent.child_by_field_name("name") {
                        let name = get_node_text(name_node, source);
                        let pos = node.start_position();

                        functions.push(ExtractedFunction {
                            name,
                            line: pos.row + 1,
                            column: pos.column,
                            is_export: is_exported(&parent),
                        });
                    }
                }
            }
        }
        "method_definition" => {
            if let Some(name_node) = node.child_by_field_name("name") {
                let name = get_node_text(name_node, source);
                let pos = node.start_position();

                functions.push(ExtractedFunction {
                    name,
                    line: pos.row + 1,
                    column: pos.column,
                    is_export: false,
                });
            }
        }
        "call_expression" => {
            if let Some(function_node) = node.child_by_field_name("function") {
                let callee = get_node_text(function_node, source);
                let pos = node.start_position();

                calls.push(ExtractedCall {
                    callee,
                    line: pos.row + 1,
                    column: pos.column,
                });
            }
        }
        _ => {}
    }

    // Recursively visit children
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_node(&child, source, functions, calls);
    }
}

fn get_node_text(node: tree_sitter::Node, source: &[u8]) -> String {
    node.utf8_text(source)
        .unwrap_or("")
        .to_string()
}

fn is_exported(node: &tree_sitter::Node) -> bool {
    if let Some(parent) = node.parent() {
        parent.kind() == "export_statement" || parent.kind() == "export_declaration"
    } else {
        false
    }
}

/// Extracted function information
#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub is_export: bool,
}

/// Extracted function call information
#[derive(Debug, Clone)]
pub struct ExtractedCall {
    pub callee: String,
    pub line: usize,
    pub column: usize,
}

/// Function extractor that combines parsing and extraction
pub struct FunctionExtractor {
    pub functions: Vec<ExtractedFunction>,
    pub calls: Vec<ExtractedCall>,
}

impl FunctionExtractor {
    pub fn new(_file_path: &str) -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
        }
    }

    pub fn extract(&mut self, source_code: &str, tree: &Tree) -> Result<()> {
        let (functions, calls) = extract_functions_and_calls(source_code, tree)?;
        self.functions = functions;
        self.calls = calls;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_simple_js() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.js");

        let code = r#"
function hello() {
    console.log("Hello");
}

function world() {
    hello();
}

world();
"#;

        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }

    #[test]
    fn test_extract_functions() {
        let code = r#"
function foo() {
    bar();
}

function bar() {
    console.log("bar");
}

foo();
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.js");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let (functions, calls) = extract_functions_and_calls(code, &tree).unwrap();

        assert_eq!(functions.len(), 2);
        assert!(functions.iter().any(|f| f.name == "foo"));
        assert!(functions.iter().any(|f| f.name == "bar"));

        // Check calls
        assert!(calls.len() >= 2); // bar(), console.log(), foo()
    }

    #[test]
    fn test_arrow_functions() {
        let code = r#"
const myFunc = () => {
    console.log("test");
};
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.js");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let (functions, _) = extract_functions_and_calls(code, &tree).unwrap();

        assert!(functions.iter().any(|f| f.name == "myFunc"));
    }
}
