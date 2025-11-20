// Gradle dependency updater (build.gradle / build.gradle.kts)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GradleUpdater;

impl DependencyUpdater for GradleUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Parse groupId:artifactId format
        let parts: Vec<&str> = package.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid package format. Expected groupId:artifactId, got: {}", package);
        }
        let group_id = parts[0];
        let artifact_id = parts[1];

        let is_kotlin = file_path.extension().is_some_and(|ext| ext == "kts");
        let updated = if is_kotlin {
            self.update_kotlin_dsl(&content, group_id, artifact_id, new_version)?
        } else {
            self.update_groovy_dsl(&content, group_id, artifact_id, new_version)?
        };

        fs::write(file_path, updated)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {}:{} in {}: {}", group_id, artifact_id,
            file_path.file_name().unwrap_or_default().to_string_lossy(), new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        // Try gradlew first, then gradle
        let gradle_cmd = if project_root.join("gradlew").exists() {
            "./gradlew"
        } else {
            "gradle"
        };

        println!("  [*] Running {} dependencies...", gradle_cmd);

        let output = Command::new(gradle_cmd)
            .args(["dependencies", "--refresh-dependencies", "-q"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute gradle dependencies")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("gradle dependencies failed:\n{}", stderr);
        }

        println!("  [✓] Gradle dependency update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let lockfile = project_root.join("gradle.lockfile");
        if lockfile.exists() {
            Some(lockfile)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "build.gradle"
    }
}

impl GradleUpdater {
    fn update_groovy_dsl(
        &self,
        content: &str,
        group_id: &str,
        artifact_id: &str,
        new_version: &str,
    ) -> Result<String> {
        let escaped_group = Self::escape_regex(group_id);
        let escaped_artifact = Self::escape_regex(artifact_id);

        // Try single quotes first: 'group:artifact:version'
        let single_quote_pattern = format!(
            r#"'{}:{}:[^']+'"#,
            escaped_group,
            escaped_artifact
        );

        let single_re = regex::Regex::new(&single_quote_pattern)
            .context("Failed to compile regex")?;

        if single_re.is_match(content) {
            let replacement = format!("'{}:{}:{}'", group_id, artifact_id, new_version);
            return Ok(single_re.replace_all(content, replacement.as_str()).to_string());
        }

        // Try double quotes: "group:artifact:version"
        let double_quote_pattern = format!(
            r#""{}:{}:[^"]+""#,
            escaped_group,
            escaped_artifact
        );

        let double_re = regex::Regex::new(&double_quote_pattern)
            .context("Failed to compile regex")?;

        if double_re.is_match(content) {
            let replacement = format!("\"{}:{}:{}\"", group_id, artifact_id, new_version);
            return Ok(double_re.replace_all(content, replacement.as_str()).to_string());
        }

        // Try alternate format: implementation group: 'x', name: 'y', version: 'z'
        self.update_map_style(content, group_id, artifact_id, new_version)
    }

    fn update_kotlin_dsl(
        &self,
        content: &str,
        group_id: &str,
        artifact_id: &str,
        new_version: &str,
    ) -> Result<String> {
        // Pattern for: implementation("group:artifact:version")
        let escaped_group = Self::escape_regex(group_id);
        let escaped_artifact = Self::escape_regex(artifact_id);
        let pattern = format!(
            r#"\("{}:{}:[^"]+"\)"#,
            escaped_group,
            escaped_artifact
        );

        let re = regex::Regex::new(&pattern)
            .context("Failed to compile regex")?;

        if !re.is_match(content) {
            anyhow::bail!("Package {}:{} not found in build.gradle.kts", group_id, artifact_id);
        }

        let replacement = format!("(\"{}:{}:{}\")", group_id, artifact_id, new_version);
        let updated = re.replace_all(content, replacement.as_str()).to_string();

        Ok(updated)
    }

    fn escape_regex(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '\\' => {
                    result.push('\\');
                    result.push(c);
                }
                _ => result.push(c),
            }
        }
        result
    }

    fn update_map_style(
        &self,
        content: &str,
        group_id: &str,
        artifact_id: &str,
        new_version: &str,
    ) -> Result<String> {
        // Pattern for: group: 'x', name: 'y', version: 'z'
        let escaped_group = Self::escape_regex(group_id);
        let escaped_artifact = Self::escape_regex(artifact_id);
        let pattern = format!(
            r#"group:\s*['"]{}['"],\s*name:\s*['"]{}['"],\s*version:\s*['"][^'"]+['"]"#,
            escaped_group,
            escaped_artifact
        );

        let re = regex::Regex::new(&pattern)
            .context("Failed to compile regex")?;

        if !re.is_match(content) {
            anyhow::bail!("Package {}:{} not found in build.gradle", group_id, artifact_id);
        }

        let replacement = format!(
            "group: '{}', name: '{}', version: '{}'",
            group_id, artifact_id, new_version
        );
        let updated = re.replace_all(content, replacement.as_str()).to_string();

        Ok(updated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_update_groovy_string_notation() {
        let gradle_content = r#"
dependencies {
    implementation 'com.google.guava:guava:31.0-jre'
    testImplementation 'junit:junit:4.13'
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let gradle_path = temp_dir.path().join("build.gradle");
        fs::write(&gradle_path, gradle_content).unwrap();

        let updater = GradleUpdater;
        updater.update_version(&gradle_path, "com.google.guava:guava", "32.0-jre").unwrap();

        let updated = fs::read_to_string(&gradle_path).unwrap();
        assert!(updated.contains("'com.google.guava:guava:32.0-jre'"));
    }

    #[test]
    fn test_update_kotlin_dsl() {
        let gradle_content = r#"
dependencies {
    implementation("com.google.guava:guava:31.0-jre")
}
"#;

        let temp_dir = TempDir::new().unwrap();
        let gradle_path = temp_dir.path().join("build.gradle.kts");
        fs::write(&gradle_path, gradle_content).unwrap();

        let updater = GradleUpdater;
        updater.update_version(&gradle_path, "com.google.guava:guava", "32.0-jre").unwrap();

        let updated = fs::read_to_string(&gradle_path).unwrap();
        assert!(updated.contains("(\"com.google.guava:guava:32.0-jre\")"));
    }
}
