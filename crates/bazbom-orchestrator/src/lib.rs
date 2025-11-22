//! Parallel Orchestration for Multi-Ecosystem Scanning
//!
//! This crate provides a high-performance orchestrator that scans multiple
//! programming language ecosystems in parallel using tokio.
//!
//! # Features
//!
//! - **Parallel Scanning** - Scan multiple ecosystems concurrently
//! - **Progress Tracking** - Real-time progress indicators
//! - **Error Resilience** - Continue scanning even if one ecosystem fails
//! - **Result Aggregation** - Combine results from all scanners
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │   ParallelOrchestrator              │
//! └─────────┬───────────────────────────┘
//!           │
//!           ├──> Detector (finds ecosystems)
//!           │
//!           ├──> Parallel Scanner Pool
//!           │    ├─> npm Scanner
//!           │    ├─> Python Scanner
//!           │    ├─> Go Scanner
//!           │    └─> ... (other scanners)
//!           │
//!           └──> Result Aggregator
//! ```

use anyhow::Result;
use bazbom_scanner::{
    detect_ecosystems, ecosystems, Ecosystem, EcosystemScanResult, EcosystemType, LicenseCache,
    ScanContext, Scanner,
};
use futures::stream::{self, StreamExt};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::task;
use tracing::{error, info, warn};

/// Configuration for parallel orchestration
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Maximum number of concurrent ecosystem scans
    pub max_concurrent: usize,

    /// Enable progress bars
    pub show_progress: bool,

    /// Enable reachability analysis
    pub enable_reachability: bool,

    /// Enable vulnerability scanning
    pub enable_vulnerabilities: bool,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: num_cpus::get(),
            show_progress: true,
            enable_reachability: true,
            enable_vulnerabilities: true,
        }
    }
}

/// Parallel orchestrator for multi-ecosystem scanning
pub struct ParallelOrchestrator {
    config: OrchestratorConfig,
    license_cache: Arc<LicenseCache>,
}

impl ParallelOrchestrator {
    /// Create a new parallel orchestrator with default configuration
    pub fn new() -> Self {
        Self {
            config: OrchestratorConfig::default(),
            license_cache: Arc::new(LicenseCache::new()),
        }
    }

    /// Create a new parallel orchestrator with custom configuration
    pub fn with_config(config: OrchestratorConfig) -> Self {
        Self {
            config,
            license_cache: Arc::new(LicenseCache::new()),
        }
    }

    /// Scan a directory for all ecosystems in parallel
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bazbom_orchestrator::ParallelOrchestrator;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let orchestrator = ParallelOrchestrator::new();
    ///     let results = orchestrator.scan_directory(".").await?;
    ///
    ///     println!("Scanned {} ecosystems", results.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn scan_directory(&self, path: impl AsRef<Path>) -> Result<Vec<EcosystemScanResult>> {
        let path = path.as_ref();
        let start_time = Instant::now();

        info!("Starting parallel scan of: {}", path.display());

        // Detect all ecosystems
        let ecosystems = detect_ecosystems(path)?;

        if ecosystems.is_empty() {
            info!("No ecosystems detected in {}", path.display());
            return Ok(Vec::new());
        }

        info!("Detected {} ecosystems to scan", ecosystems.len());

        // Set up progress tracking
        let multi_progress = if self.config.show_progress {
            Some(MultiProgress::new())
        } else {
            None
        };

        // Scan ecosystems in parallel
        let results = self
            .scan_ecosystems_parallel(ecosystems, &multi_progress)
            .await;

        let elapsed = start_time.elapsed();
        info!(
            "Parallel scan completed in {:.2}s - {} ecosystems scanned",
            elapsed.as_secs_f64(),
            results.len()
        );

        Ok(results)
    }

    /// Scan multiple ecosystems in parallel with progress tracking
    async fn scan_ecosystems_parallel(
        &self,
        ecosystems: Vec<Ecosystem>,
        multi_progress: &Option<MultiProgress>,
    ) -> Vec<EcosystemScanResult> {
        let license_cache = self.license_cache.clone();
        let config = self.config.clone();

        // Create progress bars for each ecosystem
        let progress_bars: Vec<Option<ProgressBar>> = if let Some(ref mp) = multi_progress {
            ecosystems
                .iter()
                .map(|eco| {
                    let pb = mp.add(ProgressBar::new(100));
                    pb.set_style(
                        ProgressStyle::default_bar()
                            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
                            .expect("Failed to create progress bar template")
                            .progress_chars("#>-"),
                    );
                    pb.set_message(format!("Scanning {}", eco.name));
                    Some(pb)
                })
                .collect()
        } else {
            vec![None; ecosystems.len()]
        };

        // Scan ecosystems concurrently
        let results: Vec<Result<EcosystemScanResult>> = stream::iter(
            ecosystems
                .into_iter()
                .zip(progress_bars.into_iter())
                .enumerate(),
        )
        .map(|(_idx, (ecosystem, progress_bar))| {
            let license_cache = license_cache.clone();
            let config = config.clone();

            task::spawn(async move {
                let result = scan_single_ecosystem(&ecosystem, license_cache, &config).await;

                if let Some(pb) = progress_bar {
                    match &result {
                        Ok(scan_result) => {
                            pb.set_position(100);
                            pb.finish_with_message(format!(
                                "✓ {} - {} packages",
                                ecosystem.name, scan_result.total_packages
                            ));
                        }
                        Err(e) => {
                            pb.finish_with_message(format!("✗ {} - {}", ecosystem.name, e));
                        }
                    }
                }

                result
            })
        })
        .buffer_unordered(self.config.max_concurrent)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .map(|join_result| {
            join_result.unwrap_or_else(|e| Err(anyhow::anyhow!("Task panicked: {}", e)))
        })
        .collect();

        // Filter out errors and log them
        results
            .into_iter()
            .enumerate()
            .filter_map(|(idx, result)| match result {
                Ok(scan_result) => Some(scan_result),
                Err(e) => {
                    error!("Failed to scan ecosystem {}: {}", idx, e);
                    None
                }
            })
            .collect()
    }
}

impl Default for ParallelOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Scan a single ecosystem (helper function)
async fn scan_single_ecosystem(
    ecosystem: &Ecosystem,
    license_cache: Arc<LicenseCache>,
    config: &OrchestratorConfig,
) -> Result<EcosystemScanResult> {
    info!("Starting scan of {} ecosystem", ecosystem.name);

    let mut ctx = ScanContext::new(ecosystem.root_path.clone(), license_cache);

    if let Some(ref manifest) = ecosystem.manifest_file {
        ctx = ctx.with_manifest(manifest.clone());
    }
    if let Some(ref lockfile) = ecosystem.lockfile {
        ctx = ctx.with_lockfile(lockfile.clone());
    }

    // Dispatch to appropriate scanner based on ecosystem type
    let mut result = match ecosystem.ecosystem_type {
        EcosystemType::Npm => {
            let scanner = ecosystems::npm::NpmScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Python => {
            let scanner = ecosystems::python::PythonScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Go => {
            let scanner = ecosystems::go::GoScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Rust => {
            let scanner = ecosystems::rust::RustScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Ruby => {
            let scanner = ecosystems::ruby::RubyScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Php => {
            let scanner = ecosystems::php::PhpScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Maven => {
            let scanner = ecosystems::maven::MavenScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Gradle => {
            let scanner = ecosystems::gradle::GradleScanner::new();
            scanner.scan(&ctx).await?
        }
        EcosystemType::Bazel => {
            let scanner = ecosystems::bazel::BazelScanner::new();
            scanner.scan(&ctx).await?
        }
    };

    // Optionally scan for vulnerabilities
    if config.enable_vulnerabilities && !result.packages.is_empty() {
        match bazbom_scanner::vulnerabilities::scan_vulnerabilities(&result.packages).await {
            Ok(vulns) => {
                result.vulnerabilities = vulns;
                result.total_vulnerabilities = result.vulnerabilities.len();
                info!(
                    "Found {} vulnerabilities in {}",
                    result.total_vulnerabilities, ecosystem.name
                );
            }
            Err(e) => {
                warn!(
                    "Failed to scan vulnerabilities for {}: {}",
                    ecosystem.name, e
                );
            }
        }
    }

    // Optionally perform reachability analysis
    if config.enable_reachability && !result.vulnerabilities.is_empty() {
        if let Err(e) =
            bazbom_scanner::analyze_reachability(&mut result, ecosystem.root_path.as_path()).await
        {
            warn!(
                "Failed to analyze reachability for {}: {}",
                ecosystem.name, e
            );
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_orchestrator_empty_directory() {
        let temp = TempDir::new().unwrap();
        let orchestrator = ParallelOrchestrator::new();

        let results = orchestrator.scan_directory(temp.path()).await.unwrap();

        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_orchestrator_config() {
        let config = OrchestratorConfig {
            max_concurrent: 4,
            show_progress: false,
            enable_reachability: false,
            enable_vulnerabilities: false,
        };

        let orchestrator = ParallelOrchestrator::with_config(config);
        assert_eq!(orchestrator.config.max_concurrent, 4);
        assert!(!orchestrator.config.show_progress);
    }
}
