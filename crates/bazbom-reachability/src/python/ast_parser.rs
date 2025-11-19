//! AST parsing using tree-sitter for Python code analysis

use super::error::{PythonReachabilityError, Result};
use super::models::DynamicCodeType;
use std::path::Path;
use tree_sitter::{Parser, Tree};

/// Parse a Python file into an AST
pub fn parse_file(file_path: &Path) -> Result<Tree> {
    let source_code = std::fs::read_to_string(file_path)?;

    let mut parser = Parser::new();
    let language = tree_sitter_python::LANGUAGE.into();

    parser.set_language(&language).map_err(|e| {
        PythonReachabilityError::ParseError(format!("Failed to set language: {}", e))
    })?;

    parser.parse(&source_code, None).ok_or_else(|| {
        PythonReachabilityError::ParseError(format!(
            "Failed to parse file: {}",
            file_path.display()
        ))
    })
}

/// Extracted function information from Python AST
#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub is_async: bool,
    pub is_method: bool,
    pub class_name: Option<String>,
    pub decorators: Vec<String>,
}

/// Extracted function call information
#[derive(Debug, Clone)]
pub struct ExtractedCall {
    pub callee: String,
    pub line: usize,
    pub column: usize,
    /// The function/method that contains this call
    pub caller_context: Option<String>,
}

/// Detected dynamic code pattern
#[derive(Debug, Clone)]
pub struct DynamicCodeDetection {
    pub line: usize,
    pub dynamic_type: DynamicCodeType,
    pub description: String,
}

/// Extracted import information
#[derive(Debug, Clone)]
pub struct ExtractedImport {
    pub module: String,
    pub imported_names: Vec<String>, // Empty for "import foo", contains names for "from foo import bar"
    pub alias: Option<String>,
}

/// Function extractor that walks the Python AST
pub struct FunctionExtractor {
    pub functions: Vec<ExtractedFunction>,
    pub calls: Vec<ExtractedCall>,
    pub dynamic_code: Vec<DynamicCodeDetection>,
    pub imports: Vec<ExtractedImport>,
    current_class: Option<String>,
    current_function: Option<String>,
}

impl FunctionExtractor {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            dynamic_code: Vec::new(),
            imports: Vec::new(),
            current_class: None,
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
            "function_definition" => {
                self.extract_function(node, source, false);
            }
            "class_definition" => {
                self.extract_class(node, source);
            }
            "call" => {
                self.extract_call(node, source);
            }
            "decorated_definition" => {
                self.extract_decorated_definition(node, source);
            }
            _ => {}
        }

        // Recursively visit children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(&child, source);
        }
    }

    fn extract_function(&mut self, node: &tree_sitter::Node, source: &[u8], is_async: bool) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = get_node_text(name_node, source);
            let pos = node.start_position();

            let function = ExtractedFunction {
                name: name.clone(),
                line: pos.row + 1,
                column: pos.column,
                is_async,
                is_method: self.current_class.is_some(),
                class_name: self.current_class.clone(),
                decorators: Vec::new(), // Will be filled by decorated_definition
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

    fn extract_class(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(name_node) = node.child_by_field_name("name") {
            let class_name = get_node_text(name_node, source);

            let prev_class = self.current_class.clone();
            self.current_class = Some(class_name);

            if let Some(body) = node.child_by_field_name("body") {
                self.visit_node(&body, source);
            }

            self.current_class = prev_class;
        }
    }

    fn extract_decorated_definition(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        let mut decorators = Vec::new();

        // Extract decorators
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                // Get the decorator expression (skip the @ symbol)
                let decorator_text = get_node_text(child, source);
                if let Some(stripped) = decorator_text.strip_prefix('@') {
                    decorators.push(stripped.trim().to_string());
                }
            }
        }

        // Find the actual definition
        cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "function_definition" {
                // Extract function with decorators
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = get_node_text(name_node, source);
                    let pos = child.start_position();

                    let function = ExtractedFunction {
                        name: name.clone(),
                        line: pos.row + 1,
                        column: pos.column,
                        is_async: false,
                        is_method: self.current_class.is_some(),
                        class_name: self.current_class.clone(),
                        decorators: decorators.clone(),
                    };

                    self.functions.push(function);

                    // Visit body
                    let prev_function = self.current_function.clone();
                    self.current_function = Some(name);

                    if let Some(body) = child.child_by_field_name("body") {
                        self.visit_node(&body, source);
                    }

                    self.current_function = prev_function;
                }
            }
        }
    }

    fn extract_call(&mut self, node: &tree_sitter::Node, source: &[u8]) {
        if let Some(function_node) = node.child_by_field_name("function") {
            let callee = get_node_text(function_node, source);
            let pos = node.start_position();

            // Check for dynamic code patterns
            match callee.as_str() {
                "exec" => {
                    self.dynamic_code.push(DynamicCodeDetection {
                        line: pos.row + 1,
                        dynamic_type: DynamicCodeType::Exec,
                        description: "exec() call detected - conservative analysis".to_string(),
                    });
                }
                "eval" => {
                    self.dynamic_code.push(DynamicCodeDetection {
                        line: pos.row + 1,
                        dynamic_type: DynamicCodeType::Eval,
                        description: "eval() call detected - conservative analysis".to_string(),
                    });
                }
                "getattr" => {
                    self.dynamic_code.push(DynamicCodeDetection {
                        line: pos.row + 1,
                        dynamic_type: DynamicCodeType::Getattr,
                        description: "getattr() with variable attribute - conservative analysis"
                            .to_string(),
                    });
                }
                "setattr" => {
                    self.dynamic_code.push(DynamicCodeDetection {
                        line: pos.row + 1,
                        dynamic_type: DynamicCodeType::Setattr,
                        description: "setattr() with variable attribute - conservative analysis"
                            .to_string(),
                    });
                }
                "__import__" => {
                    self.dynamic_code.push(DynamicCodeDetection {
                        line: pos.row + 1,
                        dynamic_type: DynamicCodeType::DynamicImport,
                        description: "__import__() with variable module name".to_string(),
                    });
                }
                _ => {}
            }

            self.calls.push(ExtractedCall {
                callee,
                line: pos.row + 1,
                column: pos.column,
                caller_context: self.current_function.clone(),
            });
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
    fn test_parse_simple_python() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");

        let code = r#"
def hello():
    print("Hello")

def world():
    hello()

world()
"#;

        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        assert!(tree.root_node().child_count() > 0);
    }

    #[test]
    fn test_extract_functions() {
        let code = r#"
def foo():
    bar()

def bar():
    print("bar")

foo()
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        assert_eq!(extractor.functions.len(), 2);
        assert!(extractor.functions.iter().any(|f| f.name == "foo"));
        assert!(extractor.functions.iter().any(|f| f.name == "bar"));

        // Check calls
        assert!(extractor.calls.len() >= 2); // bar(), print(), foo()
    }

    #[test]
    fn test_class_methods() {
        let code = r#"
class MyClass:
    def method(self):
        print("method")
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        let method = extractor
            .functions
            .iter()
            .find(|f| f.name == "method")
            .unwrap();
        assert!(method.is_method);
        assert_eq!(method.class_name.as_deref(), Some("MyClass"));
    }

    #[test]
    fn test_dynamic_code_detection() {
        let code = r#"
def dangerous():
    exec("print('danger')")
    eval("1 + 1")
    getattr(obj, attr_name)
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        assert!(extractor
            .dynamic_code
            .iter()
            .any(|d| matches!(d.dynamic_type, DynamicCodeType::Exec)));
        assert!(extractor
            .dynamic_code
            .iter()
            .any(|d| matches!(d.dynamic_type, DynamicCodeType::Eval)));
        assert!(extractor
            .dynamic_code
            .iter()
            .any(|d| matches!(d.dynamic_type, DynamicCodeType::Getattr)));
    }

    #[test]
    fn test_decorators() {
        let code = r#"
@app.route("/test")
@login_required
def view():
    pass
"#;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.py");
        fs::write(&file_path, code).unwrap();

        let tree = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(code, &tree).unwrap();

        let view_func = extractor
            .functions
            .iter()
            .find(|f| f.name == "view")
            .unwrap();
        assert!(view_func.decorators.iter().any(|d| d.contains("app.route")));
        assert!(view_func
            .decorators
            .iter()
            .any(|d| d.contains("login_required")));
    }
}
