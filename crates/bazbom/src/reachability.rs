use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachabilityResult {
    pub tool: String,
    pub version: String,
    pub classpath: String,
    pub entrypoints: String,
    pub detected_entrypoints: Vec<String>,
    pub reachable_methods: Vec<String>,
    pub reachable_classes: Vec<String>,
    pub reachable_packages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ReachabilityResult {
    #[allow(dead_code)]
    pub fn is_class_reachable(&self, class_name: &str) -> bool {
        self.reachable_classes.iter().any(|c| c == class_name)
    }

    pub fn is_package_reachable(&self, package_name: &str) -> bool {
        // Check if the package is exactly in the reachable list
        self.reachable_packages.iter().any(|p| p == package_name)
    }

    #[allow(dead_code)]
    pub fn is_method_reachable(&self, method_signature: &str) -> bool {
        self.reachable_methods.iter().any(|m| m.contains(method_signature))
    }
}

/// Run reachability analysis using the bazbom-reachability.jar tool
pub fn analyze_reachability(
    jar_path: &Path,
    classpath: &str,
    entrypoints: &str,
    output_path: &Path,
) -> Result<ReachabilityResult> {
    println!("[bazbom] running reachability analysis");
    println!("[bazbom] jar: {:?}", jar_path);
    println!("[bazbom] classpath entries: {}", classpath.split(':').count());

    let status = Command::new("java")
        .arg("-jar")
        .arg(jar_path)
        .arg("--classpath")
        .arg(classpath)
        .arg("--entrypoints")
        .arg(entrypoints)
        .arg("--output")
        .arg(output_path)
        .status()
        .context("failed to execute reachability analyzer")?;

    if !status.success() {
        anyhow::bail!("reachability analyzer failed with status: {:?}", status);
    }

    // Read the output JSON
    let json_content = std::fs::read_to_string(output_path)
        .context("failed to read reachability output")?;

    let result: ReachabilityResult = serde_json::from_str(&json_content)
        .context("failed to parse reachability output")?;

    if let Some(error) = &result.error {
        eprintln!("[bazbom] reachability analysis error: {}", error);
    }

    println!("[bazbom] reachability complete:");
    println!("  - detected entrypoints: {}", result.detected_entrypoints.len());
    println!("  - reachable methods: {}", result.reachable_methods.len());
    println!("  - reachable classes: {}", result.reachable_classes.len());
    println!("  - reachable packages: {}", result.reachable_packages.len());

    Ok(result)
}

/// Extract classpath from Maven project
pub fn extract_maven_classpath(project_path: &Path) -> Result<String> {
    // Run mvn dependency:build-classpath to get the classpath
    let output = Command::new("mvn")
        .arg("dependency:build-classpath")
        .arg("-DincludeScope=runtime")
        .arg("-q")
        .current_dir(project_path)
        .output()
        .context("failed to run mvn dependency:build-classpath")?;

    if !output.status.success() {
        anyhow::bail!("mvn dependency:build-classpath failed");
    }

    let classpath = String::from_utf8(output.stdout)
        .context("invalid UTF-8 in Maven output")?
        .lines()
        .filter(|line| !line.starts_with('['))
        .collect::<Vec<_>>()
        .join("");

    Ok(classpath)
}

/// Extract classpath from Gradle project
pub fn extract_gradle_classpath(project_path: &Path) -> Result<String> {
    // Check if BazBOM Gradle plugin is applied
    // If so, run the bazbomClasspath task
    let classpath_file = project_path.join("build").join("bazbom-classpath.txt");
    
    // Try to run the bazbomClasspath task
    println!("[bazbom] Running Gradle bazbomClasspath task...");
    let output = Command::new("gradle")
        .arg("bazbomClasspath")
        .arg("-q")
        .current_dir(project_path)
        .output()
        .context("failed to run gradle bazbomClasspath")?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        println!("[bazbom] Warning: gradle bazbomClasspath failed: {}", error);
        println!("[bazbom] Make sure the BazBOM Gradle plugin is applied");
        return Ok(String::new());
    }

    // Read the classpath from the output file
    if classpath_file.exists() {
        let classpath = std::fs::read_to_string(&classpath_file)
            .context("failed to read gradle classpath file")?;
        println!("[bazbom] Extracted classpath with {} entries", 
                 classpath.split(':').count());
        Ok(classpath.trim().to_string())
    } else {
        println!("[bazbom] Warning: classpath file not created by Gradle task");
        Ok(String::new())
    }
}

/// Extract classpath from Bazel project
pub fn extract_bazel_classpath(project_path: &Path, _target: &str) -> Result<String> {
    // For Bazel projects, we need to extract JARs from the external repository
    // These are typically in bazel-bin and bazel-<workspace>/external/maven
    
    // Try to find the Bazel external directory
    let external_dir = project_path.join("bazel-bin").join("external").join("maven");
    if !external_dir.exists() {
        // Try alternate location
        let workspace_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("workspace");
        let alt_external = project_path
            .join(format!("bazel-{}", workspace_name))
            .join("external")
            .join("maven");
        
        if !alt_external.exists() {
            println!(
                "[bazbom] Warning: Could not find Bazel external directory at {:?} or {:?}",
                external_dir, alt_external
            );
            return Ok(String::new());
        }
    }
    
    // For now, we'll use a simpler approach via bazel query
    // In production, this should use proper aspects
    let output = Command::new("bazel")
        .arg("query")
        .arg("@maven//:all")
        .arg("--output=location")
        .current_dir(project_path)
        .output()
        .context("failed to run bazel query")?;

    if !output.status.success() {
        println!("[bazbom] Bazel query failed, classpath extraction requires aspect integration");
        return Ok(String::new());
    }

    // Parse output to extract JAR locations
    // This is a simplified implementation
    let output_str = String::from_utf8(output.stdout)
        .context("invalid UTF-8 in Bazel output")?;
    
    let jars: Vec<&str> = output_str
        .lines()
        .filter(|line| line.contains(".jar"))
        .collect();
    
    println!("[bazbom] Found {} JAR references from Bazel", jars.len());
    
    // Return empty for now - proper implementation needs aspect-based extraction
    Ok(String::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_class_reachable() {
        let result = ReachabilityResult {
            tool: "test".to_string(),
            version: "0.1.0".to_string(),
            classpath: "".to_string(),
            entrypoints: "".to_string(),
            detected_entrypoints: vec![],
            reachable_methods: vec![],
            reachable_classes: vec![
                "com.example.Main".to_string(),
                "com.example.Utils".to_string(),
            ],
            reachable_packages: vec![],
            error: None,
        };

        assert!(result.is_class_reachable("com.example.Main"));
        assert!(result.is_class_reachable("com.example.Utils"));
        assert!(!result.is_class_reachable("com.example.Other"));
    }

    #[test]
    fn test_is_package_reachable() {
        let result = ReachabilityResult {
            tool: "test".to_string(),
            version: "0.1.0".to_string(),
            classpath: "".to_string(),
            entrypoints: "".to_string(),
            detected_entrypoints: vec![],
            reachable_methods: vec![],
            reachable_classes: vec![],
            reachable_packages: vec![
                "com.example".to_string(),
                "org.apache.commons".to_string(),
            ],
            error: None,
        };

        assert!(result.is_package_reachable("com.example"));
        assert!(result.is_package_reachable("org.apache.commons"));
        assert!(!result.is_package_reachable("com.other"));
    }

    #[test]
    fn test_is_method_reachable() {
        let result = ReachabilityResult {
            tool: "test".to_string(),
            version: "0.1.0".to_string(),
            classpath: "".to_string(),
            entrypoints: "".to_string(),
            detected_entrypoints: vec![],
            reachable_methods: vec![
                "com.example.Main.main([Ljava/lang/String;)V".to_string(),
                "com.example.Utils.helper()V".to_string(),
            ],
            reachable_classes: vec![],
            reachable_packages: vec![],
            error: None,
        };

        assert!(result.is_method_reachable("com.example.Main.main"));
        assert!(result.is_method_reachable("com.example.Utils.helper"));
        assert!(!result.is_method_reachable("com.example.Utils.other"));
    }
}
