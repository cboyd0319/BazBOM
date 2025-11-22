//! Comprehensive package risk assessment
//!
//! Analyzes packages for various risk indicators including:
//! - Binary blobs
//! - Metadata anomalies
//! - Abandonment risk
//! - Bus factor
//! - Release velocity anomalies
//! - License risks
//! - Obfuscation patterns

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use chrono::{DateTime, Utc};
use regex::Regex;

/// Package metadata for risk assessment
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub repository_url: Option<String>,
    pub homepage_url: Option<String>,
    pub license: Option<String>,
    pub maintainers: Vec<String>,
    pub last_publish_date: Option<DateTime<Utc>>,
    pub first_publish_date: Option<DateTime<Utc>>,
    pub total_versions: usize,
    pub weekly_downloads: Option<u64>,
}

/// Risk assessment result
#[derive(Debug)]
pub struct RiskAssessment {
    pub threats: Vec<ThreatIndicator>,
    pub risk_score: u32, // 0-100
}

/// Comprehensive package risk assessment
pub fn assess_package_risk(metadata: &PackageMetadata) -> RiskAssessment {
    let mut threats = Vec::new();
    let mut risk_score: u32 = 0;

    // Check metadata anomalies
    if let Some(threat) = check_metadata_anomalies(metadata) {
        risk_score += 15;
        threats.push(threat);
    }

    // Check abandonment risk
    if let Some(threat) = check_abandonment_risk(metadata) {
        risk_score += 20;
        threats.push(threat);
    }

    // Check bus factor
    if let Some(threat) = check_bus_factor(metadata) {
        risk_score += 10;
        threats.push(threat);
    }

    // Check license risk
    if let Some(threat) = check_license_risk(metadata) {
        risk_score += 15;
        threats.push(threat);
    }

    // Check release velocity
    if let Some(threat) = check_release_velocity(metadata) {
        risk_score += 25;
        threats.push(threat);
    }

    RiskAssessment {
        threats,
        risk_score: risk_score.min(100),
    }
}

/// Check for metadata anomalies
fn check_metadata_anomalies(metadata: &PackageMetadata) -> Option<ThreatIndicator> {
    let mut evidence = Vec::new();

    // Missing repository
    if metadata.repository_url.is_none() {
        evidence.push("No repository URL provided".to_string());
    }

    // Short or missing description
    if let Some(ref desc) = metadata.description {
        if desc.len() < 10 {
            evidence.push("Very short description".to_string());
        }
    } else {
        evidence.push("No description provided".to_string());
    }

    // Missing homepage
    if metadata.homepage_url.is_none() && metadata.repository_url.is_none() {
        evidence.push("No homepage or repository URL".to_string());
    }

    // Few versions for old package
    if metadata.total_versions < 3 && metadata.first_publish_date.is_some() {
        let age = Utc::now() - metadata.first_publish_date.unwrap();
        if age.num_days() > 365 {
            evidence.push("Very few versions for package age".to_string());
        }
    }

    if evidence.len() >= 2 {
        Some(ThreatIndicator {
            package_name: metadata.name.clone(),
            package_version: metadata.version.clone(),
            threat_level: ThreatLevel::Low,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Package metadata anomalies detected".to_string(),
            evidence,
            recommendation: "Verify package authenticity through independent sources".to_string(),
        })
    } else {
        None
    }
}

/// Check for abandonment risk
fn check_abandonment_risk(metadata: &PackageMetadata) -> Option<ThreatIndicator> {
    if let Some(last_publish) = metadata.last_publish_date {
        let age = Utc::now() - last_publish;
        let days = age.num_days();

        if days > 730 {
            // 2 years
            let mut evidence = vec![
                format!("Last update: {} days ago", days),
                "May not receive security updates".to_string(),
            ];

            if days > 1095 {
                // 3 years
                evidence.push("Package appears abandoned".to_string());
            }

            return Some(ThreatIndicator {
                package_name: metadata.name.clone(),
                package_version: metadata.version.clone(),
                threat_level: if days > 1095 {
                    ThreatLevel::Medium
                } else {
                    ThreatLevel::Low
                },
                threat_type: ThreatType::SuspiciousBehavior,
                description: format!("Package not updated in {} days", days),
                evidence,
                recommendation: "Consider finding an actively maintained alternative".to_string(),
            });
        }
    }

    None
}

/// Check bus factor (single maintainer risk)
fn check_bus_factor(metadata: &PackageMetadata) -> Option<ThreatIndicator> {
    let maintainer_count = metadata.maintainers.len();

    if maintainer_count == 1 {
        // Check if it's a high-profile package (by downloads)
        let is_popular = metadata.weekly_downloads.unwrap_or(0) > 10000;

        if is_popular {
            return Some(ThreatIndicator {
                package_name: metadata.name.clone(),
                package_version: metadata.version.clone(),
                threat_level: ThreatLevel::Low,
                threat_type: ThreatType::SuspiciousBehavior,
                description: "Single maintainer for popular package".to_string(),
                evidence: vec![
                    "Only 1 maintainer".to_string(),
                    format!(
                        "Weekly downloads: {}",
                        metadata.weekly_downloads.unwrap_or(0)
                    ),
                    "Bus factor risk: maintainer unavailability could impact security updates"
                        .to_string(),
                ],
                recommendation: "Monitor for maintainer activity and have contingency plans"
                    .to_string(),
            });
        }
    }

    None
}

/// Check license risks
fn check_license_risk(metadata: &PackageMetadata) -> Option<ThreatIndicator> {
    let mut evidence = Vec::new();

    match &metadata.license {
        None => {
            evidence.push("No license specified".to_string());
            evidence.push("May not be legally usable".to_string());
        }
        Some(license) => {
            let license_upper = license.to_uppercase();

            // Strong copyleft licenses
            if license_upper.contains("GPL") && !license_upper.contains("LGPL") {
                evidence.push(format!("Copyleft license: {}", license));
                evidence.push("May require open-sourcing derivative works".to_string());
            }

            if license_upper.contains("AGPL") {
                evidence.push(format!("Network copyleft license: {}", license));
                evidence.push("Network use may require source disclosure".to_string());
            }

            // Unknown or custom licenses
            if license_upper.contains("SEE LICENSE") || license_upper == "UNLICENSED" {
                evidence.push("Non-standard license declaration".to_string());
            }
        }
    }

    if !evidence.is_empty() {
        Some(ThreatIndicator {
            package_name: metadata.name.clone(),
            package_version: metadata.version.clone(),
            threat_level: ThreatLevel::Low,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "License risk detected".to_string(),
            evidence,
            recommendation: "Review license compatibility with your project".to_string(),
        })
    } else {
        None
    }
}

/// Check for suspicious release velocity
fn check_release_velocity(metadata: &PackageMetadata) -> Option<ThreatIndicator> {
    // This would need version history with timestamps
    // For now, check version number anomalies

    // Check for suspicious version jumps
    let version_parts: Vec<&str> = metadata.version.split('.').collect();
    if let Some(major) = version_parts.first() {
        if let Ok(major_num) = major.parse::<u32>() {
            // Suspicious major version (too high)
            if major_num > 50 && metadata.total_versions < 10 {
                return Some(ThreatIndicator {
                    package_name: metadata.name.clone(),
                    package_version: metadata.version.clone(),
                    threat_level: ThreatLevel::Medium,
                    threat_type: ThreatType::SuspiciousBehavior,
                    description: "Suspicious version number".to_string(),
                    evidence: vec![
                        format!(
                            "Version {} with only {} total releases",
                            metadata.version, metadata.total_versions
                        ),
                        "May indicate package takeover or manipulation".to_string(),
                    ],
                    recommendation: "Verify package history and maintainer identity".to_string(),
                });
            }
        }
    }

    None
}

/// Check for binary blobs in package files
pub fn check_binary_blobs(
    package_name: &str,
    package_version: &str,
    file_list: &[String],
) -> Option<ThreatIndicator> {
    let suspicious_extensions = [
        ".exe", ".dll", ".so", ".dylib", ".bin", ".com", ".bat", ".cmd", ".msi", ".app", ".dmg",
        ".deb", ".rpm", ".apk",
    ];

    let mut suspicious_files = Vec::new();

    for file in file_list {
        let file_lower = file.to_lowercase();
        for ext in &suspicious_extensions {
            if file_lower.ends_with(ext) {
                suspicious_files.push(file.clone());
                break;
            }
        }
    }

    if !suspicious_files.is_empty() {
        Some(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: package_version.to_string(),
            threat_level: ThreatLevel::High,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Binary files detected in package".to_string(),
            evidence: suspicious_files
                .iter()
                .map(|f| format!("Binary: {}", f))
                .collect(),
            recommendation: "Binary files in npm/pip packages are suspicious. Review necessity."
                .to_string(),
        })
    } else {
        None
    }
}

/// Check for obfuscation patterns in source code
pub fn check_obfuscation(package_name: &str, source_content: &str) -> Option<ThreatIndicator> {
    let mut evidence = Vec::new();

    // Long lines (minified code)
    let long_lines = source_content.lines().filter(|l| l.len() > 500).count();
    if long_lines > 5 {
        evidence.push(format!("{} very long lines (likely minified)", long_lines));
    }

    // Heavy eval/Function usage
    let eval_count =
        source_content.matches("eval(").count() + source_content.matches("Function(").count();
    if eval_count > 3 {
        evidence.push(format!("{} eval/Function calls", eval_count));
    }

    // Hex/unicode escapes
    if let Ok(re) = Regex::new(r"\\x[0-9a-fA-F]{2}") {
        let hex_count = re.find_iter(source_content).count();
        if hex_count > 20 {
            evidence.push(format!("{} hex-encoded characters", hex_count));
        }
    }

    // Base64 strings
    if let Ok(re) = Regex::new(r"[A-Za-z0-9+/]{50,}={0,2}") {
        let b64_count = re.find_iter(source_content).count();
        if b64_count > 3 {
            evidence.push(format!("{} potential base64 strings", b64_count));
        }
    }

    if evidence.len() >= 2 {
        Some(ThreatIndicator {
            package_name: package_name.to_string(),
            package_version: String::new(),
            threat_level: ThreatLevel::Medium,
            threat_type: ThreatType::SuspiciousBehavior,
            description: "Code obfuscation patterns detected".to_string(),
            evidence,
            recommendation: "Review source code manually or find alternative with readable source"
                .to_string(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata() -> PackageMetadata {
        PackageMetadata {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test package".to_string()),
            repository_url: Some("https://github.com/test/test".to_string()),
            homepage_url: None,
            license: Some("MIT".to_string()),
            maintainers: vec!["test@example.com".to_string()],
            last_publish_date: Some(Utc::now()),
            first_publish_date: Some(Utc::now()),
            total_versions: 10,
            weekly_downloads: Some(1000),
        }
    }

    #[test]
    fn test_healthy_package() {
        let metadata = create_test_metadata();
        let assessment = assess_package_risk(&metadata);
        assert!(assessment.threats.is_empty());
        assert_eq!(assessment.risk_score, 0);
    }

    #[test]
    fn test_abandoned_package() {
        let mut metadata = create_test_metadata();
        metadata.last_publish_date = Some(Utc::now() - chrono::Duration::days(1000));
        let assessment = assess_package_risk(&metadata);
        assert!(!assessment.threats.is_empty());
    }

    #[test]
    fn test_no_license() {
        let mut metadata = create_test_metadata();
        metadata.license = None;
        let assessment = assess_package_risk(&metadata);
        assert!(!assessment.threats.is_empty());
    }

    #[test]
    fn test_binary_blob_detection() {
        let files = vec![
            "index.js".to_string(),
            "lib/helper.exe".to_string(),
            "bin/tool.dll".to_string(),
        ];
        let result = check_binary_blobs("test-pkg", "1.0.0", &files);
        assert!(result.is_some());
        assert_eq!(result.unwrap().threat_level, ThreatLevel::High);
    }

    #[test]
    fn test_obfuscation_detection() {
        let code = r#"
            var a = eval("alert(1)");
            eval("console.log(2)");
            eval("fetch()");
            eval("more()");
            Function("return this")();
            var x = "\x48\x65\x6c\x6c\x6f\x20\x57\x6f\x72\x6c\x64\x21\x22\x23\x24\x25\x26\x27\x28\x29\x30\x31\x32\x33\x34\x35\x36\x37\x38\x39\x40\x41\x42\x43";
        "#;
        let result = check_obfuscation("test-pkg", code);
        assert!(result.is_some());
    }
}
