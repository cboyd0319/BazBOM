//! OS package upgrade intelligence
//!
//! Provides upgrade recommendations for OS packages (Alpine, Debian, Red Hat)
//! by analyzing security advisories and finding minimum versions that fix CVEs.

use anyhow::Result;
use bazbom_depsdev::System;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Upgrade recommendation for an OS package
#[derive(Debug, Clone)]
pub struct OsUpgradeRecommendation {
    /// Package name
    pub package: String,
    /// Currently installed version
    pub installed_version: String,
    /// Recommended version (fixes all known CVEs)
    pub recommended_version: Option<String>,
    /// CVEs fixed by upgrading
    pub fixes_cves: Vec<String>,
    /// Risk level of upgrade
    pub risk_level: OsUpgradeRisk,
    /// Notes about the upgrade
    pub notes: Vec<String>,
}

/// Risk level for OS package upgrades
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsUpgradeRisk {
    /// Safe patch update
    Low,
    /// Minor version change
    Medium,
    /// Major version change (potential breaking changes)
    High,
    /// No fix available
    NoFix,
}

/// Get upgrade recommendations for Alpine packages
pub fn get_alpine_upgrade_recommendations(
    packages: &[(String, String)], // (name, version)
    branch: &str,
) -> Result<Vec<OsUpgradeRecommendation>> {
    let mut recommendations = Vec::new();

    // Get all advisories for this branch
    let advisories = match bazbom_alpine::get_all_advisories(branch) {
        Ok(advs) => advs,
        Err(e) => {
            warn!("Failed to fetch Alpine advisories: {}", e);
            return Ok(vec![]);
        }
    };

    // Group advisories by package
    let mut pkg_advisories: HashMap<String, Vec<&bazbom_alpine::AlpineAdvisory>> = HashMap::new();
    for advisory in &advisories {
        pkg_advisories
            .entry(advisory.package.clone())
            .or_default()
            .push(advisory);
    }

    // Check each installed package
    for (pkg_name, installed_version) in packages {
        if let Some(advisories) = pkg_advisories.get(pkg_name) {
            let mut cves_to_fix = Vec::new();
            let mut max_fixed_version: Option<String> = None;

            for advisory in advisories {
                // Check if installed version is affected
                if bazbom_alpine::is_version_affected(installed_version, &advisory.fixed_version) {
                    cves_to_fix.extend(advisory.cves.clone());

                    // Track the highest fixed version needed
                    if let Some(ref current_max) = max_fixed_version {
                        if bazbom_alpine::is_version_affected(current_max, &advisory.fixed_version)
                        {
                            max_fixed_version = Some(advisory.fixed_version.clone());
                        }
                    } else {
                        max_fixed_version = Some(advisory.fixed_version.clone());
                    }
                }
            }

            if !cves_to_fix.is_empty() {
                let risk_level =
                    calculate_version_risk(installed_version, max_fixed_version.as_deref());

                recommendations.push(OsUpgradeRecommendation {
                    package: pkg_name.clone(),
                    installed_version: installed_version.clone(),
                    recommended_version: max_fixed_version,
                    fixes_cves: cves_to_fix,
                    risk_level,
                    notes: vec![],
                });
            }
        }
    }

    Ok(recommendations)
}

/// Get upgrade recommendations for Debian/Ubuntu packages
pub fn get_debian_upgrade_recommendations(
    packages: &[(String, String)], // (name, version)
    release: Option<&str>,
) -> Result<Vec<OsUpgradeRecommendation>> {
    let mut recommendations = Vec::new();

    for (pkg_name, installed_version) in packages {
        let advisories = match bazbom_debian::get_debian_cves(pkg_name) {
            Ok(advs) => advs,
            Err(e) => {
                debug!("Failed to fetch Debian CVEs for {}: {}", pkg_name, e);
                continue;
            }
        };

        let mut cves_to_fix = Vec::new();
        let mut max_fixed_version: Option<String> = None;
        let mut notes = Vec::new();

        for advisory in advisories {
            // Filter by release if specified
            if let Some(rel) = release {
                if advisory.release != rel {
                    continue;
                }
            }

            // Only consider open/undetermined vulnerabilities
            if advisory.status != "open" && advisory.status != "undetermined" {
                continue;
            }

            if let Some(ref fixed) = advisory.fixed_version {
                if bazbom_debian::is_version_affected(installed_version, fixed) {
                    cves_to_fix.push(advisory.cve_id.clone());

                    // Track highest fixed version
                    if let Some(ref current_max) = max_fixed_version {
                        if bazbom_debian::is_version_affected(current_max, fixed) {
                            max_fixed_version = Some(fixed.clone());
                        }
                    } else {
                        max_fixed_version = Some(fixed.clone());
                    }
                }
            } else {
                // No fix available yet
                cves_to_fix.push(advisory.cve_id.clone());
                notes.push(format!("No fix available for {} yet", advisory.cve_id));
            }
        }

        if !cves_to_fix.is_empty() {
            let risk_level = if max_fixed_version.is_none() {
                OsUpgradeRisk::NoFix
            } else {
                calculate_version_risk(installed_version, max_fixed_version.as_deref())
            };

            recommendations.push(OsUpgradeRecommendation {
                package: pkg_name.clone(),
                installed_version: installed_version.clone(),
                recommended_version: max_fixed_version,
                fixes_cves: cves_to_fix,
                risk_level,
                notes,
            });
        }
    }

    Ok(recommendations)
}

/// Get upgrade recommendations for Red Hat packages
pub fn get_redhat_upgrade_recommendations(
    packages: &[(String, String)], // (name, version)
) -> Result<Vec<OsUpgradeRecommendation>> {
    let mut recommendations = Vec::new();

    for (pkg_name, installed_version) in packages {
        let cves = match bazbom_redhat::search_cves(pkg_name, None) {
            Ok(c) => c,
            Err(e) => {
                debug!("Failed to fetch Red Hat CVEs for {}: {}", pkg_name, e);
                continue;
            }
        };

        let mut cves_to_fix = Vec::new();
        let mut max_fixed_version: Option<String> = None;
        let mut notes = Vec::new();

        for cve in cves {
            // Check affected packages
            for affected in &cve.affected_packages {
                if let Some(fixed_version) = extract_rpm_version(affected) {
                    if bazbom_redhat::is_version_affected(installed_version, &fixed_version) {
                        cves_to_fix.push(cve.cve_id.clone());

                        // Track highest fixed version
                        if let Some(ref current_max) = max_fixed_version {
                            if bazbom_redhat::is_version_affected(current_max, &fixed_version) {
                                max_fixed_version = Some(fixed_version);
                            }
                        } else {
                            max_fixed_version = Some(fixed_version);
                        }
                    }
                }
            }

            // Add advisory info
            if !cve.advisories.is_empty() {
                notes.push(format!("See advisories: {}", cve.advisories.join(", ")));
            }
        }

        if !cves_to_fix.is_empty() {
            let risk_level =
                calculate_version_risk(installed_version, max_fixed_version.as_deref());

            recommendations.push(OsUpgradeRecommendation {
                package: pkg_name.clone(),
                installed_version: installed_version.clone(),
                recommended_version: max_fixed_version,
                fixes_cves: cves_to_fix,
                risk_level,
                notes,
            });
        }
    }

    Ok(recommendations)
}

/// Get upgrade recommendations for any OS type
pub fn get_os_upgrade_recommendations(
    system: System,
    packages: &[(String, String)],
    release: Option<&str>,
) -> Result<Vec<OsUpgradeRecommendation>> {
    match system {
        System::Alpine => {
            let branch = release.unwrap_or("edge");
            get_alpine_upgrade_recommendations(packages, branch)
        }
        System::Debian | System::Rpm => {
            // Debian and Ubuntu use same API
            get_debian_upgrade_recommendations(packages, release)
        }
        _ => {
            warn!("OS upgrade recommendations not supported for {:?}", system);
            Ok(vec![])
        }
    }
}

/// Calculate risk level from version comparison
fn calculate_version_risk(installed: &str, recommended: Option<&str>) -> OsUpgradeRisk {
    let Some(recommended) = recommended else {
        return OsUpgradeRisk::NoFix;
    };

    // Simple heuristic: check major version change
    let installed_major = extract_major_version(installed);
    let recommended_major = extract_major_version(recommended);

    if installed_major != recommended_major {
        OsUpgradeRisk::High
    } else {
        // Check minor version
        let installed_minor = extract_minor_version(installed);
        let recommended_minor = extract_minor_version(recommended);

        if installed_minor != recommended_minor {
            OsUpgradeRisk::Medium
        } else {
            OsUpgradeRisk::Low
        }
    }
}

/// Extract major version number
fn extract_major_version(version: &str) -> u64 {
    version
        .split(&['.', '-', ':'][..])
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Extract minor version number
fn extract_minor_version(version: &str) -> u64 {
    version
        .split(&['.', '-', ':'][..])
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

/// Extract version from RPM package string like "openssl-1.0.2k-25.el7_9"
fn extract_rpm_version(pkg_string: &str) -> Option<String> {
    let parts: Vec<&str> = pkg_string.rsplitn(3, '-').collect();
    if parts.len() >= 2 {
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
    fn test_calculate_version_risk() {
        // Patch update
        assert_eq!(
            calculate_version_risk("1.2.3", Some("1.2.4")),
            OsUpgradeRisk::Low
        );

        // Minor update
        assert_eq!(
            calculate_version_risk("1.2.3", Some("1.3.0")),
            OsUpgradeRisk::Medium
        );

        // Major update
        assert_eq!(
            calculate_version_risk("1.2.3", Some("2.0.0")),
            OsUpgradeRisk::High
        );

        // No fix
        assert_eq!(calculate_version_risk("1.2.3", None), OsUpgradeRisk::NoFix);
    }

    #[test]
    fn test_extract_rpm_version() {
        assert_eq!(
            extract_rpm_version("openssl-1.0.2k-25.el7_9"),
            Some("1.0.2k-25.el7_9".to_string())
        );
    }
}
