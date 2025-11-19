//! Data models for Bazel reachability analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Reachability report for Bazel targets
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReachabilityReport {
    /// All target dependencies (target -> list of deps)
    pub target_dependencies: HashMap<String, Vec<String>>,

    /// Entrypoint targets (binaries, tests)
    pub entrypoints: Vec<String>,

    /// Reachable targets from entrypoints
    pub reachable_targets: HashSet<String>,

    /// Unreachable targets
    pub unreachable_targets: HashSet<String>,
}
