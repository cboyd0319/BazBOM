//! Checksum fetcher for package integrity verification
//!
//! Fetches SHA256 checksums from package registries for all supported ecosystems.
//! This module is opt-in (--fetch-checksums flag) to avoid slowing down fast scans.

use crate::ecosystems::Package;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Fetch SHA256 checksum for a package from its ecosystem registry
pub async fn fetch_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    match package.ecosystem.as_str() {
        "Maven" => fetch_maven_checksum(client, package).await,
        "Node.js/npm" | "npm" => fetch_npm_checksum(client, package).await,
        "Python" | "pip" => fetch_pypi_checksum(client, package).await,
        "Rust" => fetch_cargo_checksum(client, package).await,
        "Go" => fetch_go_checksum(client, package).await,
        "Ruby" => fetch_rubygems_checksum(client, package).await,
        "PHP" => fetch_composer_checksum(client, package).await,
        _ => Ok(None),
    }
}

/// Create HTTP client with reasonable defaults
pub fn create_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("BazBOM/6.5.0")
        .build()
        .context("failed to create HTTP client")
}

// ============================================================================
// Maven Central
// ============================================================================

async fn fetch_maven_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    // Maven publishes .sha256 files directly
    let checksum_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&checksum_url).send().await {
        Ok(response) if response.status().is_success() => {
            let checksum = response.text().await?;
            // Maven .sha256 files contain just the hash (sometimes with filename after)
            let hash = checksum.split_whitespace().next().unwrap_or(&checksum);
            Ok(Some(hash.to_string()))
        }
        _ => Ok(None), // Checksum file might not exist for all artifacts
    }
}

// ============================================================================
// npm Registry
// ============================================================================

#[derive(Deserialize)]
struct NpmPackageVersion {
    dist: NpmDist,
}

#[derive(Deserialize)]
struct NpmDist {
    shasum: String,      // SHA-1 (legacy)
    integrity: Option<String>,  // Subresource Integrity (SHA-256, SHA-384, SHA-512)
}

async fn fetch_npm_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    let api_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&api_url).send().await {
        Ok(response) if response.status().is_success() => {
            let pkg_info: NpmPackageVersion = response.json().await?;

            // Try to parse integrity field (format: "sha512-base64hash" or "sha256-base64hash")
            if let Some(integrity) = pkg_info.dist.integrity {
                if integrity.starts_with("sha256-") {
                    // Convert base64 to hex
                    let base64_hash = integrity.strip_prefix("sha256-").unwrap();
                    use base64::Engine;
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(base64_hash) {
                        return Ok(Some(hex::encode(decoded)));
                    }
                }
            }

            // Fallback: npm only provides SHA-1 in shasum, not SHA-256
            // We can't convert SHA-1 to SHA-256, so return None
            Ok(None)
        }
        _ => Ok(None),
    }
}

// ============================================================================
// PyPI
// ============================================================================

#[derive(Deserialize)]
struct PyPIRelease {
    urls: Vec<PyPIUrl>,
}

#[derive(Deserialize)]
struct PyPIUrl {
    digests: PyPIDigests,
    packagetype: String,
}

#[derive(Deserialize)]
struct PyPIDigests {
    sha256: String,
}

async fn fetch_pypi_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    let api_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&api_url).send().await {
        Ok(response) if response.status().is_success() => {
            let release: PyPIRelease = response.json().await?;

            // Find the sdist or wheel with SHA-256
            for url_info in release.urls {
                if url_info.packagetype == "sdist" || url_info.packagetype == "bdist_wheel" {
                    return Ok(Some(url_info.digests.sha256));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

// ============================================================================
// Cargo (crates.io)
// ============================================================================

#[derive(Deserialize)]
struct CargoVersion {
    #[serde(rename = "crate")]
    crate_info: CrateInfo,
}

#[derive(Deserialize)]
struct CrateInfo {
    max_version: String,
}

#[derive(Deserialize)]
struct CargoVersionDetail {
    version: CargoVersionInfo,
}

#[derive(Deserialize)]
struct CargoVersionInfo {
    checksum: String,  // This is SHA-256
}

async fn fetch_cargo_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    // Cargo API at crates.io provides checksums
    let api_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&api_url).send().await {
        Ok(response) if response.status().is_success() => {
            let version_detail: CargoVersionDetail = response.json().await?;
            Ok(Some(version_detail.version.checksum))
        }
        _ => Ok(None),
    }
}

// ============================================================================
// Go Proxy
// ============================================================================

async fn fetch_go_checksum(_client: &Client, _package: &Package) -> Result<Option<String>> {
    // Go proxy doesn't directly provide SHA-256, but we can fetch the .zip and hash it
    // For now, return None (this would require downloading the entire package)
    // Future enhancement: fetch .zip, calculate sha256
    Ok(None)
}

// ============================================================================
// RubyGems
// ============================================================================

#[derive(Deserialize)]
struct RubyGemsVersions(Vec<RubyGemVersion>);

#[derive(Deserialize)]
struct RubyGemVersion {
    number: String,
    sha: String,  // SHA-256
}

async fn fetch_rubygems_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    let api_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&api_url).send().await {
        Ok(response) if response.status().is_success() => {
            let versions: RubyGemsVersions = response.json().await?;

            // Find the version matching our package
            for version in versions.0 {
                if version.number == package.version {
                    return Ok(Some(version.sha));
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

// ============================================================================
// Composer (Packagist)
// ============================================================================

#[derive(Deserialize)]
struct PackagistResponse {
    packages: std::collections::HashMap<String, std::collections::HashMap<String, PackagistVersion>>,
}

#[derive(Deserialize)]
struct PackagistVersion {
    dist: Option<PackagistDist>,
}

#[derive(Deserialize)]
struct PackagistDist {
    shasum: Option<String>,  // SHA-1, not SHA-256
}

async fn fetch_composer_checksum(client: &Client, package: &Package) -> Result<Option<String>> {
    let api_url = package.checksum_url().context("no checksum URL")?;

    match client.get(&api_url).send().await {
        Ok(response) if response.status().is_success() => {
            let _pkg_response: PackagistResponse = response.json().await?;

            // Packagist only provides SHA-1, not SHA-256
            // We can't convert SHA-1 to SHA-256
            Ok(None)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let client = create_client();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_maven_checksum() {
        let client = create_client().unwrap();
        let package = Package {
            name: "annotations".to_string(),
            version: "4.1.1.4".to_string(),
            ecosystem: "Maven".to_string(),
            namespace: Some("com.google.android".to_string()),
            dependencies: vec![],
            license: None,
            description: None,
            homepage: None,
            repository: None,
        };

        let checksum = fetch_checksum(&client, &package).await;
        // This should succeed if Maven Central is accessible
        assert!(checksum.is_ok());
    }
}
