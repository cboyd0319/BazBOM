//! Feature extraction for machine learning
//!
//! This module converts vulnerabilities and dependencies into numerical features
//! for ML algorithms.

use serde::{Deserialize, Serialize};

/// Features extracted from a vulnerability for ML analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityFeatures {
    /// CVSS base score (0.0-10.0)
    pub cvss_score: f64,

    /// Age in days since publication
    pub age_days: u32,

    /// Whether exploit code is publicly available
    pub has_exploit: bool,

    /// EPSS score (0.0-1.0, probability of exploitation)
    pub epss: f64,

    /// Whether vulnerability is in CISA KEV (Known Exploited Vulnerabilities)
    pub in_kev: bool,

    /// Severity category (encoded: 0=LOW, 1=MEDIUM, 2=HIGH, 3=CRITICAL)
    pub severity_level: u8,

    /// Vulnerability type (encoded: 0=OTHER, 1=RCE, 2=XSS, 3=SQLi, 4=CSRF, etc.)
    pub vuln_type: u8,

    /// Whether the vulnerable code is reachable in the application
    pub is_reachable: bool,
}

impl VulnerabilityFeatures {
    /// Convert features to a vector for ML models
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.cvss_score,
            self.age_days as f64,
            if self.has_exploit { 1.0 } else { 0.0 },
            self.epss,
            if self.in_kev { 1.0 } else { 0.0 },
            self.severity_level as f64,
            self.vuln_type as f64,
            if self.is_reachable { 1.0 } else { 0.0 },
        ]
    }
}

impl Default for VulnerabilityFeatures {
    fn default() -> Self {
        Self {
            cvss_score: 0.0,
            age_days: 0,
            has_exploit: false,
            epss: 0.0,
            in_kev: false,
            severity_level: 0,
            vuln_type: 0,
            is_reachable: false,
        }
    }
}

/// Features extracted from a dependency for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyFeatures {
    /// Number of transitive dependencies
    pub transitive_count: u32,

    /// Average age of all dependencies in days
    pub avg_age_days: f64,

    /// Number of known vulnerabilities
    pub vuln_count: u32,

    /// Number of dependencies from the same group/org
    pub same_group_count: u32,

    /// Whether this is a direct dependency
    pub is_direct: bool,

    /// Package popularity score (downloads per day, normalized 0-1)
    pub popularity: f64,

    /// Maintainer reputation score (0-1, based on history)
    pub maintainer_score: f64,

    /// Number of releases in the last year
    pub recent_releases: u32,
}

impl DependencyFeatures {
    /// Convert features to a vector for ML models
    pub fn to_vector(&self) -> Vec<f64> {
        vec![
            self.transitive_count as f64,
            self.avg_age_days,
            self.vuln_count as f64,
            self.same_group_count as f64,
            if self.is_direct { 1.0 } else { 0.0 },
            self.popularity,
            self.maintainer_score,
            self.recent_releases as f64,
        ]
    }
}

impl Default for DependencyFeatures {
    fn default() -> Self {
        Self {
            transitive_count: 0,
            avg_age_days: 0.0,
            vuln_count: 0,
            same_group_count: 0,
            is_direct: false,
            popularity: 0.5,
            maintainer_score: 0.5,
            recent_releases: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_features_to_vector() {
        let features = VulnerabilityFeatures {
            cvss_score: 9.8,
            age_days: 365,
            has_exploit: true,
            epss: 0.85,
            in_kev: true,
            severity_level: 3, // CRITICAL
            vuln_type: 1,      // RCE
            is_reachable: true,
        };

        let vector = features.to_vector();
        assert_eq!(vector.len(), 8);
        assert_eq!(vector[0], 9.8);
        assert_eq!(vector[1], 365.0);
        assert_eq!(vector[2], 1.0); // has_exploit
        assert_eq!(vector[3], 0.85);
        assert_eq!(vector[4], 1.0); // in_kev
        assert_eq!(vector[5], 3.0);
        assert_eq!(vector[6], 1.0);
        assert_eq!(vector[7], 1.0); // is_reachable
    }

    #[test]
    fn test_dependency_features_to_vector() {
        let features = DependencyFeatures {
            transitive_count: 42,
            avg_age_days: 730.0,
            vuln_count: 3,
            same_group_count: 5,
            is_direct: true,
            popularity: 0.9,
            maintainer_score: 0.8,
            recent_releases: 12,
        };

        let vector = features.to_vector();
        assert_eq!(vector.len(), 8);
        assert_eq!(vector[0], 42.0);
        assert_eq!(vector[1], 730.0);
        assert_eq!(vector[2], 3.0);
        assert_eq!(vector[3], 5.0);
        assert_eq!(vector[4], 1.0); // is_direct
        assert_eq!(vector[5], 0.9);
        assert_eq!(vector[6], 0.8);
        assert_eq!(vector[7], 12.0);
    }

    #[test]
    fn test_default_features() {
        let vuln_features = VulnerabilityFeatures::default();
        assert_eq!(vuln_features.cvss_score, 0.0);
        assert_eq!(vuln_features.age_days, 0);
        assert!(!vuln_features.has_exploit);

        let dep_features = DependencyFeatures::default();
        assert_eq!(dep_features.transitive_count, 0);
        assert_eq!(dep_features.popularity, 0.5);
    }
}
