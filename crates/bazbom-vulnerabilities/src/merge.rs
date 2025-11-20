use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a vulnerability from any source (OSV, NVD, GHSA)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    /// Primary ID (CVE, GHSA, or OSV ID)
    pub id: String,
    /// All known aliases (CVE, GHSA, OSV IDs)
    pub aliases: Vec<String>,
    /// Affected packages
    pub affected: Vec<AffectedPackage>,
    /// Severity information
    pub severity: Option<Severity>,
    /// Summary description
    pub summary: Option<String>,
    /// Detailed description
    pub details: Option<String>,
    /// References (advisories, patches, etc.)
    pub references: Vec<Reference>,
    /// Published timestamp
    pub published: Option<String>,
    /// Modified timestamp
    pub modified: Option<String>,
    /// EPSS score (probability of exploitation)
    pub epss: Option<EpssScore>,
    /// KEV (Known Exploited Vulnerability) presence
    pub kev: Option<KevEntry>,
    /// Calculated priority (P0-P4)
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedPackage {
    pub ecosystem: String,
    pub package: String,
    pub ranges: Vec<VersionRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRange {
    #[serde(rename = "type")]
    pub range_type: String, // "SEMVER", "ECOSYSTEM", etc.
    pub events: Vec<VersionEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VersionEvent {
    Introduced { introduced: String },
    Fixed { fixed: String },
    LastAffected { last_affected: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Severity {
    /// CVSS v3 score
    pub cvss_v3: Option<f64>,
    /// CVSS v4 score
    pub cvss_v4: Option<f64>,
    /// Canonical severity level (normalized across sources)
    pub level: SeverityLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SeverityLevel {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpssScore {
    /// EPSS probability score (0.0 to 1.0)
    pub score: f64,
    /// Percentile (0.0 to 1.0)
    pub percentile: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KevEntry {
    /// CVE ID
    pub cve_id: String,
    /// Vendor/Project
    pub vendor_project: String,
    /// Product
    pub product: String,
    /// Vulnerability name
    pub vulnerability_name: String,
    /// Date added to KEV catalog
    pub date_added: String,
    /// Required action
    pub required_action: String,
    /// Due date for remediation
    pub due_date: String,
}

/// Priority classification (P0-P4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    /// Critical: KEV present, CVSS ≥ 9.0, or EPSS ≥ 0.9
    P0,
    /// High: CVSS ≥ 7.0 and (KEV or EPSS ≥ 0.5)
    P1,
    /// Medium-High: CVSS ≥ 7.0 or (CVSS ≥ 4.0 and EPSS ≥ 0.1)
    P2,
    /// Medium: CVSS ≥ 4.0
    P3,
    /// Low: CVSS < 4.0 or unknown
    P4,
}

impl std::fmt::Display for Priority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Priority::P0 => write!(f, "P0"),
            Priority::P1 => write!(f, "P1"),
            Priority::P2 => write!(f, "P2"),
            Priority::P3 => write!(f, "P3"),
            Priority::P4 => write!(f, "P4"),
        }
    }
}

impl Priority {
    /// Get human-readable description of the priority level
    pub fn description(&self) -> &'static str {
        match self {
            Priority::P0 => "Critical - Fix immediately",
            Priority::P1 => "High - Fix within 24 hours",
            Priority::P2 => "Medium-High - Fix within 1 week",
            Priority::P3 => "Medium - Fix within 1 month",
            Priority::P4 => "Low - Fix when convenient",
        }
    }
}

/// Merge multiple vulnerability sources into a canonical vulnerability
///
/// # Errors
///
/// Returns an error if the `vulns` vector is empty.
pub fn merge_vulnerabilities(vulns: Vec<Vulnerability>) -> Result<Vulnerability, String> {
    if vulns.is_empty() {
        return Err("Cannot merge empty vulnerability list".to_string());
    }

    // Use the first vulnerability as base
    let mut merged = vulns[0].clone();

    // Collect all aliases
    let mut all_aliases = HashSet::new();
    all_aliases.insert(merged.id.clone());
    for vuln in &vulns {
        all_aliases.insert(vuln.id.clone());
        for alias in &vuln.aliases {
            all_aliases.insert(alias.clone());
        }
    }
    merged.aliases = all_aliases.into_iter().collect();
    merged.aliases.sort();

    // Merge affected packages
    let mut package_map: HashMap<String, AffectedPackage> = HashMap::new();
    for vuln in &vulns {
        for pkg in &vuln.affected {
            let key = format!("{}:{}", pkg.ecosystem, pkg.package);
            package_map.insert(key, pkg.clone());
        }
    }
    merged.affected = package_map.into_values().collect();

    // Take highest severity
    let mut best_severity: Option<Severity> = None;
    for vuln in &vulns {
        if let Some(sev) = &vuln.severity {
            let current_cvss = best_severity
                .as_ref()
                .and_then(|s| s.cvss_v3)
                .unwrap_or(0.0);
            let new_cvss = sev.cvss_v3.unwrap_or(0.0);

            if best_severity.is_none() || current_cvss < new_cvss {
                best_severity = Some(sev.clone());
            }
        }
    }
    merged.severity = best_severity;

    // Use longest/best description
    for vuln in &vulns {
        if let Some(vuln_details) = &vuln.details {
            let should_replace = match &merged.details {
                None => true,
                Some(merged_details) => vuln_details.len() > merged_details.len(),
            };

            if should_replace {
                merged.details = Some(vuln_details.clone());
            }
        }
    }

    // Merge references (dedup by URL)
    let mut ref_map: HashMap<String, Reference> = HashMap::new();
    for vuln in &vulns {
        for r in &vuln.references {
            ref_map.insert(r.url.clone(), r.clone());
        }
    }
    merged.references = ref_map.into_values().collect();

    Ok(merged)
}

/// Calculate priority based on severity, KEV, and EPSS
pub fn calculate_priority(
    severity: &Option<Severity>,
    kev: &Option<KevEntry>,
    epss: &Option<EpssScore>,
) -> Priority {
    let cvss = severity
        .as_ref()
        .and_then(|s| s.cvss_v3.or(s.cvss_v4))
        .unwrap_or(0.0);

    let epss_score = epss.as_ref().map(|e| e.score).unwrap_or(0.0);
    let has_kev = kev.is_some();

    // P0: Critical severity or KEV with high CVSS or very high EPSS
    if has_kev && cvss >= 7.0 || cvss >= 9.0 || epss_score >= 0.9 {
        return Priority::P0;
    }

    // P1: High severity with KEV or high EPSS
    if cvss >= 7.0 && (has_kev || epss_score >= 0.5) {
        return Priority::P1;
    }

    // P2: High severity or medium with notable EPSS
    if cvss >= 7.0 || (cvss >= 4.0 && epss_score >= 0.1) {
        return Priority::P2;
    }

    // P3: Medium severity
    if cvss >= 4.0 {
        return Priority::P3;
    }

    // P4: Low or unknown
    Priority::P4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_vulnerabilities_combines_aliases() {
        let vuln1 = Vulnerability {
            id: "CVE-2024-1234".to_string(),
            aliases: vec!["GHSA-xxxx-yyyy".to_string()],
            affected: vec![],
            severity: None,
            summary: None,
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        };

        let vuln2 = Vulnerability {
            id: "GHSA-xxxx-yyyy".to_string(),
            aliases: vec!["CVE-2024-1234".to_string()],
            affected: vec![],
            severity: None,
            summary: None,
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        };

        let merged = merge_vulnerabilities(vec![vuln1, vuln2]).expect("Should merge successfully");

        assert!(merged.aliases.contains(&"CVE-2024-1234".to_string()));
        assert!(merged.aliases.contains(&"GHSA-xxxx-yyyy".to_string()));
    }

    #[test]
    fn test_calculate_priority_p0_with_kev() {
        let severity = Some(Severity {
            cvss_v3: Some(8.0),
            cvss_v4: None,
            level: SeverityLevel::High,
        });
        let kev = Some(KevEntry {
            cve_id: "CVE-2024-1234".to_string(),
            vendor_project: "Vendor".to_string(),
            product: "Product".to_string(),
            vulnerability_name: "Test".to_string(),
            date_added: "2024-01-01".to_string(),
            required_action: "Patch".to_string(),
            due_date: "2024-02-01".to_string(),
        });
        let epss = None;

        let priority = calculate_priority(&severity, &kev, &epss);
        assert_eq!(priority, Priority::P0);
    }

    #[test]
    fn test_calculate_priority_p0_with_critical_cvss() {
        let severity = Some(Severity {
            cvss_v3: Some(9.5),
            cvss_v4: None,
            level: SeverityLevel::Critical,
        });

        let priority = calculate_priority(&severity, &None, &None);
        assert_eq!(priority, Priority::P0);
    }

    #[test]
    fn test_calculate_priority_p1_high_cvss_with_epss() {
        let severity = Some(Severity {
            cvss_v3: Some(7.5),
            cvss_v4: None,
            level: SeverityLevel::High,
        });
        let epss = Some(EpssScore {
            score: 0.6,
            percentile: 0.95,
        });

        let priority = calculate_priority(&severity, &None, &epss);
        assert_eq!(priority, Priority::P1);
    }

    #[test]
    fn test_calculate_priority_p4_low_cvss() {
        let severity = Some(Severity {
            cvss_v3: Some(2.0),
            cvss_v4: None,
            level: SeverityLevel::Low,
        });

        let priority = calculate_priority(&severity, &None, &None);
        assert_eq!(priority, Priority::P4);
    }

    #[test]
    fn test_severity_level_ordering() {
        assert!(SeverityLevel::Critical > SeverityLevel::High);
        assert!(SeverityLevel::High > SeverityLevel::Medium);
        assert!(SeverityLevel::Medium > SeverityLevel::Low);
        assert!(SeverityLevel::Low > SeverityLevel::Unknown);
    }
}
