//! Parallel tool orchestration
//!
//! Runs multiple container scanning tools concurrently and aggregates results.

use crate::tools::{
    dive::DiveScanner,
    dockle::DockleScanner,
    findings::{AggregatedResults, ScanSummary, Severity},
    grype::GrypeScanner,
    syft::SyftScanner,
    trivy::TrivyScanner,
    trufflehog::TruffleHogScanner,
    ContainerTool, ToolOutput,
};
use anyhow::Result;
use futures::future::join_all;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{error, info, warn};

/// Configuration for tool orchestration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Enable Trivy scanning
    pub enable_trivy: bool,
    /// Enable Grype scanning
    pub enable_grype: bool,
    /// Enable Syft SBOM generation
    pub enable_syft: bool,
    /// Enable Dockle benchmarks
    pub enable_dockle: bool,
    /// Enable Dive efficiency analysis
    pub enable_dive: bool,
    /// Enable TruffleHog secrets scanning
    pub enable_trufflehog: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            enable_trivy: true,
            enable_grype: true,
            enable_syft: true,
            enable_dockle: true,
            enable_dive: true,
            enable_trufflehog: true,
        }
    }
}

/// Orchestrates parallel execution of container scanning tools
pub struct ToolOrchestrator {
    config: OrchestratorConfig,
    output_dir: PathBuf,
}

impl ToolOrchestrator {
    /// Create a new orchestrator with default configuration
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            config: OrchestratorConfig::default(),
            output_dir: output_dir.into(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(output_dir: impl Into<PathBuf>, config: OrchestratorConfig) -> Self {
        Self {
            config,
            output_dir: output_dir.into(),
        }
    }

    /// Check which tools are available and return missing ones
    pub fn check_tools(&self) -> Vec<(String, String)> {
        let mut missing = Vec::new();

        let tools: Vec<(bool, Box<dyn ContainerTool>)> = vec![
            (self.config.enable_trivy, Box::new(TrivyScanner::new())),
            (self.config.enable_grype, Box::new(GrypeScanner::new())),
            (self.config.enable_syft, Box::new(SyftScanner::new())),
            (self.config.enable_dockle, Box::new(DockleScanner::new())),
            (self.config.enable_dive, Box::new(DiveScanner::new())),
            (
                self.config.enable_trufflehog,
                Box::new(TruffleHogScanner::new()),
            ),
        ];

        for (enabled, tool) in tools {
            if enabled && !tool.is_available() {
                missing.push((tool.name().to_string(), tool.install_hint().to_string()));
            }
        }

        missing
    }

    /// Run all enabled tools in parallel and aggregate results
    pub async fn scan(&self, image: &str) -> Result<AggregatedResults> {
        info!("Starting parallel container scan of {}", image);

        // Ensure output directory exists
        tokio::fs::create_dir_all(&self.output_dir).await?;

        // Build list of tools to run
        let mut tasks = Vec::new();
        let mut executed_tools: Vec<String> = Vec::new();
        let mut skipped_tools: Vec<String> = Vec::new();

        if self.config.enable_trivy {
            let tool: Arc<dyn ContainerTool> = Arc::new(TrivyScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("Trivy not available, skipping");
            }
        }

        if self.config.enable_grype {
            let tool: Arc<dyn ContainerTool> = Arc::new(GrypeScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("Grype not available, skipping");
            }
        }

        if self.config.enable_syft {
            let tool: Arc<dyn ContainerTool> = Arc::new(SyftScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("Syft not available, skipping");
            }
        }

        if self.config.enable_dockle {
            let tool: Arc<dyn ContainerTool> = Arc::new(DockleScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("Dockle not available, skipping");
            }
        }

        if self.config.enable_dive {
            let tool: Arc<dyn ContainerTool> = Arc::new(DiveScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("Dive not available, skipping");
            }
        }

        if self.config.enable_trufflehog {
            let tool: Arc<dyn ContainerTool> = Arc::new(TruffleHogScanner::new());
            if tool.is_available() {
                executed_tools.push(tool.name().to_string());
                tasks.push(self.run_tool(tool, image));
            } else {
                skipped_tools.push(tool.name().to_string());
                warn!("TruffleHog not available, skipping");
            }
        }

        // Run all tools in parallel
        let results = join_all(tasks).await;

        // Aggregate results
        let mut aggregated = AggregatedResults {
            image_name: image.to_string(),
            executed_tools,
            skipped_tools,
            vulnerabilities: Vec::new(),
            secrets: Vec::new(),
            misconfigs: Vec::new(),
            benchmarks: Vec::new(),
            efficiency: None,
            packages: Vec::new(),
            summary: ScanSummary::default(),
        };

        for result in results {
            match result {
                Ok(output) => {
                    aggregated.vulnerabilities.extend(output.vulnerabilities);
                    aggregated.secrets.extend(output.secrets);
                    aggregated.misconfigs.extend(output.misconfigs);
                    aggregated.benchmarks.extend(output.benchmarks);
                    if output.efficiency.is_some() {
                        aggregated.efficiency = output.efficiency;
                    }
                    aggregated.packages.extend(output.packages);
                }
                Err(e) => {
                    error!("Tool failed: {}", e);
                }
            }
        }

        // Deduplicate vulnerabilities by CVE ID
        aggregated.vulnerabilities = deduplicate_vulns(aggregated.vulnerabilities);

        // Calculate summary
        aggregated.summary = calculate_summary(&aggregated);

        info!(
            "Scan complete: {} vulns, {} secrets, {} packages",
            aggregated.summary.total_vulnerabilities,
            aggregated.summary.secrets_count,
            aggregated.summary.total_packages
        );

        Ok(aggregated)
    }

    async fn run_tool(&self, tool: Arc<dyn ContainerTool>, image: &str) -> Result<ToolOutput> {
        let output_dir = self.output_dir.clone();
        tool.scan(image, &output_dir).await
    }
}

/// Deduplicate vulnerabilities by CVE ID, preferring entries with more info
fn deduplicate_vulns(
    vulns: Vec<crate::tools::findings::VulnerabilityFinding>,
) -> Vec<crate::tools::findings::VulnerabilityFinding> {
    let mut merged: HashMap<String, crate::tools::findings::VulnerabilityFinding> = HashMap::new();

    for vuln in vulns {
        merged
            .entry(vuln.cve_id.clone())
            .and_modify(|existing| {
                // Prefer higher severity
                if vuln.severity > existing.severity {
                    existing.severity = vuln.severity;
                }
                // Prefer highest CVSS
                if vuln
                    .cvss_score
                    .map_or(false, |s| existing.cvss_score.map_or(true, |e| s > e))
                {
                    existing.cvss_score = vuln.cvss_score;
                }
                // Keep a fixed version if missing
                if existing.fixed_version.is_none() {
                    existing.fixed_version = vuln.fixed_version.clone();
                }
                // Merge references
                let mut refs: HashSet<String> = existing.references.iter().cloned().collect();
                refs.extend(vuln.references.iter().cloned());
                existing.references = refs.into_iter().collect();
                // Prefer layer info when absent
                if existing.layer_digest.is_none() {
                    existing.layer_digest = vuln.layer_digest.clone();
                }
                // Preserve enriched flags if present
                existing.epss_score = existing.epss_score.or(vuln.epss_score);
                existing.epss_percentile = existing.epss_percentile.or(vuln.epss_percentile);
                existing.is_kev = existing.is_kev || vuln.is_kev;
                existing.kev_due_date = existing.kev_due_date.clone().or(vuln.kev_due_date.clone());
            })
            .or_insert(vuln);
    }

    let mut result: Vec<_> = merged.into_values().collect();
    result.sort_by(|a, b| a.cve_id.cmp(&b.cve_id));
    result
}

/// Calculate summary statistics
fn calculate_summary(results: &AggregatedResults) -> ScanSummary {
    let mut summary = ScanSummary {
        total_packages: results.packages.len(),
        total_vulnerabilities: results.vulnerabilities.len(),
        secrets_count: results.secrets.len(),
        misconfigs_count: results.misconfigs.len(),
        ..Default::default()
    };

    for vuln in &results.vulnerabilities {
        match vuln.severity {
            Severity::Critical => summary.critical_count += 1,
            Severity::High => summary.high_count += 1,
            Severity::Medium => summary.medium_count += 1,
            Severity::Low => summary.low_count += 1,
            Severity::Unknown => {}
        }

        if vuln.is_fixable() {
            summary.fixable_count += 1;
        }

        if vuln.is_kev {
            summary.kev_count += 1;
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert!(config.enable_trivy);
        assert!(config.enable_grype);
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = ToolOrchestrator::new("/tmp/test");
        let missing = orchestrator.check_tools();
        // Just verify it doesn't panic
        let _ = missing;
    }
}
