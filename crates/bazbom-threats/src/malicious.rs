//! Malicious package detection

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use std::collections::HashSet;

/// Check if a package is known to be malicious
pub fn check_malicious(
    package_name: &str,
    malicious_db: &HashSet<String>,
) -> Option<ThreatIndicator> {
    if malicious_db.contains(package_name) {
        Some(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: String::new(),
            threat_level: ThreatLevel::Critical,
            threat_type: ThreatType::MaliciousPackage,
            description: format!("Package '{}' is known to be malicious", package_name),
            evidence: vec![
                "Listed in malicious package database".to_string(),
                "Reported by security researchers".to_string(),
            ],
            recommendation: "Remove this package immediately and scan for indicators of compromise"
                .to_string(),
        })
    } else {
        None
    }
}

/// Load malicious package database from external sources
/// This would typically fetch from OSV, GHSA, or other sources
pub async fn fetch_malicious_db() -> Result<Vec<String>, anyhow::Error> {
    // FUTURE ENHANCEMENT: Implement fetching from threat intelligence sources
    // Sources to integrate: OSV Malicious, GHSA malware, npm advisory DB,
    // PyPI malware reports, community-maintained blocklists
    // Requires: HTTP client, caching layer, offline-first design
    // For now, return a sample database
    Ok(vec![
        "malicious-test-package".to_string(),
        "backdoor-package".to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_malicious_package() {
        let mut db = HashSet::new();
        db.insert("evil-package".to_string());

        let result = check_malicious("evil-package", &db);
        assert!(result.is_some());

        let threat = result.unwrap();
        assert_eq!(threat.threat_level, ThreatLevel::Critical);
        assert_eq!(threat.threat_type, ThreatType::MaliciousPackage);
    }

    #[test]
    fn test_check_safe_package() {
        let db = HashSet::new();
        let result = check_malicious("safe-package", &db);
        assert!(result.is_none());
    }
}
