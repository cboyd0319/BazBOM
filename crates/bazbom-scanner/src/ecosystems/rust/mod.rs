//! Rust Cargo parser
//!
//! Parses Cargo.toml and Cargo.lock files

use crate::detection::Ecosystem;
use crate::types::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use cargo_lock::Lockfile;
use std::collections::HashMap;
use std::error::Error;
use std::fs;

/// Scan Rust ecosystem with reachability analysis
pub async fn scan(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(
        "Rust".to_string(),
        ecosystem.root_path.display().to_string(),
    );

    // Parse Cargo.lock if available (most accurate)
    if let Some(ref lockfile_path) = ecosystem.lockfile {
        parse_cargo_lock(lockfile_path, &mut result)?;
    } else if let Some(ref manifest_path) = ecosystem.manifest_file {
        // Fallback to Cargo.toml (less accurate, just shows dependencies)
        parse_cargo_toml(manifest_path, &mut result)?;
    }

    // Run reachability analysis
    if let Err(e) = analyze_reachability(ecosystem, &mut result) {
        eprintln!("Warning: Rust reachability analysis failed: {}", e);
        // Print full error chain for debugging
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }
        // Continue without reachability data
    }

    Ok(result)
}

/// Analyze reachability for Rust project
fn analyze_reachability(ecosystem: &Ecosystem, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_reachability::rust::analyze_rust_project;

    let report = analyze_rust_project(&ecosystem.root_path)
        .context("Failed to run Rust reachability analysis")?;

    // Build map of vulnerable packages -> reachability
    let mut vulnerable_packages_reachable = HashMap::new();

    // Function-to-vulnerability mapping is done by bazbom-polyglot's reachability_integration module
    // Here we provide a conservative baseline: if ANY function is reachable, package is considered reachable
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
    lockfile_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let lockfile = Lockfile::load(lockfile_path).context("Failed to parse Cargo.lock")?;

    for package in &lockfile.packages {
        // Extract name and version
        let name = package.name.as_str().to_string();
        let version = package.version.to_string();

        // Extract source to determine namespace
        // Sources look like: "registry+https://github.com/rust-lang/crates.io-index"
        let namespace = package.source.as_ref().and_then(|src| {
            let src_str = src.to_string();
            if src_str.contains("crates.io") {
                Some("crates.io".to_string())
            } else if src_str.contains("github.com") {
                // Extract github org/user from source
                // e.g., "git+https://github.com/tokio-rs/tokio?branch=master#abc123"
                src_str
                    .split("github.com/")
                    .nth(1)
                    .and_then(|s| s.split('/').next())
                    .map(|org| format!("github.com/{}", org))
            } else {
                None
            }
        });

        // Extract dependencies
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
    manifest_path: &std::path::Path,
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
/// Handles both string versions ("1.0") and table versions ({ version = "1.0", features = [...] })
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
    use tempfile::TempDir;

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

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Rust,
            temp.path().to_path_buf(),
            Some(cargo_toml),
            None,
        );

        let result = scan(&ecosystem).await.unwrap();
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

    #[tokio::test]
    async fn test_parse_cargo_lock() {
        let temp = TempDir::new().unwrap();
        let cargo_lock = temp.path().join("Cargo.lock");

        fs::write(
            &cargo_lock,
            r#"
# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 3

[[package]]
name = "anyhow"
version = "1.0.75"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "a4668cab20f66d8d020e1fbc0ebe47217433c1b6c8f2040faf858554e394ace6"

[[package]]
name = "serde"
version = "1.0.193"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "25dd9975e68d0cb5aa1120c288333fc98731bd1dd12f561e468ea4728c042b89"
dependencies = [
 "serde_derive",
]

[[package]]
name = "serde_derive"
version = "1.0.193"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "43576ca501357b9b071ac53cdc7da8ef0cbd9493d8df094cd821777ea6e894d3"
"#,
        )
        .unwrap();

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Rust,
            temp.path().to_path_buf(),
            None,
            Some(cargo_lock),
        );

        let result = scan(&ecosystem).await.unwrap();
        assert_eq!(result.total_packages, 3);

        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "anyhow" && p.version == "1.0.75"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "serde" && p.version == "1.0.193"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "serde_derive" && p.version == "1.0.193"));

        // Check that serde has serde_derive as dependency
        let serde_pkg = result.packages.iter().find(|p| p.name == "serde").unwrap();
        assert!(serde_pkg.dependencies.contains(&"serde_derive".to_string()));
    }
}
