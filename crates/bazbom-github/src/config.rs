use crate::error::{GitHubError, Result};
use crate::models::AutoMergeConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// GitHub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// GitHub personal access token (environment variable name)
    pub token_env: String,

    /// Auto-merge configuration
    #[serde(default)]
    pub auto_merge: AutoMergeConfig,

    /// Default base branch
    #[serde(default = "default_base_branch")]
    pub default_base_branch: String,

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
        std::env::var(&self.token_env)
            .map_err(|_| GitHubError::Config(format!("Environment variable {} not set", self.token_env)))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.token_env.is_empty() {
            return Err(GitHubError::Config("Token environment variable name cannot be empty".to_string()));
        }

        Ok(())
    }
}

fn default_base_branch() -> String {
    "main".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = GitHubConfig {
            token_env: "GITHUB_TOKEN".to_string(),
            auto_merge: AutoMergeConfig::default(),
            default_base_branch: default_base_branch(),
            pr_template_path: None,
        };

        assert_eq!(config.default_base_branch, "main");
        assert!(!config.auto_merge.enabled);
    }
}
