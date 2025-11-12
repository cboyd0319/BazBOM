use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    #[serde(default)]
    pub threats: Option<ThreatsConfig>,
    /// Named profiles for different scanning scenarios
    #[serde(default)]
    pub profile: HashMap<String, Profile>,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThreatsConfig {
    pub enabled: Option<bool>,
    pub detection_level: Option<String>,
}

/// Named profile containing scan configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    // Core scanning options
    pub reachability: Option<bool>,
    pub fast: Option<bool>,
    pub incremental: Option<bool>,
    pub base: Option<String>,
    pub benchmark: Option<bool>,
    pub ml_risk: Option<bool>,

    // Output formats
    pub format: Option<String>,
    pub cyclonedx: Option<bool>,
    pub out_dir: Option<String>,

    // Analysis tools
    pub with_semgrep: Option<bool>,
    pub with_codeql: Option<String>,

    // Autofix
    pub autofix: Option<String>,

    // Container scanning
    pub containers: Option<String>,

    // Publishing
    pub no_upload: Option<bool>,
    pub create_issues: Option<bool>,

    // Bazel-specific
    pub bazel_targets_query: Option<String>,
    pub bazel_universe: Option<String>,

    // Policy enforcement
    pub fail_on: Option<Vec<String>>,

    // Report generation
    pub reports: Option<Vec<String>>,
}

impl Profile {
    /// Merge another profile into this one, preferring values from other when present
    pub fn merge(&mut self, other: &Profile) {
        if other.reachability.is_some() {
            self.reachability = other.reachability;
        }
        if other.fast.is_some() {
            self.fast = other.fast;
        }
        if other.incremental.is_some() {
            self.incremental = other.incremental;
        }
        if other.base.is_some() {
            self.base = other.base.clone();
        }
        if other.benchmark.is_some() {
            self.benchmark = other.benchmark;
        }
        if other.ml_risk.is_some() {
            self.ml_risk = other.ml_risk;
        }
        if other.format.is_some() {
            self.format = other.format.clone();
        }
        if other.cyclonedx.is_some() {
            self.cyclonedx = other.cyclonedx;
        }
        if other.out_dir.is_some() {
            self.out_dir = other.out_dir.clone();
        }
        if other.with_semgrep.is_some() {
            self.with_semgrep = other.with_semgrep;
        }
        if other.with_codeql.is_some() {
            self.with_codeql = other.with_codeql.clone();
        }
        if other.autofix.is_some() {
            self.autofix = other.autofix.clone();
        }
        if other.containers.is_some() {
            self.containers = other.containers.clone();
        }
        if other.no_upload.is_some() {
            self.no_upload = other.no_upload;
        }
        if other.create_issues.is_some() {
            self.create_issues = other.create_issues;
        }
        if other.bazel_targets_query.is_some() {
            self.bazel_targets_query = other.bazel_targets_query.clone();
        }
        if other.bazel_universe.is_some() {
            self.bazel_universe = other.bazel_universe.clone();
        }
        if other.fail_on.is_some() {
            self.fail_on = other.fail_on.clone();
        }
        if other.reports.is_some() {
            self.reports = other.reports.clone();
        }
    }
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

    /// Get a named profile by name
    pub fn get_profile(&self, name: &str) -> Option<&Profile> {
        self.profile.get(name)
    }

    /// List available profile names
    pub fn profile_names(&self) -> Vec<String> {
        self.profile.keys().cloned().collect()
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

    #[test]
    fn test_profiles() {
        let toml = r#"
[profile.strict]
reachability = true
with_semgrep = true
with_codeql = "security-extended"
ml_risk = true
fail_on = ["critical", "high"]

[profile.fast]
fast = true
incremental = true
no_upload = true

[profile.ci]
reachability = true
benchmark = true
format = "spdx"
cyclonedx = true
"#;
        let config: Config = toml::from_str(toml).unwrap();

        // Test strict profile
        let strict = config.get_profile("strict").unwrap();
        assert_eq!(strict.reachability, Some(true));
        assert_eq!(strict.with_semgrep, Some(true));
        assert_eq!(strict.with_codeql, Some("security-extended".to_string()));
        assert_eq!(strict.ml_risk, Some(true));
        assert_eq!(strict.fail_on, Some(vec!["critical".to_string(), "high".to_string()]));

        // Test fast profile
        let fast = config.get_profile("fast").unwrap();
        assert_eq!(fast.fast, Some(true));
        assert_eq!(fast.incremental, Some(true));
        assert_eq!(fast.no_upload, Some(true));

        // Test CI profile
        let ci = config.get_profile("ci").unwrap();
        assert_eq!(ci.reachability, Some(true));
        assert_eq!(ci.benchmark, Some(true));
        assert_eq!(ci.format, Some("spdx".to_string()));
        assert_eq!(ci.cyclonedx, Some(true));

        // Test profile names
        let names = config.profile_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"strict".to_string()));
        assert!(names.contains(&"fast".to_string()));
        assert!(names.contains(&"ci".to_string()));
    }

    #[test]
    fn test_profile_merge() {
        let mut base = Profile {
            reachability: Some(true),
            fast: Some(false),
            format: Some("spdx".to_string()),
            ..Default::default()
        };

        let override_profile = Profile {
            fast: Some(true),
            ml_risk: Some(true),
            ..Default::default()
        };

        base.merge(&override_profile);

        // Original values preserved
        assert_eq!(base.reachability, Some(true));
        assert_eq!(base.format, Some("spdx".to_string()));

        // Overridden values
        assert_eq!(base.fast, Some(true));

        // New values
        assert_eq!(base.ml_risk, Some(true));
    }
}
