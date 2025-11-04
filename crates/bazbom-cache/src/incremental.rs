//! Incremental analysis support for BazBOM
//!
//! Detects changes in git repositories to enable fast, targeted rescans

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Information about changes in a git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSet {
    /// Base commit SHA
    pub base_commit: String,
    /// Current commit SHA
    pub current_commit: String,
    /// Modified files
    pub modified_files: Vec<PathBuf>,
    /// Added files
    pub added_files: Vec<PathBuf>,
    /// Deleted files
    pub deleted_files: Vec<PathBuf>,
    /// Changed build files (pom.xml, build.gradle, BUILD.bazel)
    pub changed_build_files: Vec<PathBuf>,
}

impl ChangeSet {
    /// Check if any build files have changed
    pub fn has_build_file_changes(&self) -> bool {
        !self.changed_build_files.is_empty()
    }

    /// Check if this is a significant change requiring rescan
    pub fn requires_rescan(&self) -> bool {
        // Always rescan if build files changed
        if self.has_build_file_changes() {
            return true;
        }

        // Check if any dependency-related files changed
        self.modified_files
            .iter()
            .chain(self.added_files.iter())
            .any(|f| is_dependency_file(f))
    }

    /// Get all changed files
    pub fn all_changed_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        files.extend(self.modified_files.clone());
        files.extend(self.added_files.clone());
        files
    }
}

/// Incremental analyzer using git
pub struct IncrementalAnalyzer {
    /// Repository root directory
    repo_root: PathBuf,
}

impl IncrementalAnalyzer {
    /// Create a new incremental analyzer
    pub fn new(repo_root: PathBuf) -> Result<Self> {
        // Verify this is a git repository
        if !repo_root.join(".git").exists() {
            anyhow::bail!("Not a git repository: {}", repo_root.display());
        }

        Ok(Self { repo_root })
    }

    /// Get the current commit SHA
    pub fn get_current_commit(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to execute git rev-parse")?;

        if !output.status.success() {
            anyhow::bail!(
                "git rev-parse failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let commit = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in git output")?
            .trim()
            .to_string();

        Ok(commit)
    }

    /// Get changes since a specific commit
    pub fn get_changes_since(&self, base_commit: &str) -> Result<ChangeSet> {
        let current_commit = self.get_current_commit()?;

        // Get modified and added files
        let output = Command::new("git")
            .args(["diff", "--name-status", base_commit, "HEAD"])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to execute git diff")?;

        if !output.status.success() {
            anyhow::bail!("git diff failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        let diff_output = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;

        let mut modified_files = Vec::new();
        let mut added_files = Vec::new();
        let mut deleted_files = Vec::new();
        let mut changed_build_files = Vec::new();

        for line in diff_output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 2 {
                continue;
            }

            let status = parts[0];
            let file_path = PathBuf::from(parts[1]);

            // Track build files separately
            if is_build_file(&file_path) {
                changed_build_files.push(file_path.clone());
            }

            match status {
                "M" => modified_files.push(file_path),
                "A" => added_files.push(file_path),
                "D" => deleted_files.push(file_path),
                _ => {} // Other statuses (R, C, etc.)
            }
        }

        Ok(ChangeSet {
            base_commit: base_commit.to_string(),
            current_commit,
            modified_files,
            added_files,
            deleted_files,
            changed_build_files,
        })
    }

    /// Get untracked files that might be relevant
    pub fn get_untracked_build_files(&self) -> Result<Vec<PathBuf>> {
        let output = Command::new("git")
            .args(["ls-files", "--others", "--exclude-standard"])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to execute git ls-files")?;

        if !output.status.success() {
            anyhow::bail!(
                "git ls-files failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        let files_output = String::from_utf8(output.stdout).context("Invalid UTF-8 in git output")?;

        let untracked_build_files: Vec<PathBuf> = files_output
            .lines()
            .map(PathBuf::from)
            .filter(|p| is_build_file(p))
            .collect();

        Ok(untracked_build_files)
    }

    /// Check if incremental analysis is possible
    pub fn can_use_incremental(&self, cached_commit: &str) -> Result<bool> {
        // Check if the cached commit exists
        let output = Command::new("git")
            .args(["cat-file", "-e", cached_commit])
            .current_dir(&self.repo_root)
            .output()
            .context("Failed to check if commit exists")?;

        Ok(output.status.success())
    }
}

/// Check if a file is a build file (pom.xml, build.gradle, BUILD.bazel, etc.)
fn is_build_file(path: &Path) -> bool {
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    matches!(
        filename,
        "pom.xml"
            | "build.gradle"
            | "build.gradle.kts"
            | "settings.gradle"
            | "settings.gradle.kts"
            | "BUILD"
            | "BUILD.bazel"
            | "WORKSPACE"
            | "WORKSPACE.bazel"
            | "MODULE.bazel"
            | "gradle.properties"
            | "maven_install.json"
    )
}

/// Check if a file is dependency-related
fn is_dependency_file(path: &Path) -> bool {
    // Build files are always dependency-related
    if is_build_file(path) {
        return true;
    }

    // Check for dependency lock files
    let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    matches!(
        filename,
        "gradle.lockfile"
            | "Cargo.lock"
            | "package-lock.json"
            | "yarn.lock"
            | "pnpm-lock.yaml"
            | "Pipfile.lock"
            | "poetry.lock"
            | "go.sum"
    )
}

/// Configuration for incremental analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncrementalConfig {
    /// Enable incremental analysis
    pub enabled: bool,
    /// Force full scan even if changes are minimal
    pub force_full_scan: bool,
    /// Base commit to compare against (None = use cache)
    pub base_commit: Option<String>,
}

impl Default for IncrementalConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            force_full_scan: false,
            base_commit: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_build_file() {
        assert!(is_build_file(Path::new("pom.xml")));
        assert!(is_build_file(Path::new("build.gradle")));
        assert!(is_build_file(Path::new("BUILD.bazel")));
        assert!(is_build_file(Path::new("MODULE.bazel")));
        assert!(!is_build_file(Path::new("src/main.rs")));
        assert!(!is_build_file(Path::new("README.md")));
    }

    #[test]
    fn test_is_dependency_file() {
        assert!(is_dependency_file(Path::new("pom.xml")));
        assert!(is_dependency_file(Path::new("Cargo.lock")));
        assert!(is_dependency_file(Path::new("package-lock.json")));
        assert!(!is_dependency_file(Path::new("src/main.rs")));
    }

    #[test]
    fn test_changeset_requires_rescan() {
        let mut changeset = ChangeSet {
            base_commit: "abc123".to_string(),
            current_commit: "def456".to_string(),
            modified_files: vec![PathBuf::from("src/main.rs")],
            added_files: vec![],
            deleted_files: vec![],
            changed_build_files: vec![],
        };

        // No build files changed, src file doesn't require rescan
        assert!(!changeset.requires_rescan());

        // Add a build file
        changeset.changed_build_files.push(PathBuf::from("pom.xml"));
        assert!(changeset.requires_rescan());
    }

    #[test]
    fn test_changeset_all_changed_files() {
        let changeset = ChangeSet {
            base_commit: "abc123".to_string(),
            current_commit: "def456".to_string(),
            modified_files: vec![PathBuf::from("file1.txt")],
            added_files: vec![PathBuf::from("file2.txt")],
            deleted_files: vec![PathBuf::from("file3.txt")],
            changed_build_files: vec![],
        };

        let all_files = changeset.all_changed_files();
        assert_eq!(all_files.len(), 2); // Only modified and added, not deleted
        assert!(all_files.contains(&PathBuf::from("file1.txt")));
        assert!(all_files.contains(&PathBuf::from("file2.txt")));
    }
}
