//! PHP Composer parser
//!
//! Parses composer.json and composer.lock files

use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use crate::detection::Ecosystem;
use crate::ecosystems::{EcosystemScanResult, Package};

/// Scan PHP ecosystem
pub async fn scan(ecosystem: &Ecosystem) -> Result<EcosystemScanResult> {
    let mut result = EcosystemScanResult::new(
        "PHP".to_string(),
        ecosystem.root_path.display().to_string(),
    );

    // Parse composer.lock if available (most accurate)
    if let Some(ref lockfile_path) = ecosystem.lockfile {
        parse_composer_lock(lockfile_path, &mut result)?;
    } else if let Some(ref manifest_path) = ecosystem.manifest_file {
        // Fallback to composer.json (less accurate)
        eprintln!("Warning: composer.json found but no composer.lock - run 'composer install' for accurate versions");
        parse_composer_json(manifest_path, &mut result)?;
    }

    Ok(result)
}

/// Parse composer.lock
/// Format:
/// {
///   "packages": [
///     {
///       "name": "vendor/package",
///       "version": "1.2.3",
///       "require": {
///         "php": "^7.2",
///         "other/package": "^2.0"
///       }
///     }
///   ],
///   "packages-dev": [...]
/// }
fn parse_composer_lock(lockfile_path: &std::path::Path, result: &mut EcosystemScanResult) -> Result<()> {
    let content = fs::read_to_string(lockfile_path)
        .context("Failed to read composer.lock")?;

    let json: Value = serde_json::from_str(&content)
        .context("Failed to parse composer.lock")?;

    // Parse production packages
    if let Some(packages) = json.get("packages").and_then(|p| p.as_array()) {
        for package in packages {
            if let Some(pkg) = parse_composer_package(package) {
                result.add_package(pkg);
            }
        }
    }

    // Parse dev packages
    if let Some(dev_packages) = json.get("packages-dev").and_then(|p| p.as_array()) {
        for package in dev_packages {
            if let Some(pkg) = parse_composer_package(package) {
                result.add_package(pkg);
            }
        }
    }

    Ok(())
}

/// Parse a single package from composer.lock
fn parse_composer_package(package: &Value) -> Option<Package> {
    let name = package.get("name")?.as_str()?.to_string();
    let version = package.get("version")?.as_str()?
        .trim_start_matches('v') // Remove 'v' prefix if present
        .to_string();

    // Extract dependencies from "require" section
    let mut dependencies = Vec::new();
    if let Some(require) = package.get("require").and_then(|r| r.as_object()) {
        for (dep_name, _) in require {
            // Skip PHP itself and PHP extensions
            if dep_name != "php" && !dep_name.starts_with("ext-") {
                dependencies.push(dep_name.clone());
            }
        }
    }

    // Split vendor/package into namespace and name
    let (namespace, _pkg_name) = if let Some(pos) = name.find('/') {
        let vendor = &name[..pos];
        let pkg = &name[pos+1..];
        (Some(format!("packagist.org/{}", vendor)), pkg.to_string())
    } else {
        (Some("packagist.org".to_string()), name.clone())
    };

    Some(Package {
        name: name.clone(), // Keep full vendor/package name
        version,
        ecosystem: "Packagist".to_string(),
        namespace,
        dependencies,
        license: package.get("license")
            .and_then(|l| l.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: package.get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string()),
        homepage: package.get("homepage")
            .and_then(|h| h.as_str())
            .map(|s| s.to_string()),
        repository: None, // Could extract from "source" if needed
    })
}

/// Parse composer.json (basic fallback)
fn parse_composer_json(manifest_path: &std::path::Path, result: &mut EcosystemScanResult) -> Result<()> {
    let content = fs::read_to_string(manifest_path)
        .context("Failed to read composer.json")?;

    let json: Value = serde_json::from_str(&content)
        .context("Failed to parse composer.json")?;

    // Parse "require" section
    if let Some(require) = json.get("require").and_then(|r| r.as_object()) {
        for (name, version_spec) in require {
            // Skip PHP itself and PHP extensions
            if name != "php" && !name.starts_with("ext-") {
                let version = extract_version(version_spec.as_str().unwrap_or("*"));

                let (namespace, _) = if let Some(pos) = name.find('/') {
                    let vendor = &name[..pos];
                    (Some(format!("packagist.org/{}", vendor)), &name[pos+1..])
                } else {
                    (Some("packagist.org".to_string()), name.as_str())
                };

                result.add_package(Package {
                    name: name.clone(),
                    version,
                    ecosystem: "Packagist".to_string(),
                    namespace,
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }
    }

    // Parse "require-dev" section
    if let Some(require_dev) = json.get("require-dev").and_then(|r| r.as_object()) {
        for (name, version_spec) in require_dev {
            if name != "php" && !name.starts_with("ext-") {
                let version = extract_version(version_spec.as_str().unwrap_or("*"));

                let (namespace, _) = if let Some(pos) = name.find('/') {
                    let vendor = &name[..pos];
                    (Some(format!("packagist.org/{}", vendor)), &name[pos+1..])
                } else {
                    (Some("packagist.org".to_string()), name.as_str())
                };

                result.add_package(Package {
                    name: name.clone(),
                    version,
                    ecosystem: "Packagist".to_string(),
                    namespace,
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

/// Extract clean version from version constraint
/// Examples: "^1.2.3" -> "1.2.3", "~2.0" -> "2.0", ">=1.0" -> "1.0"
fn extract_version(version_spec: &str) -> String {
    version_spec
        .trim()
        .split("||") // Handle OR operator first
        .next()
        .unwrap_or("*")
        .trim()
        .split(',')
        .next()
        .unwrap_or("*")
        .trim()
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .trim_start_matches('v')
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_extract_version() {
        assert_eq!(extract_version("^1.2.3"), "1.2.3");
        assert_eq!(extract_version("~2.0"), "2.0");
        assert_eq!(extract_version(">=1.0"), "1.0");
        assert_eq!(extract_version("v3.5.0"), "3.5.0");
        assert_eq!(extract_version("^7.2||^8.0"), "7.2");
    }

    #[test]
    fn test_parse_composer_package() {
        let package_json = r#"{
            "name": "symfony/console",
            "version": "v5.4.0",
            "require": {
                "php": ">=7.2.5",
                "symfony/polyfill-php80": "^1.15"
            },
            "license": ["MIT"],
            "description": "Eases the creation of beautiful command line interfaces"
        }"#;

        let package: Value = serde_json::from_str(package_json).unwrap();
        let result = parse_composer_package(&package).unwrap();

        assert_eq!(result.name, "symfony/console");
        assert_eq!(result.version, "5.4.0");
        assert_eq!(result.ecosystem, "Packagist");
        assert_eq!(result.namespace, Some("packagist.org/symfony".to_string()));
        assert_eq!(result.license, Some("MIT".to_string()));
        assert!(result.dependencies.contains(&"symfony/polyfill-php80".to_string()));
        assert!(!result.dependencies.contains(&"php".to_string())); // PHP should be excluded
    }

    #[tokio::test]
    async fn test_parse_composer_lock() {
        let temp = TempDir::new().unwrap();
        let composer_lock = temp.path().join("composer.lock");

        fs::write(&composer_lock, r#"{
    "packages": [
        {
            "name": "symfony/console",
            "version": "v5.4.0",
            "require": {
                "php": ">=7.2.5",
                "symfony/polyfill-php80": "^1.15"
            },
            "license": ["MIT"]
        },
        {
            "name": "guzzlehttp/guzzle",
            "version": "7.4.0",
            "require": {
                "php": "^7.2.5 || ^8.0"
            }
        }
    ],
    "packages-dev": [
        {
            "name": "phpunit/phpunit",
            "version": "9.5.10",
            "require": {
                "php": ">=7.3"
            }
        }
    ]
}"#).unwrap();

        let ecosystem = Ecosystem::new(
            crate::detection::EcosystemType::Php,
            temp.path().to_path_buf(),
            None,
            Some(composer_lock),
        );

        let result = scan(&ecosystem).await.unwrap();
        assert_eq!(result.total_packages, 3);

        assert!(result.packages.iter().any(|p| p.name == "symfony/console" && p.version == "5.4.0"));
        assert!(result.packages.iter().any(|p| p.name == "guzzlehttp/guzzle" && p.version == "7.4.0"));
        assert!(result.packages.iter().any(|p| p.name == "phpunit/phpunit" && p.version == "9.5.10"));

        // Check dependencies
        let symfony = result.packages.iter().find(|p| p.name == "symfony/console").unwrap();
        assert!(symfony.dependencies.contains(&"symfony/polyfill-php80".to_string()));
    }
}
