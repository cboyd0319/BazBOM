//! Unified SBOM generation for polyglot projects

use anyhow::Result;
use serde_json::json;
use crate::ecosystems::{EcosystemScanResult, Package};

/// Generate a unified SPDX SBOM from multiple ecosystem scan results
pub fn generate_polyglot_sbom(results: &[EcosystemScanResult]) -> Result<serde_json::Value> {
    let mut all_packages = Vec::new();
    let mut total_packages = 0;

    // Collect all packages from all ecosystems
    for result in results {
        for package in &result.packages {
            all_packages.push(create_spdx_package(package));
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

/// Create SPDX package entry
fn create_spdx_package(package: &Package) -> serde_json::Value {
    let spdx_id = format!("SPDXRef-Package-{}-{}",
        sanitize_for_spdx_id(&package.name),
        sanitize_for_spdx_id(&package.version)
    );

    json!({
        "SPDXID": spdx_id,
        "name": package.name,
        "versionInfo": package.version,
        "downloadLocation": "NOASSERTION",
        "filesAnalyzed": false,
        "licenseConcluded": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "licenseDeclared": package.license.as_ref().unwrap_or(&"NOASSERTION".to_string()),
        "copyrightText": "NOASSERTION",
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
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '.' { c } else { '-' })
        .collect()
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
}
