// Fix application logic with testing and rollback support

use anyhow::Result;
use bazbom_core::BuildSystem;
use std::path::Path;
use tracing::{info, warn};

use super::build_systems::{apply_bazel_fix, apply_gradle_fix, apply_maven_fix};
use super::types::{ApplyResult, ApplyResultWithTests, RemediationSuggestion};
use crate::backup::{choose_backup_strategy, BackupHandle};
use crate::test_runner::{has_tests, run_tests};

/// Apply fixes automatically
pub fn apply_fixes(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
) -> Result<ApplyResult> {
    let mut applied = Vec::new();
    let mut failed = Vec::new();

    for suggestion in suggestions {
        if suggestion.fixed_version.is_none() {
            continue;
        }

        let result = match build_system {
            BuildSystem::Maven => apply_maven_fix(suggestion, project_root),
            BuildSystem::Gradle => apply_gradle_fix(suggestion, project_root),
            BuildSystem::Bazel => apply_bazel_fix(suggestion, project_root),
            _ => {
                warn!(build_system = ?build_system, "Unsupported build system for auto-fix");
                continue;
            }
        };

        match result {
            Ok(_) => applied.push(suggestion.vulnerability_id.clone()),
            Err(e) => {
                warn!(
                    package = %suggestion.affected_package,
                    error = %e,
                    "Failed to apply fix"
                );
                failed.push((suggestion.vulnerability_id.clone(), e.to_string()));
            }
        }
    }

    Ok(ApplyResult { applied, failed })
}

/// Apply fixes with testing and automatic rollback on failure
pub fn apply_fixes_with_testing(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
    skip_tests: bool,
) -> Result<ApplyResultWithTests> {
    info!("Creating backup before applying fixes");
    let strategy = choose_backup_strategy(project_root);
    let backup = BackupHandle::create(project_root, strategy)?;

    info!("Applying fixes");
    let apply_result = apply_fixes(suggestions, build_system, project_root)?;

    if apply_result.applied.is_empty() {
        info!("No fixes were applied");
        backup.cleanup()?;
        return Ok(ApplyResultWithTests {
            applied: apply_result.applied,
            failed: apply_result.failed,
            tests_passed: true,
            rollback_performed: false,
        });
    }

    let should_run_tests = !skip_tests && has_tests(build_system, project_root);

    if !should_run_tests {
        info!("Skipping tests");
        backup.cleanup()?;
        return Ok(ApplyResultWithTests {
            applied: apply_result.applied,
            failed: apply_result.failed,
            tests_passed: true,
            rollback_performed: false,
        });
    }

    info!("Running tests to verify fixes");
    let test_result = run_tests(build_system, project_root)?;

    if test_result.success {
        info!(
            duration_secs = test_result.duration.as_secs_f64(),
            "Tests passed! Fixes applied successfully"
        );

        backup.cleanup()?;

        Ok(ApplyResultWithTests {
            applied: apply_result.applied,
            failed: apply_result.failed,
            tests_passed: true,
            rollback_performed: false,
        })
    } else {
        warn!(
            exit_code = test_result.exit_code,
            "Tests failed! Rolling back changes"
        );

        backup.restore()?;

        info!("Changes rolled back successfully");

        anyhow::bail!(
            "Fixes were rolled back because tests failed. \
             Review the test output to understand the issue.\n\nTest output:\n{}",
            test_result.output
        )
    }
}
