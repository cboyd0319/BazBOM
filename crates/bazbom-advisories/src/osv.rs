use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::{Vulnerability, AffectedPackage, VersionRange, VersionEvent};

const OSV_API_BASE: &str = "https://api.osv.dev/v1";

#[derive(Debug, Serialize, Deserialize)]
struct OsvQueryRequest {
    package: OsvPackage,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvQueryResponse {
    vulns: Vec<OsvVulnerability>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvVulnerability {
    id: String,
    #[serde(default)]
    aliases: Vec<String>,
    summary: Option<String>,
    details: Option<String>,
    affected: Vec<OsvAffected>,
    references: Option<Vec<OsvReference>>,
    published: Option<String>,
    modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvAffected {
    package: OsvPackageInfo,
    ranges: Option<Vec<OsvRange>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvPackageInfo {
    name: String,
    ecosystem: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvRange {
    #[serde(rename = "type")]
    range_type: String,
    events: Vec<OsvEvent>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvEvent {
    introduced: Option<String>,
    fixed: Option<String>,
    last_affected: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvReference {
    #[serde(rename = "type")]
    ref_type: String,
    url: String,
}

/// Query OSV database for vulnerabilities affecting a specific package
pub fn query_package_vulnerabilities(
    package_name: &str,
    ecosystem: &str,
    offline: bool,
) -> Result<Vec<Vulnerability>> {
    if offline {
        return Ok(Vec::new());
    }

    let request = OsvQueryRequest {
        package: OsvPackage {
            name: package_name.to_string(),
            ecosystem: ecosystem.to_string(),
        },
    };

    let url = format!("{}/query", OSV_API_BASE);
    let response = ureq::post(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send_json(&request)
        .context("OSV API request failed")?;

    let osv_response: OsvQueryResponse = response
        .into_json()
        .context("failed to parse OSV response")?;

    Ok(osv_response
        .vulns
        .into_iter()
        .map(convert_osv_to_vulnerability)
        .collect())
}

/// Download vulnerabilities for multiple packages in batch
pub fn query_batch_vulnerabilities(
    packages: &[(String, String)], // (name, ecosystem) pairs
    offline: bool,
    cache_dir: &Path,
) -> Result<HashMap<String, Vec<Vulnerability>>> {
    let mut results = HashMap::new();

    if offline {
        // Try to load from cache
        return load_cached_vulnerabilities(cache_dir);
    }

    println!("[bazbom] querying OSV for {} packages", packages.len());

    for (i, (name, ecosystem)) in packages.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            println!("[bazbom]   progress: {}/{}", i, packages.len());
            // Rate limiting: small delay every 10 requests
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        match query_package_vulnerabilities(name, ecosystem, offline) {
            Ok(vulns) => {
                let key = format!("{}:{}", ecosystem, name);
                if !vulns.is_empty() {
                    println!("[bazbom]     {} vulnerabilities for {}", vulns.len(), name);
                    results.insert(key.clone(), vulns.clone());
                    
                    // Cache the results
                    if let Err(e) = cache_vulnerabilities(cache_dir, &key, &vulns) {
                        eprintln!("[bazbom]   warning: failed to cache results: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("[bazbom]   warning: failed to query {}: {}", name, e);
            }
        }
    }

    println!("[bazbom] OSV query complete: {} packages with vulnerabilities", results.len());
    Ok(results)
}

/// Convert OSV format to our internal Vulnerability format
fn convert_osv_to_vulnerability(osv: OsvVulnerability) -> Vulnerability {
    let affected: Vec<AffectedPackage> = osv
        .affected
        .into_iter()
        .map(|aff| {
            let ranges: Vec<VersionRange> = aff
                .ranges
                .unwrap_or_default()
                .into_iter()
                .map(|r| {
                    let events: Vec<VersionEvent> = r
                        .events
                        .into_iter()
                        .filter_map(|e| {
                            if let Some(introduced) = e.introduced {
                                Some(VersionEvent::Introduced { introduced })
                            } else if let Some(fixed) = e.fixed {
                                Some(VersionEvent::Fixed { fixed })
                            } else if let Some(last_affected) = e.last_affected {
                                Some(VersionEvent::LastAffected { last_affected })
                            } else {
                                None
                            }
                        })
                        .collect();

                    VersionRange {
                        range_type: r.range_type,
                        events,
                    }
                })
                .collect();

            AffectedPackage {
                ecosystem: aff.package.ecosystem,
                package: aff.package.name,
                ranges,
            }
        })
        .collect();

    let references = osv
        .references
        .unwrap_or_default()
        .into_iter()
        .map(|r| crate::merge::Reference {
            ref_type: r.ref_type,
            url: r.url,
        })
        .collect();

    Vulnerability {
        id: osv.id,
        aliases: osv.aliases,
        affected,
        severity: None, // OSV doesn't provide severity directly, would need to parse from database_specific
        summary: osv.summary,
        details: osv.details,
        references,
        published: osv.published,
        modified: osv.modified,
        epss: None, // Filled in by enrichment
        kev: None,  // Filled in by enrichment
        priority: None, // Calculated later
    }
}

/// Cache vulnerabilities to disk
fn cache_vulnerabilities(cache_dir: &Path, key: &str, vulns: &[Vulnerability]) -> Result<()> {
    let osv_cache = cache_dir.join("osv");
    fs::create_dir_all(&osv_cache)?;
    
    // Sanitize key for filesystem
    let safe_key = key.replace('/', "_").replace(':', "_");
    let cache_file = osv_cache.join(format!("{}.json", safe_key));
    
    let json = serde_json::to_string_pretty(vulns)?;
    fs::write(cache_file, json)?;
    
    Ok(())
}

/// Load cached vulnerabilities from disk
fn load_cached_vulnerabilities(cache_dir: &Path) -> Result<HashMap<String, Vec<Vulnerability>>> {
    let mut results = HashMap::new();
    let osv_cache = cache_dir.join("osv");
    
    if !osv_cache.exists() {
        return Ok(results);
    }
    
    for entry in fs::read_dir(osv_cache)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(vulns) = serde_json::from_str::<Vec<Vulnerability>>(&content) {
                    // Extract key from filename
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        let key = filename.replace('_', ":");
                        results.insert(key, vulns);
                    }
                }
            }
        }
    }
    
    println!("[bazbom] loaded {} cached vulnerability entries", results.len());
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_convert_osv_to_vulnerability() {
        let osv = OsvVulnerability {
            id: "GHSA-1234-5678-9abc".to_string(),
            aliases: vec!["CVE-2024-1234".to_string()],
            summary: Some("Test vulnerability".to_string()),
            details: Some("Detailed description".to_string()),
            affected: vec![OsvAffected {
                package: OsvPackageInfo {
                    name: "test-package".to_string(),
                    ecosystem: "Maven".to_string(),
                },
                ranges: Some(vec![OsvRange {
                    range_type: "SEMVER".to_string(),
                    events: vec![
                        OsvEvent {
                            introduced: Some("1.0.0".to_string()),
                            fixed: None,
                            last_affected: None,
                        },
                        OsvEvent {
                            introduced: None,
                            fixed: Some("2.0.0".to_string()),
                            last_affected: None,
                        },
                    ],
                }]),
            }],
            references: Some(vec![OsvReference {
                ref_type: "ADVISORY".to_string(),
                url: "https://example.com/advisory".to_string(),
            }]),
            published: Some("2024-01-01".to_string()),
            modified: Some("2024-01-02".to_string()),
        };

        let vuln = convert_osv_to_vulnerability(osv);
        assert_eq!(vuln.id, "GHSA-1234-5678-9abc");
        assert_eq!(vuln.aliases.len(), 1);
        assert_eq!(vuln.affected.len(), 1);
        assert_eq!(vuln.references.len(), 1);
    }

    #[test]
    fn test_cache_and_load_vulnerabilities() -> Result<()> {
        let temp = tempdir()?;
        let cache_dir = temp.path();

        let vuln = Vulnerability {
            id: "TEST-001".to_string(),
            aliases: vec![],
            affected: vec![],
            severity: None,
            summary: Some("Test".to_string()),
            details: None,
            references: vec![],
            published: None,
            modified: None,
            epss: None,
            kev: None,
            priority: None,
        };

        cache_vulnerabilities(cache_dir, "maven:test-pkg", &[vuln.clone()])?;

        let loaded = load_cached_vulnerabilities(cache_dir)?;
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key("maven:test-pkg"));

        Ok(())
    }

    #[test]
    fn test_load_empty_cache() -> Result<()> {
        let temp = tempdir()?;
        let cache_dir = temp.path();

        let loaded = load_cached_vulnerabilities(cache_dir)?;
        assert_eq!(loaded.len(), 0);

        Ok(())
    }
}
