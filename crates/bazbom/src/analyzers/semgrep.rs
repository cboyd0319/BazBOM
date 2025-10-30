use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use crate::toolchain::{run_tool, ToolCache, ToolDescriptor};
use anyhow::{Context as _, Result};
use bazbom_formats::sarif::SarifReport;

pub struct SemgrepAnalyzer;

impl SemgrepAnalyzer {
    pub fn new() -> Self {
        Self
    }

    fn get_tool_descriptor(&self) -> ToolDescriptor {
        // In a production implementation, these would come from a manifest file
        // For now, provide placeholder values
        ToolDescriptor {
            name: "semgrep".to_string(),
            version: "1.78.0".to_string(),
            url: "https://github.com/semgrep/semgrep/releases/download/v1.78.0/semgrep".to_string(),
            sha256: "placeholder".to_string(),
            executable: true,
            archive: false,
        }
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

        if semgrep_result.is_err() {
            println!("[bazbom] Semgrep not found in PATH");
            println!("[bazbom] Install Semgrep: https://semgrep.dev/docs/getting-started/");
            println!("[bazbom] Or use: pipx install semgrep");
            
            // Return empty report rather than failing
            return Ok(SarifReport::new("Semgrep", "not-installed"));
        }

        // Run semgrep with auto configuration (will use semgrep registry)
        let args = vec![
            "--config", "auto",
            "--sarif",
            "--json",
            "--timeout", "120",
            ".",
        ];

        let output = run_tool(
            &std::path::PathBuf::from("semgrep"),
            &args,
            &ctx.workspace,
            180,
        )?;

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
    use tempfile::tempdir;

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
