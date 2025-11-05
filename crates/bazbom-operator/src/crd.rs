//! BazBOMScan Custom Resource Definition
//!
//! Defines the Kubernetes custom resource for BazBOM scans

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// BazBOMScan custom resource
///
/// Represents a BazBOM scan configuration for a Kubernetes workload
#[derive(CustomResource, Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[kube(
    group = "bazbom.io",
    version = "v1",
    kind = "BazBOMScan",
    plural = "bazbomscans",
    shortname = "bbs",
    namespaced,
    status = "BazBOMScanStatus",
    printcolumn = r#"{"name":"Target", "type":"string", "jsonPath":".spec.targetDeployment"}"#,
    printcolumn = r#"{"name":"Schedule", "type":"string", "jsonPath":".spec.schedule"}"#,
    printcolumn = r#"{"name":"Last Scan", "type":"date", "jsonPath":".status.lastScanTime"}"#,
    printcolumn = r#"{"name":"Critical", "type":"integer", "jsonPath":".status.vulnerabilities.critical"}"#,
    printcolumn = r#"{"name":"High", "type":"integer", "jsonPath":".status.vulnerabilities.high"}"#
)]
#[serde(rename_all = "camelCase")]
pub struct BazBOMScanSpec {
    /// Target deployment to scan
    pub target_deployment: String,

    /// Cron schedule for scans (e.g., "0 0 * * *" for daily at midnight)
    #[serde(default)]
    pub schedule: Option<String>,

    /// Policy file ConfigMap name
    #[serde(default)]
    pub policy_config_map: Option<String>,

    /// Build system type (maven, gradle, bazel)
    #[serde(default)]
    pub build_system: Option<String>,

    /// Additional scan options
    #[serde(default)]
    pub scan_options: ScanOptions,

    /// Output format (spdx, cyclonedx, json)
    #[serde(default = "default_output_format")]
    pub output_format: String,

    /// Whether to store SBOM in ConfigMap
    #[serde(default = "default_true")]
    pub store_sbom: bool,

    /// Whether to create GitHub issues for vulnerabilities
    #[serde(default)]
    pub create_github_issues: bool,

    /// GitHub repository for issue creation (owner/repo)
    #[serde(default)]
    pub github_repository: Option<String>,
}

fn default_output_format() -> String {
    "spdx".to_string()
}

fn default_true() -> bool {
    true
}

/// Scan options
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanOptions {
    /// Scan container images
    #[serde(default)]
    pub scan_containers: bool,

    /// Include reachability analysis
    #[serde(default)]
    pub reachability_analysis: bool,

    /// ML-powered prioritization
    #[serde(default)]
    pub ml_prioritize: bool,

    /// LLM-powered fix suggestions
    #[serde(default)]
    pub llm_fixes: bool,
}

/// Status of a BazBOMScan
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct BazBOMScanStatus {
    /// Last scan timestamp (RFC3339 format)
    pub last_scan_time: Option<String>,

    /// Scan phase (Pending, Running, Complete, Failed)
    #[serde(default)]
    pub phase: String,

    /// Vulnerability counts
    #[serde(default)]
    pub vulnerabilities: VulnerabilityCounts,

    /// Total dependencies found
    #[serde(default)]
    pub total_dependencies: usize,

    /// SBOM ConfigMap name (if stored)
    pub sbom_config_map: Option<String>,

    /// URL to SBOM file
    pub sbom_url: Option<String>,

    /// Error message if scan failed
    pub error_message: Option<String>,

    /// Security score (0-100)
    #[serde(default)]
    pub security_score: u8,
}

/// Vulnerability counts by severity
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema, Default)]
pub struct VulnerabilityCounts {
    #[serde(default)]
    pub critical: usize,
    #[serde(default)]
    pub high: usize,
    #[serde(default)]
    pub medium: usize,
    #[serde(default)]
    pub low: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crd_creation() {
        let spec = BazBOMScanSpec {
            target_deployment: "my-app".to_string(),
            schedule: Some("0 0 * * *".to_string()),
            policy_config_map: None,
            build_system: Some("maven".to_string()),
            scan_options: ScanOptions::default(),
            output_format: "spdx".to_string(),
            store_sbom: true,
            create_github_issues: false,
            github_repository: None,
        };

        assert_eq!(spec.target_deployment, "my-app");
        assert_eq!(spec.schedule, Some("0 0 * * *".to_string()));
        assert_eq!(spec.output_format, "spdx");
    }

    #[test]
    fn test_status_default() {
        let status = BazBOMScanStatus::default();
        assert_eq!(status.phase, "");
        assert_eq!(status.vulnerabilities.critical, 0);
        assert_eq!(status.security_score, 0);
    }

    #[test]
    fn test_scan_options_default() {
        let opts = ScanOptions::default();
        assert!(!opts.scan_containers);
        assert!(!opts.reachability_analysis);
        assert!(!opts.ml_prioritize);
        assert!(!opts.llm_fixes);
    }
}
