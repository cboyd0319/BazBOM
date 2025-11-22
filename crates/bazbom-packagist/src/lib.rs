//! Packagist API client for PHP/Composer package intelligence
//!
//! This crate provides access to Packagist.org APIs for:
//! - Package metadata and versions
//! - Security advisories (CVEs, affected versions, severity)
//! - Download statistics
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_packagist::{get_package_info, get_security_advisories};
//!
//! // Get package metadata
//! let info = get_package_info("symfony", "symfony")?;
//! println!("Latest version: {:?}", info.versions.keys().next());
//!
//! // Get security advisories
//! let advisories = get_security_advisories(&["symfony/symfony"])?;
//! for advisory in advisories {
//!     println!("{}: {}", advisory.cve.unwrap_or_default(), advisory.title);
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

const PACKAGIST_API_BASE: &str = "https://packagist.org";
const PACKAGIST_REPO_BASE: &str = "https://repo.packagist.org";

/// Package information from Packagist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub description: Option<String>,
    pub time: Option<String>,
    pub maintainers: Vec<Maintainer>,
    pub versions: HashMap<String, VersionInfo>,
    pub r#type: Option<String>,
    pub repository: Option<String>,
    pub github_stars: Option<u32>,
    pub github_forks: Option<u32>,
    pub downloads: Option<Downloads>,
    pub favers: Option<u32>,
}

/// Package maintainer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Maintainer {
    pub name: String,
    pub avatar_url: Option<String>,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub name: String,
    pub version: String,
    pub version_normalized: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub homepage: Option<String>,
    pub license: Option<Vec<String>>,
    pub authors: Option<Vec<Author>>,
    pub source: Option<Source>,
    pub dist: Option<Dist>,
    pub require: Option<HashMap<String, String>>,
    #[serde(rename = "require-dev")]
    pub require_dev: Option<HashMap<String, String>>,
    pub time: Option<String>,
}

/// Package author
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: Option<String>,
    pub email: Option<String>,
    pub homepage: Option<String>,
}

/// Source repository info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub r#type: String,
    pub url: String,
    pub reference: Option<String>,
}

/// Distribution info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dist {
    pub r#type: String,
    pub url: String,
    pub reference: Option<String>,
    pub shasum: Option<String>,
}

/// Download statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub total: u64,
    pub monthly: u64,
    pub daily: u64,
}

/// Security advisory from Packagist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    /// Advisory ID (e.g., "PKSA-xxxx")
    #[serde(rename = "advisoryId")]
    pub advisory_id: String,
    /// Package name (vendor/package)
    #[serde(rename = "packageName")]
    pub package_name: String,
    /// Remote ID (e.g., GitHub advisory ID)
    #[serde(rename = "remoteId")]
    pub remote_id: Option<String>,
    /// Advisory title
    pub title: String,
    /// URL to advisory details
    pub link: Option<String>,
    /// CVE identifier
    pub cve: Option<String>,
    /// Affected version constraint
    #[serde(rename = "affectedVersions")]
    pub affected_versions: String,
    /// Source of the advisory
    pub source: Option<String>,
    /// Reported timestamp
    #[serde(rename = "reportedAt")]
    pub reported_at: Option<String>,
    /// Composer constraint for affected versions
    #[serde(rename = "composerRepository")]
    pub composer_repository: Option<String>,
    /// Severity (CRITICAL, HIGH, MEDIUM, LOW)
    pub severity: Option<String>,
}

/// Response wrapper for package info
#[derive(Debug, Deserialize)]
struct PackageResponse {
    package: PackageInfo,
}

/// Response wrapper for security advisories
#[derive(Debug, Deserialize)]
struct AdvisoriesResponse {
    advisories: HashMap<String, Vec<SecurityAdvisory>>,
}

/// Get package information from Packagist
///
/// # Arguments
/// * `vendor` - Package vendor (e.g., "symfony")
/// * `package` - Package name (e.g., "symfony")
///
/// # Returns
/// Package information including versions, maintainers, and metadata
pub fn get_package_info(vendor: &str, package: &str) -> Result<PackageInfo> {
    let url = format!(
        "{}/packages/{}/{}.json",
        PACKAGIST_API_BASE, vendor, package
    );
    debug!("Fetching package info from: {}", url);

    let response: PackageResponse = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch package {}/{}", vendor, package))?
        .body_mut()
        .read_json()
        .context("Failed to parse package response")?;

    Ok(response.package)
}

/// Get package metadata (Composer v2 format)
///
/// This is the preferred method for getting version information as it's
/// more efficient and used by Composer itself.
///
/// # Arguments
/// * `vendor` - Package vendor
/// * `package` - Package name
///
/// # Returns
/// Raw JSON value with package metadata
pub fn get_package_metadata(vendor: &str, package: &str) -> Result<serde_json::Value> {
    let url = format!("{}/p2/{}/{}.json", PACKAGIST_REPO_BASE, vendor, package);
    debug!("Fetching package metadata from: {}", url);

    let response: serde_json::Value = ureq::get(&url)
        .call()
        .context(format!(
            "Failed to fetch metadata for {}/{}",
            vendor, package
        ))?
        .body_mut()
        .read_json()
        .context("Failed to parse metadata response")?;

    Ok(response)
}

/// Get security advisories for packages
///
/// # Arguments
/// * `packages` - List of package names in "vendor/package" format
///
/// # Returns
/// List of security advisories affecting the specified packages
pub fn get_security_advisories(packages: &[&str]) -> Result<Vec<SecurityAdvisory>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    // Build query string with multiple packages
    let query: String = packages
        .iter()
        .map(|p| format!("packages[]={}", urlencoding::encode(p)))
        .collect::<Vec<_>>()
        .join("&");

    let url = format!("{}/api/security-advisories/?{}", PACKAGIST_API_BASE, query);
    debug!("Fetching security advisories from: {}", url);

    let response: AdvisoriesResponse = ureq::get(&url)
        .call()
        .context("Failed to fetch security advisories")?
        .body_mut()
        .read_json()
        .context("Failed to parse advisories response")?;

    // Flatten the HashMap into a Vec
    let advisories: Vec<SecurityAdvisory> = response.advisories.into_values().flatten().collect();

    Ok(advisories)
}

/// Get security advisories for a single package
///
/// # Arguments
/// * `vendor` - Package vendor
/// * `package` - Package name
///
/// # Returns
/// List of security advisories for the package
pub fn get_package_advisories(vendor: &str, package: &str) -> Result<Vec<SecurityAdvisory>> {
    let package_name = format!("{}/{}", vendor, package);
    get_security_advisories(&[&package_name])
}

/// Check if a version is affected by an advisory
///
/// # Arguments
/// * `version` - The installed version
/// * `constraint` - The affected version constraint from the advisory
///
/// # Returns
/// true if the version matches the constraint (is affected)
pub fn is_version_affected(version: &str, constraint: &str) -> bool {
    // Parse Composer version constraints
    // Examples: ">=2.0,<2.5.3", "<1.2.3|>=2.0,<2.1", ">=4.0,<4.4.50|>=5.0,<5.4.20|>=6.0,<6.3.7"

    // Split on | for OR conditions
    for or_constraint in constraint.split('|') {
        if matches_and_constraint(version, or_constraint.trim()) {
            return true;
        }
    }

    false
}

/// Check if version matches AND constraint (e.g., ">=2.0,<2.5.3")
fn matches_and_constraint(version: &str, constraint: &str) -> bool {
    // All conditions must match
    for condition in constraint.split(',') {
        if !matches_single_constraint(version, condition.trim()) {
            return false;
        }
    }
    true
}

/// Check if version matches a single constraint
fn matches_single_constraint(version: &str, constraint: &str) -> bool {
    let constraint = constraint.trim();

    if constraint.is_empty() {
        return true;
    }

    // Parse operator and version
    let (op, target) = if let Some(stripped) = constraint.strip_prefix(">=") {
        (">=", stripped)
    } else if let Some(stripped) = constraint.strip_prefix("<=") {
        ("<=", stripped)
    } else if let Some(stripped) = constraint.strip_prefix("!=") {
        ("!=", stripped)
    } else if let Some(stripped) = constraint.strip_prefix('>') {
        (">", stripped)
    } else if let Some(stripped) = constraint.strip_prefix('<') {
        ("<", stripped)
    } else if let Some(stripped) = constraint.strip_prefix('=') {
        ("=", stripped)
    } else {
        ("=", constraint)
    };

    let target = target.trim();

    // Compare versions
    match compare_versions(version, target) {
        Some(ord) => match op {
            ">=" => ord >= std::cmp::Ordering::Equal,
            "<=" => ord <= std::cmp::Ordering::Equal,
            ">" => ord == std::cmp::Ordering::Greater,
            "<" => ord == std::cmp::Ordering::Less,
            "=" => ord == std::cmp::Ordering::Equal,
            "!=" => ord != std::cmp::Ordering::Equal,
            _ => false,
        },
        None => false,
    }
}

/// Compare two version strings
fn compare_versions(a: &str, b: &str) -> Option<std::cmp::Ordering> {
    let parse_parts = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['.', '-'])
            .filter_map(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .collect()
    };

    let a_parts = parse_parts(a);
    let b_parts = parse_parts(b);

    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);

        match a_val.cmp(&b_val) {
            std::cmp::Ordering::Equal => continue,
            other => return Some(other),
        }
    }

    Some(std::cmp::Ordering::Equal)
}

/// Convert Packagist severity to standard format
pub fn normalize_severity(severity: Option<&str>) -> String {
    match severity.map(|s| s.to_uppercase()).as_deref() {
        Some("CRITICAL") => "CRITICAL".to_string(),
        Some("HIGH") => "HIGH".to_string(),
        Some("MEDIUM") => "MEDIUM".to_string(),
        Some("LOW") => "LOW".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert_eq!(
            compare_versions("1.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Equal)
        );
        assert_eq!(
            compare_versions("2.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Greater)
        );
        assert_eq!(
            compare_versions("1.0.0", "2.0.0"),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            compare_versions("1.2.3", "1.2.4"),
            Some(std::cmp::Ordering::Less)
        );
        assert_eq!(
            compare_versions("v1.0.0", "1.0.0"),
            Some(std::cmp::Ordering::Equal)
        );
    }

    #[test]
    fn test_single_constraint() {
        assert!(matches_single_constraint("2.0.0", ">=1.0.0"));
        assert!(matches_single_constraint("1.0.0", ">=1.0.0"));
        assert!(!matches_single_constraint("0.9.0", ">=1.0.0"));

        assert!(matches_single_constraint("1.0.0", "<2.0.0"));
        assert!(!matches_single_constraint("2.0.0", "<2.0.0"));

        assert!(matches_single_constraint("1.5.0", ">1.0.0"));
        assert!(!matches_single_constraint("1.0.0", ">1.0.0"));
    }

    #[test]
    fn test_and_constraint() {
        // >=2.0,<2.5.3
        assert!(matches_and_constraint("2.0.0", ">=2.0,<2.5.3"));
        assert!(matches_and_constraint("2.5.2", ">=2.0,<2.5.3"));
        assert!(!matches_and_constraint("2.5.3", ">=2.0,<2.5.3"));
        assert!(!matches_and_constraint("1.9.0", ">=2.0,<2.5.3"));
    }

    #[test]
    fn test_is_version_affected() {
        // Single range
        assert!(is_version_affected("2.0.0", ">=2.0,<2.5.3"));

        // OR conditions
        let constraint = ">=4.0,<4.4.50|>=5.0,<5.4.20|>=6.0,<6.3.7";
        assert!(is_version_affected("4.4.0", constraint));
        assert!(is_version_affected("5.4.0", constraint));
        assert!(is_version_affected("6.3.0", constraint));
        assert!(!is_version_affected("4.4.50", constraint));
        assert!(!is_version_affected("3.0.0", constraint));
    }

    #[test]
    fn test_normalize_severity() {
        assert_eq!(normalize_severity(Some("critical")), "CRITICAL");
        assert_eq!(normalize_severity(Some("HIGH")), "HIGH");
        assert_eq!(normalize_severity(Some("Medium")), "MEDIUM");
        assert_eq!(normalize_severity(None), "UNKNOWN");
    }
}
