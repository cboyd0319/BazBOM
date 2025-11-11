//! Vulnerability scanning using OSV (Open Source Vulnerabilities) API
//!
//! https://osv.dev/docs/

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use crate::ecosystems::{Package, Vulnerability};

const OSV_API_URL: &str = "https://api.osv.dev/v1/query";

/// OSV Query Request
#[derive(Debug, Serialize)]
struct OsvQueryRequest {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    ecosystem: String,
    name: String,
}

/// OSV Query Response
#[derive(Debug, Deserialize)]
struct OsvQueryResponse {
    #[serde(default)]
    vulns: Vec<OsvVulnerability>,
}

#[derive(Debug, Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    details: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    published: Option<String>,
    #[serde(default)]
    severity: Vec<OsvSeverity>,
    #[serde(default)]
    affected: Vec<OsvAffected>,
    #[serde(default)]
    references: Vec<OsvReference>,
}

#[derive(Debug, Deserialize)]
struct OsvSeverity {
    #[serde(rename = "type")]
    severity_type: String,
    score: String,
}

#[derive(Debug, Deserialize)]
struct OsvAffected {
    package: Option<OsvAffectedPackage>,
    ranges: Option<Vec<OsvRange>>,
    versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OsvAffectedPackage {
    ecosystem: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct OsvRange {
    #[serde(rename = "type")]
    range_type: String,
    events: Vec<OsvEvent>,
}

#[derive(Debug, Deserialize)]
struct OsvEvent {
    introduced: Option<String>,
    fixed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsvReference {
    #[serde(rename = "type")]
    ref_type: String,
    url: String,
}

/// Scan packages for vulnerabilities using OSV API
pub async fn scan_vulnerabilities(packages: &[Package]) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();
    let client = reqwest::Client::new();

    for package in packages {
        let osv_ecosystem = map_ecosystem(&package.ecosystem);

        // Query OSV API
        let request = OsvQueryRequest {
            package: OsvPackage {
                ecosystem: osv_ecosystem.to_string(),
                name: format_package_name(package),
            },
            version: package.version.clone(),
        };

        match query_osv(&client, &request).await {
            Ok(response) => {
                for osv_vuln in response.vulns {
                    if let Some(vuln) = convert_osv_vulnerability(&osv_vuln, package) {
                        vulnerabilities.push(vuln);
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to query OSV for {}@{}: {}",
                    package.name, package.version, e);
            }
        }

        // Rate limiting: OSV recommends max 100 requests per second
        // Sleep 10ms between requests to be safe
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    Ok(vulnerabilities)
}

/// Query OSV API
async fn query_osv(client: &reqwest::Client, request: &OsvQueryRequest) -> Result<OsvQueryResponse> {
    let response = client
        .post(OSV_API_URL)
        .json(request)
        .send()
        .await
        .context("Failed to send OSV API request")?;

    if !response.status().is_success() {
        anyhow::bail!("OSV API returned error: {}", response.status());
    }

    let osv_response = response
        .json::<OsvQueryResponse>()
        .await
        .context("Failed to parse OSV API response")?;

    Ok(osv_response)
}

/// Convert OSV vulnerability to our format
fn convert_osv_vulnerability(osv: &OsvVulnerability, package: &Package) -> Option<Vulnerability> {
    // Extract CVE ID from aliases if available
    let cve_id = osv.aliases.iter()
        .find(|alias| alias.starts_with("CVE-"))
        .cloned()
        .unwrap_or_else(|| osv.id.clone());

    // Extract CVSS score
    let cvss_score = osv.severity.iter()
        .find(|s| s.severity_type == "CVSS_V3")
        .and_then(|s| parse_cvss_score(&s.score));

    // Determine severity from CVSS or use default
    let severity = if let Some(cvss) = cvss_score {
        if cvss >= 9.0 {
            "CRITICAL"
        } else if cvss >= 7.0 {
            "HIGH"
        } else if cvss >= 4.0 {
            "MEDIUM"
        } else {
            "LOW"
        }
    } else {
        "MEDIUM" // Default if no CVSS
    };

    // Find fixed version
    let fixed_version = find_fixed_version(&osv.affected, &package.version);

    // Extract references
    let references: Vec<String> = osv.references.iter()
        .map(|r| r.url.clone())
        .collect();

    Some(Vulnerability {
        id: cve_id,
        package_name: package.name.clone(),
        package_version: package.version.clone(),
        severity: severity.to_string(),
        cvss_score,
        fixed_version,
        title: osv.summary.clone().unwrap_or_else(|| osv.id.clone()),
        description: osv.details.clone().unwrap_or_default(),
        references,
        published_date: osv.published.clone(),
    })
}

/// Parse CVSS score from string (e.g., "CVSS:3.1/AV:N/AC:L/...")
fn parse_cvss_score(score_str: &str) -> Option<f64> {
    // CVSS vector strings start with "CVSS:3.1/" and contain a base score
    // For simplicity, we'll extract the numeric score if present
    // Full parsing would require a CVSS calculator

    // Try to find a numeric score in the string
    score_str.split('/')
        .filter_map(|part| {
            if part.starts_with("S:") || part.starts_with("score:") {
                part.split(':').nth(1)?.parse::<f64>().ok()
            } else {
                None
            }
        })
        .next()
}

/// Find fixed version from OSV affected ranges
fn find_fixed_version(affected: &[OsvAffected], _current_version: &str) -> Option<String> {
    for aff in affected {
        if let Some(ranges) = &aff.ranges {
            for range in ranges {
                for event in &range.events {
                    if let Some(ref fixed) = event.fixed {
                        // Simple heuristic: return first fixed version we find
                        return Some(fixed.clone());
                    }
                }
            }
        }
    }
    None
}

/// Map our ecosystem names to OSV ecosystem names
fn map_ecosystem(ecosystem: &str) -> &str {
    match ecosystem {
        "Node.js/npm" | "npm" => "npm",
        "Python" | "pip" => "PyPI",
        "Go" => "Go",
        "Rust" | "cargo" => "crates.io",
        "Ruby" | "gem" => "RubyGems",
        "PHP" | "composer" => "Packagist",
        other => other,
    }
}

/// Format package name for OSV query
fn format_package_name(package: &Package) -> String {
    if let Some(ref ns) = package.namespace {
        format!("{}/{}", ns, package.name)
    } else {
        package.name.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_ecosystem() {
        assert_eq!(map_ecosystem("Node.js/npm"), "npm");
        assert_eq!(map_ecosystem("Python"), "PyPI");
        assert_eq!(map_ecosystem("Go"), "Go");
        assert_eq!(map_ecosystem("Rust"), "crates.io");
    }

    #[test]
    fn test_format_package_name() {
        let pkg = Package {
            name: "express".to_string(),
            version: "4.17.0".to_string(),
            ecosystem: "npm".to_string(),
            namespace: None,
            dependencies: vec![],
            license: None,
            description: None,
            homepage: None,
            repository: None,
        };
        assert_eq!(format_package_name(&pkg), "express");

        let scoped_pkg = Package {
            name: "node".to_string(),
            version: "18.0.0".to_string(),
            ecosystem: "npm".to_string(),
            namespace: Some("@types".to_string()),
            dependencies: vec![],
            license: None,
            description: None,
            homepage: None,
            repository: None,
        };
        assert_eq!(format_package_name(&scoped_pkg), "@types/node");
    }
}
