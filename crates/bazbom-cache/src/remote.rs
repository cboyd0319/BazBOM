//! Remote caching support for distributed CI/CD environments
//!
//! Enables sharing cache across machines using various backends:
//! - HTTP/HTTPS (REST API)
//! - S3-compatible storage
//! - Redis
//! - File system (NFS/shared drives)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Remote cache backend configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum RemoteCacheConfig {
    /// HTTP/HTTPS REST API backend
    Http {
        /// Base URL for the cache API
        base_url: String,
        /// Optional authentication token
        auth_token: Option<String>,
        /// Timeout in seconds
        #[serde(default = "default_timeout")]
        timeout_secs: u64,
    },
    /// S3-compatible storage (AWS S3, MinIO, etc.)
    S3 {
        /// S3 bucket name
        bucket: String,
        /// S3 region
        region: String,
        /// Optional endpoint URL (for MinIO, etc.)
        endpoint: Option<String>,
        /// Access key ID
        access_key_id: String,
        /// Secret access key
        secret_access_key: String,
        /// Path prefix within bucket
        #[serde(default)]
        prefix: String,
    },
    /// Redis key-value store
    Redis {
        /// Redis connection URL (redis://host:port/db)
        url: String,
        /// Optional password
        password: Option<String>,
        /// Key prefix for cache entries
        #[serde(default = "default_redis_prefix")]
        prefix: String,
    },
    /// Shared filesystem (NFS, SMB, etc.)
    FileSystem {
        /// Path to shared directory
        path: PathBuf,
    },
}

fn default_timeout() -> u64 {
    30
}

fn default_redis_prefix() -> String {
    "bazbom:cache:".to_string()
}

/// Remote cache backend trait
pub trait RemoteCacheBackend: Send + Sync {
    /// Check if a key exists in the remote cache
    fn exists(&self, key: &str) -> Result<bool>;

    /// Get cached data from remote
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Put data into remote cache
    fn put(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Remove entry from remote cache
    fn remove(&self, key: &str) -> Result<()>;

    /// Get cache statistics (if supported)
    fn stats(&self) -> Result<Option<RemoteCacheStats>> {
        Ok(None)
    }
}

/// Statistics about remote cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteCacheStats {
    /// Total number of entries
    pub total_entries: Option<usize>,
    /// Total size in bytes
    pub total_size_bytes: Option<usize>,
    /// Cache hit rate (0.0-1.0)
    pub hit_rate: Option<f64>,
}

/// HTTP-based remote cache implementation
pub struct HttpRemoteCache {
    base_url: String,
    auth_token: Option<String>,
    client: reqwest::blocking::Client,
}

impl HttpRemoteCache {
    /// Create a new HTTP remote cache
    pub fn new(base_url: String, auth_token: Option<String>, timeout_secs: u64) -> Result<Self> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url,
            auth_token,
            client,
        })
    }

    /// Build request with authentication
    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::blocking::RequestBuilder {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path.trim_start_matches('/'));
        let mut req = self.client.request(method, &url);

        if let Some(ref token) = self.auth_token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        req
    }
}

impl RemoteCacheBackend for HttpRemoteCache {
    fn exists(&self, key: &str) -> Result<bool> {
        let response = self
            .build_request(reqwest::Method::HEAD, &format!("/cache/{}", key))
            .send()
            .context("Failed to check if key exists")?;

        Ok(response.status().is_success())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let response = self
            .build_request(reqwest::Method::GET, &format!("/cache/{}", key))
            .send()
            .context("Failed to get cache entry")?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            anyhow::bail!("HTTP request failed with status: {}", response.status());
        }

        let data = response
            .bytes()
            .context("Failed to read response body")?
            .to_vec();

        Ok(Some(data))
    }

    fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        let response = self
            .build_request(reqwest::Method::PUT, &format!("/cache/{}", key))
            .body(data.to_vec())
            .send()
            .context("Failed to put cache entry")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP request failed with status: {}", response.status());
        }

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<()> {
        let response = self
            .build_request(reqwest::Method::DELETE, &format!("/cache/{}", key))
            .send()
            .context("Failed to remove cache entry")?;

        if !response.status().is_success() && response.status() != reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!("HTTP request failed with status: {}", response.status());
        }

        Ok(())
    }

    fn stats(&self) -> Result<Option<RemoteCacheStats>> {
        let response = self
            .build_request(reqwest::Method::GET, "/cache/stats")
            .send()
            .context("Failed to get cache stats")?;

        if !response.status().is_success() {
            return Ok(None);
        }

        let stats: RemoteCacheStats = response
            .json()
            .context("Failed to parse cache stats")?;

        Ok(Some(stats))
    }
}

/// Filesystem-based remote cache (for NFS, SMB, etc.)
pub struct FileSystemRemoteCache {
    cache_dir: PathBuf,
}

impl FileSystemRemoteCache {
    /// Create a new filesystem remote cache
    pub fn new(cache_dir: PathBuf) -> Result<Self> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

        Ok(Self { cache_dir })
    }

    /// Get file path for a cache key
    fn get_file_path(&self, key: &str) -> PathBuf {
        // Use first 2 chars as subdirectory for better filesystem performance
        let subdir = if key.len() >= 2 { &key[0..2] } else { "00" };
        self.cache_dir.join(subdir).join(format!("{}.bin", key))
    }
}

impl RemoteCacheBackend for FileSystemRemoteCache {
    fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.get_file_path(key).exists())
    }

    fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let file_path = self.get_file_path(key);

        if !file_path.exists() {
            return Ok(None);
        }

        let data = std::fs::read(&file_path)
            .with_context(|| format!("Failed to read cache file: {}", file_path.display()))?;

        Ok(Some(data))
    }

    fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        let file_path = self.get_file_path(key);

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        std::fs::write(&file_path, data)
            .with_context(|| format!("Failed to write cache file: {}", file_path.display()))?;

        Ok(())
    }

    fn remove(&self, key: &str) -> Result<()> {
        let file_path = self.get_file_path(key);

        if file_path.exists() {
            std::fs::remove_file(&file_path)
                .with_context(|| format!("Failed to remove cache file: {}", file_path.display()))?;
        }

        Ok(())
    }

    fn stats(&self) -> Result<Option<RemoteCacheStats>> {
        // Calculate stats by walking the directory
        let mut total_entries = 0;
        let mut total_size_bytes = 0;

        if let Ok(entries) = std::fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    // Check subdirectories
                    if let Ok(subentries) = std::fs::read_dir(entry.path()) {
                        for subentry in subentries.flatten() {
                            if let Ok(metadata) = subentry.metadata() {
                                total_entries += 1;
                                total_size_bytes += metadata.len() as usize;
                            }
                        }
                    }
                }
            }
        }

        Ok(Some(RemoteCacheStats {
            total_entries: Some(total_entries),
            total_size_bytes: Some(total_size_bytes),
            hit_rate: None,
        }))
    }
}

/// Two-tier cache manager with local and remote backends
pub struct TwoTierCacheManager {
    local: crate::CacheManager,
    remote: Option<Box<dyn RemoteCacheBackend>>,
}

impl TwoTierCacheManager {
    /// Create a new two-tier cache manager
    pub fn new(
        local: crate::CacheManager,
        remote: Option<Box<dyn RemoteCacheBackend>>,
    ) -> Self {
        Self { local, remote }
    }

    /// Get data from cache (local first, then remote)
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        // Try local cache first
        if let Some(data) = self.local.get(key)? {
            return Ok(Some(data));
        }

        // Try remote cache if available
        if let Some(ref remote) = self.remote {
            if let Some(data) = remote.get(key)? {
                // Store in local cache for next time
                self.local.put(key, &data, None)?;
                return Ok(Some(data));
            }
        }

        Ok(None)
    }

    /// Put data in cache (both local and remote)
    pub fn put(&mut self, key: &str, data: &[u8]) -> Result<()> {
        // Store in local cache
        self.local.put(key, data, None)?;

        // Store in remote cache if available
        if let Some(ref remote) = self.remote {
            // Don't fail if remote storage fails - just log
            if let Err(e) = remote.put(key, data) {
                eprintln!("Warning: Failed to store in remote cache: {}", e);
            }
        }

        Ok(())
    }

    /// Remove from both caches
    pub fn remove(&mut self, key: &str) -> Result<()> {
        self.local.remove(key)?;

        if let Some(ref remote) = self.remote {
            // Don't fail if remote removal fails
            let _ = remote.remove(key);
        }

        Ok(())
    }

    /// Check if key exists in either cache
    pub fn contains(&self, key: &str) -> bool {
        if self.local.contains(key) {
            return true;
        }

        if let Some(ref remote) = self.remote {
            return remote.exists(key).unwrap_or(false);
        }

        false
    }
}

/// Create a remote cache backend from configuration
pub fn create_remote_cache(config: &RemoteCacheConfig) -> Result<Box<dyn RemoteCacheBackend>> {
    match config {
        RemoteCacheConfig::Http {
            base_url,
            auth_token,
            timeout_secs,
        } => {
            let cache = HttpRemoteCache::new(
                base_url.clone(),
                auth_token.clone(),
                *timeout_secs,
            )?;
            Ok(Box::new(cache))
        }
        RemoteCacheConfig::FileSystem { path } => {
            let cache = FileSystemRemoteCache::new(path.clone())?;
            Ok(Box::new(cache))
        }
        RemoteCacheConfig::S3 { .. } => {
            // S3 implementation would go here
            anyhow::bail!("S3 remote cache not yet implemented")
        }
        RemoteCacheConfig::Redis { .. } => {
            // Redis implementation would go here
            anyhow::bail!("Redis remote cache not yet implemented")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_filesystem_remote_cache() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileSystemRemoteCache::new(temp_dir.path().to_path_buf()).unwrap();

        let key = "test-key";
        let data = b"test data";

        // Initially doesn't exist
        assert!(!cache.exists(key).unwrap());

        // Put data
        cache.put(key, data).unwrap();
        assert!(cache.exists(key).unwrap());

        // Get data
        let retrieved = cache.get(key).unwrap().unwrap();
        assert_eq!(retrieved, data);

        // Remove data
        cache.remove(key).unwrap();
        assert!(!cache.exists(key).unwrap());
    }

    #[test]
    fn test_filesystem_remote_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache = FileSystemRemoteCache::new(temp_dir.path().to_path_buf()).unwrap();

        cache.put("key1", b"data1").unwrap();
        cache.put("key2", b"data2").unwrap();

        let stats = cache.stats().unwrap().unwrap();
        assert_eq!(stats.total_entries, Some(2));
        assert!(stats.total_size_bytes.is_some());
    }
}
