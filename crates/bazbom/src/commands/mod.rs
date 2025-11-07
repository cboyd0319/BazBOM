//! Command handlers for BazBOM CLI
//!
//! This module contains the implementation of all CLI commands, extracted from main.rs
//! to improve modularity and maintainability.

pub mod scan;
pub mod policy;
pub mod fix;
pub mod license;
pub mod db;
pub mod hooks;
pub mod init;
pub mod explore;
pub mod dashboard;
pub mod team;
pub mod report;

// Re-export command handlers for convenience
pub use scan::handle_scan;
pub use policy::handle_policy;
pub use fix::handle_fix;
pub use license::handle_license;
pub use db::handle_db;
pub use hooks::handle_install_hooks;
pub use init::handle_init;
pub use explore::handle_explore;
pub use dashboard::handle_dashboard;
pub use team::handle_team;
pub use report::handle_report;
