//! Polyglot Ecosystem Support
//!
//! This module provides comprehensive support for multiple programming language
//! ecosystems beyond JVM (Java/Kotlin/Scala).
//!
//! # Supported Ecosystems
//!
//! - **Node.js/npm** - JavaScript/TypeScript packages
//! - **Python** - pip, poetry, pipenv packages
//! - **Go** - Go modules
//! - **Rust** - Cargo packages
//! - **Ruby** - Bundler/gems
//! - **PHP** - Composer packages
//!
//! # Architecture
//!
//! 1. **Detection** - Scan directory tree for ecosystem manifest files
//! 2. **Parsing** - Parse lockfiles to extract dependency graphs
//! 3. **SBOM Generation** - Convert to SPDX/CycloneDX format
//! 4. **Vulnerability Scanning** - Query OSV/GitHub Advisory databases
//!
//! # Usage
//!
//! ```rust,no_run
//! use bazbom_scanner::scan_directory;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Auto-detect and scan all ecosystems in a directory
//! let results = scan_directory(".").await?;
//!
//! for result in results {
//!     println!("Found {} packages in {}", result.packages.len(), result.ecosystem);
//! }
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod checksum_fetcher;
pub mod cicd;
pub mod detection;
pub mod ecosystems;
pub mod reachability_integration;
pub mod registry;
pub mod sbom;
pub mod scanner;
pub mod types;
pub mod vulnerabilities;
pub mod utils;

pub use cache::LicenseCache;
pub use detection::{detect_ecosystems, Ecosystem, EcosystemType};
pub use registry::ScannerRegistry;
pub use scanner::{License, LicenseContext, ScanContext, Scanner};
pub use types::{EcosystemScanResult, Package, ReachabilityData, Vulnerability};
pub use reachability_integration::analyze_reachability;
pub use sbom::{generate_github_snapshot, generate_polyglot_sbom, spdx_json_to_tag_value};

use anyhow::Result;
use std::sync::Arc;

/// Scan a directory for packages only (fast SBOM generation without vulnerabilities or reachability)
pub async fn scan_directory_sbom_only(path: &str, include_cicd: bool) -> Result<Vec<EcosystemScanResult>> {
    let ecosystems = detect_ecosystems(path)?;
    let mut results = Vec::new();

    for ecosystem in ecosystems {
        match scan_ecosystem(&ecosystem).await {
            Ok(result) => {
                // Just collect packages - no vulnerability scanning, no reachability
                results.push(result);
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan {}: {}", ecosystem.name, e);
            }
        }
    }

    // Optionally detect GitHub Actions and other CI/CD tooling
    if include_cicd {
        match cicd::detect_github_actions(std::path::Path::new(path)) {
            Ok(cicd_result) => {
                if !cicd_result.packages.is_empty() {
                    results.push(cicd_result);
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan GitHub Actions: {}", e);
            }
        }
    }

    Ok(results)
}

/// Scan a directory for all supported ecosystems and generate a unified SBOM with vulnerabilities
pub async fn scan_directory(path: &str) -> Result<Vec<EcosystemScanResult>> {
    let ecosystems = detect_ecosystems(path)?;
    let mut results = Vec::new();

    for ecosystem in ecosystems {
        match scan_ecosystem(&ecosystem).await {
            Ok(mut result) => {
                // Scan for vulnerabilities using OSV
                if !result.packages.is_empty() {
                    match vulnerabilities::scan_vulnerabilities(&result.packages).await {
                        Ok(vulns) => {
                            result.vulnerabilities = vulns;
                            result.total_vulnerabilities = result.vulnerabilities.len();
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to scan vulnerabilities for {}: {}",
                                result.ecosystem, e
                            );
                        }
                    }
                }

                // Perform reachability analysis if vulnerabilities found
                if !result.vulnerabilities.is_empty() {
                    if let Err(e) = reachability_integration::analyze_reachability(
                        &mut result,
                        ecosystem.root_path.as_path(),
                    )
                    .await
                    {
                        eprintln!(
                            "Warning: Failed to analyze reachability for {}: {}",
                            result.ecosystem, e
                        );
                    }
                }

                results.push(result);
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan {}: {}", ecosystem.name, e);
            }
        }
    }

    Ok(results)
}

/// Scan a specific ecosystem
async fn scan_ecosystem(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    match ecosystem.ecosystem_type {
        EcosystemType::Npm => {
            // Use the new trait-based scanner
            let scanner = ecosystems::npm::NpmScanner::new();
            let cache = Arc::new(LicenseCache::new());
            let mut ctx = ScanContext::new(ecosystem.root_path.clone(), cache);

            if let Some(ref manifest) = ecosystem.manifest_file {
                ctx = ctx.with_manifest(manifest.clone());
            }
            if let Some(ref lockfile) = ecosystem.lockfile {
                ctx = ctx.with_lockfile(lockfile.clone());
            }

            scanner.scan(&ctx).await
        }
        EcosystemType::Python => {
            // Use the new trait-based scanner
            let scanner = ecosystems::python::PythonScanner::new();
            let cache = Arc::new(LicenseCache::new());
            let mut ctx = ScanContext::new(ecosystem.root_path.clone(), cache);

            if let Some(ref manifest) = ecosystem.manifest_file {
                ctx = ctx.with_manifest(manifest.clone());
            }
            if let Some(ref lockfile) = ecosystem.lockfile {
                ctx = ctx.with_lockfile(lockfile.clone());
            }

            scanner.scan(&ctx).await
        }
        EcosystemType::Go => ecosystems::go::scan(ecosystem).await,
        EcosystemType::Rust => ecosystems::rust::scan(ecosystem).await,
        EcosystemType::Ruby => ecosystems::ruby::scan(ecosystem).await,
        EcosystemType::Php => ecosystems::php::scan(ecosystem).await,
        EcosystemType::Maven => ecosystems::maven::scan(ecosystem).await,
        EcosystemType::Gradle => ecosystems::gradle::scan(ecosystem).await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scan_directory() {
        // Test with a temp directory
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().to_str().unwrap();

        let results = scan_directory(path).await.unwrap();
        assert_eq!(results.len(), 0); // No ecosystems in empty dir
    }
}
