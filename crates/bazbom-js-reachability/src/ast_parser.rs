//! AST parsing using SWC for JavaScript and TypeScript

use crate::error::{JsReachabilityError, Result};
use std::path::Path;
use swc_common::sync::Lrc;
use swc_common::{
    errors::{ColorConfig, Handler},
    FileName, SourceMap,
};
use swc_ecma_ast::*;
use swc_ecma_parser::{lexer::Lexer, Parser, StringInput, Syntax, TsConfig};
use swc_ecma_visit::{Visit, VisitWith};

/// Parse a JavaScript or TypeScript file into an AST
pub fn parse_file(file_path: &Path) -> Result<Module> {
    let source_code = std::fs::read_to_string(file_path)?;

    let syntax = determine_syntax(file_path);
    let cm: Lrc<SourceMap> = Default::default();

    // Create a handler for parse errors
    let handler = Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(cm.clone()));

    let fm = cm.load_file(file_path).map_err(|e| {
        JsReachabilityError::ParseError(format!("Failed to load file: {}", e))
    })?;

    let lexer = Lexer::new(
        syntax,
        Default::default(),
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);

    parser.parse_module().map_err(|e| {
        let mut msg = format!("Parse error in {}: ", file_path.display());
        for error in parser.take_errors() {
            msg.push_str(&format!("{:?} ", error));
        }
        JsReachabilityError::ParseError(msg)
    })
}

/// Determine the syntax (JS vs TS) based on file extension
fn determine_syntax(file_path: &Path) -> Syntax {
    let extension = file_path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    match extension {
        "ts" | "tsx" => Syntax::Typescript(TsConfig {
            tsx: extension == "tsx",
            decorators: true,
            ..Default::default()
        }),
        _ => Syntax::Es(Default::default()),
    }
}

/// Extract function definitions and calls from an AST
#[derive(Debug)]
pub struct FunctionExtractor {
    pub functions: Vec<ExtractedFunction>,
    pub calls: Vec<ExtractedCall>,
    current_file: String,
}

#[derive(Debug, Clone)]
pub struct ExtractedFunction {
    pub name: String,
    pub line: usize,
    pub column: usize,
    pub is_export: bool,
}

#[derive(Debug, Clone)]
pub struct ExtractedCall {
    pub callee: String,
    pub line: usize,
    pub column: usize,
}

impl FunctionExtractor {
    pub fn new(file_path: &str) -> Self {
        Self {
            functions: Vec::new(),
            calls: Vec::new(),
            current_file: file_path.to_string(),
        }
    }

    pub fn extract_from_module(&mut self, module: &Module) {
        module.visit_with(self);
    }
}

impl Visit for FunctionExtractor {
    fn visit_fn_decl(&mut self, node: &FnDecl) {
        self.functions.push(ExtractedFunction {
            name: node.ident.sym.to_string(),
            line: 0, // TODO: Get actual line from span
            column: 0,
            is_export: false,
        });

        // Continue visiting child nodes
        node.visit_children_with(self);
    }

    fn visit_fn_expr(&mut self, node: &FnExpr) {
        if let Some(ident) = &node.ident {
            self.functions.push(ExtractedFunction {
                name: ident.sym.to_string(),
                line: 0,
                column: 0,
                is_export: false,
            });
        }

        node.visit_children_with(self);
    }

    fn visit_arrow_expr(&mut self, node: &ArrowExpr) {
        // Arrow functions are typically anonymous, handle separately
        node.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, node: &CallExpr) {
        // Extract function call information
        match &node.callee {
            Callee::Expr(expr) => match &**expr {
                Expr::Ident(ident) => {
                    self.calls.push(ExtractedCall {
                        callee: ident.sym.to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                Expr::Member(member) => {
                    // Handle method calls like obj.method()
                    if let MemberProp::Ident(ident) = &member.prop {
                        self.calls.push(ExtractedCall {
                            callee: ident.sym.to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                }
                _ => {}
            },
            _ => {}
        }

        node.visit_children_with(self);
    }

    fn visit_export_decl(&mut self, node: &ExportDecl) {
        // Mark functions as exported
        match &node.decl {
            Decl::Fn(fn_decl) => {
                self.functions.push(ExtractedFunction {
                    name: fn_decl.ident.sym.to_string(),
                    line: 0,
                    column: 0,
                    is_export: true,
                });
            }
            _ => {}
        }

        node.visit_children_with(self);
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

        let module = parse_file(&file_path).unwrap();
        assert!(!module.body.is_empty());
    }

    #[test]
    fn test_parse_typescript() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        let code = r#"
interface User {
    name: string;
    age: number;
}

function greet(user: User): string {
    return `Hello, ${user.name}!`;
}

export { greet };
"#;

        fs::write(&file_path, code).unwrap();

        let module = parse_file(&file_path).unwrap();
        assert!(!module.body.is_empty());
    }

    #[test]
    fn test_function_extraction() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.js");

        let code = r#"
function foo() {
    bar();
}

function bar() {
    console.log("bar");
}

foo();
"#;

        fs::write(&file_path, code).unwrap();

        let module = parse_file(&file_path).unwrap();
        let mut extractor = FunctionExtractor::new(file_path.to_str().unwrap());
        extractor.extract_from_module(&module);

        assert_eq!(extractor.functions.len(), 2);
        assert!(extractor.functions.iter().any(|f| f.name == "foo"));
        assert!(extractor.functions.iter().any(|f| f.name == "bar"));

        // Check calls
        assert!(extractor.calls.iter().any(|c| c.callee == "bar"));
        assert!(extractor.calls.iter().any(|c| c.callee == "foo"));
    }
}
