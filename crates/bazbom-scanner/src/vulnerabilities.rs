//! Vulnerability scanning using OSV (Open Source Vulnerabilities) API
//!
//! https://osv.dev/docs/

use crate::types::{Package, Vulnerability};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const OSV_API_URL: &str = "https://api.osv.dev/v1/query";
const OSV_BATCH_API_URL: &str = "https://api.osv.dev/v1/querybatch";

/// OSV Query Request
#[derive(Debug, Clone, Serialize)]
struct OsvQueryRequest {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Clone, Serialize)]
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

/// OSV Batch Query Request
#[derive(Debug, Serialize)]
struct OsvBatchQueryRequest {
    queries: Vec<OsvQueryRequest>,
}

/// OSV Batch Query Response
#[derive(Debug, Deserialize)]
struct OsvBatchQueryResponse {
    results: Vec<OsvQueryResponse>,
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
    #[allow(dead_code)]
    package: Option<OsvAffectedPackage>,
    ranges: Option<Vec<OsvRange>>,
    #[allow(dead_code)]
    versions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OsvAffectedPackage {
    #[allow(dead_code)]
    ecosystem: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct OsvRange {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    range_type: String,
    events: Vec<OsvEvent>,
}

#[derive(Debug, Deserialize)]
struct OsvEvent {
    #[allow(dead_code)]
    introduced: Option<String>,
    fixed: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsvReference {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    ref_type: String,
    url: String,
}

/// Scan packages for vulnerabilities using OSV API (batch mode for better performance)
pub async fn scan_vulnerabilities(packages: &[Package]) -> Result<Vec<Vulnerability>> {
    if packages.is_empty() {
        return Ok(Vec::new());
    }

    eprintln!("DEBUG: scan_vulnerabilities called with {} packages", packages.len());
    if !packages.is_empty() {
        eprintln!("DEBUG: First package: {}@{} (ecosystem: {})",
            packages[0].name, packages[0].version, packages[0].ecosystem);
    }

    let client = reqwest::Client::new();

    // Use batch API if more than 1 package, otherwise use single query
    if packages.len() == 1 {
        return scan_vulnerabilities_single(&client, &packages[0]).await;
    }

    // Build batch query request
    let queries: Vec<OsvQueryRequest> = packages
        .iter()
        .map(|package| {
            let osv_ecosystem = map_ecosystem(&package.ecosystem);
            OsvQueryRequest {
                package: OsvPackage {
                    ecosystem: osv_ecosystem.to_string(),
                    name: format_package_name(package),
                },
                version: package.version.clone(),
            }
        })
        .collect();

    eprintln!(
        "DEBUG: Sending batch query to OSV for {} packages",
        queries.len()
    );

    // Query OSV batch API
    match query_osv_batch(&client, &queries).await {
        Ok(batch_response) => {
            let mut vulnerabilities = Vec::new();

            // Process each response
            for (idx, response) in batch_response.results.iter().enumerate() {
                if idx >= packages.len() {
                    break;
                }

                let package = &packages[idx];
                eprintln!(
                    "DEBUG: OSV returned {} vulnerabilities for {}@{}",
                    response.vulns.len(),
                    package.name,
                    package.version
                );

                for osv_vuln in &response.vulns {
                    if let Some(vuln) = convert_osv_vulnerability(osv_vuln, package) {
                        vulnerabilities.push(vuln);
                    }
                }
            }

            Ok(vulnerabilities)
        }
        Err(e) => {
            eprintln!("Warning: Batch OSV query failed, falling back to individual queries: {}", e);
            // Fallback to individual queries
            scan_vulnerabilities_fallback(&client, packages).await
        }
    }
}

/// Scan a single package (used when only 1 package to scan)
async fn scan_vulnerabilities_single(
    client: &reqwest::Client,
    package: &Package,
) -> Result<Vec<Vulnerability>> {
    let osv_ecosystem = map_ecosystem(&package.ecosystem);
    eprintln!(
        "DEBUG: Querying OSV for {}@{} (OSV ecosystem: {})",
        package.name, package.version, osv_ecosystem
    );

    let request = OsvQueryRequest {
        package: OsvPackage {
            ecosystem: osv_ecosystem.to_string(),
            name: format_package_name(package),
        },
        version: package.version.clone(),
    };

    eprintln!(
        "DEBUG: OSV request: {{\"package\": {{\"ecosystem\": \"{}\", \"name\": \"{}\"}}, \"version\": \"{}\"}}",
        request.package.ecosystem, request.package.name, request.version
    );

    match query_osv(client, &request).await {
        Ok(response) => {
            eprintln!(
                "DEBUG: OSV returned {} vulnerabilities for {}@{}",
                response.vulns.len(),
                package.name,
                package.version
            );

            Ok(response
                .vulns
                .iter()
                .filter_map(|osv_vuln| convert_osv_vulnerability(osv_vuln, package))
                .collect())
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to query OSV for {}@{}: {}",
                package.name, package.version, e
            );
            Ok(Vec::new())
        }
    }
}

/// Fallback to individual queries (if batch fails)
async fn scan_vulnerabilities_fallback(
    client: &reqwest::Client,
    packages: &[Package],
) -> Result<Vec<Vulnerability>> {
    let mut vulnerabilities = Vec::new();

    for package in packages {
        match scan_vulnerabilities_single(client, package).await {
            Ok(mut vulns) => vulnerabilities.append(&mut vulns),
            Err(e) => {
                eprintln!(
                    "Warning: Failed to query OSV for {}@{}: {}",
                    package.name, package.version, e
                );
            }
        }

        // Rate limiting: OSV recommends max 100 requests per second
        // Sleep 10ms between requests to be safe
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    Ok(vulnerabilities)
}

/// Query OSV API (single package)
async fn query_osv(
    client: &reqwest::Client,
    request: &OsvQueryRequest,
) -> Result<OsvQueryResponse> {
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

/// Query OSV Batch API (multiple packages in single request)
async fn query_osv_batch(
    client: &reqwest::Client,
    queries: &[OsvQueryRequest],
) -> Result<OsvBatchQueryResponse> {
    let batch_request = OsvBatchQueryRequest {
        queries: queries.to_vec(),
    };

    let response = client
        .post(OSV_BATCH_API_URL)
        .json(&batch_request)
        .send()
        .await
        .context("Failed to send OSV batch API request")?;

    if !response.status().is_success() {
        anyhow::bail!("OSV batch API returned error: {}", response.status());
    }

    let batch_response = response
        .json::<OsvBatchQueryResponse>()
        .await
        .context("Failed to parse OSV batch API response")?;

    Ok(batch_response)
}

/// Convert OSV vulnerability to our format
fn convert_osv_vulnerability(osv: &OsvVulnerability, package: &Package) -> Option<Vulnerability> {
    // Extract CVE ID from aliases if available
    let cve_id = osv
        .aliases
        .iter()
        .find(|alias| alias.starts_with("CVE-"))
        .cloned()
        .unwrap_or_else(|| osv.id.clone());

    // Extract CVSS score
    let cvss_score = osv
        .severity
        .iter()
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
    let references: Vec<String> = osv.references.iter().map(|r| r.url.clone()).collect();

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
    score_str
        .split('/')
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
        "Maven" | "Gradle" => "Maven", // Both use Maven Central
        other => other,
    }
}

/// Format package name for OSV query
fn format_package_name(package: &Package) -> String {
    // For Maven/Gradle, the package name already includes groupId:artifactId format
    // Namespace is the groupId and is redundant - don't include it
    if package.ecosystem == "Maven" || package.ecosystem == "Gradle" {
        return package.name.clone();
    }

    // For scoped packages (e.g., npm @scope/package), namespace is the scope
    // For ecosystem-level namespaces (e.g., crates.io), we don't include it in the name
    // OSV expects the package name without ecosystem prefix, as ecosystem is sent separately
    if let Some(ref ns) = package.namespace {
        // Check if namespace looks like an ecosystem name (contains ".io", ".org", etc.)
        if ns.contains('.') || ns == "crates.io" || ns == "github.com" {
            // It's an ecosystem namespace, not a package scope - don't include it
            package.name.clone()
        } else if ns.starts_with('@') || package.ecosystem == "npm" {
            // It's an npm scope like "@typescript-eslint" - include it
            format!("{}/{}", ns, package.name)
        } else {
            // For other cases, include the namespace
            format!("{}/{}", ns, package.name)
        }
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
