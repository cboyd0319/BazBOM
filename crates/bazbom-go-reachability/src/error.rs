//! Error types for Go reachability analysis

use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, GoReachabilityError>;

#[derive(Error, Debug)]
pub enum GoReachabilityError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Call graph error: {0}")]
    CallGraphError(String),

    #[error("Module resolution error: {0}")]
    ModuleResolutionError(String),

    #[error("Entrypoint detection error: {0}")]
    EntrypointError(String),

    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    #[error("Invalid Go code: {0}")]
    InvalidGo(String),
}
