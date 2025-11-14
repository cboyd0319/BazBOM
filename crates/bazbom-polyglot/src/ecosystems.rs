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
            "Rust" => "cargo",
            "Ruby" => "gem",
            "PHP" => "composer",
            other => other,
        };

        if let Some(ref ns) = self.namespace {
            format!("pkg:{}/{}@{}", ecosystem, ns, self.version)
        } else {
            format!("pkg:{}/{}@{}", ecosystem, self.name, self.version)
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
