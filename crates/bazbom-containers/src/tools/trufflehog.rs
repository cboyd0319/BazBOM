//! TruffleHog secrets scanner integration

use crate::tools::{
    findings::{SecretFinding, Severity},
    run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::info;

/// TruffleHog secrets scanner
pub struct TruffleHogScanner;

impl TruffleHogScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TruffleHogScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for TruffleHogScanner {
    fn name(&self) -> &str {
        "trufflehog"
    }

    fn is_available(&self) -> bool {
        tool_exists("trufflehog")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install trufflehog\nOr visit: https://github.com/trufflesecurity/trufflehog#installation"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running TruffleHog secrets scan on {}", image);

        let output_file = output_dir.join("trufflehog-secrets.json");

        // TruffleHog outputs JSONL (one JSON per line)
        // Only return verified secrets to reduce noise
        let output = run_command(
            "trufflehog",
            &[
                "docker",
                "--image",
                image,
                "--json",
                "--no-update",
                "--only-verified",
            ],
        )
        .await
        .context("Failed to run TruffleHog")?;

        // Write output to file
        tokio::fs::write(&output_file, &output.stdout)
            .await
            .context("Failed to write TruffleHog output")?;

        let mut tool_output = ToolOutput::empty("trufflehog");
        tool_output.raw_output_path = Some(output_file.clone());

        // Parse JSONL output (one finding per line)
        let content = String::from_utf8_lossy(&output.stdout);
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            if let Ok(finding) = serde_json::from_str::<TruffleHogFinding>(line) {
                tool_output.secrets.push(SecretFinding {
                    secret_type: finding.detector_name,
                    severity: if finding.verified {
                        Severity::Critical
                    } else {
                        Severity::High
                    },
                    file_path: finding.source_metadata.file.unwrap_or_default(),
                    line_number: finding.source_metadata.line,
                    rule_id: finding.detector_type,
                    description: format!(
                        "{}{}",
                        finding.raw.chars().take(20).collect::<String>(),
                        if finding.raw.len() > 20 { "..." } else { "" }
                    ),
                    source: "trufflehog".to_string(),
                    layer_digest: finding.source_metadata.layer,
                });
            }
        }

        info!("TruffleHog found {} secrets", tool_output.secrets.len());

        Ok(tool_output)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TruffleHogFinding {
    detector_name: String,
    detector_type: String,
    verified: bool,
    raw: String,
    source_metadata: TruffleHogSource,
}

#[derive(Debug, Deserialize)]
struct TruffleHogSource {
    file: Option<String>,
    line: Option<u32>,
    layer: Option<String>,
}
