//! Error types for Ruby reachability analysis

use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RubyReachabilityError>;

#[derive(Error, Debug)]
pub enum RubyReachabilityError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Call graph error: {0}")]
    CallGraphError(String),

    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    #[error("Invalid Ruby code: {0}")]
    InvalidRuby(String),

    #[error("Tree-sitter error: {0}")]
    TreeSitterError(String),
}

impl From<tree_sitter::LanguageError> for RubyReachabilityError {
    fn from(err: tree_sitter::LanguageError) -> Self {
        RubyReachabilityError::TreeSitterError(err.to_string())
    }
}
