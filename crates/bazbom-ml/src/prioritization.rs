//! ML-based vulnerability prioritization
//!
//! This module provides ML-enhanced vulnerability prioritization using
//! the risk scoring and anomaly detection features.

use crate::features::VulnerabilityFeatures;
use crate::risk::{RiskScorer, RiskLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Prioritized vulnerability with ML-enhanced ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedVulnerability {
    pub cve: String,
    pub package: String,
    pub version: String,
    pub risk_level: RiskLevel,
    pub risk_score: f64,
    pub priority_rank: usize,
    pub fix_urgency: FixUrgency,
    pub explanation: String,
}

/// Fix urgency level
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum FixUrgency {
    Immediate,  // Fix within 24 hours
    High,       // Fix within 1 week
    Medium,     // Fix within 1 month
    Low,        // Fix when convenient
}

impl FixUrgency {
    pub fn from_risk_level(level: &RiskLevel) -> Self {
        match level {
            RiskLevel::Critical => FixUrgency::Immediate,
            RiskLevel::High => FixUrgency::High,
            RiskLevel::Medium => FixUrgency::Medium,
            RiskLevel::Low | RiskLevel::Minimal => FixUrgency::Low,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            FixUrgency::Immediate => "IMMEDIATE",
            FixUrgency::High => "HIGH",
            FixUrgency::Medium => "MEDIUM",
            FixUrgency::Low => "LOW",
        }
    }
}

/// Vulnerability prioritizer using ML risk scoring
pub struct VulnerabilityPrioritizer {
    risk_scorer: RiskScorer,
}

impl VulnerabilityPrioritizer {
    /// Create new prioritizer with default risk scorer
    pub fn new() -> Self {
        Self {
            risk_scorer: RiskScorer::new(),
        }
    }

    /// Create prioritizer with custom risk scorer
    pub fn with_scorer(risk_scorer: RiskScorer) -> Self {
        Self { risk_scorer }
    }

    /// Prioritize vulnerabilities using ML-enhanced risk scoring
    pub fn prioritize(
        &self,
        vulnerabilities: Vec<(VulnerabilityFeatures, String, String, String)>,
    ) -> Vec<PrioritizedVulnerability> {
        let mut scored: Vec<_> = vulnerabilities
            .into_iter()
            .map(|(features, cve, package, version)| {
                let risk_result = self.risk_scorer.score(&features);
                
                let explanation = self.generate_explanation(&features, &risk_result.explanation);
                
                PrioritizedVulnerability {
                    cve,
                    package,
                    version,
                    risk_level: risk_result.risk_level.clone(),
                    risk_score: risk_result.overall_score,
                    priority_rank: 0, // Will be set after sorting
                    fix_urgency: FixUrgency::from_risk_level(&risk_result.risk_level),
                    explanation,
                }
            })
            .collect();

        // Sort by risk score (descending)
        scored.sort_by(|a, b| {
            b.risk_score
                .partial_cmp(&a.risk_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign priority ranks
        for (i, vuln) in scored.iter_mut().enumerate() {
            vuln.priority_rank = i + 1;
        }

        scored
    }

    /// Generate human-readable explanation for prioritization
    fn generate_explanation(&self, features: &VulnerabilityFeatures, base_explanation: &str) -> String {
        let mut parts = vec![base_explanation.to_string()];

        // Add context about specific risk factors
        if features.in_kev {
            parts.push("This vulnerability is listed in CISA's Known Exploited Vulnerabilities catalog, indicating active exploitation in the wild.".to_string());
        }

        if features.epss > 0.7 {
            parts.push(format!(
                "EPSS score of {:.1}% indicates a high probability of exploitation.",
                features.epss * 100.0
            ));
        }

        if features.is_reachable {
            parts.push("This vulnerability is in code that's reachable from your application, increasing the real-world risk.".to_string());
        }

        if features.has_exploit {
            parts.push("Public exploit code is available, making exploitation easier for attackers.".to_string());
        }

        if features.age_days < 30 {
            parts.push("This is a recently disclosed vulnerability, which often see increased exploitation attempts.".to_string());
        }

        parts.join(" ")
    }

    /// Create batch fix groups optimized by risk and dependencies
    pub fn create_fix_batches(
        &self,
        prioritized: &[PrioritizedVulnerability],
        dependency_graph: &HashMap<String, Vec<String>>,
    ) -> Vec<FixBatch> {
        let mut batches = Vec::new();

        // Batch 1: Immediate urgency (isolated fixes)
        let immediate: Vec<_> = prioritized
            .iter()
            .filter(|v| v.fix_urgency == FixUrgency::Immediate)
            .filter(|v| self.is_isolated(&v.package, dependency_graph))
            .cloned()
            .collect();

        if !immediate.is_empty() {
            batches.push(FixBatch {
                urgency: FixUrgency::Immediate,
                vulnerabilities: immediate,
                estimated_time_minutes: 30,
                conflicts: vec![],
                description: "Critical vulnerabilities with no dependency conflicts - safe to fix immediately".to_string(),
            });
        }

        // Batch 2: High urgency (may have conflicts)
        let high: Vec<_> = prioritized
            .iter()
            .filter(|v| v.fix_urgency == FixUrgency::High)
            .cloned()
            .collect();

        if !high.is_empty() {
            let conflicts = self.detect_conflicts(&high, dependency_graph);
            batches.push(FixBatch {
                urgency: FixUrgency::High,
                vulnerabilities: high,
                estimated_time_minutes: 60,
                conflicts,
                description: "High priority vulnerabilities - review for dependency conflicts".to_string(),
            });
        }

        // Batch 3: Medium/Low urgency (bulk update)
        let medium_low: Vec<_> = prioritized
            .iter()
            .filter(|v| matches!(v.fix_urgency, FixUrgency::Medium | FixUrgency::Low))
            .cloned()
            .collect();

        if !medium_low.is_empty() {
            batches.push(FixBatch {
                urgency: FixUrgency::Medium,
                vulnerabilities: medium_low,
                estimated_time_minutes: 120,
                conflicts: vec![],
                description: "Lower priority vulnerabilities - can be batched together".to_string(),
            });
        }

        batches
    }

    /// Check if a package is isolated (no shared dependencies)
    fn is_isolated(&self, package: &str, graph: &HashMap<String, Vec<String>>) -> bool {
        // Check if this package is depended upon by others
        for (_, deps) in graph.iter() {
            if deps.contains(&package.to_string()) {
                return false;
            }
        }
        true
    }

    /// Detect potential conflicts between vulnerabilities
    fn detect_conflicts(
        &self,
        vulnerabilities: &[PrioritizedVulnerability],
        graph: &HashMap<String, Vec<String>>,
    ) -> Vec<String> {
        let mut conflicts = Vec::new();
        let packages: Vec<&str> = vulnerabilities.iter().map(|v| v.package.as_str()).collect();

        // Check for shared dependencies
        for i in 0..packages.len() {
            for j in (i + 1)..packages.len() {
                if self.share_dependencies(packages[i], packages[j], graph) {
                    conflicts.push(format!(
                        "{} and {} share dependencies",
                        packages[i], packages[j]
                    ));
                }
            }
        }

        conflicts
    }

    /// Check if two packages share dependencies
    fn share_dependencies(
        &self,
        pkg1: &str,
        pkg2: &str,
        graph: &HashMap<String, Vec<String>>,
    ) -> bool {
        if let (Some(deps1), Some(deps2)) = (graph.get(pkg1), graph.get(pkg2)) {
            for dep in deps1 {
                if deps2.contains(dep) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for VulnerabilityPrioritizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Batch of vulnerabilities grouped for fixing together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixBatch {
    pub urgency: FixUrgency,
    pub vulnerabilities: Vec<PrioritizedVulnerability>,
    pub estimated_time_minutes: u32,
    pub conflicts: Vec<String>,
    pub description: String,
}

impl FixBatch {
    /// Get summary of this batch
    pub fn summary(&self) -> String {
        format!(
            "{} urgency: {} vulnerabilities (~{} min) - {}",
            self.urgency.as_str(),
            self.vulnerabilities.len(),
            self.estimated_time_minutes,
            self.description
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_features(cvss: f64, epss: f64, kev: bool, reachable: bool) -> VulnerabilityFeatures {
        VulnerabilityFeatures {
            cvss_score: cvss,
            epss,
            in_kev: kev,
            is_reachable: reachable,
            age_days: 10,
            has_exploit: false,
            severity_level: 2, // HIGH
            vuln_type: 1,      // RCE
        }
    }

    #[test]
    fn test_prioritizer_creation() {
        let _prioritizer = VulnerabilityPrioritizer::new();
        assert!(true); // Just test creation
    }

    #[test]
    fn test_prioritize_vulnerabilities() {
        let prioritizer = VulnerabilityPrioritizer::new();
        
        let vulnerabilities = vec![
            (create_test_features(7.5, 0.3, false, true), "CVE-2024-0001".to_string(), "log4j-core".to_string(), "2.14.1".to_string()),
            (create_test_features(9.8, 0.8, true, true), "CVE-2024-0002".to_string(), "spring-web".to_string(), "5.3.20".to_string()),
            (create_test_features(5.0, 0.1, false, false), "CVE-2024-0003".to_string(), "guava".to_string(), "30.1".to_string()),
        ];

        let prioritized = prioritizer.prioritize(vulnerabilities);
        
        assert_eq!(prioritized.len(), 3);
        // Highest risk should be first (CVE-2024-0002 with high CVSS, EPSS, KEV)
        assert_eq!(prioritized[0].cve, "CVE-2024-0002");
        assert_eq!(prioritized[0].priority_rank, 1);
        assert_eq!(prioritized[0].fix_urgency, FixUrgency::Immediate);
    }

    #[test]
    fn test_fix_urgency_ordering() {
        // Since Immediate is first in enum, it's less than High in derived Ord
        // This is correct for sorting (Immediate will sort first)
        assert!(FixUrgency::Immediate < FixUrgency::High);
        assert!(FixUrgency::High < FixUrgency::Medium);
        assert!(FixUrgency::Medium < FixUrgency::Low);
    }

    #[test]
    fn test_fix_urgency_from_risk_level() {
        assert_eq!(FixUrgency::from_risk_level(&RiskLevel::Critical), FixUrgency::Immediate);
        assert_eq!(FixUrgency::from_risk_level(&RiskLevel::High), FixUrgency::High);
        assert_eq!(FixUrgency::from_risk_level(&RiskLevel::Medium), FixUrgency::Medium);
        assert_eq!(FixUrgency::from_risk_level(&RiskLevel::Low), FixUrgency::Low);
    }

    #[test]
    fn test_create_fix_batches() {
        let prioritizer = VulnerabilityPrioritizer::new();
        
        let prioritized = vec![
            PrioritizedVulnerability {
                cve: "CVE-2024-0001".to_string(),
                package: "log4j-core".to_string(),
                version: "2.14.1".to_string(),
                risk_level: RiskLevel::Critical,
                risk_score: 95.0,
                priority_rank: 1,
                fix_urgency: FixUrgency::Immediate,
                explanation: "Critical vulnerability".to_string(),
            },
            PrioritizedVulnerability {
                cve: "CVE-2024-0002".to_string(),
                package: "guava".to_string(),
                version: "30.1".to_string(),
                risk_level: RiskLevel::Medium,
                risk_score: 50.0,
                priority_rank: 2,
                fix_urgency: FixUrgency::Medium,
                explanation: "Medium risk".to_string(),
            },
        ];

        let graph = HashMap::new();
        let batches = prioritizer.create_fix_batches(&prioritized, &graph);
        
        assert!(!batches.is_empty());
        // Should have at least one batch for immediate fixes
        assert!(batches.iter().any(|b| b.urgency == FixUrgency::Immediate));
    }

    #[test]
    fn test_isolated_package() {
        let prioritizer = VulnerabilityPrioritizer::new();
        let mut graph = HashMap::new();
        graph.insert("package-a".to_string(), vec!["package-b".to_string()]);
        
        // package-b is not isolated (package-a depends on it)
        assert!(!prioritizer.is_isolated("package-b", &graph));
        
        // package-a is isolated (nothing depends on it)
        assert!(prioritizer.is_isolated("package-a", &graph));
    }

    #[test]
    fn test_explanation_generation() {
        let prioritizer = VulnerabilityPrioritizer::new();
        let features = VulnerabilityFeatures {
            cvss_score: 9.8,
            epss: 0.85,
            in_kev: true,
            is_reachable: true,
            age_days: 5,
            has_exploit: true,
            severity_level: 3, // CRITICAL
            vuln_type: 1,      // RCE
        };
        
        let explanation = prioritizer.generate_explanation(&features, "High risk");
        
        // Should mention multiple risk factors
        assert!(explanation.contains("KEV") || explanation.contains("Known Exploited"));
        assert!(explanation.contains("EPSS") || explanation.contains("probability"));
        assert!(explanation.contains("reachable"));
        assert!(explanation.contains("exploit"));
    }
}
