//! Scan integration for automated Jira/GitHub remediation
//!
//! This module handles auto-creation of Jira tickets and GitHub PRs during scans.

use crate::remediation::database::RemediationDatabase;
use anyhow::{Context, Result};
use bazbom_vulnerabilities::Vulnerability;
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
                println!("  FAIL {}", error);
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

    println!("\n{}", "TOOL Starting auto-remediation...".bold().cyan());

    // Initialize database
    let db = RemediationDatabase::new().context("Failed to initialize remediation database")?;

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
            println!(
                "\n{}",
                "Jira Dry-Run Mode (no tickets will be created):".yellow()
            );
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
            println!(
                "\n{}",
                "GitHub Dry-Run Mode (no PRs will be created):".yellow()
            );
        }

        result.github_created = filtered_vulns.len();
        println!("  WARN  GitHub PR creation not yet fully implemented");
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
            if let (Some(ref min_sev), Some(ref vuln_sev)) = (&config.min_severity, &vuln.severity)
            {
                let severity_order = ["CRITICAL", "HIGH", "MEDIUM", "LOW"];
                let min_idx = severity_order
                    .iter()
                    .position(|s| s == min_sev)
                    .unwrap_or(3);
                // Use the level field from Severity struct
                let vuln_sev_str = format!("{:?}", vuln_sev.level).to_uppercase();
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
    db: &RemediationDatabase,
) -> Result<(usize, usize)> {
    use bazbom_jira::client::JiraClient;
    use bazbom_jira::config::JiraConfig;
    use bazbom_jira::models::{CreateIssueRequest, IssueFields, IssueTypeRef, ProjectRef};
    use bazbom_jira::templates::TemplateEngine;
    use std::collections::HashMap;
    use std::path::PathBuf;

    // Load Jira config
    let config_path = PathBuf::from(".bazbom/jira.yml");
    if !config_path.exists() {
        anyhow::bail!("Jira configuration not found. Run 'bazbom jira init' first.");
    }

    let yaml = std::fs::read_to_string(&config_path)?;
    let jira_config: JiraConfig = serde_yaml::from_str(&yaml)?;

    println!(
        "Found {} vulnerabilities for Jira ticket creation",
        vulnerabilities.len()
    );

    if dry_run {
        for vuln in vulnerabilities {
            let package = vuln
                .affected
                .first()
                .map(|a| a.package.as_str())
                .unwrap_or("unknown");
            println!(
                "  Would create ticket for: {} in {}",
                vuln.id.yellow(),
                package
            );
        }
        return Ok((vulnerabilities.len(), 0));
    }

    // Get authentication token
    let token = std::env::var("JIRA_API_TOKEN").context(
        "JIRA_API_TOKEN environment variable not set. Please set it to use Jira integration.",
    )?;

    // Get username if needed
    let username = if let Some(username_env) = &jira_config.auth.username_env {
        std::env::var(username_env).ok()
    } else {
        None
    };

    // Create Jira client
    let client = if let Some(username) = username {
        JiraClient::with_username(&jira_config.url, &token, Some(username))
    } else {
        JiraClient::new(&jira_config.url, &token)
    };

    let template = TemplateEngine::new();
    let mut created = 0;
    let mut skipped = 0;

    for vuln in vulnerabilities {
        // Extract package info from affected packages
        let (package, version) = if let Some(affected) = vuln.affected.first() {
            let pkg = affected.package.as_str();
            // Try to extract version from ranges - VersionEvent is an enum
            let ver = affected
                .ranges
                .first()
                .and_then(|r| r.events.first())
                .map(|e| match e {
                    bazbom_vulnerabilities::VersionEvent::Introduced { introduced } => {
                        introduced.as_str()
                    }
                    bazbom_vulnerabilities::VersionEvent::Fixed { fixed } => fixed.as_str(),
                    bazbom_vulnerabilities::VersionEvent::LastAffected { last_affected } => {
                        last_affected.as_str()
                    }
                })
                .unwrap_or("unknown");
            (pkg, ver)
        } else {
            ("unknown", "unknown")
        };

        // Check for duplicates
        if let Some(existing_key) = db.jira_issue_exists(&vuln.id, package, version)? {
            println!(
                "  SKIP  Skipping {} (ticket {} already exists)",
                vuln.id.yellow(),
                existing_key.cyan()
            );
            skipped += 1;
            continue;
        }

        // Build template variables
        let mut variables = HashMap::new();
        variables.insert("cve_id".to_string(), vuln.id.clone());
        variables.insert("package".to_string(), package.to_string());
        variables.insert("version".to_string(), version.to_string());

        // Handle severity - use the level field from Severity struct
        let severity_str = vuln
            .severity
            .as_ref()
            .map(|s| format!("{:?}", s.level).to_uppercase())
            .unwrap_or_else(|| "MEDIUM".to_string());
        variables.insert("severity".to_string(), severity_str.clone());

        variables.insert(
            "summary".to_string(),
            format!("Security: {} in {}", vuln.id, package),
        );

        // Add description if available
        if let Some(ref details) = vuln.details {
            variables.insert("description".to_string(), details.clone());
        } else if let Some(ref summary) = vuln.summary {
            variables.insert("description".to_string(), summary.clone());
        }

        // Render title and description using template
        let title = template.render_title(&variables)?;
        let description = template.render_description(&variables)?;

        // Create issue request
        let request = CreateIssueRequest {
            fields: IssueFields {
                project: ProjectRef {
                    key: jira_config.project.clone(),
                },
                summary: title.clone(),
                description: Some(description),
                issuetype: IssueTypeRef {
                    name: jira_config.issue_type.clone(),
                },
                labels: Some(vec![
                    "security".to_string(),
                    "bazbom".to_string(),
                    severity_str.to_lowercase(),
                ]),
                priority: None,
                assignee: None,
                custom_fields: HashMap::new(),
            },
        };

        // Create the ticket
        match client.create_issue(request).await {
            Ok(response) => {
                let jira_url = format!("{}/browse/{}", jira_config.url, response.key);
                println!(
                    "  OK Created {} → {}",
                    vuln.id.yellow(),
                    response.key.cyan()
                );

                // Record in database
                if let Err(e) = db.record_jira_issue(
                    &vuln.id,
                    package,
                    version,
                    &response.key,
                    &jira_url,
                    "Open",
                ) {
                    eprintln!("  WARN  Failed to record in database: {}", e);
                }

                created += 1;
            }
            Err(e) => {
                eprintln!("  FAIL Failed to create ticket for {}: {}", vuln.id, e);
            }
        }
    }

    Ok((created, skipped))
}
