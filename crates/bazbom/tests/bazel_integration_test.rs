use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
#[ignore] // Run with: cargo test --test bazel_integration_test -- --ignored
fn test_bazel_full_scan_integration() {
    // This test verifies the full Bazel SCA workflow
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    // Create temp output directory
    let temp_dir = std::env::temp_dir().join("bazbom_bazel_test");
    let _ = fs::create_dir_all(&temp_dir);

    // Build the CLI
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .current_dir(&workspace_root)
        .status()
        .expect("Failed to build bazbom CLI");

    assert!(status.success(), "CLI build failed");

    // Run bazbom scan on the workspace
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
        "bazbom scan failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Verify outputs exist
    let spdx_path = temp_dir.join("sbom.spdx.json");
    let deps_path = temp_dir.join("bazel_deps.json");
    let findings_path = temp_dir.join("sca_findings.json");
    let sarif_path = temp_dir.join("sca_findings.sarif");

    assert!(spdx_path.exists(), "SPDX SBOM not generated");
    assert!(deps_path.exists(), "Dependency graph not generated");
    assert!(findings_path.exists(), "Findings JSON not generated");
    assert!(sarif_path.exists(), "SARIF report not generated");

    // Verify SPDX content
    let spdx_content = fs::read_to_string(&spdx_path).expect("Failed to read SPDX");
    let spdx: serde_json::Value = serde_json::from_str(&spdx_content).expect("Invalid SPDX JSON");

    assert_eq!(spdx["spdxVersion"], "SPDX-2.3");
    assert_eq!(spdx["dataLicense"], "CC0-1.0");
    assert!(spdx["packages"].is_array());
    assert!(spdx["relationships"].is_array());

    let packages = spdx["packages"].as_array().unwrap();
    let relationships = spdx["relationships"].as_array().unwrap();

    // Verify we extracted packages (should have guava and dependencies)
    assert!(
        packages.len() >= 3,
        "Expected at least 3 packages, found {}",
        packages.len()
    );

    // Verify we have relationships
    assert!(
        relationships.len() >= 3,
        "Expected at least 3 relationships, found {}",
        relationships.len()
    );

    // Check for DESCRIBES relationships
    let describes_count = relationships
        .iter()
        .filter(|r| r["relationshipType"] == "DESCRIBES")
        .count();
    assert!(describes_count > 0, "No DESCRIBES relationships found");

    // Check for DEPENDS_ON relationships
    let depends_on_count = relationships
        .iter()
        .filter(|r| r["relationshipType"] == "DEPENDS_ON")
        .count();
    assert!(depends_on_count > 0, "No DEPENDS_ON relationships found");

    // Verify dependency graph content
    let deps_content = fs::read_to_string(&deps_path).expect("Failed to read deps JSON");
    let deps: serde_json::Value = serde_json::from_str(&deps_content).expect("Invalid deps JSON");

    let components = deps["components"].as_array().unwrap();
    let edges = deps["edges"].as_array().unwrap();

    assert!(
        components.len() >= 3,
        "Expected at least 3 components, found {}",
        components.len()
    );

    // Verify component structure
    for component in components {
        assert!(component["name"].is_string());
        assert!(component["group"].is_string());
        assert!(component["version"].is_string());
        assert!(component["purl"].is_string());
        assert!(component["sha256"].is_string());
        assert!(component["coordinates"].is_string());
    }

    // Verify edge structure
    for edge in edges {
        assert!(edge["from"].is_string());
        assert!(edge["to"].is_string());
        assert_eq!(edge["type"], "depends_on");
    }

    // Verify PURLs are correctly formatted for any Maven package
    let maven_packages: Vec<_> = components.iter().filter(|c| c["type"] == "maven").collect();

    assert!(
        !maven_packages.is_empty(),
        "No Maven packages found in components"
    );

    for component in maven_packages {
        let purl = component["purl"].as_str().unwrap();
        let name = component["name"].as_str().unwrap();

        assert!(
            purl.starts_with("pkg:maven/"),
            "Invalid PURL format for {}: {}",
            name,
            purl
        );
        assert!(
            purl.contains(name),
            "PURL does not contain package name {}: {}",
            name,
            purl
        );
    }

    // Cleanup - intentionally ignore errors as cleanup failure doesn't affect test validity
    std::mem::drop(fs::remove_dir_all(&temp_dir));

    println!("✓ Bazel integration test passed");
    println!("  - {} packages extracted", packages.len());
    println!("  - {} relationships generated", relationships.len());
    println!("  - {} components in dependency graph", components.len());
    println!("  - {} edges in dependency graph", edges.len());
}
