use crate::merge::{
    AffectedPackage, Reference, Severity, SeverityLevel, VersionEvent, VersionRange, Vulnerability,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// NVD API 2.0 format structures
/// Based on https://nvd.nist.gov/developers/vulnerabilities
#[derive(Debug, Deserialize, Serialize)]
pub struct NvdEntry {
    pub cve: NvdCve,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdCve {
    pub id: String,
    #[serde(rename = "sourceIdentifier")]
    pub source_identifier: Option<String>,
    pub published: Option<String>,
    #[serde(rename = "lastModified")]
    pub last_modified: Option<String>,
    #[serde(rename = "vulnStatus")]
    pub vuln_status: Option<String>,
    pub descriptions: Option<Vec<NvdDescription>>,
    pub metrics: Option<NvdMetrics>,
    pub references: Option<Vec<NvdReference>>,
    pub configurations: Option<Vec<NvdConfiguration>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdDescription {
    pub lang: String,
    pub value: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdMetrics {
    #[serde(rename = "cvssMetricV31")]
    pub cvss_metric_v31: Option<Vec<NvdCvssMetric>>,
    #[serde(rename = "cvssMetricV30")]
    pub cvss_metric_v30: Option<Vec<NvdCvssMetric>>,
    #[serde(rename = "cvssMetricV40")]
    pub cvss_metric_v40: Option<Vec<NvdCvssMetric>>,
    #[serde(rename = "cvssMetricV2")]
    pub cvss_metric_v2: Option<Vec<NvdCvssMetric>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdCvssMetric {
    pub source: Option<String>,
    #[serde(rename = "type")]
    pub metric_type: Option<String>,
    #[serde(rename = "cvssData")]
    pub cvss_data: NvdCvssData,
    #[serde(rename = "baseSeverity")]
    pub base_severity: Option<String>,
    #[serde(rename = "exploitabilityScore")]
    pub exploitability_score: Option<f64>,
    #[serde(rename = "impactScore")]
    pub impact_score: Option<f64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdCvssData {
    pub version: String,
    #[serde(rename = "vectorString")]
    pub vector_string: String,
    #[serde(rename = "baseScore")]
    pub base_score: f64,
    #[serde(rename = "baseSeverity")]
    pub base_severity: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdReference {
    pub url: String,
    pub source: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdConfiguration {
    pub nodes: Option<Vec<NvdNode>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdNode {
    pub operator: Option<String>,
    #[serde(rename = "cpeMatch")]
    pub cpe_match: Option<Vec<NvdCpeMatch>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NvdCpeMatch {
    pub vulnerable: bool,
    pub criteria: String,
    #[serde(rename = "versionStartIncluding")]
    pub version_start_including: Option<String>,
    #[serde(rename = "versionStartExcluding")]
    pub version_start_excluding: Option<String>,
    #[serde(rename = "versionEndIncluding")]
    pub version_end_including: Option<String>,
    #[serde(rename = "versionEndExcluding")]
    pub version_end_excluding: Option<String>,
}

/// Parse an NVD entry into our canonical Vulnerability format
pub fn parse_nvd_entry(nvd: &NvdEntry) -> Result<Vulnerability> {
    let cve = &nvd.cve;

    // Extract description (prefer English)
    let description = cve
        .descriptions
        .as_ref()
        .and_then(|descs| {
            descs
                .iter()
                .find(|d| d.lang == "en")
                .or_else(|| descs.first())
        })
        .map(|d| d.value.clone());

    // Parse affected packages from CPE configurations
    let affected_packages = parse_nvd_configurations(&cve.configurations);

    // Parse severity from CVSS metrics
    let severity = parse_nvd_severity(&cve.metrics);

    // Parse references
    let references = cve
        .references
        .as_ref()
        .map(|refs| {
            refs.iter()
                .map(|r| {
                    let ref_type = r
                        .tags
                        .as_ref()
                        .and_then(|tags| tags.first())
                        .cloned()
                        .unwrap_or_else(|| "WEB".to_string());
                    Reference {
                        ref_type,
                        url: r.url.clone(),
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Vulnerability {
        id: cve.id.clone(),
        aliases: vec![], // NVD entries don't typically have aliases in the data
        affected: affected_packages,
        severity,
        summary: None, // NVD doesn't have separate summary
        details: description,
        references,
        published: cve.published.clone(),
        modified: cve.last_modified.clone(),
        epss: None,     // EPSS enrichment happens separately
        kev: None,      // KEV enrichment happens separately
        priority: None, // Priority calculated after enrichment
    })
}

fn parse_nvd_severity(metrics: &Option<NvdMetrics>) -> Option<Severity> {
    if let Some(m) = metrics {
        // Prefer v3.1, then v4.0, then v3.0, then v2
        if let Some(cvss_v31) = &m.cvss_metric_v31 {
            if let Some(metric) = cvss_v31.first() {
                let score = metric.cvss_data.base_score;
                let level = cvss_to_severity_level(score);
                return Some(Severity {
                    cvss_v3: Some(score),
                    cvss_v4: None,
                    level,
                });
            }
        }

        if let Some(cvss_v40) = &m.cvss_metric_v40 {
            if let Some(metric) = cvss_v40.first() {
                let score = metric.cvss_data.base_score;
                let level = cvss_to_severity_level(score);
                return Some(Severity {
                    cvss_v3: None,
                    cvss_v4: Some(score),
                    level,
                });
            }
        }

        if let Some(cvss_v30) = &m.cvss_metric_v30 {
            if let Some(metric) = cvss_v30.first() {
                let score = metric.cvss_data.base_score;
                let level = cvss_to_severity_level(score);
                return Some(Severity {
                    cvss_v3: Some(score),
                    cvss_v4: None,
                    level,
                });
            }
        }

        // Fall back to CVSS v2 if no v3/v4 available
        if let Some(cvss_v2) = &m.cvss_metric_v2 {
            if let Some(metric) = cvss_v2.first() {
                let score = metric.cvss_data.base_score;
                let level = cvss_to_severity_level(score);
                return Some(Severity {
                    cvss_v3: Some(score), // Store v2 score in v3 field for simplicity
                    cvss_v4: None,
                    level,
                });
            }
        }
    }
    None
}

fn parse_nvd_configurations(configs: &Option<Vec<NvdConfiguration>>) -> Vec<AffectedPackage> {
    let mut affected = Vec::new();

    if let Some(configs) = configs {
        for config in configs {
            if let Some(nodes) = &config.nodes {
                for node in nodes {
                    if let Some(cpe_matches) = &node.cpe_match {
                        for cpe in cpe_matches {
                            if cpe.vulnerable {
                                if let Some(pkg) = parse_cpe_to_package(&cpe.criteria, cpe) {
                                    affected.push(pkg);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    affected
}

fn parse_cpe_to_package(cpe: &str, cpe_match: &NvdCpeMatch) -> Option<AffectedPackage> {
    // CPE format: cpe:2.3:a:vendor:product:version:...
    // We'll parse this to extract vendor:product for Maven-like ecosystems
    let parts: Vec<&str> = cpe.split(':').collect();
    if parts.len() < 5 {
        return None;
    }

    let vendor = parts[3];
    let product = parts[4];

    // Build version range from CPE match constraints
    let mut events = Vec::new();

    if let Some(start_inc) = &cpe_match.version_start_including {
        events.push(VersionEvent::Introduced {
            introduced: start_inc.clone(),
        });
    } else if let Some(start_exc) = &cpe_match.version_start_excluding {
        // For excluding, we approximate by adding as introduced
        events.push(VersionEvent::Introduced {
            introduced: start_exc.clone(),
        });
    }

    if let Some(end_exc) = &cpe_match.version_end_excluding {
        events.push(VersionEvent::Fixed {
            fixed: end_exc.clone(),
        });
    } else if let Some(end_inc) = &cpe_match.version_end_including {
        events.push(VersionEvent::LastAffected {
            last_affected: end_inc.clone(),
        });
    }

    // If we have version constraints, create a range
    if !events.is_empty() {
        Some(AffectedPackage {
            ecosystem: "CPE".to_string(), // Mark as CPE, can be refined later
            package: format!("{}:{}", vendor, product),
            ranges: vec![VersionRange {
                range_type: "ECOSYSTEM".to_string(),
                events,
            }],
        })
    } else {
        None
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
    fn test_parse_nvd_entry_basic() {
        let json = r#"{
            "cve": {
                "id": "CVE-2024-1234",
                "published": "2024-01-10T10:00:00.000",
                "lastModified": "2024-01-15T20:15:00.000",
                "descriptions": [
                    {
                        "lang": "en",
                        "value": "Test vulnerability description"
                    }
                ],
                "metrics": {
                    "cvssMetricV31": [
                        {
                            "source": "nvd@nist.gov",
                            "cvssData": {
                                "version": "3.1",
                                "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
                                "baseScore": 9.8,
                                "baseSeverity": "CRITICAL"
                            }
                        }
                    ]
                },
                "references": [
                    {
                        "url": "https://example.com/advisory",
                        "tags": ["Vendor Advisory"]
                    }
                ]
            }
        }"#;

        let nvd_entry: NvdEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_nvd_entry(&nvd_entry).unwrap();

        assert_eq!(vuln.id, "CVE-2024-1234");
        assert!(vuln.details.is_some());
        assert_eq!(
            vuln.details.as_ref().unwrap(),
            "Test vulnerability description"
        );
        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.cvss_v3, Some(9.8));
        assert_eq!(severity.level, SeverityLevel::Critical);
    }

    #[test]
    fn test_parse_nvd_severity_v30() {
        let json = r#"{
            "cve": {
                "id": "CVE-2024-TEST",
                "metrics": {
                    "cvssMetricV30": [
                        {
                            "cvssData": {
                                "version": "3.0",
                                "vectorString": "CVSS:3.0/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
                                "baseScore": 8.5
                            }
                        }
                    ]
                }
            }
        }"#;

        let nvd_entry: NvdEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_nvd_entry(&nvd_entry).unwrap();

        assert!(vuln.severity.is_some());
        let severity = vuln.severity.unwrap();
        assert_eq!(severity.cvss_v3, Some(8.5));
        assert_eq!(severity.level, SeverityLevel::High);
    }

    #[test]
    fn test_parse_cpe_to_package() {
        let cpe = "cpe:2.3:a:apache:log4j:2.14.1:*:*:*:*:*:*:*";
        let cpe_match = NvdCpeMatch {
            vulnerable: true,
            criteria: cpe.to_string(),
            version_start_including: Some("2.0.0".to_string()),
            version_start_excluding: None,
            version_end_excluding: Some("2.17.0".to_string()),
            version_end_including: None,
        };

        let pkg = parse_cpe_to_package(cpe, &cpe_match);
        assert!(pkg.is_some());

        let pkg = pkg.unwrap();
        assert_eq!(pkg.ecosystem, "CPE");
        assert_eq!(pkg.package, "apache:log4j");
        assert_eq!(pkg.ranges.len(), 1);
        assert_eq!(pkg.ranges[0].events.len(), 2);
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
    fn test_parse_nvd_entry_with_multiple_references() {
        let json = r#"{
            "cve": {
                "id": "CVE-2024-TEST2",
                "references": [
                    {
                        "url": "https://example.com/advisory1",
                        "tags": ["Vendor Advisory"]
                    },
                    {
                        "url": "https://example.com/advisory2",
                        "tags": ["Patch"]
                    }
                ]
            }
        }"#;

        let nvd_entry: NvdEntry = serde_json::from_str(json).unwrap();
        let vuln = parse_nvd_entry(&nvd_entry).unwrap();

        assert_eq!(vuln.references.len(), 2);
        assert_eq!(vuln.references[0].ref_type, "Vendor Advisory");
        assert_eq!(vuln.references[1].ref_type, "Patch");
    }
}
