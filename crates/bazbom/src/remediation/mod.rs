// Remediation automation for BazBOM
// Provides "suggest" and "apply" modes for fixing vulnerabilities
//
// This module has been refactored into focused submodules:
// - types: Data structures (RemediationSuggestion, RemediationReport, etc.)
// - version: Version parsing utilities
// - suggestions: Suggestion generation logic
// - apply: Fix application with testing
// - build_systems: Build system-specific fixers
// - github: GitHub PR generation

pub mod apply;
pub mod build_systems;
pub mod github;
pub mod suggestions;
pub mod types;
pub mod version;

// Re-export commonly used types and functions
pub use apply::{apply_fixes, apply_fixes_with_testing};
pub use github::generate_pr;
pub use suggestions::{enrich_with_depsdev, generate_suggestions};
pub use types::{
    ApplyResult, ApplyResultWithTests, PrConfig, RemediationReport, RemediationSuggestion,
    RemediationSummary,
};
pub use version::parse_semantic_version;

use anyhow::Result;

/// Handle the fix command - temporary stub for refactoring
/// This function will be properly implemented once the full extraction is complete
#[allow(clippy::too_many_arguments)]
pub fn handle_fix_command(
    _suggest: bool,
    _apply: bool,
    _pr: bool,
    _interactive: bool,
    _ml_prioritize: bool,
    _llm: bool,
    _llm_provider: String,
    _llm_model: Option<String>,
) -> Result<()> {
    anyhow::bail!("Fix command temporarily disabled during refactoring")
}
