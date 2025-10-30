use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Represents a class relocation mapping (e.g., org.foo -> com.shaded.org.foo)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct RelocationMapping {
    pub pattern: String,
    pub shadedPattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excludes: Option<Vec<String>>,
}

/// Represents configuration for a shaded JAR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ShadingConfiguration {
    pub source: String,  // "maven-shade-plugin" or "gradle-shadow"
    pub relocations: Vec<RelocationMapping>,
    pub finalName: Option<String>,
}

/// Class fingerprint for matching shaded classes to original artifacts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct ClassFingerprint {
    pub className: String,
    pub methodSignatures: Vec<String>,
    pub fieldSignatures: Vec<String>,
    pub bytecodeHash: String,
}

/// Represents a match between a shaded class and its original artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ShadingMatch {
    pub shadedClassName: String,
    pub originalClassName: String,
    pub originalArtifact: Option<String>, // GAV coordinates
    pub confidence: f32, // 0.0 to 1.0
}

impl RelocationMapping {
    /// Check if a class name matches this relocation pattern
    /// 
    /// Future use: Runtime JAR analysis to match classes against relocation patterns
    /// for accurate vulnerability attribution in shaded dependencies.
    #[allow(dead_code)]
    pub fn matches(&self, class_name: &str) -> bool {
        // Check if class is in the pattern namespace
        if !class_name.starts_with(&self.pattern) {
            return false;
        }
        
        // Check includes if specified
        if let Some(includes) = &self.includes {
            if !includes.iter().any(|inc| class_name.starts_with(inc)) {
                return false;
            }
        }
        
        // Check excludes if specified
        if let Some(excludes) = &self.excludes {
            if excludes.iter().any(|exc| class_name.starts_with(exc)) {
                return false;
            }
        }
        
        true
    }
    
    /// Apply this relocation to a class name, returning the original name
    /// 
    /// Future use: Map shaded class names back to original artifacts for
    /// accurate vulnerability reporting and dependency attribution.
    #[allow(dead_code)]
    pub fn reverse_relocate(&self, shaded_class_name: &str) -> Option<String> {
        if !shaded_class_name.starts_with(&self.shadedPattern) {
            return None;
        }
        
        let suffix = shaded_class_name.strip_prefix(&self.shadedPattern)?;
        Some(format!("{}{}", self.pattern, suffix))
    }
}

/// Parse Maven Shade Plugin configuration from pom.xml
pub fn parse_maven_shade_config(pom_path: &Path) -> Result<Option<ShadingConfiguration>> {
    if !pom_path.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(pom_path)
        .context("failed to read pom.xml")?;
    
    // Quick check if shade plugin is present
    if !content.contains("maven-shade-plugin") {
        return Ok(None);
    }
    
    let mut reader = Reader::from_str(&content);
    reader.trim_text(true);
    
    let mut relocations = Vec::new();
    let mut in_shade_plugin = false;
    let mut in_configuration = false;
    let mut in_relocations = false;
    let mut in_relocation = false;
    
    let mut current_pattern = String::new();
    let mut current_shaded_pattern = String::new();
    let mut current_includes: Vec<String> = Vec::new();
    let mut current_excludes: Vec<String> = Vec::new();
    let mut in_pattern = false;
    let mut in_shaded_pattern = false;
    let mut in_includes = false;
    let mut in_excludes = false;
    let mut in_include = false;
    let mut in_exclude = false;
    let mut final_name: Option<String> = None;
    let mut in_final_name = false;
    
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"artifactId" => {
                        // Check if this is maven-shade-plugin
                        if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
                            if text.unescape().ok().as_deref() == Some("maven-shade-plugin") {
                                in_shade_plugin = true;
                            }
                        }
                    }
                    b"configuration" if in_shade_plugin => in_configuration = true,
                    b"relocations" if in_configuration => in_relocations = true,
                    b"relocation" if in_relocations => {
                        in_relocation = true;
                        // Reset current relocation data
                        current_pattern.clear();
                        current_shaded_pattern.clear();
                        current_includes.clear();
                        current_excludes.clear();
                    }
                    b"pattern" if in_relocation => in_pattern = true,
                    b"shadedPattern" if in_relocation => in_shaded_pattern = true,
                    b"includes" if in_relocation => in_includes = true,
                    b"excludes" if in_relocation => in_excludes = true,
                    b"include" if in_includes => in_include = true,
                    b"exclude" if in_excludes => in_exclude = true,
                    b"finalName" if in_configuration => in_final_name = true,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().to_string();
                if in_pattern {
                    current_pattern = text;
                } else if in_shaded_pattern {
                    current_shaded_pattern = text;
                } else if in_include {
                    current_includes.push(text);
                } else if in_exclude {
                    current_excludes.push(text);
                } else if in_final_name {
                    final_name = Some(text);
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"plugin" => in_shade_plugin = false,
                    b"configuration" => in_configuration = false,
                    b"relocations" => in_relocations = false,
                    b"relocation" => {
                        // Save completed relocation
                        if !current_pattern.is_empty() && !current_shaded_pattern.is_empty() {
                            relocations.push(RelocationMapping {
                                pattern: current_pattern.clone(),
                                shadedPattern: current_shaded_pattern.clone(),
                                includes: if current_includes.is_empty() { None } else { Some(current_includes.clone()) },
                                excludes: if current_excludes.is_empty() { None } else { Some(current_excludes.clone()) },
                            });
                        }
                        in_relocation = false;
                    }
                    b"pattern" => in_pattern = false,
                    b"shadedPattern" => in_shaded_pattern = false,
                    b"includes" => in_includes = false,
                    b"excludes" => in_excludes = false,
                    b"include" => in_include = false,
                    b"exclude" => in_exclude = false,
                    b"finalName" => in_final_name = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!("XML parse error at position {}: {}", reader.buffer_position(), e));
            }
            _ => {}
        }
        buf.clear();
    }
    
    if relocations.is_empty() {
        return Ok(None);
    }
    
    Ok(Some(ShadingConfiguration {
        source: "maven-shade-plugin".to_string(),
        relocations,
        finalName: final_name,
    }))
}

/// Parse Gradle Shadow Plugin configuration from build.gradle or build.gradle.kts
pub fn parse_gradle_shadow_config(build_file: &Path) -> Result<Option<ShadingConfiguration>> {
    if !build_file.exists() {
        return Ok(None);
    }
    
    let content = fs::read_to_string(build_file)
        .context("failed to read build file")?;
    
    // Look for shadow plugin and relocate statements
    if !content.contains("shadow") && !content.contains("com.github.johnrengelman.shadow") {
        return Ok(None);
    }
    
    let mut relocations = Vec::new();
    
    // Look for relocate() calls
    // Example: relocate 'org.apache', 'myapp.shaded.apache'
    for line in content.lines() {
        if line.contains("relocate") {
            // Try to extract pattern and shaded pattern
            // This is simplified - real implementation would parse Groovy/Kotlin properly
            if let Some((pattern, shaded)) = parse_gradle_relocate_line(line) {
                relocations.push(RelocationMapping {
                    pattern,
                    shadedPattern: shaded,
                    includes: None,
                    excludes: None,
                });
            }
        }
    }
    
    if relocations.is_empty() {
        return Ok(None);
    }
    
    Ok(Some(ShadingConfiguration {
        source: "gradle-shadow-plugin".to_string(),
        relocations,
        finalName: None,
    }))
}

/// Extract nested JARs from a fat JAR
/// 
/// Future use: Deep analysis of fat JARs (uber JARs) to extract and analyze
/// embedded dependencies, useful for Spring Boot applications and other
/// packaging formats that bundle dependencies within the main JAR.
#[allow(dead_code)]
pub fn extract_nested_jars(jar_path: &Path, output_dir: &Path) -> Result<Vec<String>> {
    use zip::ZipArchive;
    
    if !jar_path.exists() {
        return Err(anyhow::anyhow!("JAR file not found: {:?}", jar_path));
    }
    
    // Ensure output directory exists
    fs::create_dir_all(output_dir)
        .context("failed to create output directory")?;
    
    let file = fs::File::open(jar_path)
        .context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file)
        .context("failed to read ZIP archive")?;
    
    let mut nested_jars = Vec::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();
        
        // Look for nested JAR files
        if name.ends_with(".jar") {
            let output_path = output_dir.join(&name);
            
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)
                    .context("failed to create nested directory")?;
            }
            
            // Extract the nested JAR
            let mut output_file = fs::File::create(&output_path)
                .context("failed to create output file")?;
            std::io::copy(&mut file, &mut output_file)
                .context("failed to copy nested JAR")?;
            
            nested_jars.push(name);
        }
    }
    
    Ok(nested_jars)
}

/// Generate a fingerprint for a class file
/// 
/// Future use: Create unique fingerprints of class files for matching shaded
/// classes to their original artifacts when relocation patterns are ambiguous.
/// Enables high-confidence attribution even for complex shading scenarios.
#[allow(dead_code)]
pub fn fingerprint_class(class_bytes: &[u8]) -> Result<ClassFingerprint> {
    // Compute bytecode hash for matching
    let hash = blake3::hash(class_bytes).to_hex().to_string();
    
    // Extract class name from bytecode
    // The class file format has the class name at a specific offset
    // For a proper implementation, we would parse the constant pool
    // Here we provide a basic implementation that extracts the class name
    let class_name = extract_class_name_from_bytecode(class_bytes)
        .unwrap_or_else(|| "Unknown".to_string());
    
    // Note: For production use, method and field signatures should be extracted
    // using ASM or similar bytecode parsing library. This would ideally be done
    // in the Java-based bazbom-reachability tool and the results provided to Rust.
    // For now, we rely on bytecode hash for matching.
    
    Ok(ClassFingerprint {
        className: class_name,
        methodSignatures: vec![],
        fieldSignatures: vec![],
        bytecodeHash: hash,
    })
}

/// Extract class name from bytecode (basic implementation)
/// 
/// Future use: Helper function for fingerprint_class to extract class names
/// directly from bytecode for more accurate matching. Currently relies on
/// bytecode hash; future implementation would parse constant pool.
#[allow(dead_code)]
fn extract_class_name_from_bytecode(class_bytes: &[u8]) -> Option<String> {
    // Basic validation - check for Java class file magic number
    if class_bytes.len() < 10 || &class_bytes[0..4] != b"\xCA\xFE\xBA\xBE" {
        return None;
    }
    
    // For a complete implementation, we would need to:
    // 1. Parse the constant pool
    // 2. Find the this_class index
    // 3. Resolve the class name from the constant pool
    // This is complex and better done with a proper bytecode library
    
    // For now, return None and rely on bytecode hash matching
    None
}

/// Scan a JAR file and create fingerprints for all classes
/// 
/// Future use: Build a database of class fingerprints from known artifacts
/// for matching against shaded JARs. Enables identification of dependencies
/// in fat JARs even without relocation configuration.
#[allow(dead_code)]
pub fn fingerprint_jar(jar_path: &Path) -> Result<HashMap<String, ClassFingerprint>> {
    use zip::ZipArchive;
    
    let file = fs::File::open(jar_path)
        .context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file)
        .context("failed to read ZIP archive")?;
    
    let mut fingerprints = HashMap::new();
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();
        
        // Only process .class files
        if name.ends_with(".class") && !name.contains("module-info") {
            let mut class_bytes = Vec::new();
            file.read_to_end(&mut class_bytes)
                .context("failed to read class file")?;
            
            if let Ok(fingerprint) = fingerprint_class(&class_bytes) {
                // Use the file path as the key (without .class extension)
                let class_key = name.trim_end_matches(".class").replace('/', ".");
                fingerprints.insert(class_key, fingerprint);
            }
        }
    }
    
    Ok(fingerprints)
}

/// Match a shaded class to its original artifact using fingerprints
/// 
/// Future use: Compare fingerprints of shaded classes against a database
/// of known artifacts to identify dependencies and enable accurate
/// vulnerability attribution with confidence scoring.
#[allow(dead_code)]
pub fn match_shaded_class(
    shaded_class: &ClassFingerprint,
    known_fingerprints: &HashMap<String, ClassFingerprint>,
) -> Option<ShadingMatch> {
    // Compare fingerprints to find best match
    // This is a simplified implementation
    
    for (artifact_gav, original_fingerprint) in known_fingerprints {
        if shaded_class.bytecodeHash == original_fingerprint.bytecodeHash {
            return Some(ShadingMatch {
                shadedClassName: shaded_class.className.clone(),
                originalClassName: original_fingerprint.className.clone(),
                originalArtifact: Some(artifact_gav.clone()),
                confidence: 1.0,
            });
        }
    }
    
    None
}

/// Detect shading in a JAR by comparing its classes against a relocation map
/// 
/// Future use: Runtime analysis of JAR files to detect shaded classes and
/// map them to original artifacts. Complements build-time detection for
/// cases where build configuration is unavailable or incomplete.
#[allow(dead_code)]
pub fn detect_shading_in_jar(
    jar_path: &Path,
    relocation: &RelocationMapping,
) -> Result<Vec<ShadingMatch>> {
    use zip::ZipArchive;
    
    let file = fs::File::open(jar_path)
        .context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file)
        .context("failed to read ZIP archive")?;
    
    let mut matches = Vec::new();
    
    for i in 0..archive.len() {
        let file = archive.by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();
        
        // Only process .class files
        if name.ends_with(".class") {
            // Convert file path to class name
            let class_name = name
                .trim_end_matches(".class")
                .replace('/', ".");
            
            // Check if this class matches the relocation pattern
            if let Some(original_name) = relocation.reverse_relocate(&class_name) {
                matches.push(ShadingMatch {
                    shadedClassName: class_name,
                    originalClassName: original_name.clone(),
                    originalArtifact: None, // Would need fingerprint matching to determine
                    confidence: 0.8, // High confidence based on relocation pattern match
                });
            }
        }
    }
    
    Ok(matches)
}

// Helper functions

fn parse_gradle_relocate_line(line: &str) -> Option<(String, String)> {
    // Parse patterns like:
    // relocate 'org.apache', 'myapp.shaded.apache'
    // relocate("org.apache", "myapp.shaded.apache")
    
    let cleaned = line.trim()
        .replace("relocate", "")
        .replace(['(', ')', '\'', '"'], "");
    
    let parts: Vec<&str> = cleaned.split(',').map(|s| s.trim()).collect();
    
    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relocation_matches() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: None,
        };
        
        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("com.example.Foo"));
    }

    #[test]
    fn test_relocation_with_includes() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: Some(vec!["org.apache.commons".to_string()]),
            excludes: None,
        };
        
        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("org.apache.log4j.Logger"));
    }

    #[test]
    fn test_relocation_with_excludes() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: Some(vec!["org.apache.log4j".to_string()]),
        };
        
        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("org.apache.log4j.Logger"));
    }

    #[test]
    fn test_reverse_relocate() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: None,
        };
        
        let original = relocation.reverse_relocate("shaded.org.apache.commons.Lang");
        assert_eq!(original, Some("org.apache.commons.Lang".to_string()));
        
        let no_match = relocation.reverse_relocate("com.example.Foo");
        assert_eq!(no_match, None);
    }

    #[test]
    fn test_parse_gradle_relocate_line() {
        let line1 = "  relocate 'org.apache', 'myapp.shaded.apache'  ";
        let result1 = parse_gradle_relocate_line(line1);
        assert_eq!(result1, Some(("org.apache".to_string(), "myapp.shaded.apache".to_string())));
        
        let line2 = r#"  relocate("org.apache", "myapp.shaded.apache")  "#;
        let result2 = parse_gradle_relocate_line(line2);
        assert_eq!(result2, Some(("org.apache".to_string(), "myapp.shaded.apache".to_string())));
    }

    #[test]
    fn test_fingerprint_class() {
        let dummy_bytes = b"dummy class bytes";
        let fingerprint = fingerprint_class(dummy_bytes).unwrap();
        
        assert_eq!(fingerprint.bytecodeHash.len(), 64); // Blake3 hash length
    }

    #[test]
    fn test_match_shaded_class() {
        let mut known = HashMap::new();
        let original = ClassFingerprint {
            className: "org.apache.commons.Lang".to_string(),
            methodSignatures: vec![],
            fieldSignatures: vec![],
            bytecodeHash: "abc123".to_string(),
        };
        known.insert("org.apache.commons:commons-lang3:3.12.0".to_string(), original.clone());
        
        let shaded = ClassFingerprint {
            className: "shaded.org.apache.commons.Lang".to_string(),
            methodSignatures: vec![],
            fieldSignatures: vec![],
            bytecodeHash: "abc123".to_string(),
        };
        
        let match_result = match_shaded_class(&shaded, &known);
        assert!(match_result.is_some());
        
        let matched = match_result.unwrap();
        assert_eq!(matched.confidence, 1.0);
        assert_eq!(matched.originalClassName, "org.apache.commons.Lang");
    }

    #[test]
    fn test_parse_maven_shade_config_complete() {
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>shaded-app</artifactId>
    <version>1.0.0</version>
    
    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <configuration>
                    <finalName>shaded-app</finalName>
                    <relocations>
                        <relocation>
                            <pattern>org.apache.commons</pattern>
                            <shadedPattern>com.example.shaded.commons</shadedPattern>
                        </relocation>
                        <relocation>
                            <pattern>com.google.guava</pattern>
                            <shadedPattern>com.example.shaded.guava</shadedPattern>
                            <includes>
                                <include>com.google.guava.collect</include>
                            </includes>
                            <excludes>
                                <exclude>com.google.guava.base</exclude>
                            </excludes>
                        </relocation>
                    </relocations>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(pom_content.as_bytes()).unwrap();
        
        let config = parse_maven_shade_config(temp_file.path()).unwrap();
        assert!(config.is_some());
        
        let config = config.unwrap();
        assert_eq!(config.source, "maven-shade-plugin");
        assert_eq!(config.finalName, Some("shaded-app".to_string()));
        assert_eq!(config.relocations.len(), 2);
        
        let first = &config.relocations[0];
        assert_eq!(first.pattern, "org.apache.commons");
        assert_eq!(first.shadedPattern, "com.example.shaded.commons");
        assert!(first.includes.is_none());
        assert!(first.excludes.is_none());
        
        let second = &config.relocations[1];
        assert_eq!(second.pattern, "com.google.guava");
        assert_eq!(second.shadedPattern, "com.example.shaded.guava");
        assert!(second.includes.is_some());
        assert_eq!(second.includes.as_ref().unwrap().len(), 1);
        assert!(second.excludes.is_some());
        assert_eq!(second.excludes.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_maven_shade_config_no_plugin() {
        use tempfile::NamedTempFile;
        use std::io::Write;
        
        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>normal-app</artifactId>
    <version>1.0.0</version>
</project>
"#;
        
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(pom_content.as_bytes()).unwrap();
        
        let config = parse_maven_shade_config(temp_file.path()).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_extract_class_name_from_bytecode() {
        // Test with invalid bytecode
        let invalid = b"not a class file";
        let result = super::extract_class_name_from_bytecode(invalid);
        assert!(result.is_none());
        
        // Test with valid magic number but short content
        let short_valid = b"\xCA\xFE\xBA\xBE\x00\x00\x00\x34";
        let result = super::extract_class_name_from_bytecode(short_valid);
        // Should return None as we don't fully parse the constant pool
        assert!(result.is_none());
    }
}
