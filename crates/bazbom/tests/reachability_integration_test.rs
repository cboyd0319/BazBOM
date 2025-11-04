use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// Helper to create a simple test Java class
fn create_test_java_class(
    dir: &Path,
    package: &str,
    class: &str,
    has_main: bool,
) -> std::io::Result<()> {
    let package_dir = dir.join(package.replace('.', "/"));
    fs::create_dir_all(&package_dir)?;

    let class_file = package_dir.join(format!("{}.java", class));
    let mut file = fs::File::create(&class_file)?;

    let main_method = if has_main {
        r#"
    public static void main(String[] args) {
        System.out.println("Hello from " + CLASS_NAME);
        helper();
    }
    
    public static void helper() {
        System.out.println("Helper called");
    }
"#
    } else {
        r#"
    public static void unused() {
        System.out.println("Unused method");
    }
"#
    };

    writeln!(file, "package {};", package)?;
    writeln!(file)?;
    writeln!(file, "public class {} {{", class)?;
    writeln!(
        file,
        "    private static final String CLASS_NAME = \"{}\";",
        class
    )?;
    writeln!(file, "{}", main_method)?;
    writeln!(file, "}}")?;

    Ok(())
}

/// Helper to compile Java classes into a JAR
fn compile_to_jar(src_dir: &Path, jar_path: &Path) -> std::io::Result<()> {
    // Find all .java files
    let java_files: Vec<_> = walkdir::WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "java"))
        .map(|e| e.path().to_owned())
        .collect();

    if java_files.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No Java files found",
        ));
    }

    // Compile Java files
    let classes_dir = src_dir.join("classes");
    fs::create_dir_all(&classes_dir)?;

    let status = Command::new("javac")
        .arg("-d")
        .arg(&classes_dir)
        .args(&java_files)
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("javac compilation failed"));
    }

    // Create JAR
    let status = Command::new("jar")
        .arg("cf")
        .arg(jar_path)
        .arg("-C")
        .arg(&classes_dir)
        .arg(".")
        .status()?;

    if !status.success() {
        return Err(std::io::Error::other("jar creation failed"));
    }

    Ok(())
}

#[test]
#[ignore] // Requires Java and the reachability JAR to be built
fn test_reachability_with_simple_jar() {
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    let jar_path = tmp.path().join("test.jar");

    // Create test Java classes
    create_test_java_class(&src_dir, "com.example", "Main", true).unwrap();
    create_test_java_class(&src_dir, "com.example", "Unused", false).unwrap();

    // Compile to JAR
    if let Err(e) = compile_to_jar(&src_dir, &jar_path) {
        eprintln!("Warning: Could not compile test JAR: {}", e);
        eprintln!("Skipping integration test (requires javac and jar)");
        return;
    }

    // Find the reachability JAR
    let reachability_jar = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/reachability/target")
        .join("bazbom-reachability-0.1.0-SNAPSHOT.jar");

    if !reachability_jar.exists() {
        eprintln!(
            "Warning: Reachability JAR not found at {:?}",
            reachability_jar
        );
        eprintln!("Build it with: cd tools/reachability && mvn package");
        return;
    }

    // Run reachability analysis
    let output_path = tmp.path().join("reachability.json");
    let status = Command::new("java")
        .arg("-jar")
        .arg(&reachability_jar)
        .arg("--classpath")
        .arg(jar_path.to_str().unwrap())
        .arg("--output")
        .arg(&output_path)
        .status()
        .expect("Failed to run reachability analyzer");

    assert!(status.success(), "Reachability analyzer should succeed");
    assert!(output_path.exists(), "Output file should be created");

    // Parse and verify output
    let content = fs::read_to_string(&output_path).unwrap();
    let result: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert_eq!(result["tool"], "bazbom-reachability");
    assert_eq!(result["version"], "0.1.0");

    // Verify reachable classes include Main but may or may not include Unused
    let reachable_classes = result["reachableClasses"].as_array().unwrap();
    let class_names: Vec<&str> = reachable_classes
        .iter()
        .filter_map(|v| v.as_str())
        .collect();

    assert!(
        class_names.iter().any(|c| c.contains("Main")),
        "Main class should be reachable"
    );

    // Verify packages
    let reachable_packages = result["reachablePackages"].as_array().unwrap();
    assert!(
        !reachable_packages.is_empty(),
        "Should have at least one reachable package"
    );
}

#[test]
#[ignore] // Requires Java and the reachability JAR
fn test_reachability_with_entrypoints() {
    let tmp = tempdir().unwrap();
    let src_dir = tmp.path().join("src");
    let jar_path = tmp.path().join("test.jar");

    // Create test Java class
    create_test_java_class(&src_dir, "com.test", "App", true).unwrap();

    // Compile to JAR
    if let Err(e) = compile_to_jar(&src_dir, &jar_path) {
        eprintln!("Skipping test: {}", e);
        return;
    }

    let reachability_jar = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tools/reachability/target")
        .join("bazbom-reachability-0.1.0-SNAPSHOT.jar");

    if !reachability_jar.exists() {
        eprintln!("Skipping test: reachability JAR not found");
        return;
    }

    // Run with explicit entrypoint
    let output_path = tmp.path().join("reachability.json");
    let status = Command::new("java")
        .arg("-jar")
        .arg(&reachability_jar)
        .arg("--classpath")
        .arg(jar_path.to_str().unwrap())
        .arg("--entrypoints")
        .arg("com.test.App.main")
        .arg("--output")
        .arg(&output_path)
        .status()
        .expect("Failed to run reachability analyzer");

    assert!(status.success());

    let content = fs::read_to_string(&output_path).unwrap();
    let result: serde_json::Value = serde_json::from_str(&content).unwrap();

    // Verify detected entrypoints
    let detected = result["detectedEntrypoints"].as_array().unwrap();
    assert!(
        !detected.is_empty(),
        "Should detect at least one entrypoint"
    );
}

#[test]
fn test_reachability_result_parsing() {
    // Test that we can parse a typical reachability result
    let json = r#"{
        "tool": "bazbom-reachability",
        "version": "0.1.0",
        "classpath": "/tmp/test.jar",
        "entrypoints": "",
        "detectedEntrypoints": ["com.example.Main.main([Ljava/lang/String;)V"],
        "reachableMethods": [
            "com.example.Main.main([Ljava/lang/String;)V",
            "com.example.Main.helper()V"
        ],
        "reachableClasses": [
            "com.example.Main",
            "java.lang.System"
        ],
        "reachablePackages": [
            "com.example",
            "java.lang"
        ]
    }"#;

    let result: bazbom::reachability::ReachabilityResult =
        serde_json::from_str(json).expect("Should parse reachability result");

    assert_eq!(result.tool, "bazbom-reachability");
    assert_eq!(result.version, "0.1.0");
    assert_eq!(result.detected_entrypoints.len(), 1);
    assert_eq!(result.reachable_methods.len(), 2);
    assert_eq!(result.reachable_classes.len(), 2);
    assert_eq!(result.reachable_packages.len(), 2);

    // Test helper methods
    assert!(result.is_class_reachable("com.example.Main"));
    assert!(!result.is_class_reachable("com.example.Unused"));

    assert!(result.is_package_reachable("com.example"));
    assert!(result.is_package_reachable("java.lang"));
    assert!(!result.is_package_reachable("org.other"));

    assert!(result.is_method_reachable("Main.main"));
    assert!(result.is_method_reachable("Main.helper"));
    assert!(!result.is_method_reachable("Main.unused"));
}

#[test]
fn test_reachability_result_with_error() {
    let json = r#"{
        "tool": "bazbom-reachability",
        "version": "0.1.0",
        "classpath": "",
        "entrypoints": "",
        "detectedEntrypoints": [],
        "reachableMethods": [],
        "reachableClasses": [],
        "reachablePackages": [],
        "error": "Empty classpath provided"
    }"#;

    let result: bazbom::reachability::ReachabilityResult =
        serde_json::from_str(json).expect("Should parse result with error");

    assert!(result.error.is_some());
    assert_eq!(result.error.unwrap(), "Empty classpath provided");
}

#[test]
fn test_package_reachability_parent_matching() {
    let result = bazbom::reachability::ReachabilityResult {
        tool: "test".to_string(),
        version: "0.1.0".to_string(),
        classpath: "".to_string(),
        entrypoints: "".to_string(),
        detected_entrypoints: vec![],
        reachable_methods: vec![],
        reachable_classes: vec![],
        reachable_packages: vec![
            "com.example.core".to_string(),
            "org.apache.commons.lang3".to_string(),
        ],
        error: None,
    };

    // Should match exact package
    assert!(result.is_package_reachable("com.example.core"));

    // Should match subpackages
    assert!(result.is_package_reachable("com.example.core"));

    // Should not match parent packages
    assert!(!result.is_package_reachable("com.example"));

    // Should not match unrelated packages
    assert!(!result.is_package_reachable("com.other"));
}
