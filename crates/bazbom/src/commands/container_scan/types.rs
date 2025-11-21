//! Types and data structures for container scanning

use bazbom_depsdev::System;
use bazbom_upgrade_analyzer::detect_ecosystem_from_package;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Container scan options
#[derive(Debug, Clone)]
pub struct ContainerScanOptions {
    pub image_name: String,
    pub output_dir: PathBuf,
    pub format: String,
    pub baseline: bool,
    pub compare_baseline: bool,
    pub compare_image: Option<String>,
    pub create_issues_repo: Option<String>,
    pub interactive: bool,
    pub report_file: Option<String>,
    pub filter: Option<String>,
    pub with_reachability: bool,
}

/// Layer information with vulnerability attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub digest: String,
    pub size_mb: f64,
    pub packages: Vec<String>,
    pub vulnerabilities: Vec<VulnerabilityInfo>,
}

/// Vulnerability with full context and enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    pub cve_id: String,
    pub package_name: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub layer_digest: String,
    pub published_date: Option<String>,
    pub epss_score: Option<f64>,
    pub epss_percentile: Option<f64>,
    pub is_kev: bool,
    pub kev_due_date: Option<String>,
    pub cvss_score: Option<f64>,
    pub priority: Option<String>,
    pub references: Vec<String>,
    pub breaking_change: Option<bool>,
    pub upgrade_path: Option<String>,
    pub is_reachable: bool,
    pub difficulty_score: Option<u8>,
    /// Call chain from entrypoint to vulnerable function (if reachable)
    #[serde(default)]
    pub call_chain: Option<Vec<String>>,
    /// Dependency path showing how this package was introduced (for transitive deps)
    /// e.g., ["app", "spring-boot-starter", "jackson-databind"]
    #[serde(default)]
    pub dependency_path: Option<Vec<String>>,
}

impl VulnerabilityInfo {
    /// Check if vulnerability has a known fix
    #[allow(dead_code)]
    pub fn is_fixable(&self) -> bool {
        self.fixed_version.is_some()
    }
}

/// Complete container scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerScanResults {
    pub image_name: String,
    pub total_packages: usize,
    pub total_vulnerabilities: usize,
    pub layers: Vec<LayerInfo>,
    pub base_image: Option<String>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    /// Upgrade recommendations for vulnerable OS packages
    #[serde(default)]
    pub upgrade_recommendations: Vec<UpgradeRecommendation>,
    /// Reachability analysis summary
    #[serde(default)]
    pub reachability_summary: Option<ReachabilitySummary>,
    /// Compliance check results
    #[serde(default)]
    pub compliance_results: Option<ComplianceResults>,
}

/// Reachability analysis summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReachabilitySummary {
    pub total_analyzed: usize,
    pub reachable_count: usize,
    pub unreachable_count: usize,
    pub noise_reduction_percent: f64,
}

/// Compliance check results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceResults {
    pub pci_dss: ComplianceStatus,
    pub hipaa: ComplianceStatus,
    pub soc2: ComplianceStatus,
}

/// Status for a compliance framework
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceStatus {
    pub status: String,
    pub issues: Vec<String>,
}

/// Upgrade recommendation for an OS package (enhanced with UpgradeAnalyzer data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeRecommendation {
    pub package: String,
    pub installed_version: String,
    pub recommended_version: Option<String>,
    pub fixes_cves: Vec<String>,
    pub risk_level: String,
    // Enhanced fields from UpgradeAnalyzer
    pub effort_hours: Option<f32>,
    pub breaking_changes_count: Option<usize>,
    pub transitive_upgrades_count: Option<usize>,
    pub migration_guide_url: Option<String>,
    pub success_rate: Option<f32>,
    pub github_repo: Option<String>,
}

/// Quick win - easy fix with high impact
#[derive(Debug, Clone)]
pub(crate) struct QuickWin {
    pub package: String,
    pub current_version: String,
    pub fixed_version: String,
    pub vulns_fixed: Vec<String>,
    pub severity: String,
    pub estimated_minutes: u32,
}

/// Container signature verification status
#[derive(Debug, Clone)]
pub(crate) enum SignatureStatus {
    Verified,
    NotSigned,
    ToolNotAvailable,
    Invalid(String),
}

/// SLSA provenance verification status
#[derive(Debug, Clone)]
pub(crate) enum ProvenanceStatus {
    Verified,
    NotAvailable,
    ToolNotAvailable,
    Invalid(String),
}

/// Action item for prioritized plan
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ActionItem {
    pub priority: String,
    pub cve_id: String,
    pub cves_fixed: Vec<String>,
    pub package: String,
    pub fixed_version: String,
    pub description: String,
    pub estimated_hours: f32,
    pub breaking: bool,
    pub kev: bool,
    pub epss: f64,
}

/// Package ecosystem for language-specific remediation
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum PackageEcosystem {
    Java,
    Python,
    JavaScript,
    Go,
    Rust,
    Ruby,
    Php,
    Other,
}

/// Docker layer metadata
#[derive(Debug, Clone)]
pub(crate) struct DockerLayerMetadata {
    pub digest: String,
    pub size_bytes: u64,
    pub command: String,
}

/// Detect package ecosystem from package name and patterns
///
/// Uses the shared detection from bazbom-upgrade-analyzer.
pub(crate) fn detect_ecosystem(package_name: &str) -> PackageEcosystem {
    // Use shared ecosystem detection from bazbom-upgrade-analyzer
    match detect_ecosystem_from_package(package_name) {
        System::Maven => PackageEcosystem::Java,
        System::Npm => PackageEcosystem::JavaScript,
        System::PyPI => PackageEcosystem::Python,
        System::Go => PackageEcosystem::Go,
        System::Cargo => PackageEcosystem::Rust,
        System::RubyGems => PackageEcosystem::Ruby,
        System::Packagist => PackageEcosystem::Php,
        System::Hex => PackageEcosystem::Other,    // Elixir
        System::Pub => PackageEcosystem::Other,    // Dart
        System::NuGet => PackageEcosystem::Other,  // .NET
        // OS packages map to Other for now
        System::Alpine | System::Debian | System::Rpm => PackageEcosystem::Other,
    }
}
