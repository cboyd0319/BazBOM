//! Error types for Go reachability analysis

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GoReachabilityError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Go analyzer not found: {0}")]
    AnalyzerNotFound(String),

    #[error("Go analyzer failed: {0}")]
    AnalyzerFailed(String),
}

pub type Result<T> = std::result::Result<T, GoReachabilityError>;
