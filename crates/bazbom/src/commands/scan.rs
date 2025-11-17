use crate::smart_defaults::SmartDefaults;
use anyhow::Result;
use std::path::PathBuf;

/// Handle the `bazbom scan` command
///
/// This is a placeholder - the full implementation will be extracted from main.rs
/// in a subsequent refactoring pass to keep this module under 500 lines.
#[allow(clippy::too_many_arguments)]
pub async fn handle_scan(
    path: String,
    profile: Option<String>,
    mut reachability: bool,
    fast: bool,
    format: String,
    out_dir: String,
    mut json: bool,
    bazel_targets_query: Option<String>,
    bazel_targets: Option<Vec<String>>,
    bazel_affected_by_files: Option<Vec<String>>,
    bazel_universe: String,
    cyclonedx: bool,
    with_semgrep: bool,
    with_codeql: Option<bazbom::cli::CodeqlSuite>,
    autofix: Option<bazbom::cli::AutofixMode>,
    containers: Option<bazbom::cli::ContainerStrategy>,
    no_upload: bool,
    target: Option<String>,
    mut incremental: bool,
    base: String,
    mut diff: bool,
    baseline: Option<String>,
    benchmark: bool,
    ml_risk: bool,
    jira_create: bool,
    jira_dry_run: bool,
    github_pr: bool,
    github_pr_dry_run: bool,
    auto_remediate: bool,
    remediate_min_severity: Option<String>,
    remediate_reachable_only: bool,
) -> Result<()> {
    // Apply smart defaults if no flags were explicitly set
    let defaults = SmartDefaults::detect();

    // Show what we detected (if any smart defaults were applied)
    let smart_defaults_enabled = std::env::var("BAZBOM_NO_SMART_DEFAULTS").is_err();
    if smart_defaults_enabled && (defaults.is_ci || defaults.enable_reachability || defaults.is_pr)
    {
        defaults.print_detection();
    }

    // Auto-enable features based on environment (only if not explicitly set)
    if defaults.enable_json && !json && smart_defaults_enabled {
        println!("  → Enabling JSON output for CI");
        json = true;
    }

    if defaults.enable_reachability && !reachability && !fast && smart_defaults_enabled {
        println!(
            "  → Enabling reachability analysis (repo < {}MB)",
            defaults.repo_size / 1_000_000
        );
        reachability = true;
    }

    if defaults.enable_incremental && !incremental && smart_defaults_enabled {
        println!("  → Enabling incremental mode for PR");
        incremental = true;
    }

    if defaults.enable_diff && !diff && baseline.is_some() && smart_defaults_enabled {
        println!("  → Enabling diff mode (baseline found)");
        diff = true;
    }

    if smart_defaults_enabled && (defaults.is_ci || defaults.enable_reachability || defaults.is_pr)
    {
        println!();
    }

    // Load profile from bazbom.toml if specified
    if let Some(ref profile_name) = profile {
        if let Err(e) = apply_profile(profile_name, &path) {
            eprintln!("Warning: Failed to load profile '{}': {}", profile_name, e);
            eprintln!("Continuing with CLI arguments only...");
        }
    }

    // Handle diff mode - compare with baseline
    if diff {
        if let Some(ref baseline_path) = baseline {
            return compare_with_baseline(&path, baseline_path, &out_dir);
        } else {
            eprintln!("Error: --diff requires --baseline=<file>");
            eprintln!("Example: bazbom scan . --diff --baseline=baseline-findings.json");
            return Ok(());
        }
    }

    // Handle JSON output mode
    if json {
        // JSON mode: suppress normal output, return structured JSON at end
        std::env::set_var("BAZBOM_JSON_MODE", "1");
    }

    // Check if any orchestration flags are set
    let use_orchestrator = cyclonedx
        || with_semgrep
        || with_codeql.is_some()
        || autofix.is_some()
        || containers.is_some();

    if use_orchestrator {
        // Use new orchestration path
        println!("[bazbom] using orchestrated scan mode");
        let workspace = PathBuf::from(&path);
        let output_dir = PathBuf::from(&out_dir);

        let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
            workspace,
            output_dir,
            bazbom::scan_orchestrator::ScanOrchestratorOptions {
                cyclonedx,
                with_semgrep,
                with_codeql,
                autofix,
                containers,
                no_upload,
                target,
                threat_detection: None,
                incremental: false,
                benchmark,
            },
        )?;

        return orchestrator.run();
    }

    // Original scan logic - delegate to helper function
    let scan_result = bazbom::scan::handle_legacy_scan(
        path.clone(),
        reachability,
        fast,
        format.clone(),
        out_dir.clone(),
        bazel_targets_query,
        bazel_targets,
        bazel_affected_by_files,
        bazel_universe,
        incremental,
        base,
        benchmark,
        ml_risk,
    );

    // After scan completes, run auto-remediation if enabled
    if scan_result.is_ok() && (jira_create || github_pr || auto_remediate) {
        if let Err(e) = run_auto_remediation(
            &out_dir,
            jira_create,
            jira_dry_run,
            github_pr,
            github_pr_dry_run,
            auto_remediate,
            remediate_min_severity,
            remediate_reachable_only,
        )
        .await
        {
            eprintln!("Auto-remediation failed: {}", e);
            // Don't fail the scan if auto-remediation fails
        }
    }

    scan_result
}

/// Run auto-remediation after scan
async fn run_auto_remediation(
    out_dir: &str,
    jira_create: bool,
    jira_dry_run: bool,
    github_pr: bool,
    github_pr_dry_run: bool,
    auto_remediate: bool,
    remediate_min_severity: Option<String>,
    remediate_reachable_only: bool,
) -> Result<()> {
    use bazbom::remediation::{process_auto_remediation, AutoRemediationConfig};

    // Build config from flags
    let config = AutoRemediationConfig::from_flags(
        jira_create,
        jira_dry_run,
        github_pr,
        github_pr_dry_run,
        auto_remediate,
        remediate_min_severity,
        remediate_reachable_only,
    );

    // Load vulnerabilities from scan results
    let findings_path = format!("{}/sca_findings.json", out_dir);
    if !std::path::Path::new(&findings_path).exists() {
        eprintln!("Warning: Scan results not found at {}", findings_path);
        return Ok(());
    }

    let findings_content = std::fs::read_to_string(&findings_path)?;
    let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

    // Parse vulnerabilities from JSON
    let vulnerabilities: Vec<bazbom_advisories::Vulnerability> = findings
        .get("vulnerabilities")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| serde_json::from_value(v.clone()).ok())
                .collect()
        })
        .unwrap_or_default();

    if vulnerabilities.is_empty() {
        println!("\n✅ No vulnerabilities found - auto-remediation not needed");
        return Ok(());
    }

    // Run auto-remediation
    let result = process_auto_remediation(&vulnerabilities, &config).await?;

    // Print summary
    result.print_summary();

    Ok(())
}

/// Apply a named profile from bazbom.toml
fn apply_profile(profile_name: &str, project_path: &str) -> anyhow::Result<()> {
    use bazbom::config::Config;

    let config_path = std::path::Path::new(project_path).join("bazbom.toml");

    if !config_path.exists() {
        anyhow::bail!("bazbom.toml not found in project directory");
    }

    let config = Config::load(&config_path)?;

    let profile = config
        .get_profile(profile_name)
        .ok_or_else(|| anyhow::anyhow!("Profile '{}' not found in bazbom.toml", profile_name))?;

    println!("[bazbom] Loaded profile '{}':", profile_name);

    if let Some(reachability) = profile.reachability {
        println!("  - reachability: {}", reachability);
    }
    if let Some(ref format) = profile.format {
        println!("  - format: {}", format);
    }
    if let Some(ml_risk) = profile.ml_risk {
        println!("  - ml_risk: {}", ml_risk);
    }

    // Note: Profile values are informational only in this implementation
    // A complete implementation would override the function parameters
    // For now, this demonstrates the profile loading mechanism

    Ok(())
}

/// Compare current scan with baseline findings
fn compare_with_baseline(scan_path: &str, baseline_path: &str, out_dir: &str) -> Result<()> {
    use colored::Colorize;
    use std::collections::HashSet;

    println!();
    println!("{}", "🔄 Diff Mode: Comparing with baseline".bold().cyan());
    println!();

    // Load baseline
    if !std::path::Path::new(baseline_path).exists() {
        println!(
            "{}",
            format!("Error: Baseline file not found: {}", baseline_path)
                .red()
                .bold()
        );
        println!();
        println!("Generate a baseline first:");
        println!("  {} {}", "bazbom scan".green(), scan_path.dimmed());
        println!(
            "  {} {}",
            "mv".green(),
            format!("{}/sca_findings.sarif baseline-findings.json", out_dir).dimmed()
        );
        println!();
        return Ok(());
    }

    let baseline_content = std::fs::read_to_string(baseline_path)?;
    let baseline: serde_json::Value = serde_json::from_str(&baseline_content)?;

    // Run current scan
    println!("{} Running current scan...", "▶".dimmed());
    let current_findings_path = format!("{}/sca_findings.sarif", out_dir);

    // For now, check if current findings exist
    let current_content = if std::path::Path::new(&current_findings_path).exists() {
        std::fs::read_to_string(&current_findings_path)?
    } else {
        println!(
            "{}",
            "  No current findings found - using empty results".yellow()
        );
        r#"{"version":"2.1.0","runs":[]}"#.to_string()
    };

    let current: serde_json::Value = serde_json::from_str(&current_content)?;

    // Extract CVE IDs from findings
    let baseline_cves = extract_cve_ids(&baseline);
    let current_cves = extract_cve_ids(&current);

    // Calculate diff
    let new_vulns: HashSet<_> = current_cves.difference(&baseline_cves).collect();
    let fixed_vulns: HashSet<_> = baseline_cves.difference(&current_cves).collect();
    let unchanged_vulns: HashSet<_> = baseline_cves.intersection(&current_cves).collect();

    // Display results
    println!();
    println!("{}", "📊 Diff Summary:".bold());
    println!("  Baseline vulnerabilities: {}", baseline_cves.len());
    println!("  Current vulnerabilities:  {}", current_cves.len());
    println!();

    if !new_vulns.is_empty() {
        println!(
            "{} {} new {}",
            "⚠️".red(),
            new_vulns.len(),
            if new_vulns.len() == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
        for cve in &new_vulns {
            println!("  {} {}", "+".red(), cve.red());
        }
        println!();
    }

    if !fixed_vulns.is_empty() {
        println!(
            "{} {} fixed {}",
            "✓".green(),
            fixed_vulns.len(),
            if fixed_vulns.len() == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
        for cve in &fixed_vulns {
            println!("  {} {}", "-".green(), cve.green());
        }
        println!();
    }

    if !unchanged_vulns.is_empty() {
        println!(
            "{} {} unchanged {}",
            "→".dimmed(),
            unchanged_vulns.len(),
            if unchanged_vulns.len() == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
    }

    println!();
    Ok(())
}

/// Extract CVE IDs from SARIF findings
fn extract_cve_ids(findings: &serde_json::Value) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let mut cve_ids = HashSet::new();

    if let Some(runs) = findings.get("runs").and_then(|r| r.as_array()) {
        for run in runs {
            if let Some(results) = run.get("results").and_then(|r| r.as_array()) {
                for result in results {
                    if let Some(rule_id) = result.get("ruleId").and_then(|r| r.as_str()) {
                        cve_ids.insert(rule_id.to_string());
                    }
                }
            }
        }
    }

    cve_ids
}
