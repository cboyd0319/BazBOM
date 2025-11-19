//! Protestware detection
//!
//! Detects known protestware packages that intentionally break or display
//! political messages as a form of protest.

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use std::collections::HashMap;

lazy_static::lazy_static! {
    /// Known protestware packages with affected versions
    static ref PROTESTWARE_DB: HashMap<&'static str, ProtestwareEntry> = {
        let mut m = HashMap::new();

        // colors - infinite loop added in 1.4.1+
        m.insert("colors", ProtestwareEntry {
            affected_versions: vec!["1.4.1", "1.4.2", "1.4.44-liberty-2"],
            description: "Infinite loop causing denial of service",
            cve: Some("N/A - Intentional"),
            safe_version: "1.4.0",
        });

        // faker - deleted and republished with breaking changes
        m.insert("faker", ProtestwareEntry {
            affected_versions: vec!["6.6.6"],
            description: "Package wiped and republished with breaking changes",
            cve: None,
            safe_version: "5.5.3",
        });

        // node-ipc - peacenotwar module added
        m.insert("node-ipc", ProtestwareEntry {
            affected_versions: vec!["10.1.1", "10.1.2", "10.1.3", "11.0.0", "11.1.0"],
            description: "Destructive payload targeting Russian/Belarusian IPs",
            cve: Some("CVE-2022-23812"),
            safe_version: "9.2.1",
        });

        // event-source-polyfill
        m.insert("event-source-polyfill", ProtestwareEntry {
            affected_versions: vec!["1.0.26", "1.0.27", "1.0.28"],
            description: "Anti-war message display",
            cve: None,
            safe_version: "1.0.25",
        });

        // es5-ext
        m.insert("es5-ext", ProtestwareEntry {
            affected_versions: vec!["0.10.53", "0.10.54", "0.10.55", "0.10.56", "0.10.57", "0.10.58", "0.10.59", "0.10.60", "0.10.61"],
            description: "Political message in postinstall",
            cve: None,
            safe_version: "0.10.52",
        });

        // styled-components
        m.insert("styled-components", ProtestwareEntry {
            affected_versions: vec!["5.3.4"],
            description: "Anti-war console message",
            cve: None,
            safe_version: "5.3.3",
        });

        m
    };
}

/// Protestware entry with version and description
struct ProtestwareEntry {
    affected_versions: Vec<&'static str>,
    description: &'static str,
    cve: Option<&'static str>,
    safe_version: &'static str,
}

/// Check if a package is known protestware
pub fn check_protestware(
    package_name: &str,
    package_version: &str,
) -> Option<ThreatIndicator> {
    // Normalize package name
    let normalized_name = package_name.to_lowercase();

    if let Some(entry) = PROTESTWARE_DB.get(normalized_name.as_str()) {
        // Check if the version is affected
        let is_affected = entry.affected_versions.iter().any(|v| {
            version_matches(package_version, v)
        });

        if is_affected {
            let mut evidence = vec![
                format!("Package '{}' version '{}' is known protestware", package_name, package_version),
                format!("Impact: {}", entry.description),
            ];

            if let Some(cve) = entry.cve {
                evidence.push(format!("CVE: {}", cve));
            }

            return Some(ThreatIndicator {
                package_name: package_name.to_string(),
                package_version: package_version.to_string(),
                threat_level: ThreatLevel::High,
                threat_type: ThreatType::SuspiciousBehavior,
                description: format!(
                    "Package '{}@{}' is known protestware with intentional malicious behavior",
                    package_name, package_version
                ),
                evidence,
                recommendation: format!(
                    "Downgrade to safe version '{}' or find alternative package",
                    entry.safe_version
                ),
            });
        }
    }

    None
}

/// Check if version matches (simple string match or range)
fn version_matches(actual: &str, pattern: &str) -> bool {
    // Exact match
    if actual == pattern {
        return true;
    }

    // Handle version ranges (simplified)
    if pattern.ends_with('+') {
        let base = pattern.trim_end_matches('+');
        return actual >= base;
    }

    false
}

/// Get list of all known protestware packages
pub fn get_protestware_list() -> Vec<String> {
    PROTESTWARE_DB.keys().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_colors() {
        let result = check_protestware("colors", "1.4.1");
        assert!(result.is_some());
        let threat = result.unwrap();
        assert_eq!(threat.threat_level, ThreatLevel::High);
    }

    #[test]
    fn test_detect_node_ipc() {
        let result = check_protestware("node-ipc", "10.1.1");
        assert!(result.is_some());
        let threat = result.unwrap();
        assert!(threat.evidence.iter().any(|e| e.contains("CVE-2022-23812")));
    }

    #[test]
    fn test_safe_colors_version() {
        let result = check_protestware("colors", "1.4.0");
        assert!(result.is_none());
    }

    #[test]
    fn test_unknown_package() {
        let result = check_protestware("safe-package", "1.0.0");
        assert!(result.is_none());
    }
}
