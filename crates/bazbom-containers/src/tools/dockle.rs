//! Dockle CIS Docker benchmark checker

use crate::tools::{
    findings::BenchmarkResult, run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

/// Dockle CIS benchmark scanner
pub struct DockleScanner;

impl DockleScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DockleScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for DockleScanner {
    fn name(&self) -> &str {
        "dockle"
    }

    fn is_available(&self) -> bool {
        tool_exists("dockle")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install goodwithtech/r/dockle\nOr visit: https://github.com/goodwithtech/dockle#installation"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running Dockle CIS benchmark check on {}", image);

        let output_file = output_dir.join("dockle-results.json");

        let output = run_command(
            "dockle",
            &[
                "-f",
                "json",
                "-o",
                output_file.to_str().unwrap(),
                image,
            ],
        )
        .await
        .context("Failed to run Dockle")?;

        // Dockle returns non-zero on findings, which is expected
        let _ = output.status;

        let json_content = tokio::fs::read_to_string(&output_file)
            .await
            .context("Failed to read Dockle output")?;

        let dockle_output: DockleOutput =
            serde_json::from_str(&json_content).context("Failed to parse Dockle JSON")?;

        let mut tool_output = ToolOutput::empty("dockle");
        tool_output.raw_output_path = Some(output_file);

        // Convert details to benchmark results
        for detail in dockle_output.details {
            tool_output.benchmarks.push(BenchmarkResult {
                check_id: detail.code,
                level: detail.level.clone(),
                title: detail.title,
                description: detail.alerts.join("; "),
                passed: detail.level == "PASS" || detail.level == "SKIP",
                source: "dockle".to_string(),
            });
        }

        info!("Dockle completed {} checks", tool_output.benchmarks.len());

        Ok(tool_output)
    }
}

#[derive(Debug, Deserialize)]
struct DockleOutput {
    details: Vec<DockleDetail>,
}

#[derive(Debug, Deserialize)]
struct DockleDetail {
    code: String,
    title: String,
    level: String,
    alerts: Vec<String>,
}
