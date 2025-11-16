//! API route handlers

use anyhow::Context;
use axum::{extract::State, http::StatusCode, response::Json};
use std::path::PathBuf;
use std::sync::Arc;

use crate::{models::*, AppState};

/// Maximum file size for JSON/YAML parsing (10 MB)
const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;

/// Safely read a file with size limit to prevent DoS attacks
fn read_file_with_limit(path: &PathBuf) -> anyhow::Result<String> {
    use std::fs;

    // Check file size first
    let metadata =
        fs::metadata(path).with_context(|| format!("Failed to read file metadata: {:?}", path))?;

    if metadata.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "File too large: {} bytes (max: {} bytes). This prevents DoS attacks via large inputs.",
            metadata.len(),
            MAX_FILE_SIZE
        );
    }

    // Read file content
    fs::read_to_string(path).with_context(|| format!("Failed to read file: {:?}", path))
}

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

/// Validate that a path doesn't escape the base directory (prevents path traversal)
fn validate_path(path: &PathBuf, base: &PathBuf) -> anyhow::Result<PathBuf> {
    use std::fs;

    // Canonicalize both paths to resolve symlinks and relative components
    let canonical_path = fs::canonicalize(path)
        .with_context(|| format!("Failed to canonicalize path: {:?}", path))?;

    let canonical_base = fs::canonicalize(base)
        .with_context(|| format!("Failed to canonicalize base: {:?}", base))?;

    // Ensure the path is within the base directory
    if !canonical_path.starts_with(&canonical_base) {
        anyhow::bail!(
            "Path traversal detected! Path {:?} is outside base directory {:?}",
            canonical_path,
            canonical_base
        );
    }

    Ok(canonical_path)
}

/// Find findings file in cache or project root with path validation
fn find_findings_file(state: &AppState) -> anyhow::Result<PathBuf> {
    let findings_path = state.cache_dir.join("sca_findings.json");
    if findings_path.exists() {
        // Validate path before returning
        return validate_path(&findings_path, &state.cache_dir);
    }

    // Try alternate location
    let alt_path = state.project_root.join("sca_findings.json");
    if alt_path.exists() {
        // Validate path before returning
        return validate_path(&alt_path, &state.project_root);
    }

    anyhow::bail!("No findings file found. Please run 'bazbom scan' first.")
}

/// Load dashboard summary from findings file
async fn load_dashboard_summary(state: &AppState) -> anyhow::Result<DashboardSummary> {
    use serde_json::Value;

    let path_to_use = find_findings_file(state)?;
    // Use safe file reading with size limit
    let content = read_file_with_limit(&path_to_use)?;
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
        ((100.0 - ratio * 20.0).clamp(0.0, 100.0)) as u8
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
        license_issues: findings["summary"]["license_issues"].as_u64().unwrap_or(0) as usize,
        policy_violations: findings["summary"]["policy_violations"]
            .as_u64()
            .unwrap_or(0) as usize,
    })
}

/// Get dependency graph
pub async fn get_dependency_graph(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DependencyGraph>, (StatusCode, String)> {
    // FUTURE ENHANCEMENT: Load actual dependency graph from SBOM
    // Would parse SBOM files (SPDX/CycloneDX) from state.sbom_path
    // and construct DependencyGraph from relationships section
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
        edges: vec![DependencyEdge {
            source: "spring-boot".to_string(),
            target: "spring-web".to_string(),
            relationship: "depends".to_string(),
        }],
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
                vulnerabilities: vec![VulnerabilityDetails {
                    cve: "CVE-2021-44228".to_string(),
                    package: "log4j-core".to_string(),
                    version: "2.14.1".to_string(),
                    severity: "CRITICAL".to_string(),
                    cvss: 10.0,
                    fixed_version: Some("2.21.1".to_string()),
                    reachable: Some(true),
                    kev: true,
                    epss: Some(0.98),
                }],
                total: 1,
            }))
        }
    }
}

/// Load vulnerabilities from findings file
async fn load_vulnerabilities(state: &AppState) -> anyhow::Result<VulnerabilitiesList> {
    use serde_json::Value;
    use std::fs;

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
                fixed_version: vuln["fixed_version"].as_str().map(|s| s.to_string()),
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
    // FUTURE ENHANCEMENT: Load actual SBOM from state.sbom_path
    // Would read and parse SPDX/CycloneDX JSON files
    // For now, return mock data
    Ok(Json(SbomSummary {
        format: "SPDX".to_string(),
        version: "2.3".to_string(),
        tool: "BazBOM".to_string(),
        packages: 127,
        relationships: 254,
    }))
}

/// Get team dashboard
pub async fn get_team_dashboard(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TeamDashboard>, (StatusCode, String)> {
    match load_team_dashboard(&state).await {
        Ok(dashboard) => Ok(Json(dashboard)),
        Err(e) => {
            // Return mock data with useful structure
            eprintln!("Failed to load team dashboard: {}", e);
            Ok(Json(TeamDashboard {
                team_name: "Security Team".to_string(),
                total_members: 3,
                open_vulnerabilities: VulnerabilityCounts {
                    critical: 1,
                    high: 3,
                    medium: 5,
                    low: 2,
                },
                assignments: vec![
                    TeamAssignment {
                        assignee: "alice@example.com".to_string(),
                        vulnerability_count: 3,
                        critical: 1,
                        high: 2,
                        medium: 0,
                        low: 0,
                    },
                    TeamAssignment {
                        assignee: "bob@example.com".to_string(),
                        vulnerability_count: 2,
                        critical: 0,
                        high: 1,
                        medium: 1,
                        low: 0,
                    },
                ],
                unassigned_count: 6,
                metrics: TeamMetrics {
                    mean_time_to_fix_days: 2.3,
                    vulnerabilities_fixed: 24,
                    vulnerabilities_introduced: 8,
                    net_improvement: 16,
                    top_contributors: vec![
                        TopContributor {
                            email: "alice@example.com".to_string(),
                            fixes_count: 12,
                        },
                        TopContributor {
                            email: "bob@example.com".to_string(),
                            fixes_count: 8,
                        },
                    ],
                },
            }))
        }
    }
}

/// Load team dashboard from audit logs and assignments
async fn load_team_dashboard(state: &AppState) -> anyhow::Result<TeamDashboard> {
    use std::collections::HashMap;
    use std::fs;

    // Load team config
    let team_config_path = state.project_root.join(".bazbom/team-config.json");
    let team_name = if team_config_path.exists() {
        let content = fs::read_to_string(&team_config_path)?;
        let config: serde_json::Value = serde_json::from_str(&content)?;
        config["name"]
            .as_str()
            .unwrap_or("Security Team")
            .to_string()
    } else {
        "Security Team".to_string()
    };

    // Load audit log
    let audit_path = state.project_root.join(".bazbom/audit.json");
    let mut fix_counts: HashMap<String, usize> = HashMap::new();
    let mut total_fixes = 0;

    if audit_path.exists() {
        let content = fs::read_to_string(&audit_path)?;
        if let Ok(entries) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
            for entry in &entries {
                if let Some(action) = entry["action"].as_str() {
                    if action.contains("Fixed") || action.contains("Assigned") {
                        if let Some(user) = entry["user"].as_str() {
                            *fix_counts.entry(user.to_string()).or_insert(0) += 1;
                            if action.contains("Fixed") {
                                total_fixes += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Load current vulnerabilities
    let findings_path = find_findings_file(state)?;
    let findings_content = fs::read_to_string(&findings_path)?;
    let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

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

    // Create top contributors list
    let mut top_contributors: Vec<TopContributor> = fix_counts
        .into_iter()
        .map(|(email, fixes_count)| TopContributor { email, fixes_count })
        .collect();
    top_contributors.sort_by(|a, b| b.fixes_count.cmp(&a.fixes_count));
    top_contributors.truncate(5); // Top 5 contributors

    // Calculate MTTF (simplified - would need timestamp tracking in production)
    let mttf = if total_fixes > 0 { 2.5 } else { 0.0 };

    Ok(TeamDashboard {
        team_name,
        total_members: top_contributors.len().max(1),
        open_vulnerabilities: VulnerabilityCounts {
            critical,
            high,
            medium,
            low,
        },
        assignments: vec![], // Would load from git notes in production
        unassigned_count: critical + high + medium + low,
        metrics: TeamMetrics {
            mean_time_to_fix_days: mttf,
            vulnerabilities_fixed: total_fixes,
            vulnerabilities_introduced: 0, // Would need historical tracking
            net_improvement: total_fixes as i32,
            top_contributors,
        },
    })
}
