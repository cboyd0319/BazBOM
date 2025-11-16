//! Error types for tool verification

use std::path::PathBuf;

/// Result type for tool verification operations
pub type ToolVerifyResult<T> = Result<T, ToolVerifyError>;

/// Errors that can occur during tool verification
#[derive(Debug, thiserror::Error)]
pub enum ToolVerifyError {
    #[error("Tool not found in PATH: {0}")]
    ToolNotFound(String),

    #[error("Tool not in registry: {0}")]
    ToolNotInRegistry(String),

    #[error("Checksum mismatch for {tool} at {path:?}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        tool: String,
        path: PathBuf,
        expected: String,
        actual: String,
    },

    #[error("Unsupported tool version: {tool} {version}")]
    UnsupportedVersion { tool: String, version: String },

    #[error("Failed to read tool binary: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Crypto error: {0}")]
    CryptoError(#[from] bazbom_crypto::CryptoError),

    #[error("Failed to parse registry: {0}")]
    RegistryParseError(#[from] serde_json::Error),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Tool verification disabled but required")]
    VerificationRequired,

    #[error("Unknown verification error: {0}")]
    Unknown(String),
}
