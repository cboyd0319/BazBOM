// Data structures for remediation suggestions and results

use serde::{Deserialize, Serialize};

/// A remediation suggestion for a specific vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    pub vulnerability_id: String,
    pub affected_package: String,
    pub current_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub priority: String,
    pub why_fix: String,
    pub how_to_fix: String,
    pub breaking_changes: Option<String>,
    pub references: Vec<String>,
}

/// A complete remediation report with summary and suggestions
#[derive(Debug, Serialize, Deserialize)]
pub struct RemediationReport {
    pub summary: RemediationSummary,
    pub suggestions: Vec<RemediationSuggestion>,
}

/// Summary statistics for a remediation report
#[derive(Debug, Serialize, Deserialize)]
pub struct RemediationSummary {
    pub total_vulnerabilities: usize,
    pub fixable: usize,
    pub unfixable: usize,
    pub estimated_effort: String,
}

/// Result of applying fixes to a project
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyResult {
    pub applied: Vec<String>,
    pub failed: Vec<(String, String)>,
}

/// Result of applying fixes with test validation
#[derive(Debug, Serialize, Deserialize)]
pub struct ApplyResultWithTests {
    pub applied: Vec<String>,
    pub failed: Vec<(String, String)>,
    pub tests_passed: bool,
    pub rollback_performed: bool,
}

/// Configuration for GitHub PR creation
#[derive(Debug, Clone)]
pub struct PrConfig {
    pub token: String,
    pub repo: String,
    pub base_branch: String,
    pub head_branch: String,
}

impl PrConfig {
    /// Create a new PrConfig with validation
    pub fn new(
        token: String,
        repo: String,
        base_branch: String,
        head_branch: String,
    ) -> anyhow::Result<Self> {
        if token.is_empty() {
            anyhow::bail!("GitHub token cannot be empty");
        }
        if repo.is_empty() {
            anyhow::bail!("Repository cannot be empty");
        }
        if !repo.contains('/') {
            anyhow::bail!("Repository must be in format 'owner/repo'");
        }
        if base_branch.is_empty() {
            anyhow::bail!("Base branch cannot be empty");
        }
        if head_branch.is_empty() {
            anyhow::bail!("Head branch cannot be empty");
        }
        if base_branch == head_branch {
            anyhow::bail!("Base and head branches must be different");
        }

        Ok(Self {
            token,
            repo,
            base_branch,
            head_branch,
        })
    }
}
