//! Cryptographic hashing functions

use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of data
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Verify data matches expected hash
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let actual_hash = hash_data(data);

    // Constant-time comparison
    use subtle::ConstantTimeEq;
    actual_hash
        .as_bytes()
        .ct_eq(expected_hash.as_bytes())
        .into()
}

/// Compute SHA-256 hash of a file
pub fn hash_file(path: &std::path::Path) -> std::io::Result<String> {
    let data = std::fs::read(path)?;
    Ok(hash_data(&data))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_hash_data() {
        let data = b"Hello, BazBOM!";
        let hash = hash_data(data);

        assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex chars
        assert!(!hash.is_empty());

        // Same data should produce same hash
        let hash2 = hash_data(data);
        assert_eq!(hash, hash2);

        // Different data should produce different hash
        let hash3 = hash_data(b"Different data");
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_verify_hash() {
        let data = b"Test data";
        let hash = hash_data(data);

        // Correct hash should verify
        assert!(verify_hash(data, &hash));

        // Wrong hash should not verify
        assert!(!verify_hash(data, "wrong_hash"));

        // Wrong data should not verify
        assert!(!verify_hash(b"Wrong data", &hash));
    }

    #[test]
    fn test_hash_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let content = b"File content for hashing";
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content).unwrap();
        drop(file);

        let file_hash = hash_file(&file_path).unwrap();
        let data_hash = hash_data(content);

        assert_eq!(file_hash, data_hash);
    }

    #[test]
    fn test_known_hash_vectors() {
        // Test with known SHA-256 vectors
        let empty = hash_data(b"");
        assert_eq!(
            empty,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );

        let abc = hash_data(b"abc");
        assert_eq!(
            abc,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
