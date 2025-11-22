use crate::{AffectedPackage, VersionEvent, VersionRange, Vulnerability};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

const OSV_API_BASE: &str = "https://api.osv.dev/v1";

/// Fetch severity information from OSV API for CVEs with unknown severity
///
/// Queries OSV for each CVE and extracts severity from CVSS scores or database_specific fields.
/// Returns a map of CVE ID to severity string (CRITICAL, HIGH, MEDIUM, LOW).
///
/// The `os_hint` parameter can be used to optimize lookups by trying the relevant
/// OS-specific prefix first (e.g., "alpine", "debian", "ubuntu", "rhel").
pub fn fetch_osv_severities(cve_ids: &[String]) -> HashMap<String, String> {
    fetch_osv_severities_with_hint(cve_ids, None)
}

/// Fetch severity with an optional OS hint for faster lookups
pub fn fetch_osv_severities_with_hint(
    cve_ids: &[String],
    os_hint: Option<&str>,
) -> HashMap<String, String> {
    #[derive(Deserialize)]
    struct OsvSeverityResponse {
        severity: Option<Vec<OsvSeverityEntry>>,
        database_specific: Option<serde_json::Value>,
    }

    #[derive(Deserialize)]
    struct OsvSeverityEntry {
        #[serde(rename = "type")]
        severity_type: String,
        score: String,
    }

    let mut severity_map = HashMap::new();
    let mut remaining_cves: Vec<String> = Vec::new();

    for cve_id in cve_ids {
        // Build ID variants based on OS hint for faster lookups
        // See: https://osv.dev/vulnerability/CVE-2023-42363 for example of many aliases
        let id_variants: Vec<String> = match os_hint.map(|s| s.to_lowercase()).as_deref() {
            Some(os) if os.contains("alpine") => vec![format!("ALPINE-{}", cve_id), cve_id.clone()],
            Some(os) if os.contains("debian") => vec![
                format!("DSA-{}", cve_id), // Debian Security Advisory format
                cve_id.clone(),
            ],
            Some(os) if os.contains("ubuntu") => vec![
                format!("USN-{}", cve_id), // Ubuntu Security Notice format
                cve_id.clone(),
            ],
            Some(os) if os.contains("rhel") || os.contains("centos") || os.contains("fedora") => {
                vec![
                    format!("RHSA-{}", cve_id), // Red Hat Security Advisory format
                    cve_id.clone(),
                ]
            }
            _ => vec![
                cve_id.clone(), // Plain CVE ID first (most common)
            ],
        };

        let mut found = false;
        for variant in &id_variants {
            let url = format!("{}/vulns/{}", OSV_API_BASE, variant);

            let config = ureq::Agent::config_builder()
                .timeout_global(Some(std::time::Duration::from_secs(5)))
                .build();
            let agent: ureq::Agent = config.into();

            match agent.get(&url).call() {
                Ok(response) => {
                    if let Ok(body) = response.into_body().read_to_string() {
                        if let Ok(osv_data) = serde_json::from_str::<OsvSeverityResponse>(&body) {
                            // Try to get severity from CVSS score
                            if let Some(severities) = osv_data.severity {
                                for sev in severities {
                                    if sev.severity_type == "CVSS_V3"
                                        || sev.severity_type == "CVSS_V2"
                                    {
                                        if let Some(score) = parse_cvss_score(&sev.score) {
                                            let severity = cvss_to_severity(score);
                                            severity_map.insert(cve_id.clone(), severity);
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }

                            // Try database_specific.severity as fallback
                            if !found && !severity_map.contains_key(cve_id) {
                                if let Some(db_specific) = osv_data.database_specific {
                                    if let Some(severity) =
                                        db_specific.get("severity").and_then(|s| s.as_str())
                                    {
                                        severity_map
                                            .insert(cve_id.clone(), severity.to_uppercase());
                                        found = true;
                                    }
                                }
                            }
                        }
                    }
                    if found {
                        break;
                    }
                }
                Err(_) => {
                    // Try next variant
                }
            }
        }

        if !found {
            remaining_cves.push(cve_id.clone());
        }
    }

    // NVD API fallback for CVEs not found in OSV
    // Rate limit: 5 requests per 30 seconds without API key
    if !remaining_cves.is_empty() {
        let nvd_results = fetch_nvd_severities(&remaining_cves);
        severity_map.extend(nvd_results);
    }

    severity_map
}

/// Fetch severity from NVD API as fallback
/// Rate limited to 5 requests per 30 seconds (public API limit)
fn fetch_nvd_severities(cve_ids: &[String]) -> HashMap<String, String> {
    #[derive(Deserialize)]
    struct NvdResponse {
        vulnerabilities: Option<Vec<NvdVulnerability>>,
    }

    #[derive(Deserialize)]
    struct NvdVulnerability {
        cve: NvdCve,
    }

    #[derive(Deserialize)]
    struct NvdCve {
        id: String,
        metrics: Option<NvdMetrics>,
    }

    #[derive(Deserialize)]
    struct NvdMetrics {
        #[serde(rename = "cvssMetricV31")]
        cvss_v31: Option<Vec<NvdCvssMetric>>,
        #[serde(rename = "cvssMetricV30")]
        cvss_v30: Option<Vec<NvdCvssMetric>>,
        #[serde(rename = "cvssMetricV2")]
        cvss_v2: Option<Vec<NvdCvssMetric>>,
    }

    #[derive(Deserialize)]
    struct NvdCvssMetric {
        #[serde(rename = "cvssData")]
        cvss_data: NvdCvssData,
    }

    #[derive(Deserialize)]
    struct NvdCvssData {
        #[serde(rename = "baseScore")]
        base_score: f64,
    }

    let mut severity_map = HashMap::new();

    // Process in batches of 5 (NVD rate limit)
    for (batch_idx, chunk) in cve_ids.chunks(5).enumerate() {
        if batch_idx > 0 {
            // Wait 30 seconds between batches to respect rate limit
            std::thread::sleep(std::time::Duration::from_secs(30));
        }

        for cve_id in chunk {
            let url = format!(
                "https://services.nvd.nist.gov/rest/json/cves/2.0?cveId={}",
                cve_id
            );

            let config = ureq::Agent::config_builder()
                .timeout_global(Some(std::time::Duration::from_secs(10)))
                .build();
            let agent: ureq::Agent = config.into();

            match agent.get(&url).call() {
                Ok(mut response) => {
                    if let Ok(nvd_data) = response.body_mut().read_json::<NvdResponse>() {
                        if let Some(vulns) = nvd_data.vulnerabilities {
                            for vuln in vulns {
                                if vuln.cve.id == *cve_id {
                                    if let Some(metrics) = vuln.cve.metrics {
                                        // Try CVSS v3.1 first, then v3.0, then v2
                                        let score = metrics
                                            .cvss_v31
                                            .and_then(|m| m.first().map(|c| c.cvss_data.base_score))
                                            .or_else(|| {
                                                metrics.cvss_v30.and_then(|m| {
                                                    m.first().map(|c| c.cvss_data.base_score)
                                                })
                                            })
                                            .or_else(|| {
                                                metrics.cvss_v2.and_then(|m| {
                                                    m.first().map(|c| c.cvss_data.base_score)
                                                })
                                            });

                                        if let Some(score) = score {
                                            severity_map
                                                .insert(cve_id.clone(), cvss_to_severity(score));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Skip this CVE if NVD lookup fails
                }
            }
        }
    }

    severity_map
}

/// Parse CVSS score from string (number or vector)
///
/// Handles both raw scores ("7.5") and CVSS vectors ("CVSS:3.1/AV:N/AC:L/...").
pub fn parse_cvss_score(cvss_string: &str) -> Option<f64> {
    // If it's just a number, return it
    if let Ok(score) = cvss_string.parse::<f64>() {
        return Some(score);
    }

    // Parse CVSS v3.x vector string
    // Format: CVSS:3.1/AV:N/AC:L/PR:N/UI:N/S:U/C:H/I:H/A:H
    if cvss_string.starts_with("CVSS:3") {
        return parse_cvss_v3_vector(cvss_string);
    }

    None
}

/// Parse CVSS v3 vector string and calculate approximate base score
/// This is a simplified calculation - full CVSS calculation is more complex
fn parse_cvss_v3_vector(vector: &str) -> Option<f64> {
    let mut metrics: std::collections::HashMap<&str, &str> = std::collections::HashMap::new();

    for part in vector.split('/') {
        if let Some((key, value)) = part.split_once(':') {
            metrics.insert(key, value);
        }
    }

    // Impact weights
    let c_impact = match metrics.get("C") {
        Some(&"H") => 0.56,
        Some(&"L") => 0.22,
        _ => 0.0,
    };
    let i_impact = match metrics.get("I") {
        Some(&"H") => 0.56,
        Some(&"L") => 0.22,
        _ => 0.0,
    };
    let a_impact = match metrics.get("A") {
        Some(&"H") => 0.56,
        Some(&"L") => 0.22,
        _ => 0.0,
    };

    // ISS (Impact Sub Score)
    let iss = 1.0 - ((1.0 - c_impact) * (1.0 - i_impact) * (1.0 - a_impact));

    // Scope
    let scope_changed = metrics.get("S") == Some(&"C");

    // Impact
    let impact: f64 = if scope_changed {
        7.52 * (iss - 0.029) - 3.25 * f64::powf(iss - 0.02, 15.0)
    } else {
        6.42 * iss
    };

    if impact <= 0.0 {
        return Some(0.0);
    }

    // Exploitability weights
    let av = match metrics.get("AV") {
        Some(&"N") => 0.85, // Network
        Some(&"A") => 0.62, // Adjacent
        Some(&"L") => 0.55, // Local
        Some(&"P") => 0.20, // Physical
        _ => 0.85,
    };
    let ac = match metrics.get("AC") {
        Some(&"L") => 0.77, // Low
        Some(&"H") => 0.44, // High
        _ => 0.77,
    };
    let pr = match (metrics.get("PR"), scope_changed) {
        (Some(&"N"), _) => 0.85,     // None
        (Some(&"L"), false) => 0.62, // Low, unchanged
        (Some(&"L"), true) => 0.68,  // Low, changed
        (Some(&"H"), false) => 0.27, // High, unchanged
        (Some(&"H"), true) => 0.50,  // High, changed
        _ => 0.85,
    };
    let ui = match metrics.get("UI") {
        Some(&"N") => 0.85, // None
        Some(&"R") => 0.62, // Required
        _ => 0.85,
    };

    let exploitability: f64 = 8.22 * av * ac * pr * ui;

    // Base score
    let base_score: f64 = if scope_changed {
        f64::min(1.08 * (impact + exploitability), 10.0)
    } else {
        f64::min(impact + exploitability, 10.0)
    };

    // Round up to 1 decimal place
    Some((base_score * 10.0).ceil() / 10.0)
}

/// Convert CVSS score to severity string
pub fn cvss_to_severity(score: f64) -> String {
    match score {
        s if s >= 9.0 => "CRITICAL".to_string(),
        s if s >= 7.0 => "HIGH".to_string(),
        s if s >= 4.0 => "MEDIUM".to_string(),
        s if s > 0.0 => "LOW".to_string(),
        _ => "UNKNOWN".to_string(),
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OsvQueryRequest {
    version: String,
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

/// Query OSV database for vulnerabilities affecting a specific package version
pub fn query_package_vulnerabilities(
    package_name: &str,
    version: &str,
    ecosystem: &str,
    offline: bool,
) -> Result<Vec<Vulnerability>> {
    if offline {
        return Ok(Vec::new());
    }

    let request = OsvQueryRequest {
        version: version.to_string(),
        package: OsvPackage {
            name: package_name.to_string(),
            ecosystem: ecosystem.to_string(),
        },
    };

    let url = format!("{}/query", OSV_API_BASE);
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    let agent: ureq::Agent = config.into();
    let mut response = agent
        .post(&url)
        .send_json(&request)
        .context("OSV API request failed")?;

    let osv_response: OsvQueryResponse = response
        .body_mut()
        .read_json()
        .context("failed to parse OSV response")?;

    Ok(osv_response
        .vulns
        .into_iter()
        .map(convert_osv_to_vulnerability)
        .collect())
}

/// Query OSV by vulnerability ID and return fixed versions for a package
///
/// Returns a list of (ecosystem, package, fixed_version) tuples
pub fn get_fixed_versions(vuln_id: &str) -> Result<Vec<(String, String, String)>> {
    let url = format!("{}/vulns/{}", OSV_API_BASE, vuln_id);
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(10)))
        .build();
    let agent: ureq::Agent = config.into();

    let mut response = match agent.get(&url).call() {
        Ok(r) => r,
        Err(e) => {
            // Return empty if not found
            if e.to_string().contains("404") {
                return Ok(Vec::new());
            }
            return Err(anyhow::anyhow!("OSV API request failed: {}", e));
        }
    };

    let osv: OsvVulnerability = response
        .body_mut()
        .read_json()
        .context("failed to parse OSV vulnerability")?;

    let mut fixed_versions = Vec::new();

    for affected in osv.affected {
        let ecosystem = affected.package.ecosystem.clone();
        let package = affected.package.name.clone();

        if let Some(ranges) = affected.ranges {
            for range in ranges {
                for event in range.events {
                    if let Some(fixed) = event.fixed {
                        fixed_versions.push((ecosystem.clone(), package.clone(), fixed));
                    }
                }
            }
        }
    }

    Ok(fixed_versions)
}

/// Get the first fixed version for a specific package from a vulnerability
pub fn get_fixed_version_for_package(vuln_id: &str, package_name: &str) -> Result<Option<String>> {
    let fixed = get_fixed_versions(vuln_id)?;

    // Find the first fixed version for this package
    for (_, pkg, version) in fixed {
        if pkg == package_name || pkg.ends_with(&format!("/{}", package_name)) {
            return Ok(Some(version));
        }
    }

    Ok(None)
}

/// Download vulnerabilities for multiple packages in batch
pub fn query_batch_vulnerabilities(
    packages: &[(String, String, String)], // (name, version, ecosystem) tuples
    offline: bool,
    cache_dir: &Path,
) -> Result<HashMap<String, Vec<Vulnerability>>> {
    let mut results = HashMap::new();

    if offline {
        // Try to load from cache
        return load_cached_vulnerabilities(cache_dir);
    }

    println!("[bazbom] querying OSV for {} packages", packages.len());

    for (i, (name, version, ecosystem)) in packages.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            println!("[bazbom]   progress: {}/{}", i, packages.len());
            // Simple rate limiting: small delay every 10 requests
            // Note: This is intentionally simple. For production use with many packages,
            // consider implementing proper async with a token bucket rate limiter.
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        match query_package_vulnerabilities(name, version, ecosystem, offline) {
            Ok(vulns) => {
                let key = format!("{}:{}@{}", ecosystem, name, version);
                if !vulns.is_empty() {
                    println!(
                        "[bazbom]     {} vulnerabilities for {}@{}",
                        vulns.len(),
                        name,
                        version
                    );
                    results.insert(key.clone(), vulns.clone());

                    // Cache the results
                    if let Err(e) = cache_vulnerabilities(cache_dir, &key, &vulns) {
                        eprintln!("[bazbom]   warning: failed to cache results: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[bazbom]   warning: failed to query {}@{}: {}",
                    name, version, e
                );
            }
        }
    }

    println!(
        "[bazbom] OSV query complete: {} packages with vulnerabilities",
        results.len()
    );
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
                            e.introduced
                                .map(|introduced| VersionEvent::Introduced { introduced })
                                .or_else(|| e.fixed.map(|fixed| VersionEvent::Fixed { fixed }))
                                .or_else(|| {
                                    e.last_affected.map(|last_affected| {
                                        VersionEvent::LastAffected { last_affected }
                                    })
                                })
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
        epss: None,     // Filled in by enrichment
        kev: None,      // Filled in by enrichment
        priority: None, // Calculated later
    }
}

/// Encode key for safe filesystem use
fn encode_cache_key(key: &str) -> String {
    let mut encoded = String::new();
    for ch in key.chars() {
        match ch {
            '/' => encoded.push_str("_SLASH_"),
            ':' => encoded.push_str("_COLON_"),
            '_' => encoded.push_str("_UNDER_"),
            c => encoded.push(c),
        }
    }
    encoded
}

/// Decode key from filesystem-safe format
fn decode_cache_key(encoded: &str) -> String {
    encoded
        .replace("_SLASH_", "/")
        .replace("_COLON_", ":")
        .replace("_UNDER_", "_")
}

/// Cache vulnerabilities to disk
fn cache_vulnerabilities(cache_dir: &Path, key: &str, vulns: &[Vulnerability]) -> Result<()> {
    let osv_cache = cache_dir.join("osv");
    fs::create_dir_all(&osv_cache)?;

    // Encode key for filesystem safety
    let safe_key = encode_cache_key(key);
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
                    // Decode key from filename
                    if let Some(filename) = path.file_stem().and_then(|s| s.to_str()) {
                        let key = decode_cache_key(filename);
                        results.insert(key, vulns);
                    }
                }
            }
        }
    }

    println!(
        "[bazbom] loaded {} cached vulnerability entries",
        results.len()
    );
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

        cache_vulnerabilities(cache_dir, "maven:test-pkg", std::slice::from_ref(&vuln))?;

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
