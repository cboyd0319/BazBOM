//! Smart batch fixing with conflict detection and breaking change analysis
//!
//! This module implements intelligent grouping of vulnerability fixes:
//! - Batch 1: Independent updates (no shared dependencies, safe to apply together)
//! - Batch 2: Updates with breaking changes (major version bumps, require review)
//! - Batch 3: Conflicting updates (version conflicts with other dependencies)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::remediation::{parse_semantic_version, RemediationSuggestion};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RiskLevel {
    Low,      // Independent updates, no conflicts
    Moderate, // Breaking changes (major version bumps)
    High,     // Dependency conflicts detected
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update {
    pub package: String,
    pub current_version: String,
    pub target_version: String,
    pub cve: String,
    pub severity: String,
    pub is_breaking: bool,
    pub breaking_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Conflict {
    pub package: String,
    pub requested_version: String,
    pub conflicts_with: Vec<ConflictingDependency>,
}

#[derive(Debug, Clone)]
pub struct ConflictingDependency {
    pub package: String,
    pub required_version_range: String,
}

#[derive(Debug, Clone)]
pub struct Batch {
    pub risk: RiskLevel,
    pub updates: Vec<Update>,
    pub conflicts: Vec<Conflict>,
    pub breaking_changes: bool,
    pub estimated_time_seconds: u32,
    pub test_count: u32,
}

impl Batch {
    /// Create a description of this batch for display
    pub fn description(&self) -> String {
        match self.risk {
            RiskLevel::Low => format!(
                "Low-Risk Updates ({} vulnerabilities) - Independent and safe to apply together",
                self.updates.len()
            ),
            RiskLevel::Moderate => format!(
                "Moderate-Risk Updates ({} vulnerabilities) - Contains breaking changes, review recommended",
                self.updates.len()
            ),
            RiskLevel::High => format!(
                "High-Risk Updates ({} vulnerabilities) - Dependency conflicts detected",
                self.updates.len()
            ),
        }
    }

    /// Count vulnerabilities by severity in this batch
    pub fn severity_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for update in &self.updates {
            *counts.entry(update.severity.clone()).or_insert(0) += 1;
        }
        counts
    }
}

pub struct BatchFixer {
    updates: Vec<Update>,
}

impl BatchFixer {
    /// Create a new batch fixer from remediation suggestions
    pub fn new(suggestions: &[RemediationSuggestion]) -> Self {
        let updates = suggestions
            .iter()
            .filter_map(|s| {
                s.fixed_version.as_ref().map(|fixed| {
                    let is_breaking = is_breaking_change(&s.current_version, fixed);
                    let breaking_reason = if is_breaking {
                        Some(get_breaking_reason(&s.current_version, fixed))
                    } else {
                        None
                    };

                    Update {
                        package: s.affected_package.clone(),
                        current_version: s.current_version.clone(),
                        target_version: fixed.clone(),
                        cve: s.vulnerability_id.clone(),
                        severity: s.severity.clone(),
                        is_breaking,
                        breaking_reason,
                    }
                })
            })
            .collect();

        Self { updates }
    }

    /// Group updates into safe batches
    pub fn create_batches(&self) -> Vec<Batch> {
        let mut batches = Vec::new();

        // Batch 1: Independent, non-breaking updates
        let independent = self.find_independent_non_breaking_updates();
        if !independent.is_empty() {
            batches.push(Batch {
                risk: RiskLevel::Low,
                updates: independent.clone(),
                conflicts: vec![],
                breaking_changes: false,
                estimated_time_seconds: (independent.len() as u32 * 5) + 30,
                test_count: estimate_test_count(&independent),
            });
        }

        // Batch 2: Updates with breaking changes
        let breaking = self.find_breaking_updates();
        if !breaking.is_empty() {
            batches.push(Batch {
                risk: RiskLevel::Moderate,
                updates: breaking.clone(),
                conflicts: vec![],
                breaking_changes: true,
                estimated_time_seconds: (breaking.len() as u32 * 10) + 60,
                test_count: estimate_test_count(&breaking),
            });
        }

        // Batch 3: Conflicting updates (if we can detect them)
        let conflicting = self.find_potentially_conflicting_updates();
        if !conflicting.is_empty() {
            let conflicts = self.detect_conflicts(&conflicting);
            batches.push(Batch {
                risk: RiskLevel::High,
                updates: conflicting.clone(),
                conflicts,
                breaking_changes: false,
                estimated_time_seconds: (conflicting.len() as u32 * 15) + 90,
                test_count: estimate_test_count(&conflicting),
            });
        }

        batches
    }

    /// Find independent, non-breaking updates that can be safely applied together
    fn find_independent_non_breaking_updates(&self) -> Vec<Update> {
        self.updates
            .iter()
            .filter(|u| !u.is_breaking)
            .filter(|u| !self.is_commonly_conflicting(&u.package))
            .cloned()
            .collect()
    }

    /// Find updates with breaking changes
    fn find_breaking_updates(&self) -> Vec<Update> {
        self.updates
            .iter()
            .filter(|u| u.is_breaking)
            .cloned()
            .collect()
    }

    /// Find potentially conflicting updates
    fn find_potentially_conflicting_updates(&self) -> Vec<Update> {
        self.updates
            .iter()
            .filter(|u| !u.is_breaking)
            .filter(|u| self.is_commonly_conflicting(&u.package))
            .cloned()
            .collect()
    }

    /// Check if a package commonly causes conflicts
    /// This is a simplified heuristic - in production, would use dependency graph
    fn is_commonly_conflicting(&self, package: &str) -> bool {
        // Common packages that often have version conflicts
        let common_conflicts = vec![
            "netty",
            "jackson",
            "guava",
            "slf4j",
            "logback",
            "spring-",
            "kotlin-stdlib",
            "protobuf",
        ];

        common_conflicts
            .iter()
            .any(|pattern| package.to_lowercase().contains(pattern))
    }

    /// Detect actual conflicts (simplified - would need full dependency graph)
    fn detect_conflicts(&self, updates: &[Update]) -> Vec<Conflict> {
        let mut conflicts = Vec::new();

        for update in updates {
            // Check if this update is a major version bump
            if let (Some(current), Some(target)) = (
                parse_semantic_version(&update.current_version),
                parse_semantic_version(&update.target_version),
            ) {
                if target.0 > current.0 {
                    // Major version change - likely to conflict
                    conflicts.push(Conflict {
                        package: update.package.clone(),
                        requested_version: update.target_version.clone(),
                        conflicts_with: vec![ConflictingDependency {
                            package: "other-dependencies".to_string(),
                            required_version_range: format!("~{}", update.current_version),
                        }],
                    });
                }
            }
        }

        conflicts
    }
}

/// Check if a version change is a breaking change (major version bump)
fn is_breaking_change(current: &str, target: &str) -> bool {
    if let (Some(current_ver), Some(target_ver)) = (
        parse_semantic_version(current),
        parse_semantic_version(target),
    ) {
        // Major version change
        target_ver.0 > current_ver.0
    } else {
        false
    }
}

/// Get a human-readable reason for why this is a breaking change
fn get_breaking_reason(current: &str, target: &str) -> String {
    if let (Some(current_ver), Some(target_ver)) = (
        parse_semantic_version(current),
        parse_semantic_version(target),
    ) {
        format!(
            "Major version upgrade from {}.x.x to {}.x.x - API changes may be required",
            current_ver.0, target_ver.0
        )
    } else {
        "Version format change detected".to_string()
    }
}

/// Estimate number of tests that will run (simplified heuristic)
fn estimate_test_count(updates: &[Update]) -> u32 {
    // Assume ~20 tests per update as a baseline
    (updates.len() as u32) * 20
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_update(package: &str, current: &str, target: &str, severity: &str) -> Update {
        let is_breaking = is_breaking_change(current, target);
        Update {
            package: package.to_string(),
            current_version: current.to_string(),
            target_version: target.to_string(),
            cve: format!("CVE-2024-{}", package),
            severity: severity.to_string(),
            is_breaking,
            breaking_reason: if is_breaking {
                Some(get_breaking_reason(current, target))
            } else {
                None
            },
        }
    }

    #[test]
    fn test_is_breaking_change() {
        // Major version change
        assert!(is_breaking_change("1.0.0", "2.0.0"));
        assert!(is_breaking_change("2.5.3", "3.0.0"));

        // Minor version change
        assert!(!is_breaking_change("1.0.0", "1.1.0"));
        assert!(!is_breaking_change("2.5.3", "2.6.0"));

        // Patch version change
        assert!(!is_breaking_change("1.0.0", "1.0.1"));
        assert!(!is_breaking_change("2.5.3", "2.5.4"));
    }

    #[test]
    fn test_get_breaking_reason() {
        let reason = get_breaking_reason("1.0.0", "2.0.0");
        assert!(reason.contains("Major version upgrade"));
        assert!(reason.contains("1.x.x to 2.x.x"));
    }

    #[test]
    fn test_batch_fixer_groups_by_risk() {
        let updates = vec![
            create_test_update("commons-io", "2.7", "2.15.0", "MEDIUM"),
            create_test_update("guava", "30.1-jre", "32.1.3-jre", "MEDIUM"),
            create_test_update("spring-boot", "2.7.0", "3.2.0", "HIGH"),
            create_test_update("log4j-core", "2.14.1", "2.21.1", "CRITICAL"),
        ];

        let fixer = BatchFixer { updates };
        let batches = fixer.create_batches();

        // Should have at least 2 batches (low-risk and moderate-risk)
        assert!(batches.len() >= 2);

        // First batch should be low-risk
        assert_eq!(batches[0].risk, RiskLevel::Low);

        // Should have a moderate-risk batch with spring-boot (major version change)
        let has_moderate = batches.iter().any(|b| b.risk == RiskLevel::Moderate);
        assert!(has_moderate);
    }

    #[test]
    fn test_find_independent_updates() {
        let updates = vec![
            create_test_update("commons-io", "2.7", "2.15.0", "MEDIUM"),
            create_test_update("commons-codec", "1.15", "1.16.0", "LOW"),
        ];

        let fixer = BatchFixer { updates };
        let independent = fixer.find_independent_non_breaking_updates();

        assert_eq!(independent.len(), 2);
    }

    #[test]
    fn test_find_breaking_updates() {
        let updates = vec![
            create_test_update("commons-io", "2.7", "2.15.0", "MEDIUM"),
            create_test_update("spring-boot", "2.7.0", "3.2.0", "HIGH"),
            create_test_update("junit", "4.13.2", "5.10.0", "LOW"),
        ];

        let fixer = BatchFixer { updates };
        let breaking = fixer.find_breaking_updates();

        assert_eq!(breaking.len(), 2); // spring-boot and junit
        assert!(breaking.iter().any(|u| u.package.contains("spring-boot")));
        assert!(breaking.iter().any(|u| u.package.contains("junit")));
    }

    #[test]
    fn test_commonly_conflicting_packages() {
        let fixer = BatchFixer { updates: vec![] };

        assert!(fixer.is_commonly_conflicting("io.netty:netty-codec"));
        assert!(fixer.is_commonly_conflicting("com.fasterxml.jackson.core:jackson-databind"));
        assert!(fixer.is_commonly_conflicting("com.google.guava:guava"));
        assert!(!fixer.is_commonly_conflicting("commons-io:commons-io"));
    }

    #[test]
    fn test_batch_description() {
        let batch = Batch {
            risk: RiskLevel::Low,
            updates: vec![create_test_update("test", "1.0.0", "1.1.0", "HIGH")],
            conflicts: vec![],
            breaking_changes: false,
            estimated_time_seconds: 35,
            test_count: 20,
        };

        let desc = batch.description();
        assert!(desc.contains("Low-Risk"));
        assert!(desc.contains("1 vulnerabilities"));
    }

    #[test]
    fn test_severity_counts() {
        let batch = Batch {
            risk: RiskLevel::Low,
            updates: vec![
                create_test_update("pkg1", "1.0.0", "1.1.0", "CRITICAL"),
                create_test_update("pkg2", "2.0.0", "2.1.0", "HIGH"),
                create_test_update("pkg3", "3.0.0", "3.1.0", "HIGH"),
            ],
            conflicts: vec![],
            breaking_changes: false,
            estimated_time_seconds: 60,
            test_count: 60,
        };

        let counts = batch.severity_counts();
        assert_eq!(counts.get("CRITICAL"), Some(&1));
        assert_eq!(counts.get("HIGH"), Some(&2));
    }
}
