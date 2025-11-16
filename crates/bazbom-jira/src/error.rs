use thiserror::Error;

/// Jira integration error types
#[derive(Debug, Error)]
pub enum JiraError {
    #[error("Unauthorized: Invalid credentials")]
    Unauthorized,

    #[error("Forbidden: Insufficient permissions")]
    Forbidden,

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unexpected HTTP status {0}: {1}")]
    UnexpectedStatus(u16, String),

    #[error("Bulk operation limit exceeded: {0} issues (max 50)")]
    BulkLimitExceeded(usize),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Middleware error: {0}")]
    Middleware(#[from] reqwest_middleware::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Webhook verification failed")]
    WebhookVerificationFailed,

    #[error("Template error: {0}")]
    Template(String),

    #[error("Routing error: {0}")]
    Routing(String),

    #[error("Sync error: {0}")]
    Sync(String),
}

/// Result type for Jira operations
pub type Result<T> = std::result::Result<T, JiraError>;
