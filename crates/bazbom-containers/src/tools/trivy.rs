//! Trivy vulnerability scanner integration
//!
//! Trivy scans container images for:
//! - Vulnerabilities in OS packages and language dependencies
//! - Misconfigurations
//! - Secrets

use crate::tools::{
    findings::{MisconfigFinding, SecretFinding, Severity, VulnerabilityFinding},
    run_command, tool_exists, ContainerTool, ToolOutput,
};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;
use tracing::{info, warn};

/// Trivy scanner for container security analysis
pub struct TrivyScanner;

impl TrivyScanner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TrivyScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ContainerTool for TrivyScanner {
    fn name(&self) -> &str {
        "trivy"
    }

    fn is_available(&self) -> bool {
        tool_exists("trivy")
    }

    fn install_hint(&self) -> &str {
        "Install with: brew install trivy\nOr visit: https://trivy.dev/latest/getting-started/installation/"
    }

    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput> {
        info!("Running Trivy scan on {}", image);

        let output_file = output_dir.join("trivy-results.json");

        // Run trivy for vulnerability scanning only
        // (secrets handled by TruffleHog, misconfigs by Dockle)
        let output = run_command(
            "trivy",
            &[
                "image",
                "--format",
                "json",
                "--output",
                output_file.to_str().unwrap(),
                "--scanners",
                "vuln",
                "--severity",
                "UNKNOWN,LOW,MEDIUM,HIGH,CRITICAL",
                image,
            ],
        )
        .await
        .context("Failed to run Trivy")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Trivy exited with non-zero status: {}", stderr);
        }

        // Parse the JSON output
        let json_content = tokio::fs::read_to_string(&output_file)
            .await
            .context("Failed to read Trivy output")?;

        let trivy_output: TrivyOutput = match serde_json::from_str(&json_content) {
            Ok(output) => output,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "Failed to parse Trivy JSON at line {} column {}: {}",
                    e.line(),
                    e.column(),
                    e
                ));
            }
        };

        // Convert to our unified format
        let mut tool_output = ToolOutput::empty("trivy");
        tool_output.raw_output_path = Some(output_file);

        // Process results
        for result in trivy_output.results.unwrap_or_default() {
            // Process vulnerabilities
            for vuln in result.vulnerabilities.unwrap_or_default() {
                tool_output.vulnerabilities.push(VulnerabilityFinding {
                    cve_id: vuln.vulnerability_i_d.clone(),
                    package_name: vuln.pkg_name,
                    installed_version: vuln.installed_version,
                    fixed_version: vuln.fixed_version,
                    severity: Severity::from_str_loose(&vuln.severity),
                    cvss_score: vuln.cvss.and_then(|c| c.nvd.and_then(|n| n.v3_score.or(n.v2_score))),
                    title: vuln.title.unwrap_or_default(),
                    description: vuln.description.unwrap_or_default(),
                    layer_digest: vuln.layer.map(|l| l.digest),
                    source: "trivy".to_string(),
                    references: vuln.references.unwrap_or_default(),
                    epss_score: None,
                    epss_percentile: None,
                    is_kev: false,
                    kev_due_date: None,
                });
            }

            // Process secrets
            for secret in result.secrets.unwrap_or_default() {
                tool_output.secrets.push(SecretFinding {
                    secret_type: secret.rule_i_d.clone(),
                    severity: Severity::from_str_loose(&secret.severity),
                    file_path: secret.target,
                    line_number: Some(secret.start_line),
                    rule_id: secret.rule_i_d.clone(),
                    description: secret.title,
                    source: "trivy".to_string(),
                    layer_digest: None,
                });
            }

            // Process misconfigurations
            for misconfig in result.misconfigurations.unwrap_or_default() {
                tool_output.misconfigs.push(MisconfigFinding {
                    id: misconfig.id,
                    misconfig_type: misconfig.misconfig_type,
                    severity: Severity::from_str_loose(&misconfig.severity),
                    title: misconfig.title,
                    description: misconfig.description,
                    resolution: misconfig.resolution,
                    file_path: Some(result.target.clone()),
                    source: "trivy".to_string(),
                });
            }
        }

        info!(
            "Trivy found {} vulnerabilities, {} secrets, {} misconfigs",
            tool_output.vulnerabilities.len(),
            tool_output.secrets.len(),
            tool_output.misconfigs.len()
        );

        Ok(tool_output)
    }
}

// Trivy JSON output structures (minimal, add fields as needed)

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyOutput {
    results: Option<Vec<TrivyResult>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyResult {
    target: String,
    vulnerabilities: Option<Vec<TrivyVulnerability>>,
    secrets: Option<Vec<TrivySecret>>,
    misconfigurations: Option<Vec<TrivyMisconfig>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyVulnerability {
    vulnerability_i_d: String,
    pkg_name: String,
    installed_version: String,
    fixed_version: Option<String>,
    severity: String,
    title: Option<String>,
    description: Option<String>,
    references: Option<Vec<String>>,
    #[serde(rename = "CVSS")]
    cvss: Option<TrivyCvss>,
    layer: Option<TrivyLayer>,
}

#[derive(Debug, Deserialize)]
struct TrivyCvss {
    nvd: Option<TrivyCvssScore>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyCvssScore {
    #[serde(rename = "V3Score")]
    v3_score: Option<f64>,
    #[serde(rename = "V2Score")]
    v2_score: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyLayer {
    digest: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivySecret {
    rule_i_d: String,
    severity: String,
    title: String,
    target: String,
    start_line: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct TrivyMisconfig {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Type")]
    misconfig_type: String,
    severity: String,
    title: String,
    description: String,
    resolution: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trivy_available() {
        let scanner = TrivyScanner::new();
        // Just check the method works
        let _ = scanner.is_available();
    }
}
