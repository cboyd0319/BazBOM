use anyhow::{Context, Result};
use bazbom_advisories::Vulnerability as AdvisoryVuln;
use bazbom_policy::{PolicyConfig, PolicyResult, Vulnerability as PolicyVuln, SeverityLevel, Priority};
use std::fs;
use std::path::Path;

/// Convert advisory vulnerability to policy vulnerability for checking
pub fn convert_to_policy_vuln(advisory_vuln: &AdvisoryVuln, component: &str) -> PolicyVuln {
    let severity = match advisory_vuln.severity.as_ref().map(|s| s.level) {
        Some(bazbom_advisories::SeverityLevel::Critical) => SeverityLevel::Critical,
        Some(bazbom_advisories::SeverityLevel::High) => SeverityLevel::High,
        Some(bazbom_advisories::SeverityLevel::Medium) => SeverityLevel::Medium,
        Some(bazbom_advisories::SeverityLevel::Low) => SeverityLevel::Low,
        Some(bazbom_advisories::SeverityLevel::Unknown) | None => SeverityLevel::None,
    };

    let priority = match advisory_vuln.priority {
        Some(bazbom_advisories::Priority::P0) => Priority::P0,
        Some(bazbom_advisories::Priority::P1) => Priority::P1,
        Some(bazbom_advisories::Priority::P2) => Priority::P2,
        Some(bazbom_advisories::Priority::P3) => Priority::P3,
        Some(bazbom_advisories::Priority::P4) | None => Priority::P4,
    };

    let description = advisory_vuln
        .summary
        .as_ref()
        .or(advisory_vuln.details.as_ref())
        .cloned()
        .unwrap_or_else(|| format!("Vulnerability {}", advisory_vuln.id));

    // Try to find a fixed version from affected packages
    let fixed_version = advisory_vuln.affected.iter()
        .flat_map(|pkg| pkg.ranges.iter())
        .flat_map(|range| range.events.iter())
        .find_map(|event| {
            if let bazbom_advisories::VersionEvent::Fixed { fixed } = event {
                Some(fixed.clone())
            } else {
                None
            }
        });

    PolicyVuln {
        id: advisory_vuln.id.clone(),
        severity,
        priority,
        description,
        component: component.to_string(),
        fixed_version,
        kev: advisory_vuln.kev.is_some(),
        epss_score: advisory_vuln.epss.as_ref().map(|e| e.score),
        reachable: None, // Will be set when reachability analysis is implemented
    }
}

/// Load policy configuration from a YAML file
pub fn load_policy_config<P: AsRef<Path>>(path: P) -> Result<PolicyConfig> {
    let path = path.as_ref();
    if !path.exists() {
        // Return default policy if file doesn't exist
        return Ok(PolicyConfig::default());
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read policy config from {:?}", path))?;
    
    let config: PolicyConfig = serde_yaml::from_str(&content)
        .with_context(|| format!("failed to parse policy config from {:?}", path))?;
    
    Ok(config)
}

/// Check vulnerabilities against policy and return violations
pub fn check_policy(
    advisory_vulns: &[AdvisoryVuln],
    policy: &PolicyConfig,
) -> PolicyResult {
    let mut violations = Vec::new();

    for advisory_vuln in advisory_vulns {
        // Convert to policy vulnerability
        // Use first affected package as component, or "unknown" if none
        let component = advisory_vuln.affected.first()
            .map(|pkg| format!("{}:{}", pkg.ecosystem, pkg.package))
            .unwrap_or_else(|| "unknown".to_string());
        
        let policy_vuln = convert_to_policy_vuln(advisory_vuln, &component);
        
        // Check vulnerability against policy
        if let Some(violation) = policy.check_vulnerability(&policy_vuln) {
            violations.push(violation);
        }
    }

    PolicyResult {
        passed: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bazbom_advisories::{AffectedPackage, Severity, SeverityLevel as AdvSeverityLevel, VersionRange, VersionEvent, EpssScore, KevEntry};

    #[test]
    fn test_convert_to_policy_vuln_critical() {
        let advisory_vuln = AdvisoryVuln {
            id: "CVE-2024-1234".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "test-lib".to_string(),
                ranges: vec![VersionRange {
                    range_type: "SEMVER".to_string(),
                    events: vec![
                        VersionEvent::Introduced { introduced: "0.0.0".to_string() },
                        VersionEvent::Fixed { fixed: "2.0.0".to_string() },
                    ],
                }],
            }],
            severity: Some(Severity {
                cvss_v3: Some(9.8),
                cvss_v4: None,
                level: AdvSeverityLevel::Critical,
            }),
            summary: Some("Critical vulnerability".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: Some(EpssScore {
                score: 0.95,
                percentile: 0.99,
            }),
            kev: Some(KevEntry {
                cve_id: "CVE-2024-1234".to_string(),
                vendor_project: "Test Vendor".to_string(),
                product: "Test Product".to_string(),
                vulnerability_name: "Test Vuln".to_string(),
                date_added: "2024-01-01".to_string(),
                required_action: "Update".to_string(),
                due_date: "2024-02-01".to_string(),
            }),
            priority: Some(bazbom_advisories::Priority::P0),
        };

        let policy_vuln = convert_to_policy_vuln(&advisory_vuln, "Maven:test-lib");

        assert_eq!(policy_vuln.id, "CVE-2024-1234");
        assert_eq!(policy_vuln.severity, SeverityLevel::Critical);
        assert_eq!(policy_vuln.priority, Priority::P0);
        assert_eq!(policy_vuln.component, "Maven:test-lib");
        assert_eq!(policy_vuln.fixed_version, Some("2.0.0".to_string()));
        assert!(policy_vuln.kev);
        assert_eq!(policy_vuln.epss_score, Some(0.95));
    }

    #[test]
    fn test_convert_to_policy_vuln_no_severity() {
        let advisory_vuln = AdvisoryVuln {
            id: "CVE-2024-5678".to_string(),
            aliases: vec![],
            affected: vec![],
            severity: None,
            summary: None,
            details: Some("Some details".to_string()),
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        };

        let policy_vuln = convert_to_policy_vuln(&advisory_vuln, "test:component");

        assert_eq!(policy_vuln.severity, SeverityLevel::None);
        assert_eq!(policy_vuln.priority, Priority::P4);
        assert!(!policy_vuln.kev);
        assert!(policy_vuln.epss_score.is_none());
    }

    #[test]
    fn test_load_policy_config_default() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("nonexistent.yml");

        let result = load_policy_config(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.severity_threshold, Some(SeverityLevel::High));
    }

    #[test]
    fn test_load_policy_config_from_yaml() {
        let tmp = tempfile::tempdir().unwrap();
        let config_path = tmp.path().join("bazbom.yml");

        let yaml_content = r#"
severity_threshold: CRITICAL
license_allowlist:
  - MIT
  - Apache-2.0
kev_gate: true
epss_threshold: 0.8
reachability_required: false
vex_auto_apply: true
"#;
        fs::write(&config_path, yaml_content).unwrap();

        let result = load_policy_config(&config_path);
        assert!(result.is_ok());
        
        let config = result.unwrap();
        assert_eq!(config.severity_threshold, Some(SeverityLevel::Critical));
        assert_eq!(config.license_allowlist, Some(vec!["MIT".to_string(), "Apache-2.0".to_string()]));
        assert!(config.kev_gate);
        assert_eq!(config.epss_threshold, Some(0.8));
    }

    #[test]
    fn test_check_policy_no_violations() {
        let advisory_vulns = vec![AdvisoryVuln {
            id: "CVE-2024-LOW".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "test-lib".to_string(),
                ranges: vec![],
            }],
            severity: Some(Severity {
                cvss_v3: Some(3.0),
                cvss_v4: None,
                level: AdvSeverityLevel::Low,
            }),
            summary: Some("Low severity".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: Some(bazbom_advisories::Priority::P4),
        }];

        let policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };

        let result = check_policy(&advisory_vulns, &policy);
        assert!(result.passed);
        assert_eq!(result.violations.len(), 0);
    }

    #[test]
    fn test_check_policy_with_violations() {
        let advisory_vulns = vec![AdvisoryVuln {
            id: "CVE-2024-CRITICAL".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "vuln-lib".to_string(),
                ranges: vec![],
            }],
            severity: Some(Severity {
                cvss_v3: Some(9.8),
                cvss_v4: None,
                level: AdvSeverityLevel::Critical,
            }),
            summary: Some("Critical vuln".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: Some(bazbom_advisories::Priority::P0),
        }];

        let policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };

        let result = check_policy(&advisory_vulns, &policy);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "severity_threshold");
    }

    #[test]
    fn test_check_policy_kev_gate() {
        let advisory_vulns = vec![AdvisoryVuln {
            id: "CVE-2024-KEV".to_string(),
            aliases: vec![],
            affected: vec![AffectedPackage {
                ecosystem: "Maven".to_string(),
                package: "kev-lib".to_string(),
                ranges: vec![],
            }],
            severity: Some(Severity {
                cvss_v3: Some(5.0),
                cvss_v4: None,
                level: AdvSeverityLevel::Medium,
            }),
            summary: Some("KEV vuln".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: Some(KevEntry {
                cve_id: "CVE-2024-KEV".to_string(),
                vendor_project: "Test".to_string(),
                product: "Test".to_string(),
                vulnerability_name: "Test".to_string(),
                date_added: "2024-01-01".to_string(),
                required_action: "Update".to_string(),
                due_date: "2024-02-01".to_string(),
            }),
            priority: Some(bazbom_advisories::Priority::P1),
        }];

        let policy = PolicyConfig {
            kev_gate: true,
            ..Default::default()
        };

        let result = check_policy(&advisory_vulns, &policy);
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].rule, "kev_gate");
    }
}
