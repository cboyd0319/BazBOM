//! Dive image efficiency analyzer

use crate::tools::{
    findings::{EfficiencyMetrics, LayerEfficiency},
    run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

/// Dive image efficiency analyzer
pub struct DiveScanner;

impl DiveScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiveScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for DiveScanner {
    fn name(&self) -> &str {
        "dive"
    }

    fn is_available(&self) -> bool {
        tool_exists("dive")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install dive\nOr visit: https://github.com/wagoodman/dive#installation"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running Dive efficiency analysis on {}", image);

        let output_file = output_dir.join("dive-analysis.json");

        // Dive outputs JSON when given --json flag
        let output = run_command(
            "dive",
            &[
                image,
                "--json",
                output_file.to_str().unwrap(),
                "--ci", // Non-interactive mode
            ],
        )
        .await
        .context("Failed to run Dive")?;

        // Dive may return non-zero if efficiency is below threshold
        let _ = output.status;

        let json_content = tokio::fs::read_to_string(&output_file)
            .await
            .context("Failed to read Dive output")?;

        let dive_output: DiveOutput = serde_json::from_str(&json_content)
            .with_context(|| format!("Failed to parse Dive JSON: {}", &json_content[..500.min(json_content.len())]))?;

        let mut tool_output = ToolOutput::empty("dive");
        tool_output.raw_output_path = Some(output_file);

        // Extract efficiency metrics
        let layers: Vec<LayerEfficiency> = dive_output
            .layer
            .into_iter()
            .map(|l| LayerEfficiency {
                digest: l.digest,
                size: l.size_bytes,
                command: l.command,
                wasted_bytes: 0, // Dive doesn't provide per-layer waste in JSON
            })
            .collect();

        tool_output.efficiency = Some(EfficiencyMetrics {
            efficiency_score: dive_output.image.efficiency_score,
            image_size: dive_output.image.size_bytes,
            wasted_bytes: dive_output.image.inefficient_bytes,
            layer_count: layers.len(),
            layers,
        });

        info!(
            "Dive analysis complete: {:.1}% efficiency",
            dive_output.image.efficiency_score * 100.0
        );

        Ok(tool_output)
    }
}

#[derive(Debug, Deserialize)]
struct DiveOutput {
    image: DiveImage,
    layer: Vec<DiveLayer>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiveImage {
    efficiency_score: f64,
    size_bytes: u64,
    inefficient_bytes: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DiveLayer {
    #[serde(rename = "digestId")]
    digest: String,
    size_bytes: u64,
    command: Option<String>,
}
