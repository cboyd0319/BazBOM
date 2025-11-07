// Fix application logic with testing and rollback support

use anyhow::Result;
use bazbom_core::BuildSystem;
use std::path::Path;

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
                eprintln!("[bazbom] unsupported build system for auto-fix");
                continue;
            }
        };

        match result {
            Ok(_) => applied.push(suggestion.vulnerability_id.clone()),
            Err(e) => {
                eprintln!(
                    "[bazbom] failed to apply fix for {}: {}",
                    suggestion.affected_package, e
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
    println!("\n[bazbom] Creating backup before applying fixes...");
    let strategy = choose_backup_strategy(project_root);
    let backup = BackupHandle::create(project_root, strategy)?;

    println!("\n[bazbom] Applying fixes...");
    let apply_result = apply_fixes(suggestions, build_system, project_root)?;

    if apply_result.applied.is_empty() {
        println!("\n[bazbom] No fixes were applied");
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
        println!("\n[bazbom] Skipping tests");
        backup.cleanup()?;
        return Ok(ApplyResultWithTests {
            applied: apply_result.applied,
            failed: apply_result.failed,
            tests_passed: true,
            rollback_performed: false,
        });
    }

    println!("\n[bazbom] Running tests to verify fixes...");
    let test_result = run_tests(build_system, project_root)?;

    if test_result.success {
        println!("\n[+] Tests passed! Fixes applied successfully.");
        println!("   Duration: {:.2}s", test_result.duration.as_secs_f64());

        backup.cleanup()?;

        Ok(ApplyResultWithTests {
            applied: apply_result.applied,
            failed: apply_result.failed,
            tests_passed: true,
            rollback_performed: false,
        })
    } else {
        println!("\n[X] Tests failed! Rolling back changes...");
        println!("   Exit code: {}", test_result.exit_code);

        backup.restore()?;

        println!("\n[bazbom] Changes rolled back successfully.");
        println!("\nTest output:\n{}", test_result.output);

        anyhow::bail!(
            "Fixes were rolled back because tests failed. \
             Review the test output above to understand the issue."
        )
    }
}
