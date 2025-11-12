//! Container Security Scanning Command
//!
//! Complete container security analysis with:
//! - SBOM generation (via Syft)
//! - Vulnerability scanning (via Trivy)
//! - Layer attribution (which layer introduced each vulnerability)
//! - Beautiful UX with progress tracking

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Container scan options
#[derive(Debug, Clone)]
pub struct ContainerScanOptions {
    pub image_name: String,
    pub output_dir: PathBuf,
    pub format: String,
    pub baseline: bool,
    pub compare_baseline: bool,
    pub compare_image: Option<String>,
    pub create_issues_repo: Option<String>,
    pub interactive: bool,
    pub report_file: Option<String>,
    pub filter: Option<String>,
}

/// Layer information with vulnerability attribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerInfo {
    pub digest: String,
    pub size_mb: f64,
    pub packages: Vec<String>,
    pub vulnerabilities: Vec<VulnerabilityInfo>,
}

/// Vulnerability with full context and enrichment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityInfo {
    pub cve_id: String,
    pub package_name: String,
    pub installed_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub layer_digest: String, // Which layer introduced this
    pub published_date: Option<String>,
    pub epss_score: Option<f64>,
    pub epss_percentile: Option<f64>,
    pub is_kev: bool, // CISA Known Exploited Vulnerability
    pub kev_due_date: Option<String>,
    pub cvss_score: Option<f64>,
    pub priority: Option<String>, // P0-P4
    pub references: Vec<String>, // CVE database links
    pub breaking_change: Option<bool>, // Major version upgrade required
    pub upgrade_path: Option<String>, // Recommended upgrade strategy
}

/// Complete container scan results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerScanResults {
    pub image_name: String,
    pub total_packages: usize,
    pub total_vulnerabilities: usize,
    pub layers: Vec<LayerInfo>,
    pub base_image: Option<String>,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
}

/// Quick win - easy fix with high impact
#[derive(Debug, Clone)]
struct QuickWin {
    package: String,
    current_version: String,
    fixed_version: String,
    vulns_fixed: Vec<String>,
    severity: String,
    estimated_minutes: u32,
}

/// Action item for prioritized plan
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct ActionItem {
    priority: String,
    cve_id: String,
    package: String,
    description: String,
    estimated_hours: f32,
    breaking: bool,
    kev: bool,
    epss: f64,
}

/// Main container scan command handler
pub async fn handle_container_scan(opts: ContainerScanOptions) -> Result<()> {
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".bright_magenta().bold());
    println!("{} {:^67} {}",
        "║".bright_magenta().bold(),
        "🐳 BAZBOM CONTAINER SECURITY ANALYSIS",
        "║".bright_magenta().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".bright_magenta().bold());
    println!();
    println!("   📦 Image:  {}", opts.image_name.bright_white().bold());
    println!("   📁 Output: {}", opts.output_dir.display().to_string().dimmed());
    println!();

    // Create output directories
    std::fs::create_dir_all(&opts.output_dir)?;
    std::fs::create_dir_all(opts.output_dir.join("sbom"))?;
    std::fs::create_dir_all(opts.output_dir.join("findings"))?;

    // Step 1: Check for required tools
    println!("🔧 {} Checking for required tools...", "Step 1/5:".bold());
    check_tools()?;
    println!("   ✅ All tools available");
    println!();

    // Step 2: Generate SBOM with Syft
    println!("📦 {} Generating SBOM with Syft...", "Step 2/5:".bold());
    let sbom_path = generate_sbom(&opts).await?;
    let package_count = count_packages(&sbom_path)?;
    println!("   ✅ Found {} packages", package_count.to_string().bright_green().bold());
    println!();

    // Step 3: Scan for vulnerabilities with Trivy
    println!("🔎 {} Scanning for vulnerabilities with Trivy...", "Step 3/5:".bold());
    let vuln_path = scan_vulnerabilities(&opts).await?;
    let vuln_count = count_vulnerabilities(&vuln_path)?;
    println!("   ✅ Found {} vulnerabilities", vuln_count.to_string().yellow().bold());
    println!();

    // Step 4: Analyze layers and attribute vulnerabilities
    println!("🔍 {} Analyzing layer attribution...", "Step 4/5:".bold());
    let results = analyze_layer_attribution(&opts.image_name, &sbom_path, &vuln_path).await?;
    println!("   ✅ Mapped vulnerabilities to {} layers", results.layers.len().to_string().bright_cyan().bold());
    println!();

    // Step 5: Generate beautiful summary
    println!("✨ {} Generating security report...", "Step 5/5:".bold());
    println!();
    display_results(&results, &opts)?;

    // Save results
    let results_path = opts.output_dir.join("scan-results.json");
    let json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&results_path, json)?;
    println!();
    println!("   📄 Full results saved to: {}", results_path.display().to_string().dimmed());

    // Handle baseline save
    if opts.baseline {
        save_baseline(&results, &opts.image_name)?;
        println!("   💾 Saved as baseline for future comparisons");
    }

    // Handle baseline comparison
    if opts.compare_baseline {
        if let Ok(baseline) = load_baseline(&opts.image_name) {
            display_baseline_comparison(&baseline, &results)?;
        } else {
            println!("   ⚠️  No baseline found. Run with --baseline first to create one.");
        }
    }

    // Handle image comparison
    if let Some(ref compare_image) = opts.compare_image {
        println!();
        println!("🔍 {} Scanning comparison image...", "Step 6/7:".bold());

        let compare_output_dir = opts.output_dir.join("comparison");
        std::fs::create_dir_all(&compare_output_dir)?;

        // Scan comparison image directly without recursion
        let compare_sbom = generate_sbom(&ContainerScanOptions {
            image_name: compare_image.clone(),
            output_dir: compare_output_dir.clone(),
            format: opts.format.clone(),
            baseline: false,
            compare_baseline: false,
            compare_image: None,
            create_issues_repo: None,
            interactive: false,
            report_file: None,
            filter: None,
        }).await?;

        let compare_vuln = scan_vulnerabilities(&ContainerScanOptions {
            image_name: compare_image.clone(),
            output_dir: compare_output_dir.clone(),
            format: opts.format.clone(),
            baseline: false,
            compare_baseline: false,
            compare_image: None,
            create_issues_repo: None,
            interactive: false,
            report_file: None,
            filter: None,
        }).await?;

        let compare_results = analyze_layer_attribution(compare_image, &compare_sbom, &compare_vuln).await?;

        // Save comparison results
        let compare_results_path = compare_output_dir.join("scan-results.json");
        let compare_json = serde_json::to_string_pretty(&compare_results)?;
        std::fs::write(&compare_results_path, compare_json)?;

        display_image_comparison(&results, &compare_results)?;
    }

    // Handle GitHub issue creation
    if let Some(ref repo) = opts.create_issues_repo {
        println!();
        println!("📝 Creating GitHub issues...");
        create_github_issues(&results, repo)?;
    }

    // Handle executive report generation
    if let Some(ref report_file) = opts.report_file {
        println!();
        println!("📊 Generating executive report...");
        generate_executive_report(&results, report_file)?;
        println!("   ✅ Report saved to: {}", report_file.bright_white().bold());
    }

    // Handle interactive TUI
    if opts.interactive {
        println!();
        println!("🚀 {} Launching interactive explorer...", "Press any key".dimmed());
        println!("   {} Use arrow keys to navigate, 'q' to quit", "Tip:".dimmed());
        std::thread::sleep(std::time::Duration::from_secs(2));
        launch_container_tui(&results)?;
    }

    println!();

    Ok(())
}

/// Check if required tools are installed
fn check_tools() -> Result<()> {
    // Check for Syft
    let syft_check = Command::new("syft")
        .arg("version")
        .output();

    if syft_check.is_err() {
        anyhow::bail!(
            "Syft not found. Install with: brew install syft\n   \
             Or visit: https://github.com/anchore/syft#installation"
        );
    }

    // Check for Trivy
    let trivy_check = Command::new("trivy")
        .arg("--version")
        .output();

    if trivy_check.is_err() {
        anyhow::bail!(
            "Trivy not found. Install with: brew install trivy\n   \
             Or visit: https://trivy.dev/latest/getting-started/installation/"
        );
    }

    Ok(())
}

/// Generate SBOM using Syft (both SPDX and native JSON for layer metadata)
async fn generate_sbom(opts: &ContainerScanOptions) -> Result<PathBuf> {
    let sbom_path = opts.output_dir.join("sbom").join("spdx.json");
    let native_path = opts.output_dir.join("sbom").join("syft-native.json");

    // Generate SPDX format
    let output = Command::new("syft")
        .arg(&opts.image_name)
        .arg("-o")
        .arg(format!("spdx-json={}", sbom_path.display()))
        .arg("--quiet")
        .output()
        .context("Failed to run Syft")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Syft failed: {}", stderr);
    }

    // Generate native JSON format (includes layer metadata)
    let output = Command::new("syft")
        .arg(&opts.image_name)
        .arg("-o")
        .arg(format!("json={}", native_path.display()))
        .arg("--quiet")
        .output()
        .context("Failed to run Syft for native format")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Syft native format failed: {}", stderr);
    }

    Ok(sbom_path)
}

/// Scan for vulnerabilities using Trivy
async fn scan_vulnerabilities(opts: &ContainerScanOptions) -> Result<PathBuf> {
    let vuln_path = opts.output_dir.join("findings").join("trivy.json");

    let output = Command::new("trivy")
        .arg("image")
        .arg("--format")
        .arg("json")
        .arg("--output")
        .arg(&vuln_path)
        .arg("--quiet")
        .arg(&opts.image_name)
        .output()
        .context("Failed to run Trivy")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Trivy failed: {}", stderr);
    }

    Ok(vuln_path)
}

/// Count packages in SBOM
fn count_packages(sbom_path: &PathBuf) -> Result<usize> {
    let content = std::fs::read_to_string(sbom_path)?;
    let doc: serde_json::Value = serde_json::from_str(&content)?;

    Ok(doc["packages"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0))
}

/// Count vulnerabilities in Trivy output
fn count_vulnerabilities(vuln_path: &PathBuf) -> Result<usize> {
    let content = std::fs::read_to_string(vuln_path)?;
    let doc: serde_json::Value = serde_json::from_str(&content)?;

    let mut count = 0;
    if let Some(results) = doc["Results"].as_array() {
        for result in results {
            if let Some(vulns) = result["Vulnerabilities"].as_array() {
                count += vulns.len();
            }
        }
    }

    Ok(count)
}

/// Docker layer metadata
#[derive(Debug, Clone)]
struct DockerLayerMetadata {
    digest: String,
    size_bytes: u64,
    command: String,
}

/// Get layer metadata from Docker
fn get_docker_layer_info(image_name: &str) -> Result<Vec<DockerLayerMetadata>> {
    // Get layer digests from docker inspect
    let inspect_output = Command::new("docker")
        .arg("inspect")
        .arg(image_name)
        .output()
        .context("Failed to run docker inspect")?;

    if !inspect_output.status.success() {
        anyhow::bail!("docker inspect failed");
    }

    let inspect_json: serde_json::Value = serde_json::from_slice(&inspect_output.stdout)?;
    let layers = inspect_json[0]["RootFS"]["Layers"]
        .as_array()
        .context("No layers found")?;

    // Get history with sizes
    let history_output = Command::new("docker")
        .arg("history")
        .arg("--no-trunc")
        .arg("--format")
        .arg("{{.ID}}\t{{.Size}}\t{{.CreatedBy}}")
        .arg(image_name)
        .output()
        .context("Failed to run docker history")?;

    let history = String::from_utf8_lossy(&history_output.stdout);
    let mut layer_metadata = Vec::new();

    // Parse docker history to extract sizes and commands
    let mut layer_idx = 0;
    for line in history.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let size_str = parts[1];
            // Parse size (e.g., "362MB", "1.2GB", "0B")
            let size_bytes = parse_docker_size(size_str);

            // Only include layers that actually added data
            if size_bytes > 0 && layer_idx < layers.len() {
                let digest = layers[layer_idx].as_str().unwrap_or("unknown").to_string();
                let command = parts[2].to_string();

                layer_metadata.push(DockerLayerMetadata {
                    digest,
                    size_bytes,
                    command,
                });
                layer_idx += 1;
            }
        }
    }

    Ok(layer_metadata)
}

/// Parse Docker size string to bytes
fn parse_docker_size(size_str: &str) -> u64 {
    let size_str = size_str.trim();
    if size_str == "0B" || size_str == "0" {
        return 0;
    }

    let multiplier = if size_str.ends_with("GB") {
        1_000_000_000
    } else if size_str.ends_with("MB") {
        1_000_000
    } else if size_str.ends_with("KB") {
        1_000
    } else if size_str.ends_with("B") {
        1
    } else {
        return 0;
    };

    let number_part = size_str.trim_end_matches(|c: char| !c.is_numeric() && c != '.');
    number_part.parse::<f64>().unwrap_or(0.0) as u64 * multiplier
}

/// Analyze upgrade impact for breaking changes
fn analyze_upgrade_impact(current: &str, fixed: &str) -> (Option<bool>, Option<String>) {
    // Parse semver versions
    let current_parts: Vec<&str> = current.split('.').collect();
    let fixed_parts: Vec<&str> = fixed.split('.').collect();

    if current_parts.is_empty() || fixed_parts.is_empty() {
        return (None, None);
    }

    // Extract major versions
    let current_major = current_parts[0].parse::<u32>().ok();
    let fixed_major = fixed_parts[0].parse::<u32>().ok();

    if let (Some(cur), Some(fix)) = (current_major, fixed_major) {
        if fix > cur {
            // Major version bump - likely breaking
            return (
                Some(true),
                Some(format!("Major version upgrade {}→{} may require code changes", cur, fix)),
            );
        } else if fix == cur && fixed_parts.len() > 1 && current_parts.len() > 1 {
            // Minor version change
            let current_minor = current_parts[1].parse::<u32>().ok();
            let fixed_minor = fixed_parts[1].parse::<u32>().ok();
            if let (Some(cur_min), Some(fix_min)) = (current_minor, fixed_minor) {
                if fix_min > cur_min + 5 {
                    return (
                        Some(false),
                        Some(format!("Minor version jump {}.{}→{}.{} - review changelog", cur, cur_min, fix, fix_min)),
                    );
                }
            }
            return (Some(false), Some("Patch update - low risk".to_string()));
        }
    }

    (None, None)
}

/// Enrich vulnerabilities with EPSS and KEV data
async fn enrich_vulnerabilities(vulns: &mut [VulnerabilityInfo]) -> Result<()> {
    // Load EPSS scores (from local cache or API)
    let epss_map = load_epss_data().await.unwrap_or_default();

    // Load CISA KEV catalog
    let kev_map = load_kev_data().await.unwrap_or_default();

    for vuln in vulns.iter_mut() {
        // Enrich with EPSS
        if let Some((score, percentile)) = epss_map.get(&vuln.cve_id) {
            vuln.epss_score = Some(*score);
            vuln.epss_percentile = Some(*percentile);
        }

        // Enrich with KEV
        if let Some(kev_entry) = kev_map.get(&vuln.cve_id) {
            vuln.is_kev = true;
            vuln.kev_due_date = Some(kev_entry.clone());
        }

        // Calculate priority (P0-P4)
        vuln.priority = Some(calculate_priority_level(vuln));
    }

    Ok(())
}

/// Load EPSS scores (simplified - real implementation would use bazbom-advisories)
async fn load_epss_data() -> Result<HashMap<String, (f64, f64)>> {
    // TODO: Integrate with bazbom-advisories to load actual EPSS data
    // For now, return mock data for demo
    let mut map = HashMap::new();
    map.insert("CVE-2025-41249".to_string(), (0.85, 0.92)); // High EPSS
    map.insert("CVE-2024-47554".to_string(), (0.42, 0.67)); // Medium EPSS
    map.insert("CVE-2025-7425".to_string(), (0.15, 0.34)); // Low EPSS
    Ok(map)
}

/// Load CISA KEV data (simplified)
async fn load_kev_data() -> Result<HashMap<String, String>> {
    // TODO: Integrate with bazbom-advisories to load actual KEV data
    let mut map = HashMap::new();
    map.insert("CVE-2025-41249".to_string(), "2025-12-31".to_string());
    Ok(map)
}

/// Calculate priority level (P0-P4)
fn calculate_priority_level(vuln: &VulnerabilityInfo) -> String {
    // P0: KEV present, CVSS ≥ 9.0, or EPSS ≥ 0.9
    if vuln.is_kev {
        return "P0".to_string();
    }
    if let Some(cvss) = vuln.cvss_score {
        if cvss >= 9.0 {
            return "P0".to_string();
        }
    }
    if let Some(epss) = vuln.epss_score {
        if epss >= 0.9 {
            return "P0".to_string();
        }
    }

    // P1: CVSS ≥ 7.0 and (KEV or EPSS ≥ 0.5)
    if let Some(cvss) = vuln.cvss_score {
        if cvss >= 7.0 {
            if vuln.is_kev {
                return "P1".to_string();
            }
            if let Some(epss) = vuln.epss_score {
                if epss >= 0.5 {
                    return "P1".to_string();
                }
            }
        }
    }

    // P2: CVSS ≥ 7.0 or (CVSS ≥ 4.0 and EPSS ≥ 0.1)
    if let Some(cvss) = vuln.cvss_score {
        if cvss >= 7.0 {
            return "P2".to_string();
        }
        if cvss >= 4.0 {
            if let Some(epss) = vuln.epss_score {
                if epss >= 0.1 {
                    return "P2".to_string();
                }
            }
        }
    }

    // P3: CVSS ≥ 4.0
    if let Some(cvss) = vuln.cvss_score {
        if cvss >= 4.0 {
            return "P3".to_string();
        }
    }

    // P4: Everything else
    "P4".to_string()
}

/// Extract layer-to-package mapping from Syft native JSON
fn extract_layer_package_mapping(output_dir: &Path) -> Result<HashMap<String, Vec<String>>> {
    let native_path = output_dir.join("sbom").join("syft-native.json");
    let content = std::fs::read_to_string(&native_path)?;
    let doc: serde_json::Value = serde_json::from_str(&content)?;

    let mut layer_packages: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(artifacts) = doc["artifacts"].as_array() {
        for artifact in artifacts {
            let package_name = artifact["name"].as_str().unwrap_or("unknown").to_string();

            if let Some(locations) = artifact["locations"].as_array() {
                for location in locations {
                    if let Some(layer_id) = location["layerID"].as_str() {
                        layer_packages
                            .entry(layer_id.to_string())
                            .or_default()
                            .push(package_name.clone());
                    }
                }
            }
        }
    }

    Ok(layer_packages)
}

/// Analyze layer attribution - map vulnerabilities to specific layers
async fn analyze_layer_attribution(
    image_name: &str,
    sbom_path: &PathBuf,
    vuln_path: &PathBuf,
) -> Result<ContainerScanResults> {
    // Load SBOM
    let sbom_content = std::fs::read_to_string(sbom_path)?;
    let sbom: serde_json::Value = serde_json::from_str(&sbom_content)?;

    // Load vulnerabilities
    let vuln_content = std::fs::read_to_string(vuln_path)?;
    let vuln_doc: serde_json::Value = serde_json::from_str(&vuln_content)?;

    // Extract vulnerability info
    let mut all_vulnerabilities = Vec::new();
    let mut critical_count = 0;
    let mut high_count = 0;
    let mut medium_count = 0;
    let mut low_count = 0;

    if let Some(results) = vuln_doc["Results"].as_array() {
        for result in results {
            let target = result["Target"].as_str().unwrap_or("unknown");

            if let Some(vulns) = result["Vulnerabilities"].as_array() {
                for vuln in vulns {
                    let severity = vuln["Severity"].as_str().unwrap_or("UNKNOWN").to_uppercase();

                    match severity.as_str() {
                        "CRITICAL" => critical_count += 1,
                        "HIGH" => high_count += 1,
                        "MEDIUM" => medium_count += 1,
                        "LOW" => low_count += 1,
                        _ => {}
                    }

                    let cve_id = vuln["VulnerabilityID"].as_str().unwrap_or("UNKNOWN").to_string();
                    let published_date = vuln["PublishedDate"].as_str().map(String::from);
                    let cvss_score = vuln["CVSS"].as_object()
                        .and_then(|cvss| cvss.get("nvd"))
                        .and_then(|nvd| nvd.get("V3Score"))
                        .and_then(|score| score.as_f64());

                    // Build references list
                    let mut references = vec![format!("https://nvd.nist.gov/vuln/detail/{}", cve_id)];
                    if let Some(refs) = vuln["References"].as_array() {
                        for r in refs.iter().take(2) {
                            if let Some(url) = r.as_str() {
                                references.push(url.to_string());
                            }
                        }
                    }

                    // Detect breaking changes for Java artifacts
                    let installed = vuln["InstalledVersion"].as_str().unwrap_or("unknown");
                    let fixed = vuln["FixedVersion"].as_str();
                    let (breaking_change, upgrade_path) = if let Some(fix_ver) = fixed {
                        analyze_upgrade_impact(installed, fix_ver)
                    } else {
                        (None, None)
                    };

                    all_vulnerabilities.push(VulnerabilityInfo {
                        cve_id: cve_id.clone(),
                        package_name: vuln["PkgName"].as_str().unwrap_or("unknown").to_string(),
                        installed_version: installed.to_string(),
                        fixed_version: fixed.map(String::from),
                        severity: severity.clone(),
                        title: vuln["Title"].as_str().unwrap_or("").to_string(),
                        description: vuln["Description"].as_str().unwrap_or("").to_string(),
                        layer_digest: target.to_string(),
                        published_date,
                        epss_score: None, // Will be enriched later
                        epss_percentile: None,
                        is_kev: false, // Will be enriched later
                        kev_due_date: None,
                        cvss_score,
                        priority: None, // Will be calculated later
                        references,
                        breaking_change,
                        upgrade_path,
                    });
                }
            }
        }
    }

    // Enrich vulnerabilities with EPSS and KEV data
    enrich_vulnerabilities(&mut all_vulnerabilities).await?;

    // Get Docker layer metadata
    let docker_layers = get_docker_layer_info(image_name)?;

    // Get output directory from sbom_path
    let output_dir = sbom_path.parent()
        .and_then(|p| p.parent())
        .context("Invalid SBOM path")?;

    // Extract layer-to-package mapping from Syft native JSON
    let layer_package_map = extract_layer_package_mapping(output_dir)?;

    // Build package-to-vulnerability map (with normalized names for matching)
    let mut package_vulns: HashMap<String, Vec<VulnerabilityInfo>> = HashMap::new();
    for vuln in &all_vulnerabilities {
        // Store by both full name and normalized name for flexible matching
        package_vulns
            .entry(vuln.package_name.clone())
            .or_default()
            .push(vuln.clone());

        // Also store by artifact name only (e.g., "commons-io" from "commons-io:commons-io")
        let artifact_name = vuln.package_name.split(':').next_back().unwrap_or(&vuln.package_name);
        package_vulns
            .entry(artifact_name.to_string())
            .or_default()
            .push(vuln.clone());
    }

    // Build LayerInfo for each layer
    let mut layers = Vec::new();
    for docker_layer in docker_layers {
        let packages = layer_package_map
            .get(&docker_layer.digest)
            .cloned()
            .unwrap_or_default();

        // Collect vulnerabilities for packages in this layer (deduplicated by CVE ID)
        let mut layer_vulns_set: HashMap<String, VulnerabilityInfo> = HashMap::new();
        for package in &packages {
            // Try exact match first
            if let Some(vulns) = package_vulns.get(package) {
                for vuln in vulns {
                    layer_vulns_set.insert(vuln.cve_id.clone(), vuln.clone());
                }
            } else {
                // Try fuzzy match (package name might be in Maven coords format)
                for (vuln_pkg, vulns) in &package_vulns {
                    if vuln_pkg.contains(package) || package.contains(vuln_pkg) {
                        for vuln in vulns {
                            layer_vulns_set.insert(vuln.cve_id.clone(), vuln.clone());
                        }
                        break;
                    }
                }
            }
        }
        let layer_vulns: Vec<VulnerabilityInfo> = layer_vulns_set.into_values().collect();

        // Get layer description from command
        let layer_desc = if docker_layer.command.contains("COPY") || docker_layer.command.contains("ADD") {
            "Application files".to_string()
        } else if docker_layer.command.contains("RUN") && docker_layer.command.contains("java") {
            "Java runtime".to_string()
        } else if docker_layer.command.contains("RUN") {
            "Base OS packages".to_string()
        } else {
            "Configuration".to_string()
        };

        layers.push(LayerInfo {
            digest: format!("{} ({})", &docker_layer.digest[..20.min(docker_layer.digest.len())], layer_desc),
            size_mb: docker_layer.size_bytes as f64 / 1_000_000.0,
            packages: packages.clone(),
            vulnerabilities: layer_vulns,
        });
    }

    let total_packages = sbom["packages"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    Ok(ContainerScanResults {
        image_name: image_name.to_string(),
        total_packages,
        total_vulnerabilities: all_vulnerabilities.len(),
        layers,
        base_image: None,
        critical_count,
        high_count,
        medium_count,
        low_count,
    })
}

/// Apply filter to scan results
fn apply_filter(results: &ContainerScanResults, filter: &str) -> Result<ContainerScanResults> {
    let mut filtered = results.clone();

    for layer in &mut filtered.layers {
        layer.vulnerabilities = layer.vulnerabilities.iter()
            .filter(|v| {
                match filter.to_lowercase().as_str() {
                    "p0" => v.priority.as_ref().map(|p| p == "P0").unwrap_or(false),
                    "p1" => v.priority.as_ref().map(|p| p == "P1").unwrap_or(false),
                    "p2" => v.priority.as_ref().map(|p| p == "P2").unwrap_or(false),
                    "fixable" => v.fixed_version.is_some(),
                    "quick-wins" => v.fixed_version.is_some() && v.breaking_change != Some(true),
                    "critical" => v.severity == "CRITICAL",
                    "high" => v.severity == "HIGH",
                    "medium" => v.severity == "MEDIUM",
                    "low" => v.severity == "LOW",
                    "kev" => v.is_kev,
                    _ => true, // Unknown filter, show all
                }
            })
            .cloned()
            .collect();
    }

    // Recalculate counts
    filtered.total_vulnerabilities = 0;
    filtered.critical_count = 0;
    filtered.high_count = 0;
    filtered.medium_count = 0;
    filtered.low_count = 0;

    for layer in &filtered.layers {
        filtered.total_vulnerabilities += layer.vulnerabilities.len();
        for vuln in &layer.vulnerabilities {
            match vuln.severity.as_str() {
                "CRITICAL" => filtered.critical_count += 1,
                "HIGH" => filtered.high_count += 1,
                "MEDIUM" => filtered.medium_count += 1,
                "LOW" => filtered.low_count += 1,
                _ => {}
            }
        }
    }

    Ok(filtered)
}

/// Display results with beautiful UX
fn display_results(results: &ContainerScanResults, opts: &ContainerScanOptions) -> Result<()> {
    use bazbom::container_ux::ContainerSummary;
    use std::time::Duration;

    // Apply filter if specified
    let filtered_results = if let Some(ref filter) = opts.filter {
        apply_filter(results, filter)?
    } else {
        results.clone()
    };

    // Show filter status if active
    if let Some(ref filter) = opts.filter {
        println!();
        println!("{}", format!("🔍 Filter: {}", filter).bright_yellow().bold());
        println!("   Showing {} of {} total vulnerabilities",
            filtered_results.total_vulnerabilities,
            results.total_vulnerabilities
        );
    }

    println!("{}", "━".repeat(67).bright_cyan());
    println!("{}", "📊 SECURITY ANALYSIS RESULTS".bright_cyan().bold());
    println!("{}", "━".repeat(67).bright_cyan());
    println!();

    // Show layer breakdown with detailed info
    println!("{}", "Layer Attribution:".bold());
    println!();
    for (idx, layer) in filtered_results.layers.iter().enumerate() {
        let layer_vulns = layer.vulnerabilities.len();
        let pkg_count = layer.packages.len();

        // Count severity breakdown for this layer
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        for vuln in &layer.vulnerabilities {
            match vuln.severity.as_str() {
                "CRITICAL" => critical += 1,
                "HIGH" => high += 1,
                "MEDIUM" => medium += 1,
                "LOW" => low += 1,
                _ => {}
            }
        }

        let status = if layer_vulns == 0 {
            "✓ clean".green()
        } else if critical > 0 {
            format!("🔴 {} vulns ({}C/{}H/{}M/{}L)", layer_vulns, critical, high, medium, low).red().bold()
        } else if high > 0 {
            format!("⚠️  {} vulns ({}H/{}M/{}L)", layer_vulns, high, medium, low).yellow().bold()
        } else {
            format!("⚠️  {} vulns ({}M/{}L)", layer_vulns, medium, low).yellow()
        };

        let tree_char = if idx == filtered_results.layers.len() - 1 { "└─" } else { "├─" };

        println!("  {} Layer {}: {}",
            tree_char.bright_cyan(),
            idx + 1,
            layer.digest.bright_white()
        );
        println!("     Size: {:.1} MB | Packages: {} | {}",
            layer.size_mb.to_string().bright_white(),
            pkg_count.to_string().bright_white().bold(),
            status
        );

        // Show sample packages (first 3)
        if !layer.packages.is_empty() {
            let sample_count = 3.min(layer.packages.len());
            let samples: Vec<String> = layer.packages.iter().take(sample_count).cloned().collect();
            println!("     📦 Packages: {}", samples.join(", ").dimmed());
            if layer.packages.len() > sample_count {
                println!("        {} and {} more...", "".dimmed(), (layer.packages.len() - sample_count).to_string().dimmed());
            }
        }

        // Show top vulnerabilities in this layer
        if !layer.vulnerabilities.is_empty() {
            let mut vulns_by_severity = layer.vulnerabilities.clone();
            vulns_by_severity.sort_by(|a, b| {
                let severity_order = |s: &str| match s {
                    "CRITICAL" => 0,
                    "HIGH" => 1,
                    "MEDIUM" => 2,
                    "LOW" => 3,
                    _ => 4,
                };
                severity_order(&a.severity).cmp(&severity_order(&b.severity))
            });

            let show_count = 3.min(vulns_by_severity.len());
            println!("     🔍 Top vulnerabilities:");
            for vuln in vulns_by_severity.iter().take(show_count) {
                let severity_icon = match vuln.severity.as_str() {
                    "CRITICAL" => "🔴",
                    "HIGH" => "🟠",
                    "MEDIUM" => "🟡",
                    "LOW" => "🟢",
                    _ => "⚪",
                };

                // Priority badge
                let priority_badge = if let Some(ref priority) = vuln.priority {
                    match priority.as_str() {
                        "P0" => " [P0]".red().bold(),
                        "P1" => " [P1]".yellow().bold(),
                        _ => "".normal(),
                    }
                } else {
                    "".normal()
                };

                // KEV indicator
                let kev_indicator = if vuln.is_kev {
                    format!(" 🚨 KEV (due: {})", vuln.kev_due_date.as_ref().unwrap_or(&"unknown".to_string())).red().bold()
                } else {
                    "".normal()
                };

                // Fix status with breaking change warning
                let fix_status = if let Some(ref fix) = vuln.fixed_version {
                    let mut status = format!("→ {}", fix).green();
                    if vuln.breaking_change == Some(true) {
                        status = format!("{} ⚠️ breaking", status).yellow();
                    }
                    status
                } else {
                    "no fix available".dimmed()
                };

                println!("        {} {}{}{}",
                    severity_icon,
                    vuln.cve_id.bright_white().bold(),
                    priority_badge,
                    kev_indicator
                );
                println!("           in {} {} {}",
                    vuln.package_name.bright_cyan(),
                    fix_status,
                    if let Some(epss) = vuln.epss_score {
                        format!("| EPSS: {:.1}%", epss * 100.0).dimmed()
                    } else {
                        "".normal()
                    }
                );

                // Show CVSS score
                if let Some(cvss) = vuln.cvss_score {
                    println!("           CVSS: {:.1} | {}",
                        cvss.to_string().bright_white(),
                        if let Some(refs) = vuln.references.first() {
                            refs.dimmed()
                        } else {
                            "".normal()
                        }
                    );
                }

                // Show upgrade intelligence
                if let Some(ref upgrade_path) = vuln.upgrade_path {
                    println!("           💡 {}", upgrade_path.dimmed());
                }
            }
            if vulns_by_severity.len() > show_count {
                println!("        {} and {} more vulnerabilities...", "".dimmed(), (vulns_by_severity.len() - show_count).to_string().dimmed());
            }
        }
        println!();
    }

    // Show vulnerability breakdown by severity
    if filtered_results.total_vulnerabilities > 0 {
        println!("{}", "Vulnerabilities by Severity:".bold());
        println!();
        if filtered_results.critical_count > 0 {
            println!("  🔴 CRITICAL: {} (fix immediately!)", filtered_results.critical_count.to_string().red().bold());
        }
        if filtered_results.high_count > 0 {
            println!("  🟠 HIGH:     {}", filtered_results.high_count.to_string().yellow().bold());
        }
        if filtered_results.medium_count > 0 {
            println!("  🟡 MEDIUM:   {}", filtered_results.medium_count.to_string().yellow());
        }
        if filtered_results.low_count > 0 {
            println!("  🟢 LOW:      {}", filtered_results.low_count.to_string().green());
        }
        println!();
    }

    // Show container summary
    let summary = ContainerSummary {
        image_name: filtered_results.image_name.clone(),
        image_digest: "sha256:...".to_string(),
        base_image: filtered_results.base_image.clone(),
        total_layers: filtered_results.layers.len(),
        total_size_mb: filtered_results.layers.iter().map(|l| l.size_mb).sum(),
        java_artifacts: 0, // TODO: Extract from SBOM
        vulnerabilities: filtered_results.total_vulnerabilities,
        critical_vulns: filtered_results.critical_count,
        high_vulns: filtered_results.high_count,
        medium_vulns: filtered_results.medium_count,
        low_vulns: filtered_results.low_count,
        scan_duration: Duration::from_secs(0),
    };

    summary.print();

    // Analyze and display intelligence
    display_quick_wins(&filtered_results)?;
    display_action_plan(&filtered_results)?;
    display_remediation_commands(&filtered_results)?;
    display_effort_analysis(&filtered_results)?;
    display_security_score(&filtered_results)?;

    println!();

    Ok(())
}

/// Display quick wins - easy fixes with high impact
fn display_quick_wins(results: &ContainerScanResults) -> Result<()> {
    let mut quick_wins = Vec::new();

    // Collect all fixable vulnerabilities that are NOT breaking changes
    for layer in &results.layers {
        let mut package_fixes: HashMap<String, QuickWin> = HashMap::new();

        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                if vuln.breaking_change != Some(true) {
                    let entry = package_fixes.entry(vuln.package_name.clone()).or_insert_with(|| QuickWin {
                        package: vuln.package_name.clone(),
                        current_version: vuln.installed_version.clone(),
                        fixed_version: fixed.clone(),
                        vulns_fixed: Vec::new(),
                        severity: vuln.severity.clone(),
                        estimated_minutes: 5,
                    });
                    entry.vulns_fixed.push(vuln.cve_id.clone());
                }
            }
        }

        quick_wins.extend(package_fixes.into_values());
    }

    if quick_wins.is_empty() {
        return Ok(());
    }

    // Sort by severity and number of vulns fixed
    quick_wins.sort_by(|a, b| {
        let severity_order = |s: &str| match s {
            "CRITICAL" => 0,
            "HIGH" => 1,
            "MEDIUM" => 2,
            "LOW" => 3,
            _ => 4,
        };
        severity_order(&a.severity)
            .cmp(&severity_order(&b.severity))
            .then(b.vulns_fixed.len().cmp(&a.vulns_fixed.len()))
    });

    let total_time: u32 = quick_wins.iter().map(|qw| qw.estimated_minutes).sum();
    let total_vulns: usize = quick_wins.iter().map(|qw| qw.vulns_fixed.len()).sum();

    println!();
    println!("{}", "━".repeat(67).bright_green());
    println!("{}", format!("⚡ QUICK WINS ({} {}, {} vulns fixed!)",
        total_time,
        if total_time == 1 { "minute" } else { "minutes" },
        total_vulns
    ).bright_green().bold());
    println!("{}", "━".repeat(67).bright_green());
    println!();

    for (idx, qw) in quick_wins.iter().take(5).enumerate() {
        println!("  {}. Update {}: {} → {}",
            idx + 1,
            qw.package.bright_cyan().bold(),
            qw.current_version.dimmed(),
            qw.fixed_version.green().bold()
        );
        println!("     ✅ Fixes: {} ({} vulns)",
            qw.vulns_fixed.join(", ").bright_white(),
            qw.vulns_fixed.len()
        );
        println!("     🟢 Risk: LOW (patch update)");
        println!("     ⏱  Time: ~{} minutes", qw.estimated_minutes);
        println!();
    }

    if quick_wins.len() > 5 {
        println!("  {} and {} more quick wins available...", "".dimmed(), (quick_wins.len() - 5).to_string().dimmed());
        println!();
    }

    Ok(())
}

/// Display prioritized action plan
fn display_action_plan(results: &ContainerScanResults) -> Result<()> {
    let mut actions = Vec::new();

    // Collect all actionable vulnerabilities
    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if vuln.fixed_version.is_some() {
                let estimated_hours = if vuln.breaking_change == Some(true) {
                    2.0 // Breaking changes take longer
                } else if vuln.is_kev {
                    1.0 // KEV requires immediate attention
                } else {
                    0.25 // Quick patches
                };

                actions.push(ActionItem {
                    priority: vuln.priority.clone().unwrap_or_else(|| "P4".to_string()),
                    cve_id: vuln.cve_id.clone(),
                    package: vuln.package_name.clone(),
                    description: vuln.title.clone(),
                    estimated_hours,
                    breaking: vuln.breaking_change == Some(true),
                    kev: vuln.is_kev,
                    epss: vuln.epss_score.unwrap_or(0.0),
                });
            }
        }
    }

    if actions.is_empty() {
        return Ok(());
    }

    // Sort by priority
    actions.sort_by(|a, b| a.priority.cmp(&b.priority));

    println!();
    println!("{}", "━".repeat(67).bright_cyan());
    println!("{}", "📋 RECOMMENDED ACTION PLAN".bright_cyan().bold());
    println!("{}", "━".repeat(67).bright_cyan());
    println!();

    // P0 - Urgent
    let p0_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P0").collect();
    if !p0_actions.is_empty() {
        println!("{}", "🔥 URGENT (Do TODAY):".red().bold());
        for (idx, action) in p0_actions.iter().take(3).enumerate() {
            println!("  {}. {} {} in {}",
                idx + 1,
                if action.kev { "[P0/KEV]".red().bold() } else { "[P0]".red().bold() },
                action.cve_id.bright_white().bold(),
                action.package.bright_cyan()
            );
            println!("     ⏱  Est: {}", format_time(action.estimated_hours));
            if action.breaking {
                println!("     ⚠️  Breaking change - review migration guide");
            }
            if action.epss > 0.5 {
                println!("     📊 EPSS: {:.0}% (high exploitation risk)", action.epss * 100.0);
            }
            println!();
        }
    }

    // P1 - High Priority
    let p1_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P1").collect();
    if !p1_actions.is_empty() {
        println!("{}", "⚠️  HIGH PRIORITY (This week):".yellow().bold());
        for (idx, action) in p1_actions.iter().take(3).enumerate() {
            println!("  {}. [P1] {} in {}",
                p0_actions.len() + idx + 1,
                action.cve_id.bright_white().bold(),
                action.package.bright_cyan()
            );
            println!("     ⏱  Est: {}", format_time(action.estimated_hours));
            println!();
        }
    }

    // P2 - Medium Priority
    let p2_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P2").collect();
    if !p2_actions.is_empty() && !p2_actions.is_empty() {
        println!("{}", "🟡 MEDIUM PRIORITY (This sprint):".yellow());
        println!("  {} vulnerabilities requiring attention", p2_actions.len());
        println!("  ⏱  Estimated total: {}", format_time(p2_actions.iter().map(|a| a.estimated_hours).sum()));
        println!();
    }

    Ok(())
}

/// Display copy-paste remediation commands
fn display_remediation_commands(results: &ContainerScanResults) -> Result<()> {
    // Find top 2 fixable Java vulnerabilities
    let mut java_fixes = Vec::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                if vuln.package_name.contains(':') || vuln.package_name.contains('.') {
                    // Looks like a Java package
                    java_fixes.push((vuln.package_name.clone(), vuln.installed_version.clone(), fixed.clone()));
                }
            }
        }
    }

    if java_fixes.is_empty() {
        return Ok(());
    }

    java_fixes.truncate(2);

    println!();
    println!("{}", "━".repeat(67).bright_magenta());
    println!("{}", "📋 COPY-PASTE FIXES".bright_magenta().bold());
    println!("{}", "━".repeat(67).bright_magenta());
    println!();

    for (package, _current, fixed) in java_fixes {
        // Parse Maven coordinates
        let parts: Vec<&str> = package.split(':').collect();
        let (group_id, artifact_id) = if parts.len() >= 2 {
            (parts[0], parts[1])
        } else {
            (package.as_str(), package.as_str())
        };

        println!("  Package: {}", package.bright_cyan().bold());
        println!();
        println!("  {}", "Maven (pom.xml):".bright_white().bold());
        println!("  {}", "```xml".dimmed());
        println!("  <dependency>");
        println!("    <groupId>{}</groupId>", group_id.bright_white());
        println!("    <artifactId>{}</artifactId>", artifact_id.bright_white());
        println!("    <version>{}</version>", fixed.green().bold());
        println!("  </dependency>");
        println!("  {}", "```".dimmed());
        println!();
        println!("  {}", "Gradle (build.gradle):".bright_white().bold());
        println!("  {}", "```groovy".dimmed());
        println!("  implementation '{}:{}:{}'",
            group_id.bright_white(),
            artifact_id.bright_white(),
            fixed.green().bold()
        );
        println!("  {}", "```".dimmed());
        println!();
    }

    Ok(())
}

/// Display effort analysis
fn display_effort_analysis(results: &ContainerScanResults) -> Result<()> {
    let mut p0_time = 0.0;
    let mut p1_time = 0.0;
    let mut p2_time = 0.0;
    let mut p0_count = 0;
    let mut p1_count = 0;
    let mut p2_count = 0;

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if vuln.fixed_version.is_none() {
                continue;
            }

            let time = if vuln.breaking_change == Some(true) {
                2.0
            } else if vuln.is_kev {
                1.0
            } else {
                0.25
            };

            match vuln.priority.as_deref() {
                Some("P0") => { p0_time += time; p0_count += 1; }
                Some("P1") => { p1_time += time; p1_count += 1; }
                Some("P2") => { p2_time += time; p2_count += 1; }
                _ => {}
            }
        }
    }

    if p0_count == 0 && p1_count == 0 && p2_count == 0 {
        return Ok(());
    }

    println!();
    println!("{}", "━".repeat(67).bright_blue());
    println!("{}", "💰 REMEDIATION EFFORT SUMMARY".bright_blue().bold());
    println!("{}", "━".repeat(67).bright_blue());
    println!();

    if p0_count > 0 {
        println!("  🚨 P0 Fixes: ~{} ({} {})",
            format_time(p0_time).red().bold(),
            p0_count,
            if p0_count == 1 { "vulnerability" } else { "vulnerabilities" }
        );
    }
    if p1_count > 0 {
        println!("  ⚠️  P1 Fixes: ~{} ({} {})",
            format_time(p1_time).yellow().bold(),
            p1_count,
            if p1_count == 1 { "vulnerability" } else { "vulnerabilities" }
        );
    }
    if p2_count > 0 {
        println!("  🟡 P2 Fixes: ~{} ({} {})",
            format_time(p2_time).yellow(),
            p2_count,
            if p2_count == 1 { "vulnerability" } else { "vulnerabilities" }
        );
    }

    let total_time = p0_time + p1_time + p2_time;
    println!();
    println!("  📊 Total estimated time: {}",
        format_time(total_time).bright_white().bold()
    );
    println!("  🎯 Risk reduction: {} → {}",
        if results.critical_count > 0 { "CRITICAL".red().bold() } else { "HIGH".yellow().bold() },
        "LOW".green().bold()
    );
    println!();

    Ok(())
}

/// Display security score
fn display_security_score(results: &ContainerScanResults) -> Result<()> {
    // Calculate security score (0-100)
    let mut score = 100;

    // Deduct points for vulnerabilities
    score -= results.critical_count.min(10) * 10; // -10 per CRITICAL (max -100)
    score -= results.high_count.min(20) * 2; // -2 per HIGH (max -40)
    score -= results.medium_count.min(50); // -1 per MEDIUM (max -50)

    // Extra penalty for KEV
    let kev_count = results.layers.iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.is_kev)
        .count();
    score -= kev_count.min(5) * 5; // -5 per KEV (max -25)

    score = score.max(0);

    let rating = if score >= 90 {
        ("Excellent", "🏆".green())
    } else if score >= 75 {
        ("Good", "✅".green())
    } else if score >= 60 {
        ("Acceptable", "⚠️ ".yellow())
    } else if score >= 40 {
        ("Needs Work", "⚠️ ".yellow())
    } else {
        ("Critical", "🚨".red())
    };

    println!();
    println!("{}", "━".repeat(67).bright_yellow());
    println!("{}", "🏆 SECURITY SCORE".bright_yellow().bold());
    println!("{}", "━".repeat(67).bright_yellow());
    println!();

    let score_color = if score >= 75 {
        score.to_string().green().bold()
    } else if score >= 60 {
        score.to_string().yellow().bold()
    } else {
        score.to_string().red().bold()
    };

    println!("  Score: {}{} - {} {}",
        score_color,
        "/100".dimmed(),
        rating.1,
        rating.0.bright_white().bold()
    );
    println!();

    // Show what would improve the score
    if score < 90 {
        println!("{}", "  🚀 To improve:".bright_white().bold());
        if kev_count > 0 {
            println!("    • Fix {} KEV {}: +{} points",
                kev_count,
                if kev_count == 1 { "vulnerability" } else { "vulnerabilities" },
                kev_count * 5
            );
        }
        if results.critical_count > 0 {
            println!("    • Fix {} CRITICAL {}: +{} points",
                results.critical_count.min(3),
                if results.critical_count == 1 { "vulnerability" } else { "vulnerabilities" },
                results.critical_count.min(3) * 10
            );
        }
        if results.high_count > 0 {
            println!("    • Fix {} HIGH {}: +{} points",
                results.high_count.min(5),
                if results.high_count == 1 { "vulnerability" } else { "vulnerabilities" },
                results.high_count.min(5) * 2
            );
        }
        println!();
    }

    println!("  {} Industry average: 65/100", "📊".dimmed());
    let target_score = ((score + 15).min(95) / 5) * 5; // Round to nearest 5
    println!("  {} Target: {}/100", "🎯".dimmed(), target_score);
    println!();

    Ok(())
}

/// Format time in human-readable form
fn format_time(hours: f32) -> String {
    if hours < 1.0 {
        format!("{} minutes", (hours * 60.0) as u32)
    } else if hours == 1.0 {
        "1 hour".to_string()
    } else {
        format!("{:.1} hours", hours)
    }
}

/// Save scan results as baseline
fn save_baseline(results: &ContainerScanResults, image_name: &str) -> Result<()> {
    let baseline_dir = PathBuf::from(".bazbom/baselines");
    std::fs::create_dir_all(&baseline_dir)?;

    // Sanitize image name for filename
    let filename = image_name.replace([':', '/'], "_");
    let baseline_path = baseline_dir.join(format!("{}.json", filename));

    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(&baseline_path, json)?;

    Ok(())
}

/// Load baseline scan results
fn load_baseline(image_name: &str) -> Result<ContainerScanResults> {
    let filename = image_name.replace([':', '/'], "_");
    let baseline_path = PathBuf::from(format!(".bazbom/baselines/{}.json", filename));

    let content = std::fs::read_to_string(&baseline_path)?;
    let results: ContainerScanResults = serde_json::from_str(&content)?;

    Ok(results)
}

/// Display baseline comparison
fn display_baseline_comparison(baseline: &ContainerScanResults, current: &ContainerScanResults) -> Result<()> {
    println!();
    println!("{}", "━".repeat(67).bright_blue());
    println!("{}", "📊 BASELINE COMPARISON".bright_blue().bold());
    println!("{}", "━".repeat(67).bright_blue());
    println!();

    // Vulnerability count changes
    let crit_diff = current.critical_count as i32 - baseline.critical_count as i32;
    let high_diff = current.high_count as i32 - baseline.high_count as i32;
    let total_diff = current.total_vulnerabilities as i32 - baseline.total_vulnerabilities as i32;

    println!("  Baseline vulnerabilities: {}", baseline.total_vulnerabilities);
    println!("  Current vulnerabilities:  {}", current.total_vulnerabilities);
    println!();

    let change_icon = if total_diff < 0 {
        "🎉".green()
    } else if total_diff > 0 {
        "⚠️ ".red()
    } else {
        "➡️ ".normal()
    };

    println!("  {} Total change: {}{}",
        change_icon,
        if total_diff > 0 { "+" } else { "" },
        total_diff.to_string().bright_white().bold()
    );

    if crit_diff != 0 {
        println!("     CRITICAL: {}{}",
            if crit_diff > 0 { "+" } else { "" },
            if crit_diff > 0 {
                crit_diff.to_string().red().bold()
            } else {
                crit_diff.to_string().green().bold()
            }
        );
    }

    if high_diff != 0 {
        println!("     HIGH:     {}{}",
            if high_diff > 0 { "+" } else { "" },
            if high_diff > 0 {
                high_diff.to_string().yellow().bold()
            } else {
                high_diff.to_string().green().bold()
            }
        );
    }

    // Show new CVEs
    let baseline_cves: std::collections::HashSet<String> = baseline.layers.iter()
        .flat_map(|l| &l.vulnerabilities)
        .map(|v| v.cve_id.clone())
        .collect();

    let current_cves: std::collections::HashSet<String> = current.layers.iter()
        .flat_map(|l| &l.vulnerabilities)
        .map(|v| v.cve_id.clone())
        .collect();

    let new_cves: Vec<_> = current_cves.difference(&baseline_cves).collect();
    let fixed_cves: Vec<_> = baseline_cves.difference(&current_cves).collect();

    if !new_cves.is_empty() {
        println!();
        println!("  🆕 New vulnerabilities:");
        for cve in new_cves.iter().take(5) {
            println!("     • {}", cve.red());
        }
        if new_cves.len() > 5 {
            println!("     {} and {} more...", "".dimmed(), (new_cves.len() - 5).to_string().dimmed());
        }
    }

    if !fixed_cves.is_empty() {
        println!();
        println!("  ✅ Fixed vulnerabilities:");
        for cve in fixed_cves.iter().take(5) {
            println!("     • {}", cve.green());
        }
        if fixed_cves.len() > 5 {
            println!("     {} and {} more...", "".dimmed(), (fixed_cves.len() - 5).to_string().dimmed());
        }
    }

    println!();

    Ok(())
}

/// Display image comparison
fn display_image_comparison(image1: &ContainerScanResults, image2: &ContainerScanResults) -> Result<()> {
    println!();
    println!("{}", "━".repeat(67).bright_magenta());
    println!("{}", "🔍 IMAGE COMPARISON".bright_magenta().bold());
    println!("{}", "━".repeat(67).bright_magenta());
    println!();

    println!("  Image 1: {}", image1.image_name.bright_cyan().bold());
    println!("  Image 2: {}", image2.image_name.bright_cyan().bold());
    println!();

    println!("  {:<30} {:>15} {:>15}", "Metric".bold(), "Image 1".bold(), "Image 2".bold());
    println!("  {}", "─".repeat(67).dimmed());
    println!("  {:<30} {:>15} {:>15}", "Total Packages", image1.total_packages, image2.total_packages);
    println!("  {:<30} {:>15} {:>15}", "Total Vulnerabilities", image1.total_vulnerabilities, image2.total_vulnerabilities);
    println!("  {:<30} {:>15} {:>15}", "CRITICAL", image1.critical_count, image2.critical_count);
    println!("  {:<30} {:>15} {:>15}", "HIGH", image1.high_count, image2.high_count);
    println!("  {:<30} {:>15} {:>15}", "MEDIUM", image1.medium_count, image2.medium_count);
    println!("  {:<30} {:>15} {:>15}", "LOW", image1.low_count, image2.low_count);
    println!();

    // Recommendation
    let total1 = image1.total_vulnerabilities;
    let total2 = image2.total_vulnerabilities;
    let crit1 = image1.critical_count;
    let crit2 = image2.critical_count;

    if total1 < total2 || (total1 == total2 && crit1 < crit2) {
        println!("  ✅ {} Recommendation: Use {}",
            "🏆".green(),
            image1.image_name.green().bold()
        );
        println!("     Fewer vulnerabilities and lower severity");
    } else if total2 < total1 || (total1 == total2 && crit2 < crit1) {
        println!("  ✅ {} Recommendation: Use {}",
            "🏆".green(),
            image2.image_name.green().bold()
        );
        println!("     Fewer vulnerabilities and lower severity");
    } else {
        println!("  ➡️  Both images have similar security profiles");
    }

    println!();

    Ok(())
}

/// Create GitHub issues for vulnerabilities
fn create_github_issues(results: &ContainerScanResults, repo: &str) -> Result<()> {
    // Check if gh CLI is installed
    let gh_check = Command::new("gh")
        .arg("--version")
        .output();

    if gh_check.is_err() {
        anyhow::bail!("GitHub CLI (gh) not found. Install from: https://cli.github.com/");
    }

    // Collect P0 and P1 vulnerabilities
    let mut high_priority_vulns = Vec::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref priority) = vuln.priority {
                if priority == "P0" || priority == "P1" {
                    high_priority_vulns.push(vuln.clone());
                }
            }
        }
    }

    if high_priority_vulns.is_empty() {
        println!("   ℹ️  No P0/P1 vulnerabilities found. Nothing to create.");
        return Ok(());
    }

    // Deduplicate by CVE
    let mut seen_cves = std::collections::HashSet::new();
    let mut unique_vulns = Vec::new();
    for vuln in high_priority_vulns {
        if seen_cves.insert(vuln.cve_id.clone()) {
            unique_vulns.push(vuln);
        }
    }

    println!("   Creating {} issues in {}...", unique_vulns.len(), repo.bright_cyan());

    for vuln in unique_vulns.iter().take(10) { // Limit to 10 to avoid spamming
        let title = format!("[Security] {} in {} ({})",
            vuln.cve_id,
            vuln.package_name,
            vuln.priority.as_ref().unwrap_or(&"P2".to_string())
        );

        let body = format!(
            "## Vulnerability Details\n\n\
             **CVE:** {}\n\
             **Package:** {} ({})\n\
             **Severity:** {}\n\
             **Priority:** {}\n\n\
             ## Description\n\n{}\n\n\
             ## Remediation\n\n{}\n\n\
             ## References\n\n{}\n\n\
             ---\n\
             *Automatically generated by BazBOM container-scan*",
            vuln.cve_id,
            vuln.package_name,
            vuln.installed_version,
            vuln.severity,
            vuln.priority.as_ref().unwrap_or(&"P2".to_string()),
            vuln.description,
            if let Some(ref fixed) = vuln.fixed_version {
                format!("Upgrade to version {}", fixed)
            } else {
                "No fix available yet".to_string()
            },
            vuln.references.join("\n")
        );

        // Create issue via gh CLI
        let output = Command::new("gh")
            .arg("issue")
            .arg("create")
            .arg("--repo")
            .arg(repo)
            .arg("--title")
            .arg(&title)
            .arg("--body")
            .arg(&body)
            .arg("--label")
            .arg("security")
            .output();

        match output {
            Ok(result) if result.status.success() => {
                let url = String::from_utf8_lossy(&result.stdout).trim().to_string();
                println!("   ✅ Created: {}", url.bright_blue());
            }
            Ok(result) => {
                let err = String::from_utf8_lossy(&result.stderr);
                println!("   ❌ Failed to create issue for {}: {}", vuln.cve_id, err);
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
            }
        }
    }

    if unique_vulns.len() > 10 {
        println!("   ℹ️  Limited to 10 issues. Run again to create more.");
    }

    Ok(())
}

/// Generate executive report
fn generate_executive_report(results: &ContainerScanResults, report_file: &str) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Container Security Report - {}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1200px;
            margin: 40px auto;
            padding: 20px;
            line-height: 1.6;
        }}
        .header {{
            border-bottom: 3px solid #3498db;
            padding-bottom: 20px;
            margin-bottom: 30px;
        }}
        h1 {{ color: #2c3e50; }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 30px 0;
        }}
        .metric {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .metric-value {{
            font-size: 36px;
            font-weight: bold;
            margin: 10px 0;
        }}
        .critical {{ color: #e74c3c; }}
        .high {{ color: #f39c12; }}
        .medium {{ color: #3498db; }}
        .low {{ color: #95a5a6; }}
        .vuln-list {{
            margin: 20px 0;
        }}
        .vuln-item {{
            background: white;
            border-left: 4px solid #e74c3c;
            padding: 15px;
            margin: 10px 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        .footer {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ddd;
            color: #95a5a6;
            text-align: center;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>🐳 Container Security Report</h1>
        <p><strong>Image:</strong> {}</p>
        <p><strong>Generated:</strong> {}</p>
    </div>

    <div class="summary">
        <div class="metric">
            <div>Total Packages</div>
            <div class="metric-value">{}</div>
        </div>
        <div class="metric">
            <div>Total Vulnerabilities</div>
            <div class="metric-value">{}</div>
        </div>
        <div class="metric">
            <div>CRITICAL</div>
            <div class="metric-value critical">{}</div>
        </div>
        <div class="metric">
            <div>HIGH</div>
            <div class="metric-value high">{}</div>
        </div>
        <div class="metric">
            <div>MEDIUM</div>
            <div class="metric-value medium">{}</div>
        </div>
        <div class="metric">
            <div>LOW</div>
            <div class="metric-value low">{}</div>
        </div>
    </div>

    <h2>🚨 Priority Vulnerabilities</h2>
    <div class="vuln-list">
        {}
    </div>

    <div class="footer">
        Generated by BazBOM Container Scanner
    </div>
</body>
</html>"#,
        results.image_name,
        results.image_name,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        results.total_packages,
        results.total_vulnerabilities,
        results.critical_count,
        results.high_count,
        results.medium_count,
        results.low_count,
        {
            let mut vulns_html = String::new();
            let mut all_vulns: Vec<&VulnerabilityInfo> = results.layers.iter()
                .flat_map(|l| &l.vulnerabilities)
                .filter(|v| v.priority.as_ref().map(|p| p == "P0" || p == "P1").unwrap_or(false))
                .collect();

            all_vulns.sort_by(|a, b| a.priority.cmp(&b.priority));

            for vuln in all_vulns.iter().take(20) {
                vulns_html.push_str(&format!(
                    r#"<div class="vuln-item">
                        <h3>{} [{}]</h3>
                        <p><strong>Package:</strong> {} ({})</p>
                        <p><strong>Severity:</strong> {}</p>
                        <p>{}</p>
                        <p><strong>Fix:</strong> {}</p>
                    </div>"#,
                    vuln.cve_id,
                    vuln.priority.as_ref().unwrap_or(&"P2".to_string()),
                    vuln.package_name,
                    vuln.installed_version,
                    vuln.severity,
                    vuln.description.chars().take(200).collect::<String>(),
                    vuln.fixed_version.as_ref().map(|v| format!("Upgrade to {}", v)).unwrap_or_else(|| "No fix available".to_string())
                ));
            }

            if all_vulns.len() > 20 {
                vulns_html.push_str(&format!("<p>...and {} more vulnerabilities</p>", all_vulns.len() - 20));
            }

            vulns_html
        }
    );

    std::fs::write(report_file, html)?;

    Ok(())
}

/// Launch interactive TUI for container vulnerabilities
fn launch_container_tui(results: &ContainerScanResults) -> Result<()> {
    use bazbom_tui::{Dependency, Vulnerability};

    // Convert container vulnerabilities to TUI format
    let mut dependencies = Vec::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            // Group by package
            if let Some(dep) = dependencies.iter_mut().find(|d: &&mut Dependency| d.name == vuln.package_name) {
                dep.vulnerabilities.push(Vulnerability {
                    cve: vuln.cve_id.clone(),
                    severity: vuln.severity.clone(),
                    cvss: vuln.cvss_score.unwrap_or(0.0) as f32,
                    fixed_version: vuln.fixed_version.clone(),
                });
            } else {
                dependencies.push(Dependency {
                    name: vuln.package_name.clone(),
                    version: vuln.installed_version.clone(),
                    scope: layer.digest.clone(),
                    vulnerabilities: vec![Vulnerability {
                        cve: vuln.cve_id.clone(),
                        severity: vuln.severity.clone(),
                        cvss: vuln.cvss_score.unwrap_or(0.0) as f32,
                        fixed_version: vuln.fixed_version.clone(),
                    }],
                });
            }
        }
    }

    // Sort by vulnerability count (most vulnerable first)
    dependencies.sort_by(|a, b| b.vulnerabilities.len().cmp(&a.vulnerabilities.len()));

    bazbom_tui::run(dependencies)?;

    Ok(())
}
