//! Data models for JS reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for a function
pub type FunctionId = String;

/// A function node in the call graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    /// Unique identifier (e.g., "src/index.js:main")
    pub id: FunctionId,
    /// Function name
    pub name: String,
    /// Source file
    pub file: PathBuf,
    /// Line number in source
    pub line: usize,
    /// Column number in source
    pub column: usize,
    /// Functions called by this function
    pub calls: Vec<FunctionId>,
    /// Whether this function is reachable from an entrypoint
    pub reachable: bool,
    /// Whether this function is an entrypoint
    pub is_entrypoint: bool,
}

impl FunctionNode {
    pub fn new(id: FunctionId, name: String, file: PathBuf, line: usize, column: usize) -> Self {
        Self {
            id,
            name,
            file,
            line,
            column,
            calls: Vec::new(),
            reachable: false,
            is_entrypoint: false,
        }
    }
}

/// Result of reachability analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityReport {
    /// All functions discovered
    pub all_functions: HashMap<FunctionId, FunctionNode>,
    /// Functions that are reachable from entrypoints
    pub reachable_functions: HashSet<FunctionId>,
    /// Functions that are NOT reachable
    pub unreachable_functions: HashSet<FunctionId>,
    /// Entrypoints identified
    pub entrypoints: Vec<FunctionId>,
    /// Vulnerabilities with reachability information
    pub vulnerabilities: Vec<VulnerabilityReachability>,
}

/// Vulnerability with reachability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReachability {
    /// CVE ID
    pub cve_id: String,
    /// Package name (e.g., "express")
    pub package: String,
    /// Package version
    pub version: String,
    /// Vulnerable function(s) in the package
    pub vulnerable_functions: Vec<String>,
    /// Whether the vulnerable code is reachable
    pub reachable: bool,
    /// Call chain from entrypoint to vulnerable function (if reachable)
    pub call_chain: Option<Vec<String>>,
}

/// Entrypoint types in JavaScript/TypeScript projects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntrypointType {
    /// Main entry (package.json "main" field)
    Main,
    /// Exported function (package.json "exports")
    Export,
    /// HTTP handler (Express, Fastify, etc.)
    HttpHandler,
    /// Event handler (React, Vue, etc.)
    EventHandler,
    /// Test file
    Test,
}

/// Information about an entrypoint
#[derive(Debug, Clone)]
pub struct Entrypoint {
    pub file: PathBuf,
    pub function_name: String,
    pub entrypoint_type: EntrypointType,
}
