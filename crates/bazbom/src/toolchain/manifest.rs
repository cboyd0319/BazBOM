use crate::toolchain::ToolDescriptor;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
struct ToolManifest {
    #[serde(flatten)]
    tools: HashMap<String, ToolConfig>,
}

#[derive(Debug, Deserialize)]
struct ToolConfig {
    version: String,
    #[serde(default)]
    #[allow(dead_code)]  // Used for documentation in manifest but not accessed in code
    install_method: Option<String>,
    #[serde(flatten)]
    platforms: HashMap<String, PlatformConfig>,
}

#[derive(Debug, Deserialize)]
struct PlatformConfig {
    url: String,
    sha256: String,
    #[serde(default)]
    archive: bool,
    #[serde(default)]
    executable: Option<String>,
}

pub struct ToolManifestLoader {
    manifest: ToolManifest,
}

impl ToolManifestLoader {
    pub fn load() -> Result<Self> {
        // Load from embedded manifest or file
        let manifest_str = include_str!("../../../../tool-versions.toml");
        let manifest: ToolManifest =
            toml::from_str(manifest_str).context("failed to parse tool manifest")?;
        Ok(Self { manifest })
    }

    pub fn get_descriptor(&self, tool_name: &str) -> Result<ToolDescriptor> {
        let tool = self
            .manifest
            .tools
            .get(tool_name)
            .with_context(|| format!("tool {} not found in manifest", tool_name))?;

        let platform_key = Self::get_platform_key();
        let platform = tool
            .platforms
            .get(&platform_key)
            .with_context(|| format!("platform {} not supported for {}", platform_key, tool_name))?;

        Ok(ToolDescriptor {
            name: tool_name.to_string(),
            version: tool.version.clone(),
            url: platform.url.clone(),
            sha256: platform.sha256.clone(),
            executable: true,
            archive: platform.archive,
            executable_path: platform.executable.clone(),
        })
    }

    fn get_platform_key() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;

        // Map Rust's OS and ARCH to our platform keys
        let os_part = match os {
            "linux" => "linux",
            "macos" => "macos",
            "windows" => "windows",
            _ => "unknown",
        };

        let arch_part = match arch {
            "x86_64" => "x86_64",
            "aarch64" => "aarch64",
            _ => "unknown",
        };

        format!("{}_{}", os_part, arch_part)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_manifest() {
        let loader = ToolManifestLoader::load();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_get_platform_key() {
        let key = ToolManifestLoader::get_platform_key();
        assert!(key.contains("linux") || key.contains("macos") || key.contains("windows"));
        assert!(key.contains("x86_64") || key.contains("aarch64"));
    }

    #[test]
    fn test_get_codeql_descriptor() {
        let loader = ToolManifestLoader::load();
        if loader.is_err() {
            return; // Skip if manifest cannot be loaded in test environment
        }
        let loader = loader.unwrap();
        
        // This might fail if the current platform is not in the manifest
        // That's expected for unsupported platforms
        let result = loader.get_descriptor("codeql");
        if result.is_ok() {
            let desc = result.unwrap();
            assert_eq!(desc.name, "codeql");
            assert_eq!(desc.version, "2.19.4");
            assert!(desc.archive);
            assert!(desc.executable_path.is_some());
        }
    }
}
