use crate::{PolicyConfig, SeverityLevel};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MergeStrategy {
    Strict,
    Permissive,
    Override,
}

impl Default for MergeStrategy {
    fn default() -> Self {
        MergeStrategy::Strict
    }
}

pub fn merge_policies(policies: Vec<PolicyConfig>, strategy: MergeStrategy) -> PolicyConfig {
    if policies.is_empty() {
        return PolicyConfig::default();
    }

    if policies.len() == 1 {
        return policies[0].clone();
    }

    let mut merged = policies[0].clone();

    for policy in &policies[1..] {
        merged = merge_two_policies(merged, policy.clone(), strategy);
    }

    merged
}

fn merge_two_policies(
    base: PolicyConfig,
    override_policy: PolicyConfig,
    strategy: MergeStrategy,
) -> PolicyConfig {
    match strategy {
        MergeStrategy::Strict => merge_strict(base, override_policy),
        MergeStrategy::Permissive => merge_permissive(base, override_policy),
        MergeStrategy::Override => override_policy,
    }
}

fn merge_strict(base: PolicyConfig, override_policy: PolicyConfig) -> PolicyConfig {
    PolicyConfig {
        severity_threshold: select_strictest_severity(
            base.severity_threshold,
            override_policy.severity_threshold,
        ),
        license_allowlist: merge_license_allowlist_strict(
            base.license_allowlist,
            override_policy.license_allowlist,
        ),
        license_denylist: merge_license_denylist_strict(
            base.license_denylist,
            override_policy.license_denylist,
        ),
        kev_gate: base.kev_gate || override_policy.kev_gate,
        epss_threshold: select_strictest_epss(base.epss_threshold, override_policy.epss_threshold),
        reachability_required: base.reachability_required || override_policy.reachability_required,
        vex_auto_apply: base.vex_auto_apply && override_policy.vex_auto_apply,
    }
}

fn merge_permissive(base: PolicyConfig, override_policy: PolicyConfig) -> PolicyConfig {
    PolicyConfig {
        severity_threshold: select_most_permissive_severity(
            base.severity_threshold,
            override_policy.severity_threshold,
        ),
        license_allowlist: merge_license_allowlist_permissive(
            base.license_allowlist,
            override_policy.license_allowlist,
        ),
        license_denylist: merge_license_denylist_permissive(
            base.license_denylist,
            override_policy.license_denylist,
        ),
        kev_gate: base.kev_gate && override_policy.kev_gate,
        epss_threshold: select_most_permissive_epss(
            base.epss_threshold,
            override_policy.epss_threshold,
        ),
        reachability_required: base.reachability_required && override_policy.reachability_required,
        vex_auto_apply: base.vex_auto_apply || override_policy.vex_auto_apply,
    }
}

fn select_strictest_severity(
    base: Option<SeverityLevel>,
    override_val: Option<SeverityLevel>,
) -> Option<SeverityLevel> {
    match (base, override_val) {
        (Some(b), Some(o)) => Some(std::cmp::min(b, o)),
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn select_most_permissive_severity(
    base: Option<SeverityLevel>,
    override_val: Option<SeverityLevel>,
) -> Option<SeverityLevel> {
    match (base, override_val) {
        (Some(b), Some(o)) => Some(std::cmp::max(b, o)),
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn select_strictest_epss(base: Option<f64>, override_val: Option<f64>) -> Option<f64> {
    match (base, override_val) {
        (Some(b), Some(o)) => Some(b.min(o)),
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn select_most_permissive_epss(base: Option<f64>, override_val: Option<f64>) -> Option<f64> {
    match (base, override_val) {
        (Some(b), Some(o)) => Some(b.max(o)),
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn merge_license_allowlist_strict(
    base: Option<Vec<String>>,
    override_val: Option<Vec<String>>,
) -> Option<Vec<String>> {
    match (base, override_val) {
        (Some(b), Some(o)) => {
            let intersection: Vec<String> = b.into_iter().filter(|item| o.contains(item)).collect();
            if intersection.is_empty() {
                None
            } else {
                Some(intersection)
            }
        }
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn merge_license_allowlist_permissive(
    base: Option<Vec<String>>,
    override_val: Option<Vec<String>>,
) -> Option<Vec<String>> {
    match (base, override_val) {
        (Some(mut b), Some(o)) => {
            for item in o {
                if !b.contains(&item) {
                    b.push(item);
                }
            }
            Some(b)
        }
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn merge_license_denylist_strict(
    base: Option<Vec<String>>,
    override_val: Option<Vec<String>>,
) -> Option<Vec<String>> {
    match (base, override_val) {
        (Some(mut b), Some(o)) => {
            for item in o {
                if !b.contains(&item) {
                    b.push(item);
                }
            }
            Some(b)
        }
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

fn merge_license_denylist_permissive(
    base: Option<Vec<String>>,
    override_val: Option<Vec<String>>,
) -> Option<Vec<String>> {
    match (base, override_val) {
        (Some(b), Some(o)) => {
            let intersection: Vec<String> = b.into_iter().filter(|item| o.contains(item)).collect();
            if intersection.is_empty() {
                None
            } else {
                Some(intersection)
            }
        }
        (Some(b), None) => Some(b),
        (None, Some(o)) => Some(o),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_strict_severity() {
        let base = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Medium),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Strict);
        assert_eq!(merged.severity_threshold, Some(SeverityLevel::Medium));
    }

    #[test]
    fn test_merge_permissive_severity() {
        let base = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Medium),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Permissive);
        assert_eq!(merged.severity_threshold, Some(SeverityLevel::High));
    }

    #[test]
    fn test_merge_override() {
        let base = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            kev_gate: true,
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Low),
            kev_gate: false,
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Override);
        assert_eq!(merged.severity_threshold, Some(SeverityLevel::Low));
        assert!(!merged.kev_gate);
    }

    #[test]
    fn test_merge_strict_kev_gate() {
        let base = PolicyConfig {
            kev_gate: false,
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            kev_gate: true,
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Strict);
        assert!(merged.kev_gate);
    }

    #[test]
    fn test_merge_permissive_kev_gate() {
        let base = PolicyConfig {
            kev_gate: true,
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            kev_gate: false,
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Permissive);
        assert!(!merged.kev_gate);
    }

    #[test]
    fn test_merge_strict_epss() {
        let base = PolicyConfig {
            epss_threshold: Some(0.7),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            epss_threshold: Some(0.5),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Strict);
        assert_eq!(merged.epss_threshold, Some(0.5));
    }

    #[test]
    fn test_merge_permissive_epss() {
        let base = PolicyConfig {
            epss_threshold: Some(0.5),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            epss_threshold: Some(0.7),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Permissive);
        assert_eq!(merged.epss_threshold, Some(0.7));
    }

    #[test]
    fn test_merge_strict_license_allowlist() {
        let base = PolicyConfig {
            license_allowlist: Some(vec!["MIT".to_string(), "Apache-2.0".to_string()]),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            license_allowlist: Some(vec!["Apache-2.0".to_string(), "BSD-3-Clause".to_string()]),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Strict);
        let allowlist = merged.license_allowlist.unwrap();
        assert_eq!(allowlist.len(), 1);
        assert!(allowlist.contains(&"Apache-2.0".to_string()));
    }

    #[test]
    fn test_merge_permissive_license_allowlist() {
        let base = PolicyConfig {
            license_allowlist: Some(vec!["MIT".to_string()]),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            license_allowlist: Some(vec!["Apache-2.0".to_string()]),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Permissive);
        let allowlist = merged.license_allowlist.unwrap();
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.contains(&"MIT".to_string()));
        assert!(allowlist.contains(&"Apache-2.0".to_string()));
    }

    #[test]
    fn test_merge_strict_license_denylist() {
        let base = PolicyConfig {
            license_denylist: Some(vec!["GPL-3.0".to_string()]),
            ..Default::default()
        };
        let override_policy = PolicyConfig {
            license_denylist: Some(vec!["AGPL-3.0".to_string()]),
            ..Default::default()
        };

        let merged = merge_policies(vec![base, override_policy], MergeStrategy::Strict);
        let denylist = merged.license_denylist.unwrap();
        assert_eq!(denylist.len(), 2);
        assert!(denylist.contains(&"GPL-3.0".to_string()));
        assert!(denylist.contains(&"AGPL-3.0".to_string()));
    }

    #[test]
    fn test_merge_three_policies() {
        let org_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Critical),
            kev_gate: true,
            ..Default::default()
        };
        let team_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::High),
            ..Default::default()
        };
        let project_policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Medium),
            ..Default::default()
        };

        let merged = merge_policies(
            vec![org_policy, team_policy, project_policy],
            MergeStrategy::Strict,
        );
        assert_eq!(merged.severity_threshold, Some(SeverityLevel::Medium));
        assert!(merged.kev_gate);
    }

    #[test]
    fn test_merge_empty_policies() {
        let merged = merge_policies(vec![], MergeStrategy::Strict);
        assert_eq!(merged.severity_threshold, Some(SeverityLevel::High));
    }

    #[test]
    fn test_merge_single_policy() {
        let policy = PolicyConfig {
            severity_threshold: Some(SeverityLevel::Critical),
            kev_gate: true,
            ..Default::default()
        };

        let merged = merge_policies(vec![policy.clone()], MergeStrategy::Strict);
        assert_eq!(merged.severity_threshold, policy.severity_threshold);
        assert_eq!(merged.kev_gate, policy.kev_gate);
    }
}
