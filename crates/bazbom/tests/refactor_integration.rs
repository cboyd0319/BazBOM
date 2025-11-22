use assert_fs::prelude::*;
use assert_fs::TempDir;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Helper to get the bazbom binary path
fn bazbom_bin() -> PathBuf {
    // Use the built test binary (debug profile) for portability
    PathBuf::from(assert_cmd::cargo::cargo_bin!("bazbom"))
}

/// Copy a test fixture to a temporary directory
fn copy_fixture(name: &str) -> TempDir {
    let temp = assert_fs::TempDir::new().expect("tempdir");
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    // Copy fixture contents to temp dir
    copy_dir_all(&fixture_path, temp.path()).expect("copy fixture");
    temp
}

/// Recursively copy directory contents
fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> std::io::Result<()> {
    std::fs::create_dir_all(&dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

/// Run bazbom scan on a directory
fn run_scan(dir: &Path) -> std::process::Output {
    Command::new(bazbom_bin())
        .arg("scan")
        .arg(dir)
        .arg("--out-dir")
        .arg(dir)
        .env("BAZBOM_OFFLINE", "1")
        .output()
        .expect("Failed to run bazbom scan")
}

/// Read SBOM JSON from scan output
fn read_sbom(dir: &Path) -> Value {
    let sbom_path = dir.join("sbom").join("spdx.json");
    let content = std::fs::read_to_string(&sbom_path)
        .unwrap_or_else(|_| panic!("Failed to read SBOM at {:?}", sbom_path));
    serde_json::from_str(&content).expect("Failed to parse SBOM JSON")
}

/// Read SARIF JSON from scan output
fn read_sarif(dir: &Path) -> Value {
    let sarif_path = dir.join("findings").join("sca.sarif");
    let content = std::fs::read_to_string(&sarif_path)
        .unwrap_or_else(|_| panic!("Failed to read SARIF at {:?}", sarif_path));
    serde_json::from_str(&content).expect("Failed to parse SARIF JSON")
}

#[cfg(test)]
mod npm_tests {
    use super::*;

    #[test]
    fn test_npm_scan() {
        let fixture = copy_fixture("npm");
        let output = run_scan(fixture.path());

        // Verify scan succeeded
        assert!(output.status.success(), "Scan failed: {:?}", output);

        // Verify SBOM created
        assert!(fixture.child("sbom/spdx.json").exists());

        // Verify SARIF created
        assert!(fixture.child("findings/sca.sarif").exists());

        let sbom = read_sbom(fixture.path());
        let pkg_names: Vec<String> = sbom["packages"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|p| p["name"].as_str().map(|s| s.to_string()))
            .collect();
        assert!(pkg_names.iter().any(|p| p.contains("lodash")));
        assert!(pkg_names.iter().any(|p| p.contains("axios")));

        let sarif = read_sarif(fixture.path());
        assert!(sarif["runs"].is_array());
    }

    #[test]
    fn test_npm_package_count() {
        let fixture = copy_fixture("npm");
        let output = run_scan(fixture.path());

        assert!(output.status.success());

        let sbom = read_sbom(fixture.path());
        let packages = sbom["packages"].as_array().expect("No packages array");

        // npm fixture has 3 direct dependencies + transitive deps
        assert!(
            packages.len() > 3,
            "Expected more than 3 packages, got {}",
            packages.len()
        );
    }

    #[test]
    fn test_npm_vulnerabilities_detected() {
        let fixture = copy_fixture("npm");
        let output = run_scan(fixture.path());

        assert!(output.status.success());

        let sarif = read_sarif(fixture.path());
        let results = sarif["runs"][0]["results"]
            .as_array()
            .expect("No results array");

        // npm fixture uses vulnerable packages (lodash 4.17.15, axios 0.21.1)
        assert!(
            !results.is_empty(),
            "Expected vulnerabilities to be detected"
        );
    }
}

#[cfg(test)]
mod python_tests {
    use super::*;

    #[test]
    fn test_python_scan() {
        let fixture = copy_fixture("python");
        let output = run_scan(fixture.path());

        // Verify scan succeeded
        assert!(output.status.success(), "Scan failed: {:?}", output);

        // Verify SBOM created
        assert!(fixture.child("sbom/spdx.json").exists());

        // Verify SARIF created
        assert!(fixture.child("findings/sca.sarif").exists());

        let sbom = read_sbom(fixture.path());
        let pkg_names: Vec<String> = sbom["packages"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|p| p["name"].as_str().map(|s| s.to_string()))
            .collect();
        assert!(pkg_names.iter().any(|p| p.contains("Django")));
        assert!(pkg_names.iter().any(|p| p.contains("Flask")));

        let sarif = read_sarif(fixture.path());
        assert!(sarif["runs"].is_array());
    }

    #[test]
    fn test_python_package_count() {
        let fixture = copy_fixture("python");
        let output = run_scan(fixture.path());

        assert!(output.status.success());

        let sbom = read_sbom(fixture.path());
        let packages = sbom["packages"].as_array().expect("No packages array");

        // Python fixture has 4 direct dependencies + transitive deps
        assert!(
            packages.len() >= 4,
            "Expected at least 4 packages, got {}",
            packages.len()
        );
    }

    #[test]
    fn test_python_vulnerabilities_detected() {
        let fixture = copy_fixture("python");
        let output = run_scan(fixture.path());

        assert!(output.status.success());

        let sarif = read_sarif(fixture.path());
        let results = sarif["runs"][0]["results"]
            .as_array()
            .expect("No results array");

        // Python fixture uses vulnerable packages (Django 2.2.0, Flask 1.1.1)
        assert!(
            !results.is_empty(),
            "Expected vulnerabilities to be detected"
        );
    }
}
