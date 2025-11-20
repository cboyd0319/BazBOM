//! Report generation for BazBOM
//!
//! This crate provides functionality for generating various types of reports:
//! - Executive summaries (1-page PDF)
//! - Compliance reports (PCI-DSS, HIPAA, etc.)
//! - Developer reports (detailed vulnerability information)
//! - Trend reports (historical analysis)

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod compliance;
pub mod developer;
pub mod executive;
pub mod pdf;
pub mod trend;

/// Types of reports that can be generated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportType {
    /// Executive summary (1-page)
    Executive,
    /// Compliance report for specific framework
    Compliance(ComplianceFramework),
    /// Developer-focused detailed report
    Developer,
    /// Historical trend analysis
    Trend,
}

/// Compliance frameworks supported for reports
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceFramework {
    PciDss,
    Hipaa,
    FedRampModerate,
    Soc2,
    Gdpr,
    Iso27001,
    NistCsf,
}

impl ComplianceFramework {
    pub fn name(&self) -> &'static str {
        match self {
            ComplianceFramework::PciDss => "PCI-DSS v4.0",
            ComplianceFramework::Hipaa => "HIPAA Security Rule",
            ComplianceFramework::FedRampModerate => "FedRAMP Moderate",
            ComplianceFramework::Soc2 => "SOC 2 Type II",
            ComplianceFramework::Gdpr => "GDPR",
            ComplianceFramework::Iso27001 => "ISO 27001",
            ComplianceFramework::NistCsf => "NIST Cybersecurity Framework",
        }
    }
}

/// SBOM data for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomData {
    pub project_name: String,
    pub project_version: String,
    pub scan_timestamp: DateTime<Utc>,
    pub total_dependencies: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
}

/// Vulnerability findings for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityFindings {
    pub critical: Vec<VulnerabilityDetail>,
    pub high: Vec<VulnerabilityDetail>,
    pub medium: Vec<VulnerabilityDetail>,
    pub low: Vec<VulnerabilityDetail>,
}

impl VulnerabilityFindings {
    pub fn total_count(&self) -> usize {
        self.critical.len() + self.high.len() + self.medium.len() + self.low.len()
    }

    pub fn security_score(&self) -> u32 {
        // Calculate security score (0-100)
        // Perfect score is 100, deductions for each vulnerability
        let base_score: u32 = 100;
        let critical_deduction = self.critical.len() * 20;
        let high_deduction = self.high.len() * 10;
        let medium_deduction = self.medium.len() * 5;
        let low_deduction = self.low.len() * 2;

        let total_deduction =
            critical_deduction + high_deduction + medium_deduction + low_deduction;
        base_score.saturating_sub(total_deduction as u32)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityDetail {
    pub cve: String,
    pub package_name: String,
    pub package_version: String,
    pub severity: String,
    pub cvss_score: f64,
    pub description: String,
    pub fixed_version: Option<String>,
    pub is_reachable: bool,
    pub is_kev: bool,
    pub epss_score: Option<f64>,
    /// Call chain from entrypoint to vulnerable function (if reachable)
    #[serde(default)]
    pub call_chain: Option<Vec<String>>,
}

/// Policy status for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStatus {
    pub policy_violations: usize,
    pub license_issues: usize,
    pub blocked_packages: usize,
}

/// Reachability analysis summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReachabilitySummary {
    pub total_analyzed: usize,
    pub reachable_count: usize,
    pub unreachable_count: usize,
    pub noise_reduction_percent: f64,
}

/// Container compliance results
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainerCompliance {
    pub pci_dss_pass: bool,
    pub hipaa_pass: bool,
    pub soc2_pass: bool,
    pub pci_issues: Vec<String>,
    pub hipaa_issues: Vec<String>,
    pub soc2_issues: Vec<String>,
}

/// Main report generator
pub struct ReportGenerator {
    sbom: SbomData,
    vulnerabilities: VulnerabilityFindings,
    policy: PolicyStatus,
    /// Optional reachability summary (for container scans)
    reachability: Option<ReachabilitySummary>,
    /// Optional compliance details (for container scans)
    compliance: Option<ContainerCompliance>,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new(
        sbom: SbomData,
        vulnerabilities: VulnerabilityFindings,
        policy: PolicyStatus,
    ) -> Self {
        Self {
            sbom,
            vulnerabilities,
            policy,
            reachability: None,
            compliance: None,
        }
    }

    /// Create a report generator with container-specific data
    pub fn with_container_data(
        sbom: SbomData,
        vulnerabilities: VulnerabilityFindings,
        policy: PolicyStatus,
        reachability: ReachabilitySummary,
        compliance: ContainerCompliance,
    ) -> Self {
        Self {
            sbom,
            vulnerabilities,
            policy,
            reachability: Some(reachability),
            compliance: Some(compliance),
        }
    }

    /// Generate a report of the specified type
    pub fn generate(&self, report_type: ReportType, output_path: &Path) -> Result<()> {
        match report_type {
            ReportType::Executive => self.generate_executive(output_path),
            ReportType::Compliance(framework) => self.generate_compliance(framework, output_path),
            ReportType::Developer => self.generate_developer(output_path),
            ReportType::Trend => self.generate_trend(output_path),
        }
    }

    /// Generate executive summary report
    fn generate_executive(&self, output_path: &Path) -> Result<()> {
        executive::generate_executive_report(self, output_path)
    }

    /// Generate compliance report
    fn generate_compliance(
        &self,
        framework: ComplianceFramework,
        output_path: &Path,
    ) -> Result<()> {
        compliance::generate_compliance_report(self, framework, output_path)
    }

    /// Generate developer report
    fn generate_developer(&self, output_path: &Path) -> Result<()> {
        developer::generate_developer_report(self, output_path)
    }

    /// Generate trend report
    fn generate_trend(&self, output_path: &Path) -> Result<()> {
        trend::generate_trend_report(self, output_path)
    }

    /// Get SBOM data
    pub fn sbom(&self) -> &SbomData {
        &self.sbom
    }

    /// Get vulnerability findings
    pub fn vulnerabilities(&self) -> &VulnerabilityFindings {
        &self.vulnerabilities
    }

    /// Get policy status
    pub fn policy(&self) -> &PolicyStatus {
        &self.policy
    }

    /// Get reachability summary (for container scans)
    pub fn reachability(&self) -> Option<&ReachabilitySummary> {
        self.reachability.as_ref()
    }

    /// Get compliance details (for container scans)
    pub fn compliance(&self) -> Option<&ContainerCompliance> {
        self.compliance.as_ref()
    }
}

/// Helper function to write HTML content to file
pub(crate) fn write_html_file(path: &Path, content: &str) -> Result<()> {
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_sbom() -> SbomData {
        SbomData {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            scan_timestamp: Utc::now(),
            total_dependencies: 100,
            direct_dependencies: 20,
            transitive_dependencies: 80,
        }
    }

    fn create_test_vulnerabilities() -> VulnerabilityFindings {
        VulnerabilityFindings {
            critical: vec![VulnerabilityDetail {
                cve: "CVE-2021-44228".to_string(),
                package_name: "log4j-core".to_string(),
                package_version: "2.14.1".to_string(),
                severity: "CRITICAL".to_string(),
                cvss_score: 10.0,
                description: "Log4Shell RCE vulnerability".to_string(),
                fixed_version: Some("2.21.1".to_string()),
                is_reachable: true,
                is_kev: true,
                epss_score: Some(0.975),
                call_chain: Some(vec!["main()".to_string(), "Logger.error()".to_string()]),
            }],
            high: vec![],
            medium: vec![],
            low: vec![],
        }
    }

    fn create_test_policy() -> PolicyStatus {
        PolicyStatus {
            policy_violations: 1,
            license_issues: 0,
            blocked_packages: 0,
        }
    }

    #[test]
    fn test_security_score_calculation() {
        let vuln = create_test_vulnerabilities();
        // 1 critical = -20 points
        assert_eq!(vuln.security_score(), 80);
    }

    #[test]
    fn test_report_generator_creation() {
        let sbom = create_test_sbom();
        let vuln = create_test_vulnerabilities();
        let policy = create_test_policy();

        let generator = ReportGenerator::new(sbom, vuln, policy);
        assert_eq!(generator.sbom().project_name, "test-project");
        assert_eq!(generator.vulnerabilities().total_count(), 1);
    }

    #[test]
    fn test_compliance_framework_names() {
        assert_eq!(ComplianceFramework::PciDss.name(), "PCI-DSS v4.0");
        assert_eq!(ComplianceFramework::Hipaa.name(), "HIPAA Security Rule");
    }
}
