//! Data loading for the explore command (TUI)
//!
//! This module handles loading SBOM files (SPDX/CycloneDX) and findings JSON
//! to populate the interactive dependency explorer.

use anyhow::{Context, Result};
use bazbom_tui::{Dependency, Vulnerability};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load dependencies from SBOM file or findings JSON
pub fn load_dependencies(
    sbom_path: Option<&str>,
    findings_path: Option<&str>,
) -> Result<Vec<Dependency>> {
    // Try findings first if provided
    if let Some(path) = findings_path {
        return load_from_findings(path);
    }

    // Otherwise try SBOM
    if let Some(path) = sbom_path {
        return load_from_sbom(path);
    }

    // If neither provided, try to find them in current directory
    if let Ok(deps) = try_find_and_load() {
        return Ok(deps);
    }

    // Return mock data as fallback
    Ok(get_mock_dependencies())
}

/// Load dependencies from findings JSON
fn load_from_findings(path: &str) -> Result<Vec<Dependency>> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read findings file: {}", path))?;

    let findings: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse findings JSON: {}", path))?;

    let mut deps = Vec::new();

    // Parse dependencies from findings
    if let Some(vulns) = findings["vulnerabilities"].as_array() {
        for vuln in vulns {
            let package_name = vuln["package"]["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let package_version = vuln["package"]["version"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();

            // Find or create dependency entry
            let dep = deps
                .iter_mut()
                .find(|d: &&mut Dependency| d.name == package_name && d.version == package_version);

            let vulnerability = Vulnerability {
                cve: vuln["cve"].as_str().unwrap_or("UNKNOWN").to_string(),
                severity: vuln["severity"].as_str().unwrap_or("UNKNOWN").to_string(),
                cvss: vuln["cvss"].as_f64().unwrap_or(0.0) as f32,
                fixed_version: vuln["fixed_version"].as_str().map(|s| s.to_string()),
            };

            if let Some(d) = dep {
                d.vulnerabilities.push(vulnerability);
            } else {
                deps.push(Dependency {
                    name: package_name,
                    version: package_version,
                    scope: vuln["scope"].as_str().unwrap_or("compile").to_string(),
                    vulnerabilities: vec![vulnerability],
                });
            }
        }
    }

    // Add dependencies without vulnerabilities if present in summary
    if let Some(all_deps) = findings["dependencies"].as_array() {
        for dep in all_deps {
            let name = dep["name"].as_str().unwrap_or("unknown").to_string();
            let version = dep["version"].as_str().unwrap_or("unknown").to_string();

            // Only add if not already present
            if !deps.iter().any(|d| d.name == name && d.version == version) {
                deps.push(Dependency {
                    name,
                    version,
                    scope: dep["scope"].as_str().unwrap_or("compile").to_string(),
                    vulnerabilities: vec![],
                });
            }
        }
    }

    Ok(deps)
}

/// Load dependencies from SBOM file (SPDX or CycloneDX)
fn load_from_sbom(path: &str) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("Failed to read SBOM file: {}", path))?;

    let sbom: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse SBOM JSON: {}", path))?;

    let mut deps = Vec::new();

    // Try SPDX format
    if let Some(packages) = sbom["packages"].as_array() {
        for pkg in packages {
            let name = pkg["name"].as_str().unwrap_or("unknown").to_string();
            let version = pkg["versionInfo"].as_str().unwrap_or("unknown").to_string();

            deps.push(Dependency {
                name,
                version,
                scope: "compile".to_string(), // SBOM doesn't have scope info
                vulnerabilities: vec![],
            });
        }
    }
    // Try CycloneDX format
    else if let Some(components) = sbom["components"].as_array() {
        for comp in components {
            let name = comp["name"].as_str().unwrap_or("unknown").to_string();
            let version = comp["version"].as_str().unwrap_or("unknown").to_string();

            deps.push(Dependency {
                name,
                version,
                scope: "compile".to_string(),
                vulnerabilities: vec![],
            });
        }
    }

    Ok(deps)
}

/// Try to find and load SBOM/findings from common locations
fn try_find_and_load() -> Result<Vec<Dependency>> {
    let common_paths = vec![
        "sca_findings.json",
        ".bazbom/sca_findings.json",
        "sbom.spdx.json",
        "sbom.json",
        ".bazbom/sbom.spdx.json",
    ];

    for path in common_paths {
        if Path::new(path).exists() {
            if path.contains("findings") {
                if let Ok(deps) = load_from_findings(path) {
                    return Ok(deps);
                }
            } else if path.contains("sbom") {
                if let Ok(deps) = load_from_sbom(path) {
                    return Ok(deps);
                }
            }
        }
    }

    anyhow::bail!("No SBOM or findings file found")
}

/// Get mock dependencies for demo/fallback
pub fn get_mock_dependencies() -> Vec<Dependency> {
    vec![
        Dependency {
            name: "org.springframework:spring-web".to_string(),
            version: "5.3.20".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![Vulnerability {
                cve: "CVE-2024-22243".to_string(),
                severity: "HIGH".to_string(),
                cvss: 7.5,
                fixed_version: Some("5.3.31".to_string()),
            }],
        },
        Dependency {
            name: "org.apache.logging.log4j:log4j-core".to_string(),
            version: "2.14.1".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![Vulnerability {
                cve: "CVE-2021-44228".to_string(),
                severity: "CRITICAL".to_string(),
                cvss: 10.0,
                fixed_version: Some("2.21.1".to_string()),
            }],
        },
        Dependency {
            name: "com.google.guava:guava".to_string(),
            version: "31.1-jre".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![],
        },
        Dependency {
            name: "com.fasterxml.jackson.core:jackson-databind".to_string(),
            version: "2.13.0".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![Vulnerability {
                cve: "CVE-2024-12345".to_string(),
                severity: "HIGH".to_string(),
                cvss: 8.1,
                fixed_version: Some("2.16.0".to_string()),
            }],
        },
        Dependency {
            name: "commons-io:commons-io".to_string(),
            version: "2.7".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![Vulnerability {
                cve: "CVE-2024-23456".to_string(),
                severity: "MEDIUM".to_string(),
                cvss: 5.3,
                fixed_version: Some("2.15.0".to_string()),
            }],
        },
        Dependency {
            name: "org.apache.commons:commons-lang3".to_string(),
            version: "3.12.0".to_string(),
            scope: "compile".to_string(),
            vulnerabilities: vec![],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_mock_dependencies() {
        let deps = get_mock_dependencies();
        assert!(!deps.is_empty());
        assert!(deps.len() >= 3);
    }

    #[test]
    fn test_load_from_findings_json() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "vulnerabilities": [
                    {{
                        "package": {{ "name": "test-pkg", "version": "1.0.0" }},
                        "cve": "CVE-2024-0001",
                        "severity": "HIGH",
                        "cvss": 7.5,
                        "scope": "compile",
                        "fixed_version": "1.1.0"
                    }}
                ]
            }}"#
        )
        .unwrap();

        let result = load_from_findings(file.path().to_str().unwrap());
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "test-pkg");
        assert_eq!(deps[0].vulnerabilities.len(), 1);
        assert_eq!(deps[0].vulnerabilities[0].cve, "CVE-2024-0001");
    }

    #[test]
    fn test_load_from_spdx_sbom() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "spdxVersion": "SPDX-2.3",
                "packages": [
                    {{
                        "name": "test-package",
                        "versionInfo": "1.0.0"
                    }}
                ]
            }}"#
        )
        .unwrap();

        let result = load_from_sbom(file.path().to_str().unwrap());
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "test-package");
        assert_eq!(deps[0].version, "1.0.0");
    }

    #[test]
    fn test_load_from_cyclonedx_sbom() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{
                "bomFormat": "CycloneDX",
                "components": [
                    {{
                        "name": "test-component",
                        "version": "2.0.0"
                    }}
                ]
            }}"#
        )
        .unwrap();

        let result = load_from_sbom(file.path().to_str().unwrap());
        assert!(result.is_ok());

        let deps = result.unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "test-component");
        assert_eq!(deps[0].version, "2.0.0");
    }
}
