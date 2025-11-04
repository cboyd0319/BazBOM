//! Supply chain attack detection
//!
//! Detects indicators of supply chain attacks

use crate::{ThreatIndicator, ThreatLevel, ThreatType};

/// Check for supply chain attack indicators
pub fn check_supply_chain_indicators(
    package_name: &str,
    package_version: &str,
) -> Vec<ThreatIndicator> {
    let mut threats = Vec::new();

    // Check for suspicious version patterns
    if is_suspicious_version(package_version) {
        threats.push(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: package_version.to_string(),
            threat_level: ThreatLevel::Medium,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Suspicious version pattern detected".to_string(),
            evidence: vec![
                format!("Version '{}' follows unusual pattern", package_version),
                "May indicate compromised release".to_string(),
            ],
            recommendation: "Verify package authenticity and check maintainer account".to_string(),
        });
    }

    // Check for suspicious package names
    if is_suspicious_name(package_name) {
        threats.push(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: package_version.to_string(),
            threat_level: ThreatLevel::Low,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Package name contains suspicious patterns".to_string(),
            evidence: vec![
                "Name matches common attack patterns".to_string(),
                "May be attempting to appear legitimate".to_string(),
            ],
            recommendation: "Review package source code and maintainer history".to_string(),
        });
    }

    threats
}

/// Check if version string is suspicious
fn is_suspicious_version(version: &str) -> bool {
    // Very long version strings may be suspicious
    if version.len() > 20 {
        return true;
    }

    // Unusual characters in version
    if version.contains(char::is_whitespace) {
        return true;
    }

    // Non-standard version formats
    if !version
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '+')
    {
        return true;
    }

    false
}

/// Check if package name is suspicious
fn is_suspicious_name(name: &str) -> bool {
    let suspicious_patterns = [
        "test-",
        "tmp-",
        "temp-",
        "fake-",
        "malicious-",
        "-backdoor",
        "-exploit",
        "-hack",
        "-pwn",
    ];

    for pattern in &suspicious_patterns {
        if name.contains(pattern) {
            return true;
        }
    }

    false
}

/// Indicators of compromised maintainer accounts
pub fn check_compromised_account_indicators(
    _package_name: &str,
    _maintainer_email: &str,
    _recent_changes: &[String],
) -> Option<ThreatIndicator> {
    // TODO: Implement more sophisticated checks
    // For now, return None (not implemented)
    None
}

/// Check for backdoor patterns in package metadata
pub fn check_backdoor_indicators(
    _package_name: &str,
    _dependencies: &[String],
) -> Option<ThreatIndicator> {
    // TODO: Implement backdoor detection
    // This would analyze:
    // - Obfuscated dependencies
    // - Network connections in install scripts
    // - Suspicious file operations
    // - Known backdoor patterns
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suspicious_version() {
        // This is long enough to be suspicious (>20 chars)
        assert!(is_suspicious_version("1.0.0.0.0.0.0.0.0.0.0.0"));
        assert!(is_suspicious_version("1.0 .0"));
        assert!(is_suspicious_version("1.0.0@malicious"));
        assert!(!is_suspicious_version("1.0.0"));
        assert!(!is_suspicious_version("2.5.3-beta.1"));
    }

    #[test]
    fn test_suspicious_name() {
        assert!(is_suspicious_name("test-package"));
        assert!(is_suspicious_name("my-backdoor"));
        assert!(is_suspicious_name("fake-react"));
        assert!(!is_suspicious_name("react"));
        assert!(!is_suspicious_name("lodash"));
    }

    #[test]
    fn test_check_supply_chain_indicators() {
        let threats = check_supply_chain_indicators("test-package", "1.0.0@hack");
        assert!(!threats.is_empty());
    }

    #[test]
    fn test_safe_package_no_indicators() {
        let threats = check_supply_chain_indicators("safe-package", "1.0.0");
        assert!(threats.is_empty());
    }
}
