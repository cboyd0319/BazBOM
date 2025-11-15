// Test execution framework for automated remediation
// Runs project tests after applying fixes to verify changes don't break the build
//
// SECURITY NOTE: All test runner functions use HARDCODED arguments to prevent command injection.
// If future modifications allow user-controlled arguments, input validation MUST be added.
// See validate_test_arg() function below for reference implementation.

use anyhow::{bail, Context, Result};
use bazbom_core::BuildSystem;
use bazbom_depsdev::System;
use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

/// Result of running tests
#[derive(Debug)]
pub struct TestResult {
    pub success: bool,
    pub output: String,
    pub duration: Duration,
    pub exit_code: i32,
}

/// Run tests for the given build system
pub fn run_tests(build_system: BuildSystem, project_root: &Path) -> Result<TestResult> {
    let start = Instant::now();

    let output = match build_system {
        BuildSystem::Maven => run_maven_tests(project_root)?,
        BuildSystem::Gradle => run_gradle_tests(project_root)?,
        BuildSystem::Bazel => run_bazel_tests(project_root)?,
        _ => anyhow::bail!("Test execution not supported for {:?}", build_system),
    };

    let duration = start.elapsed();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    let output_text = String::from_utf8_lossy(&output.stdout).to_string()
        + String::from_utf8_lossy(&output.stderr).as_ref();

    Ok(TestResult {
        success,
        output: output_text,
        duration,
        exit_code,
    })
}

/// Run tests for polyglot ecosystems (npm, Python, Go, Rust, etc.)
pub fn run_tests_for_ecosystem(system: System, project_root: &Path) -> Result<TestResult> {
    let start = Instant::now();

    let output = match system {
        System::Maven => run_maven_tests(project_root)?,
        System::Npm => run_npm_tests(project_root)?,
        System::PyPI => run_python_tests(project_root)?,
        System::Go => run_go_tests(project_root)?,
        System::Cargo => run_rust_tests(project_root)?,
        System::RubyGems => run_ruby_tests(project_root)?,
        System::NuGet => run_php_tests(project_root)?, // Placeholder for PHP
    };

    let duration = start.elapsed();
    let exit_code = output.status.code().unwrap_or(-1);
    let success = output.status.success();

    let output_text = String::from_utf8_lossy(&output.stdout).to_string()
        + String::from_utf8_lossy(&output.stderr).as_ref();

    Ok(TestResult {
        success,
        output: output_text,
        duration,
        exit_code,
    })
}

fn run_maven_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Maven tests...");

    Command::new("mvn")
        .args(["test", "-DskipTests=false", "--batch-mode"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Maven tests")
}

fn run_gradle_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Gradle tests...");

    // Try gradlew first, then gradle
    let gradle_cmd = if project_root.join("gradlew").exists() {
        "./gradlew"
    } else if project_root.join("gradlew.bat").exists() {
        "gradlew.bat"
    } else {
        "gradle"
    };

    Command::new(gradle_cmd)
        .args(["test", "--no-daemon", "--console=plain"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Gradle tests")
}

fn run_bazel_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Bazel tests...");

    Command::new("bazel")
        .args(["test", "//...", "--test_output=errors"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Bazel tests")
}

fn run_npm_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running npm tests...");

    Command::new("npm")
        .arg("test")
        .current_dir(project_root)
        .output()
        .context("Failed to execute npm tests")
}

fn run_python_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Python tests...");

    // Try pytest first (most common)
    let pytest_result = Command::new("pytest").current_dir(project_root).output();

    if let Ok(output) = pytest_result {
        Ok(output)
    } else {
        // Fall back to unittest
        Command::new("python")
            .args(["-m", "unittest", "discover"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute Python tests")
    }
}

fn run_go_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Go tests...");

    Command::new("go")
        .args(["test", "./..."])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Go tests")
}

fn run_rust_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Rust tests...");

    Command::new("cargo")
        .arg("test")
        .current_dir(project_root)
        .output()
        .context("Failed to execute Rust tests")
}

fn run_ruby_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Ruby tests...");

    // Try RSpec first
    let rspec_result = Command::new("bundle")
        .args(["exec", "rspec"])
        .current_dir(project_root)
        .output();

    if let Ok(output) = rspec_result {
        Ok(output)
    } else {
        // Fall back to rake test
        Command::new("bundle")
            .args(["exec", "rake", "test"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute Ruby tests")
    }
}

fn run_php_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running PHP tests...");

    // Try vendor/bin/phpunit first
    let phpunit_vendor = project_root.join("vendor/bin/phpunit");

    if phpunit_vendor.exists() {
        Command::new(&phpunit_vendor)
            .current_dir(project_root)
            .output()
            .context("Failed to execute PHPUnit tests")
    } else {
        // Try global phpunit
        Command::new("phpunit")
            .current_dir(project_root)
            .output()
            .context("Failed to execute PHPUnit tests")
    }
}

/// Check if tests are available for the build system
pub fn has_tests(build_system: BuildSystem, project_root: &Path) -> bool {
    match build_system {
        BuildSystem::Maven => {
            // Check for src/test directory
            project_root.join("src/test").exists()
        }
        BuildSystem::Gradle => {
            // Check for src/test or common test directories
            project_root.join("src/test").exists() || project_root.join("app/src/test").exists()
        }
        BuildSystem::Bazel => {
            // For Bazel, we'd need to query for test targets
            // For now, assume tests exist if bazel is the build system
            true
        }
        _ => false,
    }
}

/// Validate test arguments for security (if test args become user-configurable in the future)
///
/// SECURITY: This function should be used to validate ANY user-provided test arguments
/// to prevent command injection. Currently unused but provided as reference implementation.
///
/// # Arguments
/// * `arg` - The argument to validate
///
/// # Returns
/// * `Ok(())` if the argument is safe
/// * `Err` if the argument contains suspicious characters
#[allow(dead_code)]
fn validate_test_arg(arg: &str) -> Result<()> {
    // Reject empty arguments
    if arg.is_empty() {
        bail!("Test argument cannot be empty");
    }

    // Reject arguments that are too long (potential buffer overflow)
    if arg.len() > 256 {
        bail!("Test argument too long: {}", arg.len());
    }

    // Whitelist safe characters: alphanumeric, dash, underscore, equals, dot, slash, colon
    // Reject shell metacharacters: $, `, &, |, ;, <, >, (, ), {, }, [, ], *, ?, !, \, ', "
    for ch in arg.chars() {
        if !ch.is_alphanumeric() && !"-_=./:".contains(ch) {
            bail!("Invalid character in test argument: '{}'", ch);
        }
    }

    // Reject command substitution patterns
    if arg.contains("$(") || arg.contains("`") {
        bail!("Command substitution not allowed in test arguments");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_has_tests_maven() {
        let temp = TempDir::new().unwrap();
        let src_test = temp.path().join("src/test");
        fs::create_dir_all(&src_test).unwrap();

        assert!(has_tests(BuildSystem::Maven, temp.path()));
    }

    #[test]
    fn test_has_tests_no_dir() {
        let temp = TempDir::new().unwrap();
        assert!(!has_tests(BuildSystem::Maven, temp.path()));
    }
}
