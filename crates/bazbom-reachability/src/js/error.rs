//! Error types for JS reachability analysis

use thiserror::Error;

pub type Result<T> = std::result::Result<T, JsReachabilityError>;

#[derive(Error, Debug)]
pub enum JsReachabilityError {
    #[error("Failed to parse JavaScript/TypeScript file: {0}")]
    ParseError(String),

    #[error("Module resolution failed for '{module}': {reason}")]
    ModuleResolutionError { module: String, reason: String },

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Unsupported file type: {0}")]
    UnsupportedFileType(String),

    #[error("Call graph error: {0}")]
    CallGraphError(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}
