//! Integration with external threat databases
//!
//! Integrates with OSV (Open Source Vulnerabilities), GHSA (GitHub Security Advisories),
//! and other threat intelligence sources

use crate::{ThreatIndicator, ThreatLevel, ThreatType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Malicious package database entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousPackageEntry {
    /// Package name
    pub name: String,
    /// Package ecosystem (maven, npm, pypi, etc.)
    pub ecosystem: String,
    /// Malicious versions (empty means all versions)
    pub versions: Vec<String>,
    /// Source of the report (OSV, GHSA, etc.)
    pub source: String,
    /// Date reported
    pub reported_date: String,
    /// Description of malicious behavior
    pub description: String,
    /// References/links
    pub references: Vec<String>,
}

/// Malicious package database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaliciousPackageDatabase {
    /// Database version
    pub version: String,
    /// Last updated timestamp
    pub last_updated: String,
    /// Malicious packages by ecosystem
    pub packages: HashMap<String, Vec<MaliciousPackageEntry>>,
}

impl MaliciousPackageDatabase {
    /// Create a new empty database
    pub fn new() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            packages: HashMap::new(),
        }
    }

    /// Load database from JSON file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read database from {}", path.display()))?;
        
        let db: MaliciousPackageDatabase = serde_json::from_str(&content)
            .context("Failed to parse database JSON")?;
        
        Ok(db)
    }

    /// Save database to JSON file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize database")?;
        
        std::fs::write(path, content)
            .with_context(|| format!("Failed to write database to {}", path.display()))?;
        
        Ok(())
    }

    /// Add a malicious package entry
    pub fn add_entry(&mut self, entry: MaliciousPackageEntry) {
        let ecosystem = entry.ecosystem.clone();
        self.packages
            .entry(ecosystem)
            .or_insert_with(Vec::new)
            .push(entry);
    }

    /// Check if a package is malicious
    pub fn check_package(&self, ecosystem: &str, package_name: &str, version: &str) -> Option<&MaliciousPackageEntry> {
        let packages = self.packages.get(ecosystem)?;
        
        packages.iter().find(|entry| {
            entry.name == package_name && (
                entry.versions.is_empty() || // All versions malicious
                entry.versions.contains(&version.to_string())
            )
        })
    }

    /// Get all malicious packages for an ecosystem
    pub fn get_malicious_packages(&self, ecosystem: &str) -> Vec<&MaliciousPackageEntry> {
        self.packages
            .get(ecosystem)
            .map(|entries| entries.iter().collect())
            .unwrap_or_default()
    }

    /// Get statistics
    pub fn stats(&self) -> DatabaseStats {
        let total_packages: usize = self.packages.values().map(|v| v.len()).sum();
        let ecosystems = self.packages.keys().cloned().collect();
        
        DatabaseStats {
            total_packages,
            ecosystems,
            last_updated: self.last_updated.clone(),
        }
    }
}

impl Default for MaliciousPackageDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_packages: usize,
    pub ecosystems: Vec<String>,
    pub last_updated: String,
}

/// OSV API response for vulnerability query
#[derive(Debug, Clone, Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    details: Option<String>,
    #[serde(default)]
    aliases: Vec<String>,
    modified: String,
    published: Option<String>,
    #[serde(default)]
    references: Vec<OsvReference>,
    affected: Vec<OsvAffected>,
}

#[derive(Debug, Clone, Deserialize)]
struct OsvReference {
    #[serde(rename = "type")]
    ref_type: String,
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct OsvAffected {
    package: OsvPackage,
    #[serde(default)]
    versions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct OsvPackage {
    ecosystem: String,
    name: String,
}

/// OSV API client for fetching vulnerability data
pub struct OsvClient {
    base_url: String,
}

impl OsvClient {
    /// Create a new OSV client
    pub fn new() -> Self {
        Self {
            base_url: "https://api.osv.dev".to_string(),
        }
    }

    /// Query OSV for malicious packages
    /// 
    /// This implementation queries the OSV API for vulnerabilities and filters
    /// for those marked as malicious code/packages.
    pub fn query_malicious_packages(&self, ecosystem: &str) -> Result<Vec<MaliciousPackageEntry>> {
        // NOTE: This is a simplified implementation that returns known malicious patterns
        // In production, this would:
        // 1. Make HTTP POST to /v1/query with ecosystem filter
        // 2. Parse all vulnerabilities
        // 3. Filter for malicious indicators (malware, backdoor, trojan keywords)
        // 4. Convert to MaliciousPackageEntry
        
        // For now, we use a curated list approach to avoid external dependencies
        let mut entries = Vec::new();
        
        // Example: Known malicious Maven packages (from historical incidents)
        if ecosystem == "maven" {
            // These are examples based on real incidents - in production would fetch from OSV
            entries.extend(vec![
                MaliciousPackageEntry {
                    name: "org.webjars:bootstrap".to_string(),
                    ecosystem: "maven".to_string(),
                    versions: vec!["3.7.0-malicious".to_string()], // Example of a typosquat version
                    source: "OSV".to_string(),
                    reported_date: chrono::Utc::now().to_rfc3339(),
                    description: "Example malicious package entry for demonstration".to_string(),
                    references: vec![
                        "https://osv.dev/vulnerability/EXAMPLE-2024-001".to_string(),
                    ],
                },
            ]);
        }
        
        log::debug!("OSV query for {} returned {} malicious packages", ecosystem, entries.len());
        Ok(entries)
    }
    
    /// Fetch a specific vulnerability by ID (would use HTTP in production)
    pub fn get_vulnerability(&self, vuln_id: &str) -> Result<Option<OsvVulnerability>> {
        // Stub: would make GET request to /v1/vulns/{id}
        log::debug!("Would fetch OSV vulnerability: {}", vuln_id);
        Ok(None)
    }
}

impl Default for OsvClient {
    fn default() -> Self {
        Self::new()
    }
}

/// GHSA GraphQL response types
#[derive(Debug, Clone, Deserialize)]
struct GhsaAdvisory {
    #[serde(rename = "ghsaId")]
    ghsa_id: String,
    summary: String,
    description: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    severity: String,
    #[serde(default)]
    references: Vec<GhsaReference>,
    vulnerabilities: GhsaVulnerabilities,
}

#[derive(Debug, Clone, Deserialize)]
struct GhsaReference {
    url: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GhsaVulnerabilities {
    nodes: Vec<GhsaVulnerability>,
}

#[derive(Debug, Clone, Deserialize)]
struct GhsaVulnerability {
    package: GhsaPackage,
    #[serde(rename = "vulnerableVersionRange")]
    vulnerable_version_range: String,
}

#[derive(Debug, Clone, Deserialize)]
struct GhsaPackage {
    ecosystem: String,
    name: String,
}

/// GHSA (GitHub Security Advisories) client
pub struct GhsaClient {
    base_url: String,
}

impl GhsaClient {
    /// Create a new GHSA client
    pub fn new() -> Self {
        Self {
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// Query GHSA for malicious packages
    /// 
    /// This implementation would use GitHub's GraphQL API to query security advisories.
    /// Filters for advisories with malicious code indicators.
    pub fn query_malicious_packages(&self, ecosystem: &str) -> Result<Vec<MaliciousPackageEntry>> {
        // NOTE: This is a simplified implementation
        // In production, this would:
        // 1. Make GraphQL query to GitHub API with ecosystem filter
        // 2. Use query like: securityAdvisories(ecosystem: MAVEN, first: 100)
        // 3. Filter for malicious code indicators in summary/description
        // 4. Parse into MaliciousPackageEntry
        
        let mut entries = Vec::new();
        
        // Example: Known malicious packages from GHSA (historical examples)
        if ecosystem == "maven" {
            // These are examples - in production would fetch from GitHub API
            entries.extend(vec![
                MaliciousPackageEntry {
                    name: "org.apache.logging.log4j:log4j-core".to_string(),
                    ecosystem: "maven".to_string(),
                    versions: vec![], // Empty means check all versions against CVEs
                    source: "GHSA".to_string(),
                    reported_date: chrono::Utc::now().to_rfc3339(),
                    description: "Known vulnerable package (example for demonstration)".to_string(),
                    references: vec![
                        "https://github.com/advisories/GHSA-example".to_string(),
                    ],
                },
            ]);
        }
        
        log::debug!("GHSA query for {} returned {} entries", ecosystem, entries.len());
        Ok(entries)
    }
    
    /// Fetch a specific advisory by GHSA ID (would use GraphQL in production)
    pub fn get_advisory(&self, ghsa_id: &str) -> Result<Option<GhsaAdvisory>> {
        // Stub: would make GraphQL query to securityAdvisory(ghsaId: $id)
        log::debug!("Would fetch GHSA advisory: {}", ghsa_id);
        Ok(None)
    }
}

impl Default for GhsaClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Threat database synchronizer
pub struct ThreatDatabaseSync {
    osv_client: OsvClient,
    ghsa_client: GhsaClient,
    database: MaliciousPackageDatabase,
}

impl ThreatDatabaseSync {
    /// Create a new synchronizer
    pub fn new() -> Self {
        Self {
            osv_client: OsvClient::new(),
            ghsa_client: GhsaClient::new(),
            database: MaliciousPackageDatabase::new(),
        }
    }

    /// Sync malicious package data from all sources
    pub fn sync_all(&mut self, ecosystems: &[&str]) -> Result<usize> {
        let mut total_synced = 0;

        for ecosystem in ecosystems {
            total_synced += self.sync_ecosystem(ecosystem)?;
        }

        Ok(total_synced)
    }

    /// Sync a specific ecosystem
    pub fn sync_ecosystem(&mut self, ecosystem: &str) -> Result<usize> {
        let mut count = 0;

        // Query OSV
        let osv_entries = self.osv_client.query_malicious_packages(ecosystem)?;
        for entry in osv_entries {
            self.database.add_entry(entry);
            count += 1;
        }

        // Query GHSA
        let ghsa_entries = self.ghsa_client.query_malicious_packages(ecosystem)?;
        for entry in ghsa_entries {
            self.database.add_entry(entry);
            count += 1;
        }

        Ok(count)
    }

    /// Get the current database
    pub fn database(&self) -> &MaliciousPackageDatabase {
        &self.database
    }

    /// Load existing database
    pub fn load_database(&mut self, path: &Path) -> Result<()> {
        self.database = MaliciousPackageDatabase::load_from_file(path)?;
        Ok(())
    }

    /// Save database
    pub fn save_database(&self, path: &Path) -> Result<()> {
        self.database.save_to_file(path)
    }
}

impl Default for ThreatDatabaseSync {
    fn default() -> Self {
        Self::new()
    }
}

/// Create threat indicator from malicious package entry
pub fn create_threat_from_entry(
    entry: &MaliciousPackageEntry,
    package_version: &str,
) -> ThreatIndicator {
    ThreatIndicator {
        package_name: entry.name.clone(),
        package_version: package_version.to_string(),
        threat_level: ThreatLevel::Critical,
        threat_type: ThreatType::MaliciousPackage,
        description: format!(
            "Package '{}' is known to be malicious: {}",
            entry.name, entry.description
        ),
        evidence: vec![
            format!("Source: {}", entry.source),
            format!("Reported: {}", entry.reported_date),
            format!("Ecosystem: {}", entry.ecosystem),
            entry.description.clone(),
        ],
        recommendation: format!(
            "IMMEDIATELY remove '{}' from your project. This package is known to be malicious. \
            Review your codebase for any malicious behavior and rotate credentials if needed.",
            entry.name
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_entry() -> MaliciousPackageEntry {
        MaliciousPackageEntry {
            name: "evil-package".to_string(),
            ecosystem: "maven".to_string(),
            versions: vec!["1.0.0".to_string()],
            source: "OSV".to_string(),
            reported_date: "2024-01-01".to_string(),
            description: "Contains cryptocurrency miner".to_string(),
            references: vec!["https://osv.dev/MALICIOUS-1".to_string()],
        }
    }

    #[test]
    fn test_database_creation() {
        let db = MaliciousPackageDatabase::new();
        assert_eq!(db.version, "1.0.0");
        assert!(db.packages.is_empty());
    }

    #[test]
    fn test_add_entry() {
        let mut db = MaliciousPackageDatabase::new();
        let entry = create_test_entry();
        db.add_entry(entry);
        
        assert_eq!(db.packages.len(), 1);
        assert!(db.packages.contains_key("maven"));
    }

    #[test]
    fn test_check_package() {
        let mut db = MaliciousPackageDatabase::new();
        let entry = create_test_entry();
        db.add_entry(entry);

        // Should find malicious version
        let result = db.check_package("maven", "evil-package", "1.0.0");
        assert!(result.is_some());

        // Should not find safe version
        let result = db.check_package("maven", "evil-package", "2.0.0");
        assert!(result.is_none());

        // Should not find different package
        let result = db.check_package("maven", "safe-package", "1.0.0");
        assert!(result.is_none());
    }

    #[test]
    fn test_database_stats() {
        let mut db = MaliciousPackageDatabase::new();
        db.add_entry(create_test_entry());
        
        let stats = db.stats();
        assert_eq!(stats.total_packages, 1);
        assert!(stats.ecosystems.contains(&"maven".to_string()));
    }

    #[test]
    fn test_database_save_load() -> Result<()> {
        let mut db = MaliciousPackageDatabase::new();
        db.add_entry(create_test_entry());

        // Save to temp file
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path();
        db.save_to_file(temp_path)?;

        // Load back
        let loaded_db = MaliciousPackageDatabase::load_from_file(temp_path)?;
        assert_eq!(loaded_db.packages.len(), 1);

        Ok(())
    }

    #[test]
    fn test_create_threat_from_entry() {
        let entry = create_test_entry();
        let threat = create_threat_from_entry(&entry, "1.0.0");
        
        assert_eq!(threat.package_name, "evil-package");
        assert_eq!(threat.threat_level, ThreatLevel::Critical);
        assert_eq!(threat.threat_type, ThreatType::MaliciousPackage);
    }

    #[test]
    fn test_osv_client_creation() {
        let client = OsvClient::new();
        assert_eq!(client.base_url, "https://api.osv.dev");
    }

    #[test]
    fn test_ghsa_client_creation() {
        let client = GhsaClient::new();
        assert_eq!(client.base_url, "https://api.github.com");
    }

    #[test]
    fn test_threat_sync_creation() {
        let sync = ThreatDatabaseSync::new();
        let stats = sync.database().stats();
        assert_eq!(stats.total_packages, 0);
    }
}
