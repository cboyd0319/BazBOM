use anyhow::Result;
use crate::commands::upgrade_intelligence;
use bazbom::interactive_fix::{InteractiveFix, FixableVulnerability, Severity};

/// Handle the `bazbom fix` command
///
/// This is a placeholder - the full implementation will be extracted from main.rs
/// in a subsequent refactoring pass to keep this module under 500 lines.
#[allow(clippy::too_many_arguments)]
pub async fn handle_fix(
    package: Option<String>,
    suggest: bool,
    apply: bool,
    pr: bool,
    interactive: bool,
    explain: bool,
    ml_prioritize: bool,
    llm: bool,
    llm_provider: String,
    llm_model: Option<String>,
) -> Result<()> {
    // If --explain flag is set and a package is provided, show upgrade intelligence
    if explain {
        if let Some(pkg) = package {
            return upgrade_intelligence::explain_upgrade(&pkg).await;
        } else {
            anyhow::bail!(
                "The --explain flag requires a package name.\n\
                 Example: bazbom fix org.apache.logging.log4j:log4j-core --explain"
            );
        }
    }

    // If --interactive flag is set, start interactive fix mode
    if interactive {
        return run_interactive_mode().await;
    }

    // Otherwise, delegate to the original implementation
    bazbom::remediation::handle_fix_command(
        suggest,
        apply,
        pr,
        false, // interactive handled above
        ml_prioritize,
        llm,
        llm_provider,
        llm_model,
    )
}

/// Run interactive fix mode with beautiful TUI
async fn run_interactive_mode() -> Result<()> {
    // TODO: Load actual vulnerabilities from scan results
    // For now, use demo data
    let vulnerabilities = load_vulnerabilities_from_scan()?;

    if vulnerabilities.is_empty() {
        println!("✨ No vulnerabilities found! Your project is clean.");
        return Ok(());
    }

    let mut session = InteractiveFix::new(vulnerabilities);
    session.run().await
}

/// Load vulnerabilities from scan results
fn load_vulnerabilities_from_scan() -> Result<Vec<FixableVulnerability>> {
    use std::fs;
    use std::path::PathBuf;
    use serde_json::Value;

    // Try multiple locations for findings file
    let possible_paths = vec![
        PathBuf::from("./bazbom-findings/sca_findings.json"),
        PathBuf::from("./sca_findings.json"),
        PathBuf::from(".bazbom-cache/sca_findings.json"),
    ];

    let findings_path = possible_paths
        .iter()
        .find(|p| p.exists())
        .ok_or_else(|| anyhow::anyhow!(
            "No scan results found. Run 'bazbom scan' first to generate findings.\n\
             Expected locations:\n  - {}\n  - {}\n  - {}",
            possible_paths[0].display(),
            possible_paths[1].display(),
            possible_paths[2].display()
        ))?;

    let content = fs::read_to_string(findings_path)?;
    let findings: Value = serde_json::from_str(&content)?;

    let mut fixable_vulns = Vec::new();

    // Parse vulnerabilities from findings JSON
    if let Some(vulns) = findings.get("vulnerabilities").and_then(|v| v.as_array()) {
        for vuln in vulns {
            // Extract vulnerability data
            let cve_id = vuln.get("id")
                .or_else(|| vuln.get("cve"))
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();

            let package = vuln.get("package")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let current_version = vuln.get("version")
                .or_else(|| vuln.get("current_version"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let fixed_version = vuln.get("fixed_version")
                .or_else(|| vuln.get("recommended_version"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // Only include if there's a fix available
            if let Some(fixed) = fixed_version {
                let severity_str = vuln.get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("MEDIUM");

                let severity = match severity_str.to_uppercase().as_str() {
                    "CRITICAL" => Severity::Critical,
                    "HIGH" => Severity::High,
                    "MEDIUM" => Severity::Medium,
                    _ => Severity::Low,
                };

                let epss_score = vuln.get("epss_score")
                    .or_else(|| vuln.get("epss"))
                    .and_then(|v| v.as_f64());

                let in_cisa_kev = vuln.get("in_cisa_kev")
                    .or_else(|| vuln.get("cisa_kev"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let description = vuln.get("description")
                    .or_else(|| vuln.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("No description available")
                    .to_string();

                fixable_vulns.push(FixableVulnerability {
                    cve_id,
                    package,
                    current_version,
                    fixed_version: fixed,
                    severity,
                    epss_score,
                    in_cisa_kev,
                    description,
                    breaking_changes: 0, // TODO: Could be estimated from version diff
                    estimated_effort_hours: 1.0, // TODO: Could be ML-based estimate
                });
            }
        }
    }

    // Fall back to demo data if no real vulnerabilities found
    if fixable_vulns.is_empty() {
        Ok(vec![
            FixableVulnerability {
                cve_id: "DEMO-1234".to_string(),
                package: "example:package".to_string(),
                current_version: "1.0.0".to_string(),
                fixed_version: "1.0.1".to_string(),
                severity: Severity::Medium,
                epss_score: Some(0.15),
                in_cisa_kev: false,
                description: "Demo vulnerability - no actual scan results found".to_string(),
                breaking_changes: 0,
                estimated_effort_hours: 1.0,
            },
        ])
    } else {
        Ok(fixable_vulns)
    }
}
