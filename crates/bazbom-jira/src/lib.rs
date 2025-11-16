//! # BazBOM Jira Integration
//!
//! This crate provides comprehensive Jira bidirectional integration for BazBOM v6.8.
//!
//! ## Features
//!
//! - Automatic ticket creation for vulnerabilities
//! - Bidirectional sync (Jira ↔ BazBOM)
//! - Webhook receiver for Jira events
//! - Custom field mapping and templates
//! - Smart routing and team assignment
//! - SLA tracking and sprint integration
//! - VEX generation from rejected tickets
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_jira::{JiraClient, JiraConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize Jira client
//!     let config = JiraConfig::from_file(".bazbom/jira.yml")?;
//!
//!     // Get credentials from environment variables specified in config
//!     let username_env = config.auth.username_env.as_ref()
//!         .expect("username_env not configured");
//!     let token_env = config.auth.token_env.as_ref()
//!         .expect("token_env not configured");
//!
//!     let username = std::env::var(username_env)
//!         .expect("Jira username not found in environment");
//!     let token = std::env::var(token_env)
//!         .expect("Jira token not found in environment");
//!
//!     let client = JiraClient::new(&config.url, &format!("{}:{}", username, token));
//!
//!     // Create a ticket
//!     // let issue = client.create_issue(/* ... */).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod routing;
pub mod sync;
pub mod templates;
pub mod webhook;

pub use client::JiraClient;
pub use config::JiraConfig;
pub use error::JiraError;
pub use models::*;

/// Version of the bazbom-jira crate
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
