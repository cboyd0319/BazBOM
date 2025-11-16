//! Tool verifier implementation
//!
//! This module performs the actual verification of external tools.

use crate::error::{ToolVerifyError, ToolVerifyResult};
use crate::registry::ToolRegistry;
use bazbom_crypto::hashing::hash_file;
use std::path::Path;
use std::process::Command;

/// Configuration for tool verification
#[derive(Debug, Clone)]
pub struct VerifyConfig {
    /// Whether to enforce verification (fail if verification fails)
    pub enforce: bool,

    /// Whether to check for compromised versions
    pub check_compromised: bool,

    /// Whether to allow unregistered tools
    pub allow_unregistered: bool,

    /// Custom registry (if None, uses default)
    pub custom_registry: Option<ToolRegistry>,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self {
            enforce: true,
            check_compromised: true,
            allow_unregistered: false,
            custom_registry: None,
        }
    }
}

/// Result of tool verification
#[derive(Debug, Clone, PartialEq)]
pub enum VerifyStatus {
    /// Tool verified successfully
    Verified,

    /// Tool verification failed
    Failed(String),

    /// Tool not in registry (allowed if allow_unregistered=true)
    Unregistered,

    /// Tool is a known compromised version
    Compromised,

    /// Verification skipped (not enforced)
    Skipped,
}

/// Tool verifier
pub struct ToolVerifier {
    registry: ToolRegistry,
    config: VerifyConfig,
}

impl ToolVerifier {
    /// Create a new verifier with default registry
    pub fn new() -> Self {
        Self::with_config(VerifyConfig::default())
    }

    /// Create a new verifier with custom configuration
    pub fn with_config(config: VerifyConfig) -> Self {
        let registry = config
            .custom_registry
            .clone()
            .unwrap_or_default();

        Self { registry, config }
    }

    /// Verify a tool by name (searches PATH)
    pub fn verify_tool(&self, tool_name: &str) -> ToolVerifyResult<VerifyStatus> {
        // Find tool in PATH
        let tool_path = which::which(tool_name).map_err(|_| {
            ToolVerifyError::ToolNotFound(tool_name.to_string())
        })?;

        self.verify_tool_at_path(tool_name, &tool_path)
    }

    /// Verify a tool at a specific path
    pub fn verify_tool_at_path(
        &self,
        tool_name: &str,
        tool_path: &Path,
    ) -> ToolVerifyResult<VerifyStatus> {
        // Get tool from registry
        let tool = match self.registry.get_tool(tool_name) {
            Some(t) => t,
            None => {
                if self.config.allow_unregistered {
                    log::warn!("Tool {} not in registry, but allowed", tool_name);
                    return Ok(VerifyStatus::Unregistered);
                } else {
                    return Err(ToolVerifyError::ToolNotInRegistry(tool_name.to_string()));
                }
            }
        };

        // Get tool version
        let version = self.detect_version(tool_name, tool_path)?;

        // Check if version is compromised
        if self.config.check_compromised && tool.is_version_compromised(&version) {
            log::error!("Tool {} version {} is known to be compromised!", tool_name, version);
            return Ok(VerifyStatus::Compromised);
        }

        // Get expected checksum from registry
        let tool_version = tool.get_version(&version).ok_or_else(|| {
            ToolVerifyError::UnsupportedVersion {
                tool: tool_name.to_string(),
                version: version.clone(),
            }
        })?;

        // Compute actual checksum
        let actual_checksum = hash_file(tool_path)?;

        // Verify checksum
        if actual_checksum != tool_version.sha256 {
            // Try platform-specific checksums
            let platform = Self::detect_platform();
            if let Some(platform_checksum) = tool_version.platforms.get(&platform) {
                if actual_checksum == *platform_checksum {
                    log::info!("Tool {} verified (platform-specific)", tool_name);
                    return Ok(VerifyStatus::Verified);
                }
            }

            // Checksum mismatch
            if self.config.enforce {
                return Err(ToolVerifyError::ChecksumMismatch {
                    tool: tool_name.to_string(),
                    path: tool_path.to_path_buf(),
                    expected: tool_version.sha256.clone(),
                    actual: actual_checksum,
                });
            } else {
                log::warn!(
                    "Checksum mismatch for {} but not enforcing",
                    tool_name
                );
                return Ok(VerifyStatus::Failed(format!(
                    "Checksum mismatch: expected {}, got {}",
                    tool_version.sha256, actual_checksum
                )));
            }
        }

        log::info!("Tool {} version {} verified successfully", tool_name, version);
        Ok(VerifyStatus::Verified)
    }

    /// Detect tool version by executing it
    fn detect_version(&self, tool_name: &str, tool_path: &Path) -> ToolVerifyResult<String> {
        // Try common version flags
        let version_flags = vec!["--version", "-version", "version", "-v"];

        for flag in version_flags {
            if let Ok(output) = Command::new(tool_path).arg(flag).output() {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(version) = Self::extract_version(&version_str) {
                        return Ok(version);
                    }
                }
            }
        }

        // Fallback: assume unknown version
        log::warn!("Could not detect version for {}", tool_name);
        Ok("unknown".to_string())
    }

    /// Extract version number from version output
    fn extract_version(output: &str) -> Option<String> {
        // Look for patterns like "v1.2.3" or "1.2.3"
        let re = regex::Regex::new(r"v?(\d+\.\d+\.\d+)").ok()?;
        re.captures(output)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Detect current platform
    fn detect_platform() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        format!("{}-{}", os, arch)
    }

    /// Get the tool registry
    pub fn registry(&self) -> &ToolRegistry {
        &self.registry
    }
}

impl Default for ToolVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verifier_creation() {
        let verifier = ToolVerifier::new();
        assert!(!verifier.registry().is_empty());
    }

    #[test]
    fn test_extract_version() {
        assert_eq!(
            ToolVerifier::extract_version("syft v1.2.3"),
            Some("1.2.3".to_string())
        );
        assert_eq!(
            ToolVerifier::extract_version("version 1.2.3"),
            Some("1.2.3".to_string())
        );
    }

    #[test]
    fn test_platform_detection() {
        let platform = ToolVerifier::detect_platform();
        assert!(platform.contains("-"));
    }
}
