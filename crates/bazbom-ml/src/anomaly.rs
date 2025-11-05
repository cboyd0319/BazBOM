//! Anomaly detection for unusual dependency patterns
//!
//! This module implements statistical anomaly detection to identify:
//! - Dependencies with unusual characteristics
//! - Sudden changes in dependency patterns
//! - Supply chain attack indicators

use crate::features::DependencyFeatures;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Type of anomaly detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// Unusual number of transitive dependencies
    UnusualTransitiveCount,

    /// Abnormally high vulnerability count
    HighVulnerabilityCount,

    /// Low maintainer reputation score
    LowMaintainerScore,

    /// Unusual release pattern (too many or too few)
    UnusualReleasePattern,

    /// Low popularity for a dependency (potential typosquat)
    LowPopularity,

    /// Multiple anomaly signals detected
    MultipleSignals,
}

/// An detected anomaly in dependency patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Type of anomaly
    pub anomaly_type: AnomalyType,

    /// Anomaly score (higher = more anomalous, typically 0.0-1.0)
    pub score: f64,

    /// Human-readable description
    pub description: String,

    /// Package name (if applicable)
    pub package: Option<String>,

    /// Recommendation for action
    pub recommendation: String,
}

/// Anomaly detector using statistical methods
pub struct AnomalyDetector {
    /// Statistical thresholds learned from historical data
    thresholds: AnomalyThresholds,
}

#[derive(Debug, Clone)]
struct AnomalyThresholds {
    /// Mean + 2*stddev for transitive count
    max_transitive_count: f64,

    /// Mean + 2*stddev for vulnerability count
    max_vuln_count: f64,

    /// Minimum acceptable maintainer score
    min_maintainer_score: f64,

    /// Mean ± 2*stddev for release count
    min_release_count: f64,
    max_release_count: f64,

    /// Minimum popularity threshold
    min_popularity: f64,
}

impl Default for AnomalyThresholds {
    fn default() -> Self {
        Self {
            max_transitive_count: 100.0, // Most deps have <100 transitives
            max_vuln_count: 5.0,         // >5 vulns is concerning
            min_maintainer_score: 0.3,   // Low reputation threshold
            min_release_count: 2.0,      // At least 2 releases/year expected
            max_release_count: 52.0,     // >1 release/week is unusual
            min_popularity: 0.1,         // Very unpopular packages suspicious
        }
    }
}

impl AnomalyDetector {
    /// Create a new anomaly detector with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: AnomalyThresholds::default(),
        }
    }

    /// Create a detector with custom thresholds trained on historical data
    pub fn with_thresholds(
        max_transitive_count: f64,
        max_vuln_count: f64,
        min_maintainer_score: f64,
        min_release_count: f64,
        max_release_count: f64,
        min_popularity: f64,
    ) -> Self {
        Self {
            thresholds: AnomalyThresholds {
                max_transitive_count,
                max_vuln_count,
                min_maintainer_score,
                min_release_count,
                max_release_count,
                min_popularity,
            },
        }
    }

    /// Train detector on historical dependency features
    ///
    /// This calculates statistical thresholds (mean ± 2*stddev) from historical data.
    pub fn train(historical_features: &[DependencyFeatures]) -> Result<Self> {
        if historical_features.is_empty() {
            return Ok(Self::new());
        }

        // Calculate statistics for each feature
        let transitive_counts: Vec<f64> = historical_features
            .iter()
            .map(|f| f.transitive_count as f64)
            .collect();

        let vuln_counts: Vec<f64> = historical_features
            .iter()
            .map(|f| f.vuln_count as f64)
            .collect();

        let release_counts: Vec<f64> = historical_features
            .iter()
            .map(|f| f.recent_releases as f64)
            .collect();

        let thresholds = AnomalyThresholds {
            max_transitive_count: mean(&transitive_counts) + 2.0 * stddev(&transitive_counts),
            max_vuln_count: mean(&vuln_counts) + 2.0 * stddev(&vuln_counts),
            min_maintainer_score: 0.3, // Fixed threshold
            min_release_count: mean(&release_counts) - 2.0 * stddev(&release_counts),
            max_release_count: mean(&release_counts) + 2.0 * stddev(&release_counts),
            min_popularity: 0.1, // Fixed threshold
        };

        Ok(Self { thresholds })
    }

    /// Detect anomalies in a single dependency
    pub fn detect(&self, features: &DependencyFeatures, package_name: &str) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        let mut signal_count = 0;

        // Check transitive count
        if features.transitive_count as f64 > self.thresholds.max_transitive_count {
            signal_count += 1;
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::UnusualTransitiveCount,
                score: (features.transitive_count as f64 / self.thresholds.max_transitive_count)
                    .min(1.0),
                description: format!(
                    "{} has {} transitive dependencies (expected <{:.0})",
                    package_name, features.transitive_count, self.thresholds.max_transitive_count
                ),
                package: Some(package_name.to_string()),
                recommendation: "Review transitive dependencies for unexpected additions"
                    .to_string(),
            });
        }

        // Check vulnerability count
        if features.vuln_count as f64 > self.thresholds.max_vuln_count {
            signal_count += 1;
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::HighVulnerabilityCount,
                score: (features.vuln_count as f64 / (self.thresholds.max_vuln_count * 2.0))
                    .min(1.0),
                description: format!(
                    "{} has {} known vulnerabilities (expected <{:.0})",
                    package_name, features.vuln_count, self.thresholds.max_vuln_count
                ),
                package: Some(package_name.to_string()),
                recommendation: "Prioritize upgrading this dependency or finding alternatives"
                    .to_string(),
            });
        }

        // Check maintainer score
        if features.maintainer_score < self.thresholds.min_maintainer_score {
            signal_count += 1;
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::LowMaintainerScore,
                score: 1.0 - features.maintainer_score,
                description: format!(
                    "{} has low maintainer reputation score ({:.2})",
                    package_name, features.maintainer_score
                ),
                package: Some(package_name.to_string()),
                recommendation: "Verify maintainer identity and project legitimacy".to_string(),
            });
        }

        // Check release pattern
        let releases = features.recent_releases as f64;
        if releases < self.thresholds.min_release_count
            || releases > self.thresholds.max_release_count
        {
            signal_count += 1;
            let reason = if releases < self.thresholds.min_release_count {
                "too few releases (possible abandonment)"
            } else {
                "unusually high release frequency (possible instability)"
            };

            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::UnusualReleasePattern,
                score: if releases < self.thresholds.min_release_count {
                    (self.thresholds.min_release_count - releases)
                        / self.thresholds.min_release_count
                } else {
                    (releases - self.thresholds.max_release_count) / releases
                }
                .min(1.0),
                description: format!(
                    "{} has {} releases in the last year ({})",
                    package_name, features.recent_releases, reason
                ),
                package: Some(package_name.to_string()),
                recommendation: "Review project activity and maintenance status".to_string(),
            });
        }

        // Check popularity (low popularity might indicate typosquatting)
        if features.popularity < self.thresholds.min_popularity {
            signal_count += 1;
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::LowPopularity,
                score: 1.0 - features.popularity,
                description: format!(
                    "{} has very low popularity (score: {:.2})",
                    package_name, features.popularity
                ),
                package: Some(package_name.to_string()),
                recommendation: "Verify package name spelling and check for typosquatting"
                    .to_string(),
            });
        }

        // If multiple signals, add a summary anomaly
        if signal_count >= 2 {
            anomalies.push(Anomaly {
                anomaly_type: AnomalyType::MultipleSignals,
                score: (signal_count as f64 / 5.0).min(1.0), // 5 possible signals
                description: format!(
                    "{} triggered {} anomaly signals",
                    package_name, signal_count
                ),
                package: Some(package_name.to_string()),
                recommendation:
                    "High priority review recommended - multiple risk indicators detected"
                        .to_string(),
            });
        }

        anomalies
    }

    /// Detect anomalies across multiple dependencies
    pub fn detect_batch(&self, features_map: &[(String, DependencyFeatures)]) -> Vec<Anomaly> {
        features_map
            .iter()
            .flat_map(|(name, features)| self.detect(features, name))
            .collect()
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Calculate mean of a vector of f64 values
fn mean(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f64>() / values.len() as f64
}

/// Calculate standard deviation of a vector of f64 values
fn stddev(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean_val = mean(values);
    let variance =
        values.iter().map(|v| (v - mean_val).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anomaly_detector_creation() {
        let detector = AnomalyDetector::new();
        assert!(detector.thresholds.max_transitive_count > 0.0);
    }

    #[test]
    fn test_detect_high_transitive_count() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            transitive_count: 200, // Above threshold
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "suspicious-package");
        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::UnusualTransitiveCount));
    }

    #[test]
    fn test_detect_high_vulnerability_count() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            vuln_count: 10, // Above threshold
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "vulnerable-package");
        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::HighVulnerabilityCount));
    }

    #[test]
    fn test_detect_low_maintainer_score() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            maintainer_score: 0.1, // Below threshold
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "untrusted-package");
        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::LowMaintainerScore));
    }

    #[test]
    fn test_detect_low_popularity() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            popularity: 0.05, // Below threshold
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "unknown-package");
        assert!(!anomalies.is_empty());
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::LowPopularity));
    }

    #[test]
    fn test_detect_multiple_signals() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            transitive_count: 200, // Signal 1
            vuln_count: 10,        // Signal 2
            maintainer_score: 0.1, // Signal 3
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "risky-package");
        // Should have anomalies for each signal + multiple signals summary
        assert!(anomalies.len() >= 4);
        assert!(anomalies
            .iter()
            .any(|a| a.anomaly_type == AnomalyType::MultipleSignals));
    }

    #[test]
    fn test_detect_clean_dependency() {
        let detector = AnomalyDetector::new();
        let features = DependencyFeatures {
            transitive_count: 10,
            vuln_count: 0,
            maintainer_score: 0.9,
            popularity: 0.8,
            recent_releases: 4,
            ..DependencyFeatures::default()
        };

        let anomalies = detector.detect(&features, "clean-package");
        assert!(anomalies.is_empty());
    }

    #[test]
    fn test_train_from_historical_data() {
        let historical_features = vec![
            DependencyFeatures {
                transitive_count: 10,
                vuln_count: 1,
                recent_releases: 4,
                ..DependencyFeatures::default()
            },
            DependencyFeatures {
                transitive_count: 20,
                vuln_count: 0,
                recent_releases: 6,
                ..DependencyFeatures::default()
            },
            DependencyFeatures {
                transitive_count: 15,
                vuln_count: 2,
                recent_releases: 5,
                ..DependencyFeatures::default()
            },
        ];

        let detector = AnomalyDetector::train(&historical_features).unwrap();
        // Trained detector should have reasonable thresholds
        assert!(detector.thresholds.max_transitive_count > 15.0);
    }

    #[test]
    fn test_detect_batch() {
        let detector = AnomalyDetector::new();
        let features_map = vec![
            (
                "package-a".to_string(),
                DependencyFeatures {
                    transitive_count: 200,
                    ..DependencyFeatures::default()
                },
            ),
            (
                "package-b".to_string(),
                DependencyFeatures {
                    vuln_count: 10,
                    ..DependencyFeatures::default()
                },
            ),
            ("package-c".to_string(), DependencyFeatures::default()), // Clean
        ];

        let anomalies = detector.detect_batch(&features_map);
        // Should find anomalies for package-a and package-b
        assert!(anomalies.len() >= 2);
    }

    #[test]
    fn test_mean_calculation() {
        assert_eq!(mean(&[1.0, 2.0, 3.0, 4.0, 5.0]), 3.0);
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn test_stddev_calculation() {
        let values = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let sd = stddev(&values);
        assert!(sd > 2.0 && sd < 3.0); // Approx 2.138
    }
}
