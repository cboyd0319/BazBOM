//! Error types for Java reachability analysis

use thiserror::Error;

pub type Result<T> = std::result::Result<T, JavaReachabilityError>;

#[derive(Error, Debug)]
pub enum JavaReachabilityError {
    #[error("Failed to read JAR file: {0}")]
    JarReadError(String),

    #[error("Failed to parse class file: {0}")]
    ClassParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid bytecode: {0}")]
    InvalidBytecode(String),

    #[error("Unsupported Java version: {0}")]
    UnsupportedJavaVersion(u16),

    #[error("Analysis error: {0}")]
    AnalysisError(String),
}
