// Ruby dependency updater (Gemfile)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct RubyUpdater;

impl DependencyUpdater for RubyUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut updated_lines = Vec::new();
        let mut found = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.starts_with('#') || trimmed.is_empty() {
                updated_lines.push(line.to_string());
                continue;
            }

            // Parse gem lines: gem 'rails', '~> 6.1.0'
            if trimmed.starts_with("gem ") {
                if let Some(updated_line) = self.update_gem_line(line, package, new_version) {
                    updated_lines.push(updated_line);
                    found = true;
                    continue;
                }
            }

            updated_lines.push(line.to_string());
        }

        if !found {
            anyhow::bail!("Gem {} not found in Gemfile", package);
        }

        fs::write(file_path, updated_lines.join("\n") + "\n")
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in Gemfile: {}", package, new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running bundle install...");

        let output = Command::new("bundle")
            .arg("install")
            .current_dir(project_root)
            .output()
            .context("Failed to execute bundle install")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("bundle install failed:\n{}", stderr);
        }

        println!("  [*] Running bundle update...");

        let output = Command::new("bundle")
            .arg("update")
            .arg("--conservative")
            .current_dir(project_root)
            .output()
            .context("Failed to execute bundle update")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("bundle update failed:\n{}", stderr);
        }

        println!("  [✓] Bundle update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let gemfile_lock = project_root.join("Gemfile.lock");
        if gemfile_lock.exists() {
            Some(gemfile_lock)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "Gemfile"
    }
}

impl RubyUpdater {
    /// Update a gem line if it matches the package
    /// Handles various Gemfile syntax patterns:
    /// - gem 'rails', '~> 6.1.0'
    /// - gem "rails", "~> 6.1.0"
    /// - gem 'rails', '6.1.0'
    /// - gem 'rails'
    fn update_gem_line(&self, line: &str, package: &str, new_version: &str) -> Option<String> {
        let trimmed = line.trim();

        if !trimmed.starts_with("gem ") {
            return None;
        }

        // Extract gem name from the line
        // gem 'rails', '~> 6.1.0'  or  gem "rails", "6.1.0"
        let after_gem = &trimmed[4..]; // Skip "gem "

        // Find the gem name (first quoted string)
        let gem_name = if let Some(start) = after_gem.find('\'') {
            let rest = &after_gem[start + 1..];
            if let Some(end) = rest.find('\'') {
                &rest[..end]
            } else {
                return None;
            }
        } else if let Some(start) = after_gem.find('"') {
            let rest = &after_gem[start + 1..];
            if let Some(end) = rest.find('"') {
                &rest[..end]
            } else {
                return None;
            }
        } else {
            return None;
        };

        if gem_name != package {
            return None;
        }

        // Preserve indentation
        let indent = line.len() - line.trim_start().len();
        let indent_str = " ".repeat(indent);

        // Determine quote style (single or double)
        let quote = if after_gem.contains('\'') { '\'' } else { '"' };

        // Check if version is specified
        if after_gem.contains(',') {
            // Has version specified
            Some(format!("{}gem {}{}{}, {}~> {}{}", indent_str, quote, package, quote, quote, new_version, quote))
        } else {
            // No version specified, add one
            Some(format!("{}gem {}{}{}, {}~> {}{}", indent_str, quote, package, quote, quote, new_version, quote))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_gemfile() {
        let temp_dir = TempDir::new().unwrap();
        let gemfile_path = temp_dir.path().join("Gemfile");

        let content = r#"source 'https://rubygems.org'

gem 'rails', '~> 6.1.0'
gem 'pg', '~> 1.2.3'
gem 'puma', '~> 5.0'
"#;

        fs::write(&gemfile_path, content).unwrap();

        let updater = RubyUpdater;
        updater
            .update_version(&gemfile_path, "rails", "6.1.7")
            .unwrap();

        let updated = fs::read_to_string(&gemfile_path).unwrap();
        assert!(updated.contains("gem 'rails', '~> 6.1.7'"));
        assert!(updated.contains("gem 'pg', '~> 1.2.3'"));
    }

    #[test]
    fn test_update_gem_double_quotes() {
        let temp_dir = TempDir::new().unwrap();
        let gemfile_path = temp_dir.path().join("Gemfile");

        let content = r#"gem "rails", "~> 6.1.0"
"#;

        fs::write(&gemfile_path, content).unwrap();

        let updater = RubyUpdater;
        updater
            .update_version(&gemfile_path, "rails", "6.1.7")
            .unwrap();

        let updated = fs::read_to_string(&gemfile_path).unwrap();
        assert!(updated.contains("gem \"rails\", \"~> 6.1.7\""));
    }

    #[test]
    fn test_gem_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let gemfile_path = temp_dir.path().join("Gemfile");

        let content = r#"gem 'rails', '~> 6.1.0'
"#;

        fs::write(&gemfile_path, content).unwrap();

        let updater = RubyUpdater;
        let result = updater.update_version(&gemfile_path, "nonexistent", "1.0.0");
        assert!(result.is_err());
    }
}
