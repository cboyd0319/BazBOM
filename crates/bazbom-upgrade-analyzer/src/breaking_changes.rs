/// Additional breaking change detection strategies
///
/// This module contains:
/// - JAR bytecode comparison for API surface changes
/// - Configuration file migration detection

use anyhow::{Context, Result};
use std::path::Path;

/// JAR bytecode analysis for detecting API breaking changes
pub mod bytecode {
    use super::*;
    use bazbom::shading::{compare_jars_for_breaking_changes, ApiChange, ApiChangeType};

    /// Analyze two JAR versions for breaking changes
    ///
    /// Downloads and compares JAR files from Maven Central or local cache,
    /// detecting removed/added methods, fields, and classes.
    pub async fn detect_jar_breaking_changes(
        old_jar: &Path,
        new_jar: &Path,
    ) -> Result<Vec<ApiChange>> {
        compare_jars_for_breaking_changes(old_jar, new_jar)
    }

    /// Check if a list of API changes contains breaking changes
    pub fn has_breaking_changes(changes: &[ApiChange]) -> bool {
        changes.iter().any(|change| {
            matches!(
                change.changeType,
                ApiChangeType::RemovedMethod
                    | ApiChangeType::RemovedField
                    | ApiChangeType::RemovedClass
            )
        })
    }

    /// Count breaking vs non-breaking changes
    pub fn count_changes(changes: &[ApiChange]) -> (usize, usize) {
        let breaking = changes
            .iter()
            .filter(|c| {
                matches!(
                    c.changeType,
                    ApiChangeType::RemovedMethod
                        | ApiChangeType::RemovedField
                        | ApiChangeType::RemovedClass
                )
            })
            .count();
        let non_breaking = changes.len() - breaking;
        (breaking, non_breaking)
    }
}

/// Configuration file migration detection
pub mod config {
    use super::*;
    use serde_json::Value as JsonValue;
    use std::collections::HashMap;
    use std::fs;

    /// Detected configuration change
    #[derive(Debug, Clone)]
    pub struct ConfigChange {
        pub file_name: String,
        pub change_type: ConfigChangeType,
        pub old_key: Option<String>,
        pub new_key: Option<String>,
        pub description: String,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ConfigChangeType {
        RemovedKey,
        RenamedKey,
        ChangedFormat,
        AddedKey,
    }

    /// Detect common configuration migrations for popular frameworks
    pub fn detect_config_migrations(
        project_path: &Path,
        from_version: &str,
        to_version: &str,
    ) -> Result<Vec<ConfigChange>> {
        let mut changes = Vec::new();

        // Check for Spring Boot configuration changes
        if let Ok(spring_changes) =
            detect_spring_boot_migrations(project_path, from_version, to_version)
        {
            changes.extend(spring_changes);
        }

        // Check for Log4j configuration changes
        if let Ok(log4j_changes) =
            detect_log4j_migrations(project_path, from_version, to_version)
        {
            changes.extend(log4j_changes);
        }

        Ok(changes)
    }

    /// Detect Spring Boot configuration migrations
    fn detect_spring_boot_migrations(
        project_path: &Path,
        from_version: &str,
        to_version: &str,
    ) -> Result<Vec<ConfigChange>> {
        let mut changes = Vec::new();

        // Check if application.properties or application.yml exists
        let app_props = project_path.join("src/main/resources/application.properties");
        let app_yml = project_path.join("src/main/resources/application.yml");

        if !app_props.exists() && !app_yml.exists() {
            return Ok(changes);
        }

        // Known Spring Boot migrations
        let migrations = get_spring_boot_migration_rules();

        // Parse version numbers for comparison
        let from_major = parse_major_version(from_version);
        let to_major = parse_major_version(to_version);

        // Apply migration rules based on version upgrade
        for (version_range, rules) in migrations {
            if let Some((from, to)) = version_range {
                if from_major >= from && to_major <= to {
                    for rule in rules {
                        changes.push(ConfigChange {
                            file_name: "application.yml/properties".to_string(),
                            change_type: rule.change_type.clone(),
                            old_key: rule.old_key.clone(),
                            new_key: rule.new_key.clone(),
                            description: rule.description.clone(),
                        });
                    }
                }
            }
        }

        Ok(changes)
    }

    /// Detect Log4j configuration migrations
    fn detect_log4j_migrations(
        project_path: &Path,
        from_version: &str,
        to_version: &str,
    ) -> Result<Vec<ConfigChange>> {
        let mut changes = Vec::new();

        // Check if log4j2.xml exists
        let log4j_xml = project_path.join("src/main/resources/log4j2.xml");
        if !log4j_xml.exists() {
            return Ok(changes);
        }

        let from_major = parse_major_version(from_version);
        let to_major = parse_major_version(to_version);

        // Log4j 1.x to 2.x migration
        if from_major == 1 && to_major >= 2 {
            changes.push(ConfigChange {
                file_name: "log4j2.xml".to_string(),
                change_type: ConfigChangeType::ChangedFormat,
                old_key: None,
                new_key: None,
                description: "Log4j 1.x to 2.x requires complete configuration rewrite"
                    .to_string(),
            });
        }

        Ok(changes)
    }

    /// Known Spring Boot configuration migrations
    fn get_spring_boot_migration_rules() -> HashMap<Option<(u32, u32)>, Vec<MigrationRule>> {
        let mut rules = HashMap::new();

        // Spring Boot 2.x to 3.x migrations
        rules.insert(
            Some((2, 3)),
            vec![
                MigrationRule {
                    old_key: Some("spring.datasource.initialization-mode".to_string()),
                    new_key: Some("spring.sql.init.mode".to_string()),
                    change_type: ConfigChangeType::RenamedKey,
                    description: "Datasource initialization property renamed in Spring Boot 3"
                        .to_string(),
                },
                MigrationRule {
                    old_key: Some("spring.jpa.hibernate.use-new-id-generator-mappings".to_string()),
                    new_key: None,
                    change_type: ConfigChangeType::RemovedKey,
                    description: "Property removed, now always uses new ID generator".to_string(),
                },
            ],
        );

        rules
    }

    #[derive(Debug, Clone)]
    struct MigrationRule {
        old_key: Option<String>,
        new_key: Option<String>,
        change_type: ConfigChangeType,
        description: String,
    }

    /// Parse major version from version string
    fn parse_major_version(version: &str) -> u32 {
        version
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    /// Detect changes between two JSON configuration files
    pub fn compare_json_configs(old_path: &Path, new_path: &Path) -> Result<Vec<ConfigChange>> {
        let old_content = fs::read_to_string(old_path)?;
        let new_content = fs::read_to_string(new_path)?;

        let old_json: JsonValue = serde_json::from_str(&old_content)?;
        let new_json: JsonValue = serde_json::from_str(&new_content)?;

        let mut changes = Vec::new();
        compare_json_values("", &old_json, &new_json, &mut changes);

        Ok(changes)
    }

    fn compare_json_values(
        prefix: &str,
        old: &JsonValue,
        new: &JsonValue,
        changes: &mut Vec<ConfigChange>,
    ) {
        match (old, new) {
            (JsonValue::Object(old_map), JsonValue::Object(new_map)) => {
                // Check for removed keys
                for (key, _) in old_map {
                    if !new_map.contains_key(key) {
                        let full_key = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        changes.push(ConfigChange {
                            file_name: "config.json".to_string(),
                            change_type: ConfigChangeType::RemovedKey,
                            old_key: Some(full_key.clone()),
                            new_key: None,
                            description: format!("Configuration key '{}' was removed", full_key),
                        });
                    }
                }

                // Check for added keys
                for (key, _) in new_map {
                    if !old_map.contains_key(key) {
                        let full_key = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        changes.push(ConfigChange {
                            file_name: "config.json".to_string(),
                            change_type: ConfigChangeType::AddedKey,
                            old_key: None,
                            new_key: Some(full_key.clone()),
                            description: format!("Configuration key '{}' was added", full_key),
                        });
                    }
                }

                // Recursively compare common keys
                for (key, old_value) in old_map {
                    if let Some(new_value) = new_map.get(key) {
                        let new_prefix = if prefix.is_empty() {
                            key.clone()
                        } else {
                            format!("{}.{}", prefix, key)
                        };
                        compare_json_values(&new_prefix, old_value, new_value, changes);
                    }
                }
            }
            _ => {
                // Values at this path differ (type or content change)
                // This could be tracked as a value change if needed
            }
        }
    }
}
