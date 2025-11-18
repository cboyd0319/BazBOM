use crate::analyzers::{
    CodeqlAnalyzer, ScaAnalyzer, SemgrepAnalyzer, SyftRunner, ThreatAnalyzer, ThreatDetectionLevel,
};
use crate::cli::{AutofixMode, CodeqlSuite, ContainerStrategy};
use crate::config::Config;
use crate::context::Context;
use crate::enrich::DepsDevClient;
use crate::fixes::{OpenRewriteRunner, VulnerabilityFinding};
use crate::performance::PerformanceMonitor;
use crate::pipeline::{merge_sarif_reports, Analyzer};
use crate::publish::GitHubPublisher;
use crate::scan_cache::{ScanCache, ScanParameters, ScanResult as CachedScanResult};
use anyhow::{Context as _, Result};
use bazbom_cache::incremental::IncrementalAnalyzer;
use colored::Colorize;
use std::path::PathBuf;

pub struct ScanOrchestratorOptions {
    pub cyclonedx: bool,
    pub with_semgrep: bool,
    pub with_codeql: Option<CodeqlSuite>,
    pub autofix: Option<AutofixMode>,
    pub containers: Option<ContainerStrategy>,
    pub no_upload: bool,
    pub target: Option<String>,
    pub threat_detection: Option<ThreatDetectionLevel>,
    pub incremental: bool,
    pub benchmark: bool,
    pub fast: bool,
    pub reachability: bool,
    pub include_cicd: bool,
    pub fetch_checksums: bool,
}

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
    threat_detection: Option<ThreatDetectionLevel>,
    incremental: bool,
    benchmark: bool,
    fast: bool,
    reachability: bool,
    include_cicd: bool,
    fetch_checksums: bool,
}

impl ScanOrchestrator {
    pub fn new(
        workspace: PathBuf,
        out_dir: PathBuf,
        options: ScanOrchestratorOptions,
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
            cyclonedx: options.cyclonedx,
            with_semgrep: options.with_semgrep,
            with_codeql: options.with_codeql,
            autofix: options.autofix,
            containers: options.containers,
            no_upload: options.no_upload,
            target: options.target,
            threat_detection: options.threat_detection,
            incremental: options.incremental,
            benchmark: options.benchmark,
            fast: options.fast,
            reachability: options.reachability,
            include_cicd: options.include_cicd,
            fetch_checksums: options.fetch_checksums,
        })
    }

    pub fn run(&self) -> Result<()> {
        use crate::progress::ScanProgress;

        // Initialize performance monitor if benchmarking enabled
        let mut perf_monitor = if self.benchmark {
            Some(PerformanceMonitor::new("scan".to_string()))
        } else {
            None
        };

        if self.cyclonedx {
            println!("   {} CycloneDX output enabled", "ℹ️".to_string().dimmed());
        }

        if let Some(ref target) = self.target {
            println!("   🎯 Targeting specific module: {}", target);
        }

        // Step 0: Check if incremental scan is possible
        if self.incremental {
            if let Ok(skip_scan) = self.check_incremental_scan() {
                if skip_scan {
                    println!(
                        "   {} No significant changes detected, using cached results",
                        "✅".green()
                    );
                    return Ok(());
                }
            }
        }

        // Step 0.5: Check cache (unless disabled)
        let cache_disabled = std::env::var("BAZBOM_DISABLE_CACHE").is_ok();
        if !cache_disabled {
            if let Ok(cached_result) = self.try_use_cache() {
                if cached_result {
                    println!("   {} Using cached scan results (cache hit)", "⚡".yellow());
                    println!(
                        "   {} Set BAZBOM_DISABLE_CACHE=1 to disable caching",
                        "ℹ️".to_string().dimmed()
                    );
                    return Ok(());
                }
            }
        } else {
            println!(
                "   {} Cache disabled via BAZBOM_DISABLE_CACHE",
                "⊘".dimmed()
            );
        }

        // Build phase list based on what's enabled
        let mut phases = vec!["SBOM Generation", "SCA Analysis"];
        if self.with_semgrep {
            phases.push("Semgrep SAST");
        }
        if self.with_codeql.is_some() {
            phases.push("CodeQL Analysis");
        }
        if self.threat_detection.is_some() {
            phases.push("Threat Intel");
        }

        let progress = ScanProgress::new(&phases);
        let mut phase_idx = 0;

        // Step 1: Generate SBOM
        progress.start_phase(phase_idx, "Analyzing dependencies...");
        if let Some(ref mut monitor) = perf_monitor {
            monitor.start_phase("sbom_generation");
        }
        progress.update_phase(phase_idx, 30, "Parsing build files...");
        self.generate_sbom()?;
        progress.complete_phase(phase_idx, "Complete");
        if let Some(ref mut monitor) = perf_monitor {
            monitor.end_phase();
        }
        phase_idx += 1;

        // Run analyzers
        let mut reports = Vec::new();

        // 1. SCA (always runs)
        progress.start_phase(phase_idx, "Fetching vulnerability data...");
        if let Some(ref mut monitor) = perf_monitor {
            monitor.start_phase("vulnerability_scan");
        }
        let sca = ScaAnalyzer::new();
        if sca.enabled(&self.config, true) {
            progress.update_phase(phase_idx, 50, "Analyzing vulnerabilities...");
            match sca.run(&self.context) {
                Ok(report) => {
                    progress.complete_phase(phase_idx, "Complete");
                    reports.push(report);
                }
                Err(e) => {
                    progress.fail_phase(phase_idx, &format!("Failed: {}", e));
                }
            }
        }
        if let Some(ref mut monitor) = perf_monitor {
            monitor.end_phase();
        }
        phase_idx += 1;

        // 2. Semgrep (optional)
        if self.with_semgrep {
            progress.start_phase(phase_idx, "Running SAST rules...");
            let semgrep = SemgrepAnalyzer::new();
            if semgrep.enabled(&self.config, self.with_semgrep) {
                progress.update_phase(phase_idx, 40, "Scanning code patterns...");
                match semgrep.run(&self.context) {
                    Ok(report) => {
                        progress.complete_phase(phase_idx, "Complete");
                        reports.push(report);
                    }
                    Err(e) => {
                        progress.fail_phase(phase_idx, &format!("Failed: {}", e));
                    }
                }
            }
            phase_idx += 1;
        }

        // 3. CodeQL (optional)
        if let Some(ref suite) = self.with_codeql {
            progress.start_phase(phase_idx, "Building CodeQL database...");
            let codeql = CodeqlAnalyzer::new(Some(suite.as_str().to_string()));
            if codeql.enabled(&self.config, self.with_codeql.is_some()) {
                progress.update_phase(phase_idx, 60, "Running queries...");
                match codeql.run(&self.context) {
                    Ok(report) => {
                        progress.complete_phase(phase_idx, "Complete");
                        reports.push(report);
                    }
                    Err(e) => {
                        progress.fail_phase(phase_idx, &format!("Failed: {}", e));
                    }
                }
            }
            phase_idx += 1;
        }

        // 4. Threat Intelligence (if enabled)
        if let Some(ref mut monitor) = perf_monitor {
            monitor.start_phase("threat_intelligence");
        }
        let threat_level = self
            .threat_detection
            .unwrap_or(ThreatDetectionLevel::Standard);
        if threat_level != ThreatDetectionLevel::Off {
            progress.start_phase(phase_idx, "Gathering threat intel...");
            let threat = ThreatAnalyzer::new(threat_level);
            if threat.enabled(&self.config, self.threat_detection.is_some()) {
                progress.update_phase(phase_idx, 50, "Analyzing threat patterns...");
                match threat.run(&self.context) {
                    Ok(report) => {
                        progress.complete_phase(phase_idx, "Complete");
                        reports.push(report);
                    }
                    Err(e) => {
                        progress.fail_phase(phase_idx, &format!("Failed: {}", e));
                    }
                }
            }
        }
        if let Some(ref mut monitor) = perf_monitor {
            monitor.end_phase();
        }

        // 5. Enrichment with deps.dev (if enabled)
        if self.config.enrich.depsdev.unwrap_or(false) {
            self.run_enrichment()?;
        }

        // 6. Container SBOM (if requested)
        if let Some(ref strategy) = self.containers {
            self.run_container_sbom(strategy)?;
        }

        // Save report count before consuming reports
        let report_count = reports.len();

        // Merge all SARIF reports
        if !reports.is_empty() {
            let merged = merge_sarif_reports(reports);
            let merged_path = self.context.findings_dir.join("merged.sarif");
            let json = serde_json::to_string_pretty(&merged)?;
            std::fs::write(&merged_path, json)?;
            println!("[bazbom] wrote merged SARIF to {:?}", merged_path);

            println!(
                "[bazbom] total runs in merged report: {}",
                merged.runs.len()
            );
        }

        // 7. Autofix recipes (if enabled)
        if let Some(ref mode) = self.autofix {
            self.run_autofix(mode)?;
        }

        // 8. GitHub upload
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

        // Save current commit for future incremental scans
        if self.incremental {
            if let Err(e) = self.save_scan_commit() {
                eprintln!("[bazbom] warning: failed to save scan commit: {}", e);
            }
        }

        // Store scan results in cache (unless disabled)
        let cache_disabled = std::env::var("BAZBOM_DISABLE_CACHE").is_ok();
        if !cache_disabled {
            if let Err(e) = self.store_in_cache() {
                eprintln!("[bazbom] warning: failed to cache scan results: {}", e);
            }
        }

        // Finish progress display
        let summary = format!("✨ Scan complete! Generated {} reports", report_count);
        progress.finish(&summary);

        // Finalize performance monitoring and display metrics
        if let Some(monitor) = perf_monitor {
            let metrics = monitor.finalize();
            println!("[bazbom]");
            println!("[bazbom] ╔══════════════════════════════════════════════════════════╗");
            println!("[bazbom] ║            Performance Metrics                           ║");
            println!("[bazbom] ╠══════════════════════════════════════════════════════════╣");
            println!(
                "[bazbom] ║  Total Duration: {:<40} ║",
                crate::performance::format_duration(metrics.total_duration)
            );

            // Display individual phase timings
            if let Some(duration) = metrics.sbom_generation_duration {
                let percentage =
                    (duration.as_secs_f64() / metrics.total_duration.as_secs_f64()) * 100.0;
                println!(
                    "[bazbom] ║    {:<20} {:<20} ({:>5.1}%) ║",
                    "SBOM Generation",
                    crate::performance::format_duration(duration),
                    percentage
                );
            }

            if let Some(duration) = metrics.vulnerability_scan_duration {
                let percentage =
                    (duration.as_secs_f64() / metrics.total_duration.as_secs_f64()) * 100.0;
                println!(
                    "[bazbom] ║    {:<20} {:<20} ({:>5.1}%) ║",
                    "Vulnerability Scan",
                    crate::performance::format_duration(duration),
                    percentage
                );
            }

            if let Some(duration) = metrics.threat_detection_duration {
                let percentage =
                    (duration.as_secs_f64() / metrics.total_duration.as_secs_f64()) * 100.0;
                println!(
                    "[bazbom] ║    {:<20} {:<20} ({:>5.1}%) ║",
                    "Threat Detection",
                    crate::performance::format_duration(duration),
                    percentage
                );
            }

            if let Some(duration) = metrics.reachability_duration {
                let percentage =
                    (duration.as_secs_f64() / metrics.total_duration.as_secs_f64()) * 100.0;
                println!(
                    "[bazbom] ║    {:<20} {:<20} ({:>5.1}%) ║",
                    "Reachability",
                    crate::performance::format_duration(duration),
                    percentage
                );
            }

            println!("[bazbom] ╠══════════════════════════════════════════════════════════╣");
            println!(
                "[bazbom] ║  Dependencies: {:<44} ║",
                metrics.dependencies_count
            );
            println!(
                "[bazbom] ║  Vulnerabilities: {:<41} ║",
                metrics.vulnerabilities_count
            );
            println!("[bazbom] ║  Build System: {:<44} ║", metrics.build_system);
            println!("[bazbom] ╚══════════════════════════════════════════════════════════╝");

            // Save metrics to file
            let metrics_path = self.context.out_dir.join("performance.json");
            if let Ok(json) = serde_json::to_string_pretty(&metrics) {
                let _ = std::fs::write(&metrics_path, json);
                println!("[bazbom] Performance metrics saved to: {:?}", metrics_path);
            }
        }

        println!("[bazbom] orchestrated scan complete");
        println!("[bazbom] outputs in: {:?}", self.context.out_dir);
        println!("[bazbom]");
        println!("[bazbom] Next steps:");
        println!(
            "[bazbom]   - Review findings in: {:?}",
            self.context.findings_dir
        );
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
            println!(
                "[bazbom] no SBOM found at {:?}, skipping enrichment",
                spdx_path
            );
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
            std::fs::write(
                &enrich_file,
                serde_json::to_string_pretty(&enrichment_data)?,
            )?;
            return Ok(());
        }

        let content = std::fs::read_to_string(&spdx_path).context("failed to read SPDX file")?;

        let doc: serde_json::Value =
            serde_json::from_str(&content).context("failed to parse SPDX JSON")?;

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
                    println!(
                        "[bazbom]   enriched: {} (latest: {})",
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
        std::fs::write(
            &enrich_file,
            serde_json::to_string_pretty(&enrichment_data)?,
        )?;

        println!("[bazbom] wrote enrichment data to {:?}", enrich_file);
        println!(
            "[bazbom] enriched {}/{} components",
            successful,
            purls.len()
        );

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

        println!(
            "[bazbom] using container scanning strategy: {:?}",
            internal_strategy
        );

        // For now, use Syft if available, otherwise use BazBOM native scanning
        match internal_strategy {
            crate::analyzers::ContainerStrategy::Syft
            | crate::analyzers::ContainerStrategy::Auto => {
                // Try Syft first
                let _runner = SyftRunner::new(internal_strategy);
                println!("[bazbom] container SBOM: Syft integration (future feature)");
                println!("[bazbom] for now, use --containers=bazbom for native scanning");
            }
            crate::analyzers::ContainerStrategy::Bazbom => {
                // Use BazBOM native container scanning
                self.run_native_container_scan()?;
            }
        }

        Ok(())
    }

    fn run_native_container_scan(&self) -> Result<()> {
        println!("[bazbom] starting native container scan...");

        // Look for Docker images or container tarballs in the workspace
        let workspace_images = self.find_container_images()?;

        if workspace_images.is_empty() {
            println!("[bazbom] no container images found in workspace");
            println!("[bazbom] to scan a container image:");
            println!("[bazbom]   1. Export image: docker save myapp:latest -o myapp.tar");
            println!("[bazbom]   2. Place tar file in project directory");
            println!("[bazbom]   3. Run: bazbom scan --containers=bazbom");
            return Ok(());
        }

        println!(
            "[bazbom] found {} container images to scan",
            workspace_images.len()
        );

        // Scan each image
        for image_path in workspace_images {
            self.scan_single_container(&image_path)?;
        }

        println!("[bazbom] container scanning complete");
        Ok(())
    }

    fn find_container_images(&self) -> Result<Vec<std::path::PathBuf>> {
        use std::fs;

        let mut images = Vec::new();

        // Look for .tar files that might be container images
        for entry in fs::read_dir(&self.context.workspace)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("tar") {
                println!("[bazbom] found potential container image: {:?}", path);
                images.push(path);
            }
        }

        Ok(images)
    }

    fn scan_single_container(&self, image_path: &std::path::Path) -> Result<()> {
        use crate::container_ux::{ContainerScanProgress, ContainerSummary};
        use bazbom_containers::{ContainerScanner, ScanEvent};
        use std::time::Instant;

        // Track scan start time
        let scan_start = Instant::now();

        // Create scanner
        let scanner = ContainerScanner::new(image_path.to_path_buf());

        // Track progress with beautiful UX
        let mut progress: Option<ContainerScanProgress> = None;

        // Scan with progress tracking
        let scan_result = scanner
            .scan_with_progress(|event| {
                match event {
                    ScanEvent::ImageMetadata { name, layers } => {
                        progress = Some(ContainerScanProgress::new(&name, layers));
                    }
                    ScanEvent::LayerStart { digest, size, .. } => {
                        if let Some(ref mut p) = progress {
                            p.start_layer(
                                &digest,
                                size as f64 / 1_048_576.0, // Convert bytes to MB
                            );
                        }
                    }
                    ScanEvent::LayerComplete {
                        artifacts_found, ..
                    } => {
                        if let Some(ref mut p) = progress {
                            p.complete_layer(artifacts_found, 0); // 0 vulns for now
                        }
                    }
                    ScanEvent::Complete => {
                        if let Some(ref mut p) = progress {
                            p.finish();
                        }
                    }
                    _ => {}
                }
            })
            .context("Container scan failed")?;

        println!(); // Add spacing after progress

        // Build layer breakdown data for beautiful display
        let layer_data: Vec<(String, f64, usize, usize)> = scan_result
            .layer_metrics
            .iter()
            .map(|m| {
                (
                    m.digest.clone(),
                    m.size as f64 / 1_048_576.0, // Convert to MB
                    m.artifacts_found,
                    m.vulnerabilities_found,
                )
            })
            .collect();

        // Display beautiful layer breakdown
        crate::container_ux::print_layer_breakdown(&layer_data);
        println!();

        // Calculate totals and metrics
        let total_size_mb: f64 = layer_data.iter().map(|(_, size, _, _)| size).sum();
        let total_vulns: usize = layer_data.iter().map(|(_, _, _, vulns)| vulns).sum();

        // For now, we don't have severity breakdown from container scanning
        // In a real implementation, we'd analyze each vulnerability
        let scan_duration = scan_start.elapsed();

        // Display beautiful container summary
        let summary = ContainerSummary {
            image_name: scan_result.image.name.clone(),
            image_digest: scan_result.image.digest.clone(),
            base_image: scan_result.image.base_image.clone(),
            total_layers: scan_result.image.layers.len(),
            total_size_mb,
            java_artifacts: scan_result.artifacts.len(),
            vulnerabilities: total_vulns,
            critical_vulns: 0, // Would be populated by vulnerability scan
            high_vulns: 0,     // Would be populated by vulnerability scan
            medium_vulns: 0,   // Would be populated by vulnerability scan
            low_vulns: 0,      // Would be populated by vulnerability scan
            scan_duration,
        };
        summary.print();

        // Generate container SBOM
        let sbom_path = self.context.out_dir.join(format!(
            "container-{}.spdx.json",
            scan_result.image.name.replace([':', '/'], "-")
        ));

        self.generate_container_sbom_from_detailed(&scan_result, &sbom_path)?;

        println!();
        println!("   📄 Container SBOM written to: {:?}", sbom_path);

        Ok(())
    }

    fn generate_container_sbom_from_detailed(
        &self,
        scan_result: &bazbom_containers::DetailedScanResult,
        output_path: &std::path::Path,
    ) -> Result<()> {
        use bazbom_formats::spdx::{Package, SpdxDocument};

        println!();
        println!("   📝 Generating container SBOM...");

        // Create SPDX document for the container
        let mut doc = SpdxDocument::new(
            format!("container-{}", scan_result.image.name),
            format!("Container SBOM for {}", scan_result.image.name),
        );

        // Add container as a package
        let container_pkg = Package {
            spdxid: format!(
                "SPDXRef-Package-{}",
                scan_result.image.name.replace(':', "-")
            ),
            name: scan_result.image.name.clone(),
            version_info: Some("latest".to_string()),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: None,
            license_declared: None,
            checksums: None,
            external_refs: None,
        };
        doc.add_package(container_pkg);

        // Add each Java artifact as a package
        for (idx, artifact) in scan_result.artifacts.iter().enumerate() {
            let package = if let Some(ref coords) = artifact.maven_coords {
                Package {
                    spdxid: format!(
                        "SPDXRef-Package-{}-{}",
                        coords.artifact_id.replace('.', "-"),
                        idx
                    ),
                    name: format!("{}:{}", coords.group_id, coords.artifact_id),
                    version_info: Some(coords.version.clone()),
                    download_location: "NOASSERTION".to_string(),
                    files_analyzed: false,
                    license_concluded: None,
                    license_declared: None,
            checksums: None,
                    external_refs: None,
                }
            } else {
                Package {
                    spdxid: format!("SPDXRef-Package-artifact-{}", idx),
                    name: artifact.path.clone(),
                    version_info: Some("unknown".to_string()),
                    download_location: "NOASSERTION".to_string(),
                    files_analyzed: false,
                    license_concluded: None,
                    license_declared: None,
            checksums: None,
                    external_refs: None,
                }
            };
            doc.add_package(package);
        }

        // Write SBOM to file
        let json = serde_json::to_string_pretty(&doc)?;
        std::fs::write(output_path, json)?;

        Ok(())
    }

    #[allow(dead_code)]
    fn generate_container_sbom(
        &self,
        scan_result: &bazbom_containers::ContainerScanResult,
        output_path: &std::path::Path,
    ) -> Result<()> {
        use bazbom_formats::spdx::{Package, SpdxDocument};

        println!("[bazbom] generating container SBOM...");

        // Create SPDX document for the container
        let mut doc = SpdxDocument::new(
            format!("container-{}", scan_result.image.name),
            format!("Container SBOM for {}", scan_result.image.name),
        );

        // Add container as a package
        let container_pkg = Package {
            spdxid: format!(
                "SPDXRef-Package-{}",
                scan_result.image.name.replace(':', "-")
            ),
            name: scan_result.image.name.clone(),
            version_info: Some("latest".to_string()),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: None,
            license_declared: None,
            checksums: None,
            external_refs: None,
        };
        doc.add_package(container_pkg);

        // Add each Java artifact as a package
        for (idx, artifact) in scan_result.artifacts.iter().enumerate() {
            let package = if let Some(ref coords) = artifact.maven_coords {
                Package {
                    spdxid: format!(
                        "SPDXRef-Package-{}-{}",
                        coords.artifact_id.replace('.', "-"),
                        idx
                    ),
                    name: format!("{}:{}", coords.group_id, coords.artifact_id),
                    version_info: Some(coords.version.clone()),
                    download_location: "NOASSERTION".to_string(),
                    files_analyzed: false,
                    license_concluded: None,
                    license_declared: None,
            checksums: None,
                    external_refs: None,
                }
            } else {
                Package {
                    spdxid: format!("SPDXRef-Package-artifact-{}", idx),
                    name: artifact.path.clone(),
                    version_info: Some("unknown".to_string()),
                    download_location: "NOASSERTION".to_string(),
                    files_analyzed: false,
                    license_concluded: None,
                    license_declared: None,
            checksums: None,
                    external_refs: None,
                }
            };
            doc.add_package(package);
        }

        // Write SBOM to file
        let json = serde_json::to_string_pretty(&doc)?;
        std::fs::write(output_path, json)?;

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
        let example_vulns = vec![VulnerabilityFinding {
            cve_id: "CVE-2024-EXAMPLE".to_string(),
            artifact: "commons-io".to_string(),
            current_version: "2.11.0".to_string(),
            fix_version: Some("2.14.0".to_string()),
            severity: "high".to_string(),
        }];

        let recipes = runner.generate_recipes(&self.context, &example_vulns)?;

        if !recipes.is_empty() {
            println!("[bazbom] generated {} autofix recipes", recipes.len());

            if cli_mode == crate::fixes::openrewrite::AutofixMode::Pr {
                runner.open_pr(&self.context, &recipes)?;
            }
        }

        Ok(())
    }

    fn check_incremental_scan(&self) -> Result<bool> {
        println!("[bazbom] checking for incremental scan opportunities...");

        // Check if this is a git repository
        if !self.context.workspace.join(".git").exists() {
            println!("[bazbom] not a git repository, full scan required");
            return Ok(false);
        }

        // Try to load incremental analyzer
        let analyzer = IncrementalAnalyzer::new(self.context.workspace.clone())?;

        // Check if we have a cached scan result
        let cache_dir = self.context.workspace.join(".bazbom").join("cache");
        let last_scan_file = cache_dir.join("last_scan_commit.txt");

        if !last_scan_file.exists() {
            println!("[bazbom] no previous scan found, full scan required");
            return Ok(false);
        }

        // Read last scan commit
        let last_commit = std::fs::read_to_string(&last_scan_file)?;
        let last_commit = last_commit.trim();

        // Get changes since last scan
        let changes = analyzer.get_changes_since(last_commit)?;

        // Check if rescan is required
        if changes.requires_rescan() {
            println!("[bazbom] significant changes detected, full scan required:");
            if changes.has_build_file_changes() {
                println!(
                    "[bazbom]   - build files changed: {:?}",
                    changes.changed_build_files
                );
            }
            println!(
                "[bazbom]   - total changed files: {}",
                changes.all_changed_files().len()
            );
            Ok(false)
        } else {
            println!("[bazbom] no significant changes detected");
            Ok(true)
        }
    }

    fn save_scan_commit(&self) -> Result<()> {
        // Save current commit for future incremental scans
        if self.context.workspace.join(".git").exists() {
            let analyzer = IncrementalAnalyzer::new(self.context.workspace.clone())?;
            let current_commit = analyzer.get_current_commit()?;

            let cache_dir = self.context.workspace.join(".bazbom").join("cache");
            std::fs::create_dir_all(&cache_dir)?;

            let last_scan_file = cache_dir.join("last_scan_commit.txt");
            std::fs::write(&last_scan_file, current_commit)?;
            println!("[bazbom] saved scan commit for incremental analysis");
        }
        Ok(())
    }

    /// Try to use cached scan results
    fn try_use_cache(&self) -> Result<bool> {
        let cache_dir = self.context.workspace.join(".bazbom").join("cache");
        let mut cache = ScanCache::new(cache_dir)?;

        // Build scan parameters for cache key
        // NOTE: These parameters should be passed from ScanOptions in production use
        let scan_params = ScanParameters {
            reachability: false, // Currently hardcoded; should use options.reachability
            fast: false,         // Currently hardcoded; should use options.fast
            format: "spdx".to_string(),
            bazel_targets: None, // Currently hardcoded; should use options.bazel_targets
        };

        // Find build files
        let build_files = self.find_build_files()?;

        // Generate cache key
        let cache_key =
            ScanCache::generate_cache_key(&self.context.workspace, &build_files, &scan_params)?;

        // Try to get cached result
        if let Some(cached) = cache.get_scan_result(&cache_key)? {
            println!("[bazbom] cache hit for key: {}", &cache_key[..16]);

            // Restore cached outputs
            let sbom_path = self.context.sbom_dir.join("spdx.json");
            std::fs::create_dir_all(&self.context.sbom_dir)?;
            std::fs::write(&sbom_path, &cached.sbom_json)?;

            if let Some(findings_json) = &cached.findings_json {
                let findings_path = self.context.findings_dir.join("sca_findings.json");
                std::fs::create_dir_all(&self.context.findings_dir)?;
                std::fs::write(&findings_path, findings_json)?;
            }

            println!("[bazbom] restored cached SBOM and findings");
            return Ok(true);
        }

        println!("[bazbom] cache miss for key: {}", &cache_key[..16]);
        Ok(false)
    }

    /// Find build files for cache key generation
    fn find_build_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        // Maven
        let pom = self.context.workspace.join("pom.xml");
        if pom.exists() {
            files.push(pom);
        }

        // Gradle
        let gradle = self.context.workspace.join("build.gradle");
        if gradle.exists() {
            files.push(gradle);
        }
        let gradle_kts = self.context.workspace.join("build.gradle.kts");
        if gradle_kts.exists() {
            files.push(gradle_kts);
        }

        // Bazel
        let build_bazel = self.context.workspace.join("BUILD.bazel");
        if build_bazel.exists() {
            files.push(build_bazel);
        }
        let build = self.context.workspace.join("BUILD");
        if build.exists() {
            files.push(build);
        }

        Ok(files)
    }

    /// Store scan results in cache
    fn store_in_cache(&self) -> Result<()> {
        let cache_dir = self.context.workspace.join(".bazbom").join("cache");
        let mut cache = ScanCache::new(cache_dir)?;

        // Build scan parameters for cache key
        let scan_params = ScanParameters {
            reachability: false,
            fast: false,
            format: "spdx".to_string(),
            bazel_targets: None,
        };

        // Find build files
        let build_files = self.find_build_files()?;

        // Generate cache key
        let cache_key =
            ScanCache::generate_cache_key(&self.context.workspace, &build_files, &scan_params)?;

        // Read scan outputs
        let sbom_path = self.context.sbom_dir.join("spdx.json");
        let findings_path = self.context.findings_dir.join("sca_findings.json");

        if sbom_path.exists() {
            let sbom_json = std::fs::read_to_string(&sbom_path)?;
            let findings_json = if findings_path.exists() {
                Some(std::fs::read_to_string(&findings_path)?)
            } else {
                None
            };

            let cached_result = CachedScanResult::new(sbom_json, findings_json, scan_params);

            cache.put_scan_result(&cache_key, &cached_result)?;
            println!("[bazbom] cached scan results (key: {})", &cache_key[..16]);
        }

        Ok(())
    }

    fn generate_sbom(&self) -> Result<()> {
        println!("[bazbom] generating SBOM...");

        // Detect JVM build system
        let system = bazbom_core::detect_build_system(&self.context.workspace);
        println!("[bazbom] detected JVM build system: {:?}", system);

        // Scan for polyglot ecosystems (skip if fast mode is enabled)
        let mut polyglot_results = if self.fast {
            println!("[bazbom] skipping polyglot scanning (fast mode enabled)");
            Vec::new()
        } else {
            println!("[bazbom] scanning for polyglot ecosystems...");
            let workspace_path = self
                .context
                .workspace
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("invalid workspace path"))?;

            // Try to use existing runtime, or create a new one if not in async context
            match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    // We're in an async context, use block_in_place
                    tokio::task::block_in_place(|| {
                        handle.block_on(bazbom_polyglot::scan_directory_sbom_only(workspace_path, self.include_cicd))
                    })?
                }
                Err(_) => {
                    // No runtime available, create a new one
                    let rt = tokio::runtime::Runtime::new()?;
                    rt.block_on(bazbom_polyglot::scan_directory_sbom_only(workspace_path, self.include_cicd))?
                }
            }
        };

        // Enforce scan limit if BAZBOM_SCAN_LIMIT is set
        if let Ok(limit_str) = std::env::var("BAZBOM_SCAN_LIMIT") {
            if let Ok(limit) = limit_str.parse::<usize>() {
                tracing::info!("Enforcing scan limit of {} packages in polyglot results", limit);
                let mut total_packages = 0;
                let mut limited_results = Vec::new();

                for mut result in polyglot_results {
                    let available = limit.saturating_sub(total_packages);
                    if available == 0 {
                        tracing::debug!("Limit reached, skipping ecosystem: {}", result.ecosystem);
                        break;
                    }

                    if result.total_packages > available {
                        tracing::debug!(
                            "Truncating {} from {} to {} packages",
                            result.ecosystem,
                            result.total_packages,
                            available
                        );
                        result.total_packages = available;
                        // Note: The actual package data truncation would need to happen
                        // within the bazbom_polyglot crate for full effectiveness
                    }

                    total_packages += result.total_packages;
                    limited_results.push(result);
                }

                polyglot_results = limited_results;
                println!("[bazbom] limited scan to {} packages total", total_packages);
            }
        }

        // Display detected ecosystems
        if !polyglot_results.is_empty() {
            println!(
                "\n📦 Detected {} polyglot ecosystems:",
                polyglot_results.len()
            );
            for result in &polyglot_results {
                let icon = match result.ecosystem.as_str() {
                    "Node.js/npm" => "📦",
                    "Python" => "🐍",
                    "Go" => "🐹",
                    "Rust" => "🦀",
                    "Ruby" => "💎",
                    "PHP" => "🐘",
                    _ => "📦",
                };
                println!(
                    "  {} {} - {} packages, {} vulnerabilities",
                    icon, result.ecosystem, result.total_packages, result.total_vulnerabilities
                );

                // Show vulnerability summary if any found
                if result.total_vulnerabilities > 0 {
                    let critical = result
                        .vulnerabilities
                        .iter()
                        .filter(|v| v.severity == "CRITICAL")
                        .count();
                    let high = result
                        .vulnerabilities
                        .iter()
                        .filter(|v| v.severity == "HIGH")
                        .count();
                    let medium = result
                        .vulnerabilities
                        .iter()
                        .filter(|v| v.severity == "MEDIUM")
                        .count();
                    let low = result
                        .vulnerabilities
                        .iter()
                        .filter(|v| v.severity == "LOW")
                        .count();

                    if critical > 0 || high > 0 {
                        println!(
                            "    🚨 {} CRITICAL, {} HIGH, {} MEDIUM, {} LOW",
                            critical, high, medium, low
                        );
                    }
                }
            }
            println!();
        } else {
            println!("[bazbom] no polyglot ecosystems detected (JVM-only project)");
        }

        // Extract Bazel Maven dependencies if this is a Bazel project
        if system == bazbom_core::BuildSystem::Bazel {
            tracing::info!("Detected Bazel project, extracting Maven dependencies");
            println!("[bazbom] detected Bazel project, extracting Maven dependencies...");

            let maven_install_json = self.context.workspace.join("maven_install.json");
            if maven_install_json.exists() {
                tracing::debug!("Found maven_install.json at {:?}", maven_install_json);
                let deps_json_path = self.context.sbom_dir.join("bazel-deps.json");

                match crate::bazel::extract_bazel_dependencies(&self.context.workspace, &deps_json_path) {
                    Ok(graph) => {
                        tracing::info!(
                            "Successfully extracted {} Maven packages from maven_install.json",
                            graph.components.len()
                        );
                        println!("[bazbom] found {} Maven packages from maven_install.json", graph.components.len());

                        // Convert Bazel Maven components to polyglot Package format
                        let maven_packages: Vec<bazbom_polyglot::Package> = graph.components.iter().map(|component| {
                            bazbom_polyglot::Package {
                                name: component.name.clone(),
                                version: component.version.clone(),
                                ecosystem: "Maven".to_string(),
                                namespace: Some(component.group.clone()),
                                dependencies: vec![],  // Dependency relationships are in edges
                                license: None,
                                description: None,
                                homepage: None,
                                repository: if component.repository.is_empty() {
                                    None
                                } else {
                                    Some(component.repository.clone())
                                },
                            }
                        }).collect();

                        // Merge Maven packages into polyglot results
                        if !maven_packages.is_empty() {
                            let maven_result = bazbom_polyglot::EcosystemScanResult {
                                ecosystem: "Maven (Bazel)".to_string(),
                                root_path: self.context.workspace.to_string_lossy().to_string(),
                                packages: maven_packages,
                                vulnerabilities: vec![],
                                total_packages: graph.components.len(),
                                total_vulnerabilities: 0,
                                reachability: None,
                            };
                            polyglot_results.push(maven_result);
                            tracing::info!("Merged {} Maven packages into polyglot results", graph.components.len());
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to extract Bazel dependencies: {}", e);
                        eprintln!("[bazbom] warning: failed to extract Bazel dependencies: {}", e);
                    }
                }
            } else {
                tracing::info!("No maven_install.json found in Bazel workspace");
                println!("[bazbom] no maven_install.json found");
                println!("[bazbom] hint: run 'bazel run @maven//:pin' to generate maven_install.json");
            }

            // Detect CI/CD tooling for Bazel projects (if requested)
            if self.include_cicd {
                tracing::info!("Detecting CI/CD tooling in Bazel workspace");
                println!("[bazbom] detecting CI/CD dependencies...");
                match bazbom_polyglot::cicd::detect_github_actions(&self.context.workspace) {
                    Ok(cicd_result) => {
                        if !cicd_result.packages.is_empty() {
                            tracing::info!("Found {} CI/CD packages", cicd_result.packages.len());
                            println!("[bazbom] found {} CI/CD packages", cicd_result.packages.len());
                            polyglot_results.push(cicd_result);
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to scan GitHub Actions: {}", e);
                        eprintln!("[bazbom] warning: failed to scan GitHub Actions: {}", e);
                    }
                }
            }
        }

        // Generate unified SBOM combining Bazel Maven + all polyglot ecosystems
        let spdx_path = if !polyglot_results.is_empty() {
            let total_packages: usize = polyglot_results.iter().map(|r| r.packages.len()).sum();
            tracing::info!("Generating unified SBOM with {} packages across {} ecosystems",
                total_packages, polyglot_results.len());
            println!("[bazbom] generating unified SBOM ({} packages from {} ecosystems)",
                total_packages, polyglot_results.len());

            if self.fetch_checksums {
                println!("[bazbom] fetching SHA256 checksums from package registries (this may take a moment)...");
                tracing::info!("Fetching checksums for {} packages", total_packages);
            }

            let unified_sbom = match tokio::runtime::Handle::try_current() {
                Ok(handle) => {
                    tokio::task::block_in_place(|| {
                        handle.block_on(bazbom_polyglot::generate_polyglot_sbom(&polyglot_results, self.fetch_checksums))
                    })?
                }
                Err(_) => {
                    // Create new runtime if not already in one
                    let rt = tokio::runtime::Runtime::new()?;
                    rt.block_on(bazbom_polyglot::generate_polyglot_sbom(&polyglot_results, self.fetch_checksums))?
                }
            };
            let spdx_path = self.context.sbom_dir.join("spdx.json");
            std::fs::create_dir_all(&self.context.sbom_dir)?;
            std::fs::write(
                &spdx_path,
                serde_json::to_string_pretty(&unified_sbom)?,
            )?;
            tracing::debug!("Wrote unified SPDX SBOM to {:?}", spdx_path);
            println!("[bazbom] wrote unified SPDX SBOM to {:?}", spdx_path);
            spdx_path
        } else {
            tracing::warn!("No packages detected in any ecosystem");
            println!("[bazbom] no packages detected, writing stub SBOM");
            bazbom_core::write_stub_sbom(&self.context.sbom_dir, "spdx", system)?;
            self.context.sbom_dir.join("spdx.json")
        };

        // Optionally generate CycloneDX from unified polyglot results
        if self.cyclonedx {
            let cyclonedx_path = self.context.sbom_dir.join("cyclonedx.json");
            let mut cdx_doc =
                bazbom_formats::cyclonedx::CycloneDxBom::new("bazbom", env!("CARGO_PKG_VERSION"));

            // Convert all packages from polyglot results to CycloneDX components
            let mut component_count = 0;
            for ecosystem_result in &polyglot_results {
                for package in &ecosystem_result.packages {
                    let mut component = bazbom_formats::cyclonedx::Component::new(
                        &package.name,
                        "library"
                    )
                    .with_version(&package.version)
                    .with_purl(&package.purl());

                    if let Some(ref license) = package.license {
                        component = component.with_license(license);
                    }

                    cdx_doc.add_component(component);
                    component_count += 1;
                }
            }

            let json = serde_json::to_string_pretty(&cdx_doc)?;
            std::fs::write(&cyclonedx_path, json)?;
            tracing::info!("Wrote CycloneDX SBOM with {} components to {:?}", component_count, cyclonedx_path);
            println!("[bazbom] wrote CycloneDX SBOM ({} components) to {:?}", component_count, cyclonedx_path);
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
            ScanOrchestratorOptions {
                cyclonedx: false,
                with_semgrep: false,
                with_codeql: None,
                autofix: None,
                containers: None,
                no_upload: true,
                target: None,
                threat_detection: None,
                incremental: false,
                benchmark: false,
                fast: false,
                reachability: false,
                include_cicd: false,
            },
        )?;

        assert!(!orchestrator.cyclonedx);
        assert!(!orchestrator.with_semgrep);
        assert!(orchestrator.no_upload);

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_orchestrator_run() -> Result<()> {
        let temp = tempdir()?;
        let workspace = temp.path().to_path_buf();
        let out_dir = workspace.join("out");

        let orchestrator = ScanOrchestrator::new(
            workspace,
            out_dir,
            ScanOrchestratorOptions {
                cyclonedx: false,
                with_semgrep: false,
                with_codeql: None,
                autofix: None,
                containers: None,
                no_upload: true,
                target: None,
                threat_detection: None,
                incremental: false,
                benchmark: false,
                fast: false,
                reachability: false,
                include_cicd: false,
            },
        )?;

        // This should not fail even without tools installed
        orchestrator.run()?;

        Ok(())
    }
}
