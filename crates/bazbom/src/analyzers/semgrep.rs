use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use crate::toolchain::{run_tool, ToolCache, ToolManifestLoader};
use anyhow::{Context as _, Result};
use bazbom_formats::sarif::SarifReport;

pub struct SemgrepAnalyzer;

impl SemgrepAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Analyzer for SemgrepAnalyzer {
    fn id(&self) -> &'static str {
        "semgrep"
    }

    fn enabled(&self, cfg: &Config, cli_override: bool) -> bool {
        if cli_override {
            return true;
        }
        cfg.analysis
            .semgrep
            .as_ref()
            .and_then(|s| s.enabled)
            .unwrap_or(false)
    }

    fn run(&self, ctx: &Context) -> Result<SarifReport> {
        println!("[bazbom] running Semgrep analysis...");

        // Check if semgrep is available in PATH (user installed)
        let semgrep_result = std::process::Command::new("semgrep")
            .arg("--version")
            .output();

        let semgrep_bin = if semgrep_result.is_ok() {
            // Use system-installed semgrep
            println!("[bazbom] using system-installed Semgrep");
            std::path::PathBuf::from("semgrep")
        } else {
            // Try to use managed installation from tool cache
            println!("[bazbom] Semgrep not found in PATH, checking tool cache...");
            
            match ToolManifestLoader::load() {
                Ok(loader) => match loader.get_descriptor("semgrep") {
                    Ok(desc) => {
                        let cache = ToolCache::new(ctx.tool_cache.clone());
                        match cache.ensure(&desc) {
                            Ok(path) => {
                                println!("[bazbom] using managed Semgrep from cache");
                                path
                            }
                            Err(e) => {
                                println!("[bazbom] failed to download Semgrep: {}", e);
                                println!("[bazbom] Install Semgrep: https://semgrep.dev/docs/getting-started/");
                                println!("[bazbom] Or use: pipx install semgrep");
                                return Ok(SarifReport::new("Semgrep", "not-installed"));
                            }
                        }
                    }
                    Err(e) => {
                        println!("[bazbom] tool manifest error: {}", e);
                        println!("[bazbom] Install Semgrep: https://semgrep.dev/docs/getting-started/");
                        return Ok(SarifReport::new("Semgrep", "not-installed"));
                    }
                },
                Err(e) => {
                    println!("[bazbom] failed to load tool manifest: {}", e);
                    return Ok(SarifReport::new("Semgrep", "not-installed"));
                }
            }
        };

        // Use curated JVM ruleset if available, otherwise use auto
        let rules_path = ctx.workspace.join("rules/semgrep/semgrep-jvm.yml");
        let config_value = if rules_path.exists() {
            println!("[bazbom] using curated JVM ruleset");
            rules_path.to_str().unwrap()
        } else {
            println!("[bazbom] using Semgrep auto configuration");
            "auto"
        };

        let args = vec![
            "--config", config_value,
            "--sarif",
            "--json",
            "--timeout", "120",
            ".",
        ];

        let output = run_tool(&semgrep_bin, &args, &ctx.workspace, 180)?;

        if output.exit_code != 0 && !output.stderr.is_empty() {
            eprintln!("[bazbom] Semgrep stderr: {}", output.stderr);
        }

        // Parse SARIF output from stdout
        let sarif: SarifReport = serde_json::from_str(&output.stdout)
            .context("failed to parse Semgrep SARIF output")?;

        println!("[bazbom] Semgrep found {} runs", sarif.runs.len());
        
        // Write to findings directory
        let output_path = ctx.findings_dir.join("semgrep.sarif");
        std::fs::write(&output_path, &output.stdout)
            .context("failed to write Semgrep SARIF")?;
        
        println!("[bazbom] wrote Semgrep findings to {:?}", output_path);

        Ok(sarif)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semgrep_analyzer_enabled() {
        let analyzer = SemgrepAnalyzer::new();
        
        // Test CLI override
        let config = Config::default();
        assert!(analyzer.enabled(&config, true));
        
        // Test config enabled
        let mut config = Config::default();
        config.analysis.semgrep = Some(crate::config::SemgrepConfig {
            enabled: Some(true),
            ruleset: None,
        });
        assert!(analyzer.enabled(&config, false));
        
        // Test disabled by default
        let config = Config::default();
        assert!(!analyzer.enabled(&config, false));
    }

    #[test]
    fn test_semgrep_analyzer_id() {
        let analyzer = SemgrepAnalyzer::new();
        assert_eq!(analyzer.id(), "semgrep");
    }
}
