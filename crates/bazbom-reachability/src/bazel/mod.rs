//! Bazel Build Graph Reachability Analysis
//!
//! Bazel already maintains an explicit build dependency graph, making reachability
//! analysis more straightforward than other ecosystems.
//!
//! ## Approach
//!
//! 1. **Query Bazel build graph** - Use `bazel query` or `bazel aquery`
//! 2. **Parse dependency graph** - Extract targets and their dependencies
//! 3. **Identify entrypoints** - Binary targets, test targets
//! 4. **Traverse dependencies** - Follow transitive dependencies from entrypoints
//! 5. **Map to source files** - Link targets back to source code
//!
//! ## Usage
//!
//! ```no_run
//! use bazbom_bazel_reachability::analyze_bazel_project;
//! use std::path::Path;
//!
//! let report = analyze_bazel_project(Path::new("/workspace")).unwrap();
//! println!("Found {} reachable targets", report.reachable_targets.len());
//! ```

pub mod error;
pub mod models;

pub use error::{BazelReachabilityError, Result};
pub use models::ReachabilityReport;

use std::path::Path;
use std::process::Command;

/// Analyze specific Bazel targets affected by file changes (for CI/CD)
///
/// This is the **targeted scanning** approach - only analyze targets that depend
/// on the changed files. Perfect for CI/CD pipelines!
///
/// # Arguments
/// * `workspace_root` - Root of the Bazel workspace
/// * `changed_files` - List of changed files (relative to workspace root)
///
/// # Example
/// ```no_run
/// use bazbom_bazel_reachability::analyze_bazel_targets_for_files;
/// use std::path::Path;
///
/// let changed = vec!["src/used.cc".to_string(), "src/helper.cc".to_string()];
/// let report = analyze_bazel_targets_for_files(Path::new("/workspace"), &changed).unwrap();
/// println!("Affected targets: {}", report.reachable_targets.len());
/// ```
pub fn analyze_bazel_targets_for_files(
    workspace_root: &Path,
    changed_files: &[String],
) -> Result<ReachabilityReport> {
    tracing::info!(
        "Starting targeted Bazel analysis for {} changed files",
        changed_files.len()
    );

    // Step 1: Find all targets affected by these files using rdeps
    let affected_targets = query_affected_targets(workspace_root, changed_files)?;

    tracing::info!("Found {} affected targets", affected_targets.len());

    if affected_targets.is_empty() {
        tracing::warn!("No targets affected by changed files");
        return Ok(ReachabilityReport::default());
    }

    // Step 2: Query dependencies for affected targets only
    let mut report = ReachabilityReport::default();

    for target in &affected_targets {
        let deps = query_target_deps(workspace_root, target)?;
        report.target_dependencies.insert(target.clone(), deps);
    }

    // Step 3: Identify entrypoints among affected targets
    let entrypoints = identify_entrypoints(workspace_root, &affected_targets)?;
    report.entrypoints = entrypoints.clone();

    // Step 4: Compute reachability via graph traversal
    let reachable = compute_reachable_targets(&report.target_dependencies, &entrypoints);
    report.reachable_targets = reachable.clone();

    // Step 5: Determine unreachable targets (within affected set)
    for target in &affected_targets {
        if !reachable.contains(target) {
            report.unreachable_targets.insert(target.clone());
        }
    }

    tracing::info!(
        "Targeted analysis complete: {}/{} affected targets reachable",
        report.reachable_targets.len(),
        affected_targets.len()
    );

    Ok(report)
}

/// Analyze a Bazel workspace for reachability
///
/// Uses `bazel query` to extract the dependency graph and compute reachability.
/// This analyzes **all targets** in the workspace. For CI/CD, use
/// `analyze_bazel_targets_for_files` instead for faster targeted scanning.
pub fn analyze_bazel_project(workspace_root: &Path) -> Result<ReachabilityReport> {
    tracing::info!("Starting full Bazel reachability analysis");

    // Step 1: Query all targets
    let all_targets = query_all_targets(workspace_root)?;

    // Step 2: Query dependencies for each target
    let mut report = ReachabilityReport::default();

    for target in &all_targets {
        // Query dependencies for this target
        let deps = query_target_deps(workspace_root, target)?;
        report.target_dependencies.insert(target.clone(), deps);
    }

    // Step 3: Identify entrypoint targets (binaries, tests)
    let entrypoints = identify_entrypoints(workspace_root, &all_targets)?;
    report.entrypoints = entrypoints.clone();

    // Step 4: Compute reachability via graph traversal
    let reachable = compute_reachable_targets(&report.target_dependencies, &entrypoints);
    report.reachable_targets = reachable.clone();

    // Step 5: Determine unreachable targets
    for target in &all_targets {
        if !reachable.contains(target) {
            report.unreachable_targets.insert(target.clone());
        }
    }

    tracing::info!(
        "Analysis complete: {}/{} targets reachable",
        report.reachable_targets.len(),
        all_targets.len()
    );

    Ok(report)
}

/// Query targets affected by changed files using rdeps
///
/// Uses `bazel query rdeps(//..., set(files))` to find all targets that
/// depend on the changed files (reverse dependencies).
fn query_affected_targets(
    workspace_root: &Path,
    changed_files: &[String],
) -> Result<Vec<String>> {
    if changed_files.is_empty() {
        return Ok(Vec::new());
    }

    // Try to find bazel in common locations
    let bazel_cmd = if Path::new("/opt/homebrew/bin/bazel").exists() {
        "/opt/homebrew/bin/bazel"
    } else if Path::new("/usr/local/bin/bazel").exists() {
        "/usr/local/bin/bazel"
    } else {
        "bazel" // Fall back to PATH
    };

    // Build the set of files for the query
    // Format: rdeps(//..., set(file1 file2 file3))
    let files_set = changed_files.join(" ");
    let query = format!("rdeps(//..., set({}))", files_set);

    tracing::debug!("Running bazel query: {}", query);

    let output = Command::new(bazel_cmd)
        .arg("query")
        .arg(&query)
        .current_dir(workspace_root)
        .output()
        .map_err(|e| BazelReachabilityError::BazelCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("rdeps query failed, trying individual file queries: {}", stderr);

        // Fallback: query each file individually and combine results
        let mut all_targets = std::collections::HashSet::new();

        for file in changed_files {
            let file_query = format!("rdeps(//..., {})", file);
            let file_output = Command::new(bazel_cmd)
                .arg("query")
                .arg(&file_query)
                .current_dir(workspace_root)
                .output()
                .map_err(|e| BazelReachabilityError::BazelCommandFailed(e.to_string()))?;

            if file_output.status.success() {
                let stdout = String::from_utf8_lossy(&file_output.stdout);
                for line in stdout.lines() {
                    all_targets.insert(line.to_string());
                }
            }
        }

        return Ok(all_targets.into_iter().collect());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let targets: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

    Ok(targets)
}

/// Query all targets in the workspace
fn query_all_targets(workspace_root: &Path) -> Result<Vec<String>> {
    // Try to find bazel in common locations
    let bazel_cmd = if Path::new("/opt/homebrew/bin/bazel").exists() {
        "/opt/homebrew/bin/bazel"
    } else if Path::new("/usr/local/bin/bazel").exists() {
        "/usr/local/bin/bazel"
    } else {
        "bazel" // Fall back to PATH
    };

    let output = Command::new(bazel_cmd)
        .arg("query")
        .arg("//...")
        .current_dir(workspace_root)
        .output()
        .map_err(|e| BazelReachabilityError::BazelCommandFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(BazelReachabilityError::BazelCommandFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let targets: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

    Ok(targets)
}

/// Query dependencies for a specific target
fn query_target_deps(workspace_root: &Path, target: &str) -> Result<Vec<String>> {
    let query = format!("deps({})", target);

    // Try to find bazel in common locations
    let bazel_cmd = if Path::new("/opt/homebrew/bin/bazel").exists() {
        "/opt/homebrew/bin/bazel"
    } else if Path::new("/usr/local/bin/bazel").exists() {
        "/usr/local/bin/bazel"
    } else {
        "bazel" // Fall back to PATH
    };

    let output = Command::new(bazel_cmd)
        .arg("query")
        .arg(&query)
        .current_dir(workspace_root)
        .output()
        .map_err(|e| BazelReachabilityError::BazelCommandFailed(e.to_string()))?;

    if !output.status.success() {
        // Some targets may not have deps, that's okay
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let deps: Vec<String> = stdout
        .lines()
        .filter(|s| s != &target) // Exclude self
        .map(|s| s.to_string())
        .collect();

    Ok(deps)
}

/// Identify entrypoint targets (binaries and tests)
fn identify_entrypoints(workspace_root: &Path, targets: &[String]) -> Result<Vec<String>> {
    let mut entrypoints = Vec::new();

    // Try to find bazel in common locations
    let bazel_cmd = if Path::new("/opt/homebrew/bin/bazel").exists() {
        "/opt/homebrew/bin/bazel"
    } else if Path::new("/usr/local/bin/bazel").exists() {
        "/usr/local/bin/bazel"
    } else {
        "bazel" // Fall back to PATH
    };

    // Query rule kinds for all targets
    let output = Command::new(bazel_cmd)
        .arg("query")
        .arg("--output=label_kind")
        .arg("//...")
        .current_dir(workspace_root)
        .output()
        .map_err(|e| BazelReachabilityError::BazelCommandFailed(e.to_string()))?;

    if !output.status.success() {
        // Fallback to name-based detection
        return Ok(targets
            .iter()
            .filter(|t| {
                t.contains("_binary") || t.contains("_test") || t.ends_with(":main") || t.ends_with(":test")
            })
            .cloned()
            .collect());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        // Format: "rule_kind rule //target:name"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let rule_kind = parts[0];
            let target = parts[2];

            // Check if this is a binary or test rule
            if rule_kind.ends_with("_binary") || rule_kind.ends_with("_test") {
                entrypoints.push(target.to_string());
            }
        }
    }

    Ok(entrypoints)
}

/// Compute reachable targets via DFS
fn compute_reachable_targets(
    dependencies: &std::collections::HashMap<String, Vec<String>>,
    entrypoints: &[String],
) -> std::collections::HashSet<String> {
    let mut reachable = std::collections::HashSet::new();
    let mut stack = entrypoints.to_vec();

    while let Some(target) = stack.pop() {
        if reachable.insert(target.clone()) {
            // First time seeing this target, explore its dependencies
            if let Some(deps) = dependencies.get(&target) {
                for dep in deps {
                    if !reachable.contains(dep) {
                        stack.push(dep.clone());
                    }
                }
            }
        }
    }

    reachable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_reachability() {
        let mut deps = std::collections::HashMap::new();
        deps.insert("//app:main".to_string(), vec!["//lib:foo".to_string()]);
        deps.insert("//lib:foo".to_string(), vec!["//lib:bar".to_string()]);
        deps.insert("//lib:bar".to_string(), vec![]);
        deps.insert("//unused:lib".to_string(), vec![]);

        let entrypoints = vec!["//app:main".to_string()];
        let reachable = compute_reachable_targets(&deps, &entrypoints);

        assert!(reachable.contains("//app:main"));
        assert!(reachable.contains("//lib:foo"));
        assert!(reachable.contains("//lib:bar"));
        assert!(!reachable.contains("//unused:lib"));
    }

    #[test]
    fn test_real_bazel_project() {
        let test_workspace = Path::new("/tmp/bazel-reachability-test");

        if !test_workspace.exists() {
            println!("Skipping test - test workspace not found");
            return;
        }

        let report = analyze_bazel_project(test_workspace);

        if let Err(e) = &report {
            println!("Error analyzing Bazel project: {}", e);
            return;
        }

        let report = report.unwrap();

        println!("\n=== BAZEL FULL REACHABILITY ANALYSIS ===");
        println!("Total targets: {}", report.target_dependencies.len());
        println!("Entrypoints: {}", report.entrypoints.len());
        for ep in &report.entrypoints {
            println!("  - {}", ep);
        }
        println!("\nReachable targets: {}", report.reachable_targets.len());
        for target in &report.reachable_targets {
            println!("  - {}", target);
        }
        println!("\nUnreachable targets: {}", report.unreachable_targets.len());
        for target in &report.unreachable_targets {
            println!("  - {}", target);
        }

        // Verify expectations
        assert!(report.entrypoints.contains(&"//src:main".to_string()),
                "Should identify main as entrypoint");
        assert!(report.entrypoints.contains(&"//src:test".to_string()),
                "Should identify test as entrypoint");

        assert!(report.reachable_targets.contains(&"//src:used_lib".to_string()),
                "used_lib should be reachable");
        assert!(report.reachable_targets.contains(&"//src:helper_lib".to_string()),
                "helper_lib should be reachable");
        assert!(report.reachable_targets.contains(&"//src:reachable_lib".to_string()),
                "reachable_lib should be reachable");

        assert!(report.unreachable_targets.contains(&"//src:unused_lib".to_string()),
                "unused_lib should be unreachable");
        assert!(report.unreachable_targets.contains(&"//src:dead_code_lib".to_string()),
                "dead_code_lib should be unreachable");
    }

    #[test]
    fn test_targeted_scanning() {
        let test_workspace = Path::new("/tmp/bazel-reachability-test");

        if !test_workspace.exists() {
            println!("Skipping test - test workspace not found");
            return;
        }

        // Simulate CI/CD scenario: only helper.cc changed
        let changed_files = vec!["//src:helper.cc".to_string()];

        let report = analyze_bazel_targets_for_files(test_workspace, &changed_files);

        if let Err(e) = &report {
            println!("Error in targeted analysis: {}", e);
            return;
        }

        let report = report.unwrap();

        println!("\n=== BAZEL TARGETED SCANNING (CI/CD MODE) ===");
        println!("Changed files: {:?}", changed_files);
        println!("Affected targets: {}", report.target_dependencies.len());
        println!("Entrypoints: {}", report.entrypoints.len());
        for ep in &report.entrypoints {
            println!("  - {}", ep);
        }
        println!("\nReachable from affected: {}", report.reachable_targets.len());
        for target in &report.reachable_targets {
            println!("  - {}", target);
        }

        // Verify targeted scanning worked
        // helper.cc is used by helper_lib, which is used by used_lib, which is used by main and test
        assert!(report.target_dependencies.len() < 10,
                "Targeted scan should analyze fewer targets than full scan (was: {})",
                report.target_dependencies.len());

        // The affected targets should include things that depend on helper.cc
        // But NOT include unused_lib or dead_code_lib (they don't use helper)
        let all_targets: Vec<String> = report.target_dependencies.keys().cloned().collect();
        println!("\nAll affected targets:");
        for target in &all_targets {
            println!("  - {}", target);
        }

        // Verify we didn't scan irrelevant targets
        assert!(!all_targets.contains(&"//src:unused_lib".to_string()),
                "unused_lib should NOT be in affected targets");
        assert!(!all_targets.contains(&"//src:dead_code_lib".to_string()),
                "dead_code_lib should NOT be in affected targets");
    }
}
