use crate::error::{GitHubError, Result};
use crate::models::AutoMergeConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Default GitHub owner/org
    pub owner: String,

    /// Default repository
    #[serde(default)]
    pub repo: Option<String>,

    /// Default base branch
    #[serde(default = "default_base_branch")]
    pub base_branch: String,

    /// Enable auto-merge
    #[serde(default)]
    pub auto_merge: bool,

    /// Auto-merge minimum confidence (0-100)
    #[serde(default = "default_min_confidence")]
    pub auto_merge_min_confidence: u8,

    /// Require tests to pass before merge
    #[serde(default = "default_true")]
    pub require_tests_pass: bool,

    /// Number of required approvals
    #[serde(default)]
    pub require_approvals: u8,

    /// GitHub personal access token (environment variable name, optional)
    #[serde(default)]
    pub token_env: Option<String>,

    /// Auto-merge configuration (advanced)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_merge_config: Option<AutoMergeConfig>,

    /// PR template path
    #[serde(default)]
    pub pr_template_path: Option<String>,
}

impl GitHubConfig {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| GitHubError::Config(format!("Failed to read config file: {}", e)))?;

        let config: GitHubConfig = serde_yaml::from_str(&contents)
            .map_err(|e| GitHubError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Get the GitHub token from environment
    pub fn get_token(&self) -> Result<String> {
        let token_env = self.token_env.as_deref().unwrap_or("GITHUB_TOKEN");
        std::env::var(token_env)
            .map_err(|_| GitHubError::Config(format!("Environment variable {} not set", token_env)))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.owner.is_empty() {
            return Err(GitHubError::Config("Owner cannot be empty".to_string()));
        }

        Ok(())
    }
}

fn default_base_branch() -> String {
    "main".to_string()
}

fn default_min_confidence() -> u8 {
    80
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = GitHubConfig {
            owner: "test-owner".to_string(),
            repo: Some("test-repo".to_string()),
            base_branch: default_base_branch(),
            auto_merge: false,
            auto_merge_min_confidence: default_min_confidence(),
            require_tests_pass: true,
            require_approvals: 0,
            token_env: Some("GITHUB_TOKEN".to_string()),
            auto_merge_config: Some(AutoMergeConfig::default()),
            pr_template_path: None,
        };

        assert_eq!(config.base_branch, "main");
        assert!(!config.auto_merge);
    }
}
