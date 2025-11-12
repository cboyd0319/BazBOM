// Fix application logic with testing and rollback support

use anyhow::Result;
use bazbom_core::BuildSystem;
use std::path::Path;
use tracing::{info, warn};

use super::build_systems::{
    apply_bazel_fix, apply_bundler_fix, apply_cargo_fix, apply_composer_fix, apply_go_fix,
    apply_gradle_fix, apply_maven_fix, apply_npm_fix, apply_pip_fix,
};
use super::types::{ApplyResult, ApplyResultWithTests, RemediationSuggestion};
use crate::backup::{choose_backup_strategy, BackupHandle};
use crate::test_runner::{has_tests, run_tests};

/// Package manager types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PackageManager {
    Maven,
    Gradle,
    Bazel,
    Npm,
    Pip,
    Go,
    Cargo,
    Bundler,
    Composer,
}

/// Detect package manager from project root
fn detect_package_manager(project_root: &Path) -> Option<PackageManager> {
    // Check for manifest files in priority order
    if project_root.join("package.json").exists() {
        return Some(PackageManager::Npm);
    }
    if project_root.join("Cargo.toml").exists() {
        return Some(PackageManager::Cargo);
    }
    if project_root.join("go.mod").exists() {
        return Some(PackageManager::Go);
    }
    if project_root.join("requirements.txt").exists() {
        return Some(PackageManager::Pip);
    }
    if project_root.join("Gemfile").exists() {
        return Some(PackageManager::Bundler);
    }
    if project_root.join("composer.json").exists() {
        return Some(PackageManager::Composer);
    }
    if project_root.join("pom.xml").exists() {
        return Some(PackageManager::Maven);
    }
    if project_root.join("build.gradle").exists() || project_root.join("build.gradle.kts").exists()
    {
        return Some(PackageManager::Gradle);
    }
    if project_root.join("MODULE.bazel").exists() || project_root.join("WORKSPACE").exists() {
        return Some(PackageManager::Bazel);
    }

    None
}

/// Apply fixes automatically
pub fn apply_fixes(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
) -> Result<ApplyResult> {
    let mut applied = Vec::new();
    let mut failed = Vec::new();

    // Auto-detect package manager (fallback if build_system is Unknown)
    let package_manager = if build_system == BuildSystem::Unknown {
        match detect_package_manager(project_root) {
            Some(pm) => {
                info!(package_manager = ?pm, "Auto-detected package manager");
                pm
            }
            None => {
                anyhow::bail!("Could not detect package manager from project files");
            }
        }
    } else {
        // Map BuildSystem to PackageManager
        match build_system {
            BuildSystem::Maven => PackageManager::Maven,
            BuildSystem::Gradle => PackageManager::Gradle,
            BuildSystem::Bazel => PackageManager::Bazel,
            _ => {
                // Fallback to detection for other build systems
                match detect_package_manager(project_root) {
                    Some(pm) => pm,
                    None => {
                        anyhow::bail!("Could not detect package manager from project files");
                    }
                }
            }
        }
    };

    for suggestion in suggestions {
        if suggestion.fixed_version.is_none() {
            continue;
        }

        let result = match package_manager {
            PackageManager::Maven => apply_maven_fix(suggestion, project_root),
            PackageManager::Gradle => apply_gradle_fix(suggestion, project_root),
            PackageManager::Bazel => apply_bazel_fix(suggestion, project_root),
            PackageManager::Npm => apply_npm_fix(suggestion, project_root),
            PackageManager::Pip => apply_pip_fix(suggestion, project_root),
            PackageManager::Go => apply_go_fix(suggestion, project_root),
            PackageManager::Cargo => apply_cargo_fix(suggestion, project_root),
            PackageManager::Bundler => apply_bundler_fix(suggestion, project_root),
            PackageManager::Composer => apply_composer_fix(suggestion, project_root),
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
