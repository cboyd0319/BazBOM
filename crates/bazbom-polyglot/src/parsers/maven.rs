//! Maven pom.xml parser
//!
//! Parses Maven pom.xml files to extract dependencies and their versions.
//! Supports:
//! - Direct dependencies
//! - Dependency management
//! - Properties resolution
//! - Scope handling (compile, test, runtime, provided)

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Maven dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenDependency {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub scope: String,
    pub optional: bool,
    #[serde(rename = "type")]
    pub dependency_type: String,
}

/// Maven POM structure (simplified)
#[derive(Debug, Deserialize)]
struct Pom {
    #[serde(rename = "groupId", default)]
    group_id: Option<String>,
    #[serde(rename = "artifactId", default)]
    artifact_id: Option<String>,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    properties: Option<HashMap<String, String>>,
    #[serde(default)]
    dependencies: Option<Dependencies>,
    #[serde(rename = "dependencyManagement", default)]
    dependency_management: Option<DependencyManagement>,
    #[serde(default)]
    parent: Option<Parent>,
}

#[derive(Debug, Deserialize)]
struct Dependencies {
    #[serde(rename = "dependency", default)]
    dependency: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct DependencyManagement {
    #[serde(default)]
    dependencies: Option<Dependencies>,
}

#[derive(Debug, Deserialize)]
struct Parent {
    #[serde(rename = "groupId")]
    group_id: String,
    #[serde(rename = "artifactId")]
    artifact_id: String,
    #[serde(default)]
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    #[serde(rename = "groupId")]
    group_id: String,
    #[serde(rename = "artifactId")]
    artifact_id: String,
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    scope: Option<String>,
    #[serde(default)]
    optional: Option<String>,
    #[serde(rename = "type", default)]
    dependency_type: Option<String>,
}

/// Parse a Maven pom.xml file
pub fn parse_pom(pom_path: &Path) -> Result<Vec<MavenDependency>> {
    let content = fs::read_to_string(pom_path)
        .with_context(|| format!("Failed to read pom.xml: {}", pom_path.display()))?;

    parse_pom_content(&content)
}

/// Parse pom.xml content
pub fn parse_pom_content(content: &str) -> Result<Vec<MavenDependency>> {
    // Parse XML
    let pom: Pom = serde_xml_rs::from_str(content)
        .context("Failed to parse pom.xml")?;

    let mut dependencies = Vec::new();
    let properties = pom.properties.unwrap_or_default();

    // Build dependency management map for version resolution
    let mut dep_management: HashMap<String, String> = HashMap::new();
    if let Some(dm) = pom.dependency_management {
        if let Some(deps) = dm.dependencies {
            for dep in deps.dependency {
                if let Some(version) = dep.version {
                    let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                    dep_management.insert(key, resolve_property(&version, &properties));
                }
            }
        }
    }

    // Parse direct dependencies
    if let Some(deps) = pom.dependencies {
        for dep in deps.dependency {
            // Resolve version from dependency management or direct declaration
            let version = if let Some(v) = dep.version {
                resolve_property(&v, &properties)
            } else {
                // Try to find version in dependency management
                let key = format!("{}:{}", dep.group_id, dep.artifact_id);
                dep_management.get(&key).cloned().unwrap_or_else(|| "unknown".to_string())
            };

            // Skip if no version could be resolved
            if version == "unknown" || version.is_empty() {
                continue;
            }

            let scope = dep.scope.unwrap_or_else(|| "compile".to_string());
            let optional = dep.optional.as_deref() == Some("true");
            let dependency_type = dep.dependency_type.unwrap_or_else(|| "jar".to_string());

            dependencies.push(MavenDependency {
                group_id: dep.group_id,
                artifact_id: dep.artifact_id,
                version,
                scope,
                optional,
                dependency_type,
            });
        }
    }

    Ok(dependencies)
}

/// Resolve Maven property placeholders like ${project.version}
fn resolve_property(value: &str, properties: &HashMap<String, String>) -> String {
    let mut result = value.to_string();

    // Handle ${property} syntax
    while let Some(start) = result.find("${") {
        if let Some(end) = result[start..].find('}') {
            let end = start + end;
            let property_name = &result[start + 2..end];

            // Try to resolve the property
            let resolved = if let Some(prop_value) = properties.get(property_name) {
                prop_value.clone()
            } else {
                // Keep unresolved properties as-is
                result[start..=end].to_string()
            };

            result.replace_range(start..=end, &resolved);

            // Avoid infinite loop if property wasn't resolved
            if resolved.contains("${") {
                break;
            }
        } else {
            break;
        }
    }

    result
}

/// Generate package identifier for Maven dependencies
pub fn maven_package_id(dep: &MavenDependency) -> String {
    format!("{}:{}", dep.group_id, dep.artifact_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_pom() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>my-app</artifactId>
    <version>1.0.0</version>

    <dependencies>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>5.3.20</version>
        </dependency>
        <dependency>
            <groupId>junit</groupId>
            <artifactId>junit</artifactId>
            <version>4.13.2</version>
            <scope>test</scope>
        </dependency>
    </dependencies>
</project>"#;

        let deps = parse_pom_content(pom_xml).unwrap();
        assert_eq!(deps.len(), 2);

        let spring = &deps[0];
        assert_eq!(spring.group_id, "org.springframework");
        assert_eq!(spring.artifact_id, "spring-core");
        assert_eq!(spring.version, "5.3.20");
        assert_eq!(spring.scope, "compile");

        let junit = &deps[1];
        assert_eq!(junit.group_id, "junit");
        assert_eq!(junit.artifact_id, "junit");
        assert_eq!(junit.version, "4.13.2");
        assert_eq!(junit.scope, "test");
    }

    #[test]
    fn test_parse_pom_with_properties() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>my-app</artifactId>
    <version>1.0.0</version>

    <properties>
        <spring.version>5.3.20</spring.version>
    </properties>

    <dependencies>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
            <version>${spring.version}</version>
        </dependency>
    </dependencies>
</project>"#;

        let deps = parse_pom_content(pom_xml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].version, "5.3.20");
    }

    #[test]
    fn test_parse_pom_with_dependency_management() {
        let pom_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <groupId>com.example</groupId>
    <artifactId>my-app</artifactId>
    <version>1.0.0</version>

    <dependencyManagement>
        <dependencies>
            <dependency>
                <groupId>org.springframework</groupId>
                <artifactId>spring-core</artifactId>
                <version>5.3.20</version>
            </dependency>
        </dependencies>
    </dependencyManagement>

    <dependencies>
        <dependency>
            <groupId>org.springframework</groupId>
            <artifactId>spring-core</artifactId>
        </dependency>
    </dependencies>
</project>"#;

        let deps = parse_pom_content(pom_xml).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].version, "5.3.20");
    }

    #[test]
    fn test_maven_package_id() {
        let dep = MavenDependency {
            group_id: "org.springframework".to_string(),
            artifact_id: "spring-core".to_string(),
            version: "5.3.20".to_string(),
            scope: "compile".to_string(),
            optional: false,
            dependency_type: "jar".to_string(),
        };

        assert_eq!(maven_package_id(&dep), "org.springframework:spring-core");
    }
}
