//! Data models for Java reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for a Java method
pub type MethodId = String;

/// Represents a Java method in the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodNode {
    /// Unique identifier (e.g., "com.example.Class:methodName(Ljava/lang/String;)V")
    pub id: MethodId,
    /// Method name
    pub name: String,
    /// Class name (fully qualified)
    pub class_name: String,
    /// Method descriptor (JVM signature)
    pub descriptor: String,
    /// Source file (if available in debug info)
    pub file: Option<PathBuf>,
    /// Line number (if available in debug info)
    pub line: Option<usize>,
    /// Whether this is an entrypoint
    pub is_entrypoint: bool,
    /// Whether this method is reachable from an entrypoint
    pub reachable: bool,
    /// List of methods this method calls
    pub calls: Vec<MethodId>,
    /// Whether this is a public method
    pub is_public: bool,
    /// Whether this is a static method
    pub is_static: bool,
}

impl MethodNode {
    pub fn new(id: MethodId, name: String, class_name: String, descriptor: String) -> Self {
        Self {
            id,
            name,
            class_name,
            descriptor,
            file: None,
            line: None,
            is_entrypoint: false,
            reachable: false,
            calls: Vec::new(),
            is_public: false,
            is_static: false,
        }
    }
}

/// Report of reachability analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct ReachabilityReport {
    /// All methods found in the project
    pub all_functions: HashMap<MethodId, MethodNode>,
    /// Set of reachable method IDs
    pub reachable_functions: HashSet<MethodId>,
    /// Set of unreachable method IDs
    pub unreachable_functions: HashSet<MethodId>,
    /// Entrypoint method IDs
    pub entrypoints: Vec<MethodId>,
    /// Vulnerabilities with reachability information
    pub vulnerabilities: Vec<VulnerabilityReachability>,
    /// Warnings about reflection or dynamic code
    pub reflection_warnings: Vec<ReflectionWarning>,
}

impl ReachabilityReport {
    pub fn new() -> Self {
        Self {
            all_functions: HashMap::new(),
            reachable_functions: HashSet::new(),
            unreachable_functions: HashSet::new(),
            entrypoints: Vec::new(),
            vulnerabilities: Vec::new(),
            reflection_warnings: Vec::new(),
        }
    }
}

impl Default for ReachabilityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a vulnerable method's reachability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReachability {
    /// CVE identifier
    pub cve_id: String,
    /// Package name (groupId:artifactId format)
    pub package: String,
    /// Package version
    pub version: String,
    /// Vulnerable method names
    pub vulnerable_methods: Vec<String>,
    /// Whether the vulnerability is reachable
    pub reachable: bool,
    /// Call chain from entrypoint to vulnerable method
    pub call_chain: Option<Vec<String>>,
}

/// Warning about reflection or dynamic code that limits analysis accuracy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionWarning {
    /// Class containing the reflection code
    pub class_name: String,
    /// Method containing the reflection code
    pub method_name: String,
    /// Type of reflection/dynamic code
    pub warning_type: ReflectionType,
    /// Description of the issue
    pub description: String,
}

/// Types of reflection/dynamic code patterns in Java
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReflectionType {
    /// Class.forName() or ClassLoader.loadClass()
    ClassForName,
    /// Method.invoke()
    MethodInvoke,
    /// Constructor.newInstance()
    ReflectiveConstructor,
    /// MethodHandle.invoke() or MethodHandle.invokeExact()
    MethodHandle,
    /// Dynamic proxy (Proxy.newProxyInstance)
    DynamicProxy,
}
