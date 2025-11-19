//! Error types for PHP reachability analysis

use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PhpReachabilityError>;

#[derive(Error, Debug)]
pub enum PhpReachabilityError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Call graph error: {0}")]
    CallGraphError(String),

    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    #[error("Invalid PHP code: {0}")]
    InvalidPhp(String),

    #[error("Tree-sitter error: {0}")]
    TreeSitterError(String),
}

impl From<tree_sitter::LanguageError> for PhpReachabilityError {
    fn from(err: tree_sitter::LanguageError) -> Self {
        PhpReachabilityError::TreeSitterError(err.to_string())
    }
}
