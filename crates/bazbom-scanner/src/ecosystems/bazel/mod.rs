//! Bazel build system scanner
//!
//! Bazel is polyglot - it can build projects in many languages.
//! The main dependency extraction is handled by bazbom's bazel.rs module.
//! This scanner provides the Scanner trait implementation for ecosystem detection.

use crate::scanner::{License, LicenseContext, ScanContext, Scanner};
use crate::types::{EcosystemScanResult, ReachabilityData};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;

/// Bazel ecosystem scanner
pub struct BazelScanner;

impl Default for BazelScanner {
    fn default() -> Self {
        Self
    }
}

impl BazelScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for BazelScanner {
    fn name(&self) -> &str {
        "bazel"
    }

    fn detect(&self, root: &Path) -> bool {
        // Check for Bazel workspace files
        root.join("WORKSPACE").exists()
            || root.join("WORKSPACE.bazel").exists()
            || root.join("MODULE.bazel").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result =
            EcosystemScanResult::new("Bazel".to_string(), ctx.root.display().to_string());

        // Bazel dependency extraction is handled by bazbom's bazel.rs module
        // which uses `bazel query` to extract the full dependency graph.
        // This scanner is primarily for ecosystem detection and reachability integration.

        // Run reachability analysis
        if let Err(e) = analyze_reachability(&ctx.root, &mut result) {
            tracing::warn!("Bazel reachability analysis failed: {}", e);
        }

        Ok(result)
    }

    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        // Bazel doesn't have a standard location for license files
        // License info depends on the underlying language ecosystem
        License::Unknown
    }
}

/// Analyze reachability for Bazel project
fn analyze_reachability(root: &Path, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_reachability::bazel::analyze_bazel_project;

    let report = analyze_bazel_project(root)?;
    let mut vulnerable_packages_reachable = HashMap::new();

    for package in &result.packages {
        let key = format!("{}@{}", package.name, package.version);
        // Check if package appears in any reachable target
        let is_reachable = report.reachable_targets.iter().any(|target| {
            target.contains(&package.name) || target.contains(&package.name.replace("-", "_"))
        });
        vulnerable_packages_reachable.insert(key, is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.reachable_targets.len() + report.unreachable_targets.len(),
        reachable_functions: report.reachable_targets.len(),
        unreachable_functions: report.unreachable_targets.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}
