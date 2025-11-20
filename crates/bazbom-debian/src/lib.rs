//! Debian/Ubuntu security tracker client
//!
//! Fetches vulnerability data from:
//! - Debian Security Tracker: https://security-tracker.debian.org/tracker/
//! - Ubuntu Security API: https://ubuntu.com/security/cves
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_debian::{get_debian_cves, get_ubuntu_cves};
//!
//! // Get Debian CVEs for a package
//! let cves = get_debian_cves("openssl")?;
//!
//! // Get Ubuntu CVEs
//! let ubuntu_cves = get_ubuntu_cves("openssl", "jammy")?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

const DEBIAN_TRACKER_BASE: &str = "https://security-tracker.debian.org/tracker";
const UBUNTU_API_BASE: &str = "https://ubuntu.com/security";

/// Security advisory for Debian/Ubuntu
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebianAdvisory {
    /// CVE identifier
    pub cve_id: String,
    /// Package name
    pub package: String,
    /// Fixed version (if available)
    pub fixed_version: Option<String>,
    /// Urgency (low, medium, high, critical)
    pub urgency: Option<String>,
    /// Release (e.g., "bookworm", "jammy")
    pub release: String,
    /// Status (open, resolved, not-affected)
    pub status: String,
}

/// Debian Security Tracker JSON response
#[derive(Debug, Deserialize)]
struct DebianTrackerResponse {
    #[serde(flatten)]
    releases: HashMap<String, DebianReleaseInfo>,
}

#[derive(Debug, Deserialize)]
struct DebianReleaseInfo {
    status: String,
    urgency: Option<String>,
    fixed_version: Option<String>,
}

/// Ubuntu CVE API response
#[derive(Debug, Deserialize)]
struct UbuntuCveResponse {
    cves: Vec<UbuntuCve>,
}

#[derive(Debug, Deserialize)]
struct UbuntuCve {
    id: String,
    packages: Vec<UbuntuPackage>,
}

#[derive(Debug, Deserialize)]
struct UbuntuPackage {
    name: String,
    statuses: Vec<UbuntuStatus>,
}

#[derive(Debug, Deserialize)]
struct UbuntuStatus {
    release_codename: String,
    status: String,
    #[allow(dead_code)]
    pocket: Option<String>,
}

/// Get CVEs affecting a Debian package
///
/// # Arguments
/// * `package` - Package name
///
/// # Returns
/// Map of CVE ID to release information
pub fn get_debian_cves(package: &str) -> Result<Vec<DebianAdvisory>> {
    let url = format!("{}/source-package/{}.json", DEBIAN_TRACKER_BASE, package);
    debug!("Fetching Debian CVEs from: {}", url);

    let response: HashMap<String, DebianTrackerResponse> = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch Debian CVEs for {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse Debian tracker response")?;

    let mut advisories = Vec::new();

    for (cve_id, tracker_data) in response {
        for (release, info) in tracker_data.releases {
            advisories.push(DebianAdvisory {
                cve_id: cve_id.clone(),
                package: package.to_string(),
                fixed_version: info.fixed_version,
                urgency: info.urgency,
                release,
                status: info.status,
            });
        }
    }

    Ok(advisories)
}

/// Get Ubuntu CVEs for a package in a specific release
///
/// # Arguments
/// * `package` - Package name
/// * `release` - Ubuntu release codename (e.g., "jammy", "focal")
///
/// # Returns
/// List of security advisories
pub fn get_ubuntu_cves(package: &str, release: &str) -> Result<Vec<DebianAdvisory>> {
    let url = format!(
        "{}/cves.json?package={}&release={}",
        UBUNTU_API_BASE, package, release
    );
    debug!("Fetching Ubuntu CVEs from: {}", url);

    let response: UbuntuCveResponse = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch Ubuntu CVEs for {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse Ubuntu CVE response")?;

    let mut advisories = Vec::new();

    for cve in response.cves {
        for pkg in cve.packages {
            if pkg.name == package {
                for status in pkg.statuses {
                    if status.release_codename == release {
                        advisories.push(DebianAdvisory {
                            cve_id: cve.id.clone(),
                            package: package.to_string(),
                            fixed_version: None, // Ubuntu API doesn't provide this directly
                            urgency: None,
                            release: status.release_codename,
                            status: status.status,
                        });
                    }
                }
            }
        }
    }

    Ok(advisories)
}

/// Compare Debian package versions
///
/// Debian versions use format: [epoch:]upstream_version[-debian_revision]
pub fn compare_debian_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // Parse epoch
    fn parse_epoch(v: &str) -> (u64, &str) {
        if let Some(idx) = v.find(':') {
            (v[..idx].parse().unwrap_or(0), &v[idx + 1..])
        } else {
            (0, v)
        }
    }

    let (a_epoch, a_rest) = parse_epoch(a);
    let (b_epoch, b_rest) = parse_epoch(b);

    // Compare epochs first
    match a_epoch.cmp(&b_epoch) {
        std::cmp::Ordering::Equal => {}
        other => return other,
    }

    // Compare upstream version and revision
    compare_version_strings(a_rest, b_rest)
}

/// Compare version strings character by character
fn compare_version_strings(a: &str, b: &str) -> std::cmp::Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(&ac), Some(&bc)) => {
                if ac.is_ascii_digit() && bc.is_ascii_digit() {
                    // Compare numeric segments
                    let a_num: String = a_chars
                        .by_ref()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();
                    let b_num: String = b_chars
                        .by_ref()
                        .take_while(|c| c.is_ascii_digit())
                        .collect();

                    let a_val: u64 = a_num.parse().unwrap_or(0);
                    let b_val: u64 = b_num.parse().unwrap_or(0);

                    match a_val.cmp(&b_val) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                } else {
                    // Compare characters
                    a_chars.next();
                    b_chars.next();

                    match ac.cmp(&bc) {
                        std::cmp::Ordering::Equal => continue,
                        other => return other,
                    }
                }
            }
        }
    }
}

/// Check if a version is affected (less than fixed version)
pub fn is_version_affected(installed: &str, fixed: &str) -> bool {
    compare_debian_versions(installed, fixed) == std::cmp::Ordering::Less
}

/// Get supported Debian releases
pub fn get_debian_releases() -> Vec<&'static str> {
    vec!["sid", "trixie", "bookworm", "bullseye", "buster"]
}

/// Get supported Ubuntu releases
pub fn get_ubuntu_releases() -> Vec<&'static str> {
    vec!["noble", "mantic", "jammy", "focal", "bionic"]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_debian_versions() {
        assert_eq!(
            compare_debian_versions("1.0.0", "1.0.0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_debian_versions("1.0.0", "2.0.0"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_debian_versions("1:1.0.0", "1.0.0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_debian_versions("1.0.0-1", "1.0.0-2"),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_is_version_affected() {
        assert!(is_version_affected("1.0.0", "1.0.1"));
        assert!(!is_version_affected("1.0.1", "1.0.1"));
        assert!(!is_version_affected("1.0.2", "1.0.1"));
    }
}
