//! Recursive transitive upgrade analysis with breaking change detection
//!
//! This crate provides intelligent upgrade analysis that recursively analyzes
//! not just the package you want to upgrade, but ALL dependencies it will pull in,
//! detecting breaking changes at every level.

pub mod analyzer;
pub mod breaking_changes;
pub mod community_data;
pub mod ecosystem_detection;
pub mod github;
pub mod models;
pub mod semver;

pub use analyzer::UpgradeAnalyzer;
pub use community_data::{CommunityDatabase, UpgradeSuccessData};
pub use ecosystem_detection::{detect_ecosystem_from_package, detect_ecosystem_with_confidence};
pub use models::*;
