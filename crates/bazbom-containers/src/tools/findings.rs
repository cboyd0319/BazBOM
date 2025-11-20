//! Unified finding types for container security scanning
//!
//! These types normalize output from different scanning tools into
//! a common format for aggregation and reporting.

use serde::{Deserialize, Serialize};

/// Severity levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    Unknown,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    /// Parse severity from string (case-insensitive)
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CRITICAL" => Severity::Critical,
            "HIGH" => Severity::High,
            "MEDIUM" | "MODERATE" => Severity::Medium,
            "LOW" => Severity::Low,
            _ => Severity::Unknown,
        }
    }

    /// Get numeric value for sorting (higher = more severe)
    pub fn numeric_value(&self) -> u8 {
        match self {
            Severity::Critical => 4,
            Severity::High => 3,
            Severity::Medium => 2,
            Severity::Low => 1,
            Severity::Unknown => 0,
        }
    }
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

/// A vulnerability finding from a scanner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityFinding {
    /// CVE identifier (e.g., "CVE-2021-44228")
    pub cve_id: String,

    /// Affected package name
    pub package_name: String,

    /// Installed version
    pub installed_version: String,

    /// Fixed version (if available)
    pub fixed_version: Option<String>,

    /// Severity level
    pub severity: Severity,

    /// CVSS score (0.0 - 10.0)
    pub cvss_score: Option<f64>,

    /// Short title/summary
    pub title: String,

    /// Full description
    pub description: String,

    /// Layer digest where this package was introduced
    pub layer_digest: Option<String>,

    /// Source scanner that found this
    pub source: String,

    /// Reference URLs
    pub references: Vec<String>,

    // Enrichment fields (populated later)
    /// EPSS probability score (0.0 - 1.0)
    pub epss_score: Option<f64>,

    /// EPSS percentile (0.0 - 100.0)
    pub epss_percentile: Option<f64>,

    /// CISA Known Exploited Vulnerability
    pub is_kev: bool,

    /// KEV due date for remediation
    pub kev_due_date: Option<String>,
}

impl VulnerabilityFinding {
    /// Check if this vulnerability has a fix available
    pub fn is_fixable(&self) -> bool {
        self.fixed_version.is_some()
    }
}

/// A secret detected in the image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretFinding {
    /// Type of secret (e.g., "AWS Access Key", "Private Key")
    pub secret_type: String,

    /// Severity level
    pub severity: Severity,

    /// File path where secret was found
    pub file_path: String,

    /// Line number (if available)
    pub line_number: Option<u32>,

    /// Matched rule/pattern name
    pub rule_id: String,

    /// Description of the finding
    pub description: String,

    /// Source scanner
    pub source: String,

    /// Layer digest where this was found
    pub layer_digest: Option<String>,
}

/// A misconfiguration finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MisconfigFinding {
    /// Unique identifier for the misconfiguration
    pub id: String,

    /// Type/category (e.g., "Dockerfile", "Kubernetes")
    pub misconfig_type: String,

    /// Severity level
    pub severity: Severity,

    /// Short title
    pub title: String,

    /// Full description
    pub description: String,

    /// Remediation advice
    pub resolution: String,

    /// File path
    pub file_path: Option<String>,

    /// Source scanner
    pub source: String,
}

/// A benchmark/compliance check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Check ID (e.g., "CIS-DI-0001")
    pub check_id: String,

    /// Compliance level (INFO, WARN, FATAL)
    pub level: String,

    /// Short title
    pub title: String,

    /// Full description
    pub description: String,

    /// Whether the check passed
    pub passed: bool,

    /// Source scanner (e.g., "dockle")
    pub source: String,
}

/// Image efficiency metrics from Dive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// Overall efficiency score (0.0 - 1.0)
    pub efficiency_score: f64,

    /// Total image size in bytes
    pub image_size: u64,

    /// Wasted space in bytes
    pub wasted_bytes: u64,

    /// Number of layers
    pub layer_count: usize,

    /// Per-layer details
    pub layers: Vec<LayerEfficiency>,
}

/// Efficiency metrics for a single layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerEfficiency {
    /// Layer digest
    pub digest: String,

    /// Layer size in bytes
    pub size: u64,

    /// Command that created this layer
    pub command: Option<String>,

    /// Wasted space in this layer
    pub wasted_bytes: u64,
}

/// Package information from SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,

    /// Version
    pub version: String,

    /// Package URL (PURL)
    pub purl: Option<String>,

    /// Package type (e.g., "deb", "apk", "npm", "maven")
    pub pkg_type: String,

    /// Licenses
    pub licenses: Vec<String>,

    /// Layer where this package was installed
    pub layer_digest: Option<String>,

    /// File locations
    pub locations: Vec<String>,
}

/// Aggregated results from all tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedResults {
    /// Image that was scanned
    pub image_name: String,

    /// All vulnerabilities (deduplicated)
    pub vulnerabilities: Vec<VulnerabilityFinding>,

    /// All secrets
    pub secrets: Vec<SecretFinding>,

    /// All misconfigurations
    pub misconfigs: Vec<MisconfigFinding>,

    /// Benchmark results
    pub benchmarks: Vec<BenchmarkResult>,

    /// Efficiency metrics
    pub efficiency: Option<EfficiencyMetrics>,

    /// All packages in the image
    pub packages: Vec<PackageInfo>,

    /// Summary counts
    pub summary: ScanSummary,
}

/// Summary statistics for a scan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanSummary {
    pub total_packages: usize,
    pub total_vulnerabilities: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub fixable_count: usize,
    pub kev_count: usize,
    pub secrets_count: usize,
    pub misconfigs_count: usize,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical > Severity::High);
        assert!(Severity::High > Severity::Medium);
        assert!(Severity::Medium > Severity::Low);
        assert!(Severity::Low > Severity::Unknown);
    }

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from_str_loose("CRITICAL"), Severity::Critical);
        assert_eq!(Severity::from_str_loose("critical"), Severity::Critical);
        assert_eq!(Severity::from_str_loose("moderate"), Severity::Medium);
        assert_eq!(Severity::from_str_loose("garbage"), Severity::Unknown);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::Unknown), "UNKNOWN");
    }
}
