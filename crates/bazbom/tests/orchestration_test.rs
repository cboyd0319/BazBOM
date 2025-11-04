use anyhow::Result;
use std::fs;
use tempfile::tempdir;

/// Integration test for the orchestrated scan flow
#[test]
fn test_orchestrated_scan_creates_output_structure() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create a minimal bazbom.toml config
    let config_content = r#"
[analysis]
cyclonedx = false
semgrep = { enabled = false }

[enrich]
depsdev = false

[autofix]
mode = "off"

[publish]
github_code_scanning = false
artifact = true
"#;
    fs::write(workspace.join("bazbom.toml"), config_content)?;

    // Create the orchestrator
    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace.clone(),
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    // Run the orchestrator
    orchestrator.run()?;

    // Verify output directory structure
    assert!(out_dir.exists(), "Output directory should exist");
    assert!(out_dir.join("sbom").exists(), "SBOM directory should exist");
    assert!(
        out_dir.join("findings").exists(),
        "Findings directory should exist"
    );
    assert!(
        out_dir.join("enrich").exists(),
        "Enrich directory should exist"
    );
    assert!(
        out_dir.join("fixes").exists(),
        "Fixes directory should exist"
    );

    // Verify merged SARIF was created
    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    assert!(merged_sarif.exists(), "Merged SARIF should be created");

    // Parse and validate SARIF structure
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    assert_eq!(
        sarif["version"].as_str(),
        Some("2.1.0"),
        "SARIF version should be 2.1.0"
    );
    assert!(sarif["runs"].is_array(), "SARIF should have runs array");

    Ok(())
}

#[test]
fn test_orchestrated_scan_with_enrichment() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create config with enrichment enabled
    let config_content = r#"
[enrich]
depsdev = true
"#;
    fs::write(workspace.join("bazbom.toml"), config_content)?;

    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    orchestrator.run()?;

    // Verify enrichment output exists
    let enrich_file = out_dir.join("enrich").join("depsdev.json");
    assert!(enrich_file.exists(), "Enrichment file should be created");

    // Verify it's valid JSON
    let enrich_content = fs::read_to_string(&enrich_file)?;
    let _enrich_data: serde_json::Value = serde_json::from_str(&enrich_content)?;

    Ok(())
}

#[test]
fn test_orchestrated_scan_with_autofix() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create config with autofix enabled
    let config_content = r#"
[autofix]
mode = "dry-run"
recipe_allowlist = ["commons-io", "jackson"]
"#;
    fs::write(workspace.join("bazbom.toml"), config_content)?;

    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: Some(bazbom::cli::AutofixMode::DryRun),
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    orchestrator.run()?;

    // Verify fixes output directory exists
    let fixes_dir = out_dir.join("fixes");
    assert!(fixes_dir.exists(), "Fixes directory should exist");

    // In a full implementation with actual vulnerabilities,
    // we would verify recipe files here
    // For now, just check the directory was created

    Ok(())
}

#[test]
fn test_orchestrated_scan_minimal() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // No config file - use all defaults
    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    // Should run successfully with just SCA
    orchestrator.run()?;

    // Verify basic structure exists
    assert!(out_dir.exists());
    assert!(out_dir.join("findings").join("merged.sarif").exists());

    Ok(())
}

#[test]
fn test_merged_sarif_structure() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    orchestrator.run()?;

    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    let content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&content)?;

    // Validate SARIF 2.1.0 structure
    assert_eq!(sarif["version"], "2.1.0");
    assert!(sarif["$schema"].as_str().unwrap().contains("sarif-2.1.0"));
    assert!(sarif["runs"].is_array());

    // Each run should have a tool
    let runs = sarif["runs"].as_array().unwrap();
    for run in runs {
        assert!(run["tool"].is_object());
        assert!(run["tool"]["driver"].is_object());
        assert!(run["tool"]["driver"]["name"].is_string());
        assert!(run["results"].is_array());
    }

    Ok(())
}

#[test]
fn test_output_directories_created() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    orchestrator.run()?;

    // Verify all expected directories exist
    let expected_dirs = vec![
        out_dir.join("sbom"),
        out_dir.join("findings"),
        out_dir.join("enrich"),
        out_dir.join("fixes"),
    ];

    for dir in expected_dirs {
        assert!(dir.exists(), "Directory {:?} should be created", dir);
        assert!(dir.is_dir(), "Path {:?} should be a directory", dir);
    }

    Ok(())
}

#[test]
fn test_tool_cache_directory() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let _orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace.clone(),
        out_dir,
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: false,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
        },
    )?;

    // Verify tool cache directory is created in workspace
    let tool_cache = workspace.join(".bazbom").join("tools");
    assert!(
        tool_cache.exists(),
        "Tool cache directory should be created"
    );

    Ok(())
}
