use crate::reachability::ReachabilityResult;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheMetadata {
    pub classpath_hash: String,
    pub entrypoints_hash: String,
    pub timestamp: String,
}

#[allow(dead_code)]
/// Generate a cache key from classpath and entrypoints
pub fn generate_cache_key(classpath: &str, entrypoints: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(classpath.as_bytes());
    hasher.update(b"|");
    hasher.update(entrypoints.as_bytes());
    hasher.finalize().to_hex().to_string()
}

#[allow(dead_code)]
/// Get the cache directory for reachability results
pub fn get_cache_dir() -> PathBuf {
    PathBuf::from(".bazbom/cache/reachability")
}

#[allow(dead_code)]
/// Load cached reachability result if it exists and is valid
pub fn load_cached_result(
    cache_dir: &Path,
    classpath: &str,
    entrypoints: &str,
) -> Result<Option<ReachabilityResult>> {
    let cache_key = generate_cache_key(classpath, entrypoints);
    let result_path = cache_dir.join(format!("{}.json", cache_key));
    let metadata_path = cache_dir.join(format!("{}.meta.json", cache_key));

    if !result_path.exists() || !metadata_path.exists() {
        return Ok(None);
    }

    // Verify metadata matches current inputs
    let metadata_content =
        fs::read_to_string(&metadata_path).context("failed to read cache metadata")?;
    let metadata: CacheMetadata =
        serde_json::from_str(&metadata_content).context("failed to parse cache metadata")?;

    let classpath_hash = blake3::hash(classpath.as_bytes()).to_hex().to_string();
    let entrypoints_hash = blake3::hash(entrypoints.as_bytes()).to_hex().to_string();

    if metadata.classpath_hash != classpath_hash || metadata.entrypoints_hash != entrypoints_hash {
        return Ok(None);
    }

    // Load result
    let result_content =
        fs::read_to_string(&result_path).context("failed to read cached result")?;
    let result: ReachabilityResult =
        serde_json::from_str(&result_content).context("failed to parse cached result")?;

    Ok(Some(result))
}

#[allow(dead_code)]
/// Save reachability result to cache
pub fn save_cached_result(
    cache_dir: &Path,
    classpath: &str,
    entrypoints: &str,
    result: &ReachabilityResult,
) -> Result<()> {
    fs::create_dir_all(cache_dir).context("failed to create cache directory")?;

    let cache_key = generate_cache_key(classpath, entrypoints);
    let result_path = cache_dir.join(format!("{}.json", cache_key));
    let metadata_path = cache_dir.join(format!("{}.meta.json", cache_key));

    // Save result
    let result_json = serde_json::to_string_pretty(result).context("failed to serialize result")?;
    fs::write(&result_path, result_json).context("failed to write result")?;

    // Save metadata
    let metadata = CacheMetadata {
        classpath_hash: blake3::hash(classpath.as_bytes()).to_hex().to_string(),
        entrypoints_hash: blake3::hash(entrypoints.as_bytes()).to_hex().to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };
    let metadata_json =
        serde_json::to_string_pretty(&metadata).context("failed to serialize metadata")?;
    fs::write(&metadata_path, metadata_json).context("failed to write metadata")?;

    Ok(())
}

/// Clear old cache entries (older than specified days)
#[allow(dead_code)]
pub fn clear_old_cache_entries(cache_dir: &Path, days: u64) -> Result<usize> {
    if !cache_dir.exists() {
        return Ok(0);
    }

    let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
    let mut cleared = 0;

    for entry in fs::read_dir(cache_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json")
            && path.to_string_lossy().contains(".meta.")
        {
            // This is a metadata file
            let content = fs::read_to_string(&path)?;
            if let Ok(metadata) = serde_json::from_str::<CacheMetadata>(&content) {
                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&metadata.timestamp) {
                    if timestamp.with_timezone(&chrono::Utc) < cutoff {
                        // Remove both metadata and result files
                        fs::remove_file(&path)?;
                        let result_path_str = path.to_string_lossy().replace(".meta.json", ".json");
                        let result_path = PathBuf::from(&result_path_str);
                        if result_path.exists() {
                            fs::remove_file(&result_path)?;
                        }
                        cleared += 1;
                    }
                }
            }
        }
    }

    Ok(cleared)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_cache_key() {
        let key1 = generate_cache_key("/path/to/app.jar", "com.example.Main");
        let key2 = generate_cache_key("/path/to/app.jar", "com.example.Main");
        let key3 = generate_cache_key("/path/to/other.jar", "com.example.Main");

        assert_eq!(key1, key2, "Same inputs should generate same key");
        assert_ne!(
            key1, key3,
            "Different inputs should generate different keys"
        );
        assert_eq!(key1.len(), 64, "Blake3 hash should be 64 hex characters");
    }

    #[test]
    fn test_save_and_load_cache() {
        let tmp = tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");

        let result = ReachabilityResult {
            tool: "bazbom-reachability".to_string(),
            version: "0.1.0".to_string(),
            classpath: "/path/to/app.jar".to_string(),
            entrypoints: "com.example.Main".to_string(),
            detected_entrypoints: vec!["com.example.Main.main".to_string()],
            reachable_methods: vec!["com.example.Main.main".to_string()],
            reachable_classes: vec!["com.example.Main".to_string()],
            reachable_packages: vec!["com.example".to_string()],
            error: None,
        };

        // Save to cache
        save_cached_result(&cache_dir, "/path/to/app.jar", "com.example.Main", &result).unwrap();

        // Load from cache
        let loaded = load_cached_result(&cache_dir, "/path/to/app.jar", "com.example.Main")
            .unwrap()
            .expect("Should load cached result");

        assert_eq!(loaded.tool, result.tool);
        assert_eq!(loaded.reachable_methods, result.reachable_methods);
    }

    #[test]
    fn test_cache_miss_on_different_inputs() {
        let tmp = tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");

        let result = ReachabilityResult {
            tool: "bazbom-reachability".to_string(),
            version: "0.1.0".to_string(),
            classpath: "/path/to/app.jar".to_string(),
            entrypoints: "com.example.Main".to_string(),
            detected_entrypoints: vec![],
            reachable_methods: vec![],
            reachable_classes: vec![],
            reachable_packages: vec![],
            error: None,
        };

        save_cached_result(&cache_dir, "/path/to/app.jar", "com.example.Main", &result).unwrap();

        // Try to load with different classpath
        let loaded =
            load_cached_result(&cache_dir, "/path/to/other.jar", "com.example.Main").unwrap();

        assert!(
            loaded.is_none(),
            "Should not find cached result with different classpath"
        );
    }

    #[test]
    fn test_clear_old_cache_entries() {
        let tmp = tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");
        fs::create_dir_all(&cache_dir).unwrap();

        // Create a cache entry with old timestamp
        let old_metadata = CacheMetadata {
            classpath_hash: "test123".to_string(),
            entrypoints_hash: "test456".to_string(),
            timestamp: chrono::Utc::now()
                .checked_sub_signed(chrono::Duration::days(90))
                .unwrap()
                .to_rfc3339(),
        };

        let meta_path = cache_dir.join("old.meta.json");
        fs::write(&meta_path, serde_json::to_string(&old_metadata).unwrap()).unwrap();
        fs::write(cache_dir.join("old.json"), "{}").unwrap();

        // Clear entries older than 30 days
        let cleared = clear_old_cache_entries(&cache_dir, 30).unwrap();

        assert_eq!(cleared, 1, "Should clear one old entry");
        assert!(!meta_path.exists(), "Metadata file should be removed");
    }

    #[test]
    fn test_get_cache_dir() {
        let cache_dir = get_cache_dir();
        assert!(cache_dir.to_string_lossy().contains(".bazbom"));
        assert!(cache_dir.to_string_lossy().contains("reachability"));
    }
}
