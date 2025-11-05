//! Clojure language support for BazBOM
//!
//! Provides Clojure dependency detection including:
//! - Leiningen (project.clj) dependency parsing
//! - tools.deps (deps.edn) dependency parsing
//! - SBOM generation for Clojure projects

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Clojure project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClojureProject {
    /// Project root directory
    pub root: PathBuf,
    /// Project type (Leiningen or tools.deps)
    pub project_type: ClojureProjectType,
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Dependencies
    pub dependencies: Vec<ClojureDependency>,
}

/// Type of Clojure project
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClojureProjectType {
    /// Leiningen project (project.clj)
    Leiningen,
    /// tools.deps project (deps.edn)
    ToolsDeps,
}

/// Clojure dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClojureDependency {
    /// Group ID (typically organization or author)
    pub group: String,
    /// Artifact ID (library name)
    pub artifact: String,
    /// Version
    pub version: String,
    /// Scope (compile, test, provided, etc.)
    pub scope: Option<String>,
}

impl std::fmt::Display for ClojureDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}:{}", self.group, self.artifact, self.version)
    }
}

/// Detect Clojure project
pub fn detect_clojure_project<P: AsRef<Path>>(root: P) -> Option<ClojureProject> {
    let root = root.as_ref();
    
    // Check for Leiningen (project.clj)
    let lein_project = root.join("project.clj");
    if lein_project.exists() {
        if let Ok(project) = parse_leiningen_project(&lein_project) {
            return Some(project);
        }
    }
    
    // Check for tools.deps (deps.edn)
    let deps_edn = root.join("deps.edn");
    if deps_edn.exists() {
        if let Ok(project) = parse_tools_deps_project(&deps_edn) {
            return Some(project);
        }
    }
    
    None
}

/// Parse Leiningen project.clj
fn parse_leiningen_project(project_file: &Path) -> Result<ClojureProject> {
    let content = fs::read_to_string(project_file)
        .context("Failed to read project.clj")?;
    
    // Parse Clojure EDN/code (simplified parser)
    // Real implementation would use proper EDN parser or invoke Leiningen
    
    let name = extract_project_name(&content).unwrap_or_else(|| "unknown".to_string());
    let version = extract_project_version(&content).unwrap_or_else(|| "0.0.0".to_string());
    let dependencies = parse_leiningen_dependencies(&content)?;
    
    Ok(ClojureProject {
        root: project_file.parent().unwrap().to_path_buf(),
        project_type: ClojureProjectType::Leiningen,
        name,
        version,
        dependencies,
    })
}

/// Parse tools.deps deps.edn
fn parse_tools_deps_project(deps_file: &Path) -> Result<ClojureProject> {
    let content = fs::read_to_string(deps_file)
        .context("Failed to read deps.edn")?;
    
    // Parse EDN format (simplified)
    // Real implementation would use proper EDN parser
    
    let dependencies = parse_tools_deps_dependencies(&content)?;
    
    // tools.deps doesn't have project name/version in deps.edn
    // Would need to check pom.xml or use directory name
    let name = deps_file
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();
    
    Ok(ClojureProject {
        root: deps_file.parent().unwrap().to_path_buf(),
        project_type: ClojureProjectType::ToolsDeps,
        name,
        version: "0.0.0".to_string(),
        dependencies,
    })
}

/// Extract project name from project.clj
fn extract_project_name(content: &str) -> Option<String> {
    // Look for (defproject name ...)
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("(defproject") {
            // Extract first symbol after defproject
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[1].trim_matches('"');
                return Some(name.to_string());
            }
        }
    }
    None
}

/// Extract project version from project.clj
fn extract_project_version(content: &str) -> Option<String> {
    // Look for (defproject name "version" ...)
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("(defproject") {
            // Extract second symbol after defproject (version)
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let version = parts[2].trim_matches('"');
                return Some(version.to_string());
            }
        }
    }
    None
}

/// Parse dependencies from Leiningen project.clj
fn parse_leiningen_dependencies(content: &str) -> Result<Vec<ClojureDependency>> {
    let mut dependencies = Vec::new();
    let mut in_dependencies = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Look for :dependencies vector
        if line.contains(":dependencies") {
            in_dependencies = true;
            // Check if dependencies start on same line
            if line.contains("[[") {
                // Dependencies on same line as :dependencies
                let after_deps = line.split("[[").nth(1).unwrap_or("");
                for dep_part in after_deps.split('[') {
                    if !dep_part.trim().is_empty() {
                        let dep_line = format!("[{}", dep_part);
                        if let Some(dep) = parse_leiningen_dependency_line(&dep_line) {
                            dependencies.push(dep);
                        }
                    }
                }
            }
            continue;
        }
        
        if in_dependencies {
            // End of dependencies section
            if line.starts_with(']') {
                in_dependencies = false;
                continue;
            }
            
            // Parse dependency: [org.clojure/clojure "1.11.1"]
            if line.contains('[') && !line.contains(":dependencies") {
                if let Some(dep) = parse_leiningen_dependency_line(line) {
                    dependencies.push(dep);
                }
            }
        }
    }
    
    Ok(dependencies)
}

/// Parse single Leiningen dependency line
fn parse_leiningen_dependency_line(line: &str) -> Option<ClojureDependency> {
    // Format: [org.clojure/clojure "1.11.1"]
    // or: [org.clojure/clojure "1.11.1" :scope "test"]
    
    let line = line.trim_matches(|c| c == '[' || c == ']');
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    // Parse group/artifact
    let coord = parts[0];
    let (group, artifact) = if let Some(slash_pos) = coord.find('/') {
        (
            coord[..slash_pos].to_string(),
            coord[slash_pos + 1..].to_string(),
        )
    } else {
        // Single name like "clojure" implies group=artifact
        (coord.to_string(), coord.to_string())
    };
    
    // Parse version
    let version = if parts.len() >= 2 {
        parts[1].trim_matches('"').to_string()
    } else {
        "LATEST".to_string()
    };
    
    // Parse scope if present
    let scope = parse_dependency_scope(&parts);
    
    Some(ClojureDependency {
        group,
        artifact,
        version,
        scope,
    })
}

/// Parse dependency scope from parts
fn parse_dependency_scope(parts: &[&str]) -> Option<String> {
    // Look for :scope "test" pattern
    for i in 0..parts.len().saturating_sub(1) {
        if parts[i] == ":scope" {
            return Some(parts[i + 1].trim_matches('"').to_string());
        }
    }
    None
}

/// Parse dependencies from tools.deps deps.edn
fn parse_tools_deps_dependencies(content: &str) -> Result<Vec<ClojureDependency>> {
    let mut dependencies = Vec::new();
    let mut in_deps = false;
    
    for line in content.lines() {
        let line = line.trim();
        
        // Look for :deps map
        if line.contains(":deps") {
            in_deps = true;
            continue;
        }
        
        if in_deps {
            // End of deps section
            if line.starts_with('}') {
                in_deps = false;
                continue;
            }
            
            // Parse dependency: org.clojure/clojure {:mvn/version "1.11.1"}
            if let Some(dep) = parse_tools_deps_dependency_line(line) {
                dependencies.push(dep);
            }
        }
    }
    
    Ok(dependencies)
}

/// Parse single tools.deps dependency line
fn parse_tools_deps_dependency_line(line: &str) -> Option<ClojureDependency> {
    // Format: org.clojure/clojure {:mvn/version "1.11.1"}
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.is_empty() {
        return None;
    }
    
    // Parse group/artifact
    let coord = parts[0];
    let (group, artifact) = if let Some(slash_pos) = coord.find('/') {
        (
            coord[..slash_pos].to_string(),
            coord[slash_pos + 1..].to_string(),
        )
    } else {
        (coord.to_string(), coord.to_string())
    };
    
    // Parse version from {:mvn/version "1.11.1"}
    let version = extract_mvn_version(line).unwrap_or_else(|| "LATEST".to_string());
    
    Some(ClojureDependency {
        group,
        artifact,
        version,
        scope: None,
    })
}

/// Extract Maven version from tools.deps line
fn extract_mvn_version(line: &str) -> Option<String> {
    // Look for :mvn/version "1.11.1"
    if let Some(version_pos) = line.find(":mvn/version") {
        let after_version = &line[version_pos + 12..]; // Skip ":mvn/version"
        
        // Find first quote
        if let Some(quote_start) = after_version.find('"') {
            let after_quote = &after_version[quote_start + 1..];
            
            // Find closing quote
            if let Some(quote_end) = after_quote.find('"') {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }
    None
}

/// Convert Clojure dependency to Maven coordinates
pub fn clojure_to_maven_coordinates(dep: &ClojureDependency) -> String {
    format!("{}:{}:{}", dep.group, dep.artifact, dep.version)
}

/// Generate SBOM for Clojure project
pub fn generate_clojure_sbom(project: &ClojureProject) -> Result<ClojureSbom> {
    Ok(ClojureSbom {
        project_name: project.name.clone(),
        project_version: project.version.clone(),
        project_type: project.project_type.clone(),
        dependencies: project.dependencies.clone(),
    })
}

/// SBOM for Clojure project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClojureSbom {
    pub project_name: String,
    pub project_version: String,
    pub project_type: ClojureProjectType,
    pub dependencies: Vec<ClojureDependency>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_extract_project_name() {
        let content = "(defproject my-app \"0.1.0-SNAPSHOT\"\n  :description \"Test app\"";
        assert_eq!(extract_project_name(content), Some("my-app".to_string()));
    }
    
    #[test]
    fn test_extract_project_version() {
        let content = "(defproject my-app \"0.1.0-SNAPSHOT\"\n  :description \"Test app\"";
        assert_eq!(extract_project_version(content), Some("0.1.0-SNAPSHOT".to_string()));
    }
    
    #[test]
    fn test_parse_leiningen_dependency_line() {
        let line = "[org.clojure/clojure \"1.11.1\"]";
        let dep = parse_leiningen_dependency_line(line).unwrap();
        
        assert_eq!(dep.group, "org.clojure");
        assert_eq!(dep.artifact, "clojure");
        assert_eq!(dep.version, "1.11.1");
    }
    
    #[test]
    fn test_parse_leiningen_dependency_with_scope() {
        let line = "[org.clojure/test.check \"1.1.1\" :scope \"test\"]";
        let dep = parse_leiningen_dependency_line(line).unwrap();
        
        assert_eq!(dep.group, "org.clojure");
        assert_eq!(dep.artifact, "test.check");
        assert_eq!(dep.version, "1.1.1");
        assert_eq!(dep.scope, Some("test".to_string()));
    }
    
    #[test]
    fn test_parse_tools_deps_dependency_line() {
        let line = "org.clojure/clojure {:mvn/version \"1.11.1\"}";
        let dep = parse_tools_deps_dependency_line(line).unwrap();
        
        assert_eq!(dep.group, "org.clojure");
        assert_eq!(dep.artifact, "clojure");
        assert_eq!(dep.version, "1.11.1");
    }
    
    #[test]
    fn test_extract_mvn_version() {
        let line = "org.clojure/clojure {:mvn/version \"1.11.1\"}";
        assert_eq!(extract_mvn_version(line), Some("1.11.1".to_string()));
    }
    
    #[test]
    fn test_clojure_dependency_display() {
        let dep = ClojureDependency {
            group: "org.clojure".to_string(),
            artifact: "clojure".to_string(),
            version: "1.11.1".to_string(),
            scope: None,
        };
        
        assert_eq!(dep.to_string(), "org.clojure/clojure:1.11.1");
    }
    
    #[test]
    fn test_clojure_to_maven_coordinates() {
        let dep = ClojureDependency {
            group: "org.clojure".to_string(),
            artifact: "clojure".to_string(),
            version: "1.11.1".to_string(),
            scope: None,
        };
        
        assert_eq!(clojure_to_maven_coordinates(&dep), "org.clojure:clojure:1.11.1");
    }
    
    #[test]
    fn test_parse_leiningen_project() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(
            temp_file,
            "(defproject my-app \"0.1.0-SNAPSHOT\"\n  :dependencies [[org.clojure/clojure \"1.11.1\"]\n                 [ring/ring-core \"1.9.5\"]])"
        )
        .unwrap();
        
        let project = parse_leiningen_project(temp_file.path()).unwrap();
        
        assert_eq!(project.name, "my-app");
        assert_eq!(project.version, "0.1.0-SNAPSHOT");
        assert_eq!(project.project_type, ClojureProjectType::Leiningen);
        assert_eq!(project.dependencies.len(), 2);
        assert_eq!(project.dependencies[0].group, "org.clojure");
        assert_eq!(project.dependencies[0].artifact, "clojure");
    }
    
    #[test]
    fn test_detect_clojure_project_no_files() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let result = detect_clojure_project(temp_dir.path());
        
        assert!(result.is_none());
    }
}
