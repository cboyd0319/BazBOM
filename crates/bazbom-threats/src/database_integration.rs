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

    /// Query OSV for malicious packages (stub - would make HTTP requests in real implementation)
    pub fn query_malicious_packages(&self, ecosystem: &str) -> Result<Vec<MaliciousPackageEntry>> {
        // NOTE: This is a stub implementation
        // In a real implementation, this would:
        // 1. Make HTTP requests to OSV API
        // 2. Filter for malicious package advisories
        // 3. Parse responses into MaliciousPackageEntry
        
        // For now, return empty list
        log::info!("Would query OSV API for {} malicious packages", ecosystem);
        Ok(Vec::new())
    }
}

impl Default for OsvClient {
    fn default() -> Self {
        Self::new()
    }
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

    /// Query GHSA for malicious packages (stub)
    pub fn query_malicious_packages(&self, ecosystem: &str) -> Result<Vec<MaliciousPackageEntry>> {
        // NOTE: This is a stub implementation
        // In a real implementation, this would:
        // 1. Make GraphQL requests to GitHub API
        // 2. Query security advisories
        // 3. Filter for malicious packages
        // 4. Parse into MaliciousPackageEntry
        
        log::info!("Would query GHSA API for {} malicious packages", ecosystem);
        Ok(Vec::new())
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
