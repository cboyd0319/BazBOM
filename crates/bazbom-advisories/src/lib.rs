//! Security advisory database integration for BazBOM
//!
//! This crate provides comprehensive vulnerability intelligence by integrating
//! with multiple security advisory sources:
//! - OSV (Open Source Vulnerabilities) database
//! - NVD (National Vulnerability Database)
//! - GHSA (GitHub Security Advisories)
//!
//! Features:
//! - Batch vulnerability queries for efficient lookups
//! - EPSS (Exploit Prediction Scoring System) enrichment
//! - CISA KEV (Known Exploited Vulnerabilities) catalog integration
//! - Priority scoring (P0-P4) based on CVSS, EPSS, and KEV status
//! - Vulnerability deduplication and merging across sources
//! - Version range matching for affected package detection

use anyhow::Result;
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::Path;
use time::OffsetDateTime;

pub mod enrichment;
pub mod merge;
pub mod osv;
pub mod parsers;
pub mod version_match;

pub use enrichment::{load_epss_scores, load_kev_catalog};
pub use merge::{
    calculate_priority, merge_vulnerabilities, AffectedPackage, EpssScore, KevEntry, Priority,
    Reference, Severity, SeverityLevel, VersionEvent, VersionRange, Vulnerability,
};
pub use osv::{query_batch_vulnerabilities, query_package_vulnerabilities};
pub use parsers::{parse_ghsa_entry, parse_nvd_entry, parse_osv_entry};
pub use version_match::is_version_affected;

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: String,
    pub bytes: u64,
    pub blake3: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub generated_at: String,
    pub files: Vec<ManifestEntry>,
}

fn write_file<P: AsRef<Path>>(path: P, content: &[u8]) -> Result<ManifestEntry> {
    let path_ref = path.as_ref();
    if let Some(parent) = path_ref.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path_ref, content)?;
    let hash = blake3::hash(content).to_hex().to_string();
    Ok(ManifestEntry {
        path: path_ref.to_string_lossy().into_owned(),
        bytes: content.len() as u64,
        blake3: hash,
    })
}

fn fetch_or_placeholder(url: &str, placeholder: &[u8]) -> Vec<u8> {
    match ureq::get(url).call() {
        Ok(mut resp) if resp.status() == 200 => {
            let mut buf = Vec::new();
            if resp.body_mut().as_reader().read_to_end(&mut buf).is_ok() {
                buf
            } else {
                placeholder.to_vec()
            }
        }
        _ => placeholder.to_vec(),
    }
}

pub fn db_sync<P: AsRef<Path>>(cache_dir: P, offline: bool) -> Result<Manifest> {
    let cache_dir = cache_dir.as_ref();
    fs::create_dir_all(cache_dir)?;
    let mut files = Vec::new();

    // Deterministic placeholders
    let placeholder_json = b"{\n  \"note\": \"offline or fetch failed\"\n}\n";
    let placeholder_csv = b"cve,score\n";

    // OSV (placeholder)
    let osv_bytes = if offline {
        placeholder_json.to_vec()
    } else {
        fetch_or_placeholder("https://api.osv.dev/v1/vulns", placeholder_json)
    };
    files.push(write_file(
        cache_dir.join("advisories/osv.json"),
        &osv_bytes,
    )?);

    // NVD (placeholder)
    let nvd_bytes = if offline {
        placeholder_json.to_vec()
    } else {
        fetch_or_placeholder(
            "https://services.nvd.nist.gov/rest/json/cves/2.0?resultsPerPage=1",
            placeholder_json,
        )
    };
    files.push(write_file(
        cache_dir.join("advisories/nvd.json"),
        &nvd_bytes,
    )?);

    // GHSA (placeholder)
    let ghsa_bytes = placeholder_json.to_vec(); // GitHub token required for real calls
    files.push(write_file(
        cache_dir.join("advisories/ghsa.json"),
        &ghsa_bytes,
    )?);

    // CISA KEV
    let kev_bytes = if offline {
        placeholder_json.to_vec()
    } else {
        fetch_or_placeholder(
            "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json",
            placeholder_json,
        )
    };
    files.push(write_file(
        cache_dir.join("advisories/kev.json"),
        &kev_bytes,
    )?);

    // EPSS - download gzipped, decompress before writing
    let epss_bytes = if offline {
        placeholder_csv.to_vec()
    } else {
        let gzipped_data = fetch_or_placeholder(
            "https://epss.cyentia.com/epss_scores-current.csv.gz",
            placeholder_csv,
        );

        // Decompress gzipped EPSS data
        let mut decoder = GzDecoder::new(&gzipped_data[..]);
        let mut decompressed = Vec::new();
        match decoder.read_to_end(&mut decompressed) {
            Ok(_) => decompressed,
            Err(e) => {
                eprintln!("Warning: Failed to decompress EPSS data: {}. Using placeholder.", e);
                placeholder_csv.to_vec()
            }
        }
    };
    files.push(write_file(
        cache_dir.join("advisories/epss.csv"),
        &epss_bytes,
    )?);

    // Manifest
    let now = OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "".into());
    let manifest = Manifest {
        generated_at: now,
        files,
    };
    let data = serde_json::to_vec_pretty(&manifest)?;
    write_file(cache_dir.join("manifest.json"), &data)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_sync_offline() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");

        let result = db_sync(&cache_dir, true);
        assert!(result.is_ok());

        let manifest = result.unwrap();
        assert_eq!(manifest.files.len(), 5);

        // Check that files were created
        assert!(cache_dir.join("advisories/osv.json").exists());
        assert!(cache_dir.join("advisories/nvd.json").exists());
        assert!(cache_dir.join("advisories/ghsa.json").exists());
        assert!(cache_dir.join("advisories/kev.json").exists());
        assert!(cache_dir.join("advisories/epss.csv").exists());
        assert!(cache_dir.join("manifest.json").exists());
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest {
            generated_at: "2024-01-01T00:00:00Z".to_string(),
            files: vec![ManifestEntry {
                path: "test.json".to_string(),
                bytes: 100,
                blake3: "abc123".to_string(),
            }],
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("2024-01-01T00:00:00Z"));
        assert!(json.contains("test.json"));
    }

    #[test]
    fn test_write_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.txt");
        let content = b"test content";

        let result = write_file(&path, content);
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.bytes, content.len() as u64);
        assert!(!entry.blake3.is_empty());

        // Verify file was written
        let written = fs::read(&path).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_write_file_creates_parent_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nested/dir/test.txt");
        let content = b"test";

        let result = write_file(&path, content);
        assert!(result.is_ok());
        assert!(path.exists());
    }

    #[test]
    fn test_write_file_with_empty_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("empty.txt");
        let content = b"";

        let result = write_file(&path, content);
        assert!(result.is_ok());

        let entry = result.unwrap();
        assert_eq!(entry.bytes, 0);
        assert!(!entry.blake3.is_empty()); // BLAKE3 of empty string
    }

    #[test]
    fn test_manifest_deserialization() {
        let json = r#"{
            "generated_at": "2024-01-01T00:00:00Z",
            "files": [
                {
                    "path": "test.json",
                    "bytes": 100,
                    "blake3": "abc123"
                }
            ]
        }"#;

        let manifest: Manifest = serde_json::from_str(json).unwrap();
        assert_eq!(manifest.generated_at, "2024-01-01T00:00:00Z");
        assert_eq!(manifest.files.len(), 1);
        assert_eq!(manifest.files[0].path, "test.json");
    }

    #[test]
    fn test_db_sync_creates_all_expected_files() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");

        let manifest = db_sync(&cache_dir, true).unwrap();

        // Verify all expected files are in manifest
        let file_paths: Vec<String> = manifest.files.iter().map(|f| f.path.clone()).collect();
        assert!(file_paths.iter().any(|p| p.contains("osv.json")));
        assert!(file_paths.iter().any(|p| p.contains("nvd.json")));
        assert!(file_paths.iter().any(|p| p.contains("ghsa.json")));
        assert!(file_paths.iter().any(|p| p.contains("kev.json")));
        assert!(file_paths.iter().any(|p| p.contains("epss.csv")));
    }

    #[test]
    fn test_db_sync_manifest_has_timestamp() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_dir = tmp.path().join("cache");

        let manifest = db_sync(&cache_dir, true).unwrap();

        // Verify timestamp is present and not empty
        assert!(!manifest.generated_at.is_empty());
    }

    #[test]
    fn test_manifest_entry_has_correct_fields() {
        let entry = ManifestEntry {
            path: "test.json".to_string(),
            bytes: 42,
            blake3: "hash123".to_string(),
        };

        assert_eq!(entry.path, "test.json");
        assert_eq!(entry.bytes, 42);
        assert_eq!(entry.blake3, "hash123");
    }

    #[test]
    fn test_write_file_overwrite_existing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("test.txt");

        // Write first time
        let result1 = write_file(&path, b"first");
        assert!(result1.is_ok());
        let entry1 = result1.unwrap();

        // Write second time (overwrite)
        let result2 = write_file(&path, b"second content");
        assert!(result2.is_ok());
        let entry2 = result2.unwrap();

        // Verify new content and different hash
        assert_ne!(entry1.bytes, entry2.bytes);
        assert_ne!(entry1.blake3, entry2.blake3);

        let content = fs::read(&path).unwrap();
        assert_eq!(content, b"second content");
    }

    #[test]
    fn test_blake3_hashes_are_deterministic() {
        let tmp = tempfile::tempdir().unwrap();
        let content = b"test content";

        // Write same content twice to different files
        let path1 = tmp.path().join("file1.txt");
        let path2 = tmp.path().join("file2.txt");

        let entry1 = write_file(&path1, content).unwrap();
        let entry2 = write_file(&path2, content).unwrap();

        // Hashes should be identical for same content
        assert_eq!(entry1.blake3, entry2.blake3);
    }
}
