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

    // Files are now in subdirectories
    assert!(outdir.join("sbom/spdx.json").exists());
    assert!(outdir.join("findings/sca.sarif").exists());
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

    assert!(outdir.join("sbom/cyclonedx.json").exists());
}

#[test]
fn scan_default_format_is_spdx() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan").arg(".").arg("--out-dir").arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("sbom/spdx.json").exists());
}

#[test]
fn scan_default_path_is_current_dir() {
    let mut cmd = bazbom_cmd();
    cmd.arg("scan");
    // Just verify scan completes successfully
    cmd.assert().success();
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
    // Just verify scan completes successfully with reachability flag
    cmd.assert().success();

    // Verify output files exist
    assert!(outdir.join("sbom/spdx.json").exists());
}

#[test]
fn scan_creates_sarif_output() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = bazbom_cmd();
    cmd.arg("scan").arg(".").arg("--out-dir").arg(&outdir);
    cmd.assert().success();

    assert!(outdir.join("findings/sca.sarif").exists());

    // Verify SARIF is valid JSON
    let sarif_content = fs::read_to_string(outdir.join("findings/sca.sarif")).unwrap();
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
    let tmp = tempdir().unwrap();
    let workdir = tmp.path();

    // Create findings directory with empty SARIF file
    let findings_dir = workdir.join("findings");
    fs::create_dir_all(&findings_dir).unwrap();
    fs::write(
        findings_dir.join("sca.sarif"),
        r#"{"version": "2.1.0", "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json", "runs": [{"tool": {"driver": {"name": "bazbom"}}, "results": []}]}"#
    ).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.current_dir(workdir);
    cmd.arg("fix").arg("--suggest");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0 vulnerabilities"));
}

#[test]
fn fix_apply_command() {
    let tmp = tempdir().unwrap();
    let workdir = tmp.path();

    // Create findings directory with empty SARIF file
    let findings_dir = workdir.join("findings");
    fs::create_dir_all(&findings_dir).unwrap();
    fs::write(
        findings_dir.join("sca.sarif"),
        r#"{"version": "2.1.0", "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json", "runs": [{"tool": {"driver": {"name": "bazbom"}}, "results": []}]}"#
    ).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.current_dir(workdir);
    cmd.arg("fix").arg("--apply");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("0 vulnerabilities"));
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
    let sbom_content = fs::read_to_string(outdir.join("sbom/spdx.json")).unwrap();
    let sbom: serde_json::Value = serde_json::from_str(&sbom_content).unwrap();
    assert!(sbom.is_object());

    // Verify SARIF findings is valid JSON
    let sarif_content = fs::read_to_string(outdir.join("findings/sca.sarif")).unwrap();
    let sarif: serde_json::Value = serde_json::from_str(&sarif_content).unwrap();
    assert!(sarif.is_object());
}

#[test]
fn no_command_defaults_to_scan() {
    let mut cmd = bazbom_cmd();
    // No command should default to scan and succeed
    cmd.assert().success();
}

// =============================================================================
// Threats Command Tests
// =============================================================================

#[test]
fn threats_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("threats").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Threat intelligence"))
        .stdout(predicate::str::contains("typosquatting"))
        .stdout(predicate::str::contains("dependency confusion"));
}

#[test]
fn threats_scan_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("threats").arg("scan").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("typosquatting"))
        .stdout(predicate::str::contains("scorecard"));
}

#[test]
fn threats_scan_basic() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    // Create a minimal SBOM
    let sbom_dir = outdir.join("sbom");
    fs::create_dir_all(&sbom_dir).unwrap();
    fs::write(
        sbom_dir.join("spdx.json"),
        r#"{"packages": [{"name": "lodash", "versionInfo": "4.17.21", "SPDXID": "SPDXRef-Package-lodash-4.17.21"}]}"#,
    )
    .unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.current_dir(tmp.path());
    cmd.arg("threats").arg("scan");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("threat detection"));
}

#[test]
fn threats_configure() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("threats").arg("configure");
    // Configure may produce empty output or configuration info
    cmd.assert().success();
}

// =============================================================================
// Notify Command Tests
// =============================================================================

#[test]
fn notify_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("notify").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Notification configuration"))
        .stdout(predicate::str::contains("Slack"))
        .stdout(predicate::str::contains("Teams"));
}

#[test]
fn notify_configure_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("notify").arg("configure").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("channel"));
}

#[test]
fn notify_test_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("notify").arg("test").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("channel"));
}

#[test]
fn notify_history() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("notify").arg("history");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("notification"));
}

// =============================================================================
// Anomaly Command Tests
// =============================================================================

#[test]
fn anomaly_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("anomaly").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("ML-based anomaly detection"));
}

#[test]
fn anomaly_scan_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("anomaly").arg("scan").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("anomaly detection"));
}

#[test]
fn anomaly_train_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("anomaly").arg("train").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("--from-dir"));
}

#[test]
fn anomaly_report_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("anomaly").arg("report").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("output"));
}

// =============================================================================
// LSP Command Tests
// =============================================================================

#[test]
fn lsp_command() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("lsp");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("BazBOM LSP Server"))
        .stdout(predicate::str::contains("VS CODE SETUP"))
        .stdout(predicate::str::contains("INTELLIJ SETUP"))
        .stdout(predicate::str::contains("NEOVIM SETUP"));
}

// =============================================================================
// Auth Command Tests
// =============================================================================

#[test]
fn auth_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("auth").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Authentication"))
        .stdout(predicate::str::contains("RBAC"));
}

#[test]
fn auth_init_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("auth").arg("init").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Initialize"));
}

#[test]
fn auth_user_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("auth").arg("user").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("remove"));
}

#[test]
fn auth_token_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("auth").arg("token").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("revoke"));
}

#[test]
fn auth_audit_log_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("auth").arg("audit-log").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("limit"));
}
