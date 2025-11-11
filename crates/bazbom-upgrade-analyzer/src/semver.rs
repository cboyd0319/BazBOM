use crate::models::RiskLevel;
use semver::Version;

/// Analyze upgrade risk based on semantic versioning
pub fn analyze_semver_risk(from: &str, to: &str) -> RiskLevel {
    // Try to parse as semver
    let from_ver = Version::parse(from);
    let to_ver = Version::parse(to);

    match (from_ver, to_ver) {
        (Ok(from_v), Ok(to_v)) => {
            if from_v.major != to_v.major {
                // Major version change = high risk
                RiskLevel::High
            } else if from_v.minor != to_v.minor {
                // Minor version change = medium risk
                RiskLevel::Medium
            } else {
                // Patch version change = low risk
                RiskLevel::Low
            }
        }
        _ => {
            // Non-semver versions - assume medium risk
            RiskLevel::Medium
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_risk_assessment() {
        assert_eq!(analyze_semver_risk("2.17.0", "2.17.1"), RiskLevel::Low);
        assert_eq!(analyze_semver_risk("2.17.0", "2.18.0"), RiskLevel::Medium);
        assert_eq!(analyze_semver_risk("2.17.0", "3.0.0"), RiskLevel::High);
    }
}
