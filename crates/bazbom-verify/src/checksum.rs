//! Checksum verification against GitHub releases

use anyhow::{Context, Result};
use bazbom_crypto::hashing::hash_file;
use std::path::Path;

use crate::github::fetch_release_checksums;

/// Verify binary checksum against GitHub release
pub fn verify_checksum(binary_path: &Path, version: &str, verbose: bool) -> Result<bool> {
    // Compute actual checksum
    let actual_checksum = hash_file(binary_path).context("Failed to hash binary")?;

    if verbose {
        println!("   Actual checksum: {}", actual_checksum);
    }

    // Fetch expected checksums from GitHub
    let checksums = fetch_release_checksums(version)?;

    // Determine architecture-specific expected checksum
    // For simplicity, we'll check if the actual checksum matches any in the release
    for (filename, expected) in &checksums {
        if actual_checksum == *expected {
            if verbose {
                println!("   Matched: {}", filename);
            }
            return Ok(true);
        }
    }

    if verbose {
        println!("   Expected checksums from release:");
        for (filename, checksum) in &checksums {
            println!("     {} → {}", filename, checksum);
        }
    }

    Ok(false)
}
