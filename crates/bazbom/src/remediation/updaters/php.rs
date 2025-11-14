// PHP dependency updater (composer.json)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PhpUpdater;

impl DependencyUpdater for PhpUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut composer_json: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse composer.json at {}", file_path.display()))?;

        let mut updated = false;

        // Update require dependencies
        if let Some(require) = composer_json["require"].as_object_mut() {
            if require.contains_key(package) {
                // Use caret version constraint (^) which is common in PHP
                require[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        // Update require-dev dependencies
        if let Some(require_dev) = composer_json["require-dev"].as_object_mut() {
            if require_dev.contains_key(package) {
                require_dev[package] = Value::String(format!("^{}", new_version));
                updated = true;
            }
        }

        if !updated {
            anyhow::bail!("Package {} not found in composer.json", package);
        }

        // Write back with pretty formatting
        let formatted = serde_json::to_string_pretty(&composer_json)
            .context("Failed to serialize composer.json")?;

        fs::write(file_path, formatted + "\n")
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!(
            "  [+] Updated {} in composer.json: {}",
            package, new_version
        );
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running composer update...");

        let output = Command::new("composer")
            .arg("update")
            .arg("--no-interaction")
            .current_dir(project_root)
            .output()
            .context("Failed to execute composer update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("composer update failed:\n{}", stderr);
        }

        println!("  [✓] composer update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let composer_lock = project_root.join("composer.lock");
        if composer_lock.exists() {
            Some(composer_lock)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "composer.json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let composer_path = temp_dir.path().join("composer.json");

        let content = r#"{
  "name": "test/project",
  "require": {
    "symfony/symfony": "^5.4.0",
    "doctrine/orm": "^2.12.0"
  },
  "require-dev": {
    "phpunit/phpunit": "^9.5.0"
  }
}"#;

        fs::write(&composer_path, content).unwrap();

        let updater = PhpUpdater;
        updater
            .update_version(&composer_path, "symfony/symfony", "5.4.20")
            .unwrap();

        let updated = fs::read_to_string(&composer_path).unwrap();
        assert!(updated.contains("\"symfony/symfony\": \"^5.4.20\""));
        assert!(updated.contains("\"doctrine/orm\": \"^2.12.0\""));
    }

    #[test]
    fn test_update_dev_dependency() {
        let temp_dir = TempDir::new().unwrap();
        let composer_path = temp_dir.path().join("composer.json");

        let content = r#"{
  "name": "test/project",
  "require-dev": {
    "phpunit/phpunit": "^9.5.0"
  }
}"#;

        fs::write(&composer_path, content).unwrap();

        let updater = PhpUpdater;
        updater
            .update_version(&composer_path, "phpunit/phpunit", "9.6.0")
            .unwrap();

        let updated = fs::read_to_string(&composer_path).unwrap();
        assert!(updated.contains("\"phpunit/phpunit\": \"^9.6.0\""));
    }

    #[test]
    fn test_package_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let composer_path = temp_dir.path().join("composer.json");

        let content = r#"{
  "name": "test/project",
  "require": {
    "symfony/symfony": "^5.4.0"
  }
}"#;

        fs::write(&composer_path, content).unwrap();

        let updater = PhpUpdater;
        let result = updater.update_version(&composer_path, "nonexistent/package", "1.0.0");
        assert!(result.is_err());
    }
}
