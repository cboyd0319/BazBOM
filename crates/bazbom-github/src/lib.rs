//! # BazBOM GitHub Integration
//!
//! This crate provides comprehensive GitHub PR automation for BazBOM v6.8.
//!
//! ## Features
//!
//! - Automatic PR creation with AI-powered fixes
//! - Full intelligence integration (ALL 14+ BazBOM modules)
//! - Auto-merge capabilities with safety controls
//! - Multi-PR orchestration across repositories
//! - GitHub webhook receiver for PR events
//! - CODEOWNERS integration for auto-assignment
//! - PR status tracking and Jira synchronization
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_github::{GitHubClient, GitHubConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize GitHub client
//!     let config = GitHubConfig::from_file(".bazbom/github.yml")?;
//!
//!     // Get token from environment variable specified in config
//!     let token = std::env::var(&config.token_env)
//!         .expect("GitHub token not found in environment");
//!     let client = GitHubClient::new(&token);
//!
//!     // Create a PR
//!     // let pr = client.create_pull_request(/* ... */).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod orchestrator;
pub mod pr_template;
pub mod webhook;

pub use client::GitHubClient;
pub use config::GitHubConfig;
pub use error::GitHubError;
pub use models::*;

/// Version of the bazbom-github crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
