/// Community upgrade success data tracking
///
/// Provides crowdsourced upgrade compatibility and success rate data
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeSuccessData {
    pub from_version: String,
    pub to_version: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub total_attempts: u32,
    pub success_rate: f32,
    pub common_issues: Vec<String>,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityDatabase {
    /// Map of package -> version pair -> success data
    data: HashMap<String, HashMap<String, UpgradeSuccessData>>,
    cache_path: PathBuf,
}

impl CommunityDatabase {
    /// Create a new community database
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("bazbom")
            .join("community");

        fs::create_dir_all(&cache_dir)?;
        let cache_path = cache_dir.join("upgrade_data.json");

        let data = if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            // Initialize with some sample data
            Self::initialize_sample_data()
        };

        Ok(Self { data, cache_path })
    }

    /// Create a new community database with a custom cache path (for testing)
    #[cfg(test)]
    pub fn with_path(cache_path: PathBuf) -> Result<Self> {
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Start with empty data for test isolation
        let data = HashMap::new();

        Ok(Self { data, cache_path })
    }

    /// Initialize with sample community data
    fn initialize_sample_data() -> HashMap<String, HashMap<String, UpgradeSuccessData>> {
        let mut data = HashMap::new();

        // Spring Boot upgrade data
        let mut spring_boot = HashMap::new();
        spring_boot.insert(
            "2.7->3.0".to_string(),
            UpgradeSuccessData {
                from_version: "2.7".to_string(),
                to_version: "3.0".to_string(),
                success_count: 1250,
                failure_count: 150,
                total_attempts: 1400,
                success_rate: 0.89,
                common_issues: vec![
                    "Jakarta EE namespace migration required".to_string(),
                    "Configuration property renames".to_string(),
                    "Deprecated APIs removed".to_string(),
                ],
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        );
        data.insert(
            "org.springframework.boot:spring-boot".to_string(),
            spring_boot,
        );

        // Jackson upgrade data
        let mut jackson = HashMap::new();
        jackson.insert(
            "2.13->2.14".to_string(),
            UpgradeSuccessData {
                from_version: "2.13".to_string(),
                to_version: "2.14".to_string(),
                success_count: 2800,
                failure_count: 50,
                total_attempts: 2850,
                success_rate: 0.98,
                common_issues: vec!["Minor serialization behavior changes".to_string()],
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        );
        data.insert(
            "com.fasterxml.jackson.core:jackson-databind".to_string(),
            jackson,
        );

        // Log4j upgrade data
        let mut log4j = HashMap::new();
        log4j.insert(
            "2.17->2.18".to_string(),
            UpgradeSuccessData {
                from_version: "2.17".to_string(),
                to_version: "2.18".to_string(),
                success_count: 3200,
                failure_count: 20,
                total_attempts: 3220,
                success_rate: 0.99,
                common_issues: vec![],
                last_updated: chrono::Utc::now().to_rfc3339(),
            },
        );
        data.insert("org.apache.logging.log4j:log4j-core".to_string(), log4j);

        data
    }

    /// Get upgrade success data for a specific package and version range
    pub fn get_success_data(
        &self,
        package: &str,
        from_version: &str,
        to_version: &str,
    ) -> Option<UpgradeSuccessData> {
        let version_key = format!("{}->{}", from_version, to_version);

        self.data
            .get(package)
            .and_then(|versions| versions.get(&version_key))
            .cloned()
    }

    /// Get success rate for an upgrade
    pub fn get_success_rate(
        &self,
        package: &str,
        from_version: &str,
        to_version: &str,
    ) -> Option<f32> {
        self.get_success_data(package, from_version, to_version)
            .map(|data| data.success_rate)
    }

    /// Submit upgrade feedback (for future implementation)
    pub fn submit_feedback(
        &mut self,
        package: &str,
        from_version: &str,
        to_version: &str,
        success: bool,
        issues: Vec<String>,
    ) -> Result<()> {
        let version_key = format!("{}->{}", from_version, to_version);

        let package_data = self.data.entry(package.to_string()).or_default();

        let success_data = package_data
            .entry(version_key)
            .or_insert_with(|| UpgradeSuccessData {
                from_version: from_version.to_string(),
                to_version: to_version.to_string(),
                success_count: 0,
                failure_count: 0,
                total_attempts: 0,
                success_rate: 0.0,
                common_issues: Vec::new(),
                last_updated: chrono::Utc::now().to_rfc3339(),
            });

        // Update counts
        success_data.total_attempts += 1;
        if success {
            success_data.success_count += 1;
        } else {
            success_data.failure_count += 1;
        }

        // Recalculate success rate
        success_data.success_rate =
            success_data.success_count as f32 / success_data.total_attempts as f32;

        // Add new issues
        for issue in issues {
            if !success_data.common_issues.contains(&issue) {
                success_data.common_issues.push(issue);
            }
        }

        success_data.last_updated = chrono::Utc::now().to_rfc3339();

        // Save to disk
        self.save()?;

        Ok(())
    }

    /// Save community data to disk
    fn save(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.cache_path, json)?;
        Ok(())
    }

    /// Sync with remote community database (future implementation)
    pub async fn sync_with_remote(&mut self, _api_url: &str) -> Result<()> {
        // Future: Sync with remote API
        // For now, just return success
        tracing::info!("Community data sync not yet implemented");
        Ok(())
    }

    /// Get all packages with community data
    pub fn list_packages(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Get all tracked upgrades for a package
    pub fn list_upgrades(&self, package: &str) -> Vec<String> {
        self.data
            .get(package)
            .map(|versions| versions.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Get statistics summary
    pub fn get_stats(&self) -> CommunityStats {
        let total_packages = self.data.len();
        let mut total_upgrades = 0;
        let mut total_attempts = 0;
        let mut total_successes = 0;

        for package_data in self.data.values() {
            total_upgrades += package_data.len();
            for upgrade in package_data.values() {
                total_attempts += upgrade.total_attempts;
                total_successes += upgrade.success_count;
            }
        }

        let overall_success_rate = if total_attempts > 0 {
            total_successes as f32 / total_attempts as f32
        } else {
            0.0
        };

        CommunityStats {
            total_packages,
            total_upgrades,
            total_attempts,
            overall_success_rate,
        }
    }
}

impl Default for CommunityDatabase {
    fn default() -> Self {
        Self::new().expect("Failed to create community database")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityStats {
    pub total_packages: usize,
    pub total_upgrades: usize,
    pub total_attempts: u32,
    pub overall_success_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_community_database_creation() {
        let db = CommunityDatabase::new().unwrap();
        assert!(!db.list_packages().is_empty());
    }

    #[test]
    fn test_get_success_rate() {
        let db = CommunityDatabase::new().unwrap();
        let rate = db.get_success_rate("org.springframework.boot:spring-boot", "2.7", "3.0");
        assert!(rate.is_some());
        assert!(rate.unwrap() > 0.0);
    }

    #[test]
    fn test_submit_feedback() {
        // Use temporary directory for test isolation
        let temp_dir = tempfile::tempdir().unwrap();
        let cache_path = temp_dir.path().join("test_upgrade_data.json");
        let mut db = CommunityDatabase::with_path(cache_path).unwrap();

        let result = db.submit_feedback(
            "test:package",
            "1.0",
            "2.0",
            true,
            vec!["No issues".to_string()],
        );
        assert!(result.is_ok());

        let data = db.get_success_data("test:package", "1.0", "2.0");
        assert!(data.is_some());
        assert_eq!(data.unwrap().success_count, 1);
    }

    #[test]
    fn test_stats() {
        let db = CommunityDatabase::new().unwrap();
        let stats = db.get_stats();
        assert!(stats.total_packages > 0);
        assert!(stats.overall_success_rate > 0.0);
    }
}
