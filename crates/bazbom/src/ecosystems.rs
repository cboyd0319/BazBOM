// Multi-language ecosystem support for BazBOM
// This module provides parsers for Go modules and Rust/Cargo dependencies

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a dependency from any ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemDependency {
    pub name: String,
    pub version: String,
    pub ecosystem: EcosystemType,
    pub scope: DependencyScope,
    pub purl: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EcosystemType {
    Go,
    Rust,
    Maven,
    Gradle,
    Npm,
    Python,
}

impl EcosystemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            EcosystemType::Go => "golang",
            EcosystemType::Rust => "cargo",
            EcosystemType::Maven => "maven",
            EcosystemType::Gradle => "gradle",
            EcosystemType::Npm => "npm",
            EcosystemType::Python => "pypi",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DependencyScope {
    Direct,
    Indirect,
    Dev,
    Test,
    Build,
}

/// Go modules parser for go.mod and go.sum files
pub struct GoModulesParser;

impl GoModulesParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse go.mod file and extract dependencies
    pub fn parse_go_mod(&self, path: &Path) -> Result<Vec<EcosystemDependency>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read go.mod at {:?}", path))?;

        let mut dependencies = Vec::new();
        let mut in_require_block = false;

        for line in content.lines() {
            let line = line.trim();

            // Handle require block
            if line.starts_with("require (") {
                in_require_block = true;
                continue;
            }
            if in_require_block && line == ")" {
                in_require_block = false;
                continue;
            }

            // Parse require statement
            if line.starts_with("require ") || in_require_block {
                if let Some(dep) = self.parse_require_line(line, in_require_block)? {
                    dependencies.push(dep);
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_require_line(
        &self,
        line: &str,
        in_block: bool,
    ) -> Result<Option<EcosystemDependency>> {
        let line = if in_block {
            line
        } else {
            line.strip_prefix("require ").unwrap_or(line)
        };

        // Check for indirect marker before removing comments
        let is_indirect = line.contains("// indirect");

        // Remove comments
        let line = line.split("//").next().unwrap_or(line).trim();
        if line.is_empty() {
            return Ok(None);
        }

        // Parse "module version" format
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        let name = parts[0].to_string();
        let version = parts[1].trim_start_matches('v').to_string();

        // Determine if indirect
        let scope = if is_indirect {
            DependencyScope::Indirect
        } else {
            DependencyScope::Direct
        };

        let purl = format!("pkg:golang/{}@{}", name, version);

        Ok(Some(EcosystemDependency {
            name,
            version,
            ecosystem: EcosystemType::Go,
            scope,
            purl,
        }))
    }

    /// Parse go.sum file for version validation
    pub fn parse_go_sum(&self, path: &Path) -> Result<HashMap<String, Vec<String>>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read go.sum at {:?}", path))?;

        let mut checksums = HashMap::new();

        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let module_version = format!("{}@{}", parts[0], parts[1].trim_start_matches('v'));
                let checksum = parts[2].to_string();
                checksums
                    .entry(module_version)
                    .or_insert_with(Vec::new)
                    .push(checksum);
            }
        }

        Ok(checksums)
    }
}

/// Rust/Cargo parser for Cargo.toml and Cargo.lock files
pub struct CargoParser;

impl CargoParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse Cargo.toml file and extract dependencies
    pub fn parse_cargo_toml(&self, path: &Path) -> Result<Vec<EcosystemDependency>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read Cargo.toml at {:?}", path))?;

        let value: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse Cargo.toml at {:?}", path))?;

        let mut dependencies = Vec::new();

        // Parse [dependencies]
        if let Some(deps) = value.get("dependencies").and_then(|v| v.as_table()) {
            for (name, version_spec) in deps {
                if let Some(dep) = self.parse_cargo_dependency(name, version_spec, DependencyScope::Direct)? {
                    dependencies.push(dep);
                }
            }
        }

        // Parse [dev-dependencies]
        if let Some(deps) = value.get("dev-dependencies").and_then(|v| v.as_table()) {
            for (name, version_spec) in deps {
                if let Some(dep) = self.parse_cargo_dependency(name, version_spec, DependencyScope::Dev)? {
                    dependencies.push(dep);
                }
            }
        }

        // Parse [build-dependencies]
        if let Some(deps) = value.get("build-dependencies").and_then(|v| v.as_table()) {
            for (name, version_spec) in deps {
                if let Some(dep) = self.parse_cargo_dependency(name, version_spec, DependencyScope::Build)? {
                    dependencies.push(dep);
                }
            }
        }

        Ok(dependencies)
    }

    fn parse_cargo_dependency(
        &self,
        name: &str,
        version_spec: &toml::Value,
        scope: DependencyScope,
    ) -> Result<Option<EcosystemDependency>> {
        let version = match version_spec {
            toml::Value::String(v) => v.clone(),
            toml::Value::Table(t) => {
                if let Some(v) = t.get("version") {
                    v.as_str().unwrap_or("*").to_string()
                } else if t.contains_key("git") || t.contains_key("path") {
                    // Skip git and path dependencies
                    return Ok(None);
                } else {
                    "*".to_string()
                }
            }
            _ => return Ok(None),
        };

        let purl = format!("pkg:cargo/{}@{}", name, version);

        Ok(Some(EcosystemDependency {
            name: name.to_string(),
            version,
            ecosystem: EcosystemType::Rust,
            scope,
            purl,
        }))
    }

    /// Parse Cargo.lock file for exact versions
    pub fn parse_cargo_lock(&self, path: &Path) -> Result<Vec<EcosystemDependency>> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read Cargo.lock at {:?}", path))?;

        let value: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse Cargo.lock at {:?}", path))?;

        let mut dependencies = Vec::new();

        if let Some(packages) = value.get("package").and_then(|v| v.as_array()) {
            for package in packages {
                if let Some(pkg_table) = package.as_table() {
                    let name = pkg_table
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    let version = pkg_table
                        .get("version")
                        .and_then(|v| v.as_str())
                        .unwrap_or("0.0.0");

                    let purl = format!("pkg:cargo/{}@{}", name, version);

                    dependencies.push(EcosystemDependency {
                        name: name.to_string(),
                        version: version.to_string(),
                        ecosystem: EcosystemType::Rust,
                        scope: DependencyScope::Direct, // Lock file doesn't distinguish
                        purl,
                    });
                }
            }
        }

        Ok(dependencies)
    }
}

/// Detect ecosystem type from project structure
pub fn detect_ecosystem(project_path: &Path) -> Result<Vec<EcosystemType>> {
    let mut ecosystems = Vec::new();

    if project_path.join("go.mod").exists() {
        ecosystems.push(EcosystemType::Go);
    }
    if project_path.join("Cargo.toml").exists() {
        ecosystems.push(EcosystemType::Rust);
    }
    if project_path.join("pom.xml").exists() {
        ecosystems.push(EcosystemType::Maven);
    }
    if project_path.join("build.gradle").exists()
        || project_path.join("build.gradle.kts").exists()
    {
        ecosystems.push(EcosystemType::Gradle);
    }
    if project_path.join("package.json").exists() {
        ecosystems.push(EcosystemType::Npm);
    }
    if project_path.join("requirements.txt").exists()
        || project_path.join("Pipfile").exists()
        || project_path.join("pyproject.toml").exists()
    {
        ecosystems.push(EcosystemType::Python);
    }

    Ok(ecosystems)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parse_go_mod_simple() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = r#"
module example.com/myproject

go 1.21

require (
    github.com/gin-gonic/gin v1.9.1
    github.com/stretchr/testify v1.8.4
    golang.org/x/crypto v0.14.0 // indirect
)
"#;
        fs::write(&go_mod_path, content).unwrap();

        let parser = GoModulesParser::new();
        let deps = parser.parse_go_mod(&go_mod_path).unwrap();

        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "github.com/gin-gonic/gin");
        assert_eq!(deps[0].version, "1.9.1");
        assert_eq!(deps[0].scope, DependencyScope::Direct);
        assert_eq!(deps[2].scope, DependencyScope::Indirect);
    }

    #[test]
    fn test_parse_cargo_toml_simple() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml_path = temp_dir.path().join("Cargo.toml");

        let content = r#"
[package]
name = "myproject"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = { version = "1.32", features = ["full"] }

[dev-dependencies]
mockall = "0.12"
"#;
        fs::write(&cargo_toml_path, content).unwrap();

        let parser = CargoParser::new();
        let deps = parser.parse_cargo_toml(&cargo_toml_path).unwrap();

        assert!(deps.len() >= 2);
        assert!(deps.iter().any(|d| d.name == "serde" && d.version == "1.0"));
        assert!(deps
            .iter()
            .any(|d| d.name == "tokio" && d.version == "1.32"));
        assert!(deps
            .iter()
            .any(|d| d.name == "mockall" && d.scope == DependencyScope::Dev));
    }

    #[test]
    fn test_detect_ecosystem_multi_language() {
        let temp_dir = TempDir::new().unwrap();

        // Create marker files for multiple ecosystems
        fs::write(temp_dir.path().join("go.mod"), "module test\n").unwrap();
        fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();
        fs::write(temp_dir.path().join("pom.xml"), "<project></project>").unwrap();

        let ecosystems = detect_ecosystem(temp_dir.path()).unwrap();

        assert!(ecosystems.contains(&EcosystemType::Go));
        assert!(ecosystems.contains(&EcosystemType::Rust));
        assert!(ecosystems.contains(&EcosystemType::Maven));
    }

    #[test]
    fn test_go_mod_single_require() {
        let temp_dir = TempDir::new().unwrap();
        let go_mod_path = temp_dir.path().join("go.mod");

        let content = "module test\nrequire github.com/example/lib v1.2.3\n";
        fs::write(&go_mod_path, content).unwrap();

        let parser = GoModulesParser::new();
        let deps = parser.parse_go_mod(&go_mod_path).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "github.com/example/lib");
        assert_eq!(deps[0].version, "1.2.3");
    }

    #[test]
    fn test_cargo_lock_parsing() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_lock_path = temp_dir.path().join("Cargo.lock");

        let content = r#"
[[package]]
name = "serde"
version = "1.0.193"

[[package]]
name = "tokio"
version = "1.35.0"
"#;
        fs::write(&cargo_lock_path, content).unwrap();

        let parser = CargoParser::new();
        let deps = parser.parse_cargo_lock(&cargo_lock_path).unwrap();

        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "serde" && d.version == "1.0.193"));
        assert!(deps.iter().any(|d| d.name == "tokio" && d.version == "1.35.0"));
    }
}
