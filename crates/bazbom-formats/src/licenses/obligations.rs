use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObligationType {
    Attribution,
    Disclosure,
    Copyleft,
    PatentGrant,
    NoWarranty,
    SourceCodeDistribution,
    NoticeFile,
    Trademark,
    NetworkUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Obligation {
    pub obligation_type: ObligationType,
    pub description: String,
    pub severity: ObligationSeverity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum ObligationSeverity {
    Low,
    Medium,
    High,
}

pub struct LicenseObligations {
    obligations: HashMap<String, Vec<Obligation>>,
}

impl LicenseObligations {
    pub fn new() -> Self {
        let mut obligations = HashMap::new();
        
        Self::populate_obligations(&mut obligations);
        
        Self { obligations }
    }

    fn populate_obligations(obligations: &mut HashMap<String, Vec<Obligation>>) {
        obligations.insert(
            "MIT".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright notice and permission notice in all copies or substantial portions".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::NoWarranty,
                    description: "Software provided \"as is\" without warranty".to_string(),
                    severity: ObligationSeverity::Low,
                },
            ],
        );

        obligations.insert(
            "Apache-2.0".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include NOTICE file if present in original distribution".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::PatentGrant,
                    description: "Grants express patent license to users".to_string(),
                    severity: ObligationSeverity::Low,
                },
                Obligation {
                    obligation_type: ObligationType::NoticeFile,
                    description: "Must retain all copyright, patent, trademark, and attribution notices".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::NoWarranty,
                    description: "Software provided \"as is\" without warranty".to_string(),
                    severity: ObligationSeverity::Low,
                },
            ],
        );

        obligations.insert(
            "GPL-3.0-only".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Disclosure,
                    description: "Must provide source code to recipients of binary distribution".to_string(),
                    severity: ObligationSeverity::High,
                },
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright notice and license text".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::Copyleft,
                    description: "Derivative works must use same license (GPL-3.0)".to_string(),
                    severity: ObligationSeverity::High,
                },
                Obligation {
                    obligation_type: ObligationType::PatentGrant,
                    description: "Grants patent rights to users".to_string(),
                    severity: ObligationSeverity::Low,
                },
                Obligation {
                    obligation_type: ObligationType::SourceCodeDistribution,
                    description: "Must make source code available for at least 3 years".to_string(),
                    severity: ObligationSeverity::High,
                },
            ],
        );

        obligations.insert(
            "GPL-3.0-or-later".to_string(),
            obligations.get("GPL-3.0-only").unwrap().clone(),
        );

        obligations.insert(
            "AGPL-3.0-only".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Disclosure,
                    description: "Must provide source code to recipients of binary distribution".to_string(),
                    severity: ObligationSeverity::High,
                },
                Obligation {
                    obligation_type: ObligationType::NetworkUse,
                    description: "Must provide source code to network users (not just binary recipients)".to_string(),
                    severity: ObligationSeverity::High,
                },
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright notice and license text".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::Copyleft,
                    description: "Derivative works must use same license (AGPL-3.0)".to_string(),
                    severity: ObligationSeverity::High,
                },
                Obligation {
                    obligation_type: ObligationType::SourceCodeDistribution,
                    description: "Must make source code available for at least 3 years".to_string(),
                    severity: ObligationSeverity::High,
                },
            ],
        );

        obligations.insert(
            "AGPL-3.0-or-later".to_string(),
            obligations.get("AGPL-3.0-only").unwrap().clone(),
        );

        obligations.insert(
            "BSD-2-Clause".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright notice and license text".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::NoWarranty,
                    description: "Software provided without warranty".to_string(),
                    severity: ObligationSeverity::Low,
                },
            ],
        );

        obligations.insert(
            "BSD-3-Clause".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright notice and license text".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::Trademark,
                    description: "Cannot use names of copyright holders to endorse derived products".to_string(),
                    severity: ObligationSeverity::Low,
                },
                Obligation {
                    obligation_type: ObligationType::NoWarranty,
                    description: "Software provided without warranty".to_string(),
                    severity: ObligationSeverity::Low,
                },
            ],
        );

        obligations.insert(
            "MPL-2.0".to_string(),
            vec![
                Obligation {
                    obligation_type: ObligationType::Disclosure,
                    description: "Must make source code of modified MPL files available".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::Attribution,
                    description: "Must include copyright and license notices".to_string(),
                    severity: ObligationSeverity::Medium,
                },
                Obligation {
                    obligation_type: ObligationType::PatentGrant,
                    description: "Grants patent license".to_string(),
                    severity: ObligationSeverity::Low,
                },
            ],
        );
    }

    pub fn get(&self, spdx_id: &str) -> Option<&Vec<Obligation>> {
        self.obligations.get(spdx_id)
    }

    pub fn has_high_severity_obligations(&self, spdx_id: &str) -> bool {
        self.get(spdx_id)
            .map(|obls| {
                obls.iter()
                    .any(|obl| obl.severity == ObligationSeverity::High)
            })
            .unwrap_or(false)
    }

    pub fn get_by_type(
        &self,
        spdx_id: &str,
        obligation_type: ObligationType,
    ) -> Vec<&Obligation> {
        self.get(spdx_id)
            .map(|obls| {
                obls.iter()
                    .filter(|obl| obl.obligation_type == obligation_type)
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for LicenseObligations {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_mit_obligations() {
        let obligations = LicenseObligations::new();
        let mit_obls = obligations.get("MIT").unwrap();
        assert_eq!(mit_obls.len(), 2);
        assert!(mit_obls
            .iter()
            .any(|o| o.obligation_type == ObligationType::Attribution));
    }

    #[test]
    fn test_get_apache_obligations() {
        let obligations = LicenseObligations::new();
        let apache_obls = obligations.get("Apache-2.0").unwrap();
        assert!(apache_obls.len() >= 3);
        assert!(apache_obls
            .iter()
            .any(|o| o.obligation_type == ObligationType::PatentGrant));
    }

    #[test]
    fn test_get_gpl_obligations() {
        let obligations = LicenseObligations::new();
        let gpl_obls = obligations.get("GPL-3.0-only").unwrap();
        assert!(gpl_obls
            .iter()
            .any(|o| o.obligation_type == ObligationType::Disclosure));
        assert!(gpl_obls
            .iter()
            .any(|o| o.obligation_type == ObligationType::Copyleft));
    }

    #[test]
    fn test_get_agpl_obligations() {
        let obligations = LicenseObligations::new();
        let agpl_obls = obligations.get("AGPL-3.0-only").unwrap();
        assert!(agpl_obls
            .iter()
            .any(|o| o.obligation_type == ObligationType::NetworkUse));
    }

    #[test]
    fn test_get_nonexistent_license() {
        let obligations = LicenseObligations::new();
        assert!(obligations.get("NonExistent").is_none());
    }

    #[test]
    fn test_has_high_severity_obligations() {
        let obligations = LicenseObligations::new();
        assert!(!obligations.has_high_severity_obligations("MIT"));
        assert!(obligations.has_high_severity_obligations("GPL-3.0-only"));
        assert!(obligations.has_high_severity_obligations("AGPL-3.0-only"));
    }

    #[test]
    fn test_get_by_type() {
        let obligations = LicenseObligations::new();
        let disclosures = obligations.get_by_type("GPL-3.0-only", ObligationType::Disclosure);
        assert_eq!(disclosures.len(), 1);
        assert!(disclosures[0]
            .description
            .contains("source code"));
    }

    #[test]
    fn test_get_by_type_not_found() {
        let obligations = LicenseObligations::new();
        let network = obligations.get_by_type("MIT", ObligationType::NetworkUse);
        assert_eq!(network.len(), 0);
    }

    #[test]
    fn test_obligation_serialization() {
        let obl = Obligation {
            obligation_type: ObligationType::Attribution,
            description: "Test".to_string(),
            severity: ObligationSeverity::Medium,
        };

        let json = serde_json::to_string(&obl).unwrap();
        assert!(json.contains("ATTRIBUTION"));
        assert!(json.contains("MEDIUM"));
    }
}
