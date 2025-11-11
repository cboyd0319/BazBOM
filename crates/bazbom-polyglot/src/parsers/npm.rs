//! Node.js/npm ecosystem parser
//!
//! Parses package.json and lockfiles (package-lock.json, yarn.lock, pnpm-lock.yaml)

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use crate::detection::Ecosystem;
use crate::ecosystems::{EcosystemScanResult, Package};

/// package.json structure
#[derive(Debug, Deserialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, String>,
}

/// package-lock.json structure (simplified)
#[derive(Debug, Deserialize)]
struct PackageLockJson {
    name: Option<String>,
    version: Option<String>,
    #[serde(default)]
    packages: HashMap<String, LockfilePackage>,
    #[serde(default)]
    dependencies: HashMap<String, LockfileDependency>,
}

#[derive(Debug, Deserialize)]
struct LockfilePackage {
    version: Option<String>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(rename = "devOptional")]
    dev_optional: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LockfileDependency {
    version: String,
    #[serde(default)]
    requires: HashMap<String, String>,
    #[serde(default)]
    dependencies: HashMap<String, LockfileDependency>,
}

/// Scan npm ecosystem
pub async fn scan(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(
        "Node.js/npm".to_string(),
        ecosystem.root_path.display().to_string(),
    );

    // Parse package.json
    if let Some(ref manifest_path) = ecosystem.manifest_file {
        let content = fs::read_to_string(manifest_path)
            .context("Failed to read package.json")?;
        let package_json: PackageJson = serde_json::from_str(&content)
            .context("Failed to parse package.json")?;

        // If we have a lockfile, parse it for exact versions
        if let Some(ref lockfile_path) = ecosystem.lockfile {
            let lockfile_name = lockfile_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            match lockfile_name {
                "package-lock.json" => {
                    parse_package_lock(lockfile_path, &mut result)?;
                }
                "yarn.lock" => {
                    parse_yarn_lock(lockfile_path, &mut result)?;
                }
                "pnpm-lock.yaml" => {
                    parse_pnpm_lock(lockfile_path, &mut result)?;
                }
                _ => {
                    // No lockfile, fallback to package.json dependencies
                    parse_package_json_deps(&package_json, &mut result);
                }
            }
        } else {
            // No lockfile, use package.json
            parse_package_json_deps(&package_json, &mut result);
        }
    }

    Ok(result)
}

/// Parse package-lock.json (npm v7+)
fn parse_package_lock(lockfile_path: &std::path::Path, result: &mut EcosystemScanResult) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PackageLockJson = serde_json::from_str(&content)
        .context("Failed to parse package-lock.json")?;

    // npm v7+ uses "packages" field
    if !lock.packages.is_empty() {
        for (path, pkg) in &lock.packages {
            // Skip root package (empty path or "")
            if path.is_empty() || path.is_empty() {
                continue;
            }

            // Extract package name from path (e.g., "node_modules/@types/node" -> "@types/node")
            let name = path.strip_prefix("node_modules/").unwrap_or(path);

            if let Some(version) = &pkg.version {
                let (namespace, package_name) = if name.starts_with('@') {
                    // Scoped package like "@types/node"
                    let parts: Vec<&str> = name.splitn(2, '/').collect();
                    if parts.len() == 2 {
                        (Some(parts[0].to_string()), parts[1].to_string())
                    } else {
                        (None, name.to_string())
                    }
                } else {
                    (None, name.to_string())
                };

                result.add_package(Package {
                    name: package_name,
                    version: version.clone(),
                    ecosystem: "npm".to_string(),
                    namespace,
                    dependencies: pkg.dependencies.keys().cloned().collect(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }
    } else if !lock.dependencies.is_empty() {
        // npm v6 uses "dependencies" field
        parse_v6_dependencies(&lock.dependencies, result, &mut Vec::new());
    }

    Ok(())
}

/// Parse npm v6 style dependencies (recursive)
fn parse_v6_dependencies(
    deps: &HashMap<String, LockfileDependency>,
    result: &mut EcosystemScanResult,
    path: &mut Vec<String>,
) {
    for (name, dep) in deps {
        // Avoid circular dependencies
        if path.contains(name) {
            continue;
        }

        let (namespace, package_name) = if name.starts_with('@') {
            let parts: Vec<&str> = name.splitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[0].to_string()), parts[1].to_string())
            } else {
                (None, name.to_string())
            }
        } else {
            (None, name.to_string())
        };

        result.add_package(Package {
            name: package_name,
            version: dep.version.clone(),
            ecosystem: "npm".to_string(),
            namespace,
            dependencies: dep.requires.keys().cloned().collect(),
            license: None,
            description: None,
            homepage: None,
            repository: None,
        });

        // Parse nested dependencies
        if !dep.dependencies.is_empty() {
            path.push(name.clone());
            parse_v6_dependencies(&dep.dependencies, result, path);
            path.pop();
        }
    }
}

/// Parse yarn.lock (TODO: implement proper yarn.lock parser)
fn parse_yarn_lock(_lockfile_path: &std::path::Path, _result: &mut EcosystemScanResult) -> Result<()> {
    // For now, just read the file and note that we found it
    // TODO: Implement proper yarn.lock parsing (it's a custom format, not JSON)
    eprintln!("Warning: yarn.lock parsing not yet fully implemented");
    Ok(())
}

/// Parse pnpm-lock.yaml (TODO: implement proper pnpm-lock.yaml parser)
fn parse_pnpm_lock(_lockfile_path: &std::path::Path, _result: &mut EcosystemScanResult) -> Result<()> {
    // For now, just read the file and note that we found it
    // TODO: Implement proper pnpm-lock.yaml parsing
    eprintln!("Warning: pnpm-lock.yaml parsing not yet fully implemented");
    Ok(())
}

/// Fallback: parse dependencies from package.json (without exact versions)
fn parse_package_json_deps(package_json: &PackageJson, result: &mut EcosystemScanResult) {
    for (name, version_spec) in &package_json.dependencies {
        let (namespace, package_name) = if name.starts_with('@') {
            let parts: Vec<&str> = name.splitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[0].to_string()), parts[1].to_string())
            } else {
                (None, name.to_string())
            }
        } else {
            (None, name.to_string())
        };

        // Version spec like "^1.2.3" or "~1.2.3" - just strip the prefix
        let version = version_spec.trim_start_matches(['^', '~', '=']).to_string();

        result.add_package(Package {
            name: package_name,
            version,
            ecosystem: "npm".to_string(),
            namespace,
            dependencies: Vec::new(),
            license: None,
            description: None,
            homepage: None,
            repository: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_parse_package_json() {
        let temp = TempDir::new().unwrap();
        let package_json = temp.path().join("package.json");

        fs::write(&package_json, r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "express": "^4.18.0",
                "@types/node": "^18.0.0"
            }
        }"#).unwrap();

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Npm,
            temp.path().to_path_buf(),
            Some(package_json),
            None,
        );

        let result = scan(&ecosystem).await.unwrap();
        assert_eq!(result.total_packages, 2);
        assert!(result.packages.iter().any(|p| p.name == "express"));
        assert!(result.packages.iter().any(|p| p.name == "node" && p.namespace == Some("@types".to_string())));
    }
}
