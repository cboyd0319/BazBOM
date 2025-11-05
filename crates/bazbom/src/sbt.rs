//! sbt (Scala Build Tool) support
//!
//! Provides SBOM generation for sbt projects.
//! sbt uses build.sbt for configuration and coursier for dependency resolution.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// sbt project configuration
#[derive(Debug, Clone)]
pub struct SbtProject {
    /// Project root directory
    pub root: PathBuf,
    /// Main build file (build.sbt)
    pub build_file: PathBuf,
    /// Project directory (project/)
    pub project_dir: Option<PathBuf>,
}

impl SbtProject {
    /// Detect an sbt project
    pub fn detect(root: &Path) -> Result<Option<Self>> {
        let build_file = root.join("build.sbt");
        if !build_file.exists() {
            // Also check for project/build.properties
            let build_properties = root.join("project/build.properties");
            if !build_properties.exists() {
                return Ok(None);
            }
        }

        let project_dir = root.join("project");
        let project_dir = if project_dir.exists() && project_dir.is_dir() {
            Some(project_dir)
        } else {
            None
        };

        Ok(Some(Self {
            root: root.to_path_buf(),
            build_file: build_file.clone(),
            project_dir,
        }))
    }

    /// Extract dependencies from sbt project
    pub fn extract_dependencies(&self) -> Result<Vec<SbtDependency>> {
        let content = fs::read_to_string(&self.build_file)
            .with_context(|| format!("Failed to read build.sbt: {}", self.build_file.display()))?;

        let mut dependencies = Vec::new();

        // Parse dependency specifications
        // sbt uses Scala syntax like:
        // libraryDependencies += "org.scala-lang" % "scala-library" % "2.13.0"
        // libraryDependencies += "com.typesafe.akka" %% "akka-actor" % "2.6.0"
        // libraryDependencies ++= Seq(...)
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments
            if line.starts_with("//") {
                continue;
            }

            // Look for libraryDependencies patterns
            if let Some(dep) = self.parse_dependency_line(line) {
                dependencies.push(dep);
            }
        }

        Ok(dependencies)
    }

    /// Parse a single line for dependency declaration
    fn parse_dependency_line(&self, line: &str) -> Option<SbtDependency> {
        // Match patterns like:
        // libraryDependencies += "group" % "artifact" % "version"
        // libraryDependencies += "group" %% "artifact" % "version"
        
        if !line.contains("libraryDependencies") {
            return None;
        }

        // Extract the dependency specification
        // Look for patterns: "group" % "artifact" % "version"
        let parts: Vec<&str> = line.split('"').collect();
        
        if parts.len() >= 6 {
            let group_id = parts[1].to_string();
            let artifact_id = parts[3].to_string();
            let version = parts[5].to_string();

            // Determine scope from context
            let scope = if line.contains("Test") || line.contains("test") {
                "test".to_string()
            } else {
                "compile".to_string()
            };

            // Check for %% (Scala version cross-build)
            let is_scala_cross = line.contains("%%");

            return Some(SbtDependency {
                group_id,
                artifact_id,
                version,
                scope,
                scala_cross_version: is_scala_cross,
                source: DependencySource::BuildSbt,
            });
        }

        None
    }

    /// Generate SBOM from dependencies
    pub fn generate_sbom(&self, dependencies: &[SbtDependency]) -> Result<SbtSbom> {
        Ok(SbtSbom {
            project_name: self.get_project_name()?,
            project_version: self.get_project_version()?,
            scala_version: self.get_scala_version()?,
            build_file: self.build_file.clone(),
            dependencies: dependencies.to_vec(),
        })
    }

    /// Extract project name from build.sbt
    fn get_project_name(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;
        
        // Look for name := "project-name" pattern
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") && line.contains(":=") {
                // Extract quoted project name
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        Ok("unknown-sbt-project".to_string())
    }

    /// Extract project version from build.sbt
    fn get_project_version(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;
        
        // Look for version := "1.0.0" pattern
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("version") && line.contains(":=") {
                // Extract quoted version
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        Ok("1.0.0".to_string())
    }

    /// Extract Scala version from build.sbt
    fn get_scala_version(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;
        
        // Look for scalaVersion := "2.13.0" pattern
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("scalaVersion") && line.contains(":=") {
                // Extract quoted version
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        Ok("2.13.0".to_string())
    }
}

/// sbt dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbtDependency {
    /// Maven group ID (organization in sbt)
    pub group_id: String,
    /// Maven artifact ID (name in sbt)
    pub artifact_id: String,
    /// Version
    pub version: String,
    /// Dependency scope
    pub scope: String,
    /// Whether this uses Scala cross-version (%%)
    pub scala_cross_version: bool,
    /// Source of dependency detection
    pub source: DependencySource,
}

/// Source of dependency information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencySource {
    /// From build.sbt
    BuildSbt,
}

/// sbt SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbtSbom {
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Scala version
    pub scala_version: String,
    /// Path to build.sbt
    pub build_file: PathBuf,
    /// List of dependencies
    pub dependencies: Vec<SbtDependency>,
}

/// Convert sbt dependencies to Maven-style coordinates for vulnerability scanning
pub fn sbt_to_maven_coordinates(dep: &SbtDependency, scala_version: &str) -> String {
    let artifact = if dep.scala_cross_version {
        // Append Scala binary version (e.g., 2.13 from 2.13.0)
        let scala_binary = scala_version.split('.').take(2).collect::<Vec<_>>().join(".");
        format!("{}_{}", dep.artifact_id, scala_binary)
    } else {
        dep.artifact_id.clone()
    };

    format!("{}:{}:{}", dep.group_id, artifact, dep.version)
}

/// Extract sbt project and generate SBOM (main entry point)
pub fn extract_sbt_sbom(project_root: &Path) -> Result<Option<SbtSbom>> {
    let project = match SbtProject::detect(project_root)? {
        Some(p) => p,
        None => return Ok(None),
    };

    let dependencies = project.extract_dependencies()?;
    let sbom = project.generate_sbom(&dependencies)?;

    Ok(Some(sbom))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_sbt_project_detect_with_build_sbt() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        fs::write(root.join("build.sbt"), "name := \"my-project\"").unwrap();
        
        let project = SbtProject::detect(root).unwrap();
        assert!(project.is_some());
        
        let project = project.unwrap();
        assert_eq!(project.build_file, root.join("build.sbt"));
    }

    #[test]
    fn test_sbt_project_detect_with_project_dir() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        fs::create_dir_all(root.join("project")).unwrap();
        fs::write(root.join("project/build.properties"), "sbt.version=1.9.0").unwrap();
        
        let project = SbtProject::detect(root).unwrap();
        assert!(project.is_some());
    }

    #[test]
    fn test_sbt_project_not_detected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        let project = SbtProject::detect(root).unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_dependency_line_simple() {
        let temp_dir = TempDir::new().unwrap();
        let project = SbtProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("build.sbt"),
            project_dir: None,
        };
        
        let dep = project.parse_dependency_line(
            r#"libraryDependencies += "org.scala-lang" % "scala-library" % "2.13.0""#
        ).unwrap();
        
        assert_eq!(dep.group_id, "org.scala-lang");
        assert_eq!(dep.artifact_id, "scala-library");
        assert_eq!(dep.version, "2.13.0");
        assert!(!dep.scala_cross_version);
    }

    #[test]
    fn test_parse_dependency_line_scala_cross() {
        let temp_dir = TempDir::new().unwrap();
        let project = SbtProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("build.sbt"),
            project_dir: None,
        };
        
        let dep = project.parse_dependency_line(
            r#"libraryDependencies += "com.typesafe.akka" %% "akka-actor" % "2.6.0""#
        ).unwrap();
        
        assert_eq!(dep.group_id, "com.typesafe.akka");
        assert_eq!(dep.artifact_id, "akka-actor");
        assert_eq!(dep.version, "2.6.0");
        assert!(dep.scala_cross_version);
    }

    #[test]
    fn test_sbt_to_maven_coordinates_simple() {
        let dep = SbtDependency {
            group_id: "org.scala-lang".to_string(),
            artifact_id: "scala-library".to_string(),
            version: "2.13.0".to_string(),
            scope: "compile".to_string(),
            scala_cross_version: false,
            source: DependencySource::BuildSbt,
        };
        
        let coords = sbt_to_maven_coordinates(&dep, "2.13.0");
        assert_eq!(coords, "org.scala-lang:scala-library:2.13.0");
    }

    #[test]
    fn test_sbt_to_maven_coordinates_cross_version() {
        let dep = SbtDependency {
            group_id: "com.typesafe.akka".to_string(),
            artifact_id: "akka-actor".to_string(),
            version: "2.6.0".to_string(),
            scope: "compile".to_string(),
            scala_cross_version: true,
            source: DependencySource::BuildSbt,
        };
        
        let coords = sbt_to_maven_coordinates(&dep, "2.13.5");
        assert_eq!(coords, "com.typesafe.akka:akka-actor_2.13:2.6.0");
    }

    #[test]
    fn test_dependency_source_equality() {
        assert_eq!(DependencySource::BuildSbt, DependencySource::BuildSbt);
    }

    #[test]
    fn test_sbt_sbom_structure() {
        let sbom = SbtSbom {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            scala_version: "2.13.0".to_string(),
            build_file: PathBuf::from("build.sbt"),
            dependencies: vec![],
        };
        
        assert_eq!(sbom.project_name, "test-project");
        assert_eq!(sbom.project_version, "1.0.0");
        assert_eq!(sbom.scala_version, "2.13.0");
        assert_eq!(sbom.dependencies.len(), 0);
    }
}
