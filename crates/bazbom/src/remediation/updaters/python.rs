// Python dependency updater (requirements.txt, pyproject.toml, setup.py)

use super::DependencyUpdater;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct PythonUpdater;

impl DependencyUpdater for PythonUpdater {
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()> {
        // Determine which file type we're updating
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        match file_name {
            "requirements.txt" => self.update_requirements_txt(file_path, package, new_version),
            "pyproject.toml" => self.update_pyproject_toml(file_path, package, new_version),
            "setup.py" => self.update_setup_py(file_path, package, new_version),
            _ => anyhow::bail!("Unsupported Python manifest file: {}", file_name),
        }
    }

    fn install(&self, project_root: &Path) -> Result<()> {
        println!("  [*] Running pip install...");

        // Try to find the appropriate manifest file
        let requirements_txt = project_root.join("requirements.txt");
        let pyproject_toml = project_root.join("pyproject.toml");

        let output = if requirements_txt.exists() {
            Command::new("pip")
                .args(["install", "-r", "requirements.txt"])
                .current_dir(project_root)
                .output()
                .context("Failed to execute pip install")?
        } else if pyproject_toml.exists() {
            Command::new("pip")
                .args(["install", "."])
                .current_dir(project_root)
                .output()
                .context("Failed to execute pip install")?
        } else {
            anyhow::bail!("No requirements.txt or pyproject.toml found");
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("pip install failed:\n{}", stderr);
        }

        println!("  [✓] pip install completed successfully");
        Ok(())
    }

    fn lockfile_path(&self, project_root: &Path) -> Option<PathBuf> {
        // Python doesn't have a standard lockfile like npm, but there are tools
        let poetry_lock = project_root.join("poetry.lock");
        let pipfile_lock = project_root.join("Pipfile.lock");
        let pdm_lock = project_root.join("pdm.lock");

        if poetry_lock.exists() {
            Some(poetry_lock)
        } else if pipfile_lock.exists() {
            Some(pipfile_lock)
        } else if pdm_lock.exists() {
            Some(pdm_lock)
        } else {
            None
        }
    }

    fn manifest_name(&self) -> &str {
        "requirements.txt"
    }
}

impl PythonUpdater {
    /// Update requirements.txt file
    fn update_requirements_txt(
        &self,
        file_path: &Path,
        package: &str,
        new_version: &str,
    ) -> Result<()> {
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

            // Parse requirement line (package==version or package>=version, etc.)
            if let Some(pkg_name) = self.extract_package_name(trimmed) {
                if pkg_name.eq_ignore_ascii_case(package) {
                    // Replace version
                    updated_lines.push(format!("{}=={}", package, new_version));
                    found = true;
                    continue;
                }
            }

            updated_lines.push(line.to_string());
        }

        if !found {
            anyhow::bail!("Package {} not found in requirements.txt", package);
        }

        fs::write(file_path, updated_lines.join("\n") + "\n")
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in requirements.txt: {}", package, new_version);
        Ok(())
    }

    /// Update pyproject.toml file
    fn update_pyproject_toml(
        &self,
        file_path: &Path,
        package: &str,
        new_version: &str,
    ) -> Result<()> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read {}", file_path.display()))?;

        // Parse as TOML
        let mut toml: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse pyproject.toml at {}", file_path.display()))?;

        let mut found = false;

        // Update dependencies in [project.dependencies]
        if let Some(dependencies) = toml
            .get_mut("project")
            .and_then(|p| p.get_mut("dependencies"))
            .and_then(|d| d.as_array_mut())
        {
            for dep in dependencies.iter_mut() {
                if let Some(dep_str) = dep.as_str() {
                    if let Some(pkg_name) = self.extract_package_name(dep_str) {
                        if pkg_name.eq_ignore_ascii_case(package) {
                            *dep = toml::Value::String(format!("{}=={}", package, new_version));
                            found = true;
                        }
                    }
                }
            }
        }

        // Update dependencies in [tool.poetry.dependencies]
        if let Some(poetry_deps) = toml
            .get_mut("tool")
            .and_then(|t| t.get_mut("poetry"))
            .and_then(|p| p.get_mut("dependencies"))
            .and_then(|d| d.as_table_mut())
        {
            if poetry_deps.contains_key(package) {
                poetry_deps.insert(package.to_string(), toml::Value::String(new_version.to_string()));
                found = true;
            }
        }

        if !found {
            anyhow::bail!("Package {} not found in pyproject.toml", package);
        }

        let updated_content = toml::to_string(&toml)
            .context("Failed to serialize pyproject.toml")?;

        fs::write(file_path, updated_content)
            .with_context(|| format!("Failed to write to {}", file_path.display()))?;

        println!("  [+] Updated {} in pyproject.toml: {}", package, new_version);
        Ok(())
    }

    /// Update setup.py file (legacy)
    fn update_setup_py(
        &self,
        _file_path: &Path,
        _package: &str,
        _new_version: &str,
    ) -> Result<()> {
        // setup.py is Python code, so it's harder to parse reliably
        // For now, we'll just warn the user
        anyhow::bail!(
            "Automatic update of setup.py is not supported. \
             Please update manually or migrate to pyproject.toml"
        );
    }

    /// Extract package name from requirement specifier
    /// e.g., "django==3.2.0" -> Some("django")
    ///       "requests>=2.28.0" -> Some("requests")
    ///       "pandas[extra]==1.3.0" -> Some("pandas")
    fn extract_package_name(&self, requirement: &str) -> Option<String> {
        let requirement = requirement.trim();

        // Split on comparison operators
        for op in &["==", ">=", "<=", "!=", "~=", ">", "<"] {
            if let Some(idx) = requirement.find(op) {
                let pkg_name = requirement[..idx].trim();
                // Strip extras like [extra] from package name
                if let Some(bracket_idx) = pkg_name.find('[') {
                    return Some(pkg_name[..bracket_idx].trim().to_string());
                }
                return Some(pkg_name.to_string());
            }
        }

        // If no operator, the whole string is the package name (for extras like package[extra])
        if let Some(idx) = requirement.find('[') {
            return Some(requirement[..idx].trim().to_string());
        }

        // No operator found, might be just package name
        if !requirement.is_empty() {
            Some(requirement.to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_update_requirements_txt() {
        let temp_dir = TempDir::new().unwrap();
        let req_path = temp_dir.path().join("requirements.txt");

        let content = "django==3.2.0\nrequests>=2.28.0\nflask==2.0.1\n";
        fs::write(&req_path, content).unwrap();

        let updater = PythonUpdater;
        updater.update_version(&req_path, "django", "3.2.18").unwrap();

        let updated = fs::read_to_string(&req_path).unwrap();
        assert!(updated.contains("django==3.2.18"));
        assert!(updated.contains("requests>=2.28.0"));
        assert!(updated.contains("flask==2.0.1"));
    }

    #[test]
    fn test_extract_package_name() {
        let updater = PythonUpdater;

        assert_eq!(updater.extract_package_name("django==3.2.0"), Some("django".to_string()));
        assert_eq!(updater.extract_package_name("requests>=2.28.0"), Some("requests".to_string()));
        assert_eq!(updater.extract_package_name("flask<=2.0.1"), Some("flask".to_string()));
        assert_eq!(updater.extract_package_name("numpy~=1.20.0"), Some("numpy".to_string()));
        assert_eq!(updater.extract_package_name("pandas[extra]==1.3.0"), Some("pandas".to_string()));
    }

    #[test]
    fn test_package_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let req_path = temp_dir.path().join("requirements.txt");

        let content = "django==3.2.0\n";
        fs::write(&req_path, content).unwrap();

        let updater = PythonUpdater;
        let result = updater.update_version(&req_path, "nonexistent", "1.0.0");
        assert!(result.is_err());
    }
}
