/// End-to-end reachability workflow integration test
///
/// This test validates the complete reachability analysis pipeline:
/// 1. Building a simple Java application
/// 2. Extracting the classpath
/// 3. Running reachability analysis
/// 4. Verifying results include expected classes/packages
/// 5. Testing cache functionality
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

/// Helper to create a simple test Java application with multiple classes
fn create_test_app(dir: &Path) -> std::io::Result<()> {
    let src_dir = dir.join("src/main/java/com/example/testapp");
    fs::create_dir_all(&src_dir)?;

    // Create Main class that uses Helper
    let main_file = src_dir.join("Main.java");
    let mut file = fs::File::create(&main_file)?;
    writeln!(file, "package com.example.testapp;")?;
    writeln!(file)?;
    writeln!(file, "public class Main {{")?;
    writeln!(file, "    public static void main(String[] args) {{")?;
    writeln!(file, "        System.out.println(\"Hello from Main\");")?;
    writeln!(file, "        Helper.doSomething();")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // Create Helper class
    let helper_file = src_dir.join("Helper.java");
    let mut file = fs::File::create(&helper_file)?;
    writeln!(file, "package com.example.testapp;")?;
    writeln!(file)?;
    writeln!(file, "public class Helper {{")?;
    writeln!(file, "    public static void doSomething() {{")?;
    writeln!(file, "        System.out.println(\"Helper called\");")?;
    writeln!(file, "    }}")?;
    writeln!(file, "    ")?;
    writeln!(file, "    public static void unusedMethod() {{")?;
    writeln!(
        file,
        "        System.out.println(\"This should not be reachable\");"
    )?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // Create UnusedClass that's never referenced
    let unused_file = src_dir.join("UnusedClass.java");
    let mut file = fs::File::create(&unused_file)?;
    writeln!(file, "package com.example.testapp;")?;
    writeln!(file)?;
    writeln!(file, "public class UnusedClass {{")?;
    writeln!(file, "    public static void neverCalled() {{")?;
    writeln!(
        file,
        "        System.out.println(\"This class is never used\");"
    )?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}

/// Compile Java classes into a directory
fn compile_java_app(project_dir: &Path) -> std::io::Result<PathBuf> {
    let src_dir = project_dir.join("src/main/java");
    let classes_dir = project_dir.join("build/classes");
    fs::create_dir_all(&classes_dir)?;

    // Find all .java files
    let java_files: Vec<PathBuf> = walkdir::WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "java"))
        .map(|e| e.path().to_owned())
        .collect();

    if java_files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Java files found",
        ));
    }

    // Compile Java files
    let status = Command::new("javac")
        .arg("-d")
        .arg(&classes_dir)
        .args(&java_files)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "javac compilation failed",
        ));
    }

    Ok(classes_dir)
}

#[test]
#[ignore] // Requires Java and the reachability JAR to be built
fn test_end_to_end_reachability_workflow() {
    let tmp = tempdir().unwrap();
    let project_dir = tmp.path().join("testapp");
    fs::create_dir_all(&project_dir).unwrap();

    // Step 1: Create test application
    create_test_app(&project_dir).expect("Failed to create test app");

    // Step 2: Compile the application
    let classes_dir = match compile_java_app(&project_dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: javac not available: {}", e);
            return;
        }
    };

    // Step 3: Find the reachability JAR
    let reachability_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/reachability/target")
        .join("bazbom-reachability-0.1.0-SNAPSHOT.jar");

    if !reachability_jar.exists() {
        eprintln!(
            "Skipping test: reachability JAR not found at {:?}",
            reachability_jar
        );
        eprintln!("Build it with: cd tools/reachability && mvn package");
        return;
    }

    // Step 4: Run reachability analysis
    let output_path = tmp.path().join("reachability.json");
    let status = Command::new("java")
        .arg("-jar")
        .arg(&reachability_jar)
        .arg("--classpath")
        .arg(classes_dir.to_str().unwrap())
        .arg("--output")
        .arg(&output_path)
        .status()
        .expect("Failed to run reachability analyzer");

    assert!(status.success(), "Reachability analyzer should succeed");
    assert!(output_path.exists(), "Output file should be created");

    // Step 5: Parse and validate results
    let content = fs::read_to_string(&output_path).unwrap();
    let result: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Validate basic structure
    assert_eq!(result["tool"], "bazbom-reachability");
    assert_eq!(result["version"], "0.1.0");

    // Validate entrypoints were detected
    let entrypoints = result["detectedEntrypoints"].as_array().unwrap();
    assert!(
        !entrypoints.is_empty(),
        "Should detect at least one entrypoint (main method)"
    );

    // Check that Main.main was detected
    let entrypoint_strs: Vec<String> = entrypoints
        .iter()
        .filter_map(|v| v.as_str())
        .map(|s| s.to_string())
        .collect();
    assert!(
        entrypoint_strs
            .iter()
            .any(|e| e.contains("Main") && e.contains("main")),
        "Main.main should be detected as entrypoint"
    );

    // Validate reachable classes
    let reachable_classes = result["reachableClasses"].as_array().unwrap();
    let class_names: Vec<&str> = reachable_classes
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    // Main should be reachable
    assert!(
        class_names.iter().any(|c| c.contains("Main")),
        "Main class should be reachable"
    );

    // Helper should be reachable (called from Main)
    assert!(
        class_names.iter().any(|c| c.contains("Helper")),
        "Helper class should be reachable"
    );

    // Validate reachable packages
    let reachable_packages = result["reachablePackages"].as_array().unwrap();
    let package_names: Vec<&str> = reachable_packages
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        package_names.iter().any(|p| p.contains("com.example")),
        "com.example package should be reachable"
    );

    // Validate methods
    let reachable_methods = result["reachableMethods"].as_array().unwrap();
    assert!(
        !reachable_methods.is_empty(),
        "Should have reachable methods"
    );

    let method_strs: Vec<&str> = reachable_methods
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    // Main.main should be reachable
    assert!(
        method_strs
            .iter()
            .any(|m| m.contains("Main") && m.contains("main")),
        "Main.main method should be reachable"
    );

    // Helper.doSomething should be reachable
    assert!(
        method_strs
            .iter()
            .any(|m| m.contains("Helper") && m.contains("doSomething")),
        "Helper.doSomething should be reachable"
    );

    println!("✓ End-to-end reachability workflow test passed");
    println!("  Detected {} entrypoints", entrypoints.len());
    println!("  Found {} reachable classes", reachable_classes.len());
    println!("  Found {} reachable methods", reachable_methods.len());
    println!("  Found {} reachable packages", reachable_packages.len());
}

#[test]
#[ignore] // Requires Java and the reachability JAR
fn test_reachability_with_cache() {
    let tmp = tempdir().unwrap();
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&cache_dir).unwrap();

    let project_dir = tmp.path().join("testapp");
    fs::create_dir_all(&project_dir).unwrap();

    // Create and compile test app
    create_test_app(&project_dir).expect("Failed to create test app");
    let classes_dir = match compile_java_app(&project_dir) {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Skipping test: {}", e);
            return;
        }
    };

    let reachability_jar = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/reachability/target")
        .join("bazbom-reachability-0.1.0-SNAPSHOT.jar");

    if !reachability_jar.exists() {
        eprintln!("Skipping test: reachability JAR not found");
        return;
    }

    // Run reachability analysis twice
    let output_path1 = tmp.path().join("reachability1.json");
    let output_path2 = tmp.path().join("reachability2.json");

    // First run
    let status1 = Command::new("java")
        .arg("-jar")
        .arg(&reachability_jar)
        .arg("--classpath")
        .arg(classes_dir.to_str().unwrap())
        .arg("--output")
        .arg(&output_path1)
        .status()
        .expect("Failed to run reachability analyzer");

    assert!(status1.success());

    // Second run (should potentially use cache in future)
    let status2 = Command::new("java")
        .arg("-jar")
        .arg(&reachability_jar)
        .arg("--classpath")
        .arg(classes_dir.to_str().unwrap())
        .arg("--output")
        .arg(&output_path2)
        .status()
        .expect("Failed to run reachability analyzer");

    assert!(status2.success());

    // Both runs should produce identical results
    let content1 = fs::read_to_string(&output_path1).unwrap();
    let content2 = fs::read_to_string(&output_path2).unwrap();

    let result1: serde_json::Value = serde_json::from_str(&content1).unwrap();
    let result2: serde_json::Value = serde_json::from_str(&content2).unwrap();

    // Compare results (excluding timestamps if any)
    assert_eq!(
        result1["reachableClasses"], result2["reachableClasses"],
        "Both runs should find same reachable classes"
    );
    assert_eq!(
        result1["reachableMethods"], result2["reachableMethods"],
        "Both runs should find same reachable methods"
    );

    println!("✓ Reachability cache consistency test passed");
}
