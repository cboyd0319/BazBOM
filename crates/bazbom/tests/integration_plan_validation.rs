/// Integration test to validate the BazBOM Integration Plan implementation
/// Based on docs/strategy/product-roadmap/BAZBOM_INTEGRATION_PLAN.md
///
/// This test validates:
/// 1. Directory structure (sbom/, findings/, enrich/, fixes/, publish/)
/// 2. SARIF 2.1.0 compliance
/// 3. Analyzer interfaces
/// 4. Configuration handling
/// 5. Output formats
use anyhow::Result;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_integration_plan_directory_structure() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    // Create minimal config
    let config_content = r#"
[analysis]
cyclonedx = true
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

    // Run orchestrated scan
    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace.clone(),
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: true,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // Validate directory structure per integration plan
    // Section 1: Architecture overview
    assert!(out_dir.exists(), "Output directory should exist");
    assert!(
        out_dir.join("sbom").exists(),
        "sbom/ directory should exist"
    );
    assert!(
        out_dir.join("findings").exists(),
        "findings/ directory should exist"
    );
    assert!(
        out_dir.join("enrich").exists(),
        "enrich/ directory should exist"
    );
    assert!(
        out_dir.join("fixes").exists(),
        "fixes/ directory should exist"
    );

    // Validate SARIF files exist
    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    assert!(merged_sarif.exists(), "merged.sarif should exist");

    let sca_sarif = out_dir.join("findings").join("sca.sarif");
    assert!(sca_sarif.exists(), "sca.sarif should exist");

    Ok(())
}

#[test]
fn test_sarif_2_1_0_compliance() -> Result<()> {
    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    fs::write(workspace.join("bazbom.toml"), "[analysis]\n")?;

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
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // Validate SARIF 2.1.0 structure per integration plan
    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    // Section 1: Data model - SARIF 2.1.0 required
    assert_eq!(
        sarif["version"].as_str(),
        Some("2.1.0"),
        "SARIF version must be 2.1.0 per GitHub requirements"
    );

    // Validate schema reference
    assert!(
        sarif["$schema"].as_str().is_some(),
        "SARIF should have $schema field"
    );

    // Validate runs array exists
    assert!(sarif["runs"].is_array(), "SARIF should have runs array");

    // Each run should have tool.driver with name and version
    if let Some(runs) = sarif["runs"].as_array() {
        for run in runs {
            assert!(
                run["tool"]["driver"]["name"].is_string(),
                "Each run should have tool.driver.name"
            );
            assert!(
                run["tool"]["driver"]["version"].is_string(),
                "Each run should have tool.driver.version"
            );
        }
    }

    Ok(())
}

#[test]
fn test_analyzer_interfaces() -> Result<()> {
    // Validate that all analyzer interfaces are implemented per integration plan
    // Section 1: Architecture overview lists:
    // - SCA (OSV/NVD/GHSA)
    // - Semgrep (optional)
    // - CodeQL (optional)

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    fs::write(workspace.join("bazbom.toml"), "[analysis]\n")?;

    // Test with all analyzers disabled
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
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // SCA should always run (per integration plan: "always on")
    let sca_sarif = out_dir.join("findings").join("sca.sarif");
    assert!(sca_sarif.exists(), "SCA analysis should always run");

    // Semgrep should not run when disabled
    let semgrep_sarif = out_dir.join("findings").join("semgrep.sarif");
    assert!(
        !semgrep_sarif.exists(),
        "Semgrep should not run when disabled"
    );

    // CodeQL should not run when disabled
    let codeql_sarif = out_dir.join("findings").join("codeql.sarif");
    assert!(
        !codeql_sarif.exists(),
        "CodeQL should not run when disabled"
    );

    Ok(())
}

#[test]
fn test_configuration_handling() -> Result<()> {
    // Validate bazbom.toml configuration per integration plan
    // Section 2: CLI & config

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();

    // Create config with all options from integration plan
    let config_content = r#"
[analysis]
cyclonedx = true
semgrep = { enabled = false, ruleset = "curated-jvm@sha256:..." }
codeql = { enabled = false, suite = "default" }

[enrich]
depsdev = true

[autofix]
mode = "dry-run"
recipe_allowlist = ["commons-io", "jackson", "log4j", "spring-core"]

[containers]
strategy = "auto"

[publish]
github_code_scanning = true
artifact = true
"#;
    fs::write(workspace.join("bazbom.toml"), config_content)?;

    // Load and validate config
    let config = bazbom::config::Config::load(&workspace.join("bazbom.toml"))?;

    // Validate analysis section
    assert_eq!(
        config.analysis.cyclonedx,
        Some(true),
        "cyclonedx should be configurable"
    );
    assert_eq!(
        config.analysis.semgrep.as_ref().and_then(|s| s.enabled),
        Some(false),
        "semgrep enabled should be configurable"
    );

    // Validate enrich section
    assert_eq!(
        config.enrich.depsdev,
        Some(true),
        "depsdev should be configurable"
    );

    // Validate autofix section
    assert_eq!(
        config.autofix.mode.as_deref(),
        Some("dry-run"),
        "autofix mode should be configurable"
    );

    // Validate publish section
    assert_eq!(
        config.publish.github_code_scanning,
        Some(true),
        "github_code_scanning should be configurable"
    );
    assert_eq!(
        config.publish.artifact,
        Some(true),
        "artifact should be configurable"
    );

    Ok(())
}

#[test]
fn test_output_formats() -> Result<()> {
    // Validate output formats per integration plan
    // Section 1: Architecture overview specifies:
    // - SPDX 2.3 (always)
    // - CycloneDX 1.5 (optional)
    // - SARIF 2.1.0
    // - CSAF VEX (optional)
    // - CSV (optional)

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    fs::write(
        workspace.join("bazbom.toml"),
        "[analysis]\ncyclonedx = true\n",
    )?;

    let orchestrator = bazbom::scan_orchestrator::ScanOrchestrator::new(
        workspace,
        out_dir.clone(),
        bazbom::scan_orchestrator::ScanOrchestratorOptions {
            cyclonedx: true,
            with_semgrep: false,
            with_codeql: None,
            autofix: None,
            containers: None,
            no_upload: true,
            target: None,
            threat_detection: None,
            incremental: false,
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // SARIF 2.1.0 should always be created
    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    assert!(merged_sarif.exists(), "SARIF 2.1.0 output should exist");

    // Validate SARIF is valid JSON
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let _sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    Ok(())
}

#[test]
fn test_tool_cache_structure() -> Result<()> {
    // Validate tool cache structure per integration plan Appendix A.3
    // Tool cache should store downloaded tools with SHA-256 verification

    use bazbom::context::Context;

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let ctx = Context::new(workspace, out_dir)?;

    // Tool cache should exist
    assert!(
        ctx.tool_cache.exists(),
        "Tool cache directory should be created"
    );

    // Tool cache should be under user's home directory per integration plan
    // ~/.cache/bazbom/tools/<name>/<version>/<os-arch>/
    let cache_path = ctx.tool_cache.to_string_lossy();
    assert!(
        cache_path.contains("bazbom") || cache_path.contains(".cache"),
        "Tool cache should be in standard cache location"
    );

    Ok(())
}

#[test]
fn test_merged_sarif_deduplication() -> Result<()> {
    // Validate SARIF deduplication per integration plan
    // Section 1: "de-duplicates SARIF results"
    // Section 1: "Keep one run per tool in the SARIF runs array"

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    fs::write(workspace.join("bazbom.toml"), "[analysis]\n")?;

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
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // Load merged SARIF
    let merged_sarif = out_dir.join("findings").join("merged.sarif");
    let sarif_content = fs::read_to_string(&merged_sarif)?;
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content)?;

    // Validate runs structure
    if let Some(runs) = sarif["runs"].as_array() {
        // Each run should have a unique tool name
        let mut tool_names = std::collections::HashSet::new();
        for run in runs {
            if let Some(name) = run["tool"]["driver"]["name"].as_str() {
                assert!(
                    tool_names.insert(name.to_string()),
                    "Each tool should appear only once in runs array: {}",
                    name
                );
            }
        }
    }

    Ok(())
}

#[test]
fn test_cli_flags_per_integration_plan() -> Result<()> {
    // Validate CLI flags exist per integration plan Section 2
    // This test documents the expected CLI interface

    use bazbom::cli::{AutofixMode, CodeqlSuite, ContainerStrategy};

    // Validate CodeqlSuite enum
    assert_eq!(CodeqlSuite::Default.as_str(), "default");
    assert_eq!(CodeqlSuite::SecurityExtended.as_str(), "security-extended");

    // Validate AutofixMode enum
    assert_eq!(AutofixMode::Off.as_str(), "off");
    assert_eq!(AutofixMode::DryRun.as_str(), "dry-run");
    assert_eq!(AutofixMode::Pr.as_str(), "pr");

    // Validate ContainerStrategy enum
    assert_eq!(ContainerStrategy::Auto.as_str(), "auto");
    assert_eq!(ContainerStrategy::Syft.as_str(), "syft");
    assert_eq!(ContainerStrategy::Bazbom.as_str(), "bazbom");

    Ok(())
}

#[test]
fn test_enrichment_directory() -> Result<()> {
    // Validate enrichment directory per integration plan
    // Section 1: enrich/depsdev.json

    let temp = tempdir()?;
    let workspace = temp.path().to_path_buf();
    let out_dir = workspace.join("bazbom-output");

    let config_content = "[enrich]\ndepsdev = true\n";
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
            benchmark: false,
            fast: false,
            reachability: false,
            include_cicd: false,
            fetch_checksums: false,
        },
    )?;

    orchestrator.run()?;

    // Enrich directory should exist
    let enrich_dir = out_dir.join("enrich");
    assert!(enrich_dir.exists(), "enrich/ directory should exist");

    // depsdev.json should be created when enrichment is enabled
    let depsdev_file = enrich_dir.join("depsdev.json");
    assert!(
        depsdev_file.exists(),
        "depsdev.json should be created when enrichment is enabled"
    );

    Ok(())
}
