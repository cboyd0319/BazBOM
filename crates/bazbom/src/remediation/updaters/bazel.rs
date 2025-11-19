// Bazel dependency updater (maven_install.json / WORKSPACE)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct BazelUpdater;

impl DependencyUpdater for BazelUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let filename = file_path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        if filename == "maven_install.json" {
            self.update_maven_install_json(file_path, package, new_version)
        } else if filename == "WORKSPACE" || filename == "WORKSPACE.bazel" {
            self.update_workspace(file_path, package, new_version)
        } else {
            anyhow::bail!("Unsupported Bazel file: {}", filename)
        }
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running bazel fetch @maven//...");

        let output = Command::new("bazel")
            .args(["fetch", "@maven//..."])
            .current_dir(project_root)
            .output()
            .context("Failed to execute bazel fetch")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("bazel fetch failed:\n{}", stderr);
        }

        println!("  [✓] Bazel dependency update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let maven_install = project_root.join("maven_install.json");
        if maven_install.exists() {
            Some(maven_install)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "WORKSPACE"
    }
}

impl BazelUpdater {
    fn update_maven_install_json(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut json: Value = serde_json::from_str(&content)
            .with_context(|| "Failed to parse maven_install.json")?;

        // Parse groupId:artifactId format
        let parts: Vec<&str> = package.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid package format. Expected groupId:artifactId, got: {}", package);
        }
        let group_id = parts[0];
        let artifact_id = parts[1];

        // Find and update in dependency_tree
        let mut updated = false;
        if let Some(deps) = json.get_mut("dependency_tree") {
            if let Some(dependencies) = deps.get_mut("dependencies") {
                if let Some(deps_array) = dependencies.as_array_mut() {
                    for dep in deps_array.iter_mut() {
                        if let Some(coord) = dep.get("coord").and_then(|c| c.as_str()) {
                            if coord.starts_with(&format!("{}:{}:", group_id, artifact_id)) {
                                // Update version in coord
                                let new_coord = format!("{}:{}:{}", group_id, artifact_id, new_version);
                                if let Some(obj) = dep.as_object_mut() {
                                    obj.insert("coord".to_string(), Value::String(new_coord));
                                    updated = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Also check artifacts array
        if let Some(artifacts) = json.get_mut("artifacts") {
            if let Some(artifacts_obj) = artifacts.as_object_mut() {
                let key = format!("{}:{}", group_id, artifact_id);
                if let Some(artifact) = artifacts_obj.get_mut(&key) {
                    if let Some(obj) = artifact.as_object_mut() {
                        obj.insert("version".to_string(), Value::String(new_version.to_string()));
                        updated = true;
                    }
                }
            }
        }

        if !updated {
            anyhow::bail!("Package {}:{} not found in maven_install.json", group_id, artifact_id);
        }

        let updated_content = serde_json::to_string_pretty(&json)
            .context("Failed to serialize updated JSON")?;

        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {}:{} in maven_install.json: {}", group_id, artifact_id, new_version);
        Ok(())
    }

    fn update_workspace(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Parse groupId:artifactId format
        let parts: Vec<&str> = package.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid package format. Expected groupId:artifactId, got: {}", package);
        }
        let group_id = parts[0];
        let artifact_id = parts[1];

        // Look for maven.artifact("group", "artifact", "version") pattern
        let pattern = format!(
            r#"maven\.artifact\(\s*"{}",\s*"{}",\s*"[^"]+"\s*\)"#,
            regex::escape(group_id),
            regex::escape(artifact_id)
        );

        let re = regex::Regex::new(&pattern)
            .context("Failed to compile regex")?;

        if !re.is_match(&content) {
            anyhow::bail!("Package {}:{} not found in WORKSPACE", group_id, artifact_id);
        }

        let replacement = format!(
            r#"maven.artifact("{}", "{}", "{}")"#,
            group_id, artifact_id, new_version
        );

        let updated = re.replace_all(&content, replacement.as_str()).to_string();

        fs::write(file_path, updated)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {}:{} in WORKSPACE: {}", group_id, artifact_id, new_version);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_maven_install_json() {
        let json_content = r#"{
  "dependency_tree": {
    "dependencies": [
      {
        "coord": "com.google.guava:guava:31.0-jre"
      }
    ]
  },
  "artifacts": {
    "com.google.guava:guava": {
      "version": "31.0-jre"
    }
  }
}"#;

        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("maven_install.json");
        fs::write(&json_path, json_content).unwrap();

        let updater = BazelUpdater;
        updater.update_version(&json_path, "com.google.guava:guava", "32.0-jre").unwrap();

        let updated = fs::read_to_string(&json_path).unwrap();
        assert!(updated.contains("32.0-jre"));
    }

    #[test]
    fn test_update_workspace() {
        let workspace_content = r#"
maven_install(
    artifacts = [
        maven.artifact("com.google.guava", "guava", "31.0-jre"),
    ],
)
"#;

        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path().join("WORKSPACE");
        fs::write(&workspace_path, workspace_content).unwrap();

        let updater = BazelUpdater;
        updater.update_version(&workspace_path, "com.google.guava:guava", "32.0-jre").unwrap();

        let updated = fs::read_to_string(&workspace_path).unwrap();
        assert!(updated.contains("\"32.0-jre\""));
    }
}
