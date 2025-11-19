//! Data models for Go reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionNode {
    pub id: String,
    pub name: String,
    pub file: String,
    pub line: i32,
    pub is_entrypoint: bool,
    pub reachable: bool,
    pub calls: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReachabilityReport {
    pub all_functions: HashMap<String, FunctionNode>,
    pub reachable_functions: HashSet<String>,
    pub unreachable_functions: HashSet<String>,
    pub entrypoints: Vec<String>,
}
