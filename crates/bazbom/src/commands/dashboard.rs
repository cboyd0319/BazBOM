use anyhow::{Context, Result};
use std::path::PathBuf;

/// Handle the `bazbom dashboard` command
pub fn handle_dashboard(port: u16, open: bool, export: Option<String>) -> Result<()> {
    use bazbom_dashboard::{start_dashboard, DashboardConfig};

    if let Some(export_path) = export {
        return export_static_dashboard(&export_path);
    }

    // Create dashboard configuration
    let config = DashboardConfig {
        port,
        cache_dir: PathBuf::from(".bazbom/cache"),
        project_root: PathBuf::from("."),
    };

    // Open browser if requested
    if open {
        let url = format!("http://localhost:{}", port);
        println!("[bazbom] Opening browser at {}", url);
        if let Err(e) = webbrowser::open(&url) {
            eprintln!(
                "[bazbom] warning: failed to open browser automatically: {}",
                e
            );
            eprintln!("[bazbom] Please open {} manually in your browser", url);
        }
    }

    // Start dashboard with tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async { start_dashboard(config).await })
}

fn export_static_dashboard(export_path: &str) -> Result<()> {
    use bazbom_dashboard::export_to_html;

    println!(
        "[bazbom] Exporting static HTML dashboard to: {}",
        export_path
    );

    // Load findings from cache
    let cache_dir = PathBuf::from(".bazbom/cache");
    let findings_path = cache_dir.join("sca_findings.json");

    let (summary, graph_data, vulnerabilities) = if findings_path.exists() {
        load_findings_data(&findings_path)?
    } else {
        println!("[bazbom] No findings file found, generating empty report");
        create_empty_dashboard_data()
    };

    // Export to HTML
    export_to_html(
        &PathBuf::from(export_path),
        &summary,
        &graph_data,
        &vulnerabilities,
    )?;

    println!("[bazbom] Successfully exported to: {}", export_path);
    println!("[bazbom] Open the file in your browser to view the report");
    Ok(())
}

fn load_findings_data(
    findings_path: &PathBuf,
) -> Result<(
    bazbom_dashboard::DashboardSummary,
    bazbom_dashboard::DependencyGraph,
    Vec<bazbom_dashboard::Vulnerability>,
)> {
    use bazbom_dashboard::{DashboardSummary, DependencyGraph, Vulnerability, VulnerabilityCounts};
    use std::fs;

    let findings_content =
        fs::read_to_string(findings_path).context("Failed to read findings file")?;
    let findings: serde_json::Value =
        serde_json::from_str(&findings_content).context("Failed to parse findings JSON")?;

    // Extract summary
    let summary = DashboardSummary {
        security_score: findings["summary"]["security_score"].as_u64().unwrap_or(0) as u8,
        total_dependencies: findings["summary"]["total_dependencies"]
            .as_u64()
            .unwrap_or(0) as usize,
        vulnerabilities: VulnerabilityCounts {
            critical: findings["summary"]["vulnerabilities"]["critical"]
                .as_u64()
                .unwrap_or(0) as usize,
            high: findings["summary"]["vulnerabilities"]["high"]
                .as_u64()
                .unwrap_or(0) as usize,
            medium: findings["summary"]["vulnerabilities"]["medium"]
                .as_u64()
                .unwrap_or(0) as usize,
            low: findings["summary"]["vulnerabilities"]["low"]
                .as_u64()
                .unwrap_or(0) as usize,
        },
        license_issues: 0,
        policy_violations: 0,
    };

    // Extract vulnerabilities
    let vulns: Vec<_> = findings["vulnerabilities"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .map(|v| Vulnerability {
            cve: v["cve"].as_str().unwrap_or("").to_string(),
            package_name: v["package"]["name"].as_str().unwrap_or("").to_string(),
            package_version: v["package"]["version"].as_str().unwrap_or("").to_string(),
            severity: v["severity"].as_str().unwrap_or("").to_string(),
            cvss: v["cvss"].as_f64().unwrap_or(0.0) as f32,
            description: v["description"].as_str().map(|s| s.to_string()),
            fixed_version: v["fixed_version"].as_str().map(|s| s.to_string()),
        })
        .collect();

    let graph_data = DependencyGraph {
        nodes: vec![],
        edges: vec![],
    };

    Ok((summary, graph_data, vulns))
}

fn create_empty_dashboard_data() -> (
    bazbom_dashboard::DashboardSummary,
    bazbom_dashboard::DependencyGraph,
    Vec<bazbom_dashboard::Vulnerability>,
) {
    use bazbom_dashboard::{DashboardSummary, DependencyGraph, VulnerabilityCounts};

    let summary = DashboardSummary {
        security_score: 100,
        total_dependencies: 0,
        vulnerabilities: VulnerabilityCounts {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        },
        license_issues: 0,
        policy_violations: 0,
    };
    (
        summary,
        DependencyGraph {
            nodes: vec![],
            edges: vec![],
        },
        vec![],
    )
}
