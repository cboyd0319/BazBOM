//! Apache Ant build system support
//!
//! Provides SBOM generation for Ant projects with Ivy dependency management.
//! Supports both Ivy-based dependency resolution and manual JAR detection.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Ant project configuration
#[derive(Debug, Clone)]
pub struct AntProject {
    /// Project root directory
    pub root: PathBuf,
    /// Ivy configuration file (if present)
    pub ivy_file: Option<PathBuf>,
    /// Ant build file
    pub build_file: PathBuf,
}

impl AntProject {
    /// Detect an Ant project
    pub fn detect(root: &Path) -> Result<Option<Self>> {
        let build_file = root.join("build.xml");
        if !build_file.exists() {
            return Ok(None);
        }

        let ivy_file = root.join("ivy.xml");
        let ivy_file = if ivy_file.exists() {
            Some(ivy_file)
        } else {
            None
        };

        Ok(Some(Self {
            root: root.to_path_buf(),
            ivy_file,
            build_file,
        }))
    }

    /// Extract dependencies from Ant project
    pub fn extract_dependencies(&self) -> Result<Vec<AntDependency>> {
        if let Some(ref ivy_file) = self.ivy_file {
            // Parse Ivy dependencies
            self.extract_ivy_dependencies(ivy_file)
        } else {
            // Detect manual JAR dependencies
            self.detect_manual_jars()
        }
    }

    /// Parse Ivy XML file for dependencies
    fn extract_ivy_dependencies(&self, ivy_file: &Path) -> Result<Vec<AntDependency>> {
        let content = fs::read_to_string(ivy_file)
            .with_context(|| format!("Failed to read ivy.xml: {}", ivy_file.display()))?;

        let mut dependencies = Vec::new();
        let mut reader = quick_xml::Reader::from_str(&content);
        reader.config_mut().trim_text(true);

        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) | Ok(quick_xml::events::Event::Empty(e)) => {
                    if e.name().as_ref() == b"dependency" {
                        let mut org = String::new();
                        let mut name = String::new();
                        let mut rev = String::new();
                        let mut conf = String::from("compile");

                        for attr in e.attributes().flatten() {
                            let key = attr.key.as_ref();
                            let value = String::from_utf8_lossy(&attr.value).to_string();

                            match key {
                                b"org" => org = value,
                                b"name" => name = value,
                                b"rev" => rev = value,
                                b"conf" => conf = value,
                                _ => {}
                            }
                        }

                        if !org.is_empty() && !name.is_empty() && !rev.is_empty() {
                            dependencies.push(AntDependency {
                                group_id: org,
                                artifact_id: name,
                                version: rev,
                                scope: conf,
                                source: DependencySource::Ivy,
                            });
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Error parsing ivy.xml at position {}: {}",
                        reader.buffer_position(),
                        e
                    ))
                }
                _ => {}
            }
            buf.clear();
        }

        Ok(dependencies)
    }

    /// Detect manual JAR files in lib directories
    fn detect_manual_jars(&self) -> Result<Vec<AntDependency>> {
        let mut dependencies = Vec::new();

        // Common lib directories
        let lib_dirs = vec![
            self.root.join("lib"),
            self.root.join("libs"),
            self.root.join("lib/compile"),
            self.root.join("lib/runtime"),
        ];

        for lib_dir in lib_dirs {
            if lib_dir.exists() && lib_dir.is_dir() {
                dependencies.extend(self.scan_jar_directory(&lib_dir)?);
            }
        }

        Ok(dependencies)
    }

    /// Scan a directory for JAR files
    fn scan_jar_directory(&self, dir: &Path) -> Result<Vec<AntDependency>> {
        let mut dependencies = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("jar") {
                if let Some(dep) = self.parse_jar_filename(&path) {
                    dependencies.push(dep);
                }
            }
        }

        Ok(dependencies)
    }

    /// Parse JAR filename to extract dependency info
    ///
    /// Attempts to parse common naming patterns:
    /// - groupId-artifactId-version.jar
    /// - artifactId-version.jar
    /// - artifactId.jar
    fn parse_jar_filename(&self, jar_path: &Path) -> Option<AntDependency> {
        let filename = jar_path.file_stem()?.to_str()?;

        // Try to split by '-' and identify version pattern
        let parts: Vec<&str> = filename.split('-').collect();

        if parts.len() >= 2 {
            // Check if last part looks like a version (contains digit)
            let last_part = parts[parts.len() - 1];
            if last_part.chars().any(|c| c.is_ascii_digit()) {
                let version = last_part.to_string();
                let artifact_parts = &parts[0..parts.len() - 1];
                let artifact_id = artifact_parts.join("-");

                // Try to infer group_id from common patterns
                let group_id = if artifact_parts.len() > 1 {
                    artifact_parts[0].to_string()
                } else {
                    "unknown".to_string()
                };

                return Some(AntDependency {
                    group_id,
                    artifact_id,
                    version,
                    scope: "compile".to_string(),
                    source: DependencySource::ManualJar,
                });
            }
        }

        // Fallback: just use filename as artifact_id with unknown version
        Some(AntDependency {
            group_id: "unknown".to_string(),
            artifact_id: filename.to_string(),
            version: "unknown".to_string(),
            scope: "compile".to_string(),
            source: DependencySource::ManualJar,
        })
    }

    /// Generate SBOM from dependencies
    pub fn generate_sbom(&self, dependencies: &[AntDependency]) -> Result<AntSbom> {
        Ok(AntSbom {
            project_name: self.get_project_name()?,
            project_version: self.get_project_version()?,
            build_file: self.build_file.clone(),
            ivy_file: self.ivy_file.clone(),
            dependencies: dependencies.to_vec(),
        })
    }

    /// Extract project name from build.xml
    fn get_project_name(&self) -> Result<String> {
        let content = fs::read_to_string(&self.build_file)?;

        // Simple regex-like parsing for project name attribute
        if let Some(start) = content.find("<project") {
            let project_tag = &content[start..];
            if let Some(name_start) = project_tag.find("name=\"") {
                let after_name = &project_tag[name_start + 6..];
                if let Some(name_end) = after_name.find('"') {
                    return Ok(after_name[..name_end].to_string());
                }
            }
        }

        Ok("unknown-ant-project".to_string())
    }

    /// Extract project version from build.xml or ivy.xml
    fn get_project_version(&self) -> Result<String> {
        // Try ivy.xml first
        if let Some(ref ivy_file) = self.ivy_file {
            if let Ok(content) = fs::read_to_string(ivy_file) {
                if let Some(start) = content.find("<info") {
                    let info_tag = &content[start..];
                    if let Some(rev_start) = info_tag.find("revision=\"") {
                        let after_rev = &info_tag[rev_start + 10..];
                        if let Some(rev_end) = after_rev.find('"') {
                            return Ok(after_rev[..rev_end].to_string());
                        }
                    }
                }
            }
        }

        Ok("1.0.0".to_string())
    }
}

/// Ant dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntDependency {
    /// Maven group ID (org in Ivy)
    pub group_id: String,
    /// Maven artifact ID (name in Ivy)
    pub artifact_id: String,
    /// Version (rev in Ivy)
    pub version: String,
    /// Dependency scope/configuration
    pub scope: String,
    /// Source of dependency detection
    pub source: DependencySource,
}

/// Source of dependency information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencySource {
    /// From ivy.xml
    Ivy,
    /// From manual JAR file detection
    ManualJar,
}

/// Ant SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntSbom {
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Path to build.xml
    pub build_file: PathBuf,
    /// Path to ivy.xml (if present)
    pub ivy_file: Option<PathBuf>,
    /// List of dependencies
    pub dependencies: Vec<AntDependency>,
}

/// Convert Ant dependencies to Maven-style coordinates for vulnerability scanning
pub fn ant_to_maven_coordinates(dep: &AntDependency) -> String {
    format!("{}:{}:{}", dep.group_id, dep.artifact_id, dep.version)
}

/// Extract Ant project and generate SBOM (main entry point)
pub fn extract_ant_sbom(project_root: &Path) -> Result<Option<AntSbom>> {
    let project = match AntProject::detect(project_root)? {
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
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_ant_project_detect_with_build_xml() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("build.xml"), "<project name=\"test\"/>").unwrap();

        let project = AntProject::detect(root).unwrap();
        assert!(project.is_some());

        let project = project.unwrap();
        assert_eq!(project.build_file, root.join("build.xml"));
        assert!(project.ivy_file.is_none());
    }

    #[test]
    fn test_ant_project_detect_with_ivy() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        fs::write(root.join("build.xml"), "<project name=\"test\"/>").unwrap();
        fs::write(root.join("ivy.xml"), "<ivy-module version=\"2.0\"/>").unwrap();

        let project = AntProject::detect(root).unwrap();
        assert!(project.is_some());

        let project = project.unwrap();
        assert!(project.ivy_file.is_some());
    }

    #[test]
    fn test_ant_project_not_detected() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let project = AntProject::detect(root).unwrap();
        assert!(project.is_none());
    }

    #[test]
    fn test_parse_jar_filename_with_version() {
        let temp_dir = TempDir::new().unwrap();
        let project = AntProject {
            root: temp_dir.path().to_path_buf(),
            ivy_file: None,
            build_file: temp_dir.path().join("build.xml"),
        };

        let jar_path = PathBuf::from("commons-lang3-3.12.0.jar");
        let dep = project.parse_jar_filename(&jar_path).unwrap();

        assert_eq!(dep.group_id, "commons");
        assert_eq!(dep.artifact_id, "commons-lang3");
        assert_eq!(dep.version, "3.12.0");
        assert_eq!(dep.scope, "compile");
    }

    #[test]
    fn test_parse_jar_filename_simple() {
        let temp_dir = TempDir::new().unwrap();
        let project = AntProject {
            root: temp_dir.path().to_path_buf(),
            ivy_file: None,
            build_file: temp_dir.path().join("build.xml"),
        };

        let jar_path = PathBuf::from("simple-1.0.jar");
        let dep = project.parse_jar_filename(&jar_path).unwrap();

        assert_eq!(dep.artifact_id, "simple");
        assert_eq!(dep.version, "1.0");
    }

    #[test]
    fn test_ant_to_maven_coordinates() {
        let dep = AntDependency {
            group_id: "org.apache.commons".to_string(),
            artifact_id: "commons-lang3".to_string(),
            version: "3.12.0".to_string(),
            scope: "compile".to_string(),
            source: DependencySource::Ivy,
        };

        let coords = ant_to_maven_coordinates(&dep);
        assert_eq!(coords, "org.apache.commons:commons-lang3:3.12.0");
    }

    #[test]
    fn test_dependency_source_equality() {
        assert_eq!(DependencySource::Ivy, DependencySource::Ivy);
        assert_ne!(DependencySource::Ivy, DependencySource::ManualJar);
    }

    #[test]
    fn test_ant_sbom_structure() {
        let sbom = AntSbom {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            build_file: PathBuf::from("build.xml"),
            ivy_file: None,
            dependencies: vec![],
        };

        assert_eq!(sbom.project_name, "test-project");
        assert_eq!(sbom.project_version, "1.0.0");
        assert_eq!(sbom.dependencies.len(), 0);
    }
}
