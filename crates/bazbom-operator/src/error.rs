//! Error types for the operator

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OperatorError {
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Scan job creation failed: {0}")]
    JobCreationFailed(String),

    #[error("Status update failed: {0}")]
    StatusUpdateFailed(String),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type Result<T> = std::result::Result<T, OperatorError>;
