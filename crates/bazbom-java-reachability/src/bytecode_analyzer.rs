//! Java bytecode analysis for extracting method calls
//!
//! This module analyzes .class files to build a call graph.
//! For a full implementation, this would parse the bytecode constant pool
//! and method code to extract all method invocations.

use crate::call_graph::CallGraph;
use crate::entrypoints::is_entrypoint;
use crate::error::Result;
use crate::models::MethodNode;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Analyze Java .class files in a directory
pub fn analyze_classes(project_root: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // Find all .class files
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip non-class files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("class") {
            continue;
        }

        // Try to analyze this class file
        if let Err(e) = analyze_class_file(path, call_graph) {
            tracing::warn!("Failed to analyze {}: {}", path.display(), e);
        }
    }

    Ok(())
}

/// Analyze a single .class file
fn analyze_class_file(class_path: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // Read the class file
    let _class_bytes = fs::read(class_path)?;

    // In a full implementation, we would:
    // 1. Parse the class file format (magic number, version, constant pool)
    // 2. Extract class name from constant pool
    // 3. Parse methods table
    // 4. For each method, parse Code attribute
    // 5. Extract method invocations from bytecode instructions:
    //    - invokevirtual (0xb6)
    //    - invokespecial (0xb7)
    //    - invokestatic (0xb8)
    //    - invokeinterface (0xb9)
    //    - invokedynamic (0xba)
    // 6. Resolve method references from constant pool
    // 7. Build call graph edges

    // For now, we create a minimal stub that would be filled in with actual bytecode parsing
    // This allows the crate to compile and demonstrates the architecture

    // Extract class name from file path as a placeholder
    let class_name = class_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown");

    // Create a placeholder method (in real implementation, would extract all methods)
    let method_id = format!("{}:placeholder()V", class_name);
    let mut method = MethodNode::new(
        method_id.clone(),
        "placeholder".to_string(),
        class_name.to_string(),
        "()V".to_string(),
    );

    method.is_public = true;
    method.is_entrypoint = is_entrypoint(&method);

    call_graph.add_method(method);

    Ok(())
}

/// Analyze JAR files in a directory
pub fn analyze_jars(project_root: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // Find all .jar files
    for entry in WalkDir::new(project_root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip non-JAR files
        if !path.is_file() || path.extension().and_then(|s| s.to_str()) != Some("jar") {
            continue;
        }

        // Try to analyze this JAR file
        if let Err(e) = analyze_jar_file(path, call_graph) {
            tracing::warn!("Failed to analyze JAR {}: {}", path.display(), e);
        }
    }

    Ok(())
}

/// Analyze a single JAR file
fn analyze_jar_file(jar_path: &Path, call_graph: &mut CallGraph) -> Result<()> {
    // In a full implementation, we would:
    // 1. Open the JAR as a ZIP archive
    // 2. Extract all .class files
    // 3. For each class file, call analyze_class_file()

    // For now, this is a placeholder
    tracing::debug!("Would analyze JAR: {}", jar_path.display());

    // The real implementation would integrate with the existing shading.rs
    // functionality in the main bazbom crate which already has JAR parsing

    let _call_graph = call_graph; // Suppress unused warning
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_classes_empty_dir() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let mut call_graph = CallGraph::new();

        let result = analyze_classes(temp_dir.path(), &mut call_graph);
        assert!(result.is_ok());
        assert_eq!(call_graph.methods.len(), 0);
    }
}
