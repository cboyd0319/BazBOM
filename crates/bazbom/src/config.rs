use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub analysis: AnalysisConfig,
    #[serde(default)]
    pub enrich: EnrichConfig,
    #[serde(default)]
    pub autofix: AutofixConfig,
    #[serde(default)]
    pub containers: ContainersConfig,
    #[serde(default)]
    pub publish: PublishConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AnalysisConfig {
    pub cyclonedx: Option<bool>,
    pub semgrep: Option<SemgrepConfig>,
    pub codeql: Option<CodeqlConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemgrepConfig {
    pub enabled: Option<bool>,
    pub ruleset: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeqlConfig {
    pub enabled: Option<bool>,
    pub suite: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnrichConfig {
    pub depsdev: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AutofixConfig {
    pub mode: Option<String>,
    pub recipe_allowlist: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContainersConfig {
    pub strategy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PublishConfig {
    pub github_code_scanning: Option<bool>,
    pub artifact: Option<bool>,
}

impl Config {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn load_or_default(path: &Path) -> Self {
        Self::load(path).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.analysis.cyclonedx.is_none());
        assert!(config.enrich.depsdev.is_none());
    }

    #[test]
    fn test_parse_config() {
        let toml = r#"
[analysis]
cyclonedx = true

[analysis.semgrep]
enabled = true
ruleset = "curated-jvm@sha256:abc123"

[analysis.codeql]
enabled = false
suite = "default"

[enrich]
depsdev = true

[autofix]
mode = "dry-run"
recipe_allowlist = ["commons-io", "jackson"]

[containers]
strategy = "auto"

[publish]
github_code_scanning = true
artifact = true
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.analysis.cyclonedx, Some(true));
        assert_eq!(
            config.analysis.semgrep.as_ref().and_then(|s| s.enabled),
            Some(true)
        );
        assert_eq!(
            config.analysis.codeql.as_ref().and_then(|c| c.enabled),
            Some(false)
        );
        assert_eq!(config.enrich.depsdev, Some(true));
        assert_eq!(config.autofix.mode, Some("dry-run".to_string()));
        assert_eq!(config.containers.strategy, Some("auto".to_string()));
        assert_eq!(config.publish.github_code_scanning, Some(true));
    }
}
