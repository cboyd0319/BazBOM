use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum LicenseRisk {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContaminationWarning {
    pub message: String,
    pub affected_licenses: Vec<String>,
    pub risk: LicenseRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub license: String,
}

pub struct LicenseCompatibility;

impl LicenseCompatibility {
    pub fn is_compatible(project_license: &str, dependency_license: &str) -> LicenseRisk {
        use LicenseRisk::*;

        match (project_license, dependency_license) {
            (proj, dep) if proj == dep => Safe,
            
            ("MIT", "MIT") => Safe,
            ("MIT", "Apache-2.0") => Safe,
            ("MIT", "BSD-2-Clause") => Safe,
            ("MIT", "BSD-3-Clause") => Safe,
            ("MIT", "ISC") => Safe,
            ("MIT", "0BSD") => Safe,
            ("MIT", "CC0-1.0") => Safe,
            ("MIT", "Unlicense") => Safe,
            
            ("MIT", "GPL-2.0-only") => Critical,
            ("MIT", "GPL-2.0-or-later") => Critical,
            ("MIT", "GPL-3.0-only") => Critical,
            ("MIT", "GPL-3.0-or-later") => Critical,
            ("MIT", "AGPL-3.0-only") => Critical,
            ("MIT", "AGPL-3.0-or-later") => Critical,
            
            ("Apache-2.0", "MIT") => Safe,
            ("Apache-2.0", "Apache-2.0") => Safe,
            ("Apache-2.0", "BSD-2-Clause") => Safe,
            ("Apache-2.0", "BSD-3-Clause") => Safe,
            ("Apache-2.0", "ISC") => Safe,
            ("Apache-2.0", "0BSD") => Safe,
            
            ("Apache-2.0", "GPL-2.0-only") => High,
            ("Apache-2.0", "GPL-2.0-or-later") => High,
            ("Apache-2.0", "GPL-3.0-only") => Medium,
            ("Apache-2.0", "GPL-3.0-or-later") => Medium,
            ("Apache-2.0", "AGPL-3.0-only") => High,
            ("Apache-2.0", "AGPL-3.0-or-later") => High,
            
            ("GPL-3.0-only", _) => Safe,
            ("GPL-3.0-or-later", _) => Safe,
            ("AGPL-3.0-only", _) => Safe,
            ("AGPL-3.0-or-later", _) => Safe,
            
            ("BSD-2-Clause", "MIT") => Safe,
            ("BSD-2-Clause", "Apache-2.0") => Safe,
            ("BSD-2-Clause", "BSD-3-Clause") => Safe,
            ("BSD-2-Clause", "ISC") => Safe,
            ("BSD-2-Clause", "GPL-3.0-only") => Critical,
            ("BSD-2-Clause", "AGPL-3.0-only") => Critical,
            
            ("BSD-3-Clause", "MIT") => Safe,
            ("BSD-3-Clause", "Apache-2.0") => Safe,
            ("BSD-3-Clause", "BSD-2-Clause") => Safe,
            ("BSD-3-Clause", "ISC") => Safe,
            ("BSD-3-Clause", "GPL-3.0-only") => Critical,
            ("BSD-3-Clause", "AGPL-3.0-only") => Critical,
            
            (_, "NOASSERTION") => High,
            (_, "Unknown") => High,
            (_, "NONE") => High,
            
            (_, _) => Medium,
        }
    }

    pub fn check_contamination(dependencies: &[Dependency]) -> Vec<ContaminationWarning> {
        let mut warnings = Vec::new();

        let copyleft_deps: Vec<_> = dependencies
            .iter()
            .filter(|d| Self::is_copyleft(&d.license))
            .collect();

        if !copyleft_deps.is_empty() {
            let licenses: Vec<String> = copyleft_deps.iter().map(|d| d.license.clone()).collect();
            warnings.push(ContaminationWarning {
                message: format!(
                    "Found {} copyleft dependencies. Your entire project may be subject to copyleft terms.",
                    copyleft_deps.len()
                ),
                affected_licenses: licenses,
                risk: LicenseRisk::High,
            });
        }

        let strong_copyleft_deps: Vec<_> = dependencies
            .iter()
            .filter(|d| Self::is_strong_copyleft(&d.license))
            .collect();

        if !strong_copyleft_deps.is_empty() {
            let licenses: Vec<String> = strong_copyleft_deps
                .iter()
                .map(|d| d.license.clone())
                .collect();
            warnings.push(ContaminationWarning {
                message: format!(
                    "Found {} strong copyleft (AGPL) dependencies. Network use may trigger source disclosure.",
                    strong_copyleft_deps.len()
                ),
                affected_licenses: licenses,
                risk: LicenseRisk::Critical,
            });
        }

        let unknown_deps: Vec<_> = dependencies
            .iter()
            .filter(|d| Self::is_unknown(&d.license))
            .collect();

        if !unknown_deps.is_empty() {
            warnings.push(ContaminationWarning {
                message: format!(
                    "Found {} dependencies with unknown licenses. Legal risk unclear.",
                    unknown_deps.len()
                ),
                affected_licenses: unknown_deps.iter().map(|d| d.license.clone()).collect(),
                risk: LicenseRisk::High,
            });
        }

        warnings
    }

    fn is_copyleft(license: &str) -> bool {
        matches!(
            license,
            "GPL-2.0-only"
                | "GPL-2.0-or-later"
                | "GPL-3.0-only"
                | "GPL-3.0-or-later"
                | "LGPL-2.1-only"
                | "LGPL-3.0-only"
                | "AGPL-3.0-only"
                | "AGPL-3.0-or-later"
                | "MPL-2.0"
                | "EPL-2.0"
        )
    }

    fn is_strong_copyleft(license: &str) -> bool {
        matches!(license, "AGPL-3.0-only" | "AGPL-3.0-or-later")
    }

    fn is_unknown(license: &str) -> bool {
        matches!(license, "Unknown" | "NOASSERTION" | "NONE" | "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mit_compatibility() {
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "MIT"),
            LicenseRisk::Safe
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "Apache-2.0"),
            LicenseRisk::Safe
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "GPL-3.0-only"),
            LicenseRisk::Critical
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "AGPL-3.0-only"),
            LicenseRisk::Critical
        );
    }

    #[test]
    fn test_apache_compatibility() {
        assert_eq!(
            LicenseCompatibility::is_compatible("Apache-2.0", "MIT"),
            LicenseRisk::Safe
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("Apache-2.0", "GPL-3.0-only"),
            LicenseRisk::Medium
        );
    }

    #[test]
    fn test_gpl_compatibility() {
        assert_eq!(
            LicenseCompatibility::is_compatible("GPL-3.0-only", "MIT"),
            LicenseRisk::Safe
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("GPL-3.0-only", "Apache-2.0"),
            LicenseRisk::Safe
        );
    }

    #[test]
    fn test_unknown_license() {
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "NOASSERTION"),
            LicenseRisk::High
        );
        assert_eq!(
            LicenseCompatibility::is_compatible("MIT", "Unknown"),
            LicenseRisk::High
        );
    }

    #[test]
    fn test_check_contamination_copyleft() {
        let deps = vec![
            Dependency {
                name: "lib1".to_string(),
                license: "MIT".to_string(),
            },
            Dependency {
                name: "lib2".to_string(),
                license: "GPL-3.0-only".to_string(),
            },
        ];

        let warnings = LicenseCompatibility::check_contamination(&deps);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("copyleft"));
        assert_eq!(warnings[0].risk, LicenseRisk::High);
    }

    #[test]
    fn test_check_contamination_strong_copyleft() {
        let deps = vec![
            Dependency {
                name: "lib1".to_string(),
                license: "MIT".to_string(),
            },
            Dependency {
                name: "lib2".to_string(),
                license: "AGPL-3.0-only".to_string(),
            },
        ];

        let warnings = LicenseCompatibility::check_contamination(&deps);
        assert!(warnings.len() >= 1);
        let agpl_warning = warnings.iter().find(|w| w.risk == LicenseRisk::Critical);
        assert!(agpl_warning.is_some());
        assert!(agpl_warning.unwrap().message.contains("AGPL"));
    }

    #[test]
    fn test_check_contamination_unknown() {
        let deps = vec![
            Dependency {
                name: "lib1".to_string(),
                license: "MIT".to_string(),
            },
            Dependency {
                name: "lib2".to_string(),
                license: "Unknown".to_string(),
            },
        ];

        let warnings = LicenseCompatibility::check_contamination(&deps);
        assert!(warnings.iter().any(|w| w.message.contains("unknown")));
    }

    #[test]
    fn test_check_contamination_no_issues() {
        let deps = vec![
            Dependency {
                name: "lib1".to_string(),
                license: "MIT".to_string(),
            },
            Dependency {
                name: "lib2".to_string(),
                license: "Apache-2.0".to_string(),
            },
        ];

        let warnings = LicenseCompatibility::check_contamination(&deps);
        assert_eq!(warnings.len(), 0);
    }
}
