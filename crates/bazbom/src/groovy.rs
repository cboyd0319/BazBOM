//! Groovy language support for BazBOM
//!
//! Provides enhanced Groovy dependency detection including:
//! - Grape (@Grab) annotations
//! - Groovy script dependencies
//! - GrapeConfig.xml parsing
//! - Dependency resolution from Groovy scripts

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Groovy project structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroovyProject {
    /// Project root directory
    pub root: PathBuf,
    /// Groovy scripts with dependencies
    pub scripts: Vec<GroovyScript>,
    /// Grape configuration
    pub grape_config: Option<GrapeConfig>,
}

/// Groovy script with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroovyScript {
    /// Script file path
    pub path: PathBuf,
    /// Dependencies declared via @Grab
    pub grab_dependencies: Vec<GrabDependency>,
}

/// Dependency declared via @Grab annotation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrabDependency {
    /// Group ID (organization)
    pub group: String,
    /// Artifact ID (module name)
    pub module: String,
    /// Version
    pub version: String,
    /// Classifier (optional)
    pub classifier: Option<String>,
    /// Extension (jar, war, etc.)
    pub ext: Option<String>,
}

impl std::fmt::Display for GrabDependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.group, self.module, self.version)
    }
}

/// Grape configuration from grapeConfig.xml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrapeConfig {
    /// Repository URLs
    pub repositories: Vec<String>,
    /// System properties
    pub system_properties: HashMap<String, String>,
}

/// Detect Groovy project
pub fn detect_groovy_project<P: AsRef<Path>>(root: P) -> Option<GroovyProject> {
    let root = root.as_ref();

    // Check for Groovy scripts or Grape config
    let has_groovy_files = find_groovy_scripts(root)
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    let grape_config_path = root.join(".groovy/grapeConfig.xml");
    let has_grape_config = grape_config_path.exists();

    if !has_groovy_files && !has_grape_config {
        return None;
    }

    let scripts = find_groovy_scripts(root).unwrap_or_default();
    let grape_config = if has_grape_config {
        parse_grape_config(&grape_config_path).ok()
    } else {
        None
    };

    Some(GroovyProject {
        root: root.to_path_buf(),
        scripts,
        grape_config,
    })
}

/// Find all Groovy scripts in the project
fn find_groovy_scripts(root: &Path) -> Result<Vec<GroovyScript>> {
    let mut scripts = Vec::new();

    // Recursively walk the directory tree looking for .groovy files
    find_groovy_scripts_recursive(root, &mut scripts, 0, 10)?;

    Ok(scripts)
}

/// Recursively find Groovy scripts (helper function)
fn find_groovy_scripts_recursive(
    dir: &Path,
    scripts: &mut Vec<GroovyScript>,
    depth: usize,
    max_depth: usize,
) -> Result<()> {
    if depth > max_depth {
        return Ok(());
    }

    // Read directory entries
    let entries = fs::read_dir(dir).context("Failed to read directory")?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories and common ignore patterns
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with('.')
                    || name == "target"
                    || name == "build"
                    || name == "node_modules"
                {
                    continue;
                }
            }

            // Recurse into subdirectory
            find_groovy_scripts_recursive(&path, scripts, depth + 1, max_depth)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("groovy") {
            // Parse the script for @Grab annotations
            if let Ok(grab_deps) = parse_grab_annotations(&path) {
                if !grab_deps.is_empty() {
                    scripts.push(GroovyScript {
                        path: path.clone(),
                        grab_dependencies: grab_deps,
                    });
                }
            }
        }
    }

    Ok(())
}

/// Parse @Grab annotations from a Groovy script
fn parse_grab_annotations(script_path: &Path) -> Result<Vec<GrabDependency>> {
    let content = fs::read_to_string(script_path).context("Failed to read Groovy script")?;

    let mut dependencies = Vec::new();

    // Look for @Grab annotations
    // Formats:
    // @Grab('group:module:version')
    // @Grab(group='group', module='module', version='version')
    // @Grapes([@Grab(...), @Grab(...)])

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with("//") || line.starts_with("/*") {
            continue;
        }

        // Look for @Grab annotation
        if line.contains("@Grab") {
            if let Some(dep) = parse_grab_line(line) {
                dependencies.push(dep);
            }
        }
    }

    Ok(dependencies)
}

/// Parse a single @Grab annotation line
fn parse_grab_line(line: &str) -> Option<GrabDependency> {
    // Check if it's long form (has "group=" pattern)
    if line.contains("group=") {
        // Handle long form: @Grab(group='group', module='module', version='version')
        if let Some(long_form) = extract_grab_long_form(line) {
            return Some(long_form);
        }
    }

    // Handle short form: @Grab('group:module:version')
    if let Some(short_form) = extract_grab_short_form(line) {
        return parse_maven_coordinate(&short_form);
    }

    None
}

/// Extract short form @Grab coordinate
fn extract_grab_short_form(line: &str) -> Option<String> {
    // Look for @Grab('...') or @Grab("...")
    if let Some(start) = line.find("@Grab(") {
        let after_grab = &line[start + 6..];

        // Find quote character
        if let Some(quote_start) = after_grab.find('\'').or_else(|| after_grab.find('"')) {
            let quote_char = after_grab.chars().nth(quote_start)?;
            let after_quote = &after_grab[quote_start + 1..];

            if let Some(quote_end) = after_quote.find(quote_char) {
                return Some(after_quote[..quote_end].to_string());
            }
        }
    }

    None
}

/// Extract long form @Grab parameters
fn extract_grab_long_form(line: &str) -> Option<GrabDependency> {
    // Parse @Grab(group='...', module='...', version='...')
    let group = extract_parameter(line, "group")?;
    let module = extract_parameter(line, "module")?;
    let version = extract_parameter(line, "version")?;

    let classifier = extract_parameter(line, "classifier");
    let ext = extract_parameter(line, "ext");

    Some(GrabDependency {
        group,
        module,
        version,
        classifier,
        ext,
    })
}

/// Extract a parameter value from @Grab annotation
fn extract_parameter(line: &str, param_name: &str) -> Option<String> {
    let pattern = format!("{}=", param_name);

    if let Some(start) = line.find(&pattern) {
        let after_equals = &line[start + pattern.len()..];

        // Skip whitespace
        let trimmed = after_equals.trim_start();

        // Find quote character (must be first char after trimming)
        if let Some(first_char) = trimmed.chars().next() {
            if first_char == '\'' || first_char == '"' {
                let after_quote = &trimmed[1..];

                if let Some(quote_end) = after_quote.find(first_char) {
                    return Some(after_quote[..quote_end].to_string());
                }
            }
        }
    }

    None
}

/// Parse Maven coordinate string (group:module:version[:classifier][@extension])
fn parse_maven_coordinate(coord: &str) -> Option<GrabDependency> {
    let parts: Vec<&str> = coord.split(':').collect();

    if parts.len() < 3 {
        return None;
    }

    let group = parts[0].to_string();
    let module = parts[1].to_string();
    let version_part = parts[2];

    // Check for @extension
    let (version, ext) = if let Some(at_pos) = version_part.find('@') {
        (
            version_part[..at_pos].to_string(),
            Some(version_part[at_pos + 1..].to_string()),
        )
    } else {
        (version_part.to_string(), None)
    };

    // Check for classifier in parts[3]
    let classifier = if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    };

    Some(GrabDependency {
        group,
        module,
        version,
        classifier,
        ext,
    })
}

/// Parse grapeConfig.xml
fn parse_grape_config(config_path: &Path) -> Result<GrapeConfig> {
    let content = fs::read_to_string(config_path).context("Failed to read grapeConfig.xml")?;

    // Simple XML parsing for repositories
    let mut repositories = Vec::new();
    let mut system_properties = HashMap::new();

    // Look for <repository> elements
    for line in content.lines() {
        let line = line.trim();

        if line.contains("<repository>") {
            // Extract URL from next lines
            // This is a simplified parser - production would use proper XML
            if let Some(url_start) = line.find("<url>") {
                if let Some(url_end) = line.find("</url>") {
                    let url = line[url_start + 5..url_end].to_string();
                    repositories.push(url);
                }
            }
        }

        // Look for system properties
        if line.contains("<property>") {
            // Extract name and value
            // Simplified - production would use proper XML
            if let (Some(name), Some(value)) = (
                extract_xml_tag(line, "name"),
                extract_xml_tag(line, "value"),
            ) {
                system_properties.insert(name, value);
            }
        }
    }

    // Add default Maven Central if no repositories configured
    if repositories.is_empty() {
        repositories.push("https://repo1.maven.org/maven2/".to_string());
    }

    Ok(GrapeConfig {
        repositories,
        system_properties,
    })
}

/// Extract XML tag content
fn extract_xml_tag(line: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    if let Some(start) = line.find(&open_tag) {
        if let Some(end) = line.find(&close_tag) {
            let content_start = start + open_tag.len();
            return Some(line[content_start..end].to_string());
        }
    }

    None
}

/// Generate SBOM for Groovy project
pub fn generate_groovy_sbom(project: &GroovyProject) -> Result<GroovySbom> {
    let mut dependencies = Vec::new();

    // Collect all @Grab dependencies from scripts
    for script in &project.scripts {
        for grab_dep in &script.grab_dependencies {
            dependencies.push(grab_dep.clone());
        }
    }

    // Deduplicate dependencies
    dependencies.sort_by(|a, b| {
        a.group
            .cmp(&b.group)
            .then(a.module.cmp(&b.module))
            .then(a.version.cmp(&b.version))
    });
    dependencies
        .dedup_by(|a, b| a.group == b.group && a.module == b.module && a.version == b.version);

    Ok(GroovySbom {
        project_root: project.root.clone(),
        dependencies,
        script_count: project.scripts.len(),
    })
}

/// SBOM for Groovy project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroovySbom {
    pub project_root: PathBuf,
    pub dependencies: Vec<GrabDependency>,
    pub script_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_grab_short_form() {
        let line = "@Grab('org.springframework:spring-core:5.3.0')";
        let dep = parse_grab_line(line).unwrap();

        assert_eq!(dep.group, "org.springframework");
        assert_eq!(dep.module, "spring-core");
        assert_eq!(dep.version, "5.3.0");
    }

    #[test]
    fn test_parse_grab_long_form() {
        let line = "@Grab(group='org.apache.commons', module='commons-lang3', version='3.12.0')";
        let dep = parse_grab_line(line).unwrap();

        assert_eq!(dep.group, "org.apache.commons");
        assert_eq!(dep.module, "commons-lang3");
        assert_eq!(dep.version, "3.12.0");
    }

    #[test]
    fn test_parse_maven_coordinate() {
        let coord = "com.google.guava:guava:31.1-jre";
        let dep = parse_maven_coordinate(coord).unwrap();

        assert_eq!(dep.group, "com.google.guava");
        assert_eq!(dep.module, "guava");
        assert_eq!(dep.version, "31.1-jre");
    }

    #[test]
    fn test_parse_maven_coordinate_with_classifier() {
        let coord = "org.example:mylib:1.0.0:sources";
        let dep = parse_maven_coordinate(coord).unwrap();

        assert_eq!(dep.group, "org.example");
        assert_eq!(dep.module, "mylib");
        assert_eq!(dep.version, "1.0.0");
        assert_eq!(dep.classifier, Some("sources".to_string()));
    }

    #[test]
    fn test_parse_maven_coordinate_with_extension() {
        let coord = "org.example:mylib:1.0.0@pom";
        let dep = parse_maven_coordinate(coord).unwrap();

        assert_eq!(dep.group, "org.example");
        assert_eq!(dep.module, "mylib");
        assert_eq!(dep.version, "1.0.0");
        assert_eq!(dep.ext, Some("pom".to_string()));
    }

    #[test]
    fn test_extract_parameter() {
        let line = "@Grab(group='org.test', module='test-lib', version='1.0')";

        assert_eq!(
            extract_parameter(line, "group"),
            Some("org.test".to_string())
        );
        assert_eq!(
            extract_parameter(line, "module"),
            Some("test-lib".to_string())
        );
        assert_eq!(extract_parameter(line, "version"), Some("1.0".to_string()));
    }

    #[test]
    fn test_grab_dependency_display() {
        let dep = GrabDependency {
            group: "org.example".to_string(),
            module: "test".to_string(),
            version: "1.0".to_string(),
            classifier: None,
            ext: None,
        };

        assert_eq!(dep.to_string(), "org.example:test:1.0");
    }

    #[test]
    fn test_detect_groovy_project_no_files() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let result = detect_groovy_project(temp_dir.path());

        assert!(result.is_none());
    }

    #[test]
    fn test_parse_grab_annotations_empty_script() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "println 'Hello, World!'").unwrap();

        let deps = parse_grab_annotations(temp_file.path()).unwrap();
        assert_eq!(deps.len(), 0);
    }

    #[test]
    fn test_parse_grab_annotations_with_grab() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "#!/usr/bin/env groovy").unwrap();
        writeln!(temp_file, "@Grab('commons-io:commons-io:2.11.0')").unwrap();
        writeln!(temp_file, "import org.apache.commons.io.FileUtils").unwrap();

        let deps = parse_grab_annotations(temp_file.path()).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].group, "commons-io");
        assert_eq!(deps[0].module, "commons-io");
        assert_eq!(deps[0].version, "2.11.0");
    }
}
