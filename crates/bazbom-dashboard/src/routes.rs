//! API route handlers

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use std::sync::Arc;

use crate::{
    models::*,
    AppState,
};

/// Get dashboard summary
pub async fn get_dashboard_summary(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DashboardSummary>, (StatusCode, String)> {
    // TODO: Load actual data from cache
    // For now, return mock data
    Ok(Json(DashboardSummary {
        security_score: 78,
        total_dependencies: 127,
        vulnerabilities: VulnerabilityCounts {
            critical: 1,
            high: 3,
            medium: 5,
            low: 2,
        },
        license_issues: 0,
        policy_violations: 1,
    }))
}

/// Get dependency graph
pub async fn get_dependency_graph(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DependencyGraph>, (StatusCode, String)> {
    // TODO: Load actual dependency graph from SBOM
    // For now, return mock data
    Ok(Json(DependencyGraph {
        nodes: vec![
            DependencyNode {
                id: "log4j-core:2.14.1".to_string(),
                name: "log4j-core".to_string(),
                version: "2.14.1".to_string(),
                severity: Some("CRITICAL".to_string()),
                vuln_count: 1,
            },
            DependencyNode {
                id: "spring-web:5.3.20".to_string(),
                name: "spring-web".to_string(),
                version: "5.3.20".to_string(),
                severity: Some("HIGH".to_string()),
                vuln_count: 1,
            },
        ],
        edges: vec![
            DependencyEdge {
                source: "spring-boot".to_string(),
                target: "spring-web".to_string(),
                relationship: "depends".to_string(),
            },
        ],
    }))
}

/// Get vulnerabilities list
pub async fn get_vulnerabilities(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<VulnerabilitiesList>, (StatusCode, String)> {
    // TODO: Load actual vulnerabilities from findings
    // For now, return mock data
    Ok(Json(VulnerabilitiesList {
        vulnerabilities: vec![
            VulnerabilityDetails {
                cve: "CVE-2021-44228".to_string(),
                package: "log4j-core".to_string(),
                version: "2.14.1".to_string(),
                severity: "CRITICAL".to_string(),
                cvss: 10.0,
                fixed_version: Some("2.21.1".to_string()),
                reachable: Some(true),
                kev: true,
                epss: Some(0.98),
            },
        ],
        total: 1,
    }))
}

/// Get SBOM summary
pub async fn get_sbom(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SbomSummary>, (StatusCode, String)> {
    // TODO: Load actual SBOM
    // For now, return mock data
    Ok(Json(SbomSummary {
        format: "SPDX".to_string(),
        version: "2.3".to_string(),
        tool: "BazBOM".to_string(),
        packages: 127,
        relationships: 254,
    }))
}
