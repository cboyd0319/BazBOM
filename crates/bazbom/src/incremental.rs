//! Incremental analysis for large monorepos
//!
//! Provides git-based change detection and affected target identification
//! to enable fast PR scans that only analyze changed code.

use anyhow::{Context, Result};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::info;

/// Incremental analysis engine
pub struct IncrementalAnalyzer {
    /// Workspace root directory
    workspace_root: PathBuf,
    /// Git base reference for comparison (e.g., "main", "HEAD~1")
    base_ref: String,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer
    pub fn new(workspace_root: PathBuf, base_ref: String) -> Self {
        Self {
            workspace_root,
            base_ref,
        }
    }

    /// Find targets affected by changes
    ///
    /// Uses git to detect changed files, then queries the build system
    /// to find all targets that depend on those files.
    pub fn find_affected_targets(&self) -> Result<Vec<String>> {
        let changed_files = self.get_changed_files()?;

        if changed_files.is_empty() {
            info!(
                base_ref = %self.base_ref,
                "No files changed, using cached results"
            );
            return Ok(vec![]);
        }

        info!(
            changed_count = changed_files.len(),
            base_ref = %self.base_ref,
            "Found changed files"
        );

        // Detect build system and query for affected targets
        let affected = self.query_affected_targets(&changed_files)?;

        info!(
            affected_count = affected.len(),
            "Identified affected targets for incremental scan"
        );

        Ok(affected)
    }

    /// Get list of changed files from git
    fn get_changed_files(&self) -> Result<Vec<String>> {
        let output = Command::new("git")
            .args(["diff", "--name-only", &self.base_ref])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute git diff command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Git diff failed: {}", stderr);
        }

        let files: Vec<String> = String::from_utf8(output.stdout)
            .context("Git diff output is not valid UTF-8")?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(files)
    }

    /// Query build system for affected targets
    fn query_affected_targets(&self, changed_files: &[String]) -> Result<Vec<String>> {
        // Check if this is a Bazel workspace
        if self.workspace_root.join("WORKSPACE").exists()
            || self.workspace_root.join("MODULE.bazel").exists()
        {
            return self.query_bazel_affected_targets(changed_files);
        }

        // For Maven/Gradle, we scan the entire project since dependency
        // tracking is handled by the build tool itself
        info!("Non-Bazel workspace detected, full scan required");
        Ok(vec![])
    }

    /// Query Bazel for affected targets using rdeps
    fn query_bazel_affected_targets(&self, changed_files: &[String]) -> Result<Vec<String>> {
        // Build set of changed files as Bazel labels
        let file_labels: Vec<String> = changed_files
            .iter()
            .map(|f| {
                // Convert file path to Bazel label
                if f.starts_with("//") {
                    f.clone()
                } else {
                    format!("//{}", f.trim_start_matches('/'))
                }
            })
            .collect();

        if file_labels.is_empty() {
            return Ok(vec![]);
        }

        // Build Bazel rdeps query
        // rdeps(//..., set(...)) finds all targets that depend on the changed files
        let file_set = file_labels.join(", ");
        let query = format!("rdeps(//..., set({}))", file_set);

        info!(query = %query, "Executing Bazel query");

        let output = Command::new("bazel")
            .args(["query", &query, "--output=label"])
            .current_dir(&self.workspace_root)
            .output()
            .context("Failed to execute Bazel query command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            info!(
                error = %stderr,
                "Bazel query failed, falling back to full scan"
            );
            return Ok(vec![]);
        }

        let targets: Vec<String> = String::from_utf8(output.stdout)
            .context("Bazel query output is not valid UTF-8")?
            .lines()
            .filter(|line| !line.is_empty())
            .map(|s| s.to_string())
            .collect();

        Ok(targets)
    }

    /// Check if incremental analysis is supported
    pub fn is_supported(&self) -> bool {
        // Check if we're in a git repository
        let git_dir = self.workspace_root.join(".git");
        if !git_dir.exists() {
            return false;
        }

        // Check if base ref exists
        let output = Command::new("git")
            .args(["rev-parse", "--verify", &self.base_ref])
            .current_dir(&self.workspace_root)
            .output();

        match output {
            Ok(output) => output.status.success(),
            Err(_) => false,
        }
    }
}

/// Affected target summary
#[derive(Debug, Clone)]
pub struct AffectedTargets {
    /// List of affected target labels
    pub targets: Vec<String>,
    /// Number of changed files
    pub changed_files_count: usize,
    /// Base reference used for comparison
    pub base_ref: String,
    /// Whether this represents a full scan or incremental
    pub is_incremental: bool,
}

impl AffectedTargets {
    /// Create a full scan result (all targets)
    pub fn full_scan() -> Self {
        Self {
            targets: vec![],
            changed_files_count: 0,
            base_ref: String::new(),
            is_incremental: false,
        }
    }

    /// Create an incremental scan result
    pub fn incremental(targets: Vec<String>, changed_files_count: usize, base_ref: String) -> Self {
        Self {
            targets,
            changed_files_count,
            base_ref,
            is_incremental: true,
        }
    }

    /// Get unique targets (deduplicated)
    pub fn unique_targets(&self) -> Vec<String> {
        let mut unique: HashSet<String> = self.targets.iter().cloned().collect();
        let mut result: Vec<String> = unique.drain().collect();
        result.sort();
        result
    }

    /// Estimate time savings vs. full scan
    pub fn estimate_time_savings(&self, total_targets: usize) -> f64 {
        if !self.is_incremental || total_targets == 0 {
            return 0.0;
        }

        let affected_count = self.targets.len();
        if affected_count == 0 {
            return 1.0; // 100% savings (nothing to scan)
        }

        1.0 - (affected_count as f64 / total_targets as f64)
    }
}

/// Determine if a path affects Java/JVM code
pub fn is_jvm_related(path: &Path) -> bool {
    // Check if it's a known build file (no extension check needed)
    if let Some(filename) = path.file_name() {
        let name = filename.to_str().unwrap_or("");
        if matches!(
            name,
            "pom.xml" | "build.gradle" | "build.gradle.kts" | "BUILD" | "BUILD.bazel"
        ) {
            return true;
        }
    }

    // Check file extension
    if let Some(ext) = path.extension() {
        let ext_str = ext.to_str().unwrap_or("");
        return matches!(
            ext_str,
            "java" | "kt" | "kts" | "scala" | "groovy" | "xml" | "gradle" | "properties"
        );
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_is_jvm_related() {
        assert!(is_jvm_related(&PathBuf::from("src/main/java/Foo.java")));
        assert!(is_jvm_related(&PathBuf::from("src/test/kotlin/Bar.kt")));
        assert!(is_jvm_related(&PathBuf::from("pom.xml")));
        assert!(is_jvm_related(&PathBuf::from("build.gradle")));
        assert!(is_jvm_related(&PathBuf::from("BUILD.bazel")));

        assert!(!is_jvm_related(&PathBuf::from("README.md")));
        assert!(!is_jvm_related(&PathBuf::from("main.py")));
        assert!(!is_jvm_related(&PathBuf::from("index.html")));
    }

    #[test]
    fn test_affected_targets_full_scan() {
        let result = AffectedTargets::full_scan();
        assert!(!result.is_incremental);
        assert_eq!(result.targets.len(), 0);
        assert_eq!(result.estimate_time_savings(1000), 0.0);
    }

    #[test]
    fn test_affected_targets_incremental() {
        let targets = vec![
            "//java/com/example:foo".to_string(),
            "//java/com/example:bar".to_string(),
        ];
        let result = AffectedTargets::incremental(targets.clone(), 5, "main".to_string());

        assert!(result.is_incremental);
        assert_eq!(result.targets.len(), 2);
        assert_eq!(result.changed_files_count, 5);
        assert_eq!(result.base_ref, "main");

        // 2 out of 1000 targets = 99.8% time savings
        let savings = result.estimate_time_savings(1000);
        assert!(savings > 0.99);
    }

    #[test]
    fn test_unique_targets() {
        let targets = vec![
            "//foo".to_string(),
            "//bar".to_string(),
            "//foo".to_string(), // duplicate
            "//baz".to_string(),
        ];
        let result = AffectedTargets::incremental(targets, 2, "main".to_string());

        let unique = result.unique_targets();
        assert_eq!(unique.len(), 3);
        assert!(unique.contains(&"//foo".to_string()));
        assert!(unique.contains(&"//bar".to_string()));
        assert!(unique.contains(&"//baz".to_string()));
    }

    #[test]
    fn test_analyzer_creation() {
        let analyzer = IncrementalAnalyzer::new(PathBuf::from("/tmp"), "main".to_string());
        assert_eq!(analyzer.workspace_root, PathBuf::from("/tmp"));
        assert_eq!(analyzer.base_ref, "main");
    }
}
