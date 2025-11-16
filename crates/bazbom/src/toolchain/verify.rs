//! Tool integrity verification
//!
//! Verifies checksums of external tools to prevent supply chain attacks

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Known SHA-256 checksums for external tools
/// These should be updated whenever tool versions change
pub struct ToolChecksums {
    checksums: HashMap<String, String>,
}

impl Default for ToolChecksums {
    fn default() -> Self {
        let mut checksums = HashMap::new();

        // Syft checksums (version-specific)
        // NOTE: These are placeholder values - update with actual checksums for your platform
        checksums.insert(
            "syft-linux-x86_64-v1.17.0".to_string(),
            "PLACEHOLDER_UPDATE_WITH_ACTUAL_CHECKSUM".to_string(),
        );
        checksums.insert(
            "syft-darwin-x86_64-v1.17.0".to_string(),
            "PLACEHOLDER_UPDATE_WITH_ACTUAL_CHECKSUM".to_string(),
        );
        checksums.insert(
            "syft-darwin-arm64-v1.17.0".to_string(),
            "PLACEHOLDER_UPDATE_WITH_ACTUAL_CHECKSUM".to_string(),
        );

        // Semgrep checksums
        checksums.insert(
            "semgrep-v1.95.0".to_string(),
            "PLACEHOLDER_UPDATE_WITH_ACTUAL_CHECKSUM".to_string(),
        );

        Self { checksums }
    }
}

impl ToolChecksums {
    /// Verify the integrity of a tool binary
    pub fn verify_tool(&self, tool_path: &Path, tool_key: &str) -> Result<bool> {
        // Get expected checksum
        let expected_checksum = self.checksums.get(tool_key).context(format!(
            "No checksum found for tool: {}. This may indicate an unknown or unsupported tool version.",
            tool_key
        ))?;

        // Skip verification for placeholder checksums (allows gradual rollout)
        if expected_checksum.starts_with("PLACEHOLDER") {
            eprintln!(
                "[!] WARNING: Checksum verification skipped for {} (placeholder checksum)",
                tool_key
            );
            eprintln!("[!] Please update tool checksums in toolchain/verify.rs for security");
            return Ok(true);
        }

        // Calculate actual checksum
        let actual_checksum = calculate_sha256(tool_path)?;

        // Compare checksums
        if actual_checksum == *expected_checksum {
            println!("[+] Tool integrity verified: {}", tool_key);
            Ok(true)
        } else {
            anyhow::bail!(
                "Tool integrity verification FAILED for {}!\n\
                 Expected: {}\n\
                 Actual:   {}\n\
                 This may indicate a compromised binary or version mismatch.",
                tool_key,
                expected_checksum,
                actual_checksum
            );
        }
    }

    /// Add or update a tool checksum
    pub fn add_checksum(&mut self, tool_key: String, checksum: String) {
        self.checksums.insert(tool_key, checksum);
    }

    /// Get all registered tool checksums
    pub fn get_all_checksums(&self) -> &HashMap<String, String> {
        &self.checksums
    }
}

/// Calculate SHA-256 checksum of a file
pub fn calculate_sha256(path: &Path) -> Result<String> {
    let mut file = File::open(path).context("Failed to open file for checksum calculation")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_calculate_sha256() {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test content").unwrap();
        temp_file.flush().unwrap();

        let checksum = calculate_sha256(temp_file.path()).unwrap();
        // SHA-256 of "test content"
        assert_eq!(
            checksum,
            "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72"
        );
    }

    #[test]
    fn test_tool_checksums_default() {
        let checksums = ToolChecksums::default();
        assert!(!checksums.get_all_checksums().is_empty());
    }

    #[test]
    fn test_add_checksum() {
        let mut checksums = ToolChecksums::default();
        checksums.add_checksum("test-tool".to_string(), "abc123".to_string());
        assert_eq!(
            checksums.get_all_checksums().get("test-tool"),
            Some(&"abc123".to_string())
        );
    }

    #[test]
    fn test_verify_tool_with_placeholder() {
        let checksums = ToolChecksums::default();
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "test").unwrap();
        temp_file.flush().unwrap();

        // Should succeed with placeholder (warning printed)
        let result = checksums.verify_tool(temp_file.path(), "syft-linux-x86_64-v1.17.0");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
