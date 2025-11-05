//! Buildr build system support
//!
//! Provides SBOM generation for Buildr projects (Ruby-based JVM build tool).
//! Buildr uses buildfile (lowercase) or Rakefile with Buildr DSL for configuration.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Buildr project configuration
#[derive(Debug, Clone)]
pub struct BuildrProject {
    /// Project root directory
    pub root: PathBuf,
    /// Build file (buildfile or Rakefile)
    pub build_file: PathBuf,
    /// Whether this is a Rakefile-based project
    pub is_rakefile: bool,
}

impl BuildrProject {
    /// Detect a Buildr project
    pub fn detect(root: &Path) -> Result<Option<Self>> {
        // Check for buildfile (lowercase)
        let buildfile = root.join("buildfile");
        if buildfile.exists() {
            return Ok(Some(Self {
                root: root.to_path_buf(),
                build_file: buildfile,
                is_rakefile: false,
            }));
        }

        // Check for Rakefile with Buildr content
        let rakefile = root.join("Rakefile");
        if rakefile.exists() {
            let content = fs::read_to_string(&rakefile)
                .with_context(|| format!("Failed to read Rakefile: {}", rakefile.display()))?;
            
            if content.contains("require 'buildr'")
                || content.contains("require \"buildr\"")
                || content.contains("Buildr.application")
            {
                return Ok(Some(Self {
                    root: root.to_path_buf(),
                    build_file: rakefile,
                    is_rakefile: true,
                }));
            }
        }

        Ok(None)
    }

    /// Extract dependencies from Buildr project
    pub fn extract_dependencies(&self) -> Result<Vec<BuildrDependency>> {
        let content = fs::read_to_string(&self.build_file)
            .with_context(|| format!("Failed to read build file: {}", self.build_file.display()))?;

        let mut dependencies = Vec::new();

        // Parse dependency specifications
        // Buildr uses Ruby syntax like:
        // compile.with 'org.springframework:spring-core:jar:5.3.0'
        // compile.with 'commons-io:commons-io:jar:2.11.0'
        // compile.from 'group:artifact:version'
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments
            if line.starts_with('#') {
                continue;
            }

            // Look for compile.with or compile.from patterns
            if let Some(dep) = self.parse_dependency_line(line) {
                dependencies.push(dep);
            }
        }

        Ok(dependencies)
    }

    /// Parse a single line for dependency declaration
    fn parse_dependency_line(&self, line: &str) -> Option<BuildrDependency> {
        // Match patterns like:
        // compile.with 'group:artifact:jar:version'
        // compile.from 'group:artifact:version'
        // test.with 'group:artifact:version'
        
        let patterns = [
            "compile.with",
            "compile.from",
            "test.with",
            "test.from",
        ];

        for pattern in &patterns {
            if line.contains(pattern) {
                // Extract the quoted string
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        let dep_str = &line[start + 1..start + 1 + end];
                        return self.parse_maven_coordinate(dep_str, pattern);
                    }
                }
                // Try double quotes
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let dep_str = &line[start + 1..start + 1 + end];
                        return self.parse_maven_coordinate(dep_str, pattern);
                    }
                }
            }
        }

        None
    }

    /// Parse Maven-style coordinate string
    /// Format: group:artifact:type:version or group:artifact:version
    fn parse_maven_coordinate(&self, coord: &str, scope_hint: &str) -> Option<BuildrDependency> {
        let parts: Vec<&str> = coord.split(':').collect();
        
        if parts.len() >= 3 {
            let group_id = parts[0].to_string();
            let artifact_id = parts[1].to_string();
            
            // Handle both group:artifact:version and group:artifact:type:version
            let version = if parts.len() >= 4 {
                parts[3].to_string()
            } else {
                parts[2].to_string()
            };

            let scope = if scope_hint.contains("test") {
                "test".to_string()
            } else {
                "compile".to_string()
            };

            return Some(BuildrDependency {
                group_id,
                artifact_id,
                version,
                scope,
                source: DependencySource::Buildfile,
            });
        }

        None
    }

    /// Generate SBOM from dependencies
    pub fn generate_sbom(&self, dependencies: &[BuildrDependency]) -> Result<BuildrSbom> {
        Ok(BuildrSbom {
            project_name: self.get_project_name()?,
            project_version: self.get_project_version()?,
            build_file: self.build_file.clone(),
            dependencies: dependencies.to_vec(),
        })
    }

    /// Extract project name from build file
    fn get_project_name(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;
        
        // Look for define 'project-name' pattern
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("define") {
                // Extract quoted project name
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        Ok("unknown-buildr-project".to_string())
    }

    /// Extract project version from build file
    fn get_project_version(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;
        
        // Look for VERSION = '1.0.0' or version = '1.0.0'
        for line in content.lines() {
            let line = line.trim();
            if line.to_uppercase().starts_with("VERSION") && line.contains('=') {
                // Extract quoted version
                if let Some(start) = line.find('\'') {
                    if let Some(end) = line[start + 1..].find('\'') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Ok(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        Ok("1.0.0".to_string())
    }
}

/// Buildr dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildrDependency {
    /// Maven group ID
    pub group_id: String,
    /// Maven artifact ID
    pub artifact_id: String,
    /// Version
    pub version: String,
    /// Dependency scope
    pub scope: String,
    /// Source of dependency detection
    pub source: DependencySource,
}

/// Source of dependency information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencySource {
    /// From buildfile/Rakefile
    Buildfile,
}

/// Buildr SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildrSbom {
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Path to build file
    pub build_file: PathBuf,
    /// List of dependencies
    pub dependencies: Vec<BuildrDependency>,
}

/// Convert Buildr dependencies to Maven-style coordinates for vulnerability scanning
pub fn buildr_to_maven_coordinates(dep: &BuildrDependency) -> String {
    format!("{}:{}:{}", dep.group_id, dep.artifact_id, dep.version)
}

/// Extract Buildr project and generate SBOM (main entry point)
pub fn extract_buildr_sbom(project_root: &Path) -> Result<Option<BuildrSbom>> {
    let project = match BuildrProject::detect(project_root)? {
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
    fn test_buildr_project_detect_with_buildfile() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        fs::write(root.join("buildfile"), "define 'my-project'").unwrap();
        
        let project = BuildrProject::detect(root).unwrap();
        assert!(project.is_some());
        
        let project = project.unwrap();
        assert_eq!(project.build_file, root.join("buildfile"));
        assert!(!project.is_rakefile);
    }

    #[test]
    fn test_buildr_project_detect_with_rakefile() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        fs::write(root.join("Rakefile"), "require 'buildr'\ndefine 'my-project'").unwrap();
        
        let project = BuildrProject::detect(root).unwrap();
        assert!(project.is_some());
        
        let project = project.unwrap();
        assert_eq!(project.build_file, root.join("Rakefile"));
        assert!(project.is_rakefile);
    }

    #[test]
    fn test_buildr_project_not_detected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        
        // Regular Rakefile without Buildr
        fs::write(root.join("Rakefile"), "task :default do\nend").unwrap();
        
        let project = BuildrProject::detect(root).unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_maven_coordinate_simple() {
        let temp_dir = TempDir::new().unwrap();
        let project = BuildrProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("buildfile"),
            is_rakefile: false,
        };
        
        let dep = project.parse_maven_coordinate(
            "org.springframework:spring-core:5.3.0",
            "compile.with"
        ).unwrap();
        
        assert_eq!(dep.group_id, "org.springframework");
        assert_eq!(dep.artifact_id, "spring-core");
        assert_eq!(dep.version, "5.3.0");
        assert_eq!(dep.scope, "compile");
    }

    #[test]
    fn test_parse_maven_coordinate_with_type() {
        let temp_dir = TempDir::new().unwrap();
        let project = BuildrProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("buildfile"),
            is_rakefile: false,
        };
        
        let dep = project.parse_maven_coordinate(
            "commons-io:commons-io:jar:2.11.0",
            "compile.from"
        ).unwrap();
        
        assert_eq!(dep.group_id, "commons-io");
        assert_eq!(dep.artifact_id, "commons-io");
        assert_eq!(dep.version, "2.11.0");
    }

    #[test]
    fn test_parse_dependency_line_compile_with() {
        let temp_dir = TempDir::new().unwrap();
        let project = BuildrProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("buildfile"),
            is_rakefile: false,
        };
        
        let dep = project.parse_dependency_line(
            "  compile.with 'org.apache.commons:commons-lang3:3.12.0'"
        ).unwrap();
        
        assert_eq!(dep.group_id, "org.apache.commons");
        assert_eq!(dep.artifact_id, "commons-lang3");
        assert_eq!(dep.version, "3.12.0");
    }

    #[test]
    fn test_parse_dependency_line_test_scope() {
        let temp_dir = TempDir::new().unwrap();
        let project = BuildrProject {
            root: temp_dir.path().to_path_buf(),
            build_file: temp_dir.path().join("buildfile"),
            is_rakefile: false,
        };
        
        let dep = project.parse_dependency_line(
            "  test.with 'junit:junit:4.13.2'"
        ).unwrap();
        
        assert_eq!(dep.scope, "test");
    }

    #[test]
    fn test_buildr_to_maven_coordinates() {
        let dep = BuildrDependency {
            group_id: "org.springframework".to_string(),
            artifact_id: "spring-core".to_string(),
            version: "5.3.0".to_string(),
            scope: "compile".to_string(),
            source: DependencySource::Buildfile,
        };
        
        let coords = buildr_to_maven_coordinates(&dep);
        assert_eq!(coords, "org.springframework:spring-core:5.3.0");
    }

    #[test]
    fn test_dependency_source_equality() {
        assert_eq!(DependencySource::Buildfile, DependencySource::Buildfile);
    }

    #[test]
    fn test_buildr_sbom_structure() {
        let sbom = BuildrSbom {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            build_file: PathBuf::from("buildfile"),
            dependencies: vec![],
        };
        
        assert_eq!(sbom.project_name, "test-project");
        assert_eq!(sbom.project_version, "1.0.0");
        assert_eq!(sbom.dependencies.len(), 0);
    }
}
