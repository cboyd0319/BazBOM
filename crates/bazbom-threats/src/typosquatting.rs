//! Typosquatting detection
//!
//! Detects packages that may be typosquatting attacks on popular packages

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use std::collections::HashSet;
use strsim::{levenshtein, normalized_levenshtein};

/// Check if a package might be a typosquatting attempt
pub fn check_typosquatting(package_name: &str, known_packages: &HashSet<String>) -> Option<ThreatIndicator> {
    // Find similar package names
    for known_pkg in known_packages {
        let similarity = normalized_levenshtein(package_name, known_pkg);
        let distance = levenshtein(package_name, known_pkg);

        // High similarity but not exact match suggests typosquatting
        if similarity > 0.8 && similarity < 1.0 && distance <= 2 {
            return Some(ThreatIndicator {
                package_name: package_name.to_string(),
                package_version: String::new(),
                threat_level: determine_threat_level(similarity, distance),
                threat_type: ThreatType::Typosquatting,
                description: format!(
                    "Package '{}' may be typosquatting on '{}'",
                    package_name, known_pkg
                ),
                evidence: vec![
                    format!("Similar to popular package '{}' (similarity: {:.2})", known_pkg, similarity),
                    format!("Edit distance: {} characters", distance),
                    "Common typosquatting patterns detected".to_string(),
                ],
                recommendation: format!(
                    "Verify this is the intended package. Consider using '{}' instead",
                    known_pkg
                ),
            });
        }
    }

    None
}

/// Determine threat level based on similarity metrics
fn determine_threat_level(similarity: f64, distance: usize) -> ThreatLevel {
    if similarity > 0.95 && distance == 1 {
        ThreatLevel::Critical // Very likely typosquatting (1 char difference)
    } else if similarity > 0.9 && distance <= 2 {
        ThreatLevel::High // Likely typosquatting (2 char difference)
    } else if similarity > 0.85 {
        ThreatLevel::Medium // Possible typosquatting
    } else {
        ThreatLevel::Low // Unlikely but flagged
    }
}

/// Common typosquatting patterns
pub fn detect_common_patterns(package_name: &str) -> Vec<String> {
    let mut patterns = Vec::new();

    // Check for common substitutions
    if package_name.contains("0") || package_name.contains("1") {
        patterns.push("Number substitution detected (0 for O, 1 for l)".to_string());
    }

    // Check for extra/missing characters
    if package_name.contains("--") {
        patterns.push("Double dash detected".to_string());
    }

    // Check for underscore/dash confusion
    let has_underscore = package_name.contains('_');
    let has_dash = package_name.contains('-');
    if has_underscore && has_dash {
        patterns.push("Mixed underscore and dash usage".to_string());
    }

    patterns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typosquatting_detection() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Very similar - likely typosquatting
        let result = check_typosquatting("lodosh", &known);
        assert!(result.is_some());

        let threat = result.unwrap();
        assert_eq!(threat.threat_type, ThreatType::Typosquatting);
    }

    #[test]
    fn test_safe_package_no_typosquatting() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Completely different - no typosquatting
        let result = check_typosquatting("react", &known);
        assert!(result.is_none());
    }

    #[test]
    fn test_exact_match_no_typosquatting() {
        let mut known = HashSet::new();
        known.insert("lodash".to_string());

        // Exact match - not typosquatting
        let result = check_typosquatting("lodash", &known);
        assert!(result.is_none());
    }

    #[test]
    fn test_common_patterns() {
        let patterns = detect_common_patterns("l0dash");
        assert!(patterns.iter().any(|p| p.contains("Number substitution")));

        let patterns2 = detect_common_patterns("lodash--extra");
        assert!(patterns2.iter().any(|p| p.contains("Double dash")));
    }
}
