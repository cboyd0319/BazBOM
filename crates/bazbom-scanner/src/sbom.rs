//! Unified SBOM generation for polyglot projects

use crate::checksum_fetcher;
use crate::types::{EcosystemScanResult, Package};
use anyhow::Result;
use serde_json::json;

/// Generate a unified SPDX SBOM from multiple ecosystem scan results
pub async fn generate_polyglot_sbom(
    results: &[EcosystemScanResult],
    fetch_checksums: bool,
) -> Result<serde_json::Value> {
    let mut all_packages = Vec::new();
    let mut total_packages = 0;

    // Create HTTP client if checksum fetching is enabled
    let client = if fetch_checksums {
        Some(checksum_fetcher::create_client()?)
    } else {
        None
    };

    // Collect all packages from all ecosystems
    for result in results {
        for package in &result.packages {
            let pkg_json = if let Some(ref http_client) = client {
                // Fetch checksum if enabled
                create_spdx_package_with_checksum(package, http_client).await
            } else {
                create_spdx_package(package)
            };

            all_packages.push(pkg_json);
            total_packages += 1;
        }
    }

    // Build SPDX document
    let sbom = json!({
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": "SPDXRef-DOCUMENT",
        "name": "Polyglot SBOM",
        "documentNamespace": format!("https://bazbom.dev/sbom/{}", uuid::Uuid::new_v4()),
        "creationInfo": {
            "created": chrono::Utc::now().to_rfc3339(),
            "creators": ["Tool: BazBOM"],
            "licenseListVersion": "3.21"
        },
        "packages": all_packages,
        "relationships": generate_relationships(&all_packages),
        "comment": format!("Generated SBOM containing {} packages across {} ecosystems",
            total_packages, results.len())
    });

    Ok(sbom)
}

/// Create SPDX package entry (without checksum fetching)
fn create_spdx_package(package: &Package) -> serde_json::Value {
    let spdx_id = format!(
        "SPDXRef-Package-{}-{}",
        sanitize_for_spdx_id(&package.name),
        sanitize_for_spdx_id(&package.version)
    );

    // Get download location from ecosystem registry if available
    let download_location = package
        .download_url()
        .unwrap_or_else(|| "NOASSERTION".to_string());

    json!({
        "SPDXID": spdx_id,
        "name": package.name,
        "versionInfo": package.version,
        "downloadLocation": download_location,
        "filesAnalyzed": false,
        "licenseConcluded": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "licenseDeclared": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "copyrightText": "NOASSERTION",
        "checksums": null,
        "externalRefs": [{
            "referenceCategory": "PACKAGE-MANAGER",
            "referenceType": "purl",
            "referenceLocator": package.purl()
        }],
        "description": package.description.as_ref().unwrap_or(&"".to_string()),
        "homepage": package.homepage.as_ref().unwrap_or(&"".to_string()),
        "comment": format!("Ecosystem: {}", package.ecosystem)
    })
}

/// Create SPDX package entry WITH checksum fetching
async fn create_spdx_package_with_checksum(
    package: &Package,
    client: &reqwest::Client,
) -> serde_json::Value {
    let spdx_id = format!(
        "SPDXRef-Package-{}-{}",
        sanitize_for_spdx_id(&package.name),
        sanitize_for_spdx_id(&package.version)
    );

    // Get download location from ecosystem registry if available
    let download_location = package
        .download_url()
        .unwrap_or_else(|| "NOASSERTION".to_string());

    // Fetch SHA256 checksum
    let checksums = match checksum_fetcher::fetch_checksum(client, package).await {
        Ok(Some(sha256)) => {
            tracing::debug!("Fetched checksum for {}: {}", package.name, sha256);
            json!([{
                "algorithm": "SHA256",
                "checksumValue": sha256
            }])
        }
        Ok(None) => {
            tracing::debug!("No checksum available for {}", package.name);
            json!(null)
        }
        Err(e) => {
            tracing::warn!("Failed to fetch checksum for {}: {}", package.name, e);
            json!(null)
        }
    };

    json!({
        "SPDXID": spdx_id,
        "name": package.name,
        "versionInfo": package.version,
        "downloadLocation": download_location,
        "filesAnalyzed": false,
        "licenseConcluded": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "licenseDeclared": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "copyrightText": "NOASSERTION",
        "checksums": checksums,
        "externalRefs": [{
            "referenceCategory": "PACKAGE-MANAGER",
            "referenceType": "purl",
            "referenceLocator": package.purl()
        }],
        "description": package.description.as_ref().unwrap_or(&"".to_string()),
        "homepage": package.homepage.as_ref().unwrap_or(&"".to_string()),
        "comment": format!("Ecosystem: {}", package.ecosystem)
    })
}

/// Generate SPDX relationships
fn generate_relationships(packages: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut relationships = Vec::new();

    // Add root document describes relationship for each package
    for package in packages {
        if let Some(spdx_id) = package.get("SPDXID").and_then(|v| v.as_str()) {
            relationships.push(json!({
                "spdxElementId": "SPDXRef-DOCUMENT",
                "relationshipType": "DESCRIBES",
                "relatedSpdxElement": spdx_id
            }));
        }
    }

    relationships
}

/// Sanitize string for use in SPDX ID (alphanumeric, hyphen, dot only)
fn sanitize_for_spdx_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Generate GitHub dependency snapshot format
pub fn generate_github_snapshot(
    results: &[EcosystemScanResult],
    sha: &str,
    ref_name: &str,
) -> Result<serde_json::Value> {
    use std::collections::HashMap;

    let mut manifests = serde_json::Map::new();

    for result in results {
        // Create manifest name from ecosystem
        let manifest_name = match result.ecosystem.as_str() {
            "Maven" | "Maven (Bazel)" => "pom.xml",
            "Node.js/npm" | "npm" => "package-lock.json",
            "Python" | "pip" => "requirements.txt",
            "Rust" => "Cargo.lock",
            "Go" => "go.mod",
            "Ruby" => "Gemfile.lock",
            "PHP" => "composer.lock",
            _ => "dependencies.txt",
        };

        let mut resolved = HashMap::new();

        for package in &result.packages {
            let package_key = format!("{}@{}", package.name, package.version);
            resolved.insert(
                package_key,
                json!({
                    "package_url": package.purl(),
                    "metadata": {},
                    "relationship": "direct",  // We don't have dependency graph info
                    "scope": "runtime",
                    "dependencies": []  // Empty for now
                }),
            );
        }

        manifests.insert(
            manifest_name.to_string(),
            json!({
                "name": manifest_name,
                "file": {
                    "source_location": manifest_name
                },
                "metadata": {},
                "resolved": resolved
            }),
        );
    }

    let snapshot = json!({
        "version": 0,
        "sha": sha,
        "ref": ref_name,
        "job": {
            "id": uuid::Uuid::new_v4().to_string(),
            "correlator": "bazbom_scan"
        },
        "detector": {
            "name": "BazBOM",
            "version": env!("CARGO_PKG_VERSION"),
            "url": "https://github.com/cboyd0319/BazBOM"
        },
        "scanned": chrono::Utc::now().to_rfc3339(),
        "metadata": {},
        "manifests": manifests
    });

    Ok(snapshot)
}

/// Convert SPDX JSON to tag-value format
pub fn spdx_json_to_tag_value(spdx_json: &serde_json::Value) -> Result<String> {
    let mut output = String::new();

    // Document header
    output.push_str(&format!(
        "SPDXVersion: {}\n",
        spdx_json["spdxVersion"].as_str().unwrap_or("SPDX-2.3")
    ));
    output.push_str(&format!(
        "DataLicense: {}\n",
        spdx_json["dataLicense"].as_str().unwrap_or("CC0-1.0")
    ));
    output.push_str(&format!(
        "SPDXID: {}\n",
        spdx_json["SPDXID"].as_str().unwrap_or("SPDXRef-DOCUMENT")
    ));
    output.push_str(&format!(
        "DocumentName: {}\n",
        spdx_json["name"].as_str().unwrap_or("SBOM")
    ));
    output.push_str(&format!(
        "DocumentNamespace: {}\n",
        spdx_json["documentNamespace"].as_str().unwrap_or("")
    ));

    // Creation info
    if let Some(creation_info) = spdx_json["creationInfo"].as_object() {
        if let Some(created) = creation_info["created"].as_str() {
            output.push_str(&format!("Created: {}\n", created));
        }
        if let Some(creators) = creation_info["creators"].as_array() {
            for creator in creators {
                if let Some(creator_str) = creator.as_str() {
                    output.push_str(&format!("Creator: {}\n", creator_str));
                }
            }
        }
        if let Some(license_version) = creation_info["licenseListVersion"].as_str() {
            output.push_str(&format!("LicenseListVersion: {}\n", license_version));
        }
    }

    // Document comment
    if let Some(comment) = spdx_json["comment"].as_str() {
        output.push_str(&format!("DocumentComment: <text>{}</text>\n", comment));
    }

    output.push('\n');

    // Packages
    if let Some(packages) = spdx_json["packages"].as_array() {
        for package in packages {
            output.push_str(&format!(
                "PackageName: {}\n",
                package["name"].as_str().unwrap_or("")
            ));
            output.push_str(&format!(
                "SPDXID: {}\n",
                package["SPDXID"].as_str().unwrap_or("")
            ));

            if let Some(version) = package["versionInfo"].as_str() {
                output.push_str(&format!("PackageVersion: {}\n", version));
            }

            if let Some(download_location) = package["downloadLocation"].as_str() {
                output.push_str(&format!("PackageDownloadLocation: {}\n", download_location));
            }

            output.push_str(&format!(
                "FilesAnalyzed: {}\n",
                if package["filesAnalyzed"].as_bool().unwrap_or(false) {
                    "true"
                } else {
                    "false"
                }
            ));

            // Checksums
            if let Some(checksums) = package["checksums"].as_array() {
                for checksum in checksums {
                    if let (Some(alg), Some(value)) = (
                        checksum["algorithm"].as_str(),
                        checksum["checksumValue"].as_str(),
                    ) {
                        output.push_str(&format!("PackageChecksum: {}: {}\n", alg, value));
                    }
                }
            }

            // License
            if let Some(license) = package["licenseConcluded"].as_str() {
                output.push_str(&format!("PackageLicenseConcluded: {}\n", license));
            }
            if let Some(license) = package["licenseDeclared"].as_str() {
                output.push_str(&format!("PackageLicenseDeclared: {}\n", license));
            }

            // Copyright
            if let Some(copyright) = package["copyrightText"].as_str() {
                output.push_str(&format!("PackageCopyrightText: {}\n", copyright));
            }

            // External refs
            if let Some(external_refs) = package["externalRefs"].as_array() {
                for ext_ref in external_refs {
                    if let (Some(cat), Some(typ), Some(loc)) = (
                        ext_ref["referenceCategory"].as_str(),
                        ext_ref["referenceType"].as_str(),
                        ext_ref["referenceLocator"].as_str(),
                    ) {
                        output.push_str(&format!("ExternalRef: {} {} {}\n", cat, typ, loc));
                    }
                }
            }

            // Description
            if let Some(desc) = package["description"].as_str() {
                if !desc.is_empty() {
                    output.push_str(&format!("PackageSummary: <text>{}</text>\n", desc));
                }
            }

            // Homepage
            if let Some(homepage) = package["homepage"].as_str() {
                if !homepage.is_empty() {
                    output.push_str(&format!("PackageHomePage: {}\n", homepage));
                }
            }

            // Comment (ecosystem)
            if let Some(comment) = package["comment"].as_str() {
                output.push_str(&format!("PackageComment: <text>{}</text>\n", comment));
            }

            output.push('\n');
        }
    }

    // Relationships
    if let Some(relationships) = spdx_json["relationships"].as_array() {
        for relationship in relationships {
            if let (Some(element_id), Some(rel_type), Some(related_element)) = (
                relationship["spdxElementId"].as_str(),
                relationship["relationshipType"].as_str(),
                relationship["relatedSpdxElement"].as_str(),
            ) {
                output.push_str(&format!(
                    "Relationship: {} {} {}\n",
                    element_id, rel_type, related_element
                ));
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_for_spdx_id() {
        assert_eq!(sanitize_for_spdx_id("@types/node"), "-types-node");
        assert_eq!(sanitize_for_spdx_id("express"), "express");
        assert_eq!(sanitize_for_spdx_id("1.2.3"), "1.2.3");
    }

    #[tokio::test]
    async fn test_generate_polyglot_sbom_no_checksums() {
        let results = vec![];
        let sbom = generate_polyglot_sbom(&results, false).await.unwrap();
        assert_eq!(sbom["spdxVersion"], "SPDX-2.3");
    }
}
