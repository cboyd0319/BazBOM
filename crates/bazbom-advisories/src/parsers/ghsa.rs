use crate::merge::{
    AffectedPackage, Reference, Severity, SeverityLevel, VersionEvent, VersionRange, Vulnerability,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// GitHub Security Advisory (GHSA) format structures
/// Based on GitHub's GraphQL API schema
#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaEntry {
    pub id: String,
    #[serde(rename = "ghsaId")]
    pub ghsa_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub identifiers: Vec<GhsaIdentifier>,
    #[serde(default)]
    pub references: Vec<GhsaReference>,
    #[serde(default)]
    pub vulnerabilities: Vec<GhsaVulnerability>,
    pub cvss: Option<GhsaCvss>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaIdentifier {
    #[serde(rename = "type")]
    pub id_type: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaReference {
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaVulnerability {
    pub package: GhsaPackage,
    #[serde(rename = "vulnerableVersionRange")]
    pub vulnerable_version_range: Option<String>,
    #[serde(rename = "firstPatchedVersion")]
    pub first_patched_version: Option<GhsaVersion>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaPackage {
    pub ecosystem: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaVersion {
    pub identifier: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GhsaCvss {
    pub score: Option<f64>,
    #[serde(rename = "vectorString")]
    pub vector_string: Option<String>,
}

/// Parse a GHSA entry into our canonical Vulnerability format
pub fn parse_ghsa_entry(ghsa: &GhsaEntry) -> Result<Vulnerability> {
    // Extract ID (prefer ghsaId, fall back to id)
    let id = ghsa
        .ghsa_id
        .clone()
        .unwrap_or_else(|| ghsa.id.clone());

    // Extract aliases from identifiers
    let aliases: Vec<String> = ghsa
        .identifiers
        .iter()
        .filter(|ident| ident.id_type == "CVE")
        .map(|ident| ident.value.clone())
        .collect();

    // Parse affected packages
    let affected_packages: Vec<AffectedPackage> = ghsa
        .vulnerabilities
        .iter()
        .map(|v| {
            let ranges = parse_ghsa_version_range(
                &v.vulnerable_version_range,
                &v.first_patched_version,
            );
            AffectedPackage {
                ecosystem: v.package.ecosystem.clone(),
                package: v.package.name.clone(),
                ranges,
            }
        })
        .collect();

    // Parse severity
    let severity = parse_ghsa_severity(&ghsa.severity, &ghsa.cvss);

    // Parse references
    let references = ghsa
        .references
        .iter()
        .map(|r| Reference {
            ref_type: "ADVISORY".to_string(),
            url: r.url.clone(),
        })
        .collect();

    Ok(Vulnerability {
        id,
        aliases,
        affected: affected_packages,
        severity,
        summary: ghsa.summary.clone(),
        details: ghsa.description.clone(),
        references,
        published: ghsa.published_at.clone(),
        modified: ghsa.updated_at.clone(),
        epss: None,  // EPSS enrichment happens separately
        kev: None,   // KEV enrichment happens separately
        priority: None, // Priority calculated after enrichment
    })
}

fn parse_ghsa_severity(severity_str: &Option<String>, cvss: &Option<GhsaCvss>) -> Option<Severity> {
    // Try to get CVSS score first
    if let Some(cvss_data) = cvss {
        if let Some(score) = cvss_data.score {
            let level = cvss_to_severity_level(score);
            return Some(Severity {
                cvss_v3: Some(score),
                cvss_v4: None,
                level,
            });
        }
    }

    // Fall back to severity string
    if let Some(sev_str) = severity_str {
        let level = match sev_str.to_uppercase().as_str() {
            "CRITICAL" => SeverityLevel::Critical,
            "HIGH" => SeverityLevel::High,
            "MEDIUM" | "MODERATE" => SeverityLevel::Medium,
            "LOW" => SeverityLevel::Low,
            _ => SeverityLevel::Unknown,
        };
        return Some(Severity {
            cvss_v3: None,
            cvss_v4: None,
            level,
        });
    }

    None
}

fn parse_ghsa_version_range(
    range_str: &Option<String>,
    first_patched: &Option<GhsaVersion>,
) -> Vec<VersionRange> {
    let mut events = Vec::new();

    // Parse range string (e.g., ">= 1.0.0, < 2.0.0")
    if let Some(range) = range_str {
        // Simple parsing - this is a simplified version
        // In production, we'd use a proper version range parser
        if range.contains(">=") || range.starts_with('<') {
            events.push(VersionEvent::Introduced {
                introduced: "0".to_string(),
            });
        }
    } else {
        // If no range specified, assume all versions before fix
        events.push(VersionEvent::Introduced {
            introduced: "0".to_string(),
        });
    }

    // Add fixed version if available
    if let Some(patched) = first_patched {
        events.push(VersionEvent::Fixed {
            fixed: patched.identifier.clone(),
        });
    }

    if events.is_empty() {
        vec![]
    } else {
        vec![VersionRange {
            range_type: "ECOSYSTEM".to_string(),
            events,
        }]
    }
}

fn cvss_to_severity_level(score: f64) -> SeverityLevel {
    if score >= 9.0 {
        SeverityLevel::Critical
    } else if score >= 7.0 {
        SeverityLevel::High
    } else if score >= 4.0 {
        SeverityLevel::Medium
    } else if score >= 0.1 {
        SeverityLevel::Low
    } else {
        SeverityLevel::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ghsa_entry_basic() {
        let json = r#"{
            "id": "GHSA-xxxx-yyyy-zzzz",
            "ghsaId": "GHSA-xxxx-yyyy-zzzz",
            "summary": "Test vulnerability",
            "description": "Detailed description",
            "severity": "HIGH",
            "publishedAt": "2024-01-10T10:00:00Z",
            "updatedAt": "2024-01-15T20:15:00Z",
            "identifiers": [
                {
                    "type": "GHSA",
                    "value": "GHSA-xxxx-yyyy-zzzz"
                },
                {
                    "type": "CVE",
                    "value": "CVE-2024-1234"
                }
            ],
            "references": [
                {
                    "url": "https://github.com/advisories/GHSA-xxxx-yyyy-zzzz"
                }
            ],
            "vulnerabilities": [
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:vulnerable"
                    },
                    "vulnerableVersionRange": ">= 1.0.0, < 2.0.0",
                    "firstPatchedVersion": {
                        "identifier": "2.0.0"
                    }
                }
            ]
        }"#;

        let ghsa_entry: GhsaEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_ghsa_entry(&ghsa_entry).unwrap();

        assert_eq!(vuln.id, "GHSA-xxxx-yyyy-zzzz");
        assert_eq!(vuln.aliases.len(), 1);
        assert_eq!(vuln.aliases[0], "CVE-2024-1234");
        assert_eq!(vuln.affected.len(), 1);
        assert_eq!(vuln.affected[0].ecosystem, "Maven");
        assert_eq!(vuln.affected[0].package, "com.example:vulnerable");
    }

    #[test]
    fn test_parse_ghsa_entry_with_cvss() {
        let json = r#"{
            "id": "GHSA-test-cvss",
            "summary": "Test with CVSS",
            "identifiers": [],
            "references": [],
            "vulnerabilities": [],
            "cvss": {
                "score": 9.8,
                "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"
            }
        }"#;

        let ghsa_entry: GhsaEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_ghsa_entry(&ghsa_entry).unwrap();

        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.cvss_v3, Some(9.8));
        assert_eq!(severity.level, SeverityLevel::Critical);
    }

    #[test]
    fn test_parse_ghsa_entry_severity_string_only() {
        let json = r#"{
            "id": "GHSA-test-sev",
            "summary": "Test with severity string",
            "severity": "MEDIUM",
            "identifiers": [],
            "references": [],
            "vulnerabilities": []
        }"#;

        let ghsa_entry: GhsaEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_ghsa_entry(&ghsa_entry).unwrap();

        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.level, SeverityLevel::Medium);
        assert!(severity.cvss_v3.is_none());
    }

    #[test]
    fn test_cvss_to_severity_level() {
        assert_eq!(cvss_to_severity_level(10.0), SeverityLevel::Critical);
        assert_eq!(cvss_to_severity_level(9.0), SeverityLevel::Critical);
        assert_eq!(cvss_to_severity_level(8.0), SeverityLevel::High);
        assert_eq!(cvss_to_severity_level(7.0), SeverityLevel::High);
        assert_eq!(cvss_to_severity_level(5.0), SeverityLevel::Medium);
        assert_eq!(cvss_to_severity_level(4.0), SeverityLevel::Medium);
        assert_eq!(cvss_to_severity_level(2.0), SeverityLevel::Low);
        assert_eq!(cvss_to_severity_level(0.1), SeverityLevel::Low);
        assert_eq!(cvss_to_severity_level(0.0), SeverityLevel::Unknown);
    }

    #[test]
    fn test_parse_ghsa_entry_multiple_vulnerabilities() {
        let json = r#"{
            "id": "GHSA-multi",
            "summary": "Multiple affected packages",
            "identifiers": [],
            "references": [],
            "vulnerabilities": [
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:pkg1"
                    },
                    "firstPatchedVersion": {
                        "identifier": "1.0.0"
                    }
                },
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:pkg2"
                    },
                    "firstPatchedVersion": {
                        "identifier": "2.5.0"
                    }
                }
            ]
        }"#;

        let ghsa_entry: GhsaEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_ghsa_entry(&ghsa_entry).unwrap();

        assert_eq!(vuln.affected.len(), 2);
        assert_eq!(vuln.affected[0].package, "com.example:pkg1");
        assert_eq!(vuln.affected[1].package, "com.example:pkg2");
    }
}
