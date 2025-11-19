// Remediation automation for BazBOM
// Provides "suggest" and "apply" modes for fixing vulnerabilities
//
// This module has been refactored into focused submodules:
// - types: Data structures (RemediationSuggestion, RemediationReport, etc.)
// - version: Version parsing utilities
// - suggestions: Suggestion generation logic
// - apply: Fix application with testing
// - build_systems: Build system-specific fixers
// - updaters: Polyglot dependency updaters (npm, Python, Go, Rust, Ruby, PHP)
// - github: GitHub PR generation

pub mod apply;
pub mod build_systems;
pub mod database;
pub mod github;
pub mod scan_integration;
pub mod suggestions;
pub mod types;
pub mod updaters;
pub mod version;

// Re-export commonly used types and functions
pub use apply::{apply_fixes, apply_fixes_with_testing};
pub use database::{GitHubPrRecord, JiraIssueRecord, RemediationDatabase, SyncLogRecord};
pub use github::generate_pr;
pub use scan_integration::{
    process_auto_remediation, AutoRemediationConfig, AutoRemediationResult,
};
pub use suggestions::{enrich_with_depsdev, generate_suggestions};
pub use types::{
    ApplyResult, ApplyResultWithTests, PrConfig, RemediationReport, RemediationSuggestion,
    RemediationSummary,
};
pub use updaters::{get_updater, DependencyUpdater};
pub use version::parse_semantic_version;

use anyhow::Context;
use anyhow::Result;
use bazbom_advisories::Vulnerability;
use bazbom_formats::sarif::SarifReport;
use std::path::PathBuf;

/// Handle the fix command
///
/// Generates remediation suggestions or applies fixes for vulnerabilities
#[allow(clippy::too_many_arguments)]
pub fn handle_fix_command(
    suggest: bool,
    apply_flag: bool,
    pr: bool,
    interactive: bool,
    ml_prioritize: bool,
    llm: bool,
    llm_provider: String,
    llm_model: Option<String>,
) -> Result<()> {
    use std::path::Path;

    // 1. Load vulnerabilities from scan results (SARIF or JSON)
    let vulnerabilities = load_vulnerabilities_from_scan()?;

    if vulnerabilities.is_empty() {
        println!("✅ No vulnerabilities found to fix. Your project is secure!");
        return Ok(());
    }

    println!(
        "🔍 Found {} vulnerabilities to analyze\n",
        vulnerabilities.len()
    );

    // 2. Detect build system and project root
    let project_root = Path::new(".");
    let build_system = bazbom_core::detect_build_system(project_root);

    println!("📦 Detected build system: {:?}\n", build_system);

    // 3. Generate remediation suggestions
    let report = generate_suggestions(&vulnerabilities, build_system);
    let suggestions = &report.suggestions;

    if ml_prioritize {
        println!("⚠️  ML prioritization not yet available\n");
    }

    if llm {
        println!(
            "⚠️  LLM analysis (provider: {}, model: {:?}) not yet available\n",
            llm_provider, llm_model
        );
    }

    // 4. Display suggestions
    if suggest || (!apply_flag && !pr) {
        println!("=== Remediation Suggestions ===\n");
        println!(
            "Total vulnerabilities: {}",
            report.summary.total_vulnerabilities
        );
        println!("Fixable: {}", report.summary.fixable);
        println!("Unfixable: {}", report.summary.unfixable);
        println!();

        for (i, suggestion) in suggestions.iter().enumerate() {
            println!(
                "{}. {} ({})",
                i + 1,
                suggestion.vulnerability_id,
                suggestion.severity
            );
            println!("   Package: {}", suggestion.affected_package);
            println!("   Current: {}", suggestion.current_version);
            if let Some(ref fixed) = suggestion.fixed_version {
                println!("   Fix: Update to {}", fixed);
                if let Some(ref changes) = suggestion.breaking_changes {
                    println!("   ⚠️  Breaking changes: {}", changes);
                }
            } else {
                println!("   ⚠️  No fix available yet");
            }
            println!();
        }

        if !apply_flag && !pr {
            println!("Run with --apply to apply these fixes automatically");
            println!("Run with --pr to create a GitHub pull request");
            return Ok(());
        }
    }

    // 5. Apply fixes if requested
    if apply_flag {
        println!("=== Applying Fixes ===\n");

        let fixable: Vec<_> = suggestions
            .iter()
            .filter(|s| s.fixed_version.is_some())
            .collect();

        if fixable.is_empty() {
            println!("❌ No fixable vulnerabilities found");
            return Ok(());
        }

        if interactive {
            println!("🔧 Interactive mode - prompting for each fix\n");

            let mut confirmed_fixes = Vec::new();

            for suggestion in &fixable {
                let prompt = format!(
                    "Apply fix for {}? ({} → {})",
                    suggestion.affected_package,
                    suggestion.current_version,
                    suggestion.fixed_version.as_ref().unwrap()
                );

                if dialoguer::Confirm::new().with_prompt(prompt).interact()? {
                    confirmed_fixes.push((*suggestion).clone());
                }
            }

            if confirmed_fixes.is_empty() {
                println!("No fixes confirmed");
                return Ok(());
            }

            let result =
                apply_fixes_with_testing(&confirmed_fixes, build_system, project_root, false)?;

            println!("\n✅ Applied {} fixes successfully", result.applied.len());
            if !result.failed.is_empty() {
                println!("❌ {} fixes failed:", result.failed.len());
                for (vuln_id, error) in &result.failed {
                    println!("   - {}: {}", vuln_id, error);
                }
            }
            if result.tests_passed {
                println!("✅ All tests passed!");
            }
        } else {
            println!("🔧 Automatic mode - applying all fixes\n");

            let fixable_cloned: Vec<RemediationSuggestion> = fixable.into_iter().cloned().collect();
            let result =
                apply_fixes_with_testing(&fixable_cloned, build_system, project_root, false)?;

            println!("\n✅ Applied {} fixes successfully", result.applied.len());
            if !result.failed.is_empty() {
                println!("❌ {} fixes failed:", result.failed.len());
                for (vuln_id, error) in &result.failed {
                    println!("   - {}: {}", vuln_id, error);
                }
            }

            if result.tests_passed {
                println!("✅ All tests passed!");
            } else if result.rollback_performed {
                println!("⚠️  Tests failed - changes were rolled back");
            }
        }
    }

    // 6. Generate PR if requested
    if pr {
        println!("\n=== GitHub PR Generation ===\n");

        // Get fixable suggestions
        let fixable: Vec<RemediationSuggestion> = suggestions
            .iter()
            .filter(|s| s.fixed_version.is_some())
            .cloned()
            .collect();

        if fixable.is_empty() {
            println!("❌ No fixable vulnerabilities to create PR for");
            return Ok(());
        }

        // Get GitHub token from environment
        let github_token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .map_err(|_| {
                anyhow::anyhow!(
                    "GitHub token not found. Set GITHUB_TOKEN or GH_TOKEN environment variable"
                )
            })?;

        // Detect repo from git remote
        let repo_output = std::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(project_root)
            .output()
            .context("Failed to get git remote")?;

        if !repo_output.status.success() {
            anyhow::bail!("No git remote found. Make sure this is a git repository.");
        }

        let remote_url = String::from_utf8_lossy(&repo_output.stdout)
            .trim()
            .to_string();

        // Extract owner/repo from URL (handles both HTTPS and SSH)
        let repo = if remote_url.contains("github.com") {
            remote_url
                .replace("https://github.com/", "")
                .replace("git@github.com:", "")
                .replace(".git", "")
        } else {
            anyhow::bail!("Remote URL is not a GitHub repository: {}", remote_url);
        };

        // Create PR config
        let config = PrConfig::new(
            github_token,
            repo,
            "main".to_string(), // base branch
            format!(
                "bazbom/security-fixes-{}",
                chrono::Utc::now().format("%Y%m%d")
            ),
        )?;

        println!("📝 Creating GitHub pull request...");

        match generate_pr(&fixable, build_system, project_root, config) {
            Ok(pr_url) => {
                println!("✅ Pull request created: {}", pr_url);
            }
            Err(e) => {
                println!("❌ Failed to create PR: {}", e);
                println!("\nMake sure you have:");
                println!("1. Set GITHUB_TOKEN or GH_TOKEN environment variable");
                println!("2. Pushed your changes to a remote repository");
                println!("3. Have write access to the repository");
            }
        }
    }

    Ok(())
}

/// Load vulnerabilities from scan results (SARIF or JSON)
fn load_vulnerabilities_from_scan() -> Result<Vec<Vulnerability>> {
    use bazbom_advisories::{EpssScore, KevEntry, Severity, SeverityLevel, Priority};
    use std::fs;

    // Try SARIF files first (new format), then fall back to JSON (legacy)
    let sarif_paths = [
        PathBuf::from("./findings/sca.sarif"),
        PathBuf::from("./bazbom-findings/sca.sarif"),
        PathBuf::from("./findings/merged.sarif"),
        PathBuf::from("./bazbom-findings/merged.sarif"),
    ];

    let json_paths = [
        PathBuf::from("./bazbom-findings/sca_findings.json"),
        PathBuf::from("./sca_findings.json"),
        PathBuf::from(".bazbom-cache/sca_findings.json"),
    ];

    // Try loading from SARIF first
    if let Some(sarif_path) = sarif_paths.iter().find(|p| p.exists()) {
        println!("[bazbom] loading vulnerabilities from SARIF: {}", sarif_path.display());

        let content = fs::read_to_string(sarif_path)?;
        let sarif: SarifReport = serde_json::from_str(&content)?;

        let mut vulnerabilities = Vec::new();

        // Iterate through all runs in the SARIF report
        for run in sarif.runs {
            // Iterate through all results (vulnerabilities)
            for result in run.results {
                // Extract data from properties
                if let Some(props) = result.properties {
                    let id = result.rule_id.clone();

                    let component = props.get("component")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let _version = props.get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();

                    // Parse severity from SARIF level
                    let severity_level = match result.level.as_str() {
                        "error" => SeverityLevel::Critical,
                        "warning" => SeverityLevel::High,
                        "note" => SeverityLevel::Medium,
                        _ => SeverityLevel::Low,
                    };

                    // Parse priority from properties
                    let priority = props.get("priority")
                        .and_then(|v| v.as_str())
                        .and_then(|s| match s {
                            "P0" => Some(Priority::P0),
                            "P1" => Some(Priority::P1),
                            "P2" => Some(Priority::P2),
                            "P3" => Some(Priority::P3),
                            "P4" => Some(Priority::P4),
                            _ => None,
                        });

                    let epss = props.get("epss_score")
                        .and_then(|v| v.as_f64())
                        .map(|score| EpssScore {
                            score,
                            percentile: 0.0, // Not available in SARIF
                        });

                    let kev = props.get("cisa_kev")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        .then(|| KevEntry {
                            cve_id: id.clone(),
                            vendor_project: "Unknown".to_string(),
                            product: component.clone(),
                            vulnerability_name: id.clone(),
                            date_added: "Unknown".to_string(),
                            required_action: "Apply updates per vendor instructions".to_string(),
                            due_date: "Unknown".to_string(),
                        });

                    let description = result.message.text.clone();

                    vulnerabilities.push(Vulnerability {
                        id,
                        aliases: Vec::new(),
                        affected: Vec::new(), // Will be empty for SARIF-loaded vulns
                        severity: Some(Severity {
                            cvss_v3: None,
                            cvss_v4: None,
                            level: severity_level,
                        }),
                        summary: Some(description),
                        details: None,
                        references: Vec::new(),
                        published: None,
                        modified: None,
                        epss,
                        kev,
                        priority,
                    });
                }
            }
        }

        println!("[bazbom] loaded {} vulnerabilities from SARIF", vulnerabilities.len());
        return Ok(vulnerabilities);
    }

    // Fall back to JSON (legacy format)
    let findings_path = json_paths.iter().find(|p| p.exists()).ok_or_else(|| {
        anyhow::anyhow!(
            "No scan results found. Run 'bazbom scan' first to generate findings.\n\
             Expected SARIF locations:\n  - {}\n  - {}\n  - {}\n  - {}\n\
             Expected JSON locations:\n  - {}\n  - {}\n  - {}",
            sarif_paths[0].display(),
            sarif_paths[1].display(),
            sarif_paths[2].display(),
            sarif_paths[3].display(),
            json_paths[0].display(),
            json_paths[1].display(),
            json_paths[2].display()
        )
    })?;

    println!("[bazbom] loading vulnerabilities from JSON (legacy): {}", findings_path.display());
    let content = fs::read_to_string(findings_path)?;
    let findings: serde_json::Value = serde_json::from_str(&content)?;

    // Parse vulnerabilities from JSON
    let vulnerabilities_json = findings
        .get("vulnerabilities")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid findings format: missing vulnerabilities array"))?;

    let vulnerabilities: Vec<Vulnerability> = vulnerabilities_json
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();

    Ok(vulnerabilities)
}
