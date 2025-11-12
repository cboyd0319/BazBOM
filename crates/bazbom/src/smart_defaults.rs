//! Smart context-aware defaults for BazBOM
//!
//! Auto-detects environment and adjusts behavior for optimal developer experience.

use std::env;
use std::path::Path;

/// Smart defaults based on environment detection
#[derive(Debug, Clone)]
pub struct SmartDefaults {
    pub enable_json: bool,
    pub enable_reachability: bool,
    pub enable_incremental: bool,
    pub enable_diff: bool,
    pub is_ci: bool,
    pub is_pr: bool,
    pub repo_size: u64,
}

impl SmartDefaults {
    /// Detect environment and calculate smart defaults
    pub fn detect() -> Self {
        let is_ci = Self::detect_ci();
        let is_pr = Self::detect_pr();
        let repo_size = Self::estimate_repo_size();
        let has_baseline = Path::new("bazbom-findings.json").exists();

        // Heuristics
        let enable_json = is_ci;
        let enable_reachability = repo_size < 100_000_000; // < 100MB
        let enable_incremental = is_pr;
        let enable_diff = has_baseline;

        Self {
            enable_json,
            enable_reachability,
            enable_incremental,
            enable_diff,
            is_ci,
            is_pr,
            repo_size,
        }
    }

    /// Print what was auto-detected
    pub fn print_detection(&self) {
        if self.is_ci {
            println!("🤖 CI environment detected");
        }
        if self.is_pr {
            println!("📋 Pull request detected");
        }
        if self.enable_reachability {
            println!("⚡ Small repo detected - enabling reachability (fast)");
        }
        if self.enable_diff {
            println!("📊 Baseline found - enabling diff mode");
        }
    }

    /// Detect if running in CI environment
    fn detect_ci() -> bool {
        // Check common CI environment variables
        env::var("CI").is_ok()
            || env::var("GITHUB_ACTIONS").is_ok()
            || env::var("GITLAB_CI").is_ok()
            || env::var("CIRCLECI").is_ok()
            || env::var("TRAVIS").is_ok()
            || env::var("JENKINS_HOME").is_ok()
            || env::var("BUILDKITE").is_ok()
    }

    /// Detect if running in PR context
    fn detect_pr() -> bool {
        // GitHub Actions
        if let Ok(event) = env::var("GITHUB_EVENT_NAME") {
            if event == "pull_request" || event == "pull_request_target" {
                return true;
            }
        }

        // GitLab CI
        if env::var("CI_MERGE_REQUEST_ID").is_ok() {
            return true;
        }

        // Checks if we're on a branch that's not main/master
        if let Ok(branch) = env::var("GITHUB_HEAD_REF") {
            if !branch.is_empty() && branch != "main" && branch != "master" {
                return true;
            }
        }

        false
    }

    /// Estimate repository size (very rough heuristic)
    fn estimate_repo_size() -> u64 {
        // Try to get .git directory size as proxy for repo size
        if let Ok(metadata) = std::fs::metadata(".git") {
            if metadata.is_dir() {
                // Very rough estimate: .git size * 3
                // This is just a heuristic to decide if reachability is "fast enough"
                return Self::dir_size(Path::new(".git")).unwrap_or(0) * 3;
            }
        }

        // Default to small repo if we can't detect
        50_000_000 // 50MB
    }

    /// Calculate directory size recursively
    fn dir_size(path: &Path) -> std::io::Result<u64> {
        let mut size = 0;

        if path.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let metadata = entry.metadata()?;

                if metadata.is_dir() {
                    // Limit recursion to avoid slowdown
                    size += Self::dir_size(&entry.path()).unwrap_or(0);
                } else {
                    size += metadata.len();
                }
            }
        }

        Ok(size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_ci() {
        // Should not detect CI in normal environment
        assert!(!SmartDefaults::detect_ci());
    }

    #[test]
    fn test_smart_defaults() {
        let defaults = SmartDefaults::detect();

        // In test environment, should have reasonable defaults
        assert!(!defaults.enable_json); // Not in CI
        assert!(defaults.repo_size > 0);
    }
}
