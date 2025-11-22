//! Pub.dev API client for Dart/Flutter packages
//!
//! Fetches package data from https://pub.dev/api
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_pub::{get_package_info, get_package_score};
//!
//! // Get package info
//! let pkg = get_package_info("flutter")?;
//! println!("Latest: {}", pkg.latest.version);
//!
//! // Get package score
//! let score = get_package_score("provider")?;
//! println!("Popularity: {}", score.popularity_score.unwrap_or(0.0));
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

const PUB_API_BASE: &str = "https://pub.dev/api";

/// Pub.dev package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubPackage {
    /// Package name
    pub name: String,
    /// Latest version info
    pub latest: PubVersion,
    /// All versions
    #[serde(default)]
    pub versions: Vec<PubVersion>,
}

/// Package version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubVersion {
    /// Version string
    pub version: String,
    /// Pubspec content
    pub pubspec: PubSpec,
    /// Archive URL
    pub archive_url: Option<String>,
    /// Published timestamp
    pub published: Option<String>,
}

/// Pubspec.yaml content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubSpec {
    /// Package name
    pub name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Homepage
    pub homepage: Option<String>,
    /// Repository
    pub repository: Option<String>,
    /// Environment constraints
    pub environment: Option<PubEnvironment>,
    /// Dependencies
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, serde_json::Value>,
    /// Dev dependencies
    #[serde(default)]
    pub dev_dependencies: std::collections::HashMap<String, serde_json::Value>,
}

/// Environment constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubEnvironment {
    /// Dart SDK constraint
    pub sdk: Option<String>,
    /// Flutter constraint
    pub flutter: Option<String>,
}

/// Package score/metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubScore {
    /// Grant points
    #[serde(rename = "grantedPoints")]
    pub granted_points: Option<u32>,
    /// Max points
    #[serde(rename = "maxPoints")]
    pub max_points: Option<u32>,
    /// Like count
    #[serde(rename = "likeCount")]
    pub like_count: Option<u32>,
    /// Popularity score (0-1)
    #[serde(rename = "popularityScore")]
    pub popularity_score: Option<f64>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Security advisory (if available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PubAdvisory {
    /// Advisory ID
    pub id: String,
    /// Affected package
    pub package: String,
    /// Affected versions
    pub affected_versions: String,
    /// Fixed version
    pub patched_versions: Option<String>,
    /// Severity
    pub severity: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Get package information from pub.dev
///
/// # Arguments
/// * `package` - Package name
pub fn get_package_info(package: &str) -> Result<PubPackage> {
    let url = format!("{}/packages/{}", PUB_API_BASE, package);
    debug!("Fetching pub.dev package info from: {}", url);

    let pkg: PubPackage = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch pub.dev package {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse pub.dev package response")?;

    Ok(pkg)
}

/// Get package score/metrics
///
/// # Arguments
/// * `package` - Package name
pub fn get_package_score(package: &str) -> Result<PubScore> {
    let url = format!("{}/packages/{}/score", PUB_API_BASE, package);
    debug!("Fetching pub.dev package score from: {}", url);

    let score: PubScore = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch score for {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse pub.dev score response")?;

    Ok(score)
}

/// Get specific version information
///
/// # Arguments
/// * `package` - Package name
/// * `version` - Version string
pub fn get_version_info(package: &str, version: &str) -> Result<PubVersion> {
    let url = format!("{}/packages/{}/versions/{}", PUB_API_BASE, package, version);
    debug!("Fetching pub.dev version info from: {}", url);

    let ver: PubVersion = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch {}/{}", package, version))?
        .body_mut()
        .read_json()
        .context("Failed to parse pub.dev version response")?;

    Ok(ver)
}

/// Compare Dart package versions (semver)
pub fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // Split off pre-release and build metadata
    let parse_version = |v: &str| -> (Vec<u64>, Option<String>) {
        // Remove build metadata
        let v = v.split('+').next().unwrap_or(v);

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

/// Get supported Dart SDK versions
pub fn get_supported_sdk_versions() -> Vec<&'static str> {
    vec!["3.0", "2.19", "2.18", "2.17", "2.16", "2.15"]
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
            compare_versions("1.0.0-dev.1", "1.0.0"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_versions("1.0.0+build.1", "1.0.0"),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn test_is_version_affected() {
        assert!(is_version_affected("1.0.0", "1.0.1"));
        assert!(!is_version_affected("1.0.1", "1.0.1"));
        assert!(!is_version_affected("1.0.2", "1.0.1"));
    }
}
