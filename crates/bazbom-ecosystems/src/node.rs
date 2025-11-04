//! Node.js/npm ecosystem support

use anyhow::{Context, Result};
use crate::{Dependency, DependencyGraph, DependencyScope, EcosystemMetadata, EcosystemPlugin};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Node.js ecosystem plugin
pub struct NodePlugin;

impl NodePlugin {
    /// Create a new Node.js plugin
    pub fn new() -> Self {
        Self
    }

    /// Parse package-lock.json (npm v7+)
    fn parse_package_lock(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read package-lock.json from {}", path.display()))?;

        let lock: PackageLock = serde_json::from_str(&content)
            .context("Failed to parse package-lock.json")?;

        let mut graph = DependencyGraph::new();
        let mut processed = std::collections::HashSet::new();

        // Process packages from lockfile
        if let Some(packages) = lock.packages {
            for (pkg_path, pkg_info) in packages {
                // Skip root package
                if pkg_path.is_empty() {
                    continue;
                }

                // Extract package name from path (remove "node_modules/" prefix)
                let name = pkg_path
                    .strip_prefix("node_modules/")
                    .unwrap_or(&pkg_path)
                    .to_string();

                if processed.contains(&name) {
                    continue;
                }
                processed.insert(name.clone());

                let dependency = Dependency {
                    name: name.clone(),
                    version: pkg_info.version.clone(),
                    scope: if pkg_info.dev.unwrap_or(false) {
                        DependencyScope::Development
                    } else {
                        DependencyScope::Runtime
                    },
                    ecosystem: "node".to_string(),
                    purl: Some(format!("pkg:npm/{}@{}", name, pkg_info.version)),
                    direct: pkg_info.is_direct(),
                };

                graph.add_dependency(dependency);
            }
        }

        Ok(graph)
    }

    /// Parse package.json (fallback if no lockfile)
    fn parse_package_json(&self, path: &Path) -> Result<DependencyGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read package.json from {}", path.display()))?;

        let manifest: PackageJson = serde_json::from_str(&content)
            .context("Failed to parse package.json")?;

        let mut graph = DependencyGraph::new();

        // Add production dependencies
        if let Some(deps) = manifest.dependencies {
            for (name, version) in deps {
                let dependency = Dependency {
                    name: name.clone(),
                    version: version.clone(),
                    scope: DependencyScope::Runtime,
                    ecosystem: "node".to_string(),
                    purl: Some(format!("pkg:npm/{}@{}", name, version)),
                    direct: true,
                };
                graph.add_dependency(dependency);
            }
        }

        // Add dev dependencies
        if let Some(dev_deps) = manifest.dev_dependencies {
            for (name, version) in dev_deps {
                let dependency = Dependency {
                    name: name.clone(),
                    version: version.clone(),
                    scope: DependencyScope::Development,
                    ecosystem: "node".to_string(),
                    purl: Some(format!("pkg:npm/{}@{}", name, version)),
                    direct: true,
                };
                graph.add_dependency(dependency);
            }
        }

        Ok(graph)
    }
}

impl EcosystemPlugin for NodePlugin {
    fn name(&self) -> &str {
        "node"
    }

    fn detect(&self, project_root: &Path) -> Result<bool> {
        Ok(project_root.join("package.json").exists())
    }

    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph> {
        // Try package-lock.json first (most accurate)
        let package_lock = project_root.join("package-lock.json");
        if package_lock.exists() {
            return self.parse_package_lock(&package_lock);
        }

        // Try yarn.lock
        let yarn_lock = project_root.join("yarn.lock");
        if yarn_lock.exists() {
            // TODO: Implement yarn.lock parsing
            anyhow::bail!("yarn.lock parsing not yet implemented");
        }

        // Try pnpm-lock.yaml
        let pnpm_lock = project_root.join("pnpm-lock.yaml");
        if pnpm_lock.exists() {
            // TODO: Implement pnpm-lock.yaml parsing
            anyhow::bail!("pnpm-lock.yaml parsing not yet implemented");
        }

        // Fallback to package.json (no version lock)
        let package_json = project_root.join("package.json");
        if package_json.exists() {
            return self.parse_package_json(&package_json);
        }

        anyhow::bail!("No package.json found in {}", project_root.display());
    }



    fn get_metadata(&self) -> EcosystemMetadata {
        EcosystemMetadata {
            display_name: "Node.js".to_string(),
            registry_url: Some("https://registry.npmjs.org".to_string()),
            lockfile_names: vec![
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
            ],
            manifest_names: vec!["package.json".to_string()],
        }
    }
}

impl Default for NodePlugin {
    fn default() -> Self {
        Self::new()
    }
}

/// package-lock.json structure (npm v7+)
#[derive(Debug, Deserialize, Serialize)]
struct PackageLock {
    name: Option<String>,
    version: Option<String>,
    #[serde(rename = "lockfileVersion")]
    lockfile_version: Option<u32>,
    packages: Option<HashMap<String, PackageInfo>>,
}

/// Package information in lockfile
#[derive(Debug, Deserialize, Serialize)]
struct PackageInfo {
    version: String,
    resolved: Option<String>,
    integrity: Option<String>,
    dev: Option<bool>,
    #[serde(default)]
    dependencies: HashMap<String, String>,
}

impl PackageInfo {
    fn is_direct(&self) -> bool {
        // In npm lockfile, direct dependencies don't have nested path
        // This is a heuristic - may need refinement
        !self.version.contains("node_modules")
    }
}

/// package.json structure
#[derive(Debug, Deserialize, Serialize)]
struct PackageJson {
    name: Option<String>,
    version: Option<String>,
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_package_json(temp_dir: &Path) -> Result<()> {
        let package_json = r#"{
  "name": "test-app",
  "version": "1.0.0",
  "dependencies": {
    "express": "^4.18.0",
    "lodash": "^4.17.21"
  },
  "devDependencies": {
    "jest": "^29.0.0"
  }
}"#;

        let mut file = std::fs::File::create(temp_dir.join("package.json"))?;
        file.write_all(package_json.as_bytes())?;
        Ok(())
    }

    fn create_test_package_lock(temp_dir: &Path) -> Result<()> {
        let package_lock = r#"{
  "name": "test-app",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {
    "": {
      "version": "1.0.0",
      "dependencies": {
        "express": "^4.18.0"
      }
    },
    "node_modules/express": {
      "version": "4.18.2",
      "resolved": "https://registry.npmjs.org/express/-/express-4.18.2.tgz",
      "integrity": "sha512-xyz"
    },
    "node_modules/lodash": {
      "version": "4.17.21",
      "resolved": "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz",
      "integrity": "sha512-abc"
    }
  }
}"#;

        let mut file = std::fs::File::create(temp_dir.join("package-lock.json"))?;
        file.write_all(package_lock.as_bytes())?;
        Ok(())
    }

    #[test]
    fn test_detect_node_project() {
        let temp_dir = TempDir::new().unwrap();
        let plugin = NodePlugin::new();

        // No package.json - should not detect
        assert!(!plugin.detect(temp_dir.path()).unwrap());

        // Create package.json
        create_test_package_json(temp_dir.path()).unwrap();
        assert!(plugin.detect(temp_dir.path()).unwrap());
    }

    #[test]
    fn test_parse_package_json() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path()).unwrap();

        let plugin = NodePlugin::new();
        let graph = plugin.extract_dependencies(temp_dir.path()).unwrap();

        assert_eq!(graph.dependencies.len(), 3);
        
        // Check express (production dep)
        let express = graph.dependencies.iter()
            .find(|d| d.name == "express")
            .unwrap();
        assert_eq!(express.scope, DependencyScope::Runtime);

        // Check jest (dev dep)
        let jest = graph.dependencies.iter()
            .find(|d| d.name == "jest")
            .unwrap();
        assert_eq!(jest.scope, DependencyScope::Development);
    }

    #[test]
    fn test_parse_package_lock() {
        let temp_dir = TempDir::new().unwrap();
        create_test_package_json(temp_dir.path()).unwrap();
        create_test_package_lock(temp_dir.path()).unwrap();

        let plugin = NodePlugin::new();
        let graph = plugin.extract_dependencies(temp_dir.path()).unwrap();

        // Should parse from lockfile (2 dependencies)
        assert_eq!(graph.dependencies.len(), 2);
        
        let express = graph.dependencies.iter()
            .find(|d| d.name == "express")
            .unwrap();
        assert_eq!(express.version, "4.18.2");
    }



    #[test]
    fn test_metadata() {
        let plugin = NodePlugin::new();
        let metadata = plugin.get_metadata();

        assert_eq!(metadata.display_name, "Node.js");
        assert_eq!(metadata.registry_url, Some("https://registry.npmjs.org".to_string()));
        assert!(metadata.lockfile_names.contains(&"package-lock.json".to_string()));
        assert!(metadata.manifest_names.contains(&"package.json".to_string()));
    }
}
