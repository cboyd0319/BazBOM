// Test execution framework for automated remediation
// Runs project tests after applying fixes to verify changes don't break the build

use anyhow::{Context, Result};
use bazbom_core::BuildSystem;
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
        + &String::from_utf8_lossy(&output.stderr).to_string();
    
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
        .args(&["test", "-DskipTests=false", "--batch-mode"])
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
        .args(&["test", "--no-daemon", "--console=plain"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Gradle tests")
}

fn run_bazel_tests(project_root: &Path) -> Result<Output> {
    println!("[bazbom] Running Bazel tests...");
    
    Command::new("bazel")
        .args(&["test", "//...", "--test_output=errors"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute Bazel tests")
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
            project_root.join("src/test").exists()
                || project_root.join("app/src/test").exists()
        }
        BuildSystem::Bazel => {
            // For Bazel, we'd need to query for test targets
            // For now, assume tests exist if bazel is the build system
            true
        }
        _ => false,
    }
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
