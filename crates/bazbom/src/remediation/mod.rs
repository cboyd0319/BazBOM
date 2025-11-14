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
pub mod github;
pub mod suggestions;
pub mod types;
pub mod updaters;
pub mod version;

// Re-export commonly used types and functions
pub use apply::{apply_fixes, apply_fixes_with_testing};
pub use github::generate_pr;
pub use suggestions::{enrich_with_depsdev, generate_suggestions};
pub use types::{
    ApplyResult, ApplyResultWithTests, PrConfig, RemediationReport, RemediationSuggestion,
    RemediationSummary,
};
pub use updaters::{get_updater, DependencyUpdater};
pub use version::parse_semantic_version;

use anyhow::Result;
use anyhow::Context;
use bazbom_advisories::Vulnerability;

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

    // 1. Load vulnerabilities from scan results
    let findings_path = Path::new("sca_findings.json");

    if !findings_path.exists() {
        anyhow::bail!(
            "No scan results found. Please run 'bazbom scan' first to generate sca_findings.json"
        );
    }

    let findings_content = std::fs::read_to_string(findings_path)?;
    let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

    // Parse vulnerabilities from JSON
    let vulnerabilities_json = findings
        .get("vulnerabilities")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Invalid findings format: missing vulnerabilities array"))?;

    let vulnerabilities: Vec<Vulnerability> = vulnerabilities_json
        .iter()
        .filter_map(|v| serde_json::from_value(v.clone()).ok())
        .collect();

    if vulnerabilities.is_empty() {
        println!("✅ No vulnerabilities found to fix. Your project is secure!");
        return Ok(());
    }

    println!("🔍 Found {} vulnerabilities to analyze\n", vulnerabilities.len());

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
        println!("⚠️  LLM analysis (provider: {}, model: {:?}) not yet available\n", llm_provider, llm_model);
    }

    // 4. Display suggestions
    if suggest || (!apply_flag && !pr) {
        println!("=== Remediation Suggestions ===\n");
        println!("Total vulnerabilities: {}", report.summary.total_vulnerabilities);
        println!("Fixable: {}", report.summary.fixable);
        println!("Unfixable: {}", report.summary.unfixable);
        println!();

        for (i, suggestion) in suggestions.iter().enumerate() {
            println!("{}. {} ({})", i + 1, suggestion.vulnerability_id, suggestion.severity);
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

            let result = apply_fixes_with_testing(&confirmed_fixes, build_system, project_root, false)?;

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
            let result = apply_fixes_with_testing(&fixable_cloned, build_system, project_root, false)?;

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
            .map_err(|_| anyhow::anyhow!("GitHub token not found. Set GITHUB_TOKEN or GH_TOKEN environment variable"))?;

        // Detect repo from git remote
        let repo_output = std::process::Command::new("git")
            .args(["remote", "get-url", "origin"])
            .current_dir(project_root)
            .output()
            .context("Failed to get git remote")?;

        if !repo_output.status.success() {
            anyhow::bail!("No git remote found. Make sure this is a git repository.");
        }

        let remote_url = String::from_utf8_lossy(&repo_output.stdout).trim().to_string();

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
            format!("bazbom/security-fixes-{}", chrono::Utc::now().format("%Y%m%d")),
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
