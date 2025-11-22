//! License caching for improved performance
//!
//! This module provides a thread-safe cache for license information to avoid
//! redundant file system reads when scanning multiple ecosystems or large projects.

use std::collections::HashMap;
use std::sync::RwLock;

use crate::scanner::License;

/// Thread-safe cache for license information
#[derive(Debug)]
pub struct LicenseCache {
    cache: RwLock<HashMap<String, License>>,
}

impl LicenseCache {
    /// Create a new empty license cache
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get a license from the cache
    pub fn get(&self, key: &str) -> Option<License> {
        self.cache.read().unwrap().get(key).cloned()
    }

    /// Insert a license into the cache
    pub fn insert(&self, key: String, license: License) {
        self.cache.write().unwrap().insert(key, license);
    }

    /// Get a license from cache, or compute and insert it
    pub fn get_or_insert_with<F>(&self, key: String, f: F) -> License
    where
        F: FnOnce() -> License,
    {
        // Fast path: check if it's already cached
        if let Some(license) = self.get(&key) {
            return license;
        }

        // Slow path: compute and cache
        let license = f();
        self.insert(key, license.clone());
        license
    }

    /// Get the current cache size
    pub fn len(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().unwrap().is_empty()
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.write().unwrap().clear();
    }
}

impl Default for LicenseCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic() {
        let cache = LicenseCache::new();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());

        cache.insert(
            "test:package:1.0.0".to_string(),
            License::Spdx("MIT".to_string()),
        );
        assert_eq!(cache.len(), 1);
        assert!(!cache.is_empty());

        let license = cache.get("test:package:1.0.0").unwrap();
        assert_eq!(license, License::Spdx("MIT".to_string()));
    }

    #[test]
    fn test_cache_get_or_insert() {
        let cache = LicenseCache::new();
        let mut call_count = 0;

        let license1 = cache.get_or_insert_with("test:pkg:1.0.0".to_string(), || {
            call_count += 1;
            License::Spdx("Apache-2.0".to_string())
        });
        assert_eq!(call_count, 1);
        assert_eq!(license1, License::Spdx("Apache-2.0".to_string()));

        // Second call should use cached value
        let license2 = cache.get_or_insert_with("test:pkg:1.0.0".to_string(), || {
            call_count += 1;
            License::Unknown
        });
        assert_eq!(call_count, 1); // Should NOT increment
        assert_eq!(license2, License::Spdx("Apache-2.0".to_string()));
    }
}
