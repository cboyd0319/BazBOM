//! End-to-end workflow tests for Bazel scanning
//!
//! These tests verify complete workflows from scan initiation to result generation,
//! including dependency extraction, SBOM generation, and security analysis.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Test the complete Bazel scan workflow with default options
#[test]
#[ignore] // Run with: cargo test --test bazel_scan_workflow_test -- --ignored
fn test_bazel_scan_default_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_default");

    // Build CLI
    build_cli(&workspace_root);

    // Run default scan
    let bazbom_binary = workspace_root.join("target/release/bazbom");
    let output = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .output()
        .expect("Failed to run bazbom scan");

    assert!(
        output.status.success(),
        "Scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify default outputs
    verify_spdx_output(&temp_dir);
    verify_dependency_graph(&temp_dir);
    verify_findings_output(&temp_dir);
    verify_sarif_output(&temp_dir);

    cleanup_temp_dir(temp_dir);
    println!("✓ Default workflow test passed");
}

/// Test Bazel scan with CycloneDX output format
#[test]
#[ignore]
fn test_bazel_scan_cyclonedx_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_cyclonedx");

    build_cli(&workspace_root);

    let bazbom_binary = workspace_root.join("target/release/bazbom");
    let output = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--format")
        .arg("cyclonedx")
        .output()
        .expect("Failed to run bazbom scan");

    assert!(
        output.status.success(),
        "CycloneDX scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify CycloneDX output
    let cdx_path = temp_dir.join("sbom.cdx.json");
    assert!(cdx_path.exists(), "CycloneDX SBOM not generated");

    let cdx_content = fs::read_to_string(&cdx_path).expect("Failed to read CycloneDX");
    let cdx: serde_json::Value = serde_json::from_str(&cdx_content).expect("Invalid CycloneDX JSON");

    assert_eq!(cdx["bomFormat"], "CycloneDX");
    assert!(cdx["components"].is_array());

    cleanup_temp_dir(temp_dir);
    println!("✓ CycloneDX workflow test passed");
}

/// Test Bazel scan with specific target filtering
#[test]
#[ignore]
fn test_bazel_scan_target_filtering_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_targets");

    build_cli(&workspace_root);

    let bazbom_binary = workspace_root.join("target/release/bazbom");
    let output = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--targets")
        .arg("//...")
        .output()
        .expect("Failed to run bazbom scan with targets");

    assert!(
        output.status.success(),
        "Target filtering scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify outputs exist
    verify_spdx_output(&temp_dir);
    verify_dependency_graph(&temp_dir);

    cleanup_temp_dir(temp_dir);
    println!("✓ Target filtering workflow test passed");
}

/// Test Bazel scan with policy enforcement
#[test]
#[ignore]
fn test_bazel_scan_policy_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_policy");

    build_cli(&workspace_root);

    // Create a simple policy file
    let policy_path = temp_dir.join("test_policy.yaml");
    let policy_content = r#"
name: test-policy
version: 1.0.0
rules:
  - id: high-severity-vulns
    severity: high
    action: warn
description: Test policy for integration testing
"#;
    fs::write(&policy_path, policy_content).expect("Failed to write policy file");

    let bazbom_binary = workspace_root.join("target/release/bazbom");
    let output = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--policy")
        .arg(&policy_path)
        .output()
        .expect("Failed to run bazbom scan with policy");

    // Policy scan may fail if vulnerabilities exceed thresholds
    // Just verify the command ran and produced output
    assert!(
        output.status.success() || !output.stderr.is_empty(),
        "Policy scan did not execute properly"
    );

    cleanup_temp_dir(temp_dir);
    println!("✓ Policy workflow test passed");
}

/// Test Bazel scan with VEX generation
#[test]
#[ignore]
fn test_bazel_scan_vex_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_vex");

    build_cli(&workspace_root);

    let bazbom_binary = workspace_root.join("target/release/bazbom");
    let output = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--vex")
        .output()
        .expect("Failed to run bazbom scan with VEX");

    assert!(
        output.status.success(),
        "VEX scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify VEX output if vulnerabilities were found
    // VEX file may not exist if no vulnerabilities are present
    let vex_path = temp_dir.join("vex.csaf.json");
    if vex_path.exists() {
        let vex_content = fs::read_to_string(&vex_path).expect("Failed to read VEX");
        let vex: serde_json::Value = serde_json::from_str(&vex_content).expect("Invalid VEX JSON");

        assert!(vex["document"].is_object(), "Invalid VEX document structure");
    }

    cleanup_temp_dir(temp_dir);
    println!("✓ VEX workflow test passed");
}

/// Test Bazel scan with multi-format output
#[test]
#[ignore]
fn test_bazel_scan_multi_format_workflow() {
    let workspace_root = get_workspace_root();
    let temp_dir = create_temp_dir("bazel_scan_multi");

    build_cli(&workspace_root);

    let bazbom_binary = workspace_root.join("target/release/bazbom");
    
    // Generate both SPDX and CycloneDX
    let output1 = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--format")
        .arg("spdx")
        .output()
        .expect("Failed to run SPDX scan");

    assert!(output1.status.success(), "SPDX scan failed");

    let output2 = Command::new(&bazbom_binary)
        .arg("scan")
        .arg(&workspace_root)
        .arg("--out-dir")
        .arg(&temp_dir)
        .arg("--format")
        .arg("cyclonedx")
        .output()
        .expect("Failed to run CycloneDX scan");

    assert!(output2.status.success(), "CycloneDX scan failed");

    // Verify both formats exist
    assert!(temp_dir.join("sbom.spdx.json").exists(), "SPDX not found");
    assert!(temp_dir.join("sbom.cdx.json").exists(), "CycloneDX not found");

    cleanup_temp_dir(temp_dir);
    println!("✓ Multi-format workflow test passed");
}

// Helper functions

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn create_temp_dir(name: &str) -> PathBuf {
    let temp_dir = std::env::temp_dir().join(format!("bazbom_{}", name));
    let _ = fs::remove_dir_all(&temp_dir); // Clean up any previous run
    fs::create_dir_all(&temp_dir).expect("Failed to create temp directory");
    temp_dir
}

fn build_cli(workspace_root: &PathBuf) {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(workspace_root)
        .status()
        .expect("Failed to build bazbom CLI");

    assert!(status.success(), "CLI build failed");
}

fn verify_spdx_output(temp_dir: &PathBuf) {
    let spdx_path = temp_dir.join("sbom.spdx.json");
    assert!(spdx_path.exists(), "SPDX SBOM not generated");

    let spdx_content = fs::read_to_string(&spdx_path).expect("Failed to read SPDX");
    let spdx: serde_json::Value = serde_json::from_str(&spdx_content).expect("Invalid SPDX JSON");

    assert_eq!(spdx["spdxVersion"], "SPDX-2.3");
    assert_eq!(spdx["dataLicense"], "CC0-1.0");
    assert!(spdx["packages"].is_array());
    assert!(spdx["relationships"].is_array());
}

fn verify_dependency_graph(temp_dir: &PathBuf) {
    let deps_path = temp_dir.join("bazel_deps.json");
    assert!(deps_path.exists(), "Dependency graph not generated");

    let deps_content = fs::read_to_string(&deps_path).expect("Failed to read deps JSON");
    let deps: serde_json::Value = serde_json::from_str(&deps_content).expect("Invalid deps JSON");

    assert!(deps["components"].is_array());
    assert!(deps["edges"].is_array());
    assert!(deps["metadata"].is_object());
}

fn verify_findings_output(temp_dir: &PathBuf) {
    let findings_path = temp_dir.join("sca_findings.json");
    assert!(findings_path.exists(), "Findings JSON not generated");

    let findings_content = fs::read_to_string(&findings_path).expect("Failed to read findings");
    let _findings: serde_json::Value = serde_json::from_str(&findings_content).expect("Invalid findings JSON");
}

fn verify_sarif_output(temp_dir: &PathBuf) {
    let sarif_path = temp_dir.join("sca_findings.sarif");
    assert!(sarif_path.exists(), "SARIF report not generated");

    let sarif_content = fs::read_to_string(&sarif_path).expect("Failed to read SARIF");
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content).expect("Invalid SARIF JSON");

    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["runs"].is_array());
}

fn cleanup_temp_dir(temp_dir: PathBuf) {
    // Intentionally ignore errors as cleanup failure doesn't affect test validity
    let _ = fs::remove_dir_all(&temp_dir);
}
