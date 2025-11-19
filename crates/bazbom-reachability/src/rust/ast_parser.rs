//! AST parsing using syn for Rust code analysis

use super::error::{Result, RustReachabilityError};
use std::path::Path;
use syn::visit::Visit;
use syn::{Attribute, Expr, File, ItemFn};

/// Parse a Rust file into an AST
pub fn parse_file(file_path: &Path) -> Result<File> {
    let source_code = std::fs::read_to_string(file_path)?;

    syn::parse_file(&source_code).map_err(|e| RustReachabilityError::ParseError(format!("{}", e)))
}

#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub is_pub: bool,
    pub is_async: bool,
    pub is_test: bool,
    pub attributes: Vec<String>,
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
    current_function: Option<String>,
}

impl FunctionExtractor {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            current_function: None,
        }
    }

    pub fn extract(&mut self, file: &File) {
        self.visit_file(file);
    }

    fn extract_attributes(attrs: &[Attribute]) -> Vec<String> {
        attrs
            .iter()
            .filter_map(|attr| attr.path().get_ident().map(|ident| ident.to_string()))
            .collect()
    }

    fn is_test_function(attrs: &[Attribute]) -> bool {
        attrs
            .iter()
            .any(|attr| attr.path().is_ident("test") || attr.path().is_ident("tokio::test"))
    }

    fn extract_call_from_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Call(call) => {
                if let Some(name) = self.get_function_name_from_expr(&call.func) {
                    self.calls.push(ExtractedCall {
                        callee: name,
                        line: 0, // syn doesn't provide line numbers easily
                        caller_context: self.current_function.clone(),
                    });
                }
                // Visit arguments
                for arg in &call.args {
                    self.extract_call_from_expr(arg);
                }
            }
            Expr::MethodCall(method_call) => {
                let method_name = method_call.method.to_string();
                self.calls.push(ExtractedCall {
                    callee: method_name,
                    line: 0,
                    caller_context: self.current_function.clone(),
                });
                // Visit receiver and arguments
                self.extract_call_from_expr(&method_call.receiver);
                for arg in &method_call.args {
                    self.extract_call_from_expr(arg);
                }
            }
            _ => {}
        }
    }

    fn get_function_name_from_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Path(path) => path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::")
                .into(),
            _ => None,
        }
    }
}

impl<'ast> Visit<'ast> for FunctionExtractor {
    fn visit_item_fn(&mut self, func: &'ast ItemFn) {
        let name = func.sig.ident.to_string();
        let attributes = Self::extract_attributes(&func.attrs);
        let is_test = Self::is_test_function(&func.attrs);

        let extracted = ExtractedFunction {
            name: name.clone(),
            line: 0,
            is_pub: matches!(func.vis, syn::Visibility::Public(_)),
            is_async: func.sig.asyncness.is_some(),
            is_test,
            attributes,
        };

        self.functions.push(extracted);

        // Visit function body to extract calls
        let prev = self.current_function.clone();
        self.current_function = Some(name);

        for stmt in &func.block.stmts {
            if let syn::Stmt::Expr(expr, _) = stmt {
                self.extract_call_from_expr(expr);
            }
        }

        self.current_function = prev;
    }
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
    fn test_parse_simple_rust() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.rs");

        let code = r#"
fn hello() {
    println!("Hello");
}

fn main() {
    hello();
}
"#;

        fs::write(&file_path, code).unwrap();
        let ast = parse_file(&file_path).unwrap();
        assert!(ast.items.len() >= 2);
    }

    #[test]
    fn test_extract_functions() {
        let code = r#"
pub fn public_func() {}
fn private_func() {}

#[test]
fn test_something() {}
"#;

        let ast = syn::parse_file(code).unwrap();
        let mut extractor = FunctionExtractor::new();
        extractor.extract(&ast);

        assert_eq!(extractor.functions.len(), 3);
        assert!(extractor
            .functions
            .iter()
            .any(|f| f.name == "public_func" && f.is_pub));
        assert!(extractor
            .functions
            .iter()
            .any(|f| f.name == "test_something" && f.is_test));
    }
}
