use serde::{Deserialize, Serialize};

/// GitHub pull request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// PR number
    pub number: u64,

    /// PR title
    pub title: String,

    /// PR body/description
    pub body: Option<String>,

    /// Head branch
    pub head: Branch,

    /// Base branch
    pub base: Branch,

    /// PR state
    pub state: String,

    /// Draft PR
    #[serde(default)]
    pub draft: bool,

    /// HTML URL
    pub html_url: String,

    /// Created at
    pub created_at: String,

    /// Updated at
    pub updated_at: String,
}

/// Git branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// Branch name (ref)
    #[serde(rename = "ref")]
    pub branch_ref: String,

    /// SHA
    pub sha: String,

    /// Repository
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repo: Option<Repository>,
}

/// GitHub repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name
    pub name: String,

    /// Full name (owner/repo)
    pub full_name: String,

    /// Owner
    pub owner: User,

    /// Default branch
    pub default_branch: String,
}

/// GitHub user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Username
    pub login: String,

    /// User ID
    pub id: u64,
}

/// PR creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePullRequestRequest {
    /// PR title
    pub title: String,

    /// PR body
    pub body: String,

    /// Head branch (source)
    pub head: String,

    /// Base branch (target)
    pub base: String,

    /// Draft PR
    #[serde(default)]
    pub draft: bool,
}

/// PR update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePullRequestRequest {
    /// PR title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// PR body
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,

    /// PR state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// BazBOM PR metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BazBomPrMetadata {
    /// CVE ID
    pub cve_id: String,

    /// Package name
    pub package: String,

    /// Current version
    pub current_version: String,

    /// Fix version
    pub fix_version: String,

    /// Severity
    pub severity: String,

    /// ML Risk score (0-100)
    pub ml_risk_score: u8,

    /// Reachability status
    pub reachable: bool,

    /// Auto-merge eligible
    pub auto_merge_eligible: bool,

    /// Jira ticket key
    pub jira_ticket: Option<String>,

    /// BazBOM scan URL
    pub bazbom_scan_url: Option<String>,
}

/// Auto-merge configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoMergeConfig {
    /// Enable auto-merge
    pub enabled: bool,

    /// Require passing tests
    pub require_passing_tests: bool,

    /// Require approvals
    pub require_approvals: u8,

    /// Require CODEOWNERS approval
    pub require_codeowners_approval: bool,

    /// Maximum severity for auto-merge
    pub max_severity: String,

    /// Minimum upgrade confidence (0-100)
    pub min_confidence: u8,

    /// No breaking changes
    pub no_breaking_changes: bool,

    /// Trusted dependencies only
    pub trusted_dependencies_only: bool,

    /// Maximum version jump (major, minor, patch)
    pub max_version_jump: String,

    /// Merge delay (hours)
    pub merge_delay_hours: u32,

    /// Business hours only
    pub business_hours_only: bool,

    /// Auto-rollback on failure
    pub auto_rollback_on_failure: bool,
}

impl Default for AutoMergeConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            require_passing_tests: true,
            require_approvals: 1,
            require_codeowners_approval: true,
            max_severity: "MEDIUM".to_string(),
            min_confidence: 90,
            no_breaking_changes: true,
            trusted_dependencies_only: true,
            max_version_jump: "minor".to_string(),
            merge_delay_hours: 2,
            business_hours_only: true,
            auto_rollback_on_failure: true,
        }
    }
}
