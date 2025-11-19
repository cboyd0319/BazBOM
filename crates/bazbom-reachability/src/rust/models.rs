//! Data models for Rust reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

pub type FunctionId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    pub id: FunctionId,
    pub name: String,
    pub file: PathBuf,
    pub line: usize,
    pub column: usize,
    pub is_entrypoint: bool,
    pub reachable: bool,
    pub calls: Vec<FunctionId>,
    pub is_pub: bool,
    pub is_async: bool,
    pub is_test: bool,
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
            is_pub: false,
            is_async: false,
            is_test: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReachabilityReport {
    pub all_functions: HashMap<FunctionId, FunctionNode>,
    pub reachable_functions: HashSet<FunctionId>,
    pub unreachable_functions: HashSet<FunctionId>,
    pub entrypoints: Vec<FunctionId>,
    pub vulnerabilities: Vec<VulnerabilityReachability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityReachability {
    pub cve_id: String,
    pub package: String,
    pub version: String,
    pub vulnerable_functions: Vec<String>,
    pub reachable: bool,
    pub call_chain: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct Entrypoint {
    pub file: PathBuf,
    pub function_name: String,
    pub entrypoint_type: EntrypointType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntrypointType {
    Main,
    Test,
    Benchmark,
    TokioMain,
    ActixMain,
}
