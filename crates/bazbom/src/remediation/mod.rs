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

/// Handle the fix command
///
/// Generates remediation suggestions or applies fixes for vulnerabilities
#[allow(clippy::too_many_arguments)]
pub fn handle_fix_command(
    suggest: bool,
    apply: bool,
    pr: bool,
    interactive: bool,
    ml_prioritize: bool,
    llm: bool,
    llm_provider: String,
    llm_model: Option<String>,
) -> Result<()> {
    use std::path::Path;

    // Output the mode flags first (for testing/verification)
    if suggest {
        println!("suggest=true");
    }
    if apply {
        println!("apply=true");
    }
    if pr {
        println!("pr=true");
    }
    if interactive {
        println!("interactive=true");
    }
    if ml_prioritize {
        println!("ml_prioritize=true");
    }
    if llm {
        println!("llm=true provider={} model={:?}", llm_provider, llm_model);
    }

    // 1. Load vulnerabilities from scan results (if available)
    let findings_path = Path::new("sca_findings.json");
    let vulnerability_count = if findings_path.exists() {
        let findings_content = std::fs::read_to_string(findings_path)?;
        let findings: serde_json::Value = serde_json::from_str(&findings_content)?;

        let vulnerabilities = findings
            .get("findings")
            .and_then(|f| f.as_array())
            .ok_or_else(|| anyhow::anyhow!("Invalid findings format"))?;

        if vulnerabilities.is_empty() {
            println!("No vulnerabilities found to fix. Your project is secure!");
            return Ok(());
        }

        vulnerabilities.len()
    } else {
        // No scan results available - this is OK for testing/demonstration
        0
    };

    if vulnerability_count > 0 {
        println!("Found {} vulnerabilities to analyze", vulnerability_count);
    }

    // 2. Generate remediation suggestions
    if suggest {
        println!("\n=== Remediation Suggestions ===");
        println!("Mode: Suggest only");
        println!("\nSuggestions would be displayed here for each vulnerability.");
        println!("This requires integration with the full scan pipeline.");
    }

    // 3. Apply fixes if requested
    if apply {
        if interactive {
            println!("\n=== Interactive Fix Application ===");
            println!("Would prompt for confirmation for each fix");
        } else {
            println!("\n=== Automatic Fix Application ===");
            println!("Would apply fixes automatically");
        }

        // Note: Actual implementation would call apply_fixes() or apply_fixes_with_testing()
        println!("Fix application requires integration with build system detection.");
    }

    // 4. Generate PR if requested
    if pr {
        println!("\n=== GitHub PR Generation ===");
        println!("Would create a PR with the applied fixes");
        // Note: Actual implementation would call generate_pr()
        println!("PR generation requires GitHub credentials and repo context.");
    }

    if !suggest && !apply && !pr {
        println!("No action specified. Use --suggest, --apply, or --pr");
        println!("Run 'bazbom fix --help' for more information");
    }

    Ok(())
}
