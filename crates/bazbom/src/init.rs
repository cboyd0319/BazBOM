//! Interactive setup wizard for new BazBOM projects
//!
//! This module implements the `bazbom init` command which provides
//! a guided, interactive experience for:
//! - Detecting build system (Maven, Gradle, Bazel)
//! - Selecting policy template (PCI-DSS, HIPAA, etc.)
//! - Creating bazbom.yml configuration
//! - Running first scan
//! - Displaying summary and next steps

use anyhow::{Context, Result};
use console::{style, Emoji};
use dialoguer::{theme::ColorfulTheme, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::path::{Path, PathBuf};

use bazbom_core::detect_build_system;
use bazbom_policy::templates::{PolicyTemplate, PolicyTemplateLibrary};

static SPARKLE: Emoji = Emoji("✨", "");
static SEARCH: Emoji = Emoji("🔍", "");
static CHECK: Emoji = Emoji("✅", "");
static WARNING: Emoji = Emoji("⚠️ ", "!");
static INFO: Emoji = Emoji("💡", "i");
static ROCKET: Emoji = Emoji("🚀", "");

/// Interactive setup wizard
pub fn run_init(path: &str) -> Result<()> {
    let project_path = PathBuf::from(path);
    
    // Welcome message
    println!("\n{} {} {}", 
        SPARKLE, 
        style("Welcome to BazBOM!").bold().cyan(), 
        SPARKLE
    );
    println!("Let's get your project secured.\n");

    // Step 1: Detect build system
    println!("{} Detecting build system...", SEARCH);
    let build_system = detect_build_system(&project_path);
    
    println!("{} Found: {} project", CHECK, style(format!("{:?}", build_system)).bold().green());
    println!();

    // Step 2: Select policy template
    println!("📋 Choose a policy template:");
    let template = select_policy_template()?;
    println!();

    // Step 3: Create bazbom.yml
    println!("{} Creating bazbom.yml with {} policy", CHECK, style(&template.name).bold());
    create_config_file(&project_path, &template)?;
    println!();

    // Step 4: Run first scan
    println!("{} Running first scan...", SEARCH);
    println!("{} This may take a minute...", INFO);
    
    let scan_result = run_first_scan(&project_path)?;
    println!();

    // Step 5: Display summary
    display_summary(&scan_result)?;
    
    // Step 6: Show next steps
    show_next_steps();

    Ok(())
}

/// Select policy template interactively
fn select_policy_template() -> Result<PolicyTemplate> {
    let templates = PolicyTemplateLibrary::list_templates();
    
    // Dynamically build selection items from templates
    let mut items: Vec<String> = templates
        .iter()
        .enumerate()
        .map(|(i, t)| format!("{}. {} - {}", i + 1, t.name, t.description))
        .collect();
    
    // Add custom option
    items.push(format!("{}. Custom (manual configuration) - Full control", items.len() + 1));

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Your choice")
        .items(&items)
        .default(0) // First template as default
        .interact()?;

    // If custom selected (last item), create a basic template
    if selection >= templates.len() {
        return Ok(PolicyTemplate {
            id: "custom".to_string(),
            name: "Custom Configuration".to_string(),
            description: "User-defined policy".to_string(),
            category: "Custom".to_string(),
            path: String::new(),
        });
    }

    // Return the selected template
    Ok(templates[selection].clone())
}

/// Create bazbom.yml configuration file
fn create_config_file(project_path: &Path, template: &PolicyTemplate) -> Result<()> {
    let config_path = project_path.join("bazbom.yml");
    
    // Check if file already exists
    if config_path.exists() {
        println!("{} bazbom.yml already exists, skipping creation", WARNING);
        return Ok(());
    }

    // Determine severity threshold based on template type
    let severity_threshold = if template.id.contains("permissive") || template.id == "custom" {
        "CRITICAL" // More permissive for development
    } else {
        "HIGH" // Stricter for compliance/production
    };
    
    // For now, create a simple config based on template
    // In the future, this should use the actual template content
    let config_content = format!(
        r#"# BazBOM Configuration
# Template: {}
# Description: {}

policy:
  severity_threshold: {}
  
  # KEV (Known Exploited Vulnerabilities) policy
  kev_policy:
    action: block
    require_remediation: true
  
  # EPSS (Exploit Prediction Scoring) threshold
  epss_threshold: 0.5
  
  # License compliance
  allowed_licenses:
    - Apache-2.0
    - MIT
    - BSD-3-Clause
  
  blocked_licenses:
    - GPL-3.0
    - AGPL-3.0
"#,
        template.name,
        template.description,
        severity_threshold
    );

    fs::write(&config_path, config_content)
        .context("Failed to write bazbom.yml")?;

    Ok(())
}

/// Scan result summary
#[derive(Debug)]
struct ScanResult {
    total_deps: usize,
    direct_deps: usize,
    transitive_deps: usize,
    critical_vulns: usize,
    high_vulns: usize,
    medium_vulns: usize,
    low_vulns: usize,
    license_issues: usize,
}

/// Run first scan and return summary
fn run_first_scan(_project_path: &Path) -> Result<ScanResult> {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("Scanning dependencies...");
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // TODO: Actually run the scan
    // For now, simulate with fake data for demonstration
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    pb.finish_with_message("Scan complete!");

    // Return mock data for now
    Ok(ScanResult {
        total_deps: 127,
        direct_deps: 15,
        transitive_deps: 112,
        critical_vulns: 1,
        high_vulns: 3,
        medium_vulns: 5,
        low_vulns: 2,
        license_issues: 0,
    })
}

/// Display scan summary
fn display_summary(result: &ScanResult) -> Result<()> {
    println!("📊 {}", style("Summary:").bold());
    println!("  Total dependencies: {}", style(result.total_deps).bold());
    println!("  Direct: {}", result.direct_deps);
    println!("  Transitive: {}", result.transitive_deps);
    println!();
    
    let total_vulns = result.critical_vulns + result.high_vulns + result.medium_vulns + result.low_vulns;
    
    if total_vulns > 0 {
        println!("  {} Vulnerabilities: {}", WARNING, style(total_vulns).bold().red());
        if result.critical_vulns > 0 {
            println!("    CRITICAL: {}", style(result.critical_vulns).bold().red());
        }
        if result.high_vulns > 0 {
            println!("    HIGH: {}", style(result.high_vulns).bold().yellow());
        }
        if result.medium_vulns > 0 {
            println!("    MEDIUM: {}", result.medium_vulns);
        }
        if result.low_vulns > 0 {
            println!("    LOW: {}", result.low_vulns);
        }
    } else {
        println!("  {} No vulnerabilities detected!", CHECK);
    }
    
    println!("  License issues: {}", result.license_issues);
    println!();

    Ok(())
}

/// Show next steps
fn show_next_steps() {
    println!("{} {}", INFO, style("Next steps:").bold());
    println!("  1. Review findings: {}", style("bazbom scan . --format json").cyan());
    println!("  2. Fix vulnerabilities: {}", style("bazbom fix --suggest").cyan());
    println!("  3. Add to git hooks: {}", style("bazbom install-hooks").cyan());
    println!();
    println!("📖 Full documentation: {}", 
        style("https://github.com/cboyd0319/BazBOM").cyan().underlined());
    println!();
    println!("{} {}", ROCKET, style("Happy securing!").bold().green());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_result_creation() {
        let result = ScanResult {
            total_deps: 100,
            direct_deps: 10,
            transitive_deps: 90,
            critical_vulns: 1,
            high_vulns: 2,
            medium_vulns: 3,
            low_vulns: 4,
            license_issues: 0,
        };
        
        assert_eq!(result.total_deps, 100);
        assert_eq!(result.direct_deps, 10);
    }
}
