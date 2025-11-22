//! Command handlers for BazBOM CLI
//!
//! This module contains the implementation of all CLI commands, extracted from main.rs
//! to improve modularity and maintainability.

pub mod anomaly;
pub mod auth;
pub mod compare;
pub mod container_scan;
pub mod dashboard;
pub mod db;
pub mod explain;
pub mod explore;
pub mod fix;
pub mod github;
pub mod hooks;
pub mod init;
pub mod jira;
pub mod license;
pub mod lsp;
pub mod notify;
pub mod policy;
pub mod report;
pub mod scan;
pub mod status;
pub mod team;
pub mod threats;
pub mod upgrade_intelligence;
pub mod vex;
pub mod watch;

// Re-export command handlers for convenience
pub use compare::handle_compare;
pub use dashboard::handle_dashboard;
pub use db::handle_db;
pub use explain::handle_explain;
pub use explore::handle_explore;
pub use fix::handle_fix;
pub use github::handle_github;
pub use hooks::handle_install_hooks;
pub use init::handle_init;
pub use jira::handle_jira;
pub use license::handle_license;
pub use policy::handle_policy;
pub use report::handle_report;
pub use scan::handle_scan;
pub use status::handle_status;
pub use team::handle_team;
#[allow(unused_imports)]
pub use vex::{handle_vex_apply, handle_vex_create, handle_vex_list};
pub use watch::handle_watch;
