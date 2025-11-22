//! Alpine Linux security database client
//!
//! Fetches vulnerability data from Alpine's secdb (security database).
//! Data source: https://secdb.alpinelinux.org/
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bazbom_alpine::{get_advisories, is_version_affected};
//!
//! // Get all advisories for Alpine 3.19
//! let advisories = get_advisories("v3.19", "main")?;
//!
//! // Check if a package version is vulnerable
//! for advisory in &advisories {
//!     if advisory.package == "openssl" {
//!         println!("CVEs: {:?}", advisory.cves);
//!     }
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

const ALPINE_SECDB_BASE: &str = "https://secdb.alpinelinux.org";

/// Security advisory from Alpine secdb
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlpineAdvisory {
    /// Package name
    pub package: String,
    /// Fixed version (packages below this are vulnerable)
    pub fixed_version: String,
    /// CVE identifiers
    pub cves: Vec<String>,
    /// Alpine branch (e.g., "v3.19")
    pub branch: String,
    /// Repository (main, community)
    pub repository: String,
}

/// Raw secdb format
#[derive(Debug, Deserialize)]
struct SecdbFile {
    #[serde(default)]
    packages: Vec<SecdbPackage>,
}

#[derive(Debug, Deserialize)]
struct SecdbPackage {
    pkg: SecdbPkgInfo,
}

#[derive(Debug, Deserialize)]
struct SecdbPkgInfo {
    name: String,
    secfixes: HashMap<String, Vec<String>>,
}

/// Get security advisories for an Alpine branch and repository
///
/// # Arguments
/// * `branch` - Alpine branch (e.g., "v3.19", "v3.18", "edge")
/// * `repository` - Repository name ("main" or "community")
///
/// # Returns
/// List of security advisories
pub fn get_advisories(branch: &str, repository: &str) -> Result<Vec<AlpineAdvisory>> {
    let url = format!("{}/{}/{}.json", ALPINE_SECDB_BASE, branch, repository);
    debug!("Fetching Alpine secdb from: {}", url);

    let response: SecdbFile = ureq::get(&url)
        .call()
        .context(format!(
            "Failed to fetch Alpine secdb for {}/{}",
            branch, repository
        ))?
        .body_mut()
        .read_json()
        .context("Failed to parse Alpine secdb response")?;

    let mut advisories = Vec::new();

    for pkg in response.packages {
        for (version, cves) in pkg.pkg.secfixes {
            if !cves.is_empty() {
                advisories.push(AlpineAdvisory {
                    package: pkg.pkg.name.clone(),
                    fixed_version: version.clone(),
                    cves: cves.into_iter().filter(|c| c.starts_with("CVE-")).collect(),
                    branch: branch.to_string(),
                    repository: repository.to_string(),
                });
            }
        }
    }

    Ok(advisories)
}

/// Get advisories for all repositories in a branch
pub fn get_all_advisories(branch: &str) -> Result<Vec<AlpineAdvisory>> {
    let mut all = Vec::new();

    for repo in &["main", "community"] {
        match get_advisories(branch, repo) {
            Ok(advisories) => all.extend(advisories),
            Err(e) => {
                debug!("Failed to fetch {}/{}: {}", branch, repo, e);
            }
        }
    }

    Ok(all)
}

/// Check if an installed version is affected by an advisory
///
/// Alpine versions use format like "1.2.3-r4" where -r4 is the release number
pub fn is_version_affected(installed: &str, fixed: &str) -> bool {
    compare_alpine_versions(installed, fixed) == std::cmp::Ordering::Less
}

/// Compare two Alpine package versions
fn compare_alpine_versions(a: &str, b: &str) -> std::cmp::Ordering {
    // Split version and release (e.g., "1.2.3-r4" -> "1.2.3", "4")
    let parse_version = |v: &str| -> (Vec<u64>, u64) {
        let (version, release) = if let Some(idx) = v.rfind("-r") {
            (&v[..idx], v[idx + 2..].parse().unwrap_or(0))
        } else {
            (v, 0u64)
        };

        let parts: Vec<u64> = version
            .split('.')
            .filter_map(|s| {
                s.chars()
                    .take_while(|c| c.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .ok()
            })
            .collect();

        (parts, release)
    };

    let (a_parts, a_rel) = parse_version(a);
    let (b_parts, b_rel) = parse_version(b);

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

    // If versions are equal, compare release numbers
    a_rel.cmp(&b_rel)
}

/// Get the list of supported Alpine branches
pub fn get_supported_branches() -> Vec<&'static str> {
    vec!["edge", "v3.20", "v3.19", "v3.18", "v3.17", "v3.16"]
}

/// Detect Alpine branch from /etc/alpine-release content
pub fn detect_branch(release_content: &str) -> Option<String> {
    let version = release_content.trim();
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() >= 2 {
        Some(format!("v{}.{}", parts[0], parts[1]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_versions() {
        assert_eq!(
            compare_alpine_versions("1.2.3-r0", "1.2.3-r0"),
            std::cmp::Ordering::Equal
        );
        assert_eq!(
            compare_alpine_versions("1.2.3-r0", "1.2.3-r1"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_alpine_versions("1.2.3-r1", "1.2.3-r0"),
            std::cmp::Ordering::Greater
        );
        assert_eq!(
            compare_alpine_versions("1.2.3", "1.2.4"),
            std::cmp::Ordering::Less
        );
        assert_eq!(
            compare_alpine_versions("2.0.0", "1.9.9"),
            std::cmp::Ordering::Greater
        );
    }

    #[test]
    fn test_is_version_affected() {
        assert!(is_version_affected("1.2.2-r0", "1.2.3-r0"));
        assert!(!is_version_affected("1.2.3-r0", "1.2.3-r0"));
        assert!(!is_version_affected("1.2.4-r0", "1.2.3-r0"));
    }

    #[test]
    fn test_detect_branch() {
        assert_eq!(detect_branch("3.19.1"), Some("v3.19".to_string()));
        assert_eq!(detect_branch("3.18.0"), Some("v3.18".to_string()));
        assert_eq!(detect_branch("edge"), None);
    }
}
