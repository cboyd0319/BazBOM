// Kotlin Multiplatform (KMP) support for JVM targets
//
// Detects and generates SBOM for Kotlin Multiplatform projects targeting JVM.
// NOTE: BazBOM focuses exclusively on JVM ecosystems. Non-JVM targets (JS, Native, Wasm)
// are detected but not analyzed.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Kotlin Multiplatform project
#[derive(Debug, Clone)]
pub struct KotlinMultiplatformProject {
    pub root: std::path::PathBuf,
    pub build_file: std::path::PathBuf,
    pub has_jvm_target: bool,
    pub has_android_target: bool,
    pub other_targets: Vec<String>, // JS, Native, Wasm (informational only)
}

/// Kotlin Multiplatform SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmpSbom {
    pub project_name: String,
    pub project_version: String,
    pub jvm_dependencies: Vec<KmpDependency>,
    pub android_dependencies: Vec<KmpDependency>,
    pub targets: Vec<String>,
}

/// Kotlin Multiplatform dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KmpDependency {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub scope: String, // commonMain, jvmMain, androidMain, etc.
}

/// Detect if a project is a Kotlin Multiplatform project
pub fn is_kotlin_multiplatform(project_root: &Path) -> bool {
    // Check for build.gradle.kts or build.gradle
    let build_gradle_kts = project_root.join("build.gradle.kts");
    let build_gradle = project_root.join("build.gradle");
    
    let build_file = if build_gradle_kts.exists() {
        Some(build_gradle_kts)
    } else if build_gradle.exists() {
        Some(build_gradle)
    } else {
        None
    };
    
    if let Some(build_file) = build_file {
        if let Ok(content) = fs::read_to_string(build_file) {
            // Look for kotlin { } block with multiplatform plugin
            return content.contains("kotlin(\"multiplatform\")")
                || content.contains("id(\"org.jetbrains.kotlin.multiplatform\")")
                || content.contains("kotlin-multiplatform")
                || (content.contains("kotlin {") && 
                    (content.contains("jvm()") || content.contains("android()") || 
                     content.contains("js()") || content.contains("iosTarget")));
        }
    }
    
    false
}

/// Detect Kotlin Multiplatform project details
pub fn detect_kmp_project(project_root: &Path) -> Option<KotlinMultiplatformProject> {
    if !is_kotlin_multiplatform(project_root) {
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
    
    // Detect targets
    let has_jvm_target = content.contains("jvm()") || content.contains("jvm {");
    let has_android_target = content.contains("android()") || content.contains("android {");
    
    let mut other_targets = Vec::new();
    if content.contains("js(") || content.contains("js {") {
        other_targets.push("js".to_string());
    }
    if content.contains("iosTarget") || content.contains("ios()") {
        other_targets.push("ios".to_string());
    }
    if content.contains("linuxX64") || content.contains("macosX64") || content.contains("mingwX64") {
        other_targets.push("native".to_string());
    }
    if content.contains("wasm") {
        other_targets.push("wasm".to_string());
    }
    
    Some(KotlinMultiplatformProject {
        root: project_root.to_path_buf(),
        build_file,
        has_jvm_target,
        has_android_target,
        other_targets,
    })
}

/// Extract SBOM from Kotlin Multiplatform project (JVM targets only)
pub fn extract_kmp_sbom(project: &KotlinMultiplatformProject) -> Result<KmpSbom> {
    let content = fs::read_to_string(&project.build_file)
        .context("Failed to read build file")?;
    
    // Extract project name and version
    let project_name = extract_project_name(&content, &project.root);
    let project_version = extract_project_version(&content);
    
    // Collect target names
    let mut targets = Vec::new();
    if project.has_jvm_target {
        targets.push("jvm".to_string());
    }
    if project.has_android_target {
        targets.push("android".to_string());
    }
    targets.extend(project.other_targets.clone());
    
    // Parse dependencies from sourceSets
    let jvm_dependencies = parse_source_set_dependencies(&content, "jvmMain");
    let android_dependencies = parse_source_set_dependencies(&content, "androidMain");
    
    // Also parse common dependencies (shared across all targets)
    let common_dependencies = parse_source_set_dependencies(&content, "commonMain");
    
    // Combine common with JVM-specific
    let mut all_jvm_deps = common_dependencies.clone();
    all_jvm_deps.extend(jvm_dependencies);
    
    // Combine common with Android-specific
    let mut all_android_deps = common_dependencies;
    all_android_deps.extend(android_dependencies);
    
    Ok(KmpSbom {
        project_name,
        project_version,
        jvm_dependencies: all_jvm_deps,
        android_dependencies: all_android_deps,
        targets,
    })
}

/// Extract project name from build file or directory
fn extract_project_name(content: &str, project_root: &Path) -> String {
    // Try to find rootProject.name in settings.gradle.kts
    let settings_file = project_root.join("settings.gradle.kts");
    if settings_file.exists() {
        if let Ok(settings_content) = fs::read_to_string(settings_file) {
            if let Some(name) = parse_project_name(&settings_content) {
                return name;
            }
        }
    }
    
    // Try to find group in build.gradle.kts
    if let Some(name) = parse_project_name(content) {
        return name;
    }
    
    // Fallback to directory name
    project_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

/// Parse project name from Gradle script
fn parse_project_name(content: &str) -> Option<String> {
    // Look for rootProject.name = "..."
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("rootProject.name") {
            // Extract the name from quotes
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed.rfind('"') {
                    if end > start {
                        return Some(trimmed[start + 1..end].to_string());
                    }
                }
            }
        }
    }
    None
}

/// Extract project version from build file
fn extract_project_version(content: &str) -> String {
    // Look for version = "..." or version("...")
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("version") {
            // Extract the version from quotes
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

/// Parse dependencies from a specific source set
fn parse_source_set_dependencies(content: &str, source_set: &str) -> Vec<KmpDependency> {
    let mut dependencies = Vec::new();
    
    // Find the sourceSet block
    let source_set_pattern = format!("{}Dependencies", source_set);
    let mut in_source_set = false;
    let mut brace_count = 0;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        // Check if we're entering the source set dependencies block
        if trimmed.contains(&source_set_pattern) || trimmed.contains(&format!("val {} by getting", source_set)) {
            in_source_set = true;
        }
        
        if in_source_set {
            // Track braces to know when we exit the block
            brace_count += trimmed.matches('{').count() as i32;
            brace_count -= trimmed.matches('}').count() as i32;
            
            // Parse dependency lines: implementation("group:artifact:version")
            if trimmed.starts_with("implementation(") || 
                trimmed.starts_with("api(") || 
                trimmed.starts_with("compileOnly(") ||
                trimmed.starts_with("runtimeOnly(") {
                
                if let Some(dep) = parse_dependency_line(trimmed, source_set) {
                    dependencies.push(dep);
                }
            }
            
            // Exit when we've closed all braces
            if brace_count <= 0 && in_source_set {
                break;
            }
        }
    }
    
    dependencies
}

/// Parse a single dependency line
fn parse_dependency_line(line: &str, scope: &str) -> Option<KmpDependency> {
    // Extract content between quotes: implementation("group:artifact:version")
    let start = line.find('"')?;
    let end = line.rfind('"')?;
    if end <= start {
        return None;
    }
    
    let dep_string = &line[start + 1..end];
    let parts: Vec<&str> = dep_string.split(':').collect();
    
    if parts.len() >= 2 {
        let group = parts[0].to_string();
        let artifact = parts[1].to_string();
        let version = if parts.len() >= 3 {
            parts[2].to_string()
        } else {
            "unspecified".to_string()
        };
        
        Some(KmpDependency {
            group,
            artifact,
            version,
            scope: scope.to_string(),
        })
    } else {
        None
    }
}

/// Convert KMP dependency to Maven coordinates (for vulnerability scanning)
pub fn kmp_to_maven_coordinates(dep: &KmpDependency) -> String {
    format!("{}:{}:{}", dep.group, dep.artifact, dep.version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_kmp_project_detection() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                kotlin("multiplatform") version "1.9.0"
            }
            
            kotlin {
                jvm()
                js()
            }
        "#).unwrap();
        
        assert!(is_kotlin_multiplatform(temp_dir.path()));
    }

    #[test]
    fn test_kmp_project_not_detected() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                id("java")
            }
        "#).unwrap();
        
        assert!(!is_kotlin_multiplatform(temp_dir.path()));
    }

    #[test]
    fn test_detect_kmp_targets() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                kotlin("multiplatform")
            }
            
            kotlin {
                jvm()
                android()
                js()
                iosTarget()
            }
        "#).unwrap();
        
        let project = detect_kmp_project(temp_dir.path()).unwrap();
        assert!(project.has_jvm_target);
        assert!(project.has_android_target);
        assert!(project.other_targets.contains(&"js".to_string()));
        assert!(project.other_targets.contains(&"ios".to_string()));
    }

    #[test]
    fn test_parse_dependency_line() {
        let line = r#"implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.0")"#;
        let dep = parse_dependency_line(line, "jvmMain").unwrap();
        
        assert_eq!(dep.group, "org.jetbrains.kotlinx");
        assert_eq!(dep.artifact, "kotlinx-coroutines-core");
        assert_eq!(dep.version, "1.7.0");
        assert_eq!(dep.scope, "jvmMain");
    }

    #[test]
    fn test_kmp_to_maven_coordinates() {
        let dep = KmpDependency {
            group: "org.jetbrains.kotlin".to_string(),
            artifact: "kotlin-stdlib".to_string(),
            version: "1.9.0".to_string(),
            scope: "commonMain".to_string(),
        };
        
        let coords = kmp_to_maven_coordinates(&dep);
        assert_eq!(coords, "org.jetbrains.kotlin:kotlin-stdlib:1.9.0");
    }

    #[test]
    fn test_extract_project_name() {
        let content = r#"
            rootProject.name = "my-kmp-project"
        "#;
        let temp_dir = TempDir::new().unwrap();
        let name = extract_project_name(content, temp_dir.path());
        assert_eq!(name, "my-kmp-project");
    }

    #[test]
    fn test_extract_project_version() {
        let content = r#"
            version = "2.0.0"
        "#;
        let version = extract_project_version(content);
        assert_eq!(version, "2.0.0");
    }

    #[test]
    fn test_parse_source_set_dependencies() {
        let content = r#"
            kotlin {
                sourceSets {
                    val jvmMain by getting {
                        dependencies {
                            implementation("io.ktor:ktor-server-core:2.3.0")
                            api("org.jetbrains.kotlinx:kotlinx-serialization-json:1.5.0")
                        }
                    }
                }
            }
        "#;
        
        let deps = parse_source_set_dependencies(content, "jvmMain");
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].group, "io.ktor");
        assert_eq!(deps[0].artifact, "ktor-server-core");
        assert_eq!(deps[1].group, "org.jetbrains.kotlinx");
    }

    #[test]
    fn test_kmp_sbom_structure() {
        let temp_dir = TempDir::new().unwrap();
        let build_file = temp_dir.path().join("build.gradle.kts");
        
        fs::write(&build_file, r#"
            plugins {
                kotlin("multiplatform")
            }
            
            version = "1.0.0"
            
            kotlin {
                jvm()
                
                sourceSets {
                    val commonMain by getting {
                        dependencies {
                            implementation("org.jetbrains.kotlin:kotlin-stdlib:1.9.0")
                        }
                    }
                    val jvmMain by getting {
                        dependencies {
                            implementation("io.ktor:ktor-server-core:2.3.0")
                        }
                    }
                }
            }
        "#).unwrap();
        
        let settings_file = temp_dir.path().join("settings.gradle.kts");
        fs::write(&settings_file, r#"
            rootProject.name = "test-kmp"
        "#).unwrap();
        
        let project = detect_kmp_project(temp_dir.path()).unwrap();
        let sbom = extract_kmp_sbom(&project).unwrap();
        
        assert_eq!(sbom.project_name, "test-kmp");
        assert_eq!(sbom.project_version, "1.0.0");
        assert!(sbom.targets.contains(&"jvm".to_string()));
        assert_eq!(sbom.jvm_dependencies.len(), 2); // common + jvm
    }
}
