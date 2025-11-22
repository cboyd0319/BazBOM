//! Ecosystem data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Package information from any ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>, // e.g., "@types" for npm, "github.com/user" for Go
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependencies: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

impl Package {
    /// Get PURL (Package URL) for this package
    pub fn purl(&self) -> String {
        let ecosystem = match self.ecosystem.as_str() {
            "Node.js/npm" | "npm" => "npm",
            "Python" | "pip" => "pypi",
            "Go" => "golang",
            "Rust" | "cargo" => "cargo",
            "Ruby" | "gem" | "RubyGems" => "gem",
            "PHP" | "composer" => "composer",
            "Maven" | "Gradle" => "maven",
            other => other,
        };

        // Special handling for Maven/Gradle: namespace is groupId, name is "groupId:artifactId"
        if self.ecosystem == "Maven" || self.ecosystem == "Gradle" {
            let artifact = if self.name.contains(':') {
                self.name.split(':').nth(1).unwrap_or(&self.name)
            } else {
                &self.name
            };
            if let Some(ref groupid) = self.namespace {
                return format!(
                    "pkg:{}/{}/{}@{}",
                    ecosystem, groupid, artifact, self.version
                );
            } else {
                return format!("pkg:{}/{}@{}", ecosystem, self.name, self.version);
            }
        }

        // Special handling for Go: namespace is import path prefix, combine with name for full path
        if self.ecosystem == "Go" {
            if let Some(ref ns) = self.namespace {
                return format!("pkg:{}/{}/{}@{}", ecosystem, ns, self.name, self.version);
            } else {
                return format!("pkg:{}/{}@{}", ecosystem, self.name, self.version);
            }
        }

        if let Some(ref ns) = self.namespace {
            // Check if namespace is an ecosystem identifier (like "crates.io") or a package scope (like "@types")
            if ns == "crates.io"
                || ns == "rubygems.org"
                || ns == "packagist.org"
                || ns.starts_with("packagist.org/")
            {
                // Ecosystem namespace - don't include in purl, just use package name
                format!("pkg:{}/{}@{}", ecosystem, self.name, self.version)
            } else if ns.starts_with('@') || self.ecosystem == "npm" {
                // npm scoped package - include namespace in path
                format!("pkg:{}/{}/{}@{}", ecosystem, ns, self.name, self.version)
            } else {
                // Other namespaces (like Go import paths) - combine with name
                format!("pkg:{}/{}/{}@{}", ecosystem, ns, self.name, self.version)
            }
        } else {
            format!("pkg:{}/{}@{}", ecosystem, self.name, self.version)
        }
    }

    /// Get download URL for this package from its ecosystem registry
    pub fn download_url(&self) -> Option<String> {
        match self.ecosystem.as_str() {
            "Maven" => {
                // Maven: https://repo1.maven.org/maven2/{group-as-path}/{artifact}/{version}/{artifact}-{version}.jar
                // Parse group from namespace or name (format: "group:artifact" or namespace="group", name="artifact")
                let (group, artifact) = if let Some(ref ns) = self.namespace {
                    (ns.as_str(), self.name.as_str())
                } else if self.name.contains(':') {
                    let parts: Vec<&str> = self.name.split(':').collect();
                    if parts.len() == 2 {
                        (parts[0], parts[1])
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };

                let group_path = group.replace('.', "/");
                Some(format!(
                    "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.jar",
                    group_path, artifact, self.version, artifact, self.version
                ))
            }
            "Node.js/npm" | "npm" => {
                // npm: https://registry.npmjs.org/{package}/-/{package}-{version}.tgz
                let package_name = if let Some(ref ns) = self.namespace {
                    format!("{}/{}", ns, self.name)
                } else {
                    self.name.clone()
                };
                Some(format!(
                    "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                    package_name,
                    self.name.trim_start_matches('@'),
                    self.version
                ))
            }
            "Python" | "pip" => {
                // PyPI: https://pypi.org/project/{package}/{version}/
                Some(format!(
                    "https://pypi.org/project/{}/{}/",
                    self.name, self.version
                ))
            }
            "Rust" => {
                // Cargo: https://crates.io/crates/{crate}/{version}
                Some(format!(
                    "https://crates.io/crates/{}/{}",
                    self.name, self.version
                ))
            }
            "Go" => {
                // Go: https://proxy.golang.org/{module}/@v/{version}.zip
                let module = if let Some(ref ns) = self.namespace {
                    format!("{}/{}", ns, self.name)
                } else {
                    self.name.clone()
                };
                Some(format!(
                    "https://proxy.golang.org/{}/@v/{}.zip",
                    module, self.version
                ))
            }
            "Ruby" => {
                // RubyGems: https://rubygems.org/gems/{gem}/versions/{version}
                Some(format!(
                    "https://rubygems.org/gems/{}/versions/{}",
                    self.name, self.version
                ))
            }
            "PHP" => {
                // Composer: https://packagist.org/packages/{vendor}/{package}#{version}
                if let Some(ref ns) = self.namespace {
                    Some(format!(
                        "https://packagist.org/packages/{}/{}#{}",
                        ns, self.name, self.version
                    ))
                } else if self.name.contains('/') {
                    Some(format!(
                        "https://packagist.org/packages/{}#{}",
                        self.name, self.version
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get checksum URL or API endpoint for this package (for SHA256 verification)
    pub fn checksum_url(&self) -> Option<String> {
        match self.ecosystem.as_str() {
            "Maven" => {
                // Maven: https://repo1.maven.org/maven2/{group-as-path}/{artifact}/{version}/{artifact}-{version}.jar.sha256
                let (group, artifact) = if let Some(ref ns) = self.namespace {
                    (ns.as_str(), self.name.as_str())
                } else if self.name.contains(':') {
                    let parts: Vec<&str> = self.name.split(':').collect();
                    if parts.len() == 2 {
                        (parts[0], parts[1])
                    } else {
                        return None;
                    }
                } else {
                    return None;
                };

                let group_path = group.replace('.', "/");
                Some(format!(
                    "https://repo1.maven.org/maven2/{}/{}/{}/{}-{}.jar.sha256",
                    group_path, artifact, self.version, artifact, self.version
                ))
            }
            "Node.js/npm" | "npm" => {
                // npm: Registry API at https://registry.npmjs.org/{package}/{version}
                let package_name = if let Some(ref ns) = self.namespace {
                    format!("{}/{}", ns, self.name)
                } else {
                    self.name.clone()
                };
                Some(format!(
                    "https://registry.npmjs.org/{}/{}",
                    package_name, self.version
                ))
            }
            "Python" | "pip" => {
                // PyPI: JSON API at https://pypi.org/pypi/{package}/{version}/json
                Some(format!(
                    "https://pypi.org/pypi/{}/{}/json",
                    self.name, self.version
                ))
            }
            "Rust" => {
                // Cargo: API at https://crates.io/api/v1/crates/{crate}/{version}
                Some(format!(
                    "https://crates.io/api/v1/crates/{}/{}",
                    self.name, self.version
                ))
            }
            "Go" => {
                // Go: Proxy has .info and .mod files
                let module = if let Some(ref ns) = self.namespace {
                    format!("{}/{}", ns, self.name)
                } else {
                    self.name.clone()
                };
                Some(format!(
                    "https://proxy.golang.org/{}/@v/{}.info",
                    module, self.version
                ))
            }
            "Ruby" => {
                // RubyGems: API at https://rubygems.org/api/v1/versions/{gem}.json
                Some(format!(
                    "https://rubygems.org/api/v1/versions/{}.json",
                    self.name
                ))
            }
            "PHP" => {
                // Composer/Packagist: API at https://repo.packagist.org/p2/{vendor}/{package}.json
                if let Some(ref ns) = self.namespace {
                    Some(format!(
                        "https://repo.packagist.org/p2/{}/{}.json",
                        ns, self.name
                    ))
                } else if self.name.contains('/') {
                    let parts: Vec<&str> = self.name.split('/').collect();
                    if parts.len() == 2 {
                        Some(format!(
                            "https://repo.packagist.org/p2/{}/{}.json",
                            parts[0], parts[1]
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String, // CVE-YYYY-NNNNN or GHSA-xxxx
    pub package_name: String,
    pub package_version: String,
    pub severity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cvss_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_version: Option<String>,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub references: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_date: Option<String>,
}

/// Reachability analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReachabilityData {
    pub analyzed: bool,
    pub total_functions: usize,
    pub reachable_functions: usize,
    pub unreachable_functions: usize,
    /// Map of vulnerable package -> is_reachable
    pub vulnerable_packages_reachable: HashMap<String, bool>,
}

/// Results from scanning an ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemScanResult {
    pub ecosystem: String,
    pub root_path: String,
    pub packages: Vec<Package>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub total_packages: usize,
    pub total_vulnerabilities: usize,
    /// Reachability analysis results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reachability: Option<ReachabilityData>,
}

impl EcosystemScanResult {
    pub fn new(ecosystem: String, root_path: String) -> Self {
        Self {
            ecosystem,
            root_path,
            packages: Vec::new(),
            vulnerabilities: Vec::new(),
            total_packages: 0,
            total_vulnerabilities: 0,
            reachability: None,
        }
    }

    pub fn add_package(&mut self, package: Package) {
        self.packages.push(package);
        self.total_packages += 1;
    }

    pub fn add_vulnerability(&mut self, vuln: Vulnerability) {
        self.vulnerabilities.push(vuln);
        self.total_vulnerabilities += 1;
    }
}
