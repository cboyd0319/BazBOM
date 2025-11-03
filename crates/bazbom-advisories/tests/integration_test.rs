/// Integration tests for the complete advisory pipeline
/// Tests parsing, enrichment, merging, and priority calculation
use bazbom_advisories::{
    calculate_priority, load_epss_scores, load_kev_catalog, merge_vulnerabilities,
    parse_ghsa_entry, parse_nvd_entry, parse_osv_entry,
};
use std::io::Write;
use tempfile::NamedTempFile;

/// Test complete OSV parsing, enrichment, and priority calculation
#[test]
fn test_osv_complete_pipeline() {
    let osv_json = r#"{
        "id": "GHSA-xxxx-yyyy-zzzz",
        "modified": "2024-01-15T20:15:00Z",
        "published": "2024-01-10T10:00:00Z",
        "aliases": ["CVE-2024-1234"],
        "summary": "Critical vulnerability in example package",
        "details": "This is a critical security vulnerability.",
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
        ],
        "severity": [
            {
                "type": "CVSS_V3",
                "score": "9.8"
            }
        ]
    }"#;

    let osv_entry: bazbom_advisories::parsers::osv::OsvEntry =
        serde_json::from_str(osv_json).unwrap();
    let mut vuln = parse_osv_entry(&osv_entry).unwrap();

    assert_eq!(vuln.id, "GHSA-xxxx-yyyy-zzzz");
    assert_eq!(vuln.aliases.len(), 1);
    assert_eq!(vuln.affected.len(), 1);

    // Create KEV catalog
    let kev_json = r#"{
        "title": "CISA Catalog",
        "vulnerabilities": [
            {
                "cveID": "CVE-2024-1234",
                "vendorProject": "Example",
                "product": "Package",
                "vulnerabilityName": "Critical Bug",
                "dateAdded": "2024-01-10",
                "requiredAction": "Apply patch",
                "dueDate": "2024-02-10"
            }
        ]
    }"#;

    let mut kev_file = NamedTempFile::new().unwrap();
    kev_file.write_all(kev_json.as_bytes()).unwrap();
    kev_file.flush().unwrap();
    let kev_map = load_kev_catalog(kev_file.path()).unwrap();

    // Create EPSS scores
    let epss_csv = "cve,epss,percentile\nCVE-2024-1234,0.95,0.99\n";
    let mut epss_file = NamedTempFile::new().unwrap();
    epss_file.write_all(epss_csv.as_bytes()).unwrap();
    epss_file.flush().unwrap();
    let epss_map = load_epss_scores(epss_file.path()).unwrap();

    // Enrich vulnerability
    vuln.kev =
        bazbom_advisories::enrichment::kev::find_kev_entry(&vuln.id, &vuln.aliases, &kev_map);
    vuln.epss =
        bazbom_advisories::enrichment::epss::find_epss_score(&vuln.id, &vuln.aliases, &epss_map);

    assert!(vuln.kev.is_some());
    assert!(vuln.epss.is_some());

    // Calculate priority
    let priority = calculate_priority(&vuln.severity, &vuln.kev, &vuln.epss);
    vuln.priority = Some(priority);

    // Should be P0 due to KEV + high CVSS
    assert_eq!(vuln.priority.unwrap(), bazbom_advisories::Priority::P0);
}

/// Test NVD parsing and enrichment
#[test]
fn test_nvd_complete_pipeline() {
    let nvd_json = r#"{
        "cve": {
            "id": "CVE-2024-5678",
            "published": "2024-01-10T10:00:00.000",
            "descriptions": [
                {
                    "lang": "en",
                    "value": "High severity vulnerability"
                }
            ],
            "metrics": {
                "cvssMetricV31": [
                    {
                        "cvssData": {
                            "version": "3.1",
                            "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
                            "baseScore": 8.5
                        }
                    }
                ]
            },
            "references": [
                {
                    "url": "https://example.com/advisory"
                }
            ]
        }
    }"#;

    let nvd_entry: bazbom_advisories::parsers::nvd::NvdEntry =
        serde_json::from_str(nvd_json).unwrap();
    let mut vuln = parse_nvd_entry(&nvd_entry).unwrap();

    assert_eq!(vuln.id, "CVE-2024-5678");
    assert!(vuln.severity.is_some());

    // Create EPSS scores (no KEV for this one)
    let epss_csv = "cve,epss,percentile\nCVE-2024-5678,0.6,0.95\n";
    let mut epss_file = NamedTempFile::new().unwrap();
    epss_file.write_all(epss_csv.as_bytes()).unwrap();
    epss_file.flush().unwrap();
    let epss_map = load_epss_scores(epss_file.path()).unwrap();

    // Enrich with EPSS
    vuln.epss =
        bazbom_advisories::enrichment::epss::find_epss_score(&vuln.id, &vuln.aliases, &epss_map);

    // Calculate priority
    let priority = calculate_priority(&vuln.severity, &vuln.kev, &vuln.epss);
    vuln.priority = Some(priority);

    // Should be P1 due to high CVSS (8.5) and high EPSS (0.6)
    assert_eq!(vuln.priority.unwrap(), bazbom_advisories::Priority::P1);
}

/// Test GHSA parsing and enrichment
#[test]
fn test_ghsa_complete_pipeline() {
    let ghsa_json = r#"{
        "id": "GHSA-abcd-efgh-ijkl",
        "ghsaId": "GHSA-abcd-efgh-ijkl",
        "summary": "Medium severity vulnerability",
        "severity": "MEDIUM",
        "identifiers": [
            {
                "type": "CVE",
                "value": "CVE-2024-9999"
            }
        ],
        "references": [],
        "vulnerabilities": [
            {
                "package": {
                    "ecosystem": "Maven",
                    "name": "com.example:package"
                },
                "firstPatchedVersion": {
                    "identifier": "2.0.0"
                }
            }
        ],
        "cvss": {
            "score": 5.5
        }
    }"#;

    let ghsa_entry: bazbom_advisories::parsers::ghsa::GhsaEntry =
        serde_json::from_str(ghsa_json).unwrap();
    let mut vuln = parse_ghsa_entry(&ghsa_entry).unwrap();

    assert_eq!(vuln.id, "GHSA-abcd-efgh-ijkl");
    assert_eq!(vuln.aliases.len(), 1);
    assert_eq!(vuln.aliases[0], "CVE-2024-9999");

    // Calculate priority (no KEV, no EPSS)
    let priority = calculate_priority(&vuln.severity, &vuln.kev, &vuln.epss);
    vuln.priority = Some(priority);

    // Should be P3 due to medium CVSS (5.5)
    assert_eq!(vuln.priority.unwrap(), bazbom_advisories::Priority::P3);
}

/// Test merging vulnerabilities from multiple sources
#[test]
fn test_merge_multiple_sources() {
    // OSV entry
    let osv_json = r#"{
        "id": "GHSA-test-test-test",
        "aliases": ["CVE-2024-TEST"],
        "summary": "Test vulnerability from OSV",
        "affected": [
            {
                "package": {
                    "ecosystem": "Maven",
                    "name": "com.example:test"
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
            }
        ],
        "references": [
            {
                "type": "ADVISORY",
                "url": "https://osv.dev/advisory"
            }
        ]
    }"#;

    // NVD entry for same CVE
    let nvd_json = r#"{
        "cve": {
            "id": "CVE-2024-TEST",
            "descriptions": [
                {
                    "lang": "en",
                    "value": "Detailed description from NVD - much longer and more comprehensive"
                }
            ],
            "metrics": {
                "cvssMetricV31": [
                    {
                        "cvssData": {
                            "version": "3.1",
                            "vectorString": "CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H",
                            "baseScore": 9.8
                        }
                    }
                ]
            },
            "references": [
                {
                    "url": "https://nvd.nist.gov/vuln/detail/CVE-2024-TEST"
                }
            ]
        }
    }"#;

    let osv_entry: bazbom_advisories::parsers::osv::OsvEntry =
        serde_json::from_str(osv_json).unwrap();
    let nvd_entry: bazbom_advisories::parsers::nvd::NvdEntry =
        serde_json::from_str(nvd_json).unwrap();

    let vuln1 = parse_osv_entry(&osv_entry).unwrap();
    let vuln2 = parse_nvd_entry(&nvd_entry).unwrap();

    // Merge vulnerabilities
    let merged = merge_vulnerabilities(vec![vuln1, vuln2]);

    // Check merged data
    assert!(merged.aliases.contains(&"CVE-2024-TEST".to_string()));
    assert!(merged.aliases.contains(&"GHSA-test-test-test".to_string()));

    // Should take the longer/better description from NVD
    assert!(merged.details.is_some());
    assert!(merged.details.as_ref().unwrap().contains("NVD"));

    // Should take the highest severity (from NVD)
    assert!(merged.severity.is_some());
    assert_eq!(merged.severity.as_ref().unwrap().cvss_v3, Some(9.8));

    // Should have references from both sources
    assert!(merged.references.len() >= 2);
}

/// Test complete enrichment workflow with priority calculation
#[test]
fn test_complete_enrichment_workflow() {
    let vulnerabilities = vec![
        // P0: Critical CVSS + KEV
        (9.8, true, 0.5, bazbom_advisories::Priority::P0),
        // P1: High CVSS + High EPSS
        (8.0, false, 0.6, bazbom_advisories::Priority::P1),
        // P2: High CVSS, no KEV, low EPSS
        (7.5, false, 0.05, bazbom_advisories::Priority::P2),
        // P3: Medium CVSS
        (5.0, false, 0.0, bazbom_advisories::Priority::P3),
        // P4: Low CVSS
        (2.0, false, 0.0, bazbom_advisories::Priority::P4),
    ];

    for (cvss, has_kev, epss_score, expected_priority) in vulnerabilities {
        let severity = Some(bazbom_advisories::Severity {
            cvss_v3: Some(cvss),
            cvss_v4: None,
            level: if cvss >= 9.0 {
                bazbom_advisories::SeverityLevel::Critical
            } else if cvss >= 7.0 {
                bazbom_advisories::SeverityLevel::High
            } else if cvss >= 4.0 {
                bazbom_advisories::SeverityLevel::Medium
            } else {
                bazbom_advisories::SeverityLevel::Low
            },
        });

        let kev = if has_kev {
            Some(bazbom_advisories::KevEntry {
                cve_id: "CVE-TEST".to_string(),
                vendor_project: "Test".to_string(),
                product: "Test".to_string(),
                vulnerability_name: "Test".to_string(),
                date_added: "2024-01-01".to_string(),
                required_action: "Patch".to_string(),
                due_date: "2024-02-01".to_string(),
            })
        } else {
            None
        };

        let epss = if epss_score > 0.0 {
            Some(bazbom_advisories::EpssScore {
                score: epss_score,
                percentile: 0.9,
            })
        } else {
            None
        };

        let priority = calculate_priority(&severity, &kev, &epss);
        assert_eq!(
            priority, expected_priority,
            "Failed for CVSS={}, KEV={}, EPSS={}",
            cvss, has_kev, epss_score
        );
    }
}
