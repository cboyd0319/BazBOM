use thiserror::Error;

#[derive(Error, Debug)]
pub enum DepsDevError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Package not found: {system}/{name}@{version}")]
    PackageNotFound {
        system: String,
        name: String,
        version: String,
    },

    #[error("Invalid package system: {0}")]
    InvalidSystem(String),

    #[error("API returned error: {0}")]
    ApiError(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(#[from] serde_json::Error),

    #[error("Rate limited by deps.dev API")]
    RateLimited,
}

pub type Result<T> = std::result::Result<T, DepsDevError>;
