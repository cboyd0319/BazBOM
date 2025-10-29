use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub severity_threshold: Option<SeverityLevel>,
    pub license_allowlist: Option<Vec<String>>,
    pub license_denylist: Option<Vec<String>>,
    pub kev_gate: bool,
    pub epss_threshold: Option<f64>,
    pub reachability_required: bool,
    pub vex_auto_apply: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SeverityLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Priority {
    P0,
    P1,
    P2,
    P3,
    P4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: SeverityLevel,
    pub priority: Priority,
    pub description: String,
    pub component: String,
    pub fixed_version: Option<String>,
    pub kev: bool,
    pub epss_score: Option<f64>,
    pub reachable: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    pub rule: String,
    pub message: String,
    pub vulnerability: Option<Vulnerability>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PolicyResult {
    pub passed: bool,
    pub violations: Vec<PolicyViolation>,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            severity_threshold: Some(SeverityLevel::High),
            license_allowlist: None,
            license_denylist: None,
            kev_gate: false,
            epss_threshold: None,
            reachability_required: false,
            vex_auto_apply: true,
        }
    }
}

impl PolicyConfig {
    pub fn check_vulnerability(&self, vuln: &Vulnerability) -> Option<PolicyViolation> {
        if let Some(threshold) = self.severity_threshold {
            if vuln.severity >= threshold {
                return Some(PolicyViolation {
                    rule: "severity_threshold".to_string(),
                    message: format!(
                        "Vulnerability {} has severity {:?} which meets or exceeds threshold {:?}",
                        vuln.id, vuln.severity, threshold
                    ),
                    vulnerability: Some(vuln.clone()),
                });
            }
        }

        if self.kev_gate && vuln.kev {
            return Some(PolicyViolation {
                rule: "kev_gate".to_string(),
                message: format!("Vulnerability {} is in CISA KEV", vuln.id),
                vulnerability: Some(vuln.clone()),
            });
        }

        if let Some(threshold) = self.epss_threshold {
            if let Some(score) = vuln.epss_score {
                if score >= threshold {
                    return Some(PolicyViolation {
                        rule: "epss_threshold".to_string(),
                        message: format!(
                            "Vulnerability {} has EPSS score {} which exceeds threshold {}",
                            vuln.id, score, threshold
                        ),
                        vulnerability: Some(vuln.clone()),
                    });
                }
            }
        }

        None
    }

    pub fn check_license(&self, license: &str) -> Option<PolicyViolation> {
        if let Some(denylist) = &self.license_denylist {
            if denylist.contains(&license.to_string()) {
                return Some(PolicyViolation {
                    rule: "license_denylist".to_string(),
                    message: format!("License {} is in denylist", license),
                    vulnerability: None,
                });
            }
        }

        if let Some(allowlist) = &self.license_allowlist {
            if !allowlist.contains(&license.to_string()) {
                return Some(PolicyViolation {
                    rule: "license_allowlist".to_string(),
                    message: format!("License {} is not in allowlist", license),
                    vulnerability: None,
                });
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = PolicyConfig::default();
        assert_eq!(policy.severity_threshold, Some(SeverityLevel::High));
        assert!(!policy.kev_gate);
        assert!(policy.vex_auto_apply);
    }

    #[test]
    fn test_severity_threshold_violation() {
        let policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };

        let vuln = Vulnerability {
            id: "CVE-2024-1234".to_string(),
            severity: SeverityLevel::Critical,
            priority: Priority::P0,
            description: "Test vuln".to_string(),
            component: "test:lib:1.0".to_string(),
            fixed_version: Some("2.0".to_string()),
            kev: false,
            epss_score: None,
            reachable: None,
        };

        let violation = policy.check_vulnerability(&vuln);
        assert!(violation.is_some());
    }

    #[test]
    fn test_no_violation_below_threshold() {
        let policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Critical),
            ..Default::default()
        };

        let vuln = Vulnerability {
            id: "CVE-2024-1234".to_string(),
            severity: SeverityLevel::High,
            priority: Priority::P1,
            description: "Test vuln".to_string(),
            component: "test:lib:1.0".to_string(),
            fixed_version: None,
            kev: false,
            epss_score: None,
            reachable: None,
        };

        let violation = policy.check_vulnerability(&vuln);
        assert!(violation.is_none());
    }

    #[test]
    fn test_kev_gate() {
        let policy = PolicyConfig {
            kev_gate: true,
            ..Default::default()
        };

        let vuln = Vulnerability {
            id: "CVE-2024-1234".to_string(),
            severity: SeverityLevel::Medium,
            priority: Priority::P2,
            description: "Test vuln".to_string(),
            component: "test:lib:1.0".to_string(),
            fixed_version: None,
            kev: true,
            epss_score: None,
            reachable: None,
        };

        let violation = policy.check_vulnerability(&vuln);
        assert!(violation.is_some());
        assert_eq!(violation.unwrap().rule, "kev_gate");
    }
}
