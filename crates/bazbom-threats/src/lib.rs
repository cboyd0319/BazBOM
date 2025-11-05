//! Threat intelligence for BazBOM
//!
//! This crate provides threat intelligence capabilities:
//! - Malicious package detection
//! - Typosquatting detection
//! - Supply chain attack indicators
//! - Continuous monitoring

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub mod custom_feeds;
pub mod database_integration;
pub mod dependency_confusion;
pub mod maintainer_takeover;
pub mod malicious;
pub mod monitoring;
pub mod notifications;
pub mod scorecard;
pub mod supply_chain;
pub mod typosquatting;

/// Threat level classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatLevel {
    /// Critical threat - immediate action required
    Critical,
    /// High threat - action required soon
    High,
    /// Medium threat - should be reviewed
    Medium,
    /// Low threat - informational
    Low,
    /// No threat detected
    None,
}

/// Threat indicator details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    /// Package identifier
    pub package_name: String,
    /// Package version
    pub package_version: String,
    /// Threat level
    pub threat_level: ThreatLevel,
    /// Threat type
    pub threat_type: ThreatType,
    /// Description of the threat
    pub description: String,
    /// Evidence supporting the threat classification
    pub evidence: Vec<String>,
    /// Recommended action
    pub recommendation: String,
}

/// Types of threats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatType {
    /// Known malicious package
    MaliciousPackage,
    /// Typosquatting attempt
    Typosquatting,
    /// Supply chain attack indicators
    SupplyChainAttack,
    /// Suspicious behavior
    SuspiciousBehavior,
    /// Compromised maintainer account
    CompromisedAccount,
    /// Backdoor detected
    Backdoor,
}

/// Threat analyzer
pub struct ThreatAnalyzer {
    malicious_db: HashSet<String>,
    known_packages: HashSet<String>,
}

impl ThreatAnalyzer {
    /// Create a new threat analyzer
    pub fn new() -> Self {
        Self {
            malicious_db: HashSet::new(),
            known_packages: HashSet::new(),
        }
    }

    /// Load malicious package database
    pub fn load_malicious_db(&mut self, packages: Vec<String>) {
        self.malicious_db = packages.into_iter().collect();
    }

    /// Load known legitimate packages for typosquatting detection
    pub fn load_known_packages(&mut self, packages: Vec<String>) {
        self.known_packages = packages.into_iter().collect();
    }

    /// Analyze a package for threats
    pub fn analyze_package(
        &self,
        package_name: &str,
        package_version: &str,
    ) -> Result<Vec<ThreatIndicator>> {
        let mut threats = Vec::new();

        // Check for malicious packages
        if let Some(threat) = malicious::check_malicious(package_name, &self.malicious_db) {
            threats.push(threat);
        }

        // Check for typosquatting
        if let Some(threat) = typosquatting::check_typosquatting(package_name, &self.known_packages)
        {
            threats.push(threat);
        }

        // Check for supply chain attack indicators
        threats.extend(supply_chain::check_supply_chain_indicators(
            package_name,
            package_version,
        ));

        Ok(threats)
    }

    /// Analyze multiple packages
    pub fn analyze_packages(&self, packages: &[(String, String)]) -> Result<Vec<ThreatIndicator>> {
        let mut all_threats = Vec::new();
        for (name, version) in packages {
            all_threats.extend(self.analyze_package(name, version)?);
        }
        Ok(all_threats)
    }
}

impl Default for ThreatAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threat_analyzer_creation() {
        let analyzer = ThreatAnalyzer::new();
        assert!(analyzer.malicious_db.is_empty());
        assert!(analyzer.known_packages.is_empty());
    }

    #[test]
    fn test_load_malicious_db() {
        let mut analyzer = ThreatAnalyzer::new();
        analyzer.load_malicious_db(vec!["evil-package".to_string()]);
        assert_eq!(analyzer.malicious_db.len(), 1);
    }

    #[test]
    fn test_load_known_packages() {
        let mut analyzer = ThreatAnalyzer::new();
        analyzer.load_known_packages(vec!["react".to_string(), "lodash".to_string()]);
        assert_eq!(analyzer.known_packages.len(), 2);
    }

    #[test]
    fn test_analyze_safe_package() {
        let analyzer = ThreatAnalyzer::new();
        let threats = analyzer.analyze_package("safe-package", "1.0.0").unwrap();
        assert!(threats.is_empty());
    }
}
