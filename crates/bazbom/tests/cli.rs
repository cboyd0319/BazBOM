use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn shows_help() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bazbom"));
}

#[test]
fn scan_writes_stub_outputs() {
    let tmp = tempdir().unwrap();
    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("bazbom"));
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
