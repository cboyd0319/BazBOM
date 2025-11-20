//! Node.js/npm ecosystem scanner
//!
//! Parses package.json and lockfiles (package-lock.json, yarn.lock, pnpm-lock.yaml)

use crate::cache::LicenseCache;
use crate::scanner::{License, LicenseContext, ScanContext, Scanner};
use crate::types::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// npm ecosystem scanner
pub struct NpmScanner;

impl Default for NpmScanner {
    fn default() -> Self {
        Self
    }
}

impl NpmScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for NpmScanner {
    fn name(&self) -> &str {
        "npm"
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("package.json").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result = EcosystemScanResult::new(
            "Node.js/npm".to_string(),
            ctx.root.display().to_string(),
        );

        // Parse package.json if provided
        if let Some(ref manifest_path) = ctx.manifest {
            let content = fs::read_to_string(manifest_path).context("Failed to read package.json")?;
            let package_json: PackageJson =
                serde_json::from_str(&content).context("Failed to parse package.json")?;

            // If we have a lockfile, parse it for exact versions
            if let Some(ref lockfile_path) = ctx.lockfile {
                let lockfile_name = lockfile_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                match lockfile_name {
                    "package-lock.json" => {
                        parse_package_lock(lockfile_path, &ctx.root, &mut result, &ctx.cache)?;
                    }
                    "yarn.lock" => {
                        parse_yarn_lock(lockfile_path, &ctx.root, &mut result, &ctx.cache)?;
                    }
                    "pnpm-lock.yaml" => {
                        parse_pnpm_lock(lockfile_path, &ctx.root, &mut result, &ctx.cache)?;
                    }
                    _ => {
                        // No recognized lockfile, fallback to package.json dependencies
                        parse_package_json_deps(&package_json, &mut result);
                    }
                }
            } else {
                // No lockfile, use package.json
                parse_package_json_deps(&package_json, &mut result);
            }
        }

        // Run reachability analysis
        if let Err(e) = analyze_reachability(&ctx.root, &mut result) {
            eprintln!("Warning: npm reachability analysis failed: {}", e);
        }

        Ok(result)
    }

    fn fetch_license_uncached(&self, ctx: &LicenseContext) -> License {
        // Parse namespace from package name
        let (namespace, package_name) = if ctx.package.starts_with('@') {
            let parts: Vec<&str> = ctx.package.splitn(2, '/').collect();
            if parts.len() == 2 {
                (Some(parts[0].to_string()), parts[1])
            } else {
                (None, ctx.package)
            }
        } else {
            (None, ctx.package)
        };

        // Try to read license from node_modules/{package}/package.json
        if let Some(license_str) = read_license_from_node_modules(ctx.root, package_name, &namespace) {
            License::Spdx(license_str)
        } else {
            License::Unknown
        }
    }
}

/// package.json structure
#[derive(Debug, Deserialize)]
struct PackageJson {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    description: Option<String>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    #[allow(dead_code)]
    dev_dependencies: HashMap<String, String>,
    license: Option<serde_json::Value>, // Can be string or object
}

/// package-lock.json structure (simplified)
#[derive(Debug, Deserialize)]
struct PackageLockJson {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

/// Analyze reachability for npm/Node.js project
fn analyze_reachability(root: &Path, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_reachability::js::analyze_js_project;

    let report = analyze_js_project(root)?;
    let mut vulnerable_packages_reachable = HashMap::new();

    for package in &result.packages {
        let key = format!("{}@{}", package.name, package.version);
        vulnerable_packages_reachable.insert(key, !report.reachable_functions.is_empty());
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

/// Read license from node_modules/{package}/package.json
fn read_license_from_node_modules(
    root_path: &Path,
    package_name: &str,
    namespace: &Option<String>,
) -> Option<String> {
    // Build path to package.json
    let pkg_path = if let Some(ns) = namespace {
        root_path.join("node_modules").join(ns).join(package_name).join("package.json")
    } else {
        root_path.join("node_modules").join(package_name).join("package.json")
    };

    // Try to read and parse package.json
    if let Ok(content) = fs::read_to_string(&pkg_path) {
        if let Ok(pkg_json) = serde_json::from_str::<PackageJson>(&content) {
            return extract_license_string(pkg_json.license);
        }
    }

    None
}

/// Extract license string from serde_json::Value (can be string or object)
fn extract_license_string(license: Option<serde_json::Value>) -> Option<String> {
    match license? {
        serde_json::Value::String(s) => Some(s),
        serde_json::Value::Object(obj) => {
            // license can be {"type": "MIT"} or {"type": "MIT", "url": "..."}
            obj.get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        }
        _ => None,
    }
}

/// Parse package-lock.json (npm v7+)
fn parse_package_lock(
    lockfile_path: &Path,
    root_path: &Path,
    result: &mut EcosystemScanResult,
    cache: &LicenseCache,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PackageLockJson =
        serde_json::from_str(&content).context("Failed to parse package-lock.json")?;

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

                // Use cache for license fetching
                let license_ctx = LicenseContext {
                    root: root_path,
                    package: name,
                    version,
                    cache,
                };
                let license = NpmScanner::new().fetch_license(&license_ctx);

                result.add_package(Package {
                    name: package_name,
                    version: version.clone(),
                    ecosystem: "npm".to_string(),
                    namespace,
                    dependencies: pkg.dependencies.keys().cloned().collect(),
                    license: Some(license.as_spdx()),
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }
    } else if !lock.dependencies.is_empty() {
        // npm v6 uses "dependencies" field
        parse_v6_dependencies(&lock.dependencies, root_path, result, cache, &mut Vec::new());
    }

    Ok(())
}

/// Parse npm v6 style dependencies (recursive)
fn parse_v6_dependencies(
    deps: &HashMap<String, LockfileDependency>,
    root_path: &Path,
    result: &mut EcosystemScanResult,
    cache: &LicenseCache,
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

        // Use cache for license fetching
        let license_ctx = LicenseContext {
            root: root_path,
            package: name,
            version: &dep.version,
            cache,
        };
        let license = NpmScanner::new().fetch_license(&license_ctx);

        result.add_package(Package {
            name: package_name,
            version: dep.version.clone(),
            ecosystem: "npm".to_string(),
            namespace,
            dependencies: dep.requires.keys().cloned().collect(),
            license: Some(license.as_spdx()),
            description: None,
            homepage: None,
            repository: None,
        });

        // Parse nested dependencies
        if !dep.dependencies.is_empty() {
            path.push(name.clone());
            parse_v6_dependencies(&dep.dependencies, root_path, result, cache, path);
            path.pop();
        }
    }
}

/// Parse yarn.lock
///
/// Yarn.lock uses a custom format that looks like:
/// ```text
/// package-name@version-range:
///   version "actual-version"
///   resolved "url"
///   integrity "hash"
///   dependencies:
///     dep1 "version"
///     dep2 "version"
/// ```
fn parse_yarn_lock(
    lockfile_path: &Path,
    root_path: &Path,
    result: &mut EcosystemScanResult,
    cache: &LicenseCache,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path).context("Failed to read yarn.lock")?;

    let mut current_package: Option<String> = None;
    let mut current_version: Option<String> = None;
    let mut current_deps: Vec<String> = Vec::new();
    let mut in_dependencies = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Check if this is a package declaration line (no leading whitespace, ends with :)
        if !line.starts_with(' ') && !line.starts_with('\t') && trimmed.ends_with(':') {
            // Save the previous package if we have one
            if let (Some(pkg_name), Some(version)) =
                (current_package.take(), current_version.take())
            {
                add_yarn_package(result, root_path, &pkg_name, &version, &current_deps, cache);
                current_deps.clear();
            }

            in_dependencies = false;

            // Parse package name from declaration like "@babel/code-frame@^7.0.0:"
            // Can also be multiple specs like "package@^1.0.0, package@^2.0.0:"
            let package_spec = trimmed.trim_end_matches(':');

            // Take the first package spec if there are multiple
            let first_spec = package_spec
                .split(',')
                .next()
                .unwrap_or(package_spec)
                .trim();

            // Extract package name (everything before the last @)
            if let Some(at_pos) = first_spec.rfind('@') {
                // Handle scoped packages like @babel/code-frame@^7.0.0
                let pkg = if let Some(stripped) = first_spec.strip_prefix('@') {
                    // Find the second @ for scoped packages
                    if let Some(second_at) = stripped.find('@') {
                        &first_spec[..second_at + 1]
                    } else {
                        first_spec
                    }
                } else {
                    &first_spec[..at_pos]
                };
                current_package = Some(pkg.to_string());
            }
        } else if trimmed.starts_with("version ") {
            // Extract version like: version "7.18.6"
            if let Some(version_str) = extract_quoted_value(trimmed) {
                current_version = Some(version_str);
            }
        } else if trimmed.starts_with("dependencies:") {
            in_dependencies = true;
        } else if in_dependencies && (line.starts_with("    ") || line.starts_with("\t\t")) {
            // This is a dependency entry like:    "@babel/highlight" "^7.18.6"
            if let Some(dep_name) = extract_dependency_name(trimmed) {
                current_deps.push(dep_name);
            }
        } else if !trimmed.starts_with("resolved ")
            && !trimmed.starts_with("integrity ")
            && !trimmed.is_empty()
        {
            // If we hit a non-dependency field, we're out of dependencies
            if in_dependencies && !line.starts_with("    ") && !line.starts_with("\t\t") {
                in_dependencies = false;
            }
        }
    }

    // Don't forget the last package
    if let (Some(pkg_name), Some(version)) = (current_package, current_version) {
        add_yarn_package(result, root_path, &pkg_name, &version, &current_deps, cache);
    }

    Ok(())
}

/// Extract a quoted value from a yarn.lock line like: version "7.18.6"
fn extract_quoted_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.splitn(2, ' ').collect();
    if parts.len() == 2 {
        let value = parts[1].trim();
        // Remove quotes
        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            return Some(value[1..value.len() - 1].to_string());
        }
    }
    None
}

/// Extract dependency name from a line like:    "@babel/highlight" "^7.18.6"
fn extract_dependency_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if let Some(first_quote) = trimmed.find('"') {
        if let Some(second_quote) = trimmed[first_quote + 1..].find('"') {
            return Some(trimmed[first_quote + 1..first_quote + 1 + second_quote].to_string());
        }
    }
    None
}

/// Add a yarn package to the result
fn add_yarn_package(
    result: &mut EcosystemScanResult,
    root_path: &Path,
    name: &str,
    version: &str,
    deps: &[String],
    cache: &LicenseCache,
) {
    let (namespace, package_name) = if name.starts_with('@') {
        // Scoped package like "@babel/code-frame"
        let parts: Vec<&str> = name.splitn(2, '/').collect();
        if parts.len() == 2 {
            (Some(parts[0].to_string()), parts[1].to_string())
        } else {
            (None, name.to_string())
        }
    } else {
        (None, name.to_string())
    };

    // Use cache for license fetching
    let license_ctx = LicenseContext {
        root: root_path,
        package: name,
        version,
        cache,
    };
    let license = NpmScanner::new().fetch_license(&license_ctx);

    result.add_package(Package {
        name: package_name,
        version: version.to_string(),
        ecosystem: "npm".to_string(),
        namespace,
        dependencies: deps.to_vec(),
        license: Some(license.as_spdx()),
        description: None,
        homepage: None,
        repository: None,
    });
}

/// pnpm-lock.yaml structure (supports both v5 and v6+ formats)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmLockfile {
    #[allow(dead_code)]
    lockfile_version: Option<serde_yaml::Value>,
    // v5 format - top-level dependencies
    #[serde(default)]
    #[allow(dead_code)]
    dependencies: HashMap<String, PnpmDependency>,
    #[serde(default)]
    #[allow(dead_code)]
    dev_dependencies: HashMap<String, PnpmDependency>,
    // v6+ format - importers section with workspace projects
    #[serde(default)]
    #[allow(dead_code)]
    importers: HashMap<String, PnpmImporter>,
    // Both formats have packages section
    #[serde(default)]
    packages: HashMap<String, PnpmPackage>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmImporter {
    #[serde(default)]
    #[allow(dead_code)]
    dependencies: HashMap<String, PnpmDependency>,
    #[serde(default)]
    #[allow(dead_code)]
    dev_dependencies: HashMap<String, PnpmDependency>,
}

#[derive(Debug, Deserialize)]
struct PnpmDependency {
    #[allow(dead_code)]
    specifier: Option<String>,
    #[allow(dead_code)]
    version: String,
}

#[derive(Debug, Deserialize)]
struct PnpmPackage {
    #[allow(dead_code)]
    resolution: Option<serde_yaml::Value>, // Can be string (v5) or object with integrity (v6)
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
    dev: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    engines: Option<serde_yaml::Value>,
    #[serde(default, rename = "peerDependencies")]
    #[allow(dead_code)]
    peer_dependencies: HashMap<String, String>,
}

/// Parse pnpm-lock.yaml
///
/// pnpm uses a YAML format with a "packages" section that contains all resolved dependencies
/// in a flat structure with their full paths as keys.
fn parse_pnpm_lock(
    lockfile_path: &Path,
    root_path: &Path,
    result: &mut EcosystemScanResult,
    cache: &LicenseCache,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path).context("Failed to read pnpm-lock.yaml")?;

    let lockfile: PnpmLockfile =
        serde_yaml::from_str(&content).context("Failed to parse pnpm-lock.yaml")?;

    // Parse packages section - this is where pnpm stores all resolved dependencies
    for (path, package) in &lockfile.packages {
        // Extract package name and version from path like "/@babel/code-frame@7.18.6"
        // or "/express@4.18.2"
        if let Some((name, version)) = parse_pnpm_package_path(path) {
            let (namespace, package_name) = if name.starts_with('@') {
                // Scoped package like "@babel/code-frame"
                let parts: Vec<&str> = name.splitn(2, '/').collect();
                if parts.len() == 2 {
                    (Some(parts[0].to_string()), parts[1].to_string())
                } else {
                    (None, name.to_string())
                }
            } else {
                (None, name.to_string())
            };

            let deps: Vec<String> = package.dependencies.keys().cloned().collect();

            // Use cache for license fetching
            let license_ctx = LicenseContext {
                root: root_path,
                package: &name,
                version: &version,
                cache,
            };
            let license = NpmScanner::new().fetch_license(&license_ctx);

            result.add_package(Package {
                name: package_name,
                version: version.to_string(),
                ecosystem: "npm".to_string(),
                namespace,
                dependencies: deps,
                license: Some(license.as_spdx()),
                description: None,
                homepage: None,
                repository: None,
            });
        }
    }

    Ok(())
}

/// Parse pnpm package path like "/@babel/code-frame@7.18.6" or "/express@4.18.2"
/// Also handles v6 format with trailing colon: "/@babel/code-frame@7.18.6:"
/// Returns (name, version) tuple
fn parse_pnpm_package_path(path: &str) -> Option<(String, String)> {
    // Remove leading slash and trailing colon (v6 format)
    let path = path
        .strip_prefix('/')
        .unwrap_or(path)
        .strip_suffix(':')
        .unwrap_or(path.strip_prefix('/').unwrap_or(path));

    // For scoped packages like "@babel/code-frame@7.18.6"
    if path.starts_with('@') {
        // Find the version separator (the @ after the package name)
        // For @babel/code-frame@7.18.6, we want to split at the last @
        if let Some(last_at) = path.rfind('@') {
            if last_at > 0 {
                let name = &path[..last_at];
                let version = &path[last_at + 1..];

                // Handle version with parentheses like "7.18.6(patch_hash=...)"
                let version = if let Some(paren_pos) = version.find('(') {
                    &version[..paren_pos]
                } else {
                    version
                };

                return Some((name.to_string(), version.to_string()));
            }
        }
    } else {
        // For regular packages like "express@4.18.2"
        if let Some(at_pos) = path.find('@') {
            let name = &path[..at_pos];
            let version = &path[at_pos + 1..];

            // Handle version with parentheses
            let version = if let Some(paren_pos) = version.find('(') {
                &version[..paren_pos]
            } else {
                version
            };

            return Some((name.to_string(), version.to_string()));
        }
    }

    None
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
    use std::sync::Arc;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_npm_scanner_detect() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();

        let scanner = NpmScanner::new();
        assert!(scanner.detect(temp.path()));
    }

    #[tokio::test]
    async fn test_parse_package_json() {
        let temp = TempDir::new().unwrap();
        let package_json = temp.path().join("package.json");

        fs::write(
            &package_json,
            r#"{
            "name": "test-package",
            "version": "1.0.0",
            "dependencies": {
                "express": "^4.18.0",
                "@types/node": "^18.0.0"
            }
        }"#,
        )
        .unwrap();

        let scanner = NpmScanner::new();
        let cache = Arc::new(LicenseCache::new());
        let ctx = ScanContext::new(temp.path().to_path_buf(), cache)
            .with_manifest(package_json);

        let result = scanner.scan(&ctx).await.unwrap();
        assert_eq!(result.total_packages, 2);
        assert!(result.packages.iter().any(|p| p.name == "express"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "node" && p.namespace == Some("@types".to_string())));
    }
}
