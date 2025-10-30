use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use crate::toolchain::run_tool;
use anyhow::{Context as _, Result};
use bazbom_formats::sarif::SarifReport;
use std::path::PathBuf;

pub struct CodeqlAnalyzer {
    suite: String,
}

impl CodeqlAnalyzer {
    pub fn new(suite: Option<String>) -> Self {
        Self {
            suite: suite.unwrap_or_else(|| "default".to_string()),
        }
    }
}

impl Analyzer for CodeqlAnalyzer {
    fn id(&self) -> &'static str {
        "codeql"
    }

    fn enabled(&self, cfg: &Config, cli_override: bool) -> bool {
        if cli_override {
            return true;
        }
        cfg.analysis
            .codeql
            .as_ref()
            .and_then(|c| c.enabled)
            .unwrap_or(false)
    }

    fn run(&self, ctx: &Context) -> Result<SarifReport> {
        println!("[bazbom] running CodeQL analysis (suite: {})...", self.suite);

        // Check if codeql is available in PATH
        let codeql_result = std::process::Command::new("codeql")
            .arg("version")
            .output();

        if codeql_result.is_err() {
            println!("[bazbom] CodeQL CLI not found in PATH");
            println!("[bazbom] Install CodeQL CLI: https://github.com/github/codeql-cli-binaries/releases");
            
            // Return empty report rather than failing
            return Ok(SarifReport::new("CodeQL", "not-installed"));
        }

        // Create a database directory
        let db_dir = ctx.out_dir.join("codeql-db");
        std::fs::create_dir_all(&db_dir)?;

        // For now, just return a placeholder since full CodeQL integration
        // requires build system-specific logic
        println!("[bazbom] CodeQL integration is a placeholder");
        println!("[bazbom] Full implementation requires:");
        println!("  1. Database creation with build command");
        println!("  2. Query pack selection based on suite");
        println!("  3. Database analysis");
        
        let report = SarifReport::new("CodeQL", "placeholder");
        
        // Write empty report to findings
        let output_path = ctx.findings_dir.join("codeql.sarif");
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&output_path, json)?;
        
        println!("[bazbom] wrote CodeQL findings to {:?}", output_path);

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_codeql_analyzer_enabled() {
        let analyzer = CodeqlAnalyzer::new(None);
        
        // Test CLI override
        let config = Config::default();
        assert!(analyzer.enabled(&config, true));
        
        // Test config enabled
        let mut config = Config::default();
        config.analysis.codeql = Some(crate::config::CodeqlConfig {
            enabled: Some(true),
            suite: None,
        });
        assert!(analyzer.enabled(&config, false));
        
        // Test disabled by default
        let config = Config::default();
        assert!(!analyzer.enabled(&config, false));
    }

    #[test]
    fn test_codeql_analyzer_id() {
        let analyzer = CodeqlAnalyzer::new(None);
        assert_eq!(analyzer.id(), "codeql");
    }

    #[test]
    fn test_codeql_analyzer_suite() {
        let analyzer1 = CodeqlAnalyzer::new(None);
        assert_eq!(analyzer1.suite, "default");
        
        let analyzer2 = CodeqlAnalyzer::new(Some("security-extended".to_string()));
        assert_eq!(analyzer2.suite, "security-extended");
    }
}
