//! Data models for Go reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for a Go function/method
pub type FunctionId = String;

/// Represents a Go function or method in the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    /// Unique identifier (e.g., "package/file.go:FuncName")
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
    /// Whether this is an exported function (starts with uppercase)
    pub is_exported: bool,
    /// Type name if this is a method
    pub receiver_type: Option<String>,
    /// Whether this is a goroutine launch point
    pub launches_goroutine: bool,
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
            is_exported: false,
            receiver_type: None,
            launches_goroutine: false,
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
    /// Warnings about reflection or unsafe code
    pub reflection_warnings: Vec<ReflectionWarning>,
}

/// Information about a vulnerable function's reachability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReachability {
    /// CVE identifier (GO-YYYY-NNNN format)
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

/// Represents a detected Go entrypoint
#[derive(Debug, Clone)]
pub struct Entrypoint {
    /// File containing the entrypoint
    pub file: PathBuf,
    /// Function name
    pub function_name: String,
    /// Type of entrypoint
    pub entrypoint_type: EntrypointType,
    /// Additional metadata (e.g., HTTP route path)
    pub metadata: HashMap<String, String>,
}

/// Types of Go entrypoints
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntrypointType {
    /// func main() in main package
    Main,
    /// HTTP handler (http.HandleFunc, gin.GET, etc.)
    HttpHandler,
    /// gRPC service method
    GrpcService,
    /// Cobra CLI command
    CobraCommand,
    /// Test function (func Test*)
    Test,
    /// Benchmark function (func Benchmark*)
    Benchmark,
}

/// Warning about reflection or dynamic code that limits analysis accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionWarning {
    /// File containing the reflection code
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Type of reflection/dynamic code
    pub warning_type: ReflectionType,
    /// Description of the issue
    pub description: String,
}

/// Types of reflection/dynamic code patterns in Go
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReflectionType {
    /// reflect.Value.Call() or reflect.Value.CallSlice()
    ReflectCall,
    /// reflect.Value.MethodByName()
    MethodByName,
    /// reflect.Value.FieldByName()
    FieldByName,
    /// Interface type assertion with variable type
    TypeAssertion,
}
