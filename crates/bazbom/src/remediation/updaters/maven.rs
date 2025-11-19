// Maven dependency updater (pom.xml)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct MavenUpdater;

impl DependencyUpdater for MavenUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Parse groupId:artifactId format
        let parts: Vec<&str> = package.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid Maven package format. Expected groupId:artifactId, got: {}", package);
        }
        let group_id = parts[0];
        let artifact_id = parts[1];

        // Use regex to find and update the dependency
        let updated = self.update_dependency_version(&content, group_id, artifact_id, new_version)?;

        fs::write(file_path, updated)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {}:{} in pom.xml: {}", group_id, artifact_id, new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running mvn dependency:resolve...");

        let output = Command::new("mvn")
            .args(["dependency:resolve", "-q"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute mvn dependency:resolve")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("mvn dependency:resolve failed:\n{}", stderr);
        }

        println!("  [✓] Maven dependency update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, _project_root: &Path) -> Option<PathBuf> {
        // Maven doesn't have a traditional lockfile
        None
    }

    fn manifest_name(&self) -> &str {
        "pom.xml"
    }
}

impl MavenUpdater {
    fn update_dependency_version(
        &self,
        content: &str,
        group_id: &str,
        artifact_id: &str,
        new_version: &str,
    ) -> Result<String> {
        let mut result = String::new();
        let mut in_target_dependency = false;
        let mut found_group = false;
        let mut found_artifact = false;
        let mut updated = false;
        let mut depth: u32 = 0;

        for line in content.lines() {
            let trimmed = line.trim();

            // Track dependency blocks
            if trimmed.starts_with("<dependency>") {
                depth += 1;
                in_target_dependency = depth == 1;
                found_group = false;
                found_artifact = false;
            }

            if in_target_dependency {
                // Check for groupId match
                if trimmed.starts_with("<groupId>") && trimmed.contains(group_id) {
                    found_group = true;
                }

                // Check for artifactId match
                if trimmed.starts_with("<artifactId>") && trimmed.contains(artifact_id) {
                    found_artifact = true;
                }

                // Update version if we found our target dependency
                if found_group && found_artifact && trimmed.starts_with("<version>") {
                    let indent = line.len() - line.trim_start().len();
                    let spaces = " ".repeat(indent);
                    result.push_str(&format!("{}<version>{}</version>\n", spaces, new_version));
                    updated = true;

                    // Reset for next dependency
                    in_target_dependency = false;
                    continue;
                }
            }

            if trimmed.starts_with("</dependency>") {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    in_target_dependency = false;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        if !updated {
            anyhow::bail!("Package {}:{} not found in pom.xml", group_id, artifact_id);
        }

        // Remove trailing newline if original didn't have one
        if !content.ends_with('\n') && result.ends_with('\n') {
            result.pop();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_maven_version() {
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <dependencies>
        <dependency>
            <groupId>com.google.guava</groupId>
            <artifactId>guava</artifactId>
            <version>31.0-jre</version>
        </dependency>
    </dependencies>
</project>"#;

        let temp_dir = TempDir::new().unwrap();
        let pom_path = temp_dir.path().join("pom.xml");
        fs::write(&pom_path, pom_content).unwrap();

        let updater = MavenUpdater;
        updater.update_version(&pom_path, "com.google.guava:guava", "32.0-jre").unwrap();

        let updated = fs::read_to_string(&pom_path).unwrap();
        assert!(updated.contains("<version>32.0-jre</version>"));
    }

    #[test]
    fn test_invalid_package_format() {
        let temp_dir = TempDir::new().unwrap();
        let pom_path = temp_dir.path().join("pom.xml");
        fs::write(&pom_path, "<project></project>").unwrap();

        let updater = MavenUpdater;
        let result = updater.update_version(&pom_path, "invalid-format", "1.0.0");
        assert!(result.is_err());
    }
}
