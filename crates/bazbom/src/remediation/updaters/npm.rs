// npm dependency updater

use super::DependencyUpdater;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct NpmUpdater;

impl DependencyUpdater for NpmUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut pkg_json: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse package.json at {}", file_path.display()))?;

        let mut updated = false;

        // Update dependencies
        if let Some(deps) = pkg_json["dependencies"].as_object_mut() {
            if deps.contains_key(package) {
                deps[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        // Update devDependencies
        if let Some(dev_deps) = pkg_json["devDependencies"].as_object_mut() {
            if dev_deps.contains_key(package) {
                dev_deps[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        // Update peerDependencies
        if let Some(peer_deps) = pkg_json["peerDependencies"].as_object_mut() {
            if peer_deps.contains_key(package) {
                peer_deps[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        // Update optionalDependencies
        if let Some(opt_deps) = pkg_json["optionalDependencies"].as_object_mut() {
            if opt_deps.contains_key(package) {
                opt_deps[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        if !updated {
            anyhow::bail!("Package {} not found in package.json dependencies", package);
        }

        // Write back with pretty formatting
        let formatted =
            serde_json::to_string_pretty(&pkg_json).context("Failed to serialize package.json")?;

        fs::write(file_path, formatted + "\n")
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in package.json: {}", package, new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running npm install...");

        let output = Command::new("npm")
            .arg("install")
            .current_dir(project_root)
            .output()
            .context("Failed to execute npm install")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("npm install failed:\n{}", stderr);
        }

        println!("  [✓] npm install completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let npm_lock = project_root.join("package-lock.json");
        let yarn_lock = project_root.join("yarn.lock");
        let pnpm_lock = project_root.join("pnpm-lock.yaml");

        if npm_lock.exists() {
            Some(npm_lock)
        } else if yarn_lock.exists() {
            Some(yarn_lock)
        } else if pnpm_lock.exists() {
            Some(pnpm_lock)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "package.json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_npm_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let content = r#"{
  "name": "test-project",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.17.0",
    "lodash": "^4.17.20"
  },
  "devDependencies": {
    "jest": "^27.0.0"
  }
}"#;

        fs::write(&package_json_path, content).unwrap();

        let updater = NpmUpdater;
        updater
            .update_version(&package_json_path, "express", "4.18.0")
            .unwrap();

        let updated_content = fs::read_to_string(&package_json_path).unwrap();
        assert!(updated_content.contains("\"express\": \"^4.18.0\""));
        assert!(updated_content.contains("\"lodash\": \"^4.17.20\""));
    }

    #[test]
    fn test_update_dev_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let content = r#"{
  "name": "test-project",
  "devDependencies": {
    "jest": "^27.0.0"
  }
}"#;

        fs::write(&package_json_path, content).unwrap();

        let updater = NpmUpdater;
        updater
            .update_version(&package_json_path, "jest", "29.0.0")
            .unwrap();

        let updated_content = fs::read_to_string(&package_json_path).unwrap();
        assert!(updated_content.contains("\"jest\": \"^29.0.0\""));
    }

    #[test]
    fn test_package_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let package_json_path = temp_dir.path().join("package.json");

        let content = r#"{
  "name": "test-project",
  "dependencies": {}
}"#;

        fs::write(&package_json_path, content).unwrap();

        let updater = NpmUpdater;
        let result = updater.update_version(&package_json_path, "nonexistent", "1.0.0");
        assert!(result.is_err());
    }
}
