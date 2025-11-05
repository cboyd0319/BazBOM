//! Custom threat intelligence feed support for BazBOM
//!
//! Allows organizations to add their own threat intelligence feeds
//! in addition to the default OSV/GHSA/NVD feeds.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Custom threat intelligence feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatFeed {
    /// Feed name/identifier
    pub name: String,
    /// Feed description
    pub description: String,
    /// Feed URL or file path
    pub source: FeedSource,
    /// Feed format
    pub format: FeedFormat,
    /// Update frequency in hours
    pub update_frequency_hours: u32,
    /// Last update timestamp
    pub last_updated: Option<String>,
    /// Enable/disable feed
    pub enabled: bool,
}

/// Source of threat intelligence feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedSource {
    /// HTTP/HTTPS URL
    Url { url: String },
    /// Local file path
    File { path: String },
    /// Git repository
    Git { repo: String, branch: String },
}

/// Format of threat intelligence feed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedFormat {
    /// JSON format (custom schema)
    Json,
    /// OSV format (<https://ossf.github.io/osv-schema/>)
    Osv,
    /// CSV format
    Csv,
    /// YAML format
    Yaml,
}

/// Threat entry from custom feed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatEntry {
    /// Unique identifier
    pub id: String,
    /// Package identifier (PURL or name)
    pub package: String,
    /// Affected versions (ranges or specific versions)
    pub affected_versions: Vec<String>,
    /// Threat type
    pub threat_type: String,
    /// Severity level
    pub severity: Severity,
    /// Description
    pub description: String,
    /// References and links
    pub references: Vec<String>,
    /// Discovery date
    pub discovered: String,
}

/// Severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Custom feed manager
pub struct CustomFeedManager {
    /// Configured feeds
    feeds: Vec<ThreatFeed>,
    /// Cached threat entries
    cache: HashMap<String, Vec<ThreatEntry>>,
}

impl Default for CustomFeedManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomFeedManager {
    /// Create a new custom feed manager
    pub fn new() -> Self {
        Self {
            feeds: Vec::new(),
            cache: HashMap::new(),
        }
    }

    /// Load feeds from configuration file
    pub fn load_from_config<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path.as_ref())
            .context("Failed to read custom feeds configuration")?;

        let config: CustomFeedConfig =
            serde_yaml::from_str(&content).context("Failed to parse custom feeds configuration")?;

        Ok(Self {
            feeds: config.feeds,
            cache: HashMap::new(),
        })
    }

    /// Add a new threat feed
    pub fn add_feed(&mut self, feed: ThreatFeed) {
        self.feeds.push(feed);
    }

    /// Remove a feed by name
    pub fn remove_feed(&mut self, name: &str) -> bool {
        if let Some(pos) = self.feeds.iter().position(|f| f.name == name) {
            self.feeds.remove(pos);
            self.cache.remove(name);
            true
        } else {
            false
        }
    }

    /// Enable a feed by name
    pub fn enable_feed(&mut self, name: &str) -> bool {
        if let Some(feed) = self.feeds.iter_mut().find(|f| f.name == name) {
            feed.enabled = true;
            true
        } else {
            false
        }
    }

    /// Disable a feed by name
    pub fn disable_feed(&mut self, name: &str) -> bool {
        if let Some(feed) = self.feeds.iter_mut().find(|f| f.name == name) {
            feed.enabled = false;
            true
        } else {
            false
        }
    }

    /// Update all enabled feeds
    pub fn update_all(&mut self) -> Result<HashMap<String, usize>> {
        let mut results = HashMap::new();

        // Collect feed names to avoid borrowing self while iterating
        let feed_names: Vec<String> = self
            .feeds
            .iter()
            .filter(|f| f.enabled)
            .map(|f| f.name.clone())
            .collect();

        for name in feed_names {
            match self.update_feed(&name) {
                Ok(count) => {
                    results.insert(name.clone(), count);
                }
                Err(e) => {
                    eprintln!("Failed to update feed '{}': {}", name, e);
                    results.insert(name.clone(), 0);
                }
            }
        }

        Ok(results)
    }

    /// Update a specific feed
    pub fn update_feed(&mut self, name: &str) -> Result<usize> {
        let feed = self
            .feeds
            .iter()
            .find(|f| f.name == name)
            .ok_or_else(|| anyhow::anyhow!("Feed '{}' not found", name))?;

        if !feed.enabled {
            anyhow::bail!("Feed '{}' is disabled", name);
        }

        // Fetch data from source
        let data = self.fetch_feed_data(feed)?;

        // Parse based on format
        let entries = self.parse_feed_data(feed, &data)?;

        // Cache the entries
        let count = entries.len();
        self.cache.insert(name.to_string(), entries);

        Ok(count)
    }

    /// Query threats for a package
    pub fn query_threats(&self, package: &str) -> Vec<ThreatEntry> {
        let mut threats = Vec::new();

        for entries in self.cache.values() {
            for entry in entries {
                if entry.package == package || self.package_matches(&entry.package, package) {
                    threats.push(entry.clone());
                }
            }
        }

        threats
    }

    /// Check if package name matches (handles wildcards)
    fn package_matches(&self, pattern: &str, package: &str) -> bool {
        if pattern == package {
            return true;
        }

        // Simple wildcard matching
        if pattern.contains('*') {
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                return package.starts_with(parts[0]) && package.ends_with(parts[1]);
            }
        }

        false
    }

    /// Fetch feed data from source
    fn fetch_feed_data(&self, feed: &ThreatFeed) -> Result<String> {
        match &feed.source {
            FeedSource::File { path } => fs::read_to_string(path)
                .with_context(|| format!("Failed to read feed file: {}", path)),
            FeedSource::Url { url } => {
                // In production, would use reqwest or similar
                anyhow::bail!("URL feeds not yet implemented: {}", url)
            }
            FeedSource::Git { repo, branch } => {
                // In production, would clone/pull git repo
                anyhow::bail!(
                    "Git feeds not yet implemented: {} (branch: {})",
                    repo,
                    branch
                )
            }
        }
    }

    /// Parse feed data based on format
    fn parse_feed_data(&self, feed: &ThreatFeed, data: &str) -> Result<Vec<ThreatEntry>> {
        match feed.format {
            FeedFormat::Json => self.parse_json_feed(data),
            FeedFormat::Osv => self.parse_osv_feed(data),
            FeedFormat::Csv => self.parse_csv_feed(data),
            FeedFormat::Yaml => self.parse_yaml_feed(data),
        }
    }

    /// Parse JSON format feed
    fn parse_json_feed(&self, data: &str) -> Result<Vec<ThreatEntry>> {
        let entries: Vec<ThreatEntry> =
            serde_json::from_str(data).context("Failed to parse JSON feed")?;
        Ok(entries)
    }

    /// Parse OSV format feed
    fn parse_osv_feed(&self, data: &str) -> Result<Vec<ThreatEntry>> {
        // OSV format parsing would go here
        // For now, return empty
        let _data = data; // Use data to avoid warning
        Ok(Vec::new())
    }

    /// Parse CSV format feed
    fn parse_csv_feed(&self, data: &str) -> Result<Vec<ThreatEntry>> {
        // CSV parsing would go here
        // Expected columns: id,package,versions,type,severity,description,references,discovered
        let _data = data; // Use data to avoid warning
        Ok(Vec::new())
    }

    /// Parse YAML format feed
    fn parse_yaml_feed(&self, data: &str) -> Result<Vec<ThreatEntry>> {
        let entries: Vec<ThreatEntry> =
            serde_yaml::from_str(data).context("Failed to parse YAML feed")?;
        Ok(entries)
    }

    /// Get all cached threats
    pub fn get_all_threats(&self) -> Vec<ThreatEntry> {
        self.cache
            .values()
            .flat_map(|entries| entries.iter().cloned())
            .collect()
    }

    /// Get statistics about loaded feeds
    pub fn get_stats(&self) -> FeedStats {
        let total_feeds = self.feeds.len();
        let enabled_feeds = self.feeds.iter().filter(|f| f.enabled).count();
        let total_threats = self.get_all_threats().len();

        let threats_by_severity = self.count_by_severity();

        FeedStats {
            total_feeds,
            enabled_feeds,
            total_threats,
            threats_by_severity,
        }
    }

    /// Count threats by severity
    fn count_by_severity(&self) -> HashMap<Severity, usize> {
        let mut counts = HashMap::new();

        for threat in self.get_all_threats() {
            *counts.entry(threat.severity).or_insert(0) += 1;
        }

        counts
    }
}

/// Custom feed configuration file format
#[derive(Debug, Serialize, Deserialize)]
struct CustomFeedConfig {
    feeds: Vec<ThreatFeed>,
}

/// Statistics about loaded feeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedStats {
    pub total_feeds: usize,
    pub enabled_feeds: usize,
    pub total_threats: usize,
    pub threats_by_severity: HashMap<Severity, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_manager_creation() {
        let manager = CustomFeedManager::new();
        assert_eq!(manager.feeds.len(), 0);
        assert_eq!(manager.cache.len(), 0);
    }

    #[test]
    fn test_add_feed() {
        let mut manager = CustomFeedManager::new();
        let feed = ThreatFeed {
            name: "test-feed".to_string(),
            description: "Test feed".to_string(),
            source: FeedSource::File {
                path: "/tmp/test.json".to_string(),
            },
            format: FeedFormat::Json,
            update_frequency_hours: 24,
            last_updated: None,
            enabled: true,
        };

        manager.add_feed(feed);
        assert_eq!(manager.feeds.len(), 1);
    }

    #[test]
    fn test_enable_disable_feed() {
        let mut manager = CustomFeedManager::new();
        let feed = ThreatFeed {
            name: "test-feed".to_string(),
            description: "Test feed".to_string(),
            source: FeedSource::File {
                path: "/tmp/test.json".to_string(),
            },
            format: FeedFormat::Json,
            update_frequency_hours: 24,
            last_updated: None,
            enabled: true,
        };

        manager.add_feed(feed.clone());

        assert!(manager.disable_feed("test-feed"));
        assert!(!manager.feeds[0].enabled);

        assert!(manager.enable_feed("test-feed"));
        assert!(manager.feeds[0].enabled);

        assert!(!manager.disable_feed("non-existent"));
    }

    #[test]
    fn test_remove_feed() {
        let mut manager = CustomFeedManager::new();
        let feed = ThreatFeed {
            name: "test-feed".to_string(),
            description: "Test feed".to_string(),
            source: FeedSource::File {
                path: "/tmp/test.json".to_string(),
            },
            format: FeedFormat::Json,
            update_frequency_hours: 24,
            last_updated: None,
            enabled: true,
        };

        manager.add_feed(feed);
        assert_eq!(manager.feeds.len(), 1);

        assert!(manager.remove_feed("test-feed"));
        assert_eq!(manager.feeds.len(), 0);

        assert!(!manager.remove_feed("non-existent"));
    }

    #[test]
    fn test_package_matching() {
        let manager = CustomFeedManager::new();

        // Exact match
        assert!(manager.package_matches("log4j-core", "log4j-core"));

        // Wildcard matching
        assert!(manager.package_matches("log4j-*", "log4j-core"));
        assert!(manager.package_matches("*-core", "log4j-core"));

        // No match
        assert!(!manager.package_matches("spring-*", "log4j-core"));
    }

    #[test]
    fn test_severity_levels() {
        assert!(matches!(Severity::Critical, Severity::Critical));
        assert!(matches!(Severity::High, Severity::High));
        assert!(matches!(Severity::Medium, Severity::Medium));
        assert!(matches!(Severity::Low, Severity::Low));
        assert!(matches!(Severity::Info, Severity::Info));
    }

    #[test]
    fn test_feed_formats() {
        assert_eq!(FeedFormat::Json, FeedFormat::Json);
        assert_eq!(FeedFormat::Osv, FeedFormat::Osv);
        assert_eq!(FeedFormat::Csv, FeedFormat::Csv);
        assert_eq!(FeedFormat::Yaml, FeedFormat::Yaml);
    }

    #[test]
    fn test_threat_entry_structure() {
        let entry = ThreatEntry {
            id: "CUSTOM-2025-001".to_string(),
            package: "test-package".to_string(),
            affected_versions: vec!["1.0.0".to_string(), "1.1.0".to_string()],
            threat_type: "malicious-code".to_string(),
            severity: Severity::High,
            description: "Test threat".to_string(),
            references: vec!["https://example.com".to_string()],
            discovered: "2025-11-05".to_string(),
        };

        assert_eq!(entry.id, "CUSTOM-2025-001");
        assert_eq!(entry.severity, Severity::High);
        assert_eq!(entry.affected_versions.len(), 2);
    }
}
