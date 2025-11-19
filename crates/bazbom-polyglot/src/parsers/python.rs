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
                parse_poetry_lock(lockfile_path, &mut result)?;
            }
            "Pipfile.lock" => {
                parse_pipfile_lock(lockfile_path, &mut result)?;
            }
            "requirements-lock.txt" => {
                parse_requirements_file(lockfile_path, &mut result)?;
            }
            _ => {
                // Unknown lockfile, try requirements.txt
                if let Some(ref manifest) = ecosystem.manifest_file {
                    parse_requirements_file(manifest, &mut result)?;
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
                parse_requirements_file(manifest_path, &mut result)?;
            }
            "pyproject.toml" => {
                // For now, just note that we found it
                // Full implementation would parse pyproject.toml
                eprintln!("Warning: pyproject.toml parsing not yet fully implemented");
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
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PoetryLock = toml::from_str(&content).context("Failed to parse poetry.lock")?;

    for pkg in &lock.package {
        result.add_package(Package {
            name: pkg.name.clone(),
            version: pkg.version.clone(),
            ecosystem: "PyPI".to_string(),
            namespace: None,
            dependencies: pkg.dependencies.keys().cloned().collect(),
            license: None,
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
    result: &mut EcosystemScanResult,
) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)?;
    let lock: PipfileLock =
        serde_json::from_str(&content).context("Failed to parse Pipfile.lock")?;

    // Parse default (production) dependencies
    for (name, dep) in &lock.default {
        let version = dep.version.trim_start_matches("==").to_string();
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

    // Parse dev dependencies
    for (name, dep) in &lock.dev {
        let version = dep.version.trim_start_matches("==").to_string();
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

    Ok(())
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
