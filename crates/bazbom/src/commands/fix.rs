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
    let vulnerabilities = load_vulnerabilities_from_scan()?;

    if vulnerabilities.is_empty() {
        println!("✨ No vulnerabilities found! Your project is clean.");
        return Ok(());
    }

    let mut session = InteractiveFix::new(vulnerabilities);
    session.run().await
}

/// Estimate breaking changes based on version difference using semver analysis
fn estimate_breaking_changes(current_version: &str, fixed_version: &str) -> usize {
    // Parse version numbers (major.minor.patch)
    let parse_version = |v: &str| -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 3 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                parts[2].parse().ok()?,
            ))
        } else if parts.len() == 2 {
            Some((
                parts[0].parse().ok()?,
                parts[1].parse().ok()?,
                0,
            ))
        } else if parts.len() == 1 {
            Some((
                parts[0].parse().ok()?,
                0,
                0,
            ))
        } else {
            None
        }
    };

    match (parse_version(current_version), parse_version(fixed_version)) {
        (Some((from_major, from_minor, _)), Some((to_major, to_minor, _))) => {
            if from_major != to_major {
                // Major version change = likely many breaking changes
                8
            } else if from_minor != to_minor {
                // Minor version change = possible breaking changes
                2
            } else {
                // Patch version change = minimal/no breaking changes
                0
            }
        }
        _ => {
            // Non-semver versions - assume some risk
            1
        }
    }
}

/// Estimate effort in hours based on severity, version difference, and breaking changes
fn estimate_effort_hours(
    severity: &str,
    current_version: &str,
    fixed_version: &str,
    breaking_changes: usize,
) -> f64 {
    let mut hours = 0.5; // Base overhead for any update

    // Factor in severity
    hours += match severity {
        "CRITICAL" => 2.0,
        "HIGH" => 1.0,
        "MEDIUM" => 0.5,
        _ => 0.25,
    };

    // Factor in version change magnitude
    let parse_version = |v: &str| -> Option<(u32, u32)> {
        let parts: Vec<&str> = v.split('.').collect();
        if parts.len() >= 2 {
            Some((parts[0].parse().ok()?, parts[1].parse().ok()?))
        } else if parts.len() == 1 {
            Some((parts[0].parse().ok()?, 0))
        } else {
            None
        }
    };

    if let (Some((from_major, from_minor)), Some((to_major, to_minor))) =
        (parse_version(current_version), parse_version(fixed_version)) {
        if from_major != to_major {
            hours += 4.0; // Major version bump
        } else if from_minor != to_minor {
            hours += 1.0; // Minor version bump
        }
    }

    // Factor in breaking changes (30 min per change)
    hours += (breaking_changes as f64) * 0.5;

    // Round to nearest 0.25 hours
    (hours * 4.0).round() / 4.0
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

                // Estimate breaking changes based on version difference
                let breaking_changes = estimate_breaking_changes(&current_version, &fixed);

                // Estimate effort hours based on severity, version diff, and breaking changes
                let estimated_effort_hours = estimate_effort_hours(
                    severity_str,
                    &current_version,
                    &fixed,
                    breaking_changes,
                );

                fixable_vulns.push(FixableVulnerability {
                    cve_id,
                    package,
                    current_version,
                    fixed_version: fixed,
                    severity,
                    epss_score,
                    in_cisa_kev,
                    description,
                    breaking_changes,
                    estimated_effort_hours,
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
