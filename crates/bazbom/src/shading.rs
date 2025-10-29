use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Represents a class relocation mapping (e.g., org.foo -> com.shaded.org.foo)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
pub struct ShadingConfiguration {
    pub source: String,  // "maven-shade-plugin" or "gradle-shadow"
    pub relocations: Vec<RelocationMapping>,
    pub finalName: Option<String>,
}

/// Class fingerprint for matching shaded classes to original artifacts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClassFingerprint {
    pub className: String,
    pub methodSignatures: Vec<String>,
    pub fieldSignatures: Vec<String>,
    pub bytecodeHash: String,
}

/// Represents a match between a shaded class and its original artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadingMatch {
    pub shadedClassName: String,
    pub originalClassName: String,
    pub originalArtifact: Option<String>, // GAV coordinates
    pub confidence: f32, // 0.0 to 1.0
}

impl RelocationMapping {
    /// Check if a class name matches this relocation pattern
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
    
    // Look for maven-shade-plugin configuration
    if !content.contains("maven-shade-plugin") {
        return Ok(None);
    }
    
    // Parse XML to find relocations
    // For now, use a simple regex-based approach
    // In production, use a proper XML parser
    let mut relocations = Vec::new();
    
    // Look for <relocation> blocks
    for line in content.lines() {
        if line.contains("<pattern>") && line.contains("</pattern>") {
            if let Some(pattern) = extract_xml_value(line, "pattern") {
                // Look ahead for shadedPattern
                // This is simplified - real implementation would parse the whole relocation block
                relocations.push(RelocationMapping {
                    pattern: pattern.clone(),
                    shadedPattern: format!("shaded.{}", pattern), // Default assumption
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
        source: "maven-shade-plugin".to_string(),
        relocations,
        finalName: None,
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
pub fn extract_nested_jars(jar_path: &Path, output_dir: &Path) -> Result<Vec<String>> {
    // Use jar command or zip utilities to extract nested JARs
    // This would be implemented using std::process::Command
    
    // For now, return empty list as placeholder
    let _ = (jar_path, output_dir);
    Ok(Vec::new())
}

/// Generate a fingerprint for a class file
pub fn fingerprint_class(class_bytes: &[u8]) -> Result<ClassFingerprint> {
    // Use ASM or similar bytecode library to parse class
    // Extract method signatures, field signatures, and compute hash
    
    // Placeholder implementation
    let hash = blake3::hash(class_bytes).to_hex().to_string();
    
    Ok(ClassFingerprint {
        className: "placeholder.Class".to_string(),
        methodSignatures: vec![],
        fieldSignatures: vec![],
        bytecodeHash: hash,
    })
}

/// Match a shaded class to its original artifact using fingerprints
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

// Helper functions

fn extract_xml_value(line: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);
    
    if let Some(start_idx) = line.find(&start_tag) {
        if let Some(end_idx) = line.find(&end_tag) {
            let value_start = start_idx + start_tag.len();
            if value_start < end_idx {
                return Some(line[value_start..end_idx].trim().to_string());
            }
        }
    }
    
    None
}

fn parse_gradle_relocate_line(line: &str) -> Option<(String, String)> {
    // Parse patterns like:
    // relocate 'org.apache', 'myapp.shaded.apache'
    // relocate("org.apache", "myapp.shaded.apache")
    
    let cleaned = line.trim()
        .replace("relocate", "")
        .replace('(', "")
        .replace(')', "")
        .replace('\'', "")
        .replace('"', "");
    
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
    fn test_extract_xml_value() {
        let line = "  <pattern>org.apache</pattern>  ";
        let value = extract_xml_value(line, "pattern");
        assert_eq!(value, Some("org.apache".to_string()));
        
        let no_match = extract_xml_value(line, "other");
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
}
