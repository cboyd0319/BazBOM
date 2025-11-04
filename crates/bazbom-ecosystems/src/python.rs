//! Python/pip ecosystem support

use anyhow::{Context, Result};
use crate::{Dependency, DependencyGraph, DependencyScope, EcosystemMetadata, EcosystemPlugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Python ecosystem plugin
pub struct PythonPlugin;

impl PythonPlugin {
    /// Create a new Python plugin
    pub fn new() -> Self {
        Self
    }

    /// Parse requirements.txt
    fn parse_requirements_txt(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read requirements.txt from {}", path.display()))?;

        let mut graph = DependencyGraph::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse package==version format
            if let Some((name, version)) = self.parse_requirement_line(line) {
                let dependency = Dependency {
                    name: name.clone(),
                    version: version.clone(),
                    scope: DependencyScope::Runtime,
                    ecosystem: "python".to_string(),
                    purl: Some(format!("pkg:pypi/{}@{}", name, version)),
                    direct: true,
                };
                graph.add_dependency(dependency);
            }
        }

        Ok(graph)
    }

    /// Parse a single requirement line
    fn parse_requirement_line(&self, line: &str) -> Option<(String, String)> {
        // Handle common formats:
        // package==version
        // package>=version
        // package~=version
        // package
        
        // Remove inline comments
        let line = line.split('#').next()?.trim();
        
        if line.contains("==") {
            let parts: Vec<&str> = line.split("==").collect();
            if parts.len() == 2 {
                return Some((parts[0].trim().to_string(), parts[1].trim().to_string()));
            }
        } else if line.contains(">=") {
            let parts: Vec<&str> = line.split(">=").collect();
            if parts.len() == 2 {
                return Some((parts[0].trim().to_string(), parts[1].trim().to_string()));
            }
        } else if line.contains("~=") {
            let parts: Vec<&str> = line.split("~=").collect();
            if parts.len() == 2 {
                return Some((parts[0].trim().to_string(), parts[1].trim().to_string()));
            }
        } else {
            // No version specified - use "latest"
            return Some((line.to_string(), "latest".to_string()));
        }
        
        None
    }

    /// Parse Pipfile.lock
    fn parse_pipfile_lock(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read Pipfile.lock from {}", path.display()))?;

        let lock: PipfileLock = serde_json::from_str(&content)
            .context("Failed to parse Pipfile.lock")?;

        let mut graph = DependencyGraph::new();

        // Add default (runtime) dependencies
        if let Some(default_deps) = lock.default {
            for (name, info) in default_deps {
                let version = info.version.trim_start_matches("==").to_string();
                let dependency = Dependency {
                    name: name.clone(),
                    version,
                    scope: DependencyScope::Runtime,
                    ecosystem: "python".to_string(),
                    purl: Some(format!("pkg:pypi/{}@{}", name, info.version.trim_start_matches("=="))),
                    direct: true,
                };
                graph.add_dependency(dependency);
            }
        }

        // Add develop (development) dependencies
        if let Some(develop_deps) = lock.develop {
            for (name, info) in develop_deps {
                let version = info.version.trim_start_matches("==").to_string();
                let dependency = Dependency {
                    name: name.clone(),
                    version,
                    scope: DependencyScope::Development,
                    ecosystem: "python".to_string(),
                    purl: Some(format!("pkg:pypi/{}@{}", name, info.version.trim_start_matches("=="))),
                    direct: true,
                };
                graph.add_dependency(dependency);
            }
        }

        Ok(graph)
    }

    /// Parse poetry.lock
    fn parse_poetry_lock(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read poetry.lock from {}", path.display()))?;

        // poetry.lock is in TOML format
        // For simplicity, we'll parse it as text for now
        let mut graph = DependencyGraph::new();
        
        let mut current_package: Option<(String, String)> = None;
        let mut is_dev = false;

        for line in content.lines() {
            let line = line.trim();
            
            // Parse package name
            if line.starts_with("name = \"") {
                let name = line.trim_start_matches("name = \"")
                    .trim_end_matches('"')
                    .to_string();
                if let Some((pkg_name, pkg_version)) = current_package.take() {
                    // Save previous package
                    let dependency = Dependency {
                        name: pkg_name.clone(),
                        version: pkg_version.clone(),
                        scope: if is_dev { DependencyScope::Development } else { DependencyScope::Runtime },
                        ecosystem: "python".to_string(),
                        purl: Some(format!("pkg:pypi/{}@{}", pkg_name, pkg_version)),
                        direct: true,
                    };
                    graph.add_dependency(dependency);
                }
                current_package = Some((name, String::new()));
                is_dev = false;
            }
            
            // Parse version
            if line.starts_with("version = \"") {
                let version = line.trim_start_matches("version = \"")
                    .trim_end_matches('"')
                    .to_string();
                if let Some((name, _)) = current_package.as_mut() {
                    current_package = Some((name.clone(), version));
                }
            }
            
            // Check if dev dependency
            if line.contains("category = \"dev\"") {
                is_dev = true;
            }
        }
        
        // Add last package
        if let Some((name, version)) = current_package {
            let dependency = Dependency {
                name: name.clone(),
                version: version.clone(),
                scope: if is_dev { DependencyScope::Development } else { DependencyScope::Runtime },
                ecosystem: "python".to_string(),
                purl: Some(format!("pkg:pypi/{}@{}", name, version)),
                direct: true,
            };
            graph.add_dependency(dependency);
        }

        Ok(graph)
    }
}

impl EcosystemPlugin for PythonPlugin {
    fn name(&self) -> &str {
        "python"
    }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("requirements.txt").exists()
            || project_root.join("Pipfile").exists()
            || project_root.join("pyproject.toml").exists()
            || project_root.join("setup.py").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Try Pipfile.lock first (most accurate)
        let pipfile_lock = project_root.join("Pipfile.lock");
        if pipfile_lock.exists() {
            return self.parse_pipfile_lock(&pipfile_lock);
        }

        // Try poetry.lock
        let poetry_lock = project_root.join("poetry.lock");
        if poetry_lock.exists() {
            return self.parse_poetry_lock(&poetry_lock);
        }

        // Fallback to requirements.txt
        let requirements_txt = project_root.join("requirements.txt");
        if requirements_txt.exists() {
            return self.parse_requirements_txt(&requirements_txt);
        }

        anyhow::bail!("No supported Python dependency file found in {}", project_root.display());
    }

    fn get_metadata(&self) -> EcosystemMetadata {
        EcosystemMetadata {
            display_name: "Python".to_string(),
            registry_url: Some("https://pypi.org".to_string()),
            lockfile_names: vec![
                "Pipfile.lock".to_string(),
                "poetry.lock".to_string(),
                "requirements.txt".to_string(),
            ],
            manifest_names: vec![
                "Pipfile".to_string(),
                "pyproject.toml".to_string(),
                "setup.py".to_string(),
            ],
        }
    }
}

impl Default for PythonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// Pipfile.lock structure
#[derive(Debug, Deserialize, Serialize)]
struct PipfileLock {
    _meta: Option<serde_json::Value>,
    default: Option<HashMap<String, PipfilePackage>>,
    develop: Option<HashMap<String, PipfilePackage>>,
}

/// Package information in Pipfile.lock
#[derive(Debug, Deserialize, Serialize)]
struct PipfilePackage {
    version: String,
    #[serde(default)]
    hashes: Vec<String>,
    #[serde(default)]
    index: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_requirements_txt(temp_dir: &Path) -> Result<()> {
        let requirements = r#"# Test requirements
requests==2.31.0
numpy>=1.24.0
pandas~=2.0.0
pytest  # no version
"#;

        let mut file = std::fs::File::create(temp_dir.join("requirements.txt"))?;
        file.write_all(requirements.as_bytes())?;
        Ok(())
    }

    fn create_test_pipfile_lock(temp_dir: &Path) -> Result<()> {
        let pipfile_lock = r#"{
    "_meta": {},
    "default": {
        "requests": {
            "version": "==2.31.0",
            "hashes": []
        },
        "numpy": {
            "version": "==1.24.0",
            "hashes": []
        }
    },
    "develop": {
        "pytest": {
            "version": "==7.4.0",
            "hashes": []
        }
    }
}"#;

        let mut file = std::fs::File::create(temp_dir.join("Pipfile.lock"))?;
        file.write_all(pipfile_lock.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_detect_python_project() {
        let temp_dir = TempDir::new().unwrap();
        let plugin = PythonPlugin::new();

        // No Python files - should not detect
        assert!(!plugin.detect(temp_dir.path()).unwrap());

        // Create requirements.txt
        create_test_requirements_txt(temp_dir.path()).unwrap();
        assert!(plugin.detect(temp_dir.path()).unwrap());
    }

    #[test]
    fn test_parse_requirements_txt() {
        let temp_dir = TempDir::new().unwrap();
        create_test_requirements_txt(temp_dir.path()).unwrap();

        let plugin = PythonPlugin::new();
        let graph = plugin.extract_dependencies(temp_dir.path()).unwrap();

        assert_eq!(graph.dependencies.len(), 4);
        
        // Check requests (exact version)
        let requests = graph.dependencies.iter()
            .find(|d| d.name == "requests")
            .unwrap();
        assert_eq!(requests.version, "2.31.0");
        assert_eq!(requests.scope, DependencyScope::Runtime);

        // Check numpy (>=)
        let numpy = graph.dependencies.iter()
            .find(|d| d.name == "numpy")
            .unwrap();
        assert_eq!(numpy.version, "1.24.0");
    }

    #[test]
    fn test_parse_pipfile_lock() {
        let temp_dir = TempDir::new().unwrap();
        create_test_pipfile_lock(temp_dir.path()).unwrap();

        let plugin = PythonPlugin::new();
        let graph = plugin.extract_dependencies(temp_dir.path()).unwrap();

        assert_eq!(graph.dependencies.len(), 3);
        
        // Check requests (runtime dep)
        let requests = graph.dependencies.iter()
            .find(|d| d.name == "requests")
            .unwrap();
        assert_eq!(requests.scope, DependencyScope::Runtime);

        // Check pytest (dev dep)
        let pytest = graph.dependencies.iter()
            .find(|d| d.name == "pytest")
            .unwrap();
        assert_eq!(pytest.scope, DependencyScope::Development);
    }

    #[test]
    fn test_parse_requirement_line() {
        let plugin = PythonPlugin::new();

        // Exact version
        let (name, version) = plugin.parse_requirement_line("requests==2.31.0").unwrap();
        assert_eq!(name, "requests");
        assert_eq!(version, "2.31.0");

        // Minimum version
        let (name, version) = plugin.parse_requirement_line("numpy>=1.24.0").unwrap();
        assert_eq!(name, "numpy");
        assert_eq!(version, "1.24.0");

        // Compatible version
        let (name, version) = plugin.parse_requirement_line("pandas~=2.0.0").unwrap();
        assert_eq!(name, "pandas");
        assert_eq!(version, "2.0.0");

        // No version
        let (name, version) = plugin.parse_requirement_line("pytest").unwrap();
        assert_eq!(name, "pytest");
        assert_eq!(version, "latest");
    }

    #[test]
    fn test_metadata() {
        let plugin = PythonPlugin::new();
        let metadata = plugin.get_metadata();

        assert_eq!(metadata.display_name, "Python");
        assert_eq!(metadata.registry_url, Some("https://pypi.org".to_string()));
        assert!(metadata.lockfile_names.contains(&"Pipfile.lock".to_string()));
        assert!(metadata.manifest_names.contains(&"pyproject.toml".to_string()));
    }
}
