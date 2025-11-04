//! Multi-language ecosystem support for BazBOM
//!
//! Provides a plugin architecture for supporting different package ecosystems
//! beyond the JVM (Node.js, Python, Go, Rust, etc.)

pub mod node;
pub mod python;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Simple dependency representation for ecosystem plugins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub scope: DependencyScope,
    pub ecosystem: String,
    pub purl: Option<String>,
    pub direct: bool,
}

/// Dependency scope
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyScope {
    Runtime,
    Development,
    Test,
    Optional,
}

/// Dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub dependencies: Vec<Dependency>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    pub fn add_dependency(&mut self, dep: Dependency) {
        self.dependencies.push(dep);
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Trait for ecosystem plugins
pub trait EcosystemPlugin: Send + Sync {
    /// Name of the ecosystem (e.g., "node", "python", "go")
    fn name(&self) -> &str;

    /// Detect if project uses this ecosystem
    fn detect(&self, project_root: &Path) -> Result<bool>;

    /// Extract dependency graph from lockfiles/manifests
    fn extract_dependencies(&self, project_root: &Path) -> Result<DependencyGraph>;

    /// Optional: Get ecosystem-specific information
    fn get_metadata(&self) -> EcosystemMetadata {
        EcosystemMetadata::default()
    }
}

/// Metadata about an ecosystem
#[derive(Debug, Clone)]
pub struct EcosystemMetadata {
    /// Display name (e.g., "Node.js")
    pub display_name: String,
    /// Package registry URL (e.g., "https://registry.npmjs.org")
    pub registry_url: Option<String>,
    /// Lockfile names (e.g., ["package-lock.json", "yarn.lock"])
    pub lockfile_names: Vec<String>,
    /// Manifest file names (e.g., ["package.json"])
    pub manifest_names: Vec<String>,
}

impl Default for EcosystemMetadata {
    fn default() -> Self {
        Self {
            display_name: String::new(),
            registry_url: None,
            lockfile_names: Vec::new(),
            manifest_names: Vec::new(),
        }
    }
}

/// Registry for ecosystem plugins
pub struct EcosystemRegistry {
    plugins: Vec<Box<dyn EcosystemPlugin>>,
}

impl EcosystemRegistry {
    /// Create a new registry with default plugins
    pub fn new() -> Self {
        Self {
            plugins: vec![
                Box::new(node::NodePlugin::new()),
                Box::new(python::PythonPlugin::new()),
            ],
        }
    }

    /// Add a custom plugin
    pub fn add_plugin(&mut self, plugin: Box<dyn EcosystemPlugin>) {
        self.plugins.push(plugin);
    }

    /// Detect which ecosystems are present in a project
    pub fn detect_ecosystems(&self, project_root: &Path) -> Result<Vec<&str>> {
        let mut detected = Vec::new();

        for plugin in &self.plugins {
            if plugin.detect(project_root)? {
                detected.push(plugin.name());
            }
        }

        Ok(detected)
    }

    /// Get plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<&dyn EcosystemPlugin> {
        self.plugins
            .iter()
            .find(|p| p.name() == name)
            .map(|b| b.as_ref())
    }

    /// Get all plugins
    pub fn get_all_plugins(&self) -> Vec<&dyn EcosystemPlugin> {
        self.plugins.iter().map(|b| b.as_ref()).collect()
    }
}

impl Default for EcosystemRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = EcosystemRegistry::new();
        let plugins = registry.get_all_plugins();
        
        // Should have at least Node.js plugin
        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.name() == "node"));
    }

    #[test]
    fn test_get_plugin() {
        let registry = EcosystemRegistry::new();
        
        let node_plugin = registry.get_plugin("node");
        assert!(node_plugin.is_some());
        assert_eq!(node_plugin.unwrap().name(), "node");
        
        let unknown_plugin = registry.get_plugin("unknown");
        assert!(unknown_plugin.is_none());
    }
}
