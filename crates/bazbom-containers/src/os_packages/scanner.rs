//! OS package vulnerability scanner
//!
//! Scans installed OS packages against native BazBOM advisory databases.

use super::{detect_os, parse_apk_installed, parse_dpkg_status, parse_rpm_database};
use super::{InstalledPackage, OsInfo, OsType};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::Path;
use tracing::{debug, info, warn};

/// Vulnerability found in an OS package
#[derive(Debug, Clone)]
pub struct OsVulnerability {
    /// CVE identifier
    pub cve_id: String,
    /// Affected package name
    pub package: String,
    /// Installed version
    pub installed_version: String,
    /// Fixed version (if available)
    pub fixed_version: Option<String>,
    /// Severity (CRITICAL, HIGH, MEDIUM, LOW)
    pub severity: String,
    /// OS type where found
    pub os_type: OsType,
}

/// Result of scanning OS packages
#[derive(Debug)]
pub struct OsScanResult {
    /// OS information
    pub os_info: OsInfo,
    /// Installed packages
    pub packages: Vec<InstalledPackage>,
    /// Vulnerabilities found
    pub vulnerabilities: Vec<OsVulnerability>,
}

/// Scan OS packages in a container filesystem for vulnerabilities
///
/// # Arguments
/// * `root` - Path to extracted container filesystem
///
/// # Returns
/// Scan results including OS info, packages, and vulnerabilities
pub fn scan_os_packages(root: &Path) -> Result<OsScanResult> {
    // 1. Detect OS type
    let os_info = detect_os(root).context("Failed to detect OS")?;
    info!(
        "Detected OS: {} {}",
        os_info.pretty_name, os_info.version_id
    );

    // 2. Parse installed packages based on OS type
    let packages = match os_info.os_type {
        OsType::Alpine => parse_apk_installed(root)?,
        OsType::Debian | OsType::Ubuntu => parse_dpkg_status(root)?,
        OsType::Rhel | OsType::CentOS | OsType::Fedora => parse_rpm_database(root)?,
        OsType::Unknown(ref name) => {
            warn!("Unknown OS type: {}, trying all parsers", name);
            // Try each parser
            let mut pkgs = parse_dpkg_status(root)?;
            if pkgs.is_empty() {
                pkgs = parse_apk_installed(root)?;
            }
            if pkgs.is_empty() {
                pkgs = parse_rpm_database(root)?;
            }
            pkgs
        }
    };

    info!("Found {} installed packages", packages.len());

    // 3. Look up vulnerabilities
    let vulnerabilities = match os_info.os_type {
        OsType::Alpine => scan_alpine_packages(&packages, &os_info)?,
        OsType::Debian => scan_debian_packages(&packages, &os_info)?,
        OsType::Ubuntu => scan_ubuntu_packages(&packages, &os_info)?,
        OsType::Rhel | OsType::CentOS | OsType::Fedora => {
            scan_redhat_packages(&packages, &os_info)?
        }
        OsType::Unknown(_) => {
            warn!("Cannot scan vulnerabilities for unknown OS type");
            vec![]
        }
    };

    info!("Found {} vulnerabilities", vulnerabilities.len());

    Ok(OsScanResult {
        os_info,
        packages,
        vulnerabilities,
    })
}

fn scan_alpine_packages(
    packages: &[InstalledPackage],
    os_info: &OsInfo,
) -> Result<Vec<OsVulnerability>> {
    let mut vulnerabilities = Vec::new();

    // Determine Alpine branch from version (e.g., "3.19.1" -> "v3.19")
    let branch =
        bazbom_alpine::detect_branch(&os_info.version_id).unwrap_or_else(|| "edge".to_string());

    debug!("Fetching Alpine advisories for branch: {}", branch);

    // Get all advisories for this branch
    let advisories = match bazbom_alpine::get_all_advisories(&branch) {
        Ok(advs) => advs,
        Err(e) => {
            warn!("Failed to fetch Alpine advisories: {}", e);
            return Ok(vec![]);
        }
    };

    // Check each installed package against advisories
    for pkg in packages {
        for advisory in &advisories {
            if advisory.package == pkg.name {
                // Check if installed version is affected
                if bazbom_alpine::is_version_affected(&pkg.version, &advisory.fixed_version) {
                    for cve in &advisory.cves {
                        vulnerabilities.push(OsVulnerability {
                            cve_id: cve.clone(),
                            package: pkg.name.clone(),
                            installed_version: pkg.version.clone(),
                            fixed_version: Some(advisory.fixed_version.clone()),
                            severity: "UNKNOWN".to_string(), // Alpine secdb doesn't provide severity
                            os_type: OsType::Alpine,
                        });
                    }
                }
            }
        }
    }

    Ok(vulnerabilities)
}

fn scan_debian_packages(
    packages: &[InstalledPackage],
    _os_info: &OsInfo,
) -> Result<Vec<OsVulnerability>> {
    // Use parallel iteration for faster API calls
    let vulnerabilities: Vec<OsVulnerability> = packages
        .par_iter()
        .flat_map(|pkg| {
            // Use source package name if available
            let pkg_name = pkg.source.as_ref().unwrap_or(&pkg.name);

            let advisories = match bazbom_debian::get_debian_cves(pkg_name) {
                Ok(advs) => advs,
                Err(e) => {
                    debug!("Failed to fetch Debian CVEs for {}: {}", pkg_name, e);
                    return vec![];
                }
            };

            let mut pkg_vulns = Vec::new();
            for advisory in advisories {
                // Only check "open" status vulnerabilities
                if advisory.status != "open" && advisory.status != "undetermined" {
                    continue;
                }

                // Check if we have a fixed version and if installed is affected
                if let Some(ref fixed) = advisory.fixed_version {
                    if bazbom_debian::is_version_affected(&pkg.version, fixed) {
                        let severity = match advisory.urgency.as_deref() {
                            Some("critical") | Some("high") => "HIGH",
                            Some("medium") => "MEDIUM",
                            Some("low") => "LOW",
                            _ => "UNKNOWN",
                        }
                        .to_string();

                        pkg_vulns.push(OsVulnerability {
                            cve_id: advisory.cve_id,
                            package: pkg.name.clone(),
                            installed_version: pkg.version.clone(),
                            fixed_version: Some(fixed.clone()),
                            severity,
                            os_type: OsType::Debian,
                        });
                    }
                } else {
                    // No fix available yet - still vulnerable
                    pkg_vulns.push(OsVulnerability {
                        cve_id: advisory.cve_id,
                        package: pkg.name.clone(),
                        installed_version: pkg.version.clone(),
                        fixed_version: None,
                        severity: "UNKNOWN".to_string(),
                        os_type: OsType::Debian,
                    });
                }
            }
            pkg_vulns
        })
        .collect();

    Ok(vulnerabilities)
}

fn scan_ubuntu_packages(
    packages: &[InstalledPackage],
    os_info: &OsInfo,
) -> Result<Vec<OsVulnerability>> {
    // Map version ID to codename
    let release = match os_info.version_id.as_str() {
        "24.04" => "noble",
        "23.10" => "mantic",
        "22.04" => "jammy",
        "20.04" => "focal",
        "18.04" => "bionic",
        _ => {
            warn!("Unknown Ubuntu version: {}", os_info.version_id);
            return Ok(vec![]);
        }
    };

    // Use parallel iteration for faster API calls
    let vulnerabilities: Vec<OsVulnerability> = packages
        .par_iter()
        .flat_map(|pkg| {
            let pkg_name = pkg.source.as_ref().unwrap_or(&pkg.name);

            let advisories = match bazbom_debian::get_ubuntu_cves(pkg_name, release) {
                Ok(advs) => advs,
                Err(e) => {
                    debug!("Failed to fetch Ubuntu CVEs for {}: {}", pkg_name, e);
                    return vec![];
                }
            };

            let mut pkg_vulns = Vec::new();
            for advisory in advisories {
                if advisory.status == "needed" || advisory.status == "pending" {
                    pkg_vulns.push(OsVulnerability {
                        cve_id: advisory.cve_id,
                        package: pkg.name.clone(),
                        installed_version: pkg.version.clone(),
                        fixed_version: advisory.fixed_version,
                        severity: "UNKNOWN".to_string(),
                        os_type: OsType::Ubuntu,
                    });
                }
            }
            pkg_vulns
        })
        .collect();

    Ok(vulnerabilities)
}

fn scan_redhat_packages(
    packages: &[InstalledPackage],
    _os_info: &OsInfo,
) -> Result<Vec<OsVulnerability>> {
    let mut vulnerabilities = Vec::new();

    for pkg in packages {
        let cves = match bazbom_redhat::search_cves(&pkg.name, None) {
            Ok(c) => c,
            Err(e) => {
                debug!("Failed to fetch Red Hat CVEs for {}: {}", pkg.name, e);
                continue;
            }
        };

        for cve in cves {
            // Check affected packages for version info
            for affected in &cve.affected_packages {
                // affected_packages contains strings like "openssl-1.0.2k-25.el7_9"
                // We need to extract version and compare
                if let Some(fixed_version) = extract_rpm_version(affected) {
                    if bazbom_redhat::is_version_affected(&pkg.version, &fixed_version) {
                        let severity = bazbom_redhat::normalize_severity(cve.severity.as_deref());

                        vulnerabilities.push(OsVulnerability {
                            cve_id: cve.cve_id.clone(),
                            package: pkg.name.clone(),
                            installed_version: pkg.version.clone(),
                            fixed_version: Some(fixed_version),
                            severity,
                            os_type: OsType::Rhel,
                        });
                    }
                }
            }
        }
    }

    Ok(vulnerabilities)
}

/// Extract version from RPM package string like "openssl-1.0.2k-25.el7_9"
fn extract_rpm_version(pkg_string: &str) -> Option<String> {
    // RPM names are: name-version-release.arch
    // Find the second-to-last dash to split name from version
    let parts: Vec<&str> = pkg_string.rsplitn(3, '-').collect();
    if parts.len() >= 2 {
        // Reconstruct version-release
        let release = parts[0];
        let version = parts[1];
        Some(format!("{}-{}", version, release))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_rpm_version() {
        assert_eq!(
            extract_rpm_version("openssl-1.0.2k-25.el7_9"),
            Some("1.0.2k-25.el7_9".to_string())
        );
        assert_eq!(
            extract_rpm_version("kernel-3.10.0-1160.el7"),
            Some("3.10.0-1160.el7".to_string())
        );
    }
}
