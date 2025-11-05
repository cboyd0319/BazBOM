use crate::config::Config;
use crate::context::Context;
use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutofixMode {
    Off,
    DryRun,
    Pr,
}

impl AutofixMode {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "off" => AutofixMode::Off,
            "pr" => AutofixMode::Pr,
            _ => AutofixMode::DryRun,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            AutofixMode::Off => "off",
            AutofixMode::DryRun => "dry-run",
            AutofixMode::Pr => "pr",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRewriteRecipe {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub recipe_type: RecipeType,
    pub target_artifact: String,
    pub current_version: String,
    pub target_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecipeType {
    UpgradeDependency,
    ReplaceArtifact,
    RemoveVulnerability,
}

pub struct OpenRewriteRunner {
    mode: AutofixMode,
    allowlist: Vec<String>,
}

impl OpenRewriteRunner {
    pub fn new(config: &Config, cli_mode: Option<AutofixMode>) -> Self {
        let mode = cli_mode.unwrap_or_else(|| {
            config
                .autofix
                .mode
                .as_ref()
                .map(|m| AutofixMode::from_str(m))
                .unwrap_or(AutofixMode::Off)
        });

        let allowlist = config.autofix.recipe_allowlist.clone().unwrap_or_else(|| {
            // Default allowlist from integration plan
            vec![
                "commons-io".to_string(),
                "jackson".to_string(),
                "log4j".to_string(),
                "spring-core".to_string(),
                "commons-".to_string(), // Prefix match for all commons-* libraries
            ]
        });

        Self { mode, allowlist }
    }

    pub fn is_enabled(&self) -> bool {
        self.mode != AutofixMode::Off
    }

    /// Generate OpenRewrite recipes for vulnerable dependencies
    pub fn generate_recipes(
        &self,
        ctx: &Context,
        vulnerabilities: &[VulnerabilityFinding],
    ) -> Result<Vec<OpenRewriteRecipe>> {
        if !self.is_enabled() {
            return Ok(Vec::new());
        }

        let mut recipes = Vec::new();

        for vuln in vulnerabilities {
            // Check if artifact is in allowlist
            if !self.is_allowed(&vuln.artifact) {
                println!(
                    "[bazbom] skipping recipe for {} (not in allowlist)",
                    vuln.artifact
                );
                continue;
            }

            if let Some(ref fix_version) = vuln.fix_version {
                let recipe = OpenRewriteRecipe {
                    name: format!("upgrade-{}-to-{}", vuln.artifact, fix_version),
                    display_name: format!("Upgrade {} to {}", vuln.artifact, fix_version),
                    description: format!(
                        "Upgrade {} from {} to {} to fix {}",
                        vuln.artifact, vuln.current_version, fix_version, vuln.cve_id
                    ),
                    recipe_type: RecipeType::UpgradeDependency,
                    target_artifact: vuln.artifact.clone(),
                    current_version: vuln.current_version.clone(),
                    target_version: fix_version.clone(),
                };
                recipes.push(recipe);
            }
        }

        // Write recipes to disk
        if !recipes.is_empty() {
            let recipes_file = ctx.fixes_dir.join("openrewrite-recipes.json");
            let json = serde_json::to_string_pretty(&recipes)?;
            std::fs::write(&recipes_file, json).context("failed to write OpenRewrite recipes")?;

            println!(
                "[bazbom] wrote {} OpenRewrite recipes to {:?}",
                recipes.len(),
                recipes_file
            );

            if self.mode == AutofixMode::DryRun {
                self.generate_dry_run_patches(ctx, &recipes)?;
            }
        }

        Ok(recipes)
    }

    fn is_allowed(&self, artifact: &str) -> bool {
        // Check if artifact matches any allowlist entry
        // Support prefix matching for entries ending with '-'
        self.allowlist.iter().any(|entry| {
            if entry.ends_with('-') {
                artifact.starts_with(entry)
            } else {
                artifact.contains(entry)
            }
        })
    }

    fn generate_dry_run_patches(&self, ctx: &Context, recipes: &[OpenRewriteRecipe]) -> Result<()> {
        println!(
            "[bazbom] generating dry-run patches for {} recipes",
            recipes.len()
        );

        // For each recipe, generate a placeholder patch file
        // In a full implementation, this would actually run OpenRewrite CLI
        // and generate real diffs
        for recipe in recipes {
            let patch_file = ctx.fixes_dir.join(format!("{}.patch", recipe.name));
            let patch_content = format!(
                "# OpenRewrite Recipe: {}\n\
                 # Description: {}\n\
                 #\n\
                 # This is a dry-run patch. To apply:\n\
                 #   bazbom fix --apply\n\
                 #\n\
                 # Or manually:\n\
                 #   # Maven: Update version in pom.xml\n\
                 #   # Gradle: Update version in build.gradle\n\
                 #\n\
                 --- a/build.gradle\n\
                 +++ b/build.gradle\n\
                 @@ -1,1 +1,1 @@\n\
                 -    implementation '{}:{}'\n\
                 +    implementation '{}:{}'\n",
                recipe.display_name,
                recipe.description,
                recipe.target_artifact,
                recipe.current_version,
                recipe.target_artifact,
                recipe.target_version
            );
            std::fs::write(&patch_file, patch_content)?;
        }

        println!("[bazbom] dry-run patches written to {:?}", ctx.fixes_dir);

        Ok(())
    }

    /// Open a PR with the fixes (placeholder for now)
    pub fn open_pr(&self, _ctx: &Context, _recipes: &[OpenRewriteRecipe]) -> Result<()> {
        if self.mode != AutofixMode::Pr {
            return Ok(());
        }

        println!("[bazbom] autofix PR mode: not yet implemented");
        println!("[bazbom] Would open PR with:");
        println!("[bazbom]   - Applied OpenRewrite recipes");
        println!("[bazbom]   - Build verification");
        println!("[bazbom]   - Link to SARIF findings");

        // FUTURE ENHANCEMENT: Implement PR creation via GitHub API
        // This would automate:
        // - Apply recipes
        // - Run builds and tests
        // - Create branch
        // - Open PR with detailed description
        // Blocked on: GitHub API integration module (see remediation.rs for partial implementation)

        Ok(())
    }
}

/// Represents a vulnerability finding that needs fixing
#[derive(Debug, Clone)]
pub struct VulnerabilityFinding {
    pub cve_id: String,
    pub artifact: String,
    pub current_version: String,
    pub fix_version: Option<String>,
    pub severity: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_autofix_mode_from_str() {
        assert_eq!(AutofixMode::from_str("off"), AutofixMode::Off);
        assert_eq!(AutofixMode::from_str("dry-run"), AutofixMode::DryRun);
        assert_eq!(AutofixMode::from_str("pr"), AutofixMode::Pr);
        assert_eq!(AutofixMode::from_str("unknown"), AutofixMode::DryRun);
    }

    #[test]
    fn test_autofix_mode_as_str() {
        assert_eq!(AutofixMode::Off.as_str(), "off");
        assert_eq!(AutofixMode::DryRun.as_str(), "dry-run");
        assert_eq!(AutofixMode::Pr.as_str(), "pr");
    }

    #[test]
    fn test_openrewrite_runner_default_allowlist() {
        let config = Config::default();
        let runner = OpenRewriteRunner::new(&config, None);

        assert!(!runner.allowlist.is_empty());
        assert!(runner.allowlist.contains(&"log4j".to_string()));
    }

    #[test]
    fn test_openrewrite_runner_custom_allowlist() {
        let mut config = Config::default();
        config.autofix.recipe_allowlist = Some(vec!["custom-lib".to_string()]);

        let runner = OpenRewriteRunner::new(&config, None);
        assert_eq!(runner.allowlist.len(), 1);
        assert!(runner.allowlist.contains(&"custom-lib".to_string()));
    }

    #[test]
    fn test_is_allowed() {
        let config = Config::default();
        let runner = OpenRewriteRunner::new(&config, None);

        assert!(runner.is_allowed("org.apache.commons.commons-io"));
        assert!(runner.is_allowed("com.fasterxml.jackson.core.jackson-databind"));
        assert!(runner.is_allowed("org.apache.logging.log4j.log4j-core"));
        assert!(!runner.is_allowed("some.random.library"));
    }

    #[test]
    fn test_is_allowed_prefix_match() {
        let mut config = Config::default();
        config.autofix.recipe_allowlist = Some(vec!["commons-".to_string()]);
        let runner = OpenRewriteRunner::new(&config, None);

        assert!(runner.is_allowed("commons-io"));
        assert!(runner.is_allowed("commons-lang3"));
        assert!(!runner.is_allowed("not-commons"));
    }

    #[test]
    fn test_is_enabled() {
        let config = Config::default();

        let runner_off = OpenRewriteRunner::new(&config, Some(AutofixMode::Off));
        assert!(!runner_off.is_enabled());

        let runner_dry = OpenRewriteRunner::new(&config, Some(AutofixMode::DryRun));
        assert!(runner_dry.is_enabled());

        let runner_pr = OpenRewriteRunner::new(&config, Some(AutofixMode::Pr));
        assert!(runner_pr.is_enabled());
    }

    #[test]
    fn test_generate_recipes() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");
        let ctx = Context::new(workspace, out_dir)?;

        let config = Config::default();
        let runner = OpenRewriteRunner::new(&config, Some(AutofixMode::DryRun));

        let vulns = vec![
            VulnerabilityFinding {
                cve_id: "CVE-2024-1234".to_string(),
                artifact: "commons-io".to_string(),
                current_version: "2.11.0".to_string(),
                fix_version: Some("2.14.0".to_string()),
                severity: "high".to_string(),
            },
            VulnerabilityFinding {
                cve_id: "CVE-2024-5678".to_string(),
                artifact: "unknown-lib".to_string(),
                current_version: "1.0.0".to_string(),
                fix_version: Some("1.1.0".to_string()),
                severity: "medium".to_string(),
            },
        ];

        let recipes = runner.generate_recipes(&ctx, &vulns)?;

        // Should only generate recipe for commons-io (in allowlist)
        assert_eq!(recipes.len(), 1);
        assert_eq!(recipes[0].target_artifact, "commons-io");
        assert_eq!(recipes[0].target_version, "2.14.0");

        Ok(())
    }
}
