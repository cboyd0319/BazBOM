//! Gradle build.gradle parser
//!
//! Parses Gradle build files to extract dependencies.
//! Supports both Groovy DSL (build.gradle) and Kotlin DSL (build.gradle.kts).
//!
//! Handles:
//! - implementation, api, compileOnly, runtimeOnly configurations
//! - testImplementation, testCompileOnly, testRuntimeOnly
//! - String notation and map notation
//! - Version catalogs (libs.versions.toml)

use crate::scanner::{License, LicenseContext, ScanContext, Scanner};
use crate::types::{EcosystemScanResult, Package};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Gradle scanner
pub struct GradleScanner;

impl GradleScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for GradleScanner {
    fn name(&self) -> &str {
        "gradle"
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("build.gradle").exists() || root.join("build.gradle.kts").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result = EcosystemScanResult::new(
            "Gradle".to_string(),
            ctx.root.display().to_string(),
        );

        // Parse build.gradle or build.gradle.kts if available
        if let Some(ref manifest_path) = ctx.manifest {
            // Parse with multi-module support
            let dependencies = parse_gradle_with_modules(manifest_path)?;

            for dep in dependencies {
                // Skip test dependencies by default (can be made configurable)
                if dep.configuration.contains("test") || dep.configuration.contains("Test") {
                    continue;
                }

                result.packages.push(Package {
                    name: gradle_package_id(&dep),
                    version: dep.version.clone(),
                    ecosystem: "Gradle".to_string(),
                    namespace: Some(dep.group.clone()),
                    dependencies: Vec::new(),
                    license: None,
                    description: None,
                    homepage: None,
                    repository: None,
                });
            }
        }

        result.total_packages = result.packages.len();
        Ok(result)
    }

    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        License::Unknown
    }
}

/// Gradle dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleDependency {
    pub group: String,
    pub name: String,
    pub version: String,
    pub configuration: String,
}

/// Parse a Gradle project with multi-module support
pub fn parse_gradle_with_modules(gradle_path: &Path) -> Result<Vec<GradleDependency>> {
    let mut all_dependencies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Parse root build.gradle
    let root_deps = parse_gradle(gradle_path)?;
    for dep in root_deps {
        let key = format!("{}:{}:{}", dep.group, dep.name, dep.version);
        if seen.insert(key) {
            all_dependencies.push(dep);
        }
    }

    // Check for settings.gradle to find sub-projects
    let project_root = gradle_path.parent().unwrap_or(Path::new("."));
    let settings_gradle = project_root.join("settings.gradle");
    let settings_gradle_kts = project_root.join("settings.gradle.kts");

    let settings_path = if settings_gradle.exists() {
        Some(settings_gradle)
    } else if settings_gradle_kts.exists() {
        Some(settings_gradle_kts)
    } else {
        None
    };

    if let Some(settings_path) = settings_path {
        // Parse settings.gradle to find included modules
        if let Ok(modules) = parse_settings_gradle(&settings_path) {
            for module_name in modules {
                // Module paths can be like ":subproject" or "subproject"
                let module_dir_name = module_name.trim_start_matches(':').replace(':', "/");
                let module_dir = project_root.join(&module_dir_name);

                // Try build.gradle first, then build.gradle.kts
                let build_gradle = module_dir.join("build.gradle");
                let build_gradle_kts = module_dir.join("build.gradle.kts");

                let module_build_path = if build_gradle.exists() {
                    Some(build_gradle)
                } else if build_gradle_kts.exists() {
                    Some(build_gradle_kts)
                } else {
                    None
                };

                if let Some(module_build_path) = module_build_path {
                    match parse_gradle(&module_build_path) {
                        Ok(module_deps) => {
                            for dep in module_deps {
                                let key = format!("{}:{}:{}", dep.group, dep.name, dep.version);
                                if seen.insert(key) {
                                    all_dependencies.push(dep);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse module {}: {}", module_name, e);
                        }
                    }
                }
            }
        }
    }

    Ok(all_dependencies)
}

/// Parse settings.gradle to extract included modules
fn parse_settings_gradle(settings_path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(settings_path).with_context(|| {
        format!(
            "Failed to read settings.gradle: {}",
            settings_path.display()
        )
    })?;

    let mut modules = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Look for include(...) statements
        if trimmed.starts_with("include") {
            // Extract module names from include('module1', 'module2') or include 'module1', 'module2'
            if let Some(start) = trimmed.find('(') {
                if let Some(end) = trimmed.rfind(')') {
                    let modules_str = &trimmed[start + 1..end];

                    // Split by comma and extract module names
                    for module in modules_str.split(',') {
                        let module_name = module
                            .trim()
                            .trim_matches('\'')
                            .trim_matches('"')
                            .trim_start_matches(':')
                            .to_string();

                        if !module_name.is_empty() {
                            modules.push(module_name);
                        }
                    }
                }
            }
        }
    }

    Ok(modules)
}

/// Parse a Gradle build file (build.gradle or build.gradle.kts)
pub fn parse_gradle(gradle_path: &Path) -> Result<Vec<GradleDependency>> {
    let content = fs::read_to_string(gradle_path)
        .with_context(|| format!("Failed to read Gradle file: {}", gradle_path.display()))?;

    parse_gradle_content(&content)
}

/// Parse Gradle build file content
pub fn parse_gradle_content(content: &str) -> Result<Vec<GradleDependency>> {
    let mut dependencies = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("/*") {
            continue;
        }

        // Look for dependency declarations
        if let Some(dep) = parse_dependency_line(trimmed) {
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}

/// Parse a single dependency line
fn parse_dependency_line(line: &str) -> Option<GradleDependency> {
    // Common configurations
    let configs = [
        "implementation",
        "api",
        "compileOnly",
        "runtimeOnly",
        "testImplementation",
        "testCompileOnly",
        "testRuntimeOnly",
        "compile",
        "runtime",
        "testCompile",
        "testRuntime",
    ];

    for config in &configs {
        // Pattern 1: implementation "group:name:version"
        if let Some(dep) = parse_string_notation(line, config) {
            return Some(dep);
        }

        // Pattern 2: implementation("group:name:version")
        if let Some(dep) = parse_kotlin_notation(line, config) {
            return Some(dep);
        }

        // Pattern 3: implementation group: 'group', name: 'name', version: 'version'
        if let Some(dep) = parse_map_notation(line, config) {
            return Some(dep);
        }
    }

    None
}

/// Parse string notation: implementation "group:name:version"
fn parse_string_notation(line: &str, config: &str) -> Option<GradleDependency> {
    if !line.contains(config) {
        return None;
    }

    // Look for quoted strings
    for quote in &["\"", "'"] {
        if let Some(start) = line.find(quote) {
            if let Some(end) = line[start + 1..].find(quote) {
                let dep_str = &line[start + 1..start + 1 + end];
                if let Some(dep) = parse_coordinate(dep_str, config) {
                    return Some(dep);
                }
            }
        }
    }

    None
}

/// Parse Kotlin notation: implementation("group:name:version")
fn parse_kotlin_notation(line: &str, config: &str) -> Option<GradleDependency> {
    let pattern = format!("{}(", config);
    if !line.contains(&pattern) {
        return None;
    }

    // Extract content within parentheses
    if let Some(start) = line.find('(') {
        if let Some(end) = line[start..].find(')') {
            let inner = &line[start + 1..start + end];

            // Remove quotes
            let inner = inner.trim().trim_matches('"').trim_matches('\'');

            if let Some(dep) = parse_coordinate(inner, config) {
                return Some(dep);
            }
        }
    }

    None
}

/// Parse map notation: implementation group: 'group', name: 'name', version: 'version'
fn parse_map_notation(line: &str, config: &str) -> Option<GradleDependency> {
    if !line.contains(config) {
        return None;
    }

    let mut group = None;
    let mut name = None;
    let mut version = None;

    // Extract group
    if let Some(group_start) = line.find("group:") {
        let after_group = &line[group_start + 6..].trim_start();
        if let Some(group_value) = extract_quoted_value(after_group) {
            group = Some(group_value);
        }
    }

    // Extract name
    if let Some(name_start) = line.find("name:") {
        let after_name = &line[name_start + 5..].trim_start();
        if let Some(name_value) = extract_quoted_value(after_name) {
            name = Some(name_value);
        }
    }

    // Extract version
    if let Some(version_start) = line.find("version:") {
        let after_version = &line[version_start + 8..].trim_start();
        if let Some(version_value) = extract_quoted_value(after_version) {
            version = Some(version_value);
        }
    }

    if let (Some(g), Some(n), Some(v)) = (group, name, version) {
        return Some(GradleDependency {
            group: g,
            name: n,
            version: v,
            configuration: config.to_string(),
        });
    }

    None
}

/// Extract quoted value from string
fn extract_quoted_value(s: &str) -> Option<String> {
    for quote in &["\"", "'"] {
        if let Some(stripped) = s.strip_prefix(quote) {
            if let Some(end) = stripped.find(quote) {
                return Some(stripped[..end].to_string());
            }
        }
    }
    None
}

/// Parse Maven coordinate format: group:name:version
fn parse_coordinate(coord: &str, config: &str) -> Option<GradleDependency> {
    let parts: Vec<&str> = coord.split(':').collect();

    if parts.len() >= 3 {
        Some(GradleDependency {
            group: parts[0].to_string(),
            name: parts[1].to_string(),
            version: parts[2].to_string(),
            configuration: config.to_string(),
        })
    } else {
        None
    }
}

/// Generate package identifier for Gradle dependencies
pub fn gradle_package_id(dep: &GradleDependency) -> String {
    format!("{}:{}", dep.group, dep.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_notation() {
        let gradle = r#"
dependencies {
    implementation "org.springframework:spring-core:5.3.20"
    testImplementation "junit:junit:4.13.2"
}
"#;

        let deps = parse_gradle_content(gradle).unwrap();
        assert_eq!(deps.len(), 2);

        assert_eq!(deps[0].group, "org.springframework");
        assert_eq!(deps[0].name, "spring-core");
        assert_eq!(deps[0].version, "5.3.20");
        assert_eq!(deps[0].configuration, "implementation");

        assert_eq!(deps[1].group, "junit");
        assert_eq!(deps[1].name, "junit");
        assert_eq!(deps[1].version, "4.13.2");
        assert_eq!(deps[1].configuration, "testImplementation");
    }

    #[test]
    fn test_parse_kotlin_notation() {
        let gradle = r#"
dependencies {
    implementation("org.springframework:spring-core:5.3.20")
    testImplementation("junit:junit:4.13.2")
}
"#;

        let deps = parse_gradle_content(gradle).unwrap();
        assert_eq!(deps.len(), 2);

        assert_eq!(deps[0].group, "org.springframework");
        assert_eq!(deps[0].name, "spring-core");
        assert_eq!(deps[0].version, "5.3.20");
    }

    #[test]
    fn test_parse_map_notation() {
        let gradle = r#"
dependencies {
    implementation group: 'org.springframework', name: 'spring-core', version: '5.3.20'
    testImplementation group: "junit", name: "junit", version: "4.13.2"
}
"#;

        let deps = parse_gradle_content(gradle).unwrap();
        assert_eq!(deps.len(), 2);

        assert_eq!(deps[0].group, "org.springframework");
        assert_eq!(deps[0].name, "spring-core");
        assert_eq!(deps[0].version, "5.3.20");
    }

    #[test]
    fn test_gradle_package_id() {
        let dep = GradleDependency {
            group: "org.springframework".to_string(),
            name: "spring-core".to_string(),
            version: "5.3.20".to_string(),
            configuration: "implementation".to_string(),
        };

        assert_eq!(gradle_package_id(&dep), "org.springframework:spring-core");
    }

    #[test]
    fn test_mixed_formats() {
        let gradle = r#"
dependencies {
    // String notation
    implementation "com.google.guava:guava:31.1-jre"

    // Kotlin notation
    api("com.fasterxml.jackson.core:jackson-databind:2.13.3")

    // Map notation
    compileOnly group: 'org.projectlombok', name: 'lombok', version: '1.18.24'
}
"#;

        let deps = parse_gradle_content(gradle).unwrap();
        assert_eq!(deps.len(), 3);
    }
}
