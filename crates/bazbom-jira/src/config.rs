use crate::error::{JiraError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Jira configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Jira instance URL
    pub url: String,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// Default project key
    pub project: String,

    /// Default issue type
    #[serde(default = "default_issue_type")]
    pub issue_type: String,

    /// Auto-create configuration
    #[serde(default)]
    pub auto_create: AutoCreateConfig,

    /// Custom field mappings
    #[serde(default)]
    pub custom_fields: std::collections::HashMap<String, String>,

    /// Routing rules
    #[serde(default)]
    pub routing: Vec<RoutingRule>,

    /// SLA configuration
    #[serde(default)]
    pub sla: SlaConfig,

    /// Sync configuration
    #[serde(default)]
    pub sync: SyncConfig,

    /// Webhook configuration
    #[serde(default)]
    pub webhook: WebhookConfig,
}

impl JiraConfig {
    /// Load configuration from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(path)
            .map_err(|e| JiraError::Config(format!("Failed to read config file: {}", e)))?;

        let config: JiraConfig = serde_yaml::from_str(&contents)
            .map_err(|e| JiraError::Config(format!("Failed to parse config file: {}", e)))?;

        Ok(config)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(JiraError::Config("URL cannot be empty".to_string()));
        }

        if self.project.is_empty() {
            return Err(JiraError::Config("Project key cannot be empty".to_string()));
        }

        Ok(())
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Auth type: "api-token", "pat", or "oauth2"
    #[serde(rename = "type")]
    pub auth_type: String,

    /// Environment variable containing the token
    #[serde(default)]
    pub token_env: Option<String>,

    /// Environment variable containing the username (for Basic auth)
    #[serde(default)]
    pub username_env: Option<String>,
}

/// Auto-create configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutoCreateConfig {
    /// Enable auto-create
    #[serde(default)]
    pub enabled: bool,

    /// Minimum priority (P0, P1, P2, P3, P4)
    #[serde(default)]
    pub min_priority: Option<String>,

    /// Only create tickets for reachable vulnerabilities
    #[serde(default = "default_true")]
    pub only_reachable: bool,
}

/// Routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    /// Package pattern (regex)
    pub pattern: String,

    /// Project key
    #[serde(default)]
    pub project: Option<String>,

    /// Component name
    #[serde(default)]
    pub component: Option<String>,

    /// Assignee
    #[serde(default)]
    pub assignee: Option<String>,

    /// Labels
    #[serde(default)]
    pub labels: Vec<String>,

    /// Priority
    #[serde(default)]
    pub priority: Option<String>,
}

/// SLA configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlaConfig {
    /// SLA for P0 (e.g., "24h")
    #[serde(rename = "P0", default)]
    pub p0: Option<String>,

    /// SLA for P1 (e.g., "7d")
    #[serde(rename = "P1", default)]
    pub p1: Option<String>,

    /// SLA for P2 (e.g., "30d")
    #[serde(rename = "P2", default)]
    pub p2: Option<String>,

    /// SLA for P3 (e.g., "90d")
    #[serde(rename = "P3", default)]
    pub p3: Option<String>,

    /// SLA for P4 (e.g., "none")
    #[serde(rename = "P4", default)]
    pub p4: Option<String>,
}

/// Sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Enable bidirectional sync
    #[serde(default = "default_true")]
    pub bidirectional: bool,

    /// Auto-close tickets when vulnerability is fixed
    #[serde(default = "default_true")]
    pub auto_close_on_fix: bool,

    /// Update tickets on re-scan
    #[serde(default = "default_true")]
    pub update_on_rescan: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            bidirectional: true,
            auto_close_on_fix: true,
            update_on_rescan: true,
        }
    }
}

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebhookConfig {
    /// Enable webhook server
    #[serde(default)]
    pub enabled: bool,

    /// Port for webhook server
    #[serde(default = "default_webhook_port")]
    pub port: u16,

    /// Environment variable containing webhook secret
    #[serde(default)]
    pub secret_env: Option<String>,
}

fn default_issue_type() -> String {
    "Bug".to_string()
}

fn default_true() -> bool {
    true
}

fn default_webhook_port() -> u16 {
    8080
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_values() {
        let config = SyncConfig::default();
        assert!(config.bidirectional);
        assert!(config.auto_close_on_fix);
        assert!(config.update_on_rescan);
    }
}
