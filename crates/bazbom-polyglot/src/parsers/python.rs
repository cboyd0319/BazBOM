//! Python ecosystem parser
//!
//! Parses requirements.txt, poetry.lock, and Pipfile.lock

use crate::detection::Ecosystem;
use crate::ecosystems::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

/// poetry.lock structure (TOML)
#[derive(Debug, Deserialize)]
struct PoetryLock {
    #[serde(default)]
    package: Vec<PoetryPackage>,
}

#[derive(Debug, Deserialize)]
struct PoetryPackage {
    name: String,
    version: String,
    description: Option<String>,
    #[allow(dead_code)]
    category: Option<String>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

/// Pipfile.lock structure (JSON)
#[derive(Debug, Deserialize)]
struct PipfileLock {
    #[serde(default)]
    default: HashMap<String, PipfileDependency>,
    #[serde(default, rename = "develop")]
    dev: HashMap<String, PipfileDependency>,
}

#[derive(Debug, Deserialize)]
struct PipfileDependency {
    version: String,
}

/// pyproject.toml structure (PEP 621 / Poetry format)
#[derive(Debug, Deserialize)]
struct PyProjectToml {
    project: Option<PyProjectProject>,
    tool: Option<PyProjectTool>,
}

#[derive(Debug, Deserialize)]
struct PyProjectProject {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
    #[serde(default, rename = "optional-dependencies")]
    optional_dependencies: HashMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PyProjectTool {
    poetry: Option<PoetryConfig>,
}

#[derive(Debug, Deserialize)]
struct PoetryConfig {
    #[serde(default)]
    dependencies: HashMap<String, serde_json::Value>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, serde_json::Value>,
}

/// Read license from Python package metadata
/// Tries to find and parse METADATA or PKG-INFO from site-packages
fn read_python_license(root_path: &std::path::Path, package_name: &str) -> Option<String> {
    // Common site-packages locations to check
    let possible_paths = vec![
        // Virtual environment
        root_path.join("venv/lib").to_path_buf(),
        root_path.join(".venv/lib").to_path_buf(),
        // System Python (macOS/Linux)
        std::path::PathBuf::from("/usr/local/lib"),
        std::path::PathBuf::from("/usr/lib"),
    ];

    // Normalize package name (replace - with _)
    let normalized_name = package_name.replace('-', "_");

    for base_path in possible_paths {
        // Try to find python*/site-packages directories
        if let Ok(entries) = fs::read_dir(&base_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.file_name().and_then(|n| n.to_str()).map_or(false, |n| n.starts_with("python")) {
                    let site_packages = path.join("site-packages");
                    if site_packages.exists() {
                        // Try .dist-info directory
                        if let Ok(pkg_entries) = fs::read_dir(&site_packages) {
                            for pkg_entry in pkg_entries.flatten() {
                                let pkg_path = pkg_entry.path();
                                let dir_name = pkg_path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                                // Match package-version.dist-info
                                if dir_name.starts_with(&normalized_name) && dir_name.ends_with(".dist-info") {
                                    let metadata_file = pkg_path.join("METADATA");
                                    if let Ok(content) = fs::read_to_string(&metadata_file) {
                                        // Parse METADATA file for License: field
                                        for line in content.lines() {
                                            if line.starts_with("License:") {
                                                if let Some(license) = line.strip_prefix("License:").map(|s| s.trim()) {
                                                    if !license.is_empty() && license != "UNKNOWN" {
                                                        return Some(license.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}

/// Scan Python ecosystem
pub async fn scan(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(
        "Python".to_string(),
        ecosystem.root_path.display().to_string(),
    );

    // Try to parse lockfiles first (most accurate)
    if let Some(ref lockfile_path) = ecosystem.lockfile {
        let lockfile_name = lockfile_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        match lockfile_name {
            "poetry.lock" => {
                parse_poetry_lock(lockfile_path, &ecosystem.root_path, &mut result)?;
            }
            "Pipfile.lock" => {
                parse_pipfile_lock(lockfile_path, &ecosystem.root_path, &mut result)?;
            }
            "requirements-lock.txt" => {
                parse_requirements_file(lockfile_path, &ecosystem.root_path, &mut result)?;
            }
            _ => {
                // Unknown lockfile, try requirements.txt
                if let Some(ref manifest) = ecosystem.manifest_file {
                    parse_requirements_file(manifest, &ecosystem.root_path, &mut result)?;
                }
            }
        }
    } else if let Some(ref manifest_path) = ecosystem.manifest_file {
        // No lockfile, use manifest
        let manifest_name = manifest_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        match manifest_name {
            "requirements.txt" => {
                parse_requirements_file(manifest_path, &ecosystem.root_path, &mut result)?;
            }
            "pyproject.toml" => {
                parse_pyproject_toml(manifest_path, &mut result)?;
            }
            "Pipfile" => {
                eprintln!("Warning: Pipfile found but no Pipfile.lock - run 'pipenv lock' for accurate versions");
            }
            _ => {}
        }
    }

    // Run reachability analysis
    if let Err(e) = analyze_reachability(ecosystem, &mut result) {
        eprintln!("Warning: Python reachability analysis failed: {}", e);
    }

    Ok(result)
}

/// Analyze reachability for Python project
fn analyze_reachability(ecosystem: &Ecosystem, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_python_reachability::analyze_python_project;

    let report = analyze_python_project(&ecosystem.root_path)?;
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

/// Parse requirements.txt format
/// Supports: package==version, package>=version, package~=version
fn parse_requirements_file(
    file_path: &std::path::Path,
    root_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(file_path).context("Failed to read requirements file")?;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip -e (editable installs) and -r (recursive requirements)
        if line.starts_with("-e") || line.starts_with("-r") || line.starts_with("--") {
            continue;
        }

        // Parse package specification
        if let Some((name, version)) = parse_requirement_line(line) {
            let license = read_python_license(root_path, name);

            result.add_package(Package {
                name: name.to_string(),
                version: version.to_string(),
                ecosystem: "PyPI".to_string(),
                namespace: None,
                dependencies: Vec::new(),
                license,
                description: None,
                homepage: None,
                repository: None,
            });
        }
    }

    Ok(())
}

/// Parse a single requirement line
/// Examples:
///   Django==3.2.0
///   requests>=2.25.0
///   pytest~=7.0
///   six==1.16.0 ; python_version >= "3.6"
fn parse_requirement_line(line: &str) -> Option<(&str, &str)> {
    // Remove environment markers (e.g., ; python_version >= "3.6")
    let line = line.split(';').next()?.trim();

    // Remove inline comments (e.g., # CVE-2019-14234)
    let line = line.split('#').next()?.trim();

    // Try different operators in order of specificity
    for op in &["===", "==", "~=", ">=", "<=", ">", "<", "!="] {
        if let Some(idx) = line.find(op) {
            let name = line[..idx].trim();
            let version = line[idx + op.len()..].trim();

            // Clean up version (remove extras like [dev])
            let version = version.split('[').next().unwrap_or(version).trim();

            if !name.is_empty() && !version.is_empty() {
                return Some((name, version));
            }
        }
    }

    None
}

/// Parse poetry.lock (TOML format)
fn parse_poetry_lock(
    lockfile_path: &std::path::Path,
    root_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PoetryLock = toml::from_str(&content).context("Failed to parse poetry.lock")?;

    for pkg in &lock.package {
        let license = read_python_license(root_path, &pkg.name);

        result.add_package(Package {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            ecosystem: "PyPI".to_string(),
            namespace: None,
            dependencies: pkg.dependencies.keys().cloned().collect(),
            license,
            description: pkg.description.clone(),
            homepage: None,
            repository: None,
        });
    }

    Ok(())
}

/// Parse Pipfile.lock (JSON format)
fn parse_pipfile_lock(
    lockfile_path: &std::path::Path,
    root_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PipfileLock =
        serde_json::from_str(&content).context("Failed to parse Pipfile.lock")?;

    // Parse default (production) dependencies
    for (name, dep) in &lock.default {
        let version = dep.version.trim_start_matches("==").to_string();
        let license = read_python_license(root_path, name);

        result.add_package(Package {
            name: name.clone(),
            version,
            ecosystem: "PyPI".to_string(),
            namespace: None,
            dependencies: Vec::new(),
            license,
            description: None,
            homepage: None,
            repository: None,
        });
    }

    // Parse dev dependencies
    for (name, dep) in &lock.dev {
        let version = dep.version.trim_start_matches("==").to_string();
        let license = read_python_license(root_path, name);

        result.add_package(Package {
            name: name.clone(),
            version,
            ecosystem: "PyPI".to_string(),
            namespace: None,
            dependencies: Vec::new(),
            license,
            description: None,
            homepage: None,
            repository: None,
        });
    }

    Ok(())
}

/// Parse pyproject.toml (PEP 621 or Poetry format)
fn parse_pyproject_toml(
    file_path: &std::path::Path,
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(file_path).context("Failed to read pyproject.toml")?;
    let pyproject: PyProjectToml =
        toml::from_str(&content).context("Failed to parse pyproject.toml")?;

    // Parse PEP 621 format ([project] section)
    if let Some(project) = pyproject.project {
        // Parse main dependencies
        for dep_spec in &project.dependencies {
            if let Some((name, version)) = parse_dependency_spec(dep_spec) {
                result.add_package(Package {
                    name: name.to_string(),
                    version: version.to_string(),
                    ecosystem: "PyPI".to_string(),
                    namespace: None,
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }

        // Parse optional dependencies (extras)
        for (_extra_name, deps) in &project.optional_dependencies {
            for dep_spec in deps {
                if let Some((name, version)) = parse_dependency_spec(dep_spec) {
                    result.add_package(Package {
                        name: name.to_string(),
                        version: version.to_string(),
                        ecosystem: "PyPI".to_string(),
                        namespace: None,
                        dependencies: Vec::new(),
                        license: None,
                        description: None,
                        homepage: None,
                        repository: None,
                    });
                }
            }
        }
    }

    // Parse Poetry format ([tool.poetry] section)
    if let Some(tool) = pyproject.tool {
        if let Some(poetry) = tool.poetry {
            // Parse Poetry dependencies
            for (name, version_spec) in &poetry.dependencies {
                // Skip python version constraint
                if name == "python" {
                    continue;
                }

                // Handle different Poetry version formats:
                // - Simple: package = "^1.2.3"
                // - Complex: package = { version = "^1.2.3", ... }
                let version = match version_spec {
                    serde_json::Value::String(v) => extract_poetry_version(v),
                    serde_json::Value::Object(obj) => {
                        obj.get("version")
                            .and_then(|v| v.as_str())
                            .map(extract_poetry_version)
                            .unwrap_or_else(|| "latest".to_string())
                    }
                    _ => "latest".to_string(),
                };

                result.add_package(Package {
                    name: name.clone(),
                    version,
                    ecosystem: "PyPI".to_string(),
                    namespace: None,
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }

            // Parse Poetry dev dependencies
            for (name, version_spec) in &poetry.dev_dependencies {
                let version = match version_spec {
                    serde_json::Value::String(v) => extract_poetry_version(v),
                    serde_json::Value::Object(obj) => {
                        obj.get("version")
                            .and_then(|v| v.as_str())
                            .map(extract_poetry_version)
                            .unwrap_or_else(|| "latest".to_string())
                    }
                    _ => "latest".to_string(),
                };

                result.add_package(Package {
                    name: name.clone(),
                    version,
                    ecosystem: "PyPI".to_string(),
                    namespace: None,
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }
    }

    Ok(())
}

/// Parse PEP 508 dependency specifier (e.g., "protobuf>=5", "django==3.2.0")
/// Returns (name, version) tuple, extracting the minimum/exact version
fn parse_dependency_spec(spec: &str) -> Option<(&str, &str)> {
    // Remove environment markers (e.g., ; python_version >= "3.6")
    let spec = spec.split(';').next()?.trim();

    // Try different operators (same as parse_requirement_line)
    for op in &["===", "==", "~=", ">=", "<=", ">", "<", "!="] {
        if let Some(idx) = spec.find(op) {
            let name = spec[..idx].trim();
            let version = spec[idx + op.len()..].trim();

            // Clean up version (remove extras like [dev])
            let version = version.split('[').next().unwrap_or(version).trim();
            // Remove trailing commas from compound specs (e.g., ">=5,<6")
            let version = version.split(',').next().unwrap_or(version).trim();

            if !name.is_empty() && !version.is_empty() {
                return Some((name, version));
            }
        }
    }

    // If no version specifier, it's just a package name (use "latest")
    if !spec.is_empty() && !spec.contains(|c: char| c.is_whitespace() || c == '[') {
        return Some((spec, "latest"));
    }

    None
}

/// Extract version from Poetry version specifier
/// Handles: "^1.2.3" -> "1.2.3", "~1.2" -> "1.2", ">=1.0" -> "1.0"
fn extract_poetry_version(spec: &str) -> String {
    let spec = spec.trim();

    // Handle Poetry caret (^) and tilde (~) operators
    if let Some(stripped) = spec.strip_prefix('^').or_else(|| spec.strip_prefix('~')) {
        return stripped.to_string();
    }

    // Handle comparison operators
    for op in &["===", "==", ">=", "<=", ">", "<", "!="] {
        if let Some(idx) = spec.find(op) {
            let version = spec[idx + op.len()..].trim();
            // Remove trailing comma from compound specs
            let version = version.split(',').next().unwrap_or(version).trim();
            if !version.is_empty() {
                return version.to_string();
            }
        }
    }

    // Return as-is if no special operators
    spec.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_requirement_line() {
        assert_eq!(
            parse_requirement_line("Django==3.2.0"),
            Some(("Django", "3.2.0"))
        );
        assert_eq!(
            parse_requirement_line("requests>=2.25.0"),
            Some(("requests", "2.25.0"))
        );
        assert_eq!(
            parse_requirement_line("pytest~=7.0"),
            Some(("pytest", "7.0"))
        );
        assert_eq!(
            parse_requirement_line("six==1.16.0 ; python_version >= \"3.6\""),
            Some(("six", "1.16.0"))
        );
        assert_eq!(parse_requirement_line("# comment"), None);
    }

    #[tokio::test]
    async fn test_parse_requirements_txt() {
        let temp = TempDir::new().unwrap();
        let requirements = temp.path().join("requirements.txt");

        fs::write(
            &requirements,
            r#"
# This is a comment
Django==3.2.0
requests>=2.25.0
pytest~=7.0

# Another comment
six==1.16.0 ; python_version >= "3.6"
"#,
        )
        .unwrap();

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Python,
            temp.path().to_path_buf(),
            Some(requirements),
            None,
        );

        let result = scan(&ecosystem).await.unwrap();
        assert_eq!(result.total_packages, 4);
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "Django" && p.version == "3.2.0"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "requests" && p.version == "2.25.0"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "pytest" && p.version == "7.0"));
        assert!(result
            .packages
            .iter()
            .any(|p| p.name == "six" && p.version == "1.16.0"));
    }
}
