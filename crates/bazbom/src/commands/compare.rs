use anyhow::{Context, Result};
use colored::Colorize;

/// Handle the `bazbom compare` command
pub fn handle_compare(base: String, target: Option<String>, verbose: bool) -> Result<()> {
    println!();
    println!(
        "{}",
        "🔄 Comparing security posture between branches"
            .bold()
            .cyan()
    );
    println!();

    let target_ref = target.unwrap_or_else(|| "HEAD".to_string());

    println!("  {} {}", "Base:".dimmed(), base.bold());
    println!("  {} {}", "Target:".dimmed(), target_ref.bold());
    println!();

    // Check if git is available
    check_git_available()?;

    // Verify branches/commits exist
    verify_git_ref(&base)?;
    verify_git_ref(&target_ref)?;

    // Scan base branch
    println!("{} Scanning base ({})...", "▶".dimmed(), base);
    scan_git_ref(&base, "./baseline-scan")?;

    // Scan target branch
    println!("{} Scanning target ({})...", "▶".dimmed(), target_ref);
    scan_git_ref(&target_ref, "./target-scan")?;

    // Compare results
    compare_scan_results(
        "./baseline-scan/sca_findings.sarif",
        "./target-scan/sca_findings.sarif",
        verbose,
    )?;

    // Cleanup temp dirs
    let _ = std::fs::remove_dir_all("./baseline-scan");
    let _ = std::fs::remove_dir_all("./target-scan");

    Ok(())
}

/// Check if git is available
fn check_git_available() -> Result<()> {
    let output = std::process::Command::new("git")
        .arg("--version")
        .output()
        .context("Git not found in PATH")?;

    if !output.status.success() {
        anyhow::bail!("Git is not available");
    }

    Ok(())
}

/// Verify a git reference exists
fn verify_git_ref(git_ref: &str) -> Result<()> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", git_ref])
        .output()
        .context(format!("Failed to verify git ref: {}", git_ref))?;

    if !output.status.success() {
        anyhow::bail!("Git reference '{}' not found", git_ref);
    }

    Ok(())
}

/// Scan a specific git reference
fn scan_git_ref(git_ref: &str, output_dir: &str) -> Result<()> {
    // Create worktree for the ref
    let worktree_path = format!("{}-worktree", output_dir);

    let _ = std::process::Command::new("git")
        .args(["worktree", "add", &worktree_path, git_ref])
        .output();

    // Run scan in worktree
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "--",
            "scan",
            &worktree_path,
            "-o",
            output_dir,
            "--no-upload",
            "--fast",
        ])
        .status()
        .context("Failed to run scan")?;

    // Cleanup worktree
    let _ = std::process::Command::new("git")
        .args(["worktree", "remove", &worktree_path])
        .output();

    if !status.success() {
        println!("{} Scan failed for {}", "⚠️".yellow(), git_ref);
    }

    Ok(())
}

/// Compare two scan results
fn compare_scan_results(baseline_path: &str, target_path: &str, verbose: bool) -> Result<()> {
    use std::collections::HashSet;

    // Read baseline
    let baseline_content = std::fs::read_to_string(baseline_path)
        .unwrap_or_else(|_| r#"{"version":"2.1.0","runs":[]}"#.to_string());
    let baseline: serde_json::Value = serde_json::from_str(&baseline_content)?;

    // Read target
    let target_content = std::fs::read_to_string(target_path)
        .unwrap_or_else(|_| r#"{"version":"2.1.0","runs":[]}"#.to_string());
    let target: serde_json::Value = serde_json::from_str(&target_content)?;

    // Extract CVE IDs
    let baseline_cves = extract_cve_ids(&baseline);
    let target_cves = extract_cve_ids(&target);

    // Calculate diff
    let new_vulns: HashSet<_> = target_cves.difference(&baseline_cves).collect();
    let fixed_vulns: HashSet<_> = baseline_cves.difference(&target_cves).collect();

    // Display results
    println!();
    println!("{}", "📊 Comparison Results:".bold());
    println!();
    println!("  Baseline vulnerabilities: {}", baseline_cves.len());
    println!("  Target vulnerabilities:   {}", target_cves.len());
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
        if verbose {
            for cve in &new_vulns {
                println!("  {} {}", "+".red(), cve.red());
            }
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
        if verbose {
            for cve in &fixed_vulns {
                println!("  {} {}", "-".green(), cve.green());
            }
        }
        println!();
    }

    if new_vulns.is_empty() && fixed_vulns.is_empty() {
        println!("{} No changes in vulnerabilities", "→".dimmed());
        println!();
    }

    // Risk assessment
    let risk_delta = (new_vulns.len() as i32) - (fixed_vulns.len() as i32);
    if risk_delta > 0 {
        println!(
            "{} {} Security posture is WORSE",
            "📉".red(),
            "RISK:".red().bold()
        );
        println!("   {} more vulnerabilities than baseline", risk_delta.abs());
    } else if risk_delta < 0 {
        println!(
            "{} {} Security posture is BETTER",
            "📈".green(),
            "IMPROVEMENT:".green().bold()
        );
        println!(
            "   {} fewer vulnerabilities than baseline",
            risk_delta.abs()
        );
    } else {
        println!(
            "{} {} Security posture is UNCHANGED",
            "→".dimmed(),
            "STATUS:".dimmed()
        );
    }
    println!();

    Ok(())
}

/// Extract CVE IDs from SARIF
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cve_ids() {
        let sarif = serde_json::json!({
            "version": "2.1.0",
            "runs": [{
                "results": [
                    {"ruleId": "CVE-2024-1234"},
                    {"ruleId": "CVE-2024-5678"}
                ]
            }]
        });

        let cves = extract_cve_ids(&sarif);
        assert_eq!(cves.len(), 2);
        assert!(cves.contains("CVE-2024-1234"));
        assert!(cves.contains("CVE-2024-5678"));
    }
}
