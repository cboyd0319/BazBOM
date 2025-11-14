// Rust dependency updater (Cargo.toml)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RustUpdater;

impl DependencyUpdater for RustUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Parse as TOML
        let mut toml: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse Cargo.toml at {}", file_path.display()))?;

        let mut found = false;

        // Update dependencies in [dependencies]
        if let Some(deps) = toml.get_mut("dependencies").and_then(|d| d.as_table_mut()) {
            if deps.contains_key(package) {
                deps.insert(
                    package.to_string(),
                    toml::Value::String(new_version.to_string()),
                );
                found = true;
            }
        }

        // Update dependencies in [dev-dependencies]
        if let Some(dev_deps) = toml
            .get_mut("dev-dependencies")
            .and_then(|d| d.as_table_mut())
        {
            if dev_deps.contains_key(package) {
                dev_deps.insert(
                    package.to_string(),
                    toml::Value::String(new_version.to_string()),
                );
                found = true;
            }
        }

        // Update dependencies in [build-dependencies]
        if let Some(build_deps) = toml
            .get_mut("build-dependencies")
            .and_then(|d| d.as_table_mut())
        {
            if build_deps.contains_key(package) {
                build_deps.insert(
                    package.to_string(),
                    toml::Value::String(new_version.to_string()),
                );
                found = true;
            }
        }

        if !found {
            anyhow::bail!("Package {} not found in Cargo.toml", package);
        }

        let updated_content = toml::to_string(&toml).context("Failed to serialize Cargo.toml")?;

        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in Cargo.toml: {}", package, new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running cargo update...");

        let output = Command::new("cargo")
            .arg("update")
            .current_dir(project_root)
            .output()
            .context("Failed to execute cargo update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("cargo update failed:\n{}", stderr);
        }

        println!("  [✓] cargo update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let cargo_lock = project_root.join("Cargo.lock");
        if cargo_lock.exists() {
            Some(cargo_lock)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "Cargo.toml"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0.150"
tokio = "1.25.0"

[dev-dependencies]
reqwest = "0.11.14"
"#;

        fs::write(&cargo_toml_path, content).unwrap();

        let updater = RustUpdater;
        updater
            .update_version(&cargo_toml_path, "tokio", "1.29.0")
            .unwrap();

        let updated = fs::read_to_string(&cargo_toml_path).unwrap();
        assert!(updated.contains("tokio = \"1.29.0\""));
        assert!(updated.contains("serde = \"1.0.150\""));
    }

    #[test]
    fn test_update_dev_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"[package]
name = "test-project"
version = "0.1.0"

[dev-dependencies]
reqwest = "0.11.14"
"#;

        fs::write(&cargo_toml_path, content).unwrap();

        let updater = RustUpdater;
        updater
            .update_version(&cargo_toml_path, "reqwest", "0.11.20")
            .unwrap();

        let updated = fs::read_to_string(&cargo_toml_path).unwrap();
        assert!(updated.contains("reqwest = \"0.11.20\""));
    }

    #[test]
    fn test_package_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"[package]
name = "test-project"
version = "0.1.0"

[dependencies]
serde = "1.0.150"
"#;

        fs::write(&cargo_toml_path, content).unwrap();

        let updater = RustUpdater;
        let result = updater.update_version(&cargo_toml_path, "nonexistent", "1.0.0");
        assert!(result.is_err());
    }
}
