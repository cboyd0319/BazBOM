use crate::analyzers::{CodeqlAnalyzer, ScaAnalyzer, SemgrepAnalyzer};
use crate::cli::{AutofixMode, CodeqlSuite, ContainerStrategy};
use crate::config::Config;
use crate::context::Context;
use crate::pipeline::{merge_sarif_reports, Analyzer};
use anyhow::Result;
use std::path::PathBuf;

pub struct ScanOrchestrator {
    config: Config,
    context: Context,
    cyclonedx: bool,
    with_semgrep: bool,
    with_codeql: Option<CodeqlSuite>,
    _autofix: Option<AutofixMode>,
    _containers: Option<ContainerStrategy>,
    no_upload: bool,
    _target: Option<String>,
}

impl ScanOrchestrator {
    pub fn new(
        workspace: PathBuf,
        out_dir: PathBuf,
        cyclonedx: bool,
        with_semgrep: bool,
        with_codeql: Option<CodeqlSuite>,
        autofix: Option<AutofixMode>,
        containers: Option<ContainerStrategy>,
        no_upload: bool,
        target: Option<String>,
    ) -> Result<Self> {
        // Load config from bazbom.toml if it exists
        let config_path = workspace.join("bazbom.toml");
        let config = if config_path.exists() {
            Config::load(&config_path)?
        } else {
            Config::default()
        };

        let context = Context::new(workspace, out_dir)?;

        Ok(Self {
            config,
            context,
            cyclonedx,
            with_semgrep,
            with_codeql,
            _autofix: autofix,
            _containers: containers,
            no_upload,
            _target: target,
        })
    }

    pub fn run(&self) -> Result<()> {
        println!("[bazbom] orchestrated scan starting...");
        
        if self.cyclonedx {
            println!("[bazbom] CycloneDX output enabled");
        }
        
        if let Some(ref target) = self._target {
            println!("[bazbom] targeting specific module: {}", target);
        }

        // Run analyzers
        let mut reports = Vec::new();

        // 1. SCA (always runs)
        let sca = ScaAnalyzer::new();
        if sca.enabled(&self.config, true) {
            match sca.run(&self.context) {
                Ok(report) => {
                    println!("[bazbom] SCA analysis complete");
                    reports.push(report);
                }
                Err(e) => eprintln!("[bazbom] SCA analysis failed: {}", e),
            }
        }

        // 2. Semgrep (optional)
        if self.with_semgrep {
            let semgrep = SemgrepAnalyzer::new();
            if semgrep.enabled(&self.config, self.with_semgrep) {
                match semgrep.run(&self.context) {
                    Ok(report) => {
                        println!("[bazbom] Semgrep analysis complete");
                        reports.push(report);
                    }
                    Err(e) => eprintln!("[bazbom] Semgrep analysis failed: {}", e),
                }
            }
        }

        // 3. CodeQL (optional)
        if let Some(ref suite) = self.with_codeql {
            let codeql = CodeqlAnalyzer::new(Some(suite.as_str().to_string()));
            if codeql.enabled(&self.config, self.with_codeql.is_some()) {
                match codeql.run(&self.context) {
                    Ok(report) => {
                        println!("[bazbom] CodeQL analysis complete");
                        reports.push(report);
                    }
                    Err(e) => eprintln!("[bazbom] CodeQL analysis failed: {}", e),
                }
            }
        }

        // Merge all SARIF reports
        if !reports.is_empty() {
            let merged = merge_sarif_reports(reports);
            let merged_path = self.context.findings_dir.join("merged.sarif");
            let json = serde_json::to_string_pretty(&merged)?;
            std::fs::write(&merged_path, json)?;
            println!("[bazbom] wrote merged SARIF to {:?}", merged_path);
            
            println!("[bazbom] total runs in merged report: {}", merged.runs.len());
        }

        // Handle autofix if requested
        if let Some(ref mode) = self._autofix {
            match mode {
                AutofixMode::Off => {}
                AutofixMode::DryRun => {
                    println!("[bazbom] autofix dry-run mode (not yet implemented)");
                }
                AutofixMode::Pr => {
                    println!("[bazbom] autofix PR mode (not yet implemented)");
                }
            }
        }

        // Handle upload
        if !self.no_upload {
            println!("[bazbom] GitHub upload enabled (not yet implemented)");
        } else {
            println!("[bazbom] skipping GitHub upload (--no-upload)");
        }

        println!("[bazbom] orchestrated scan complete");
        println!("[bazbom] outputs in: {:?}", self.context.out_dir);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_orchestrator_creation() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");

        let orchestrator = ScanOrchestrator::new(
            workspace,
            out_dir,
            false,
            false,
            None,
            None,
            None,
            true,
            None,
        )?;

        assert!(!orchestrator.cyclonedx);
        assert!(!orchestrator.with_semgrep);
        assert!(orchestrator.no_upload);

        Ok(())
    }

    #[test]
    fn test_orchestrator_run() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");

        let orchestrator = ScanOrchestrator::new(
            workspace,
            out_dir,
            false,
            false,
            None,
            None,
            None,
            true,
            None,
        )?;

        // This should not fail even without tools installed
        orchestrator.run()?;

        Ok(())
    }
}
