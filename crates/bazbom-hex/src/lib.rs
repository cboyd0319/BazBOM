//! Hex.pm API client for Elixir/Erlang packages
//!
//! Fetches package data from https://hex.pm/api
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_hex::{get_package_info, get_package_versions};
//!
//! // Get package info
//! let pkg = get_package_info("phoenix")?;
//! println!("Downloads: {}", pkg.downloads.all);
//!
//! // Get all versions
//! let versions = get_package_versions("ecto")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

const HEX_API_BASE: &str = "https://hex.pm/api";

/// Hex package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexPackage {
    /// Package name
    pub name: String,
    /// Latest stable version
    pub latest_stable_version: Option<String>,
    /// Latest version (may be pre-release)
    pub latest_version: Option<String>,
    /// Package description
    pub meta: HexMeta,
    /// Download statistics
    pub downloads: HexDownloads,
    /// Repository URL
    pub url: String,
    /// HTML URL
    pub html_url: String,
}

/// Package metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexMeta {
    /// Description
    pub description: Option<String>,
    /// License
    #[serde(default)]
    pub licenses: Vec<String>,
    /// Links
    #[serde(default)]
    pub links: std::collections::HashMap<String, String>,
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexDownloads {
    /// Total downloads
    pub all: u64,
    /// Recent downloads
    pub recent: u64,
    /// Week downloads
    pub week: u64,
    /// Day downloads
    pub day: u64,
}

/// Package version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexVersion {
    /// Version string
    pub version: String,
    /// Inserted at timestamp
    pub inserted_at: String,
    /// Has documentation
    pub has_docs: bool,
    /// Requirements (dependencies)
    #[serde(default)]
    pub requirements: std::collections::HashMap<String, HexRequirement>,
}

/// Dependency requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HexRequirement {
    /// Required package
    pub app: Option<String>,
    /// Version requirement
    pub requirement: String,
    /// Optional dependency
    pub optional: bool,
}

/// Get package information from Hex.pm
///
/// # Arguments
/// * `package` - Package name
pub fn get_package_info(package: &str) -> Result<HexPackage> {
    let url = format!("{}/packages/{}", HEX_API_BASE, package);
    debug!("Fetching Hex package info from: {}", url);

    let pkg: HexPackage = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch Hex package {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse Hex package response")?;

    Ok(pkg)
}

/// Get all versions of a package
///
/// # Arguments
/// * `package` - Package name
pub fn get_package_versions(package: &str) -> Result<Vec<HexVersion>> {
    let url = format!("{}/packages/{}/releases", HEX_API_BASE, package);
    debug!("Fetching Hex package versions from: {}", url);

    let versions: Vec<HexVersion> = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch versions for {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse Hex versions response")?;

    Ok(versions)
}

/// Get specific version information
///
/// # Arguments
/// * `package` - Package name
/// * `version` - Version string
pub fn get_version_info(package: &str, version: &str) -> Result<HexVersion> {
    let url = format!("{}/packages/{}/releases/{}", HEX_API_BASE, package, version);
    debug!("Fetching Hex version info from: {}", url);

    let ver: HexVersion = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch {}/{}", package, version))?
        .body_mut()
        .read_json()
        .context("Failed to parse Hex version response")?;

    Ok(ver)
}

/// Compare Elixir/Erlang versions
///
/// Versions follow semver with optional pre-release tags
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // Split off pre-release suffix
    let parse_version = |v: &str| -> (Vec<u64>, Option<String>) {
        let (version, pre) = if let Some(idx) = v.find('-') {
            (&v[..idx], Some(v[idx + 1..].to_string()))
        } else {
            (v, None)
        };

        let parts: Vec<u64> = version.split('.').filter_map(|s| s.parse().ok()).collect();

        (parts, pre)
    };

    let (a_parts, a_pre) = parse_version(a);
    let (b_parts, b_pre) = parse_version(b);

    // Compare version parts
    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }

    // Pre-release versions are less than release versions
    match (&a_pre, &b_pre) {
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (Some(_), None) => std::cmp::Ordering::Less,
        (Some(a_pre), Some(b_pre)) => a_pre.cmp(b_pre),
        (None, None) => std::cmp::Ordering::Equal,
    }
}

/// Check if a version is affected (less than fixed version)
pub fn is_version_affected(installed: &str, fixed: &str) -> bool {
    compare_versions(installed, fixed) == std::cmp::Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(compare_versions("1.0.0", "2.0.0"), std::cmp::Ordering::Less);
        assert_eq!(
            compare_versions("1.0.0-rc.1", "1.0.0"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_versions("1.0.1", "1.0.0"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_is_version_affected() {
        assert!(is_version_affected("1.0.0", "1.0.1"));
        assert!(!is_version_affected("1.0.1", "1.0.1"));
        assert!(!is_version_affected("1.0.2", "1.0.1"));
    }
}
