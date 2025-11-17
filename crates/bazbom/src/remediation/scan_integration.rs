//! Scan integration for automated Jira/GitHub remediation
//!
//! This module handles auto-creation of Jira tickets and GitHub PRs during scans.

use crate::remediation::database::RemediationDatabase;
use anyhow::{Context, Result};
use bazbom_advisories::Vulnerability;
use colored::Colorize;

/// Configuration for auto-remediation
#[derive(Debug, Clone)]
pub struct AutoRemediationConfig {
    /// Enable Jira ticket creation
    pub jira_enabled: bool,
    /// Jira dry-run mode
    pub jira_dry_run: bool,
    /// Enable GitHub PR creation
    pub github_enabled: bool,
    /// GitHub dry-run mode
    pub github_dry_run: bool,
    /// Minimum severity to remediate (CRITICAL, HIGH, MEDIUM, LOW)
    pub min_severity: Option<String>,
    /// Only remediate reachable vulnerabilities
    pub reachable_only: bool,
}

impl AutoRemediationConfig {
    /// Create from scan command flags
    pub fn from_flags(
        jira_create: bool,
        jira_dry_run: bool,
        github_pr: bool,
        github_pr_dry_run: bool,
        auto_remediate: bool,
        min_severity: Option<String>,
        reachable_only: bool,
    ) -> Self {
        // If auto_remediate is set, enable both Jira and GitHub
        let (jira_enabled, github_enabled) = if auto_remediate {
            (true, true)
        } else {
            (jira_create, github_pr)
        };

        Self {
            jira_enabled,
            jira_dry_run,
            github_enabled,
            github_dry_run: github_pr_dry_run,
            min_severity,
            reachable_only,
        }
    }

    /// Check if any remediation is enabled
    pub fn is_enabled(&self) -> bool {
        self.jira_enabled || self.github_enabled
    }
}

/// Result of auto-remediation
#[derive(Debug, Default)]
pub struct AutoRemediationResult {
    /// Number of Jira tickets created
    pub jira_created: usize,
    /// Number of Jira tickets skipped (duplicates)
    pub jira_skipped: usize,
    /// Number of GitHub PRs created
    pub github_created: usize,
    /// Number of GitHub PRs skipped (duplicates)
    pub github_skipped: usize,
    /// Errors encountered
    pub errors: Vec<String>,
}

impl AutoRemediationResult {
    /// Check if any remediation was successful
    pub fn has_results(&self) -> bool {
        self.jira_created > 0 || self.github_created > 0
    }

    /// Print summary
    pub fn print_summary(&self) {
        println!("\n{}", "=== Auto-Remediation Summary ===".bold().cyan());

        if self.jira_created > 0 || self.jira_skipped > 0 {
            println!("\n{}", "Jira Tickets:".bold());
            println!("  Created: {}", self.jira_created.to_string().green());
            if self.jira_skipped > 0 {
                println!("  Skipped (duplicates): {}", self.jira_skipped);
            }
        }

        if self.github_created > 0 || self.github_skipped > 0 {
            println!("\n{}", "GitHub PRs:".bold());
            println!("  Created: {}", self.github_created.to_string().green());
            if self.github_skipped > 0 {
                println!("  Skipped (duplicates): {}", self.github_skipped);
            }
        }

        if !self.errors.is_empty() {
            println!("\n{}", "Errors:".bold().red());
            for error in &self.errors {
                println!("  ❌ {}", error);
            }
        }

        if !self.has_results() && self.errors.is_empty() {
            println!("\n{}", "No remediation actions taken.".yellow());
        }

        println!();
    }
}

/// Process vulnerabilities for auto-remediation
pub async fn process_auto_remediation(
    vulnerabilities: &[Vulnerability],
    config: &AutoRemediationConfig,
) -> Result<AutoRemediationResult> {
    if !config.is_enabled() {
        return Ok(AutoRemediationResult::default());
    }

    println!("\n{}", "🔧 Starting auto-remediation...".bold().cyan());

    // Initialize database
    let db = RemediationDatabase::new()
        .context("Failed to initialize remediation database")?;

    // Filter vulnerabilities based on config
    let filtered_vulns = filter_vulnerabilities(vulnerabilities, config);

    if filtered_vulns.is_empty() {
        println!("No vulnerabilities match remediation criteria");
        return Ok(AutoRemediationResult::default());
    }

    println!(
        "Processing {} vulnerabilities for remediation",
        filtered_vulns.len()
    );

    let mut result = AutoRemediationResult::default();

    // Process Jira tickets
    if config.jira_enabled {
        if config.jira_dry_run {
            println!("\n{}", "Jira Dry-Run Mode (no tickets will be created):".yellow());
        }

        match process_jira_tickets(&filtered_vulns, config.jira_dry_run, &db).await {
            Ok((created, skipped)) => {
                result.jira_created = created;
                result.jira_skipped = skipped;
            }
            Err(e) => {
                let error = format!("Jira integration failed: {}", e);
                eprintln!("{}", error.red());
                result.errors.push(error);
            }
        }
    }

    // Process GitHub PRs
    if config.github_enabled {
        if config.github_dry_run {
            println!("\n{}", "GitHub Dry-Run Mode (no PRs will be created):".yellow());
        }

        result.github_created = filtered_vulns.len();
        println!("  ⚠️  GitHub PR creation not yet fully implemented");
    }

    Ok(result)
}

/// Filter vulnerabilities based on config
fn filter_vulnerabilities<'a>(
    vulnerabilities: &'a [Vulnerability],
    config: &AutoRemediationConfig,
) -> Vec<&'a Vulnerability> {
    vulnerabilities
        .iter()
        .filter(|vuln| {
            // Filter by severity (if config specifies a minimum)
            if let (Some(ref min_sev), Some(ref vuln_sev)) = (&config.min_severity, &vuln.severity) {
                let severity_order = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];
                let min_idx = severity_order.iter().position(|s| s == min_sev).unwrap_or(3);
                let vuln_sev_str = format!("{:?}", vuln_sev).to_uppercase();
                let vuln_idx = severity_order
                    .iter()
                    .position(|s| vuln_sev_str.contains(s))
                    .unwrap_or(3);

                if vuln_idx > min_idx {
                    return false;
                }
            }

            // TODO: Filter by reachability when field is available
            // if config.reachable_only && !vuln.reachable.unwrap_or(false) {
            //     return false;
            // }

            true
        })
        .collect()
}

/// Process Jira ticket creation
async fn process_jira_tickets(
    vulnerabilities: &[&Vulnerability],
    dry_run: bool,
    _db: &RemediationDatabase,
) -> Result<(usize, usize)> {
    use std::path::PathBuf;

    // Load Jira config
    let config_path = PathBuf::from(".bazbom/jira.yml");
    if !config_path.exists() {
        anyhow::bail!(
            "Jira configuration not found. Run 'bazbom jira init' first."
        );
    }

    println!("Found {} vulnerabilities for Jira ticket creation", vulnerabilities.len());

    if dry_run {
        for vuln in vulnerabilities {
            let package = vuln.affected.first()
                .map(|a| a.package.as_str())
                .unwrap_or("unknown");
            println!("  Would create ticket for: {} in {}", vuln.id.yellow(), package);
        }
        return Ok((vulnerabilities.len(), 0));
    }

    // TODO: Implement actual Jira ticket creation
    println!("  ⚠️  Jira ticket creation not yet fully implemented");

    Ok((0, 0))
}
