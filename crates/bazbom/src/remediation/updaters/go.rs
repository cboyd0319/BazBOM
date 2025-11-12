// Go dependency updater (go.mod)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct GoUpdater;

impl DependencyUpdater for GoUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        let mut updated_lines = Vec::new();
        let mut found = false;
        let mut in_require_block = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check if we're entering/exiting a require block
            if trimmed.starts_with("require (") {
                in_require_block = true;
                updated_lines.push(line.to_string());
                continue;
            }

            if in_require_block && trimmed == ")" {
                in_require_block = false;
                updated_lines.push(line.to_string());
                continue;
            }

            // Parse require lines
            if trimmed.starts_with("require ") || in_require_block {
                if let Some(updated_line) = self.update_require_line(line, package, new_version) {
                    updated_lines.push(updated_line);
                    found = true;
                    continue;
                }
            }

            updated_lines.push(line.to_string());
        }

        if !found {
            anyhow::bail!("Package {} not found in go.mod", package);
        }

        fs::write(file_path, updated_lines.join("\n") + "\n")
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in go.mod: {}", package, new_version);
        Ok(())
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running go mod tidy...");

        let output = Command::new("go")
            .args(["mod", "tidy"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute go mod tidy")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("go mod tidy failed:\n{}", stderr);
        }

        println!("  [*] Running go mod download...");

        let output = Command::new("go")
            .args(["mod", "download"])
            .current_dir(project_root)
            .output()
            .context("Failed to execute go mod download")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("go mod download failed:\n{}", stderr);
        }

        println!("  [✓] Go module update completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        let go_sum = project_root.join("go.sum");
        if go_sum.exists() {
            Some(go_sum)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "go.mod"
    }
}

impl GoUpdater {
    /// Update a single require line if it matches the package
    /// Returns Some(updated_line) if updated, None if not matching
    fn update_require_line(
        &self,
        line: &str,
        package: &str,
        new_version: &str,
    ) -> Option<String> {
        let trimmed = line.trim();

        // Parse lines like:
        // require github.com/gin-gonic/gin v1.7.0
        // github.com/gin-gonic/gin v1.7.0 (in require block)

        let parts: Vec<&str> = if trimmed.starts_with("require ") {
            trimmed["require ".len()..].split_whitespace().collect()
        } else {
            trimmed.split_whitespace().collect()
        };

        if parts.is_empty() {
            return None;
        }

        // First part is the module path
        let module_path = parts[0];

        if module_path == package {
            // Normalize version (Go versions start with 'v')
            let normalized_version = if new_version.starts_with('v') {
                new_version.to_string()
            } else {
                format!("v{}", new_version)
            };

            // Preserve indentation
            let indent = line.len() - line.trim_start().len();
            let indent_str = " ".repeat(indent);

            if trimmed.starts_with("require ") {
                return Some(format!("{}require {} {}", indent_str, package, normalized_version));
            } else {
                return Some(format!("{}{} {}", indent_str, package, normalized_version));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_go_mod_single_require() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = r#"module example.com/myapp

go 1.20

require github.com/gin-gonic/gin v1.7.0
require github.com/gorilla/mux v1.8.0
"#;

        fs::write(&go_mod_path, content).unwrap();

        let updater = GoUpdater;
        updater
            .update_version(&go_mod_path, "github.com/gin-gonic/gin", "1.9.0")
            .unwrap();

        let updated = fs::read_to_string(&go_mod_path).unwrap();
        assert!(updated.contains("github.com/gin-gonic/gin v1.9.0"));
        assert!(updated.contains("github.com/gorilla/mux v1.8.0"));
    }

    #[test]
    fn test_update_go_mod_require_block() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = r#"module example.com/myapp

go 1.20

require (
    github.com/gin-gonic/gin v1.7.0
    github.com/gorilla/mux v1.8.0
)
"#;

        fs::write(&go_mod_path, content).unwrap();

        let updater = GoUpdater;
        updater
            .update_version(&go_mod_path, "github.com/gin-gonic/gin", "v1.9.0")
            .unwrap();

        let updated = fs::read_to_string(&go_mod_path).unwrap();
        assert!(updated.contains("github.com/gin-gonic/gin v1.9.0"));
        assert!(updated.contains("github.com/gorilla/mux v1.8.0"));
    }

    #[test]
    fn test_package_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = r#"module example.com/myapp

go 1.20

require github.com/gin-gonic/gin v1.7.0
"#;

        fs::write(&go_mod_path, content).unwrap();

        let updater = GoUpdater;
        let result = updater.update_version(&go_mod_path, "github.com/nonexistent/pkg", "1.0.0");
        assert!(result.is_err());
    }
}
