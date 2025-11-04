//! API route handlers

use anyhow::Context;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{
    models::*,
    AppState,
};

/// Get dashboard summary
pub async fn get_dashboard_summary(
    State(state): State<Arc<AppState>>,
) -> Result<Json<DashboardSummary>, (StatusCode, String)> {
    // Try to load actual data from cache, fallback to mock data
    match load_dashboard_summary(&state).await {
        Ok(summary) => Ok(Json(summary)),
        Err(_) => {
            // Return mock data as fallback
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
    }
}

/// Find findings file in cache or project root
fn find_findings_file(state: &AppState) -> anyhow::Result<PathBuf> {
    let findings_path = state.cache_dir.join("sca_findings.json");
    if findings_path.exists() {
        return Ok(findings_path);
    }
    
    // Try alternate location
    let alt_path = state.project_root.join("sca_findings.json");
    if alt_path.exists() {
        return Ok(alt_path);
    }
    
    anyhow::bail!("No findings file found. Please run 'bazbom scan' first.")
}

/// Load dashboard summary from findings file
async fn load_dashboard_summary(state: &AppState) -> anyhow::Result<DashboardSummary> {
    use std::fs;
    use serde_json::Value;
    
    let path_to_use = find_findings_file(state)?;
    let content = fs::read_to_string(&path_to_use)
        .with_context(|| format!("Failed to read findings file: {:?}", path_to_use))?;
    let findings: Value = serde_json::from_str(&content)?;
    
    // Count vulnerabilities by severity
    let mut critical = 0;
    let mut high = 0;
    let mut medium = 0;
    let mut low = 0;
    
    if let Some(vulns) = findings["vulnerabilities"].as_array() {
        for vuln in vulns {
            match vuln["severity"].as_str() {
                Some("CRITICAL") => critical += 1,
                Some("HIGH") => high += 1,
                Some("MEDIUM") => medium += 1,
                Some("LOW") => low += 1,
                _ => {}
            }
        }
    }
    
    let total_deps = findings["summary"]["total_dependencies"]
        .as_u64()
        .unwrap_or(0) as usize;
    
    // Calculate security score (simple formula: 100 - weighted vulns)
    let score = if total_deps > 0 {
        let weight = (critical * 10 + high * 5 + medium * 2 + low) as f32;
        let ratio = weight / total_deps as f32;
        ((100.0 - ratio * 20.0).max(0.0).min(100.0)) as u8
    } else {
        100
    };
    
    Ok(DashboardSummary {
        security_score: score,
        total_dependencies: total_deps,
        vulnerabilities: VulnerabilityCounts {
            critical,
            high,
            medium,
            low,
        },
        license_issues: findings["summary"]["license_issues"]
            .as_u64()
            .unwrap_or(0) as usize,
        policy_violations: findings["summary"]["policy_violations"]
            .as_u64()
            .unwrap_or(0) as usize,
    })
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
    State(state): State<Arc<AppState>>,
) -> Result<Json<VulnerabilitiesList>, (StatusCode, String)> {
    // Try to load actual vulnerabilities from findings
    match load_vulnerabilities(&state).await {
        Ok(list) => Ok(Json(list)),
        Err(_) => {
            // Return mock data as fallback
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
    }
}

/// Load vulnerabilities from findings file
async fn load_vulnerabilities(state: &AppState) -> anyhow::Result<VulnerabilitiesList> {
    use std::fs;
    use serde_json::Value;
    
    let path_to_use = find_findings_file(state)?;
    let content = fs::read_to_string(&path_to_use)
        .with_context(|| format!("Failed to read findings file: {:?}", path_to_use))?;
    let findings: Value = serde_json::from_str(&content)?;
    
    let mut vulnerabilities = Vec::new();
    
    if let Some(vulns) = findings["vulnerabilities"].as_array() {
        for vuln in vulns {
            vulnerabilities.push(VulnerabilityDetails {
                cve: vuln["cve"].as_str().unwrap_or("UNKNOWN").to_string(),
                package: vuln["package"]["name"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                version: vuln["package"]["version"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string(),
                severity: vuln["severity"].as_str().unwrap_or("UNKNOWN").to_string(),
                cvss: vuln["cvss"].as_f64().unwrap_or(0.0) as f32,
                fixed_version: vuln["fixed_version"]
                    .as_str()
                    .map(|s| s.to_string()),
                reachable: vuln["reachable"].as_bool(),
                kev: vuln["kev"].as_bool().unwrap_or(false),
                epss: vuln["epss"].as_f64().map(|e| e as f32),
            });
        }
    }
    
    let total = vulnerabilities.len();
    
    Ok(VulnerabilitiesList {
        vulnerabilities,
        total,
    })
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
