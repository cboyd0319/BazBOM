//! Dependency confusion attack detection
//!
//! Detects potential dependency confusion attacks where an attacker
//! publishes a malicious package with the same name as an internal package

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Package source/registry information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageRegistry {
    /// Maven Central
    MavenCentral,
    /// Private/Internal Maven repository
    PrivateMaven(String),
    /// npm registry
    NpmRegistry,
    /// Private npm registry
    PrivateNpm(String),
    /// PyPI
    PyPI,
    /// Private Python index
    PrivatePyPI(String),
    /// Unknown registry
    Unknown,
}

/// Internal package configuration for confusion detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalPackageConfig {
    /// Package name
    pub name: String,
    /// Expected registry
    pub registry: PackageRegistry,
    /// Group ID (for Maven)
    pub group_id: Option<String>,
    /// Scope (for npm)
    pub scope: Option<String>,
}

/// Dependency confusion detector
pub struct DependencyConfusionDetector {
    internal_packages: HashSet<String>,
    internal_configs: Vec<InternalPackageConfig>,
}

impl DependencyConfusionDetector {
    /// Create a new detector
    pub fn new() -> Self {
        Self {
            internal_packages: HashSet::new(),
            internal_configs: Vec::new(),
        }
    }

    /// Load internal package names
    pub fn load_internal_packages(&mut self, packages: Vec<String>) {
        self.internal_packages = packages.into_iter().collect();
    }

    /// Add internal package configuration
    pub fn add_internal_config(&mut self, config: InternalPackageConfig) {
        self.internal_packages.insert(config.name.clone());
        self.internal_configs.push(config);
    }

    /// Check if a package might be a dependency confusion attack
    pub fn check_dependency_confusion(
        &self,
        package_name: &str,
        registry: &PackageRegistry,
        version: &str,
    ) -> Option<ThreatIndicator> {
        // Check if package name matches an internal package
        if !self.internal_packages.contains(package_name) {
            return None;
        }

        // Find the expected configuration
        let expected_config = self
            .internal_configs
            .iter()
            .find(|c| c.name == package_name);

        if let Some(config) = expected_config {
            // Check if registry matches expected
            if registry != &config.registry {
                return Some(ThreatIndicator {
                    package_name: package_name.to_string(),
                    package_version: version.to_string(),
                    threat_level: ThreatLevel::Critical,
                    threat_type: ThreatType::SupplyChainAttack,
                    description: format!(
                        "Potential dependency confusion attack: internal package '{}' resolved from unexpected registry",
                        package_name
                    ),
                    evidence: vec![
                        format!("Package name matches internal package: {}", package_name),
                        format!("Expected registry: {:?}", config.registry),
                        format!("Actual registry: {:?}", registry),
                        "This could be a dependency confusion attack".to_string(),
                    ],
                    recommendation: format!(
                        "Verify that '{}' v{} is from the correct internal registry. \
                        If not, an attacker may have published a malicious package with the same name to a public registry.",
                        package_name, version
                    ),
                });
            }
        } else {
            // Internal package but no config - suspicious
            if is_public_registry(registry) {
                return Some(ThreatIndicator {
                    package_name: package_name.to_string(),
                    package_version: version.to_string(),
                    threat_level: ThreatLevel::High,
                    threat_type: ThreatType::SupplyChainAttack,
                    description: format!(
                        "Potential dependency confusion: package '{}' matches internal naming but resolved from public registry",
                        package_name
                    ),
                    evidence: vec![
                        format!("Package name: {}", package_name),
                        "Name matches internal package pattern".to_string(),
                        format!("Resolved from public registry: {:?}", registry),
                        "No explicit internal registry configuration found".to_string(),
                    ],
                    recommendation: format!(
                        "Verify '{}' v{} is legitimate. Configure explicit registry for internal packages to prevent confusion attacks.",
                        package_name, version
                    ),
                });
            }
        }

        None
    }

    /// Detect suspicious version patterns that may indicate confusion attacks
    pub fn detect_suspicious_version(&self, package_name: &str, version: &str) -> Option<Vec<String>> {
        let mut evidence = Vec::new();

        // Extremely high version numbers (999.x.x) are suspicious
        if let Some(major) = version.split('.').next() {
            if let Ok(major_num) = major.parse::<u32>() {
                if major_num >= 999 {
                    evidence.push(format!(
                        "Suspicious version number: v{} (attackers often use high versions to win version resolution)",
                        version
                    ));
                }
            }
        }

        // Check for unusual version patterns
        if version.contains("999") || version.contains("9999") {
            evidence.push("Version contains multiple 9's (common in confusion attacks)".to_string());
        }

        if evidence.is_empty() {
            None
        } else {
            Some(evidence)
        }
    }
}

impl Default for DependencyConfusionDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if registry is public
fn is_public_registry(registry: &PackageRegistry) -> bool {
    matches!(
        registry,
        PackageRegistry::MavenCentral | PackageRegistry::NpmRegistry | PackageRegistry::PyPI
    )
}

/// Analyze dependency configuration for confusion vulnerabilities
pub fn analyze_dependency_config(
    dependencies: &[(String, String)],
    internal_packages: &HashSet<String>,
) -> Result<Vec<String>> {
    let mut recommendations = Vec::new();

    // Check if any internal packages are in the dependency list
    for (pkg_name, _) in dependencies {
        if internal_packages.contains(pkg_name) {
            recommendations.push(format!(
                "Internal package '{}' found in dependencies. Ensure registry priority is configured correctly.",
                pkg_name
            ));
        }
    }

    // General recommendations
    if !internal_packages.is_empty() {
        recommendations.push(
            "Configure dependency resolution to prioritize internal registries over public ones.".to_string()
        );
        recommendations.push(
            "Use explicit registry URLs in dependency declarations where possible.".to_string()
        );
        recommendations.push(
            "Consider using package scoping (npm @scope, Maven groupId) for internal packages.".to_string()
        );
    }

    Ok(recommendations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = DependencyConfusionDetector::new();
        assert!(detector.internal_packages.is_empty());
    }

    #[test]
    fn test_load_internal_packages() {
        let mut detector = DependencyConfusionDetector::new();
        detector.load_internal_packages(vec!["internal-api".to_string(), "internal-utils".to_string()]);
        assert_eq!(detector.internal_packages.len(), 2);
    }

    #[test]
    fn test_dependency_confusion_detection() {
        let mut detector = DependencyConfusionDetector::new();
        detector.add_internal_config(InternalPackageConfig {
            name: "internal-api".to_string(),
            registry: PackageRegistry::PrivateMaven("https://internal.example.com".to_string()),
            group_id: Some("com.example".to_string()),
            scope: None,
        });

        // Should detect confusion when internal package comes from public registry
        let result = detector.check_dependency_confusion(
            "internal-api",
            &PackageRegistry::MavenCentral,
            "1.0.0",
        );
        assert!(result.is_some());

        let threat = result.unwrap();
        assert_eq!(threat.threat_level, ThreatLevel::Critical);
        assert_eq!(threat.threat_type, ThreatType::SupplyChainAttack);
    }

    #[test]
    fn test_no_confusion_for_external_packages() {
        let detector = DependencyConfusionDetector::new();
        let result = detector.check_dependency_confusion(
            "spring-boot",
            &PackageRegistry::MavenCentral,
            "3.2.0",
        );
        assert!(result.is_none());
    }

    #[test]
    fn test_suspicious_version_detection() {
        let detector = DependencyConfusionDetector::new();
        
        // High version number
        let evidence = detector.detect_suspicious_version("test-package", "999.0.0");
        assert!(evidence.is_some());
        
        // Normal version
        let evidence = detector.detect_suspicious_version("test-package", "1.2.3");
        assert!(evidence.is_none());
    }

    #[test]
    fn test_is_public_registry() {
        assert!(is_public_registry(&PackageRegistry::MavenCentral));
        assert!(is_public_registry(&PackageRegistry::NpmRegistry));
        assert!(!is_public_registry(&PackageRegistry::PrivateMaven("https://example.com".to_string())));
    }

    #[test]
    fn test_analyze_dependency_config() {
        let mut internal = HashSet::new();
        internal.insert("internal-api".to_string());

        let deps = vec![
            ("internal-api".to_string(), "1.0.0".to_string()),
            ("spring-boot".to_string(), "3.2.0".to_string()),
        ];

        let recommendations = analyze_dependency_config(&deps, &internal).unwrap();
        assert!(!recommendations.is_empty());
        assert!(recommendations.iter().any(|r| r.contains("internal-api")));
    }
}
