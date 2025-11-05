//! Data models for the dashboard API

use serde::{Deserialize, Serialize};

/// Dashboard summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub security_score: u8,
    pub total_dependencies: usize,
    pub vulnerabilities: VulnerabilityCounts,
    pub license_issues: usize,
    pub policy_violations: usize,
}

/// Vulnerability counts by severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityCounts {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Dependency graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub name: String,
    pub version: String,
    pub severity: Option<String>,
    pub vuln_count: usize,
}

/// Dependency graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub source: String,
    pub target: String,
    pub relationship: String,
}

/// Dependency graph response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
}

/// Vulnerability details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityDetails {
    pub cve: String,
    pub package: String,
    pub version: String,
    pub severity: String,
    pub cvss: f32,
    pub fixed_version: Option<String>,
    pub reachable: Option<bool>,
    pub kev: bool,
    pub epss: Option<f32>,
}

/// Vulnerabilities list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilitiesList {
    pub vulnerabilities: Vec<VulnerabilityDetails>,
    pub total: usize,
}

/// SBOM summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomSummary {
    pub format: String,
    pub version: String,
    pub tool: String,
    pub packages: usize,
    pub relationships: usize,
}

/// Team dashboard summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDashboard {
    pub team_name: String,
    pub total_members: usize,
    pub open_vulnerabilities: VulnerabilityCounts,
    pub assignments: Vec<TeamAssignment>,
    pub unassigned_count: usize,
    pub metrics: TeamMetrics,
}

/// Team assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamAssignment {
    pub assignee: String,
    pub vulnerability_count: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Team metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMetrics {
    pub mean_time_to_fix_days: f32,
    pub vulnerabilities_fixed: usize,
    pub vulnerabilities_introduced: usize,
    pub net_improvement: i32,
    pub top_contributors: Vec<TopContributor>,
}

/// Top contributor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopContributor {
    pub email: String,
    pub fixes_count: usize,
}
