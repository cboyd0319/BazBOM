use anyhow::Result;
use bazbom::scan_orchestrator::ScanOrchestrator;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_orchestrator_basic_scan() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create a minimal Java file for testing
    let src_dir = workspace.join("src");
    fs::create_dir_all(&src_dir)?;
    fs::write(
        src_dir.join("Example.java"),
        "public class Example { public static void main(String[] args) {} }",
    )?;

    // Run orchestrator with basic options (no external tools required)
    let orchestrator = ScanOrchestrator::new(
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
            benchmark: false,
        },
    )?;

    orchestrator.run()?;

    // Verify output directories were created
    assert!(out_dir.exists());
    assert!(out_dir.join("sbom").exists());
    assert!(out_dir.join("findings").exists());
    assert!(out_dir.join("enrich").exists());
    assert!(out_dir.join("fixes").exists());

    // Verify merged SARIF was created (should at least have SCA)
    let merged_sarif = out_dir.join("findings/merged.sarif");
    assert!(merged_sarif.exists());

    // Verify SARIF is valid JSON
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    // Check SARIF structure
    assert!(sarif.get("version").is_some());
    assert!(sarif.get("runs").is_some());

    let runs = sarif["runs"].as_array().unwrap();
    assert!(!runs.is_empty(), "SARIF should have at least one run (SCA)");

    Ok(())
}

#[test]
fn test_orchestrator_with_config_file() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create a bazbom.toml config file
    let config_content = r#"
[analysis]
cyclonedx = true

[enrich]
depsdev = false

[publish]
github_code_scanning = false
"#;
    fs::write(workspace.join("bazbom.toml"), config_content)?;

    // Run orchestrator - should pick up config from bazbom.toml
    let orchestrator = ScanOrchestrator::new(
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
            benchmark: false,
        },
    )?;

    orchestrator.run()?;

    // Verify it ran successfully
    assert!(out_dir.exists());
    assert!(out_dir.join("findings/merged.sarif").exists());

    Ok(())
}

#[test]
fn test_orchestrator_creates_all_directories() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("custom-output");

    let orchestrator = ScanOrchestrator::new(
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
            benchmark: false,
        },
    )?;

    orchestrator.run()?;

    // Verify all expected directories exist
    assert!(out_dir.join("sbom").exists(), "sbom directory should exist");
    assert!(
        out_dir.join("findings").exists(),
        "findings directory should exist"
    );
    assert!(
        out_dir.join("enrich").exists(),
        "enrich directory should exist"
    );
    assert!(
        out_dir.join("fixes").exists(),
        "fixes directory should exist"
    );

    Ok(())
}

#[test]
fn test_orchestrator_no_upload_flag() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let orchestrator = ScanOrchestrator::new(
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
            benchmark: false,
        },
    )?;

    // Should run without attempting upload
    let result = orchestrator.run();
    assert!(result.is_ok());

    Ok(())
}

#[test]
fn test_orchestrator_merged_sarif_structure() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let orchestrator = ScanOrchestrator::new(
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
            benchmark: false,
        },
    )?;

    orchestrator.run()?;

    let merged_sarif = out_dir.join("findings/merged.sarif");
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    // Verify SARIF 2.1.0 structure
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(
        sarif["$schema"],
        "https://json.schemastore.org/sarif-2.1.0.json"
    );

    let runs = sarif["runs"].as_array().unwrap();
    assert!(!runs.is_empty());

    // Each run should have a tool driver
    for run in runs {
        assert!(run.get("tool").is_some());
        assert!(run["tool"].get("driver").is_some());
        assert!(run["tool"]["driver"].get("name").is_some());
    }

    Ok(())
}
