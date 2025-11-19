//! Integration layer connecting vulnerability scanning with reachability analysis
//!
//! This module provides the glue code to run reachability analysis on vulnerable
//! packages and enrich vulnerability reports with reachability information.

use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::types::{EcosystemScanResult, ReachabilityData};

/// Perform reachability analysis on vulnerable packages for a scanned ecosystem
pub async fn analyze_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    // Skip if no vulnerabilities found
    if result.vulnerabilities.is_empty() {
        return Ok(());
    }

    match result.ecosystem.as_str() {
        "Node.js/npm" => analyze_js_reachability(result, project_root).await,
        "Python" => analyze_python_reachability(result, project_root).await,
        "Go" => analyze_go_reachability(result, project_root).await,
        "Rust" => analyze_rust_reachability(result, project_root).await,
        "Ruby" => analyze_ruby_reachability(result, project_root).await,
        "PHP" => analyze_php_reachability(result, project_root).await,
        "Maven" => analyze_maven_reachability(result, project_root).await,
        "Gradle" => analyze_gradle_reachability(result, project_root).await,
        _ => Ok(()), // Unknown ecosystem, skip reachability
    }
}

/// Analyze JavaScript/TypeScript reachability
async fn analyze_js_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::js::analyzer::JsReachabilityAnalyzer;

    let mut analyzer = JsReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_root)?;

    let mut vulnerable_packages_reachable = HashMap::new();

    // For each vulnerability, check if it's reachable
    for vuln in &result.vulnerabilities {
        // Extract package name and try to map to vulnerable functions
        // This is a heuristic - in practice, we'd need CVE-to-function mappings
        let package_name = &vuln.package_name;

        // Check if any functions from this package are reachable
        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(&format!("node_modules/{}", package_name)));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Python reachability
async fn analyze_python_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::python::analyzer::PythonReachabilityAnalyzer;

    let mut analyzer = PythonReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_root)?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        // Check if any functions from this package are reachable
        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(&package_name.replace("-", "_")));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Go reachability
async fn analyze_go_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::go::analyzer::GoReachabilityAnalyzer;

    let mut analyzer = GoReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_root)?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        // Check if any functions from this package are reachable
        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(package_name));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Rust reachability
async fn analyze_rust_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::rust::analyzer::RustReachabilityAnalyzer;

    let mut analyzer = RustReachabilityAnalyzer::new(project_root.to_path_buf());
    let report = analyzer.analyze()?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        // Check if any functions from this package are reachable
        // Rust uses crate names with underscores
        let crate_name = package_name.replace("-", "_");
        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(&crate_name));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Ruby reachability
async fn analyze_ruby_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::ruby::analyzer::RubyReachabilityAnalyzer;

    let mut analyzer = RubyReachabilityAnalyzer::new(project_root.to_path_buf());
    let report = analyzer.analyze()?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(package_name));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze PHP reachability
async fn analyze_php_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::php::analyzer::PhpReachabilityAnalyzer;

    let mut analyzer = PhpReachabilityAnalyzer::new(project_root.to_path_buf());
    let report = analyzer.analyze()?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(package_name));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Maven reachability
async fn analyze_maven_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::java::analyzer::JavaReachabilityAnalyzer;

    let mut analyzer = JavaReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_root)?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        // Maven packages are in format "groupId:artifactId"
        // Check if any methods from this package are reachable
        // Convert group.id to group/id for class path matching
        let group_path = if let Some((group_id, _artifact_id)) = package_name.split_once(':') {
            group_id.replace('.', "/")
        } else {
            package_name.replace('.', "/")
        };

        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(&group_path));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

/// Analyze Gradle reachability
async fn analyze_gradle_reachability(
    result: &mut EcosystemScanResult,
    project_root: &Path,
) -> Result<()> {
    use bazbom_reachability::java::analyzer::JavaReachabilityAnalyzer;

    let mut analyzer = JavaReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_root)?;

    let mut vulnerable_packages_reachable = HashMap::new();

    for vuln in &result.vulnerabilities {
        let package_name = &vuln.package_name;

        // Gradle packages are also in format "groupId:artifactId"
        // Check if any methods from this package are reachable
        // Convert group.id to group/id for class path matching
        let group_path = if let Some((group_id, _artifact_id)) = package_name.split_once(':') {
            group_id.replace('.', "/")
        } else {
            package_name.replace('.', "/")
        };

        let is_reachable = report
            .reachable_functions
            .iter()
            .any(|func_id| func_id.contains(&group_path));

        vulnerable_packages_reachable.insert(package_name.clone(), is_reachable);
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reachability_integration() {
        // This is a placeholder test - in practice we'd need test fixtures
        let mut result =
            EcosystemScanResult::new("Node.js/npm".to_string(), "/tmp/test".to_string());

        // Should not crash on empty vulnerabilities
        assert!(analyze_reachability(&mut result, Path::new("/tmp/test"))
            .await
            .is_ok());
    }
}
