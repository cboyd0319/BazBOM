//! Syft SBOM generator integration

use crate::tools::{
    findings::PackageInfo, run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

/// Syft SBOM generator
pub struct SyftScanner;

impl SyftScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SyftScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for SyftScanner {
    fn name(&self) -> &str {
        "syft"
    }

    fn is_available(&self) -> bool {
        tool_exists("syft")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install syft\nOr visit: https://github.com/anchore/syft#installation"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running Syft SBOM generation on {}", image);

        let output_file = output_dir.join("syft-sbom.json");
        let spdx_file = output_dir.join("sbom.spdx.json");
        let cyclonedx_file = output_dir.join("sbom.cdx.json");

        // Generate native JSON (for parsing), SPDX, and CycloneDX all at once
        let output = run_command(
            "syft",
            &[
                image,
                "-o",
                &format!("json={}", output_file.to_str().unwrap()),
                "-o",
                &format!("spdx-json={}", spdx_file.to_str().unwrap()),
                "-o",
                &format!("cyclonedx-json={}", cyclonedx_file.to_str().unwrap()),
            ],
        )
        .await
        .context("Failed to run Syft")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Syft exited with non-zero status: {}", stderr);
        }

        let json_content = tokio::fs::read_to_string(&output_file)
            .await
            .context("Failed to read Syft output")?;

        let syft_output: SyftOutput =
            serde_json::from_str(&json_content).context("Failed to parse Syft JSON")?;

        let mut tool_output = ToolOutput::empty("syft");
        tool_output.raw_output_path = Some(output_file);

        // Convert artifacts to packages
        for artifact in syft_output.artifacts {
            tool_output.packages.push(PackageInfo {
                name: artifact.name,
                version: artifact.version,
                purl: artifact.purl,
                pkg_type: artifact.pkg_type,
                licenses: artifact
                    .licenses
                    .unwrap_or_default()
                    .into_iter()
                    .map(|l| l.value)
                    .collect(),
                layer_digest: artifact
                    .metadata
                    .and_then(|m| m.layer_digest),
                locations: artifact
                    .locations
                    .into_iter()
                    .map(|l| l.path)
                    .collect(),
            });
        }

        info!("Syft found {} packages", tool_output.packages.len());

        Ok(tool_output)
    }
}

#[derive(Debug, Deserialize)]
struct SyftOutput {
    artifacts: Vec<SyftArtifact>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyftArtifact {
    name: String,
    version: String,
    #[serde(rename = "type")]
    pkg_type: String,
    purl: Option<String>,
    licenses: Option<Vec<SyftLicense>>,
    locations: Vec<SyftLocation>,
    metadata: Option<SyftMetadata>,
}

#[derive(Debug, Deserialize)]
struct SyftLicense {
    value: String,
}

#[derive(Debug, Deserialize)]
struct SyftLocation {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SyftMetadata {
    layer_digest: Option<String>,
}
