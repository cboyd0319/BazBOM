//! Scan result caching for faster incremental analysis
//!
//! Provides caching of scan results to speed up repeated scans and enable
//! incremental analysis for large projects.

use anyhow::{Context, Result};
use bazbom_cache::CacheManager;
use std::path::{Path, PathBuf};

/// Scan cache wrapper
pub struct ScanCache {
    cache_manager: CacheManager,
}

impl ScanCache {
    /// Create a new scan cache
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Default to 1GB max cache size
        let max_size_bytes = 1024 * 1024 * 1024; // 1GB
        let cache_manager = CacheManager::new(cache_dir, max_size_bytes)?;
        Ok(Self { cache_manager })
    }

    /// Generate cache key for a scan
    ///
    /// Cache key is based on:
    /// - Project path
    /// - Build files content hash (pom.xml, build.gradle, etc.)
    /// - Scan parameters (reachability, fast mode, etc.)
    pub fn generate_cache_key(
        project_path: &Path,
        build_files: &[PathBuf],
        scan_params: &ScanParameters,
    ) -> Result<String> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();

        // Hash project path
        hasher.update(project_path.to_string_lossy().as_bytes());

        // Hash build file contents
        for build_file in build_files {
            if build_file.exists() {
                let content = std::fs::read(build_file)
                    .with_context(|| format!("Failed to read {}", build_file.display()))?;
                hasher.update(&content);
            }
        }

        // Hash scan parameters
        hasher.update(format!("{:?}", scan_params).as_bytes());

        let hash = hasher.finalize();
        Ok(hex::encode(hash))
    }

    /// Get cached scan result
    pub fn get_scan_result(&mut self, cache_key: &str) -> Result<Option<ScanResult>> {
        if let Some(data) = self.cache_manager.get(cache_key)? {
            let result: ScanResult = serde_json::from_slice(&data)
                .context("Failed to deserialize cached scan result")?;
            Ok(Some(result))
        } else {
            Ok(None)
        }
    }

    /// Store scan result in cache
    pub fn put_scan_result(&mut self, cache_key: &str, result: &ScanResult) -> Result<()> {
        let data = serde_json::to_vec(result).context("Failed to serialize scan result")?;

        // Cache for 1 hour by default
        let ttl = chrono::Duration::hours(1);
        self.cache_manager.put(cache_key, &data, Some(ttl))?;

        Ok(())
    }

    /// Clear all cached scan results
    pub fn clear(&mut self) -> Result<()> {
        self.cache_manager.clear()
    }
}

/// Scan parameters for cache key generation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanParameters {
    pub reachability: bool,
    pub fast: bool,
    pub format: String,
    pub bazel_targets: Option<Vec<String>>,
}

/// Cached scan result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScanResult {
    /// SBOM content (JSON)
    pub sbom_json: String,
    /// Findings content (JSON)
    pub findings_json: Option<String>,
    /// Scan timestamp
    pub scanned_at: String,
    /// Scan parameters used
    pub parameters: ScanParameters,
}

impl ScanResult {
    /// Create a new scan result
    pub fn new(
        sbom_json: String,
        findings_json: Option<String>,
        parameters: ScanParameters,
    ) -> Self {
        Self {
            sbom_json,
            findings_json,
            scanned_at: chrono::Utc::now().to_rfc3339(),
            parameters,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_cache_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let _cache = ScanCache::new(temp_dir.path().to_path_buf())?;
        assert!(temp_dir.path().exists());
        Ok(())
    }

    #[test]
    fn test_cache_key_generation() -> Result<()> {
        let project_path = Path::new("/tmp/test-project");
        let build_files = vec![];
        let params = ScanParameters {
            reachability: false,
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let key = ScanCache::generate_cache_key(project_path, &build_files, &params)?;
        assert!(!key.is_empty());
        assert_eq!(key.len(), 64); // SHA-256 hex = 64 chars

        Ok(())
    }

    #[test]
    fn test_cache_key_consistency() -> Result<()> {
        let project_path = Path::new("/tmp/test-project");
        let build_files = vec![];
        let params = ScanParameters {
            reachability: false,
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let key1 = ScanCache::generate_cache_key(project_path, &build_files, &params)?;
        let key2 = ScanCache::generate_cache_key(project_path, &build_files, &params)?;

        assert_eq!(key1, key2);

        Ok(())
    }

    #[test]
    fn test_cache_key_differs_with_params() -> Result<()> {
        let project_path = Path::new("/tmp/test-project");
        let build_files = vec![];

        let params1 = ScanParameters {
            reachability: false,
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let params2 = ScanParameters {
            reachability: true, // Different
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let key1 = ScanCache::generate_cache_key(project_path, &build_files, &params1)?;
        let key2 = ScanCache::generate_cache_key(project_path, &build_files, &params2)?;

        assert_ne!(key1, key2);

        Ok(())
    }

    #[test]
    fn test_scan_result_creation() {
        let params = ScanParameters {
            reachability: false,
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let result = ScanResult::new("{}".to_string(), Some("[]".to_string()), params.clone());

        assert_eq!(result.sbom_json, "{}");
        assert_eq!(result.findings_json, Some("[]".to_string()));
        assert_eq!(result.parameters.fast, true);
    }

    #[test]
    fn test_cache_put_and_get() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache = ScanCache::new(temp_dir.path().to_path_buf())?;

        let params = ScanParameters {
            reachability: false,
            fast: true,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        let result = ScanResult::new("{}".to_string(), Some("[]".to_string()), params.clone());

        // Put in cache
        cache.put_scan_result("test-key", &result)?;

        // Get from cache
        let cached = cache.get_scan_result("test-key")?;
        assert!(cached.is_some());

        let cached = cached.unwrap();
        assert_eq!(cached.sbom_json, "{}");
        assert_eq!(cached.findings_json, Some("[]".to_string()));

        Ok(())
    }

    #[test]
    fn test_cache_miss() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let mut cache = ScanCache::new(temp_dir.path().to_path_buf())?;

        let result = cache.get_scan_result("nonexistent-key")?;
        assert!(result.is_none());

        Ok(())
    }
}
