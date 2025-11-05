//! Enhanced risk scoring for vulnerabilities
//!
//! This module provides ML-enhanced risk scoring that goes beyond simple CVSS scores
//! by considering multiple factors including EPSS, KEV status, reachability, and more.

use crate::features::VulnerabilityFeatures;
use serde::{Deserialize, Serialize};

/// Enhanced risk score with breakdown by factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedRiskScore {
    /// Overall risk score (0.0-1.0, higher = more risky)
    pub overall_score: f64,
    
    /// Risk level category
    pub risk_level: RiskLevel,
    
    /// Breakdown of score components
    pub components: RiskComponents,
    
    /// Explanation of the score
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Critical, // 0.8-1.0
    High,     // 0.6-0.8
    Medium,   // 0.4-0.6
    Low,      // 0.2-0.4
    Minimal,  // 0.0-0.2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    /// CVSS contribution (0.0-1.0)
    pub cvss_component: f64,
    
    /// EPSS contribution (0.0-1.0)
    pub epss_component: f64,
    
    /// KEV contribution (0.0-1.0)
    pub kev_component: f64,
    
    /// Reachability contribution (0.0-1.0)
    pub reachability_component: f64,
    
    /// Age/freshness contribution (0.0-1.0)
    pub age_component: f64,
    
    /// Exploit availability contribution (0.0-1.0)
    pub exploit_component: f64,
}

/// Risk scorer that combines multiple signals
pub struct RiskScorer {
    /// Weights for each component (sum to 1.0)
    weights: RiskWeights,
}

#[derive(Debug, Clone)]
struct RiskWeights {
    cvss_weight: f64,
    epss_weight: f64,
    kev_weight: f64,
    reachability_weight: f64,
    age_weight: f64,
    exploit_weight: f64,
}

impl Default for RiskWeights {
    fn default() -> Self {
        Self {
            cvss_weight: 0.25,        // 25% CVSS score
            epss_weight: 0.20,        // 20% EPSS
            kev_weight: 0.20,         // 20% KEV status
            reachability_weight: 0.20, // 20% Reachability
            age_weight: 0.05,         // 5% Age
            exploit_weight: 0.10,     // 10% Exploit availability
        }
    }
}

impl RiskScorer {
    /// Create a new risk scorer with default weights
    pub fn new() -> Self {
        Self {
            weights: RiskWeights::default(),
        }
    }
    
    /// Create a scorer with custom weights
    ///
    /// # Arguments
    ///
    /// * `cvss_weight` - Weight for CVSS score (0.0-1.0)
    /// * `epss_weight` - Weight for EPSS score (0.0-1.0)
    /// * `kev_weight` - Weight for KEV status (0.0-1.0)
    /// * `reachability_weight` - Weight for reachability (0.0-1.0)
    /// * `age_weight` - Weight for vulnerability age (0.0-1.0)
    /// * `exploit_weight` - Weight for exploit availability (0.0-1.0)
    ///
    /// Note: Weights should sum to 1.0 for meaningful scores
    pub fn with_weights(
        cvss_weight: f64,
        epss_weight: f64,
        kev_weight: f64,
        reachability_weight: f64,
        age_weight: f64,
        exploit_weight: f64,
    ) -> Self {
        Self {
            weights: RiskWeights {
                cvss_weight,
                epss_weight,
                kev_weight,
                reachability_weight,
                age_weight,
                exploit_weight,
            },
        }
    }
    
    /// Calculate enhanced risk score from vulnerability features
    pub fn score(&self, features: &VulnerabilityFeatures) -> EnhancedRiskScore {
        // Normalize CVSS to 0.0-1.0
        let cvss_normalized = features.cvss_score / 10.0;
        
        // EPSS is already 0.0-1.0
        let epss_normalized = features.epss;
        
        // KEV is binary: present = 1.0, absent = 0.0
        let kev_normalized = if features.in_kev { 1.0 } else { 0.0 };
        
        // Reachability is binary: reachable = 1.0, unreachable = 0.0
        let reachability_normalized = if features.is_reachable { 1.0 } else { 0.0 };
        
        // Age: newer vulnerabilities are riskier (more likely to be actively exploited)
        // 0 days = 1.0, 365+ days = 0.0
        let age_normalized = 1.0 - (features.age_days as f64 / 365.0).min(1.0);
        
        // Exploit availability is binary
        let exploit_normalized = if features.has_exploit { 1.0 } else { 0.0 };
        
        // Calculate weighted components
        let components = RiskComponents {
            cvss_component: cvss_normalized * self.weights.cvss_weight,
            epss_component: epss_normalized * self.weights.epss_weight,
            kev_component: kev_normalized * self.weights.kev_weight,
            reachability_component: reachability_normalized * self.weights.reachability_weight,
            age_component: age_normalized * self.weights.age_weight,
            exploit_component: exploit_normalized * self.weights.exploit_weight,
        };
        
        // Overall score is the sum of all components
        let overall_score = components.cvss_component
            + components.epss_component
            + components.kev_component
            + components.reachability_component
            + components.age_component
            + components.exploit_component;
        
        // Determine risk level
        let risk_level = if overall_score >= 0.8 {
            RiskLevel::Critical
        } else if overall_score >= 0.6 {
            RiskLevel::High
        } else if overall_score >= 0.4 {
            RiskLevel::Medium
        } else if overall_score >= 0.2 {
            RiskLevel::Low
        } else {
            RiskLevel::Minimal
        };
        
        // Generate explanation
        let explanation = self.generate_explanation(features, &components, overall_score);
        
        EnhancedRiskScore {
            overall_score,
            risk_level,
            components,
            explanation,
        }
    }
    
    fn generate_explanation(&self, features: &VulnerabilityFeatures, components: &RiskComponents, score: f64) -> String {
        let mut parts = Vec::new();
        
        parts.push(format!("Risk score: {:.2}/1.00", score));
        
        if components.cvss_component > 0.15 {
            parts.push(format!("High CVSS score ({:.1})", features.cvss_score));
        }
        
        if components.epss_component > 0.15 {
            parts.push(format!("High exploit probability (EPSS: {:.1}%)", features.epss * 100.0));
        }
        
        if components.kev_component > 0.0 {
            parts.push("In CISA KEV (actively exploited)".to_string());
        }
        
        if components.reachability_component > 0.0 {
            parts.push("Vulnerable code is reachable".to_string());
        } else {
            parts.push("Vulnerable code is NOT reachable (lower risk)".to_string());
        }
        
        if components.exploit_component > 0.0 {
            parts.push("Public exploit code available".to_string());
        }
        
        if components.age_component > 0.05 {
            parts.push(format!("Recent vulnerability ({} days old)", features.age_days));
        }
        
        parts.join(". ")
    }
}

impl Default for RiskScorer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_risk_scorer_creation() {
        let scorer = RiskScorer::new();
        assert_eq!(scorer.weights.cvss_weight, 0.25);
    }
    
    #[test]
    fn test_critical_risk_vulnerability() {
        let scorer = RiskScorer::new();
        let features = VulnerabilityFeatures {
            cvss_score: 9.8,
            age_days: 30,
            has_exploit: true,
            epss: 0.95,
            in_kev: true,
            severity_level: 3,
            vuln_type: 1,
            is_reachable: true,
        };
        
        let score = scorer.score(&features);
        assert!(score.overall_score > 0.8, "Expected critical risk, got {}", score.overall_score);
        assert_eq!(score.risk_level, RiskLevel::Critical);
        assert!(score.explanation.contains("CISA KEV"));
        assert!(score.explanation.contains("reachable"));
    }
    
    #[test]
    fn test_low_risk_vulnerability() {
        let scorer = RiskScorer::new();
        let features = VulnerabilityFeatures {
            cvss_score: 3.0,
            age_days: 730,
            has_exploit: false,
            epss: 0.01,
            in_kev: false,
            severity_level: 0,
            vuln_type: 0,
            is_reachable: false,
        };
        
        let score = scorer.score(&features);
        assert!(score.overall_score < 0.3, "Expected low risk, got {}", score.overall_score);
        assert!(matches!(score.risk_level, RiskLevel::Low | RiskLevel::Minimal));
        assert!(score.explanation.contains("NOT reachable"));
    }
    
    #[test]
    fn test_unreachable_reduces_risk() {
        let scorer = RiskScorer::new();
        
        // Same vulnerability, but one is reachable and one is not
        let reachable = VulnerabilityFeatures {
            cvss_score: 8.0,
            is_reachable: true,
            ..VulnerabilityFeatures::default()
        };
        
        let unreachable = VulnerabilityFeatures {
            cvss_score: 8.0,
            is_reachable: false,
            ..VulnerabilityFeatures::default()
        };
        
        let reachable_score = scorer.score(&reachable);
        let unreachable_score = scorer.score(&unreachable);
        
        assert!(reachable_score.overall_score > unreachable_score.overall_score);
    }
    
    #[test]
    fn test_kev_increases_risk() {
        let scorer = RiskScorer::new();
        
        let with_kev = VulnerabilityFeatures {
            cvss_score: 7.0,
            in_kev: true,
            ..VulnerabilityFeatures::default()
        };
        
        let without_kev = VulnerabilityFeatures {
            cvss_score: 7.0,
            in_kev: false,
            ..VulnerabilityFeatures::default()
        };
        
        let with_kev_score = scorer.score(&with_kev);
        let without_kev_score = scorer.score(&without_kev);
        
        assert!(with_kev_score.overall_score > without_kev_score.overall_score);
    }
    
    #[test]
    fn test_custom_weights() {
        // Reachability-focused scoring (give it 50% weight)
        let scorer = RiskScorer::with_weights(
            0.2,  // cvss
            0.1,  // epss
            0.1,  // kev
            0.5,  // reachability (high weight)
            0.05, // age
            0.05, // exploit
        );
        
        let reachable = VulnerabilityFeatures {
            cvss_score: 5.0,
            is_reachable: true,
            ..VulnerabilityFeatures::default()
        };
        
        let score = scorer.score(&reachable);
        // With 50% weight on reachability, score should be at least 0.5
        assert!(score.overall_score >= 0.5);
    }
    
    #[test]
    fn test_risk_level_thresholds() {
        let scorer = RiskScorer::new();
        
        // Test vulnerabilities that should fall into each risk level
        // These are realistic combinations of factors, not just CVSS alone
        
        // Critical: High CVSS + EPSS + KEV + reachable
        let critical = VulnerabilityFeatures {
            cvss_score: 9.5,
            epss: 0.9,
            in_kev: true,
            is_reachable: true,
            has_exploit: true,
            age_days: 10,
            ..VulnerabilityFeatures::default()
        };
        let score = scorer.score(&critical);
        assert_eq!(score.risk_level, RiskLevel::Critical, "Expected Critical, got {:?} (score: {:.2})", score.risk_level, score.overall_score);
        
        // High: Good CVSS + EPSS, reachable
        let high = VulnerabilityFeatures {
            cvss_score: 8.0,
            epss: 0.7,
            is_reachable: true,
            has_exploit: true,
            ..VulnerabilityFeatures::default()
        };
        let score = scorer.score(&high);
        assert_eq!(score.risk_level, RiskLevel::High, "Expected High, got {:?} (score: {:.2})", score.risk_level, score.overall_score);
        
        // Medium: Moderate CVSS, some factors
        let medium = VulnerabilityFeatures {
            cvss_score: 6.0,
            epss: 0.4,
            is_reachable: true,
            ..VulnerabilityFeatures::default()
        };
        let score = scorer.score(&medium);
        assert_eq!(score.risk_level, RiskLevel::Medium, "Expected Medium, got {:?} (score: {:.2})", score.risk_level, score.overall_score);
        
        // Low: Lower CVSS, but has some EPSS
        let low = VulnerabilityFeatures {
            cvss_score: 5.5,
            epss: 0.3,
            is_reachable: false,
            has_exploit: false,
            age_days: 200,
            ..VulnerabilityFeatures::default()
        };
        let score = scorer.score(&low);
        assert_eq!(score.risk_level, RiskLevel::Low, "Expected Low, got {:?} (score: {:.2})", score.risk_level, score.overall_score);
        
        // Minimal: Low CVSS, no factors
        let minimal = VulnerabilityFeatures {
            cvss_score: 3.0,
            epss: 0.01,
            is_reachable: false,
            age_days: 730,
            ..VulnerabilityFeatures::default()
        };
        let score = scorer.score(&minimal);
        assert_eq!(score.risk_level, RiskLevel::Minimal, "Expected Minimal, got {:?} (score: {:.2})", score.risk_level, score.overall_score);
    }
    
    #[test]
    fn test_age_factor() {
        let scorer = RiskScorer::new();
        
        let new_vuln = VulnerabilityFeatures {
            cvss_score: 7.0,
            age_days: 1,
            ..VulnerabilityFeatures::default()
        };
        
        let old_vuln = VulnerabilityFeatures {
            cvss_score: 7.0,
            age_days: 730,
            ..VulnerabilityFeatures::default()
        };
        
        let new_score = scorer.score(&new_vuln);
        let old_score = scorer.score(&old_vuln);
        
        // Newer vulnerabilities should score higher
        assert!(new_score.overall_score > old_score.overall_score);
    }
    
    #[test]
    fn test_explanation_content() {
        let scorer = RiskScorer::new();
        let features = VulnerabilityFeatures {
            cvss_score: 9.0,
            age_days: 10,
            has_exploit: true,
            epss: 0.8,
            in_kev: true,
            severity_level: 3,
            vuln_type: 1,
            is_reachable: true,
        };
        
        let score = scorer.score(&features);
        
        // Check that explanation contains key information
        assert!(score.explanation.contains("Risk score"));
        assert!(score.explanation.contains("CVSS"));
        assert!(score.explanation.contains("EPSS"));
        assert!(score.explanation.contains("KEV"));
        assert!(score.explanation.contains("reachable"));
        assert!(score.explanation.contains("exploit"));
    }
}
