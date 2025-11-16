use thiserror::Error;

/// GitHub integration error types
#[derive(Debug, Error)]
pub enum GitHubError {
    #[error("Unauthorized: Invalid credentials or token")]
    Unauthorized,

    #[error("Forbidden: Insufficient permissions")]
    Forbidden,

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("Pull request not found: {0}")]
    PullRequestNotFound(u64),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unexpected HTTP status {0}: {1}")]
    UnexpectedStatus(u16, String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Webhook verification failed")]
    WebhookVerificationFailed,

    #[error("Template error: {0}")]
    Template(String),

    #[error("Orchestration error: {0}")]
    Orchestration(String),

    #[error("Auto-merge blocked: {0}")]
    AutoMergeBlocked(String),

    #[error("GitHub API error: {0}")]
    Api(String),
}

/// Result type for GitHub operations
pub type Result<T> = std::result::Result<T, GitHubError>;
