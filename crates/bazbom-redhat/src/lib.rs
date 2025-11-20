//! Red Hat Security Data API client
//!
//! Fetches vulnerability data from Red Hat Security Data API:
//! https://access.redhat.com/documentation/en-us/red_hat_security_data_api/1.0/
//!
//! Covers: RHEL, CentOS, Fedora
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_redhat::{get_cve_info, search_cves};
//!
//! // Get CVE details
//! let cve = get_cve_info("CVE-2024-1234")?;
//!
//! // Search for CVEs affecting a package
//! let cves = search_cves("openssl", None)?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::debug;

const REDHAT_API_BASE: &str = "https://access.redhat.com/hydra/rest/securitydata";

/// Red Hat CVE information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedHatCve {
    /// CVE identifier
    #[serde(rename = "CVE")]
    pub cve_id: String,
    /// Severity (low, moderate, important, critical)
    pub severity: Option<String>,
    /// Public date
    pub public_date: Option<String>,
    /// Advisories (RHSA/RHBA)
    #[serde(default)]
    pub advisories: Vec<String>,
    /// Bugzilla ID
    pub bugzilla: Option<String>,
    /// Bugzilla description
    pub bugzilla_description: Option<String>,
    /// CVSS v3 score
    pub cvss3_score: Option<String>,
    /// CVSS v3 scoring vector
    pub cvss3_scoring_vector: Option<String>,
    /// CWE ID
    pub cwe: Option<String>,
    /// Affected packages
    #[serde(default)]
    pub affected_packages: Vec<String>,
    /// Package state
    #[serde(default)]
    pub package_state: Vec<PackageState>,
    /// Resource URL
    pub resource_url: Option<String>,
}

/// Package state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageState {
    /// Product name
    pub product_name: String,
    /// Fix state (Affected, Fixed, Not affected, etc.)
    pub fix_state: String,
    /// Package name
    pub package_name: String,
    /// CPE (Common Platform Enumeration)
    pub cpe: Option<String>,
}

/// Red Hat Security Advisory (RHSA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedHatAdvisory {
    /// Advisory ID (e.g., "RHSA-2024:1234")
    #[serde(rename = "RHSA")]
    pub rhsa_id: String,
    /// Severity
    pub severity: Option<String>,
    /// Release date
    pub released_on: Option<String>,
    /// CVEs fixed
    #[serde(default, rename = "CVEs")]
    pub cves: Vec<String>,
    /// Bugzillas
    #[serde(default)]
    pub bugzillas: Vec<String>,
    /// Affected packages
    #[serde(default)]
    pub released_packages: Vec<String>,
    /// Resource URL
    pub resource_url: Option<String>,
}

/// Get detailed CVE information
///
/// # Arguments
/// * `cve_id` - CVE identifier (e.g., "CVE-2024-1234")
pub fn get_cve_info(cve_id: &str) -> Result<RedHatCve> {
    let url = format!("{}/cve/{}.json", REDHAT_API_BASE, cve_id);
    debug!("Fetching Red Hat CVE info from: {}", url);

    let cve: RedHatCve = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch CVE {}", cve_id))?
        .body_mut()
        .read_json()
        .context("Failed to parse Red Hat CVE response")?;

    Ok(cve)
}

/// Search for CVEs
///
/// # Arguments
/// * `package` - Package name to search for
/// * `after_date` - Only return CVEs after this date (YYYY-MM-DD)
pub fn search_cves(package: &str, after_date: Option<&str>) -> Result<Vec<RedHatCve>> {
    let mut url = format!("{}/cve.json?package={}", REDHAT_API_BASE, package);

    if let Some(date) = after_date {
        url.push_str(&format!("&after={}", date));
    }

    debug!("Searching Red Hat CVEs: {}", url);

    let cves: Vec<RedHatCve> = ureq::get(&url)
        .call()
        .context(format!("Failed to search CVEs for {}", package))?
        .body_mut()
        .read_json()
        .context("Failed to parse Red Hat CVE search response")?;

    Ok(cves)
}

/// Get RHSA advisory details
///
/// # Arguments
/// * `rhsa_id` - Advisory ID (e.g., "RHSA-2024:1234")
pub fn get_advisory(rhsa_id: &str) -> Result<RedHatAdvisory> {
    let url = format!("{}/advisory/{}.json", REDHAT_API_BASE, rhsa_id);
    debug!("Fetching Red Hat advisory from: {}", url);

    let advisory: RedHatAdvisory = ureq::get(&url)
        .call()
        .context(format!("Failed to fetch advisory {}", rhsa_id))?
        .body_mut()
        .read_json()
        .context("Failed to parse Red Hat advisory response")?;

    Ok(advisory)
}

/// Search for advisories
///
/// # Arguments
/// * `package` - Package name
/// * `severity` - Filter by severity (low, moderate, important, critical)
pub fn search_advisories(
    package: Option<&str>,
    severity: Option<&str>,
) -> Result<Vec<RedHatAdvisory>> {
    let mut url = format!("{}/advisory.json?", REDHAT_API_BASE);

    if let Some(pkg) = package {
        url.push_str(&format!("package={}&", pkg));
    }

    if let Some(sev) = severity {
        url.push_str(&format!("severity={}&", sev));
    }

    debug!("Searching Red Hat advisories: {}", url);

    let advisories: Vec<RedHatAdvisory> = ureq::get(&url)
        .call()
        .context("Failed to search advisories")?
        .body_mut()
        .read_json()
        .context("Failed to parse Red Hat advisory search response")?;

    Ok(advisories)
}

/// Compare RPM package versions
///
/// RPM versions use: [epoch:]version-release
pub fn compare_rpm_versions(a: &str, b: &str) -> std::cmp::Ordering {
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

    match a_epoch.cmp(&b_epoch) {
        std::cmp::Ordering::Equal => {}
        other => return other,
    }

    // Compare version-release
    compare_version_strings(a_rest, b_rest)
}

fn compare_version_strings(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split(&['.', '-'][..]).collect();
    let b_parts: Vec<&str> = b.split(&['.', '-'][..]).collect();

    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        let a_part = a_parts.get(i).unwrap_or(&"0");
        let b_part = b_parts.get(i).unwrap_or(&"0");

        // Try numeric comparison first
        match (a_part.parse::<u64>(), b_part.parse::<u64>()) {
            (Ok(a_num), Ok(b_num)) => match a_num.cmp(&b_num) {
                std::cmp::Ordering::Equal => continue,
                other => return other,
            },
            _ => {
                // Fall back to string comparison
                match a_part.cmp(b_part) {
                    std::cmp::Ordering::Equal => continue,
                    other => return other,
                }
            }
        }
    }

    std::cmp::Ordering::Equal
}

/// Check if a version is affected
pub fn is_version_affected(installed: &str, fixed: &str) -> bool {
    compare_rpm_versions(installed, fixed) == std::cmp::Ordering::Less
}

/// Normalize Red Hat severity to standard format
pub fn normalize_severity(severity: Option<&str>) -> String {
    match severity.map(|s| s.to_lowercase()).as_deref() {
        Some("critical") => "CRITICAL".to_string(),
        Some("important") => "HIGH".to_string(),
        Some("moderate") => "MEDIUM".to_string(),
        Some("low") => "LOW".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_rpm_versions() {
        assert_eq!(
            compare_rpm_versions("1.0.0-1", "1.0.0-1"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_rpm_versions("1.0.0-1", "1.0.0-2"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_rpm_versions("1:1.0.0-1", "1.0.0-1"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_normalize_severity() {
        assert_eq!(normalize_severity(Some("Critical")), "CRITICAL");
        assert_eq!(normalize_severity(Some("important")), "HIGH");
        assert_eq!(normalize_severity(Some("MODERATE")), "MEDIUM");
        assert_eq!(normalize_severity(None), "UNKNOWN");
    }
}
