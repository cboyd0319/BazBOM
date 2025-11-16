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
//!     let client = JiraClient::new(&config.url, &config.auth_token);
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
