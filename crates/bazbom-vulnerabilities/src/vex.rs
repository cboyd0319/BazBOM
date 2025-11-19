//! VEX (Vulnerability Exploitability eXchange) support
//!
//! Implements OpenVEX format for documenting vulnerability exploitability.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// VEX Document (OpenVEX format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexDocument {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    pub author: String,
    pub timestamp: String,
    pub version: u32,
    pub statements: Vec<VexStatement>,
}

/// VEX Statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexStatement {
    pub vulnerability: VexVulnerability,
    #[serde(default)]
    pub products: Vec<VexProduct>,
    pub status: VexStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justification: Option<VexJustification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impact_statement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action_statement: Option<String>,
}

/// Vulnerability identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexVulnerability {
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Product identifier (typically a PURL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VexProduct {
    #[serde(rename = "@id")]
    pub id: String,
}

/// VEX Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VexStatus {
    NotAffected,
    Affected,
    Fixed,
    UnderInvestigation,
}

impl std::fmt::Display for VexStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VexStatus::NotAffected => write!(f, "not_affected"),
            VexStatus::Affected => write!(f, "affected"),
            VexStatus::Fixed => write!(f, "fixed"),
            VexStatus::UnderInvestigation => write!(f, "under_investigation"),
        }
    }
}

/// VEX Justification (for not_affected status)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VexJustification {
    ComponentNotPresent,
    VulnerableCodeNotPresent,
    VulnerableCodeNotInExecutePath,
    VulnerableCodeCannotBeControlledByAdversary,
    InlineMitigationsAlreadyExist,
}

impl std::fmt::Display for VexJustification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VexJustification::ComponentNotPresent => write!(f, "component_not_present"),
            VexJustification::VulnerableCodeNotPresent => write!(f, "vulnerable_code_not_present"),
            VexJustification::VulnerableCodeNotInExecutePath => {
                write!(f, "vulnerable_code_not_in_execute_path")
            }
            VexJustification::VulnerableCodeCannotBeControlledByAdversary => {
                write!(f, "vulnerable_code_cannot_be_controlled_by_adversary")
            }
            VexJustification::InlineMitigationsAlreadyExist => {
                write!(f, "inline_mitigations_already_exist")
            }
        }
    }
}

impl VexDocument {
    /// Create a new VEX document
    pub fn new(id: &str, author: &str) -> Self {
        Self {
            context: "https://openvex.dev/ns/v0.2.0".to_string(),
            id: id.to_string(),
            author: author.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: 1,
            statements: Vec::new(),
        }
    }

    /// Add a statement to the document
    pub fn add_statement(&mut self, statement: VexStatement) {
        self.statements.push(statement);
    }

    /// Load VEX document from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read VEX file: {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse VEX file: {}", path.display()))
    }

    /// Save VEX document to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write VEX file: {}", path.display()))
    }

    /// Load all VEX documents from a directory
    pub fn load_all(dir: &Path) -> Result<Vec<Self>> {
        let mut documents = Vec::new();

        if !dir.exists() {
            return Ok(documents);
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                match Self::load(&path) {
                    Ok(doc) => documents.push(doc),
                    Err(e) => {
                        tracing::warn!("Failed to load VEX file {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(documents)
    }
}

impl VexStatement {
    /// Create a new VEX statement
    pub fn new(cve: &str, status: VexStatus) -> Self {
        Self {
            vulnerability: VexVulnerability {
                id: cve.to_string(),
                name: None,
                description: None,
            },
            products: Vec::new(),
            status,
            justification: None,
            impact_statement: None,
            action_statement: None,
        }
    }

    /// Set justification
    pub fn with_justification(mut self, justification: VexJustification) -> Self {
        self.justification = Some(justification);
        self
    }

    /// Set impact statement
    pub fn with_impact_statement(mut self, statement: &str) -> Self {
        self.impact_statement = Some(statement.to_string());
        self
    }

    /// Add product
    pub fn with_product(mut self, purl: &str) -> Self {
        self.products.push(VexProduct {
            id: purl.to_string(),
        });
        self
    }
}

/// VEX filter for applying VEX statements to vulnerability findings
pub struct VexFilter {
    /// CVE -> status mapping
    cve_status: HashMap<String, VexStatus>,
    /// CVE -> (package PURL -> status) for package-specific VEX
    package_status: HashMap<String, HashMap<String, VexStatus>>,
}

impl VexFilter {
    /// Create a new VEX filter from documents
    pub fn from_documents(documents: &[VexDocument]) -> Self {
        let mut cve_status = HashMap::new();
        let mut package_status: HashMap<String, HashMap<String, VexStatus>> = HashMap::new();

        for doc in documents {
            for statement in &doc.statements {
                let cve = &statement.vulnerability.id;

                if statement.products.is_empty() {
                    // Global VEX for CVE (applies to all packages)
                    cve_status.insert(cve.clone(), statement.status);
                } else {
                    // Package-specific VEX
                    for product in &statement.products {
                        package_status
                            .entry(cve.clone())
                            .or_default()
                            .insert(product.id.clone(), statement.status);
                    }
                }
            }
        }

        Self {
            cve_status,
            package_status,
        }
    }

    /// Load VEX filter from directory
    pub fn load(dir: &Path) -> Result<Self> {
        let documents = VexDocument::load_all(dir)?;
        Ok(Self::from_documents(&documents))
    }

    /// Check if a vulnerability should be suppressed
    pub fn should_suppress(&self, cve: &str, package_purl: Option<&str>) -> bool {
        // Check package-specific VEX first
        if let Some(purl) = package_purl {
            if let Some(pkg_statuses) = self.package_status.get(cve) {
                if let Some(status) = pkg_statuses.get(purl) {
                    return matches!(status, VexStatus::NotAffected | VexStatus::Fixed);
                }
            }
        }

        // Check global CVE VEX
        if let Some(status) = self.cve_status.get(cve) {
            return matches!(status, VexStatus::NotAffected | VexStatus::Fixed);
        }

        false
    }

    /// Get VEX status for a vulnerability
    pub fn get_status(&self, cve: &str, package_purl: Option<&str>) -> Option<VexStatus> {
        // Check package-specific VEX first
        if let Some(purl) = package_purl {
            if let Some(pkg_statuses) = self.package_status.get(cve) {
                if let Some(status) = pkg_statuses.get(purl) {
                    return Some(*status);
                }
            }
        }

        // Check global CVE VEX
        self.cve_status.get(cve).copied()
    }

    /// Get all suppressed CVEs
    pub fn suppressed_cves(&self) -> HashSet<String> {
        let mut suppressed = HashSet::new();

        for (cve, status) in &self.cve_status {
            if matches!(status, VexStatus::NotAffected | VexStatus::Fixed) {
                suppressed.insert(cve.clone());
            }
        }

        suppressed
    }
}

/// Filter vulnerabilities using VEX statements
pub fn filter_vulnerabilities(
    vulnerabilities: Vec<crate::Vulnerability>,
    vex_filter: &VexFilter,
) -> (Vec<crate::Vulnerability>, Vec<crate::Vulnerability>) {
    let mut kept = Vec::new();
    let mut suppressed = Vec::new();

    for vuln in vulnerabilities {
        // Get CVE from id or aliases
        let cve_id = &vuln.id;

        // Construct PURL from first affected package if available
        let purl = vuln.affected.first().map(|a| {
            format!("pkg:{}/{}",
                a.ecosystem.to_lowercase().replace("pypi", "pypi").replace("crates.io", "cargo"),
                a.package
            )
        });
        let purl_ref = purl.as_deref();

        // Check main ID
        let mut should_suppress = vex_filter.should_suppress(cve_id, purl_ref);

        // Also check aliases (CVE might be in aliases if main id is OSV/GHSA)
        if !should_suppress {
            for alias in &vuln.aliases {
                if vex_filter.should_suppress(alias, purl_ref) {
                    should_suppress = true;
                    break;
                }
            }
        }

        if should_suppress {
            suppressed.push(vuln);
        } else {
            kept.push(vuln);
        }
    }

    (kept, suppressed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vex_document_creation() {
        let mut doc = VexDocument::new(
            "https://example.com/vex/2025-001",
            "security@example.com",
        );

        let statement = VexStatement::new("CVE-2023-12345", VexStatus::NotAffected)
            .with_justification(VexJustification::VulnerableCodeNotInExecutePath)
            .with_impact_statement("The vulnerable code path is never reached")
            .with_product("pkg:cargo/bazbom@6.5.0");

        doc.add_statement(statement);

        assert_eq!(doc.statements.len(), 1);
        assert_eq!(doc.statements[0].vulnerability.id, "CVE-2023-12345");
        assert_eq!(doc.statements[0].status, VexStatus::NotAffected);
    }

    #[test]
    fn test_vex_filter() {
        let mut doc = VexDocument::new("test", "test@test.com");

        // Global VEX
        doc.add_statement(VexStatement::new("CVE-2023-11111", VexStatus::NotAffected));

        // Package-specific VEX
        doc.add_statement(
            VexStatement::new("CVE-2023-22222", VexStatus::NotAffected)
                .with_product("pkg:cargo/foo@1.0.0")
        );

        let filter = VexFilter::from_documents(&[doc]);

        // Global VEX suppresses any package
        assert!(filter.should_suppress("CVE-2023-11111", None));
        assert!(filter.should_suppress("CVE-2023-11111", Some("pkg:cargo/bar@2.0.0")));

        // Package-specific VEX only suppresses matching package
        assert!(filter.should_suppress("CVE-2023-22222", Some("pkg:cargo/foo@1.0.0")));
        assert!(!filter.should_suppress("CVE-2023-22222", Some("pkg:cargo/bar@2.0.0")));
        assert!(!filter.should_suppress("CVE-2023-22222", None));
    }

    #[test]
    fn test_vex_serialization() {
        let doc = VexDocument::new("test-id", "test@example.com");
        let json = serde_json::to_string_pretty(&doc).unwrap();

        assert!(json.contains("\"@context\""));
        assert!(json.contains("\"@id\""));
        assert!(json.contains("openvex.dev"));
    }
}
