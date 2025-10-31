use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    pub spdx_id: String,
    pub name: String,
    pub category: LicenseCategory,
    pub is_osi_approved: bool,
    pub is_fsf_libre: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LicenseCategory {
    Permissive,
    Copyleft,
    StrongCopyleft,
    Proprietary,
    Unknown,
}

pub struct LicenseDetector {
    licenses: HashMap<String, LicenseInfo>,
}

impl LicenseDetector {
    pub fn new() -> Self {
        let mut licenses = HashMap::new();
        
        Self::populate_common_licenses(&mut licenses);
        
        Self { licenses }
    }

    fn populate_common_licenses(licenses: &mut HashMap<String, LicenseInfo>) {
        let common_licenses = vec![
            LicenseInfo {
                spdx_id: "MIT".to_string(),
                name: "MIT License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "Apache-2.0".to_string(),
                name: "Apache License 2.0".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "BSD-2-Clause".to_string(),
                name: "BSD 2-Clause License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "BSD-3-Clause".to_string(),
                name: "BSD 3-Clause License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "ISC".to_string(),
                name: "ISC License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "0BSD".to_string(),
                name: "BSD Zero Clause License".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "GPL-2.0-only".to_string(),
                name: "GNU General Public License v2.0 only".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "GPL-2.0-or-later".to_string(),
                name: "GNU General Public License v2.0 or later".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "GPL-3.0-only".to_string(),
                name: "GNU General Public License v3.0 only".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "GPL-3.0-or-later".to_string(),
                name: "GNU General Public License v3.0 or later".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "LGPL-2.1-only".to_string(),
                name: "GNU Lesser General Public License v2.1 only".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "LGPL-3.0-only".to_string(),
                name: "GNU Lesser General Public License v3.0 only".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "AGPL-3.0-only".to_string(),
                name: "GNU Affero General Public License v3.0 only".to_string(),
                category: LicenseCategory::StrongCopyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "AGPL-3.0-or-later".to_string(),
                name: "GNU Affero General Public License v3.0 or later".to_string(),
                category: LicenseCategory::StrongCopyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "MPL-2.0".to_string(),
                name: "Mozilla Public License 2.0".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "EPL-2.0".to_string(),
                name: "Eclipse Public License 2.0".to_string(),
                category: LicenseCategory::Copyleft,
                is_osi_approved: true,
                is_fsf_libre: false,
            },
            LicenseInfo {
                spdx_id: "CC0-1.0".to_string(),
                name: "Creative Commons Zero v1.0 Universal".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: false,
                is_fsf_libre: true,
            },
            LicenseInfo {
                spdx_id: "Unlicense".to_string(),
                name: "The Unlicense".to_string(),
                category: LicenseCategory::Permissive,
                is_osi_approved: false,
                is_fsf_libre: true,
            },
        ];

        for license in common_licenses {
            licenses.insert(license.spdx_id.clone(), license);
        }
    }

    pub fn detect(&self, spdx_id: &str) -> Option<LicenseInfo> {
        self.licenses.get(spdx_id).cloned()
    }

    pub fn detect_from_pom_name(&self, pom_license_name: &str) -> Option<String> {
        match pom_license_name {
            "The Apache Software License, Version 2.0" => Some("Apache-2.0".to_string()),
            "Apache License, Version 2.0" => Some("Apache-2.0".to_string()),
            "MIT License" => Some("MIT".to_string()),
            "The MIT License" => Some("MIT".to_string()),
            "BSD 3-Clause License" => Some("BSD-3-Clause".to_string()),
            "BSD 2-Clause License" => Some("BSD-2-Clause".to_string()),
            "ISC License" => Some("ISC".to_string()),
            "Eclipse Public License 2.0" => Some("EPL-2.0".to_string()),
            "Mozilla Public License 2.0" => Some("MPL-2.0".to_string()),
            "GNU General Public License, version 2" => Some("GPL-2.0-only".to_string()),
            "GNU General Public License, version 3" => Some("GPL-3.0-only".to_string()),
            _ => None,
        }
    }

    pub fn is_copyleft(&self, spdx_id: &str) -> bool {
        self.detect(spdx_id)
            .map(|info| {
                info.category == LicenseCategory::Copyleft
                    || info.category == LicenseCategory::StrongCopyleft
            })
            .unwrap_or(false)
    }

    pub fn list_all_licenses(&self) -> Vec<&LicenseInfo> {
        self.licenses.values().collect()
    }
}

impl Default for LicenseDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_mit() {
        let detector = LicenseDetector::new();
        let info = detector.detect("MIT").unwrap();
        assert_eq!(info.spdx_id, "MIT");
        assert_eq!(info.category, LicenseCategory::Permissive);
        assert!(info.is_osi_approved);
    }

    #[test]
    fn test_detect_gpl() {
        let detector = LicenseDetector::new();
        let info = detector.detect("GPL-3.0-only").unwrap();
        assert_eq!(info.spdx_id, "GPL-3.0-only");
        assert_eq!(info.category, LicenseCategory::Copyleft);
    }

    #[test]
    fn test_detect_agpl() {
        let detector = LicenseDetector::new();
        let info = detector.detect("AGPL-3.0-only").unwrap();
        assert_eq!(info.spdx_id, "AGPL-3.0-only");
        assert_eq!(info.category, LicenseCategory::StrongCopyleft);
    }

    #[test]
    fn test_detect_not_found() {
        let detector = LicenseDetector::new();
        let info = detector.detect("NonExistent");
        assert!(info.is_none());
    }

    #[test]
    fn test_detect_from_pom_name() {
        let detector = LicenseDetector::new();
        assert_eq!(
            detector.detect_from_pom_name("The Apache Software License, Version 2.0"),
            Some("Apache-2.0".to_string())
        );
        assert_eq!(
            detector.detect_from_pom_name("MIT License"),
            Some("MIT".to_string())
        );
    }

    #[test]
    fn test_is_copyleft() {
        let detector = LicenseDetector::new();
        assert!(!detector.is_copyleft("MIT"));
        assert!(!detector.is_copyleft("Apache-2.0"));
        assert!(detector.is_copyleft("GPL-3.0-only"));
        assert!(detector.is_copyleft("AGPL-3.0-only"));
    }

    #[test]
    fn test_list_all_licenses() {
        let detector = LicenseDetector::new();
        let licenses = detector.list_all_licenses();
        assert!(!licenses.is_empty());
        assert!(licenses.iter().any(|l| l.spdx_id == "MIT"));
        assert!(licenses.iter().any(|l| l.spdx_id == "GPL-3.0-only"));
    }
}
