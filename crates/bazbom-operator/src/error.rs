//! Error types for the operator

use thiserror::Error;

/// Error types that can occur during operator operations
#[derive(Error, Debug)]
pub enum OperatorError {
    /// Error from Kubernetes API operations
    #[error("Kubernetes API error: {0}")]
    Kube(#[from] kube::Error),

    /// Error during JSON serialization/deserialization
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid operator or scan configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Failed to create Kubernetes Job for scanning
    #[error("Scan job creation failed: {0}")]
    JobCreationFailed(String),

    /// Failed to update BazBOMScan status
    #[error("Status update failed: {0}")]
    StatusUpdateFailed(String),

    /// Requested Kubernetes resource was not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Generic error for uncategorized failures
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Result type alias for operator operations
pub type Result<T> = std::result::Result<T, OperatorError>;
