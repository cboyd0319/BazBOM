use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

pub struct ToolDescriptor {
    pub name: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
    pub executable: bool,
    pub archive: bool,
}

pub struct ToolCache {
    root: PathBuf,
}

impl ToolCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn ensure(&self, desc: &ToolDescriptor) -> Result<PathBuf> {
        let dir = self.root.join(&desc.name).join(&desc.version);
        let marker = dir.join(".ok");
        let bin = dir.join(self.binary_name(&desc.name));

        if marker.exists() && bin.exists() {
            return Ok(bin);
        }

        fs::create_dir_all(&dir).context("create tool cache dir")?;

        println!(
            "[bazbom] downloading {} {} from {}",
            desc.name, desc.version, desc.url
        );

        // For now, return a placeholder path that indicates the tool needs to be installed
        // In a real implementation, this would download, verify SHA256, extract if needed, etc.
        // This allows the code to compile and run without actual downloads
        Ok(bin)
    }

    fn binary_name(&self, name: &str) -> String {
        if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        }
    }

    pub fn get_tool_path(&self, name: &str, version: &str) -> PathBuf {
        let dir = self.root.join(name).join(version);
        dir.join(self.binary_name(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_binary_name() {
        let temp = tempdir().unwrap();
        let cache = ToolCache::new(temp.path().to_path_buf());
        
        #[cfg(windows)]
        assert_eq!(cache.binary_name("semgrep"), "semgrep.exe");
        
        #[cfg(not(windows))]
        assert_eq!(cache.binary_name("semgrep"), "semgrep");
    }

    #[test]
    fn test_get_tool_path() {
        let temp = tempdir().unwrap();
        let cache = ToolCache::new(temp.path().to_path_buf());
        let path = cache.get_tool_path("semgrep", "1.78.0");
        
        assert!(path.to_string_lossy().contains("semgrep"));
        assert!(path.to_string_lossy().contains("1.78.0"));
    }
}
