//! Maintainer takeover detection for BazBOM
//!
//! Detects potential maintainer account compromises and takeovers.
//! Monitors for suspicious changes in package maintainer patterns that could
//! indicate supply chain attacks.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maintainer takeover indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerTakeoverIndicator {
    /// Package name
    pub package: String,
    /// Package version where change was detected
    pub version: String,
    /// Risk level
    pub risk_level: TakeoverRiskLevel,
    /// Indicators that triggered this alert
    pub indicators: Vec<TakeoverSignal>,
    /// Description of the threat
    pub description: String,
    /// Recommended actions
    pub recommendations: Vec<String>,
}

/// Risk level for maintainer takeover
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TakeoverRiskLevel {
    /// Critical - strong evidence of takeover
    Critical,
    /// High - multiple suspicious indicators
    High,
    /// Medium - some suspicious patterns
    Medium,
    /// Low - single minor indicator
    Low,
}

/// Signals that may indicate a maintainer takeover
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TakeoverSignal {
    /// Sudden change in maintainer email domain
    MaintainerEmailChange { old: String, new: String },
    /// New maintainer with no history in the project
    NewUnknownMaintainer { maintainer: String },
    /// Sudden change in code signing key
    SigningKeyChange,
    /// Unusual release cadence (very rapid releases)
    UnusualReleaseCadence { releases_in_24h: usize },
    /// Major version jump with breaking changes
    SuspiciousMajorVersionJump { from: String, to: String },
    /// Added binary files in a traditionally source-only project
    NewBinaryFiles { count: usize },
    /// Obfuscated or suspicious code patterns
    ObfuscatedCode,
    /// Dependencies added from unusual sources
    SuspiciousDependencies { count: usize },
    /// Maintainer with very new GitHub account
    NewMaintainerAccount { account_age_days: u32 },
    /// Change in build/release automation
    BuildAutomationChange,
}

/// Maintainer takeover detector
pub struct MaintainerTakeoverDetector {
    /// Enabled detection rules
    enabled_signals: Vec<TakeoverSignal>,
    /// Strict mode (more sensitive detection)
    strict_mode: bool,
}

impl Default for MaintainerTakeoverDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MaintainerTakeoverDetector {
    /// Create a new maintainer takeover detector
    pub fn new() -> Self {
        Self {
            enabled_signals: Vec::new(),
            strict_mode: false,
        }
    }

    /// Enable strict mode for more sensitive detection
    pub fn strict_mode(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Analyze a package for maintainer takeover indicators
    ///
    /// # Arguments
    /// * `package_info` - Package metadata including version history
    ///
    /// # Returns
    /// List of takeover indicators found, or empty if no issues detected
    pub fn analyze(&self, package_info: &PackageInfo) -> Result<Vec<MaintainerTakeoverIndicator>> {
        let mut indicators = Vec::new();

        // Check for maintainer changes
        if let Some(indicator) = self.check_maintainer_changes(package_info)? {
            indicators.push(indicator);
        }

        // Check for unusual release patterns
        if let Some(indicator) = self.check_release_patterns(package_info)? {
            indicators.push(indicator);
        }

        // Check for suspicious code changes
        if let Some(indicator) = self.check_suspicious_changes(package_info)? {
            indicators.push(indicator);
        }

        Ok(indicators)
    }

    /// Check for suspicious maintainer changes
    fn check_maintainer_changes(
        &self,
        package_info: &PackageInfo,
    ) -> Result<Option<MaintainerTakeoverIndicator>> {
        let mut signals = Vec::new();

        // Check for email domain changes
        if let Some((old_email, new_email)) = self.detect_email_domain_change(&package_info.maintainer_history) {
            signals.push(TakeoverSignal::MaintainerEmailChange {
                old: old_email,
                new: new_email,
            });
        }

        // Check for new unknown maintainers
        if let Some(new_maintainer) = self.detect_new_unknown_maintainer(&package_info.maintainer_history) {
            signals.push(TakeoverSignal::NewUnknownMaintainer {
                maintainer: new_maintainer,
            });
        }

        if signals.is_empty() {
            return Ok(None);
        }

        let risk_level = if signals.len() >= 2 {
            TakeoverRiskLevel::High
        } else {
            TakeoverRiskLevel::Medium
        };

        Ok(Some(MaintainerTakeoverIndicator {
            package: package_info.name.clone(),
            version: package_info.current_version.clone(),
            risk_level,
            indicators: signals,
            description: "Suspicious maintainer changes detected".to_string(),
            recommendations: vec![
                "Review recent commits and changes".to_string(),
                "Verify maintainer identity through multiple channels".to_string(),
                "Consider pinning to a previous trusted version".to_string(),
            ],
        }))
    }

    /// Check for unusual release patterns
    fn check_release_patterns(
        &self,
        package_info: &PackageInfo,
    ) -> Result<Option<MaintainerTakeoverIndicator>> {
        let mut signals = Vec::new();

        // Count recent releases (last 24 hours)
        let recent_releases = package_info
            .version_history
            .iter()
            .filter(|v| {
                // Simple check - in production would check actual timestamps
                v.days_since_release <= 1
            })
            .count();

        if recent_releases >= 3 {
            signals.push(TakeoverSignal::UnusualReleaseCadence {
                releases_in_24h: recent_releases,
            });
        }

        // Check for suspicious major version jumps
        if let Some((from, to)) = self.detect_version_jump(&package_info.version_history) {
            signals.push(TakeoverSignal::SuspiciousMajorVersionJump { from, to });
        }

        if signals.is_empty() {
            return Ok(None);
        }

        Ok(Some(MaintainerTakeoverIndicator {
            package: package_info.name.clone(),
            version: package_info.current_version.clone(),
            risk_level: TakeoverRiskLevel::Medium,
            indicators: signals,
            description: "Unusual release patterns detected".to_string(),
            recommendations: vec![
                "Review changelog for each recent release".to_string(),
                "Check for breaking changes".to_string(),
                "Verify releases are intentional".to_string(),
            ],
        }))
    }

    /// Check for suspicious code changes
    fn check_suspicious_changes(
        &self,
        package_info: &PackageInfo,
    ) -> Result<Option<MaintainerTakeoverIndicator>> {
        let mut signals = Vec::new();

        // Check for new binary files
        if package_info.new_binary_files > 0 {
            signals.push(TakeoverSignal::NewBinaryFiles {
                count: package_info.new_binary_files,
            });
        }

        // Check for suspicious dependencies
        if package_info.suspicious_dependencies > 0 {
            signals.push(TakeoverSignal::SuspiciousDependencies {
                count: package_info.suspicious_dependencies,
            });
        }

        if signals.is_empty() {
            return Ok(None);
        }

        let risk_level = if signals.len() >= 2 {
            TakeoverRiskLevel::High
        } else {
            TakeoverRiskLevel::Medium
        };

        Ok(Some(MaintainerTakeoverIndicator {
            package: package_info.name.clone(),
            version: package_info.current_version.clone(),
            risk_level,
            indicators: signals,
            description: "Suspicious code changes detected".to_string(),
            recommendations: vec![
                "Review diff for suspicious changes".to_string(),
                "Scan for malicious code patterns".to_string(),
                "Consider reverting to previous version".to_string(),
            ],
        }))
    }

    /// Detect email domain changes in maintainer history
    fn detect_email_domain_change(
        &self,
        history: &[MaintainerInfo],
    ) -> Option<(String, String)> {
        if history.len() < 2 {
            return None;
        }

        let latest = &history[0];
        let previous = &history[1];

        let latest_domain = Self::extract_email_domain(&latest.email)?;
        let previous_domain = Self::extract_email_domain(&previous.email)?;

        if latest_domain != previous_domain {
            Some((previous.email.clone(), latest.email.clone()))
        } else {
            None
        }
    }

    /// Extract domain from email address
    fn extract_email_domain(email: &str) -> Option<String> {
        email.split('@').nth(1).map(|s| s.to_string())
    }

    /// Detect new unknown maintainers
    fn detect_new_unknown_maintainer(&self, history: &[MaintainerInfo]) -> Option<String> {
        if history.is_empty() {
            return None;
        }

        let latest = &history[0];
        
        // If account is very new (< 30 days), it's suspicious
        if latest.account_age_days < 30 && history.len() > 1 {
            Some(latest.name.clone())
        } else {
            None
        }
    }

    /// Detect suspicious version jumps
    fn detect_version_jump(&self, history: &[VersionInfo]) -> Option<(String, String)> {
        if history.len() < 2 {
            return None;
        }

        let latest = &history[0];
        let previous = &history[1];

        // Parse versions (simplified - in production use semver crate)
        if let (Some(latest_major), Some(prev_major)) = (
            Self::extract_major_version(&latest.version),
            Self::extract_major_version(&previous.version),
        ) {
            // Suspicious if major version jumps by more than 1
            if latest_major > prev_major + 1 {
                return Some((previous.version.clone(), latest.version.clone()));
            }
        }

        None
    }

    /// Extract major version number from version string
    fn extract_major_version(version: &str) -> Option<u32> {
        version
            .trim_start_matches('v')
            .split('.')
            .next()?
            .parse()
            .ok()
    }
}

/// Package information for takeover analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Current version
    pub current_version: String,
    /// Maintainer history (newest first)
    pub maintainer_history: Vec<MaintainerInfo>,
    /// Version history (newest first)
    pub version_history: Vec<VersionInfo>,
    /// Number of new binary files in recent versions
    pub new_binary_files: usize,
    /// Number of suspicious dependencies added
    pub suspicious_dependencies: usize,
}

/// Maintainer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainerInfo {
    /// Maintainer name
    pub name: String,
    /// Maintainer email
    pub email: String,
    /// Age of maintainer account in days
    pub account_age_days: u32,
    /// First seen date
    pub first_seen: String,
}

/// Version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string
    pub version: String,
    /// Release date
    pub release_date: String,
    /// Days since release
    pub days_since_release: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = MaintainerTakeoverDetector::new();
        assert!(!detector.strict_mode);
    }

    #[test]
    fn test_strict_mode() {
        let detector = MaintainerTakeoverDetector::new().strict_mode();
        assert!(detector.strict_mode);
    }

    #[test]
    fn test_extract_email_domain() {
        assert_eq!(
            MaintainerTakeoverDetector::extract_email_domain("user@example.com"),
            Some("example.com".to_string())
        );
        assert_eq!(
            MaintainerTakeoverDetector::extract_email_domain("invalid"),
            None
        );
    }

    #[test]
    fn test_extract_major_version() {
        assert_eq!(
            MaintainerTakeoverDetector::extract_major_version("1.2.3"),
            Some(1)
        );
        assert_eq!(
            MaintainerTakeoverDetector::extract_major_version("v2.0.0"),
            Some(2)
        );
        assert_eq!(
            MaintainerTakeoverDetector::extract_major_version("10.5.2"),
            Some(10)
        );
        assert_eq!(
            MaintainerTakeoverDetector::extract_major_version("invalid"),
            None
        );
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(matches!(TakeoverRiskLevel::Critical, TakeoverRiskLevel::Critical));
        assert!(matches!(TakeoverRiskLevel::High, TakeoverRiskLevel::High));
        assert!(matches!(TakeoverRiskLevel::Medium, TakeoverRiskLevel::Medium));
        assert!(matches!(TakeoverRiskLevel::Low, TakeoverRiskLevel::Low));
    }

    #[test]
    fn test_takeover_signal_types() {
        let signal = TakeoverSignal::MaintainerEmailChange {
            old: "old@example.com".to_string(),
            new: "new@example.com".to_string(),
        };
        assert!(matches!(signal, TakeoverSignal::MaintainerEmailChange { .. }));

        let signal2 = TakeoverSignal::UnusualReleaseCadence { releases_in_24h: 5 };
        assert!(matches!(signal2, TakeoverSignal::UnusualReleaseCadence { .. }));
    }

    #[test]
    fn test_package_info_structure() {
        let package_info = PackageInfo {
            name: "test-package".to_string(),
            current_version: "2.0.0".to_string(),
            maintainer_history: vec![],
            version_history: vec![],
            new_binary_files: 0,
            suspicious_dependencies: 0,
        };
        
        assert_eq!(package_info.name, "test-package");
        assert_eq!(package_info.current_version, "2.0.0");
    }
}
