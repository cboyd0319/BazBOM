//! Intelligent caching for BazBOM
//!
//! Provides caching mechanisms to speed up repeated scans and enable incremental analysis

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Cache entry metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Unique key for this cache entry
    pub key: String,
    /// Content hash (SHA-256)
    pub content_hash: String,
    /// When this entry was created
    pub created_at: DateTime<Utc>,
    /// When this entry was last accessed
    pub last_accessed: DateTime<Utc>,
    /// When this entry expires (None = never)
    pub expires_at: Option<DateTime<Utc>>,
    /// Size in bytes
    pub size_bytes: usize,
    /// File path to cached data
    pub file_path: PathBuf,
}

impl CacheEntry {
    /// Check if this entry is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if this entry is valid for the given content hash
    pub fn is_valid_for(&self, content_hash: &str) -> bool {
        !self.is_expired() && self.content_hash == content_hash
    }
}

/// Cache manager for scan results
pub struct CacheManager {
    /// Cache directory
    cache_dir: PathBuf,
    /// Cache index (key -> entry)
    index: HashMap<String, CacheEntry>,
    /// Index file path
    index_path: PathBuf,
    /// Maximum cache size in bytes
    max_size_bytes: usize,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(cache_dir: PathBuf, max_size_bytes: usize) -> Result<Self> {
        // Create cache directory if it doesn't exist
        std::fs::create_dir_all(&cache_dir)
            .with_context(|| format!("Failed to create cache directory: {}", cache_dir.display()))?;

        let index_path = cache_dir.join("index.json");
        
        // Load existing index or create new one
        let index = if index_path.exists() {
            Self::load_index(&index_path)?
        } else {
            HashMap::new()
        };

        Ok(Self {
            cache_dir,
            index,
            index_path,
            max_size_bytes,
        })
    }

    /// Load cache index from file
    fn load_index(path: &Path) -> Result<HashMap<String, CacheEntry>> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read index from {}", path.display()))?;
        
        let index: HashMap<String, CacheEntry> = serde_json::from_str(&content)
            .context("Failed to parse index JSON")?;
        
        Ok(index)
    }

    /// Save cache index to file
    fn save_index(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.index)
            .context("Failed to serialize index")?;
        
        std::fs::write(&self.index_path, content)
            .with_context(|| format!("Failed to write index to {}", self.index_path.display()))?;
        
        Ok(())
    }

    /// Calculate content hash (SHA-256)
    pub fn calculate_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        hex::encode(hasher.finalize())
    }

    /// Get cached data by key
    pub fn get(&mut self, key: &str) -> Result<Option<Vec<u8>>> {
        // Check if entry exists and is not expired
        let is_expired = self.index.get(key).map(|e| e.is_expired()).unwrap_or(false);
        
        if is_expired {
            self.remove(key)?;
            return Ok(None);
        }

        // Get entry (now we know it exists and is not expired)
        if let Some(entry) = self.index.get(key) {
            let file_path = entry.file_path.clone();
            
            // Update last accessed time
            if let Some(entry) = self.index.get_mut(key) {
                entry.last_accessed = Utc::now();
            }
            self.save_index()?;

            // Read cached data
            let data = std::fs::read(&file_path)
                .with_context(|| format!("Failed to read cached data from {}", file_path.display()))?;
            
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Put data in cache
    pub fn put(&mut self, key: &str, data: &[u8], ttl: Option<chrono::Duration>) -> Result<()> {
        let content_hash = Self::calculate_hash(data);
        let file_name = format!("{}.bin", content_hash);
        let file_path = self.cache_dir.join(&file_name);

        // Write data to file
        std::fs::write(&file_path, data)
            .with_context(|| format!("Failed to write cache data to {}", file_path.display()))?;

        let expires_at = ttl.map(|d| Utc::now() + d);

        let entry = CacheEntry {
            key: key.to_string(),
            content_hash,
            created_at: Utc::now(),
            last_accessed: Utc::now(),
            expires_at,
            size_bytes: data.len(),
            file_path,
        };

        // Remove old entry if exists
        if let Some(old_entry) = self.index.get(key) {
            let _ = std::fs::remove_file(&old_entry.file_path);
        }

        self.index.insert(key.to_string(), entry);
        
        // Check cache size and evict if necessary
        self.evict_if_needed()?;
        
        self.save_index()?;
        
        Ok(())
    }

    /// Remove entry from cache
    pub fn remove(&mut self, key: &str) -> Result<()> {
        if let Some(entry) = self.index.remove(key) {
            let _ = std::fs::remove_file(&entry.file_path);
            self.save_index()?;
        }
        Ok(())
    }

    /// Check if key exists and is not expired
    pub fn contains(&self, key: &str) -> bool {
        self.index
            .get(key)
            .map(|e| !e.is_expired())
            .unwrap_or(false)
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let total_entries = self.index.len();
        let total_size: usize = self.index.values().map(|e| e.size_bytes).sum();
        let expired_entries = self.index.values().filter(|e| e.is_expired()).count();

        CacheStats {
            total_entries,
            total_size_bytes: total_size,
            max_size_bytes: self.max_size_bytes,
            expired_entries,
        }
    }

    /// Evict entries if cache is over size limit
    fn evict_if_needed(&mut self) -> Result<()> {
        let mut total_size: usize = self.index.values().map(|e| e.size_bytes).sum();
        
        if total_size <= self.max_size_bytes {
            return Ok(());
        }

        // Collect entries with keys for sorting
        let mut entries: Vec<_> = self.index.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        entries.sort_by_key(|(_, e)| e.last_accessed);

        // Evict oldest entries until we're under the limit
        for (key, entry) in entries {
            if total_size <= self.max_size_bytes {
                break;
            }

            total_size -= entry.size_bytes;
            let _ = std::fs::remove_file(&entry.file_path);
            self.index.remove(&key);
        }

        self.save_index()?;
        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&mut self) -> Result<()> {
        for entry in self.index.values() {
            let _ = std::fs::remove_file(&entry.file_path);
        }
        self.index.clear();
        self.save_index()?;
        Ok(())
    }

    /// Prune expired entries
    pub fn prune_expired(&mut self) -> Result<usize> {
        let expired_keys: Vec<String> = self
            .index
            .iter()
            .filter(|(_, e)| e.is_expired())
            .map(|(k, _)| k.clone())
            .collect();

        let count = expired_keys.len();

        for key in expired_keys {
            self.remove(&key)?;
        }

        Ok(count)
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub total_size_bytes: usize,
    pub max_size_bytes: usize,
    pub expired_entries: usize,
}

impl CacheStats {
    /// Get cache usage percentage
    pub fn usage_percent(&self) -> f64 {
        (self.total_size_bytes as f64 / self.max_size_bytes as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_cache() -> (CacheManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();
        let max_size = 1024 * 1024; // 1 MB
        let cache = CacheManager::new(cache_dir, max_size).unwrap();
        (cache, temp_dir)
    }

    #[test]
    fn test_cache_creation() {
        let (cache, _temp) = create_test_cache();
        assert_eq!(cache.index.len(), 0);
    }

    #[test]
    fn test_put_and_get() {
        let (mut cache, _temp) = create_test_cache();
        let data = b"test data";
        
        cache.put("test-key", data, None).unwrap();
        let retrieved = cache.get("test-key").unwrap().unwrap();
        
        assert_eq!(retrieved, data);
    }

    #[test]
    fn test_contains() {
        let (mut cache, _temp) = create_test_cache();
        
        assert!(!cache.contains("test-key"));
        
        cache.put("test-key", b"data", None).unwrap();
        assert!(cache.contains("test-key"));
    }

    #[test]
    fn test_remove() {
        let (mut cache, _temp) = create_test_cache();
        
        cache.put("test-key", b"data", None).unwrap();
        assert!(cache.contains("test-key"));
        
        cache.remove("test-key").unwrap();
        assert!(!cache.contains("test-key"));
    }

    #[test]
    fn test_cache_stats() {
        let (mut cache, _temp) = create_test_cache();
        
        cache.put("key1", b"data1", None).unwrap();
        cache.put("key2", b"data2", None).unwrap();
        
        let stats = cache.stats();
        assert_eq!(stats.total_entries, 2);
        assert!(stats.total_size_bytes > 0);
    }

    #[test]
    fn test_expiration() {
        let (mut cache, _temp) = create_test_cache();
        
        // Put with very short TTL
        let ttl = chrono::Duration::milliseconds(-1); // Already expired
        cache.put("test-key", b"data", Some(ttl)).unwrap();
        
        // Should be expired
        let result = cache.get("test-key").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear() {
        let (mut cache, _temp) = create_test_cache();
        
        cache.put("key1", b"data1", None).unwrap();
        cache.put("key2", b"data2", None).unwrap();
        
        cache.clear().unwrap();
        assert_eq!(cache.index.len(), 0);
    }

    #[test]
    fn test_calculate_hash() {
        let data = b"test data";
        let hash = CacheManager::calculate_hash(data);
        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
    }

    #[test]
    fn test_prune_expired() {
        let (mut cache, _temp) = create_test_cache();
        
        // Add expired entry
        let ttl = chrono::Duration::milliseconds(-1);
        cache.put("expired", b"data", Some(ttl)).unwrap();
        
        // Add non-expired entry
        cache.put("valid", b"data", None).unwrap();
        
        let pruned = cache.prune_expired().unwrap();
        assert_eq!(pruned, 1);
        assert_eq!(cache.index.len(), 1);
    }
}
