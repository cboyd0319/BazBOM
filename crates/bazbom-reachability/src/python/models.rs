//! Data models for Python reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for a Python function/method
pub type FunctionId = String;

/// Represents a Python function or method in the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    /// Unique identifier (e.g., "module.py:ClassName.method_name")
    pub id: FunctionId,
    /// Function/method name
    pub name: String,
    /// File path where the function is defined
    pub file: PathBuf,
    /// Line number in the file
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Whether this is an entrypoint
    pub is_entrypoint: bool,
    /// Whether this function is reachable from an entrypoint
    pub reachable: bool,
    /// List of functions this function calls
    pub calls: Vec<FunctionId>,
    /// Whether this function is exported (visible to other modules)
    pub is_export: bool,
    /// Class name if this is a method
    pub class_name: Option<String>,
    /// Whether this is an async function
    pub is_async: bool,
    /// Decorators applied to this function
    pub decorators: Vec<String>,
}

impl FunctionNode {
    pub fn new(id: FunctionId, name: String, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            id,
            name,
            file,
            line,
            column,
            is_entrypoint: false,
            reachable: false,
            calls: Vec::new(),
            is_export: false,
            class_name: None,
            is_async: false,
            decorators: Vec::new(),
        }
    }
}

/// Report of reachability analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ReachabilityReport {
    /// All functions found in the project
    pub all_functions: HashMap<FunctionId, FunctionNode>,
    /// Set of reachable function IDs
    pub reachable_functions: HashSet<FunctionId>,
    /// Set of unreachable function IDs
    pub unreachable_functions: HashSet<FunctionId>,
    /// Entrypoint function IDs
    pub entrypoints: Vec<FunctionId>,
    /// Vulnerabilities with reachability information
    pub vulnerabilities: Vec<VulnerabilityReachability>,
    /// Warnings about dynamic code
    pub dynamic_code_warnings: Vec<DynamicCodeWarning>,
}

/// Information about a vulnerable function's reachability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReachability {
    /// CVE identifier
    pub cve_id: String,
    /// Package name
    pub package: String,
    /// Package version
    pub version: String,
    /// Vulnerable function names
    pub vulnerable_functions: Vec<String>,
    /// Whether the vulnerability is reachable
    pub reachable: bool,
    /// Call chain from entrypoint to vulnerable function
    pub call_chain: Option<Vec<String>>,
}

/// Represents a detected Python entrypoint
#[derive(Debug, Clone)]
pub struct Entrypoint {
    /// File containing the entrypoint
    pub file: PathBuf,
    /// Function name
    pub function_name: String,
    /// Type of entrypoint (e.g., "main", "flask_route", "django_view")
    pub entrypoint_type: EntrypointType,
    /// Additional metadata (e.g., route path for web frameworks)
    pub metadata: HashMap<String, String>,
}

/// Types of Python entrypoints
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntrypointType {
    /// if __name__ == "__main__":
    Main,
    /// Flask route decorator
    FlaskRoute,
    /// Django view function
    DjangoView,
    /// FastAPI route decorator
    FastApiRoute,
    /// Click CLI command
    ClickCommand,
    /// Pytest test function
    PytestTest,
    /// Celery task
    CeleryTask,
    /// AWS Lambda handler
    LambdaHandler,
}

/// Warning about dynamic code that limits analysis accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicCodeWarning {
    /// File containing the dynamic code
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Type of dynamic code
    pub warning_type: DynamicCodeType,
    /// Description of the issue
    pub description: String,
}

/// Types of dynamic code patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DynamicCodeType {
    /// exec() call
    Exec,
    /// eval() call
    Eval,
    /// getattr() with variable attribute name
    Getattr,
    /// setattr() with variable attribute name
    Setattr,
    /// __import__() with variable module name
    DynamicImport,
    /// Metaclass usage
    Metaclass,
}
