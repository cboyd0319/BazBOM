//! OpenSSF Scorecard integration for BazBOM
//!
//! Integrates with OpenSSF Scorecard to assess repository security health.
//! Scorecard checks various security best practices and provides a score.
//!
//! See: <https://github.com/ossf/scorecard>

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

/// OpenSSF Scorecard result for a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardResult {
    /// Repository URL
    pub repo: String,
    /// Overall score (0-10)
    pub score: f64,
    /// Date of the check
    pub date: String,
    /// Individual check results
    pub checks: Vec<ScorecardCheck>,
    /// Commit SHA that was analyzed
    pub commit: Option<String>,
}

/// Individual scorecard check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScorecardCheck {
    /// Check name
    pub name: String,
    /// Score for this check (0-10, or -1 for unavailable)
    pub score: i32,
    /// Reason for the score
    pub reason: String,
    /// Documentation URL
    pub documentation: Option<String>,
}

/// Scorecard integration client
pub struct ScorecardClient {
    /// Path to scorecard binary
    scorecard_path: String,
    /// Enable verbose output
    verbose: bool,
}

impl Default for ScorecardClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ScorecardClient {
    /// Create a new scorecard client
    pub fn new() -> Self {
        Self {
            scorecard_path: "scorecard".to_string(),
            verbose: false,
        }
    }

    /// Create a scorecard client with custom binary path
    pub fn with_path(path: String) -> Self {
        Self {
            scorecard_path: path,
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Check if scorecard is available
    pub fn is_available(&self) -> bool {
        Command::new(&self.scorecard_path)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Run scorecard on a repository
    ///
    /// # Arguments
    /// * `repo_url` - GitHub repository URL (e.g., "github.com/owner/repo")
    ///
    /// # Returns
    /// Scorecard results or error if scorecard is not available or fails
    pub fn check_repo(&self, repo_url: &str) -> Result<ScorecardResult> {
        if !self.is_available() {
            anyhow::bail!(
                "OpenSSF Scorecard not found at '{}'. Install from: https://github.com/ossf/scorecard",
                self.scorecard_path
            );
        }

        // Format repo URL for scorecard
        let repo_arg = if repo_url.starts_with("http") {
            repo_url.to_string()
        } else if repo_url.starts_with("github.com/") {
            format!("https://{}", repo_url)
        } else {
            format!("https://github.com/{}", repo_url)
        };

        // Run scorecard with JSON output
        let mut cmd = Command::new(&self.scorecard_path);
        cmd.arg("--repo").arg(&repo_arg).arg("--format").arg("json");

        if self.verbose {
            cmd.arg("--show-details");
        }

        let output = cmd
            .output()
            .context("Failed to execute scorecard command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Scorecard failed: {}", stderr);
        }

        // Parse JSON output
        let json =
            String::from_utf8(output.stdout).context("Scorecard output is not valid UTF-8")?;

        let scorecard_output: ScorecardJsonOutput =
            serde_json::from_str(&json).context("Failed to parse scorecard JSON output")?;

        // Convert to our format
        Ok(self.convert_result(repo_url, scorecard_output))
    }

    /// Check multiple repositories
    pub fn check_repos(&self, repo_urls: &[String]) -> Vec<Result<ScorecardResult>> {
        repo_urls.iter().map(|url| self.check_repo(url)).collect()
    }

    /// Convert scorecard JSON output to our format
    fn convert_result(&self, repo: &str, output: ScorecardJsonOutput) -> ScorecardResult {
        let checks = output
            .checks
            .iter()
            .map(|check| ScorecardCheck {
                name: check.name.clone(),
                score: check.score,
                reason: check.reason.clone(),
                documentation: check.documentation.clone(),
            })
            .collect();

        ScorecardResult {
            repo: repo.to_string(),
            score: output.score,
            date: output.date.clone(),
            checks,
            commit: output.commit.clone(),
        }
    }

    /// Get risk level based on scorecard score
    pub fn get_risk_level(score: f64) -> RiskLevel {
        if score >= 8.0 {
            RiskLevel::Low
        } else if score >= 6.0 {
            RiskLevel::Medium
        } else if score >= 4.0 {
            RiskLevel::High
        } else {
            RiskLevel::Critical
        }
    }
}

/// Risk level based on scorecard score
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk (score >= 8.0)
    Low,
    /// Medium risk (score 6.0-7.9)
    Medium,
    /// High risk (score 4.0-5.9)
    High,
    /// Critical risk (score < 4.0)
    Critical,
}

/// Scorecard JSON output structure (subset of fields we care about)
#[derive(Debug, Deserialize)]
struct ScorecardJsonOutput {
    /// Repository being analyzed
    #[allow(dead_code)]
    repo: ScorecardRepo,
    /// Overall score
    score: f64,
    /// Check date
    date: String,
    /// Commit SHA
    commit: Option<String>,
    /// Individual checks
    checks: Vec<ScorecardCheckJson>,
}

#[derive(Debug, Deserialize)]
struct ScorecardRepo {
    /// Repository name
    #[allow(dead_code)]
    name: String,
    /// Commit SHA
    #[allow(dead_code)]
    commit: String,
}

#[derive(Debug, Deserialize)]
struct ScorecardCheckJson {
    /// Check name
    name: String,
    /// Score (0-10, or -1 for unavailable)
    score: i32,
    /// Reason for the score
    reason: String,
    /// Documentation URL
    documentation: Option<String>,
}

/// Extract repository URL from package metadata
pub fn extract_repo_url(purl: &str) -> Option<String> {
    // Parse PURL to extract repository information
    // Format: pkg:maven/groupId/artifactId@version

    // For Maven Central packages, we try to find the repo from:
    // 1. PURL metadata
    // 2. Maven metadata
    // 3. Known mappings

    // This is a simplified version - in production we'd query Maven Central
    // or use the package's POM file to find the SCM URL

    if purl.starts_with("pkg:maven/") {
        // Example heuristic: many Apache projects follow a pattern
        if purl.contains("/org.apache.") {
            // Extract artifact ID and guess repo
            // This would need proper implementation
            return None;
        }
    }

    None
}

/// Get repository from dependency metadata
pub fn get_dependency_repo(group_id: &str, artifact_id: &str) -> Option<String> {
    // Known repository mappings for common packages
    let known_repos: HashMap<(&str, &str), &str> = [
        (
            ("org.springframework", "spring-core"),
            "spring-projects/spring-framework",
        ),
        (
            ("org.springframework.boot", "spring-boot"),
            "spring-projects/spring-boot",
        ),
        (("com.google.guava", "guava"), "google/guava"),
        (
            ("org.apache.logging.log4j", "log4j-core"),
            "apache/logging-log4j2",
        ),
        (
            ("com.fasterxml.jackson.core", "jackson-core"),
            "FasterXML/jackson-core",
        ),
        (("junit", "junit"), "junit-team/junit4"),
        (("org.junit.jupiter", "junit-jupiter"), "junit-team/junit5"),
        (("org.slf4j", "slf4j-api"), "qos-ch/slf4j"),
        (("ch.qos.logback", "logback-classic"), "qos-ch/logback"),
        (("com.google.code.gson", "gson"), "google/gson"),
        (
            ("org.apache.commons", "commons-lang3"),
            "apache/commons-lang",
        ),
        (("commons-io", "commons-io"), "apache/commons-io"),
        (
            ("org.apache.httpcomponents", "httpclient"),
            "apache/httpcomponents-client",
        ),
    ]
    .iter()
    .cloned()
    .collect();

    known_repos
        .get(&(group_id, artifact_id))
        .map(|repo| repo.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scorecard_client_creation() {
        let client = ScorecardClient::new();
        assert_eq!(client.scorecard_path, "scorecard");
        assert!(!client.verbose);
    }

    #[test]
    fn test_scorecard_client_with_path() {
        let client = ScorecardClient::with_path("/usr/local/bin/scorecard".to_string());
        assert_eq!(client.scorecard_path, "/usr/local/bin/scorecard");
    }

    #[test]
    fn test_scorecard_client_verbose() {
        let client = ScorecardClient::new().verbose();
        assert!(client.verbose);
    }

    #[test]
    fn test_risk_level_calculation() {
        assert_eq!(ScorecardClient::get_risk_level(9.0), RiskLevel::Low);
        assert_eq!(ScorecardClient::get_risk_level(7.0), RiskLevel::Medium);
        assert_eq!(ScorecardClient::get_risk_level(5.0), RiskLevel::High);
        assert_eq!(ScorecardClient::get_risk_level(2.0), RiskLevel::Critical);
    }

    #[test]
    fn test_known_repository_mappings() {
        assert_eq!(
            get_dependency_repo("org.springframework", "spring-core"),
            Some("spring-projects/spring-framework".to_string())
        );
        assert_eq!(
            get_dependency_repo("com.google.guava", "guava"),
            Some("google/guava".to_string())
        );
        assert_eq!(
            get_dependency_repo("unknown.group", "unknown-artifact"),
            None
        );
    }

    #[test]
    fn test_scorecard_result_structure() {
        let result = ScorecardResult {
            repo: "github.com/owner/repo".to_string(),
            score: 7.5,
            date: "2025-11-05".to_string(),
            checks: vec![ScorecardCheck {
                name: "Code-Review".to_string(),
                score: 8,
                reason: "Found code review".to_string(),
                documentation: Some("https://...".to_string()),
            }],
            commit: Some("abc123".to_string()),
        };

        assert_eq!(result.repo, "github.com/owner/repo");
        assert_eq!(result.score, 7.5);
        assert_eq!(result.checks.len(), 1);
    }
}
