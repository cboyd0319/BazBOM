use std::fs;
use tempfile::TempDir;

#[test]
fn test_shading_detection_maven() {
    // Create a temporary directory with a Maven project
    let temp_dir = TempDir::new().unwrap();
    let pom_path = temp_dir.path().join("pom.xml");
    
    // Create a pom.xml with maven-shade-plugin
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-app</artifactId>
    <version>1.0.0</version>
    
    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <configuration>
                    <relocations>
                        <relocation>
                            <pattern>org.apache.commons</pattern>
                            <shadedPattern>com.example.shaded.commons</shadedPattern>
                        </relocation>
                    </relocations>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>
"#;
    
    fs::write(&pom_path, pom_content).unwrap();
    
    // Test that we can detect the shading configuration
    let shading_config = bazbom::shading::parse_maven_shade_config(&pom_path).unwrap();
    assert!(shading_config.is_some());
    
    let config = shading_config.unwrap();
    assert_eq!(config.source, "maven-shade-plugin");
    assert_eq!(config.relocations.len(), 1);
    assert_eq!(config.relocations[0].pattern, "org.apache.commons");
    assert_eq!(config.relocations[0].shadedPattern, "com.example.shaded.commons");
}

#[test]
fn test_shading_detection_gradle() {
    // Create a temporary directory with a Gradle project
    let temp_dir = TempDir::new().unwrap();
    let build_gradle_path = temp_dir.path().join("build.gradle");
    
    // Create a build.gradle with shadow plugin
    let build_gradle_content = r#"
plugins {
    id 'com.github.johnrengelman.shadow' version '7.1.2'
    id 'java'
}

shadowJar {
    relocate 'org.apache.commons', 'myapp.shaded.commons'
    relocate 'com.google.guava', 'myapp.shaded.guava'
}
"#;
    
    fs::write(&build_gradle_path, build_gradle_content).unwrap();
    
    // Test that we can detect the shading configuration
    let shading_config = bazbom::shading::parse_gradle_shadow_config(&build_gradle_path).unwrap();
    assert!(shading_config.is_some());
    
    let config = shading_config.unwrap();
    assert_eq!(config.source, "gradle-shadow-plugin");
    assert_eq!(config.relocations.len(), 2);
}

#[test]
fn test_no_shading_detected_maven() {
    // Create a temporary directory with a Maven project without shading
    let temp_dir = TempDir::new().unwrap();
    let pom_path = temp_dir.path().join("pom.xml");
    
    // Create a simple pom.xml without maven-shade-plugin
    let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>test-app</artifactId>
    <version>1.0.0</version>
</project>
"#;
    
    fs::write(&pom_path, pom_content).unwrap();
    
    // Test that no shading configuration is detected
    let shading_config = bazbom::shading::parse_maven_shade_config(&pom_path).unwrap();
    assert!(shading_config.is_none());
}

#[test]
fn test_no_shading_detected_gradle() {
    // Create a temporary directory with a Gradle project without shading
    let temp_dir = TempDir::new().unwrap();
    let build_gradle_path = temp_dir.path().join("build.gradle");
    
    // Create a simple build.gradle without shadow plugin
    let build_gradle_content = r#"
plugins {
    id 'java'
}

dependencies {
    implementation 'com.google.guava:guava:30.1-jre'
}
"#;
    
    fs::write(&build_gradle_path, build_gradle_content).unwrap();
    
    // Test that no shading configuration is detected
    let shading_config = bazbom::shading::parse_gradle_shadow_config(&build_gradle_path).unwrap();
    assert!(shading_config.is_none());
}
