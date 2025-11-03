use crate::merge::{
    AffectedPackage, Reference, Severity, SeverityLevel, VersionEvent, VersionRange, Vulnerability,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// OSV format structures based on https://ossf.github.io/osv-schema/
#[derive(Debug, Deserialize, Serialize)]
pub struct OsvEntry {
    pub id: String,
    pub modified: Option<String>,
    pub published: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub summary: Option<String>,
    pub details: Option<String>,
    #[serde(default)]
    pub affected: Vec<OsvAffected>,
    #[serde(default)]
    pub references: Vec<OsvReference>,
    pub database_specific: Option<serde_json::Value>,
    pub severity: Option<Vec<OsvSeverity>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsvAffected {
    pub package: OsvPackage,
    #[serde(default)]
    pub ranges: Vec<OsvRange>,
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsvPackage {
    pub ecosystem: String,
    pub name: String,
    pub purl: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsvRange {
    #[serde(rename = "type")]
    pub range_type: String,
    #[serde(default)]
    pub events: Vec<OsvEvent>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum OsvEvent {
    Introduced { introduced: String },
    Fixed { fixed: String },
    LastAffected { last_affected: String },
    Limit { limit: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsvReference {
    #[serde(rename = "type")]
    pub ref_type: String,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OsvSeverity {
    #[serde(rename = "type")]
    pub severity_type: String,
    pub score: String,
}

/// Parse an OSV entry into our canonical Vulnerability format
pub fn parse_osv_entry(osv: &OsvEntry) -> Result<Vulnerability> {
    // Parse affected packages
    let mut affected_packages = Vec::new();
    for aff in &osv.affected {
        let mut ranges = Vec::new();
        for r in &aff.ranges {
            let mut events = Vec::new();
            for e in &r.events {
                let event = match e {
                    OsvEvent::Introduced { introduced } => VersionEvent::Introduced {
                        introduced: introduced.clone(),
                    },
                    OsvEvent::Fixed { fixed } => VersionEvent::Fixed {
                        fixed: fixed.clone(),
                    },
                    OsvEvent::LastAffected { last_affected } => VersionEvent::LastAffected {
                        last_affected: last_affected.clone(),
                    },
                    OsvEvent::Limit { limit: _ } => continue, // Skip limit events for now
                };
                events.push(event);
            }
            ranges.push(VersionRange {
                range_type: r.range_type.clone(),
                events,
            });
        }

        affected_packages.push(AffectedPackage {
            ecosystem: aff.package.ecosystem.clone(),
            package: aff.package.name.clone(),
            ranges,
        });
    }

    // Parse severity from CVSS if available
    let severity = parse_osv_severity(&osv.severity, &osv.database_specific);

    // Parse references
    let references = osv
        .references
        .iter()
        .map(|r| Reference {
            ref_type: r.ref_type.clone(),
            url: r.url.clone(),
        })
        .collect();

    Ok(Vulnerability {
        id: osv.id.clone(),
        aliases: osv.aliases.clone(),
        affected: affected_packages,
        severity,
        summary: osv.summary.clone(),
        details: osv.details.clone(),
        references,
        published: osv.published.clone(),
        modified: osv.modified.clone(),
        epss: None,     // EPSS enrichment happens separately
        kev: None,      // KEV enrichment happens separately
        priority: None, // Priority calculated after enrichment
    })
}

fn parse_osv_severity(
    severity_list: &Option<Vec<OsvSeverity>>,
    database_specific: &Option<serde_json::Value>,
) -> Option<Severity> {
    // Try to extract CVSS score from severity list
    if let Some(severities) = severity_list {
        for sev in severities {
            if sev.severity_type == "CVSS_V3" {
                // Parse CVSS vector string to extract score
                if let Some(score) = extract_cvss_score(&sev.score) {
                    let level = cvss_to_severity_level(score);
                    return Some(Severity {
                        cvss_v3: Some(score),
                        cvss_v4: None,
                        level,
                    });
                }
            } else if sev.severity_type == "CVSS_V4" {
                if let Some(score) = extract_cvss_score(&sev.score) {
                    let level = cvss_to_severity_level(score);
                    return Some(Severity {
                        cvss_v3: None,
                        cvss_v4: Some(score),
                        level,
                    });
                }
            }
        }
    }

    // Fallback: check database_specific for severity string
    if let Some(db_spec) = database_specific {
        if let Some(sev_str) = db_spec.get("severity").and_then(|v| v.as_str()) {
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
    }

    None
}

fn extract_cvss_score(cvss_string: &str) -> Option<f64> {
    // CVSS vector format: "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H"
    // Or just a numeric score: "9.8"

    // Try direct numeric parse first
    if let Ok(score) = cvss_string.parse::<f64>() {
        return Some(score);
    }

    // Try to extract base score from vector string
    // For now, we'll use a simple heuristic - in real implementation,
    // we'd use a proper CVSS parser library
    None
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
    fn test_parse_osv_entry_basic() {
        let json = r#"{
            "id": "GHSA-xxxx-yyyy-zzzz",
            "modified": "2024-01-15T20:15:00Z",
            "published": "2024-01-10T10:00:00Z",
            "aliases": ["CVE-2024-1234"],
            "summary": "Test vulnerability",
            "details": "Detailed description",
            "affected": [
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:vulnerable"
                    },
                    "ranges": [
                        {
                            "type": "ECOSYSTEM",
                            "events": [
                                {"introduced": "0"},
                                {"fixed": "1.2.3"}
                            ]
                        }
                    ]
                }
            ],
            "references": [
                {
                    "type": "ADVISORY",
                    "url": "https://example.com/advisory"
                }
            ]
        }"#;

        let osv_entry: OsvEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_osv_entry(&osv_entry).unwrap();

        assert_eq!(vuln.id, "GHSA-xxxx-yyyy-zzzz");
        assert_eq!(vuln.aliases.len(), 1);
        assert_eq!(vuln.aliases[0], "CVE-2024-1234");
        assert_eq!(vuln.affected.len(), 1);
        assert_eq!(vuln.affected[0].ecosystem, "Maven");
        assert_eq!(vuln.affected[0].package, "com.example:vulnerable");
        assert_eq!(vuln.references.len(), 1);
    }

    #[test]
    fn test_parse_osv_entry_with_cvss() {
        let json = r#"{
            "id": "GHSA-test-cvss",
            "summary": "Test with CVSS",
            "affected": [],
            "references": [],
            "severity": [
                {
                    "type": "CVSS_V3",
                    "score": "9.8"
                }
            ]
        }"#;

        let osv_entry: OsvEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_osv_entry(&osv_entry).unwrap();

        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.cvss_v3, Some(9.8));
        assert_eq!(severity.level, SeverityLevel::Critical);
    }

    #[test]
    fn test_parse_osv_entry_with_database_specific_severity() {
        let json = r#"{
            "id": "GHSA-test-db-sev",
            "summary": "Test with db severity",
            "affected": [],
            "references": [],
            "database_specific": {
                "severity": "HIGH"
            }
        }"#;

        let osv_entry: OsvEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_osv_entry(&osv_entry).unwrap();

        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.level, SeverityLevel::High);
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
    fn test_parse_osv_entry_multiple_affected() {
        let json = r#"{
            "id": "GHSA-multi",
            "summary": "Multiple affected packages",
            "affected": [
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:pkg1"
                    },
                    "ranges": [
                        {
                            "type": "ECOSYSTEM",
                            "events": [
                                {"introduced": "0"},
                                {"fixed": "1.0.0"}
                            ]
                        }
                    ]
                },
                {
                    "package": {
                        "ecosystem": "Maven",
                        "name": "com.example:pkg2"
                    },
                    "ranges": [
                        {
                            "type": "ECOSYSTEM",
                            "events": [
                                {"introduced": "2.0.0"},
                                {"fixed": "2.5.0"}
                            ]
                        }
                    ]
                }
            ],
            "references": []
        }"#;

        let osv_entry: OsvEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_osv_entry(&osv_entry).unwrap();

        assert_eq!(vuln.affected.len(), 2);
        assert_eq!(vuln.affected[0].package, "com.example:pkg1");
        assert_eq!(vuln.affected[1].package, "com.example:pkg2");
    }
}
