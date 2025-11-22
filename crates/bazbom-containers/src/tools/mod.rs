//! Container Security Tool Integrations
//!
//! This module provides a unified interface for running external security
//! scanning tools against container images.
//!
//! # Supported Tools
//!
//! - **Trivy** - Vulnerability, misconfiguration, and secrets scanning
//! - **Grype** - Backup vulnerability scanner
//! - **Syft** - SBOM generation
//! - **Dockle** - CIS Docker benchmark checks
//! - **Dive** - Image efficiency analysis
//! - **TruffleHog** - Deep secrets detection
//!
//! # Architecture
//!
//! All tools implement the `ContainerTool` trait which provides:
//! - Availability checking
//! - Async execution
//! - Structured output parsing
//!
//! The `ToolOrchestrator` runs multiple tools in parallel and aggregates results.

pub mod dive;
pub mod dockle;
pub mod findings;
pub mod grype;
pub mod orchestrator;
pub mod syft;
pub mod trivy;
pub mod trufflehog;

pub use findings::*;
pub use orchestrator::{OrchestratorConfig, ToolOrchestrator};

use anyhow::Result;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;
use tokio::time::timeout;

/// Trait for container security scanning tools
#[async_trait]
pub trait ContainerTool: Send + Sync {
    /// Human-readable name of the tool
    fn name(&self) -> &str;

    /// Check if the tool binary is available on the system
    fn is_available(&self) -> bool;

    /// Get the installation instructions for this tool
    fn install_hint(&self) -> &str;

    /// Run the tool against a container image
    ///
    /// # Arguments
    /// * `image` - Container image reference (e.g., "nginx:latest")
    /// * `output_dir` - Directory to write results
    async fn scan(&self, image: &str, output_dir: &Path) -> Result<ToolOutput>;
}

/// Output from a container scanning tool
#[derive(Debug, Clone)]
pub struct ToolOutput {
    /// Name of the tool that produced this output
    pub tool_name: String,

    /// Vulnerabilities found
    pub vulnerabilities: Vec<VulnerabilityFinding>,

    /// Secrets detected
    pub secrets: Vec<SecretFinding>,

    /// Misconfigurations found
    pub misconfigs: Vec<MisconfigFinding>,

    /// Benchmark/compliance results
    pub benchmarks: Vec<BenchmarkResult>,

    /// Image efficiency metrics
    pub efficiency: Option<EfficiencyMetrics>,

    /// SBOM packages (if tool generates SBOM)
    pub packages: Vec<PackageInfo>,

    /// Raw JSON output path (for debugging)
    pub raw_output_path: Option<std::path::PathBuf>,
}

impl ToolOutput {
    /// Create an empty output for a tool
    pub fn empty(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            vulnerabilities: Vec::new(),
            secrets: Vec::new(),
            misconfigs: Vec::new(),
            benchmarks: Vec::new(),
            efficiency: None,
            packages: Vec::new(),
            raw_output_path: None,
        }
    }
}

/// Check if a tool binary exists on the system
pub fn tool_exists(tool_name: &str) -> bool {
    which::which(tool_name).is_ok()
}

/// Run a command and capture output
pub async fn run_command(program: &str, args: &[&str]) -> Result<std::process::Output> {
    use tokio::process::Command;

    let output = timeout(
        Duration::from_secs(300),
        Command::new(program).args(args).output(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timed out running {}", program))?
    .map_err(|e| anyhow::anyhow!("Failed to run {}: {}", program, e))?;

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_exists() {
        // Common tools that should exist
        assert!(tool_exists("ls"));
        // Tool that shouldn't exist
        assert!(!tool_exists("definitely-not-a-real-tool-12345"));
    }

    #[test]
    fn test_empty_output() {
        let output = ToolOutput::empty("test-tool");
        assert_eq!(output.tool_name, "test-tool");
        assert!(output.vulnerabilities.is_empty());
    }
}
