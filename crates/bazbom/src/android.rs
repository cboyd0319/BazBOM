// Android JVM artifact detection and SBOM generation
//
// Detects and analyzes Android projects focusing on JVM dependencies.
// NOTE: BazBOM is JVM-focused. This module handles Android APK/AAR JVM artifacts only.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Android project type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AndroidProjectType {
    Application, // APK
    Library,     // AAR
}

/// Android project
#[derive(Debug, Clone)]
pub struct AndroidProject {
    pub root: std::path::PathBuf,
    pub build_file: std::path::PathBuf,
    pub project_type: AndroidProjectType,
    pub package_name: Option<String>,
    pub min_sdk_version: Option<u32>,
    pub target_sdk_version: Option<u32>,
}

/// Android SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidSbom {
    pub project_name: String,
    pub project_version: String,
    pub package_name: Option<String>,
    pub project_type: String, // "application" or "library"
    pub min_sdk_version: Option<u32>,
    pub target_sdk_version: Option<u32>,
    pub dependencies: Vec<AndroidDependency>,
    pub android_dependencies: Vec<AndroidDependency>, // Android-specific (androidx, etc.)
}

/// Android dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidDependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub scope: String, // implementation, api, debugImplementation, etc.
    pub is_android_specific: bool,
}

/// Detect if a project is an Android project
pub fn is_android_project(project_root: &Path) -> bool {
    // Check for build.gradle or build.gradle.kts with Android plugin
    let build_gradle = project_root.join("build.gradle");
    let build_gradle_kts = project_root.join("build.gradle.kts");
    
    let build_file = if build_gradle_kts.exists() {
        Some(build_gradle_kts)
    } else if build_gradle.exists() {
        Some(build_gradle)
    } else {
        None
    };
    
    if let Some(build_file) = build_file {
        if let Ok(content) = fs::read_to_string(build_file) {
            // Look for Android Gradle plugins
            return content.contains("com.android.application")
                || content.contains("com.android.library")
                || content.contains("id(\"com.android.application\")")
                || content.contains("id(\"com.android.library\")")
                || content.contains("apply plugin: 'com.android.application'")
                || content.contains("apply plugin: 'com.android.library'");
        }
    }
    
    // Also check for AndroidManifest.xml
    let manifest = project_root.join("src/main/AndroidManifest.xml");
    manifest.exists()
}

/// Detect Android project details
pub fn detect_android_project(project_root: &Path) -> Option<AndroidProject> {
    if !is_android_project(project_root) {
        return None;
    }
    
    let build_gradle_kts = project_root.join("build.gradle.kts");
    let build_gradle = project_root.join("build.gradle");
    
    let build_file = if build_gradle_kts.exists() {
        build_gradle_kts
    } else if build_gradle.exists() {
        build_gradle
    } else {
        return None;
    };
    
    let content = fs::read_to_string(&build_file).ok()?;
    
    // Determine project type
    let project_type = if content.contains("com.android.application") ||
                          content.contains("id(\"com.android.application\")") {
        AndroidProjectType::Application
    } else {
        AndroidProjectType::Library
    };
    
    // Extract Android configuration
    let package_name = extract_package_name(project_root);
    let min_sdk_version = extract_min_sdk(&content);
    let target_sdk_version = extract_target_sdk(&content);
    
    Some(AndroidProject {
        root: project_root.to_path_buf(),
        build_file,
        project_type,
        package_name,
        min_sdk_version,
        target_sdk_version,
    })
}

/// Extract Android SBOM
pub fn extract_android_sbom(project: &AndroidProject) -> Result<AndroidSbom> {
    let content = fs::read_to_string(&project.build_file)
        .context("Failed to read build file")?;
    
    // Extract project metadata
    let project_name = extract_project_name(&project.root);
    let project_version = extract_version(&content);
    
    // Parse dependencies
    let mut all_dependencies = parse_dependencies(&content);
    
    // Separate Android-specific dependencies
    let android_dependencies: Vec<_> = all_dependencies
        .iter()
        .filter(|d| d.is_android_specific)
        .cloned()
        .collect();
    
    // Mark Android-specific status
    for dep in &mut all_dependencies {
        dep.is_android_specific = is_android_specific_artifact(dep);
    }
    
    let project_type_str = match project.project_type {
        AndroidProjectType::Application => "application",
        AndroidProjectType::Library => "library",
    };
    
    Ok(AndroidSbom {
        project_name,
        project_version,
        package_name: project.package_name.clone(),
        project_type: project_type_str.to_string(),
        min_sdk_version: project.min_sdk_version,
        target_sdk_version: project.target_sdk_version,
        dependencies: all_dependencies,
        android_dependencies,
    })
}

/// Extract package name from AndroidManifest.xml
fn extract_package_name(project_root: &Path) -> Option<String> {
    let manifest = project_root.join("src/main/AndroidManifest.xml");
    if !manifest.exists() {
        return None;
    }
    
    let content = fs::read_to_string(manifest).ok()?;
    
    // Look for package="..." in manifest tag
    for line in content.lines() {
        if line.contains("package=") {
            if let Some(start) = line.find("package=\"") {
                let start = start + 9; // len("package=\"")
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start + end].to_string());
                }
            }
        }
    }
    
    None
}

/// Extract minSdkVersion from build file
fn extract_min_sdk(content: &str) -> Option<u32> {
    // Look for minSdkVersion or minSdk
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("minSdkVersion") || trimmed.starts_with("minSdk") {
            // Extract number
            if let Some(num) = extract_number(trimmed) {
                return Some(num);
            }
        }
    }
    None
}

/// Extract targetSdkVersion from build file
fn extract_target_sdk(content: &str) -> Option<u32> {
    // Look for targetSdkVersion or targetSdk
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("targetSdkVersion") || trimmed.starts_with("targetSdk") {
            // Extract number
            if let Some(num) = extract_number(trimmed) {
                return Some(num);
            }
        }
    }
    None
}

/// Extract number from a line
fn extract_number(line: &str) -> Option<u32> {
    // Handle both = and () syntax
    // Examples: minSdkVersion = 21, minSdk(21), minSdk = 21
    
    let parts: Vec<&str> = if line.contains('=') {
        line.split('=').collect()
    } else if line.contains('(') {
        line.split('(').collect()
    } else {
        return None;
    };
    
    if parts.len() < 2 {
        return None;
    }
    
    // Get the number part and clean it
    let num_str = parts[1]
        .trim()
        .trim_matches(|c: char| !c.is_numeric());
    
    num_str.parse::<u32>().ok()
}

/// Extract project name from settings or directory
fn extract_project_name(project_root: &Path) -> String {
    // Try settings.gradle.kts
    let settings = project_root.join("settings.gradle.kts");
    if settings.exists() {
        if let Ok(content) = fs::read_to_string(settings) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("rootProject.name") {
                    if let Some(start) = trimmed.find('"') {
                        if let Some(end) = trimmed.rfind('"') {
                            if end > start {
                                return trimmed[start + 1..end].to_string();
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Fallback to directory name
    project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Extract version from build file
fn extract_version(content: &str) -> String {
    // Look for versionName or version
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("versionName") || (trimmed.starts_with("version") && !trimmed.starts_with("versionCode")) {
            // Extract version from quotes
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed.rfind('"') {
                    if end > start {
                        return trimmed[start + 1..end].to_string();
                    }
                }
            }
        }
    }
    "1.0.0".to_string()
}

/// Parse dependencies from build file
fn parse_dependencies(content: &str) -> Vec<AndroidDependency> {
    let mut dependencies = Vec::new();
    let mut in_dependencies_block = false;
    let mut brace_count = 0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Check if we're in dependencies block
        if trimmed == "dependencies {" || trimmed.starts_with("dependencies {") {
            in_dependencies_block = true;
            brace_count = 1;
            continue;
        }
        
        if in_dependencies_block {
            // Track braces
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;
            
            // Parse dependency lines
            if let Some(dep) = parse_dependency_line(trimmed) {
                dependencies.push(dep);
            }
            
            // Exit when braces are balanced
            if brace_count <= 0 {
                break;
            }
        }
    }
    
    dependencies
}

/// Parse a single dependency line
fn parse_dependency_line(line: &str) -> Option<AndroidDependency> {
    // Handle various formats:
    // implementation("androidx.core:core-ktx:1.12.0")
    // implementation 'com.google.android.material:material:1.10.0'
    // debugImplementation("androidx.test:runner:1.5.0")
    
    let scope = if line.starts_with("implementation") {
        "implementation"
    } else if line.starts_with("api") {
        "api"
    } else if line.starts_with("debugImplementation") {
        "debugImplementation"
    } else if line.starts_with("releaseImplementation") {
        "releaseImplementation"
    } else if line.starts_with("testImplementation") {
        "testImplementation"
    } else if line.starts_with("androidTestImplementation") {
        "androidTestImplementation"
    } else {
        return None;
    };
    
    // Extract dependency string between quotes or parentheses
    let dep_string = if let Some(start) = line.find('"') {
        let end = line.rfind('"')?;
        if end <= start {
            return None;
        }
        &line[start + 1..end]
    } else if let Some(start) = line.find('\'') {
        let end = line.rfind('\'')?;
        if end <= start {
            return None;
        }
        &line[start + 1..end]
    } else {
        return None;
    };
    
    // Parse group:artifact:version
    let parts: Vec<&str> = dep_string.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let group = parts[0].to_string();
    let artifact = parts[1].to_string();
    let version = if parts.len() >= 3 {
        parts[2].to_string()
    } else {
        "unspecified".to_string()
    };
    
    let dep = AndroidDependency {
        group: group.clone(),
        artifact: artifact.clone(),
        version,
        scope: scope.to_string(),
        is_android_specific: false, // Will be set later
    };
    
    Some(dep)
}

/// Check if an artifact is Android-specific
fn is_android_specific_artifact(dep: &AndroidDependency) -> bool {
    // Android-specific packages
    dep.group.starts_with("androidx.")
        || dep.group.starts_with("com.android.")
        || dep.group == "android"
        || dep.artifact.starts_with("android-")
}

/// Convert Android dependency to Maven coordinates
pub fn android_to_maven_coordinates(dep: &AndroidDependency) -> String {
    format!("{}:{}:{}", dep.group, dep.artifact, dep.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_android_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                id("com.android.application")
            }
        "#).unwrap();
        
        assert!(is_android_project(temp_dir.path()));
    }

    #[test]
    fn test_android_project_not_detected() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                id("java")
            }
        "#).unwrap();
        
        assert!(!is_android_project(temp_dir.path()));
    }

    #[test]
    fn test_detect_android_application() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                id("com.android.application")
            }
            
            android {
                minSdk = 21
                targetSdk = 34
            }
        "#).unwrap();
        
        let project = detect_android_project(temp_dir.path()).unwrap();
        assert_eq!(project.project_type, AndroidProjectType::Application);
        assert_eq!(project.min_sdk_version, Some(21));
        assert_eq!(project.target_sdk_version, Some(34));
    }

    #[test]
    fn test_parse_dependency_line() {
        let line = r#"implementation("androidx.core:core-ktx:1.12.0")"#;
        let dep = parse_dependency_line(line).unwrap();
        
        assert_eq!(dep.group, "androidx.core");
        assert_eq!(dep.artifact, "core-ktx");
        assert_eq!(dep.version, "1.12.0");
        assert_eq!(dep.scope, "implementation");
    }

    #[test]
    fn test_is_android_specific_artifact() {
        let androidx_dep = AndroidDependency {
            group: "androidx.core".to_string(),
            artifact: "core-ktx".to_string(),
            version: "1.12.0".to_string(),
            scope: "implementation".to_string(),
            is_android_specific: false,
        };
        assert!(is_android_specific_artifact(&androidx_dep));
        
        let regular_dep = AndroidDependency {
            group: "com.squareup.okhttp3".to_string(),
            artifact: "okhttp".to_string(),
            version: "4.12.0".to_string(),
            scope: "implementation".to_string(),
            is_android_specific: false,
        };
        assert!(!is_android_specific_artifact(&regular_dep));
    }

    #[test]
    fn test_android_to_maven_coordinates() {
        let dep = AndroidDependency {
            group: "androidx.appcompat".to_string(),
            artifact: "appcompat".to_string(),
            version: "1.6.1".to_string(),
            scope: "implementation".to_string(),
            is_android_specific: true,
        };
        
        let coords = android_to_maven_coordinates(&dep);
        assert_eq!(coords, "androidx.appcompat:appcompat:1.6.1");
    }

    #[test]
    fn test_extract_number() {
        assert_eq!(extract_number("minSdkVersion = 21"), Some(21));
        assert_eq!(extract_number("minSdk(21)"), Some(21));
        assert_eq!(extract_number("targetSdk = 34"), Some(34));
    }

    #[test]
    fn test_parse_dependencies() {
        let content = r#"
            dependencies {
                implementation("androidx.core:core-ktx:1.12.0")
                implementation("com.google.android.material:material:1.10.0")
                testImplementation("junit:junit:4.13.2")
            }
        "#;
        
        let deps = parse_dependencies(content);
        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].group, "androidx.core");
        assert_eq!(deps[1].artifact, "material");
        assert_eq!(deps[2].scope, "testImplementation");
    }

    #[test]
    fn test_android_sbom_structure() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                id("com.android.application")
            }
            
            android {
                namespace = "com.example.app"
                compileSdk = 34
                
                defaultConfig {
                    minSdk = 21
                    targetSdk = 34
                    versionName = "1.0.0"
                }
            }
            
            dependencies {
                implementation("androidx.core:core-ktx:1.12.0")
                implementation("com.squareup.okhttp3:okhttp:4.12.0")
            }
        "#).unwrap();
        
        // Create settings.gradle.kts
        let settings_file = temp_dir.path().join("settings.gradle.kts");
        fs::write(&settings_file, r#"
            rootProject.name = "MyAndroidApp"
        "#).unwrap();
        
        let project = detect_android_project(temp_dir.path()).unwrap();
        let sbom = extract_android_sbom(&project).unwrap();
        
        assert_eq!(sbom.project_name, "MyAndroidApp");
        assert_eq!(sbom.project_version, "1.0.0");
        assert_eq!(sbom.project_type, "application");
        assert_eq!(sbom.min_sdk_version, Some(21));
        assert_eq!(sbom.target_sdk_version, Some(34));
        assert_eq!(sbom.dependencies.len(), 2);
    }
}
