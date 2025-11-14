use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

// Helper function to create a command with caching disabled
fn bazbom_cmd() -> Command {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.env("BAZBOM_DISABLE_CACHE", "1");
    cmd
}

#[test]
fn shows_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bazbom"));
}

#[test]
fn shows_version() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bazbom"));
}

#[test]
fn scan_writes_stub_outputs() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan")
        .arg(".")
        .arg("--format")
        .arg("spdx")
        .arg("--out-dir")
        .arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("sbom.spdx.json").exists());
    assert!(outdir.join("sca_findings.json").exists());
}

#[test]
fn scan_cyclonedx_format() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan")
        .arg(".")
        .arg("--format")
        .arg("cyclonedx")
        .arg("--out-dir")
        .arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("sbom.cyclonedx.json").exists());
}

#[test]
fn scan_default_format_is_spdx() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan").arg(".").arg("--out-dir").arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("sbom.spdx.json").exists());
}

#[test]
fn scan_default_path_is_current_dir() {
    let mut cmd = bazbom_cmd();
    cmd.arg("scan");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("scan path=."));
}

#[test]
fn scan_with_reachability_flag() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan")
        .arg(".")
        .arg("--reachability")
        .arg("--out-dir")
        .arg(&outdir);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("reachability=true"));
}

#[test]
fn scan_creates_sarif_output() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan").arg(".").arg("--out-dir").arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("sca_findings.sarif").exists());

    // Verify SARIF is valid JSON
    let sarif_content = fs::read_to_string(outdir.join("sca_findings.sarif")).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content).unwrap();
    assert_eq!(sarif["version"], "2.1.0");
}

#[test]
fn policy_check_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("policy").arg("check");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("policy check"));
}

#[test]
fn fix_suggest_command() {
    // Create a minimal sca_findings.json with empty vulnerabilities
    std::fs::write("sca_findings.json", r#"{"vulnerabilities": []}"#)
        .expect("Failed to create test findings file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("fix").arg("--suggest");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No vulnerabilities found"));

    // Cleanup
    let _ = std::fs::remove_file("sca_findings.json");
}

#[test]
fn fix_apply_command() {
    // Create a minimal sca_findings.json with empty vulnerabilities
    std::fs::write("sca_findings.json", r#"{"vulnerabilities": []}"#)
        .expect("Failed to create test findings file");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("fix").arg("--apply");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("No vulnerabilities found"));

    // Cleanup
    let _ = std::fs::remove_file("sca_findings.json");
}

#[test]
fn db_sync_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("db").arg("sync");
    cmd.env("BAZBOM_OFFLINE", "1"); // Set offline mode to avoid network calls
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("db sync"));
}

#[test]
fn scan_outputs_contain_valid_json() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan").arg(".").arg("--out-dir").arg(&outdir);
    cmd.assert().success();

    // Verify SPDX SBOM is valid JSON
    let sbom_content = fs::read_to_string(outdir.join("sbom.spdx.json")).unwrap();
    let sbom: serde_json::Value = serde_json::from_str(&sbom_content).unwrap();
    assert!(sbom.is_object());

    // Verify findings is valid JSON
    let findings_content = fs::read_to_string(outdir.join("sca_findings.json")).unwrap();
    let findings: serde_json::Value = serde_json::from_str(&findings_content).unwrap();
    assert!(findings.is_object());
}

#[test]
fn no_command_defaults_to_scan() {
    let mut cmd = bazbom_cmd();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("scan path=."));
}
