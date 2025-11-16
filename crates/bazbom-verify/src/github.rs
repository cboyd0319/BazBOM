//! GitHub release data fetching

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::collections::HashMap;

/// Fetch SHA-256 checksums for a release from GitHub
pub fn fetch_release_checksums(version: &str) -> Result<HashMap<String, String>> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    // Construct GitHub API URL for release
    let url = format!(
        "https://api.github.com/repos/cboyd0319/BazBOM/releases/tags/{}",
        version
    );

    let response = client
        .get(&url)
        .header("User-Agent", "bazbom-verify")
        .send()
        .context("Failed to fetch release data from GitHub")?;

    if !response.status().is_success() {
        anyhow::bail!(
            "GitHub API returned status {}: version {} not found",
            response.status(),
            version
        );
    }

    let release: Value = response.json().context("Failed to parse GitHub response")?;

    // Find checksums file in assets
    let mut checksums = HashMap::new();

    if let Some(assets) = release["assets"].as_array() {
        for asset in assets {
            if let Some(name) = asset["name"].as_str() {
                if name.ends_with(".sha256") {
                    // Download checksum file
                    if let Some(download_url) = asset["browser_download_url"].as_str() {
                        let content = client
                            .get(download_url)
                            .header("User-Agent", "bazbom-verify")
                            .send()
                            .context("Failed to download checksum file")?
                            .text()?;

                        // Parse checksum file (format: "HASH  filename")
                        for line in content.lines() {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let hash = parts[0];
                                let filename = parts[1];
                                checksums.insert(filename.to_string(), hash.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    if checksums.is_empty() {
        anyhow::bail!("No checksums found in release {}", version);
    }

    Ok(checksums)
}
