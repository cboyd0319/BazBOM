use crate::config::Config;
use crate::context::Context;
use crate::pipeline::Analyzer;
use anyhow::Result;
use bazbom_formats::sarif::SarifReport;

pub struct ScaAnalyzer;

impl ScaAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl Analyzer for ScaAnalyzer {
    fn id(&self) -> &'static str {
        "bazbom-sca"
    }

    fn enabled(&self, _cfg: &Config, _cli_override: bool) -> bool {
        // SCA is always enabled
        true
    }

    fn run(&self, _ctx: &Context) -> Result<SarifReport> {
        // Create a SARIF report for SCA findings
        let report = SarifReport::new("BazBOM-SCA", env!("CARGO_PKG_VERSION"));
        
        // For now, this is a placeholder that returns an empty report
        // In a full implementation, this would:
        // 1. Load SBOM from ctx.sbom_dir
        // 2. Load advisory data
        // 3. Match vulnerabilities to components
        // 4. Convert findings to SARIF results
        
        println!("[bazbom] SCA analyzer: placeholder (existing logic would be migrated here)");
        
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::path::PathBuf;

    #[test]
    fn test_sca_analyzer_enabled() {
        let analyzer = ScaAnalyzer::new();
        let config = Config::default();
        assert!(analyzer.enabled(&config, false));
    }

    #[test]
    fn test_sca_analyzer_run() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");
        let ctx = Context::new(workspace, out_dir)?;

        let analyzer = ScaAnalyzer::new();
        let report = analyzer.run(&ctx)?;
        
        assert_eq!(report.runs.len(), 1);
        assert_eq!(report.runs[0].tool.driver.name, "BazBOM-SCA");
        
        Ok(())
    }
}
