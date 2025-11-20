//! Rust Cargo scanner
//!
//! Parses Cargo.toml and Cargo.lock files

use crate::scanner::{License, LicenseContext, ScanContext, Scanner};
use crate::types::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use async_trait::async_trait;
use cargo_lock::Lockfile;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Rust ecosystem scanner
pub struct RustScanner;

impl Default for RustScanner {
    fn default() -> Self {
        Self
    }
}

impl RustScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for RustScanner {
    fn name(&self) -> &str {
        "rust"
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("Cargo.toml").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result = EcosystemScanResult::new(
            "Rust".to_string(),
            ctx.root.display().to_string(),
        );

        // Parse Cargo.lock if available (most accurate)
        if let Some(ref lockfile_path) = ctx.lockfile {
            parse_cargo_lock(lockfile_path, &mut result)?;
        } else if let Some(ref manifest_path) = ctx.manifest {
            // Fallback to Cargo.toml (less accurate, just shows dependencies)
            parse_cargo_toml(manifest_path, &mut result)?;
        }

        // Run reachability analysis
        if let Err(e) = analyze_reachability(&ctx.root, &mut result) {
            eprintln!("Warning: Rust reachability analysis failed: {}", e);
            let mut source = e.source();
            while let Some(err) = source {
                eprintln!("  Caused by: {}", err);
                source = err.source();
            }
        }

        Ok(result)
    }

    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        // Rust doesn't have a standard location for installed package licenses
        // License info would need to be fetched from Cargo.toml or crates.io API
        License::Unknown
    }
}

/// Analyze reachability for Rust project
fn analyze_reachability(root: &Path, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_reachability::rust::analyze_rust_project;

    let report = analyze_rust_project(root)
        .context("Failed to run Rust reachability analysis")?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for package in &result.packages {
        let package_key = format!("{}@{}", package.name, package.version);
        vulnerable_packages_reachable.insert(package_key, !report.reachable_functions.is_empty());
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Parse Cargo.lock using the cargo-lock crate
fn parse_cargo_lock(
    lockfile_path: &Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let lockfile = Lockfile::load(lockfile_path).context("Failed to parse Cargo.lock")?;

    for package in &lockfile.packages {
        let name = package.name.as_str().to_string();
        let version = package.version.to_string();

        let namespace = package.source.as_ref().and_then(|src| {
            let src_str = src.to_string();
            if src_str.contains("crates.io") {
                Some("crates.io".to_string())
            } else if src_str.contains("github.com") {
                src_str
                    .split("github.com/")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .map(|org| format!("github.com/{}", org))
            } else {
                None
            }
        });

        let dependencies: Vec<String> = package
            .dependencies
            .iter()
            .map(|dep| dep.name.to_string())
            .collect();

        result.add_package(Package {
            name,
            version,
            ecosystem: "Rust".to_string(),
            namespace,
            dependencies,
            license: None,
            description: None,
            homepage: None,
            repository: None,
        });
    }

    Ok(())
}

/// Parse Cargo.toml (basic fallback, less accurate)
fn parse_cargo_toml(
    manifest_path: &Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(manifest_path).context("Failed to read Cargo.toml")?;

    let toml_value: toml::Value = toml::from_str(&content).context("Failed to parse Cargo.toml")?;

    // Parse [dependencies] section
    if let Some(deps) = toml_value.get("dependencies").and_then(|v| v.as_table()) {
        for (name, version_spec) in deps {
            let version = extract_version_from_toml(version_spec);

            result.add_package(Package {
                name: name.clone(),
                version,
                ecosystem: "Rust".to_string(),
                namespace: Some("crates.io".to_string()),
                dependencies: Vec::new(),
                license: None,
                description: None,
                homepage: None,
                repository: None,
            });
        }
    }

    // Parse [dev-dependencies] section
    if let Some(dev_deps) = toml_value
        .get("dev-dependencies")
        .and_then(|v| v.as_table())
    {
        for (name, version_spec) in dev_deps {
            let version = extract_version_from_toml(version_spec);

            result.add_package(Package {
                name: name.clone(),
                version,
                ecosystem: "Rust".to_string(),
                namespace: Some("crates.io".to_string()),
                dependencies: Vec::new(),
                license: None,
                description: None,
                homepage: None,
                repository: None,
            });
        }
    }

    Ok(())
}

/// Extract version from TOML value
fn extract_version_from_toml(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::cache::LicenseCache;
    use tempfile::TempDir;

    #[test]
    fn test_rust_scanner_detect() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("Cargo.toml"), "[package]\nname=\"test\"").unwrap();

        let scanner = RustScanner::new();
        assert!(scanner.detect(temp.path()));
    }

    #[test]
    fn test_extract_version_from_toml() {
        let string_version = toml::Value::String("1.0.0".to_string());
        assert_eq!(extract_version_from_toml(&string_version), "1.0.0");

        let table_version = toml::from_str(r#"version = "2.0.0""#).unwrap();
        assert_eq!(extract_version_from_toml(&table_version), "2.0.0");
    }

    #[tokio::test]
    async fn test_parse_cargo_toml() {
        let temp = TempDir::new().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");

        fs::write(
            &cargo_toml,
            r#"
[package]
name = "test-crate"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"
"#,
        )
        .unwrap();

        let scanner = RustScanner::new();
        let cache = Arc::new(LicenseCache::new());
        let ctx = ScanContext::new(temp.path().to_path_buf(), cache)
            .with_manifest(cargo_toml);

        let result = scanner.scan(&ctx).await.unwrap();
        assert_eq!(result.total_packages, 3);

        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "serde" && p.version == "1.0"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "tokio" && p.version == "1.0"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "tempfile" && p.version == "3.0"));
    }
}
