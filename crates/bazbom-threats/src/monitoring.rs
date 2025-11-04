//! Continuous monitoring capabilities
//!
//! Monitors packages for new threats over time

use crate::{ThreatIndicator, ThreatAnalyzer};
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Interval between checks (in seconds)
    pub check_interval: u64,
    /// Packages to monitor
    pub watched_packages: Vec<WatchedPackage>,
    /// Alert thresholds
    pub alert_threshold: crate::ThreatLevel,
}

/// Package being monitored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchedPackage {
    pub name: String,
    pub version: String,
    pub last_checked: DateTime<Utc>,
}

/// Monitoring result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringResult {
    pub timestamp: DateTime<Utc>,
    pub new_threats: Vec<ThreatIndicator>,
    pub resolved_threats: Vec<String>,
}

/// Continuous monitoring service
pub struct MonitoringService {
    config: MonitoringConfig,
    analyzer: ThreatAnalyzer,
    previous_threats: HashMap<String, Vec<ThreatIndicator>>,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(config: MonitoringConfig, analyzer: ThreatAnalyzer) -> Self {
        Self {
            config,
            analyzer,
            previous_threats: HashMap::new(),
        }
    }

    /// Perform a monitoring check
    pub async fn check(&mut self) -> Result<MonitoringResult> {
        let mut new_threats = Vec::new();
        let mut resolved_threats = Vec::new();

        for package in &self.config.watched_packages {
            let current_threats = self.analyzer.analyze_package(&package.name, &package.version)?;

            // Check for new threats
            let previous = self.previous_threats.get(&package.name);
            for threat in &current_threats {
                if let Some(prev) = previous {
                    if !prev.iter().any(|t| t.description == threat.description) {
                        new_threats.push(threat.clone());
                    }
                } else {
                    new_threats.push(threat.clone());
                }
            }

            // Check for resolved threats
            if let Some(prev) = previous {
                for old_threat in prev {
                    if !current_threats.iter().any(|t| t.description == old_threat.description) {
                        resolved_threats.push(old_threat.description.clone());
                    }
                }
            }

            // Update stored threats
            self.previous_threats.insert(package.name.clone(), current_threats);
        }

        Ok(MonitoringResult {
            timestamp: Utc::now(),
            new_threats,
            resolved_threats,
        })
    }

    /// Start continuous monitoring (blocking)
    pub async fn start_monitoring(&mut self) -> Result<()> {
        loop {
            let result = self.check().await?;

            // Alert on new threats above threshold
            for threat in result.new_threats {
                if threat.threat_level as u8 >= self.config.alert_threshold as u8 {
                    println!("ALERT: New threat detected: {}", threat.description);
                }
            }

            // Wait for next check
            tokio::time::sleep(tokio::time::Duration::from_secs(self.config.check_interval)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> MonitoringConfig {
        MonitoringConfig {
            check_interval: 60,
            watched_packages: vec![
                WatchedPackage {
                    name: "test-package".to_string(),
                    version: "1.0.0".to_string(),
                    last_checked: Utc::now(),
                },
            ],
            alert_threshold: crate::ThreatLevel::High,
        }
    }

    #[test]
    fn test_monitoring_config_creation() {
        let config = create_test_config();
        assert_eq!(config.check_interval, 60);
        assert_eq!(config.watched_packages.len(), 1);
    }

    #[test]
    fn test_monitoring_service_creation() {
        let config = create_test_config();
        let analyzer = ThreatAnalyzer::new();
        let service = MonitoringService::new(config, analyzer);
        assert!(service.previous_threats.is_empty());
    }

    #[tokio::test]
    async fn test_monitoring_check() {
        let config = create_test_config();
        let analyzer = ThreatAnalyzer::new();
        let mut service = MonitoringService::new(config, analyzer);

        let result = service.check().await.unwrap();
        // Note: may have threats due to suspicious name pattern "test-package"
        // Just verify the check completes successfully
        assert!(result.new_threats.len() <= 1);
        assert!(result.resolved_threats.is_empty());
    }
}
