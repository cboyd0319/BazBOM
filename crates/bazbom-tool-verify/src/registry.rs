//! Tool registry for known-good versions
//!
//! This module maintains a registry of external security tools with their
//! expected checksums and versions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A tool version with its verification metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolVersion {
    /// Version string (e.g., "1.2.3")
    pub version: String,

    /// SHA-256 checksum of the tool binary
    pub sha256: String,

    /// Release date in ISO 8601 format
    pub release_date: String,

    /// Platform-specific checksums
    #[serde(default)]
    pub platforms: HashMap<String, String>,

    /// Whether this version is known to be compromised
    #[serde(default)]
    pub compromised: bool,

    /// Optional GPG signature URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpg_signature_url: Option<String>,

    /// Optional Cosign signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cosign_signature: Option<String>,
}

/// A tool in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool name (e.g., "syft", "semgrep")
    pub name: String,

    /// Tool description
    pub description: String,

    /// Official download URL pattern
    pub download_url_pattern: String,

    /// Supported versions with their verification data
    pub versions: Vec<ToolVersion>,

    /// Recommended version
    pub recommended_version: String,

    /// Minimum supported version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
}

impl Tool {
    /// Get the recommended version
    pub fn get_recommended_version(&self) -> Option<&ToolVersion> {
        self.versions
            .iter()
            .find(|v| v.version == self.recommended_version)
    }

    /// Get a specific version
    pub fn get_version(&self, version: &str) -> Option<&ToolVersion> {
        self.versions.iter().find(|v| v.version == version)
    }

    /// Check if a version is compromised
    pub fn is_version_compromised(&self, version: &str) -> bool {
        self.get_version(version)
            .map(|v| v.compromised)
            .unwrap_or(false)
    }
}

/// Registry of all known tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRegistry {
    /// Schema version for the registry format
    pub schema_version: String,

    /// Last updated timestamp
    pub last_updated: String,

    /// All registered tools
    pub tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            last_updated: chrono::Utc::now().to_rfc3339(),
            tools: HashMap::new(),
        }
    }

    /// Get a tool by name
    pub fn get_tool(&self, name: &str) -> Option<&Tool> {
        self.tools.get(name)
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }

    /// Load registry from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save registry to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        // Embedded default registry with known-good tool versions
        let registry_json = include_str!("../data/tool-registry.json");
        Self::from_json(registry_json).unwrap_or_else(|_| Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_version_serialization() {
        let version = ToolVersion {
            version: "1.0.0".to_string(),
            sha256: "abc123".to_string(),
            release_date: "2025-01-01".to_string(),
            platforms: HashMap::new(),
            compromised: false,
            gpg_signature_url: None,
            cosign_signature: None,
        };

        let json = serde_json::to_string(&version).unwrap();
        let deserialized: ToolVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(version, deserialized);
    }

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert_eq!(registry.schema_version, "1.0.0");
        assert!(registry.is_empty());
    }
}
