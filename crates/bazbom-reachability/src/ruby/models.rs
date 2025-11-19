//! Data models for Ruby reachability analysis

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
    pub is_public: bool,
    pub is_class_method: bool,
    pub is_instance_method: bool,
    pub class_name: Option<String>,
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
            is_public: true, // Ruby methods are public by default
            is_class_method: false,
            is_instance_method: true,
            class_name: None,
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
    pub has_dynamic_code: bool,
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
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntrypointType {
    Main,
    RailsController,
    RailsJob,
    RailsMailer,
    RSpecTest,
    MinitestTest,
    SinatraRoute,
    RakeTask,
}
