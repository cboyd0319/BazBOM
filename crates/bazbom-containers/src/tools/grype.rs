//! Grype vulnerability scanner integration
//!
//! Grype provides a second opinion on vulnerabilities, useful for
//! comparison and validation against Trivy results.

use crate::tools::{
    findings::{Severity, VulnerabilityFinding},
    run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::{info, warn};

/// Grype vulnerability scanner
pub struct GrypeScanner;

impl GrypeScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GrypeScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for GrypeScanner {
    fn name(&self) -> &str {
        "grype"
    }

    fn is_available(&self) -> bool {
        tool_exists("grype")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install grype\nOr visit: https://github.com/anchore/grype#installation"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running Grype scan on {}", image);

        let output_file = output_dir.join("grype-results.json");

        let output = run_command(
            "grype",
            &[
                image,
                "-o",
                "json",
                "--file",
                output_file.to_str().unwrap(),
            ],
        )
        .await
        .context("Failed to run Grype")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Grype exited with non-zero status: {}", stderr);
        }

        // Parse JSON output
        let json_content = tokio::fs::read_to_string(&output_file)
            .await
            .context("Failed to read Grype output")?;

        let grype_output: GrypeOutput =
            serde_json::from_str(&json_content).context("Failed to parse Grype JSON")?;

        let mut tool_output = ToolOutput::empty("grype");
        tool_output.raw_output_path = Some(output_file);

        // Convert matches to findings
        for m in grype_output.matches {
            let vuln = &m.vulnerability;
            let artifact = &m.artifact;

            tool_output.vulnerabilities.push(VulnerabilityFinding {
                cve_id: vuln.id.clone(),
                package_name: artifact.name.clone(),
                installed_version: artifact.version.clone(),
                fixed_version: vuln.fix.as_ref().and_then(|f| {
                    if f.state == "fixed" {
                        f.versions.first().cloned()
                    } else {
                        None
                    }
                }),
                severity: Severity::from_str_loose(&vuln.severity),
                cvss_score: vuln.cvss.first().map(|c| c.metrics.base_score),
                title: vuln.id.clone(), // Grype doesn't provide title
                description: vuln.description.clone().unwrap_or_default(),
                layer_digest: None, // Grype doesn't provide layer info
                source: "grype".to_string(),
                references: vuln.urls.clone().unwrap_or_default(),
                epss_score: None,
                epss_percentile: None,
                is_kev: false,
                kev_due_date: None,
            });
        }

        info!("Grype found {} vulnerabilities", tool_output.vulnerabilities.len());

        Ok(tool_output)
    }
}

// Grype JSON structures

#[derive(Debug, Deserialize)]
struct GrypeOutput {
    matches: Vec<GrypeMatch>,
}

#[derive(Debug, Deserialize)]
struct GrypeMatch {
    vulnerability: GrypeVulnerability,
    artifact: GrypeArtifact,
}

#[derive(Debug, Deserialize)]
struct GrypeVulnerability {
    id: String,
    severity: String,
    description: Option<String>,
    urls: Option<Vec<String>>,
    fix: Option<GrypeFix>,
    cvss: Vec<GrypeCvss>,
}

#[derive(Debug, Deserialize)]
struct GrypeFix {
    state: String,
    versions: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct GrypeCvss {
    metrics: GrypeCvssMetrics,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GrypeCvssMetrics {
    base_score: f64,
}

#[derive(Debug, Deserialize)]
struct GrypeArtifact {
    name: String,
    version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grype_available() {
        let scanner = GrypeScanner::new();
        let _ = scanner.is_available();
    }
}
