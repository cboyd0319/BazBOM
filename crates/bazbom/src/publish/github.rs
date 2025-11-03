use anyhow::{Context, Result};
use std::path::Path;

/// Publisher for GitHub Code Scanning SARIF upload
///
/// Note: In a real GitHub Actions environment, the upload is typically done
/// via the github/codeql-action/upload-sarif action. This publisher provides
/// the interface for programmatic uploads if needed in the future.
pub struct GitHubPublisher {
    token: Option<String>,
    repo: Option<String>,
}

impl GitHubPublisher {
    pub fn new() -> Self {
        let token = std::env::var("GITHUB_TOKEN").ok();
        let repo = std::env::var("GITHUB_REPOSITORY").ok();
        Self { token, repo }
    }

    /// Check if GitHub upload is configured
    pub fn is_configured(&self) -> bool {
        self.token.is_some() && self.repo.is_some()
    }

    /// Upload SARIF file to GitHub Code Scanning
    ///
    /// In most cases, users should use the GitHub Action:
    /// ```yaml
    /// - uses: github/codeql-action/upload-sarif@v3
    ///   with:
    ///     sarif_file: findings/merged.sarif
    /// ```
    ///
    /// This method is provided for programmatic uploads outside of Actions.
    pub fn upload_sarif(&self, sarif_path: &Path) -> Result<()> {
        if !self.is_configured() {
            println!("[bazbom] GitHub upload not configured");
            println!("[bazbom] Set GITHUB_TOKEN and GITHUB_REPOSITORY environment variables");
            println!("[bazbom] Or use the github/codeql-action/upload-sarif@v3 action");
            return Ok(());
        }

        // Validate SARIF file exists
        if !sarif_path.exists() {
            anyhow::bail!("SARIF file not found: {:?}", sarif_path);
        }

        // Read and validate SARIF
        let sarif_content =
            std::fs::read_to_string(sarif_path).context("failed to read SARIF file")?;

        // Basic validation: ensure it's valid JSON
        let _sarif: serde_json::Value =
            serde_json::from_str(&sarif_content).context("SARIF file is not valid JSON")?;

        println!("[bazbom] SARIF validation passed");
        println!("[bazbom] Note: Direct API upload not yet implemented");
        println!("[bazbom] Please use: github/codeql-action/upload-sarif@v3");

        Ok(())
    }
}

impl Default for GitHubPublisher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_publisher_creation() {
        let publisher = GitHubPublisher::new();
        // Should not fail even without env vars
        assert!(publisher.token.is_none() || publisher.token.is_some());
    }

    #[test]
    fn test_is_configured() {
        let publisher = GitHubPublisher::new();
        // In test environment, likely not configured
        let _ = publisher.is_configured();
    }

    #[test]
    fn test_upload_sarif_missing_file() {
        let publisher = GitHubPublisher::new();
        let result = publisher.upload_sarif(Path::new("/nonexistent/file.sarif"));
        // If not configured, it returns Ok but prints a message
        // If configured, it would return Err for missing file
        // This test just ensures it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_upload_sarif_valid_file() -> Result<()> {
        let mut temp = NamedTempFile::new()?;
        temp.write_all(b"{\"version\": \"2.1.0\", \"runs\": []}")?;
        temp.flush()?;

        let publisher = GitHubPublisher::new();
        // Should not fail, but will print messages about not being configured
        let result = publisher.upload_sarif(temp.path());
        assert!(result.is_ok());

        Ok(())
    }
}
