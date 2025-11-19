//! Error types for Bazel reachability analysis

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BazelReachabilityError {
    #[error("Bazel command failed: {0}")]
    BazelCommandFailed(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, BazelReachabilityError>;
