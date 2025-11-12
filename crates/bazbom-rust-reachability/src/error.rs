//! Error types for Rust reachability analysis

use std::io;
use std::path::PathBuf;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RustReachabilityError>;

#[derive(Error, Debug)]
pub enum RustReachabilityError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("Call graph error: {0}")]
    CallGraphError(String),

    #[error("File not found: {}", .0.display())]
    FileNotFound(PathBuf),

    #[error("Invalid Rust code: {0}")]
    InvalidRust(String),
}
