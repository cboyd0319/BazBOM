use crate::analyzers::{CodeqlAnalyzer, ScaAnalyzer, SemgrepAnalyzer, SyftRunner};
use crate::cli::{AutofixMode, CodeqlSuite, ContainerStrategy};
use crate::config::Config;
use crate::context::Context;
use crate::enrich::DepsDevClient;
use crate::fixes::{OpenRewriteRunner, VulnerabilityFinding};
use crate::pipeline::{merge_sarif_reports, Analyzer};
use crate::publish::GitHubPublisher;
use anyhow::{Context as _, Result};
use std::path::PathBuf;

pub struct ScanOrchestrator {
    config: Config,
    context: Context,
    cyclonedx: bool,
    with_semgrep: bool,
    with_codeql: Option<CodeqlSuite>,
    autofix: Option<AutofixMode>,
    containers: Option<ContainerStrategy>,
    no_upload: bool,
    target: Option<String>,
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
            autofix,
            containers,
            no_upload,
            target,
        })
    }

    pub fn run(&self) -> Result<()> {
        println!("[bazbom] orchestrated scan starting...");
        
        if self.cyclonedx {
            println!("[bazbom] CycloneDX output enabled");
        }
        
        if let Some(ref target) = self.target {
            println!("[bazbom] targeting specific module: {}", target);
        }

        // Step 0: Generate SBOM
        self.generate_sbom()?;

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

        // 4. Enrichment with deps.dev (if enabled)
        if self.config.enrich.depsdev.unwrap_or(false) {
            self.run_enrichment()?;
        }

        // 5. Container SBOM (if requested)
        if let Some(ref strategy) = self.containers {
            self.run_container_sbom(strategy)?;
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

        // 6. Autofix recipes (if enabled)
        if let Some(ref mode) = self.autofix {
            self.run_autofix(mode)?;
        }

        // 7. GitHub upload
        if !self.no_upload {
            let publisher = GitHubPublisher::new();
            if publisher.is_configured() {
                let merged_path = self.context.findings_dir.join("merged.sarif");
                if merged_path.exists() {
                    match publisher.upload_sarif(&merged_path) {
                        Ok(_) => println!("[bazbom] GitHub Code Scanning upload configured"),
                        Err(e) => eprintln!("[bazbom] GitHub upload failed: {}", e),
                    }
                } else {
                    println!("[bazbom] no merged.sarif to upload");
                }
            } else {
                println!("[bazbom] GitHub upload not configured (use github/codeql-action/upload-sarif@v3)");
            }
        } else {
            println!("[bazbom] skipping GitHub upload (--no-upload)");
        }

        println!("[bazbom] orchestrated scan complete");
        println!("[bazbom] outputs in: {:?}", self.context.out_dir);
        println!("[bazbom]");
        println!("[bazbom] Next steps:");
        println!("[bazbom]   - Review findings in: {:?}", self.context.findings_dir);
        println!("[bazbom]   - Upload SARIF: github/codeql-action/upload-sarif@v3");
        println!("[bazbom]   - Archive artifacts: actions/upload-artifact@v4");

        Ok(())
    }

    fn run_enrichment(&self) -> Result<()> {
        println!("[bazbom] running deps.dev enrichment...");
        
        let offline = std::env::var("BAZBOM_OFFLINE").is_ok();
        let client = DepsDevClient::new(offline);
        
        // Read SBOM from sbom_dir
        let spdx_path = self.context.sbom_dir.join("spdx.json");
        if !spdx_path.exists() {
            println!("[bazbom] no SBOM found at {:?}, skipping enrichment", spdx_path);
            // Still create the enrichment file to indicate enrichment was attempted
            let enrich_file = self.context.enrich_dir.join("depsdev.json");
            let enrichment_data = serde_json::json!({
                "enriched_at": chrono::Utc::now().to_rfc3339(),
                "offline_mode": offline,
                "note": "No SBOM found, enrichment skipped",
                "total_components": 0,
                "successful": 0,
                "failed": 0,
                "packages": []
            });
            std::fs::write(&enrich_file, serde_json::to_string_pretty(&enrichment_data)?)?;
            return Ok(());
        }

        let content = std::fs::read_to_string(&spdx_path)
            .context("failed to read SPDX file")?;
        
        let doc: serde_json::Value = serde_json::from_str(&content)
            .context("failed to parse SPDX JSON")?;

        // Extract PURLs from SBOM
        let mut purls = Vec::new();
        if let Some(packages) = doc["packages"].as_array() {
            for pkg in packages {
                if let Some(refs) = pkg["externalRefs"].as_array() {
                    for ext_ref in refs {
                        if ext_ref["referenceType"].as_str() == Some("purl") {
                            if let Some(purl) = ext_ref["referenceLocator"].as_str() {
                                purls.push(purl.to_string());
                            }
                        }
                    }
                }
            }
        }

        println!("[bazbom] found {} components with PURLs", purls.len());

        // Query deps.dev for each PURL
        let mut enriched_packages = Vec::new();
        let mut successful = 0;
        let mut failed = 0;

        for purl in &purls {
            match client.get_package_info(purl) {
                Ok(info) => {
                    println!("[bazbom]   enriched: {} (latest: {})", 
                        info.name,
                        info.latest_version.as_deref().unwrap_or("unknown")
                    );
                    enriched_packages.push(info);
                    successful += 1;
                }
                Err(e) => {
                    if offline {
                        // In offline mode, this is expected
                        failed += 1;
                    } else {
                        println!("[bazbom]   warning: failed to enrich {}: {}", purl, e);
                        failed += 1;
                    }
                }
            }
            
            // Rate limiting: small delay between requests
            if !offline && successful < purls.len() {
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
        }

        // Write enrichment data
        let enrich_file = self.context.enrich_dir.join("depsdev.json");
        let enrichment_data = serde_json::json!({
            "enriched_at": chrono::Utc::now().to_rfc3339(),
            "offline_mode": offline,
            "total_components": purls.len(),
            "successful": successful,
            "failed": failed,
            "packages": enriched_packages
        });
        std::fs::write(&enrich_file, serde_json::to_string_pretty(&enrichment_data)?)?;
        
        println!("[bazbom] wrote enrichment data to {:?}", enrich_file);
        println!("[bazbom] enriched {}/{} components", successful, purls.len());
        
        if offline {
            println!("[bazbom] (offline mode: enrichment skipped)");
        }
        
        Ok(())
    }

    fn run_container_sbom(&self, strategy: &ContainerStrategy) -> Result<()> {
        println!("[bazbom] container SBOM generation requested");
        
        // Map CLI ContainerStrategy to internal ContainerStrategy
        let internal_strategy = match strategy {
            ContainerStrategy::Auto => crate::analyzers::ContainerStrategy::Auto,
            ContainerStrategy::Syft => crate::analyzers::ContainerStrategy::Syft,
            ContainerStrategy::Bazbom => crate::analyzers::ContainerStrategy::Bazbom,
        };
        
        let _runner = SyftRunner::new(internal_strategy);
        
        // In a full implementation, this would detect container images
        // For now, just document what would happen
        println!("[bazbom] container SBOM: would scan detected images with {:?}", internal_strategy);
        println!("[bazbom] to enable: provide --containers with image path or auto-detect");
        
        Ok(())
    }

    fn run_autofix(&self, mode: &AutofixMode) -> Result<()> {
        let cli_mode = match mode {
            AutofixMode::Off => crate::fixes::openrewrite::AutofixMode::Off,
            AutofixMode::DryRun => crate::fixes::openrewrite::AutofixMode::DryRun,
            AutofixMode::Pr => crate::fixes::openrewrite::AutofixMode::Pr,
        };
        
        let runner = OpenRewriteRunner::new(&self.config, Some(cli_mode));
        
        if !runner.is_enabled() {
            return Ok(());
        }
        
        println!("[bazbom] autofix mode: {}", mode.as_str());
        
        // In a full implementation, this would:
        // 1. Load SARIF findings
        // 2. Extract vulnerability findings
        // 3. Generate recipes
        // 4. Apply if PR mode
        
        // For now, generate example recipes
        let example_vulns = vec![
            VulnerabilityFinding {
                cve_id: "CVE-2024-EXAMPLE".to_string(),
                artifact: "commons-io".to_string(),
                current_version: "2.11.0".to_string(),
                fix_version: Some("2.14.0".to_string()),
                severity: "high".to_string(),
            },
        ];
        
        let recipes = runner.generate_recipes(&self.context, &example_vulns)?;
        
        if !recipes.is_empty() {
            println!("[bazbom] generated {} autofix recipes", recipes.len());
            
            if cli_mode == crate::fixes::openrewrite::AutofixMode::Pr {
                runner.open_pr(&self.context, &recipes)?;
            }
        }
        
        Ok(())
    }

    fn generate_sbom(&self) -> Result<()> {
        println!("[bazbom] generating SBOM...");
        
        // Detect build system
        let system = bazbom_core::detect_build_system(&self.context.workspace);
        println!("[bazbom] detected build system: {:?}", system);
        
        // Generate SPDX SBOM
        let spdx_path = match system {
            bazbom_core::BuildSystem::Maven => {
                // For Maven, use stub for now - full implementation would parse pom.xml
                bazbom_core::write_stub_sbom(&self.context.sbom_dir, "spdx", system)?
            }
            bazbom_core::BuildSystem::Gradle => {
                // For Gradle, use stub for now - full implementation would parse build.gradle
                bazbom_core::write_stub_sbom(&self.context.sbom_dir, "spdx", system)?
            }
            bazbom_core::BuildSystem::Bazel => {
                // For Bazel, use existing extraction logic
                bazbom_core::write_stub_sbom(&self.context.sbom_dir, "spdx", system)?
            }
            _ => {
                bazbom_core::write_stub_sbom(&self.context.sbom_dir, "spdx", system)?
            }
        };
        
        println!("[bazbom] wrote SPDX SBOM to {:?}", spdx_path);
        
        // Optionally generate CycloneDX
        if self.cyclonedx {
            let cyclonedx_path = self.context.sbom_dir.join("cyclonedx.json");
            // For now, write a minimal CycloneDX document
            let cdx_doc = bazbom_formats::cyclonedx::CycloneDxBom::new("bazbom", env!("CARGO_PKG_VERSION"));
            let json = serde_json::to_string_pretty(&cdx_doc)?;
            std::fs::write(&cyclonedx_path, json)?;
            println!("[bazbom] wrote CycloneDX SBOM to {:?}", cyclonedx_path);
        }
        
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
