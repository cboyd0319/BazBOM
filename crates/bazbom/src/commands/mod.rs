//! Command handlers for BazBOM CLI
//!
//! This module contains the implementation of all CLI commands, extracted from main.rs
//! to improve modularity and maintainability.

pub mod container_scan;
pub mod dashboard;
pub mod db;
pub mod explain;
pub mod explore;
pub mod fix;
pub mod hooks;
pub mod init;
pub mod license;
pub mod policy;
pub mod report;
pub mod scan;
pub mod team;
pub mod upgrade_intelligence;

// Re-export command handlers for convenience
pub use dashboard::handle_dashboard;
pub use db::handle_db;
pub use explain::handle_explain;
pub use explore::handle_explore;
pub use fix::handle_fix;
pub use hooks::handle_install_hooks;
pub use init::handle_init;
pub use license::handle_license;
pub use policy::handle_policy;
pub use report::handle_report;
pub use scan::handle_scan;
pub use team::handle_team;
