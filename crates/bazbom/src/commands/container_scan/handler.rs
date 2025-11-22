//! Container scan handler implementation

use anyhow::{Context, Result};
use colored::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

// Threat intel
use bazbom_threats::ThreatAnalyzer;

// OS upgrade intelligence
use bazbom_upgrade_analyzer::os_upgrade;
use bazbom_upgrade_analyzer::UpgradeAnalyzer;

// Reports
use bazbom_reports::{
    ComplianceFramework, ContainerCompliance, PolicyStatus,
    ReachabilitySummary as ReportReachability, ReportGenerator, ReportType, SbomData,
    VulnerabilityDetail, VulnerabilityFindings,
};

// SARIF format
use bazbom_formats::sarif::{Result as SarifResult, Rule, SarifReport};

// Import from parent module (types.rs, enrichment.rs, display.rs)
use super::{
    analyze_upgrade_impact,
    // Display functions
    apply_filter,
    create_github_issues,
    display_baseline_comparison,
    display_image_comparison,
    display_results,
    enrich_vulnerabilities,
    enrich_vulnerabilities_with_os,
    load_baseline,
    save_baseline,
    ComplianceResults,
    ComplianceStatus,
    ContainerScanOptions,
    ContainerScanResults,
    DockerLayerMetadata,
    LayerInfo,
    ProvenanceStatus,
    ReachabilitySummary,
    SignatureStatus,
    UpgradeRecommendation,
    VulnerabilityInfo,
};

// Tool orchestrator integration
use bazbom_containers::tools::{findings::AggregatedResults, OrchestratorConfig, ToolOrchestrator};

// Native OS package scanning (fallback when external tools unavailable)
use bazbom_containers::os_packages;

// Types are imported from super::types
use super::dependency_graph::DependencyGraph;

// Polyglot scanner for JAR files
use bazbom_orchestrator::{OrchestratorConfig as PolyglotOrchestratorConfig, ParallelOrchestrator};
use bazbom_scanner::EcosystemScanResult;

/// Main container scan command handler
pub async fn handle_container_scan(opts: ContainerScanOptions) -> Result<()> {
    println!();
    println!(
        "{}",
        "====================================================================="
            .bright_magenta()
            .bold()
    );
    println!(
        "{} {:^67} {}",
        "|".bright_magenta().bold(),
        "BAZBOM CONTAINER SECURITY ANALYSIS",
        "|".bright_magenta().bold()
    );
    println!(
        "{}",
        "====================================================================="
            .bright_magenta()
            .bold()
    );
    println!();
    println!("   Image:  {}", opts.image_name.bright_white().bold());
    println!(
        "   Output: {}",
        opts.output_dir.display().to_string().dimmed()
    );
    println!();

    // Create output directories
    std::fs::create_dir_all(&opts.output_dir)?;
    std::fs::create_dir_all(opts.output_dir.join("sbom"))?;
    std::fs::create_dir_all(opts.output_dir.join("findings"))?;

    // Track warnings/issues during scan for summary at end
    let mut scan_warnings: Vec<String> = Vec::new();

    // Step 1: Check for required tools
    println!("TOOL {} Checking for required tools...", "Step 1/8:".bold());
    let preflight = check_tools(&opts)?;
    if !preflight.docker_available {
        let msg = "Docker is not available or the daemon is unreachable.";
        if opts.offline {
            println!("   WARN  {} {}", "Warning:".yellow(), msg);
            scan_warnings.push(format!("{} (offline mode)", msg));
        } else {
            anyhow::bail!("{} Please start Docker or install it before scanning.", msg);
        }
    }

    let mut fatal_tools: Vec<String> = Vec::new();
    for (name, hint) in &preflight.missing_tools {
        match name.as_str() {
            "syft" | "trivy" => fatal_tools.push(format!("{} ({})", name, hint)),
            _ => {
                println!("   WARN  {} {} - {}", "Warning:".yellow(), name, hint);
                scan_warnings.push(format!("Missing tool {} - {}", name, hint));
            }
        }
    }

    if !fatal_tools.is_empty() {
        anyhow::bail!(format!(
            "Required tools missing: {}",
            fatal_tools.join(", ")
        ));
    }

    if !preflight.missing_tools.is_empty() {
        println!(
            "   WARN  {} Some optional tools are missing; results may be partial.",
            "Warning:".yellow()
        );
    } else {
        println!("   OK All tools available");
    }
    println!();

    // Step 2: Pull container image (ensures local cache before tools run)
    if opts.offline || opts.skip_pull {
        let reason = if opts.offline {
            "offline mode"
        } else {
            "skip-pull requested"
        };
        println!(
            "PULL {} Skipping image pull ({})",
            "Step 2/8:".bold(),
            reason
        );
        scan_warnings.push(format!("Image pull skipped ({})", reason));
    } else {
        println!("PULL {} Pulling container image...", "Step 2/8:".bold());
        let pull_output = Command::new("docker")
            .args(["pull", &opts.image_name])
            .output()
            .context("Failed to run docker pull")?;

        if pull_output.status.success() {
            println!("   OK Image pulled successfully");
        } else {
            let stderr = String::from_utf8_lossy(&pull_output.stderr);
            if stderr.contains("up to date") || stderr.contains("Already exists") {
                println!("   OK Image already cached locally");
            } else {
                // Non-fatal - tools may still work if image exists
                let msg = format!(
                    "Image pull failed: {}",
                    stderr.lines().next().unwrap_or("unknown error")
                );
                println!("   WARN  {} {}", "Warning:".yellow(), msg);
                scan_warnings.push(msg);
            }
        }
        println!();
    }

    // Step 3: Verify container signature and provenance
    println!(
        "SECURE {} Verifying container signature and provenance...",
        "Step 3/8:".bold()
    );
    if opts.offline {
        println!(
            "   SKIP  {} Offline mode - skipping signature and provenance verification",
            "Skipped:".dimmed()
        );
        scan_warnings.push("Skipped signature/provenance checks (offline mode)".to_string());
    } else {
        let signature_status = verify_container_signature(&opts.image_name).await?;
        match &signature_status {
            SignatureStatus::Verified => {
                println!("   OK Container signature verified (cosign)");
            }
            SignatureStatus::NotSigned => {
                let msg = "Container is not signed";
                println!("   WARN  {} {}", "Warning:".yellow(), msg);
                scan_warnings.push(msg.to_string());
            }
            SignatureStatus::ToolNotAvailable => {
                let msg = "cosign not installed; signature check skipped";
                println!("   SKIP  {} {}", "Skipped:".dimmed(), msg);
                scan_warnings.push(msg.to_string());
            }
            SignatureStatus::Invalid(err) => {
                let first = err.lines().next().unwrap_or("unknown");
                let msg = format!("Invalid signature: {}", first);
                println!("   FAIL {} {}", "Error:".red(), msg);
                if !opts.allow_unsigned {
                    anyhow::bail!(msg);
                }
                scan_warnings.push(format!(
                    "Proceeding despite invalid signature (allow-unsigned): {}",
                    first
                ));
            }
        }

        let provenance_status = verify_slsa_provenance(&opts.image_name).await?;
        match &provenance_status {
            ProvenanceStatus::Verified => {
                println!("   OK SLSA provenance attestation verified");
            }
            ProvenanceStatus::NotAvailable => {
                let msg = "No SLSA provenance attestation";
                println!("   WARN  {} {}", "Warning:".yellow(), msg);
                scan_warnings.push(msg.to_string());
            }
            ProvenanceStatus::ToolNotAvailable => {
                // Already reported for signature check
            }
            ProvenanceStatus::Invalid(err) => {
                let first = err.lines().next().unwrap_or("unknown");
                let msg = format!("Invalid provenance: {}", first);
                println!("   FAIL {} {}", "Error:".red(), msg);
                if !opts.allow_unsigned {
                    anyhow::bail!(msg);
                }
                scan_warnings.push(format!(
                    "Proceeding despite invalid provenance (allow-unsigned): {}",
                    first
                ));
            }
        }
    }
    println!();

    // Step 4: Run comprehensive security scan (all tools in parallel)
    println!(
        "SCAN {} Running comprehensive security scan...",
        "Step 4/8:".bold()
    );
    println!(
        "   {} Trivy, Grype, Syft, Dockle, Dive, TruffleHog",
        "Tools:".dimmed()
    );

    let aggregated_results = run_orchestrated_scan(&opts).await?;

    println!(
        "   OK Found {} packages",
        aggregated_results
            .summary
            .total_packages
            .to_string()
            .bright_green()
            .bold()
    );
    println!(
        "   OK Found {} vulnerabilities ({} critical, {} high)",
        aggregated_results
            .summary
            .total_vulnerabilities
            .to_string()
            .yellow()
            .bold(),
        aggregated_results
            .summary
            .critical_count
            .to_string()
            .red()
            .bold(),
        aggregated_results
            .summary
            .high_count
            .to_string()
            .bright_red()
    );
    if aggregated_results.summary.secrets_count > 0 {
        println!(
            "   WARN  Found {} secrets",
            aggregated_results
                .summary
                .secrets_count
                .to_string()
                .red()
                .bold()
        );
    }
    if aggregated_results.summary.misconfigs_count > 0 {
        println!(
            "   WARN  Found {} misconfigurations",
            aggregated_results
                .summary
                .misconfigs_count
                .to_string()
                .yellow()
                .bold()
        );
    }
    if !aggregated_results.skipped_tools.is_empty() {
        let skipped_list = aggregated_results.skipped_tools.join(", ");
        let msg = format!("Skipped tools: {}", skipped_list);
        println!("   WARN  {} {}", "Warning:".yellow(), msg);
        scan_warnings.push(msg);
    }
    if aggregated_results.executed_tools.is_empty() {
        anyhow::bail!("No container scanning tools executed; cannot proceed.");
    }
    println!();

    // Use the orchestrator output files for layer attribution
    let sbom_path = opts.output_dir.join("syft-sbom.json");
    let vuln_path = opts.output_dir.join("trivy-results.json");
    if !sbom_path.exists() {
        anyhow::bail!(
            "Syft SBOM not found at {}. Ensure syft is installed and succeeded.",
            sbom_path.display()
        );
    }
    if !vuln_path.exists() {
        anyhow::bail!(
            "Trivy results not found at {}. Ensure trivy is installed and succeeded.",
            vuln_path.display()
        );
    }

    // Step 5: Analyze layers and attribute vulnerabilities
    println!("SCAN {} Analyzing layer attribution...", "Step 5/8:".bold());
    let mut results = analyze_layer_attribution(&opts.image_name, &sbom_path, &vuln_path).await?;
    println!(
        "   OK Mapped vulnerabilities to {} layers",
        results.layers.len().to_string().bright_cyan().bold()
    );
    println!();

    // Step 5.5: Native OS scanning and reachability analysis
    println!(
        "TARGET {} Running native OS scanning...",
        "Step 5.5/8:".bold()
    );
    match extract_container_filesystem(&opts.image_name, &opts.output_dir).await {
        Ok(filesystem_dir) => {
            // Run native OS package scanning as supplementary/fallback
            match os_packages::scan_os_packages(&filesystem_dir) {
                Ok(os_results) => {
                    println!(
                        "   OK Native scan: {} {} packages, {} vulnerabilities",
                        os_results.os_info.pretty_name,
                        os_results.packages.len().to_string().bright_cyan(),
                        os_results.vulnerabilities.len().to_string().yellow()
                    );

                    // Merge native scan vulnerabilities with existing results
                    // This provides fallback data when external tools miss packages
                    merge_native_vulnerabilities(&mut results, &os_results);

                    // Re-enrich all vulnerabilities to get EPSS/KEV/severity for native scan vulns
                    let mut all_vulns: Vec<VulnerabilityInfo> = results
                        .layers
                        .iter()
                        .flat_map(|l| l.vulnerabilities.clone())
                        .collect();
                    if let Err(e) = enrich_vulnerabilities(&mut all_vulns).await {
                        tracing::warn!("Failed to enrich native scan vulnerabilities: {}", e);
                    } else {
                        // Update results with enriched vulnerabilities
                        let mut vuln_map: std::collections::HashMap<
                            (String, String),
                            VulnerabilityInfo,
                        > = all_vulns
                            .into_iter()
                            .map(|v| ((v.cve_id.clone(), v.package_name.clone()), v))
                            .collect();
                        for layer in &mut results.layers {
                            for vuln in &mut layer.vulnerabilities {
                                if let Some(enriched) = vuln_map
                                    .remove(&(vuln.cve_id.clone(), vuln.package_name.clone()))
                                {
                                    *vuln = enriched;
                                }
                            }
                        }
                    }

                    // Get upgrade recommendations for vulnerable packages
                    if !os_results.vulnerabilities.is_empty() {
                        let packages: Vec<(String, String)> = os_results
                            .packages
                            .iter()
                            .map(|p| (p.name.clone(), p.version.clone()))
                            .collect();

                        // Map OS type to deps.dev System
                        let system = match os_results.os_info.os_type {
                            os_packages::OsType::Alpine => bazbom_depsdev::System::Alpine,
                            os_packages::OsType::Debian | os_packages::OsType::Ubuntu => {
                                bazbom_depsdev::System::Debian
                            }
                            _ => bazbom_depsdev::System::Rpm,
                        };

                        let release = match os_results.os_info.os_type {
                            os_packages::OsType::Alpine => {
                                // Extract branch like "3.18" from version
                                let parts: Vec<&str> =
                                    os_results.os_info.version_id.split('.').collect();
                                if parts.len() >= 2 {
                                    Some(format!("v{}.{}", parts[0], parts[1]))
                                } else {
                                    Some("edge".to_string())
                                }
                            }
                            _ => None,
                        };

                        match os_upgrade::get_os_upgrade_recommendations(
                            system,
                            &packages,
                            release.as_deref(),
                        ) {
                            Ok(recommendations) if !recommendations.is_empty() => {
                                println!(
                                    "   PKG {} upgrade recommendations available",
                                    recommendations.len().to_string().bright_cyan()
                                );

                                // Store recommendations in results for display
                                results.upgrade_recommendations = recommendations
                                    .into_iter()
                                    .map(|r| UpgradeRecommendation {
                                        package: r.package,
                                        installed_version: r.installed_version,
                                        recommended_version: r.recommended_version,
                                        fixes_cves: r.fixes_cves,
                                        risk_level: format!("{:?}", r.risk_level),
                                        // OS package upgrades don't have detailed analysis yet
                                        effort_hours: None,
                                        breaking_changes_count: None,
                                        transitive_upgrades_count: None,
                                        migration_guide_url: None,
                                        success_rate: None,
                                        github_repo: None,
                                    })
                                    .collect();
                            }
                            Ok(_) => {
                                // No recommendations needed
                            }
                            Err(e) => {
                                tracing::debug!("Failed to get upgrade recommendations: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    // Non-fatal - external tools may have already found everything
                    tracing::debug!("Native OS scan unavailable: {}", e);
                }
            }

            match analyze_container_reachability(&mut results, &filesystem_dir).await {
                Ok(_) => {
                    let total_vulns = results
                        .layers
                        .iter()
                        .flat_map(|l| &l.vulnerabilities)
                        .count();
                    let reachable_count = results
                        .layers
                        .iter()
                        .flat_map(|l| &l.vulnerabilities)
                        .filter(|v| v.is_reachable)
                        .count();
                    let unreachable_count = total_vulns - reachable_count;
                    let noise_reduction = if total_vulns > 0 {
                        (unreachable_count as f64 / total_vulns as f64) * 100.0
                    } else {
                        0.0
                    };

                    // Populate reachability summary
                    results.reachability_summary = Some(ReachabilitySummary {
                        total_analyzed: total_vulns,
                        reachable_count,
                        unreachable_count,
                        noise_reduction_percent: noise_reduction,
                    });

                    println!(
                        "   OK Found {} reachable vulnerabilities",
                        reachable_count.to_string().red().bold()
                    );
                }
                Err(e) => {
                    let msg = format!("Reachability analysis error: {}", e);
                    tracing::warn!("{}", msg);
                    eprintln!("   WARN  {}", msg);
                    scan_warnings.push(msg);
                }
            }
        }
        Err(e) => {
            let msg = format!("Filesystem extraction failed: {}", e);
            tracing::warn!("{}", msg);
            eprintln!("   WARN  {}", msg);
            scan_warnings.push(msg);
        }
    }

    // Clean up extracted filesystem (can be gigabytes)
    let filesystem_dir = opts.output_dir.join("filesystem");
    if filesystem_dir.exists() {
        if let Err(e) = std::fs::remove_dir_all(&filesystem_dir) {
            tracing::warn!("Failed to clean up filesystem directory: {}", e);
        }
    }

    // Populate compliance results based on vulnerability counts
    let mut pci_issues = vec![];
    let mut hipaa_issues = vec![];
    let mut soc2_issues = vec![];

    if results.critical_count > 0 {
        pci_issues.push(format!(
            "{} critical vulnerabilities present",
            results.critical_count
        ));
        hipaa_issues.push(format!(
            "{} critical vulnerabilities present",
            results.critical_count
        ));
        soc2_issues.push(format!(
            "{} critical vulnerabilities present",
            results.critical_count
        ));
    }
    if results.high_count > 0 {
        pci_issues.push(format!(
            "{} high severity vulnerabilities",
            results.high_count
        ));
        hipaa_issues.push(format!(
            "{} high severity vulnerabilities",
            results.high_count
        ));
    }

    // Count KEV vulns
    let kev_count = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.is_kev)
        .count();
    if kev_count > 0 {
        pci_issues.push(format!(
            "{} known exploited vulnerabilities (CISA KEV)",
            kev_count
        ));
        hipaa_issues.push(format!(
            "{} known exploited vulnerabilities (CISA KEV)",
            kev_count
        ));
        soc2_issues.push(format!(
            "{} known exploited vulnerabilities (CISA KEV)",
            kev_count
        ));
    }

    results.compliance_results = Some(ComplianceResults {
        pci_dss: ComplianceStatus {
            status: if pci_issues.is_empty() {
                "Pass".to_string()
            } else {
                "Fail".to_string()
            },
            issues: pci_issues,
        },
        hipaa: ComplianceStatus {
            status: if hipaa_issues.is_empty() {
                "Pass".to_string()
            } else {
                "Fail".to_string()
            },
            issues: hipaa_issues,
        },
        soc2: ComplianceStatus {
            status: if soc2_issues.is_empty() {
                "Pass".to_string()
            } else {
                "Fail".to_string()
            },
            issues: soc2_issues,
        },
    });

    // Step 5.6: Run recursive transitive upgrade analysis for top fixes
    println!();
    println!(
        "📊 {} Analyzing upgrade impact (breaking changes, transitive deps)...",
        "Step 5.6/8:".bold()
    );

    // Get top 5 critical/high vulns with fixes for deep analysis
    let vulns_for_analysis: Vec<_> = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.fixed_version.is_some() && (v.severity == "CRITICAL" || v.severity == "HIGH"))
        .take(5)
        .collect();

    if !vulns_for_analysis.is_empty() {
        match UpgradeAnalyzer::new() {
            Ok(mut analyzer) => {
                let mut analyzed_count = 0;
                for vuln in &vulns_for_analysis {
                    if let Some(ref fix_ver) = vuln.fixed_version {
                        match analyzer
                            .analyze_upgrade(&vuln.package_name, &vuln.installed_version, fix_ver)
                            .await
                        {
                            Ok(analysis) => {
                                analyzed_count += 1;

                                // Log the analysis results
                                if !analysis.required_upgrades.is_empty() {
                                    tracing::info!(
                                        "Upgrade {} -> {} requires {} transitive upgrades",
                                        vuln.package_name,
                                        fix_ver,
                                        analysis.required_upgrades.len()
                                    );
                                }
                                if !analysis.direct_breaking_changes.is_empty() {
                                    tracing::info!(
                                        "Upgrade {} -> {} has {} breaking changes",
                                        vuln.package_name,
                                        fix_ver,
                                        analysis.direct_breaking_changes.len()
                                    );
                                }

                                // Save to upgrade_recommendations array (NEW!)
                                results.upgrade_recommendations.push(UpgradeRecommendation {
                                    package: vuln.package_name.clone(),
                                    installed_version: vuln.installed_version.clone(),
                                    recommended_version: Some(fix_ver.clone()),
                                    fixes_cves: vec![vuln.cve_id.clone()],
                                    risk_level: analysis.overall_risk.label().to_string(),
                                    effort_hours: Some(analysis.estimated_effort_hours),
                                    breaking_changes_count: Some(analysis.total_breaking_changes()),
                                    transitive_upgrades_count: Some(
                                        analysis.required_upgrades.len(),
                                    ),
                                    migration_guide_url: analysis.migration_guide_url.clone(),
                                    success_rate: analysis.success_rate,
                                    github_repo: analysis.github_repo.clone(),
                                });
                            }
                            Err(e) => {
                                tracing::debug!(
                                    "Failed to analyze upgrade for {}: {}",
                                    vuln.package_name,
                                    e
                                );
                            }
                        }
                    }
                }
                if analyzed_count > 0 {
                    println!(
                        "   OK Analyzed {} package upgrades for breaking changes",
                        analyzed_count.to_string().bright_cyan()
                    );
                }
            }
            Err(e) => {
                tracing::debug!("Failed to initialize upgrade analyzer: {}", e);
            }
        }
    } else {
        println!("   INFO  No critical/high vulnerabilities with fixes to analyze");
    }

    println!();

    // Step 6: Generate beautiful summary
    println!("✨ {} Generating security report...", "Step 6/8:".bold());
    println!();

    // Apply filter if specified
    let filtered_results;
    let display_results_data = if let Some(ref filter) = opts.filter {
        println!("   SCAN Filtering results by: {}", filter.bright_cyan());
        filtered_results = apply_filter(&results, filter)?;
        &filtered_results
    } else {
        &results
    };

    display_results(display_results_data, &opts)?;

    // Save results
    let results_path = opts.output_dir.join("scan-results.json");
    let json = serde_json::to_string_pretty(&results)?;
    std::fs::write(&results_path, json)?;

    // Generate SARIF output if requested
    if opts.format == "sarif" {
        let sarif_path = opts.output_dir.join("findings.sarif");
        let sarif = generate_sarif_report(&results);
        let sarif_json = serde_json::to_string_pretty(&sarif)?;
        std::fs::write(&sarif_path, sarif_json)?;
        println!(
            "   NOTE SARIF report saved to: {}",
            sarif_path.display().to_string().dimmed()
        );
    }

    // Always generate HTML report
    let report_path = opts.output_dir.join("report.html");
    generate_executive_report(&results, &report_path.display().to_string())?;

    println!();
    println!(
        "   📄 Full results saved to: {}",
        results_path.display().to_string().dimmed()
    );
    println!(
        "   📊 HTML report saved to: {}",
        report_path.display().to_string().dimmed()
    );

    // Step 7: Enhanced reporting (threat intel, compliance, PDF)
    println!();
    println!("NOTE {} Generating enhanced reports...", "Step 7/8:".bold());

    // Threat intel analysis
    let packages: Vec<(String, String)> = results
        .layers
        .iter()
        .flat_map(|l| l.packages.iter().map(|p| (p.clone(), "latest".to_string())))
        .collect();

    let threat_analyzer = ThreatAnalyzer::new();
    let threats = threat_analyzer.analyze_packages(&packages)?;
    if !threats.is_empty() {
        println!(
            "   WARN  Found {} supply chain threats",
            threats.len().to_string().red().bold()
        );
        for threat in threats.iter().take(3) {
            println!("      • {} ({:?})", threat.package_name, threat.threat_type);
        }
    } else {
        println!("   OK No supply chain threats detected");
    }

    // Convert results to ReportGenerator format
    let sbom_data = SbomData {
        project_name: results.image_name.clone(),
        project_version: "container".to_string(),
        scan_timestamp: chrono::Utc::now(),
        total_dependencies: results.total_packages,
        direct_dependencies: results.total_packages,
        transitive_dependencies: 0,
    };

    let vuln_findings = convert_to_vuln_findings(&results);
    let policy_status = PolicyStatus {
        policy_violations: results.critical_count,
        license_issues: 0,
        blocked_packages: threats.len(),
    };

    // Build container-specific data for PDF report
    let report_reachability = results
        .reachability_summary
        .as_ref()
        .map(|r| ReportReachability {
            total_analyzed: r.total_analyzed,
            reachable_count: r.reachable_count,
            unreachable_count: r.unreachable_count,
            noise_reduction_percent: r.noise_reduction_percent,
        })
        .unwrap_or_default();

    let report_compliance = results
        .compliance_results
        .as_ref()
        .map(|c| ContainerCompliance {
            pci_dss_pass: c.pci_dss.status == "Pass",
            hipaa_pass: c.hipaa.status == "Pass",
            soc2_pass: c.soc2.status == "Pass",
            pci_issues: c.pci_dss.issues.clone(),
            hipaa_issues: c.hipaa.issues.clone(),
            soc2_issues: c.soc2.issues.clone(),
        })
        .unwrap_or_default();

    let report_gen = ReportGenerator::with_container_data(
        sbom_data,
        vuln_findings,
        policy_status,
        report_reachability,
        report_compliance,
    );

    // Generate compliance reports
    let compliance_dir = opts.output_dir.join("compliance");
    std::fs::create_dir_all(&compliance_dir)?;

    for framework in [
        ComplianceFramework::PciDss,
        ComplianceFramework::Hipaa,
        ComplianceFramework::Soc2,
    ] {
        let filename = match framework {
            ComplianceFramework::PciDss => "pci-dss.html",
            ComplianceFramework::Hipaa => "hipaa.html",
            ComplianceFramework::Soc2 => "soc2.html",
            _ => continue,
        };
        let path = compliance_dir.join(filename);
        report_gen.generate(ReportType::Compliance(framework), &path)?;
    }
    println!("   OK Compliance reports: PCI-DSS, HIPAA, SOC2");

    // Generate PDF report
    let pdf_path = opts.output_dir.join("report.pdf");
    bazbom_reports::pdf::generate_pdf(&report_gen, &pdf_path)?;
    println!(
        "   📑 PDF report saved to: {}",
        pdf_path.display().to_string().dimmed()
    );

    // Generate Jira ticket markdown files for Critical and High vulns
    let all_vulns: Vec<&VulnerabilityInfo> = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .collect();

    let critical_vulns: Vec<_> = all_vulns
        .iter()
        .filter(|v| v.severity == "CRITICAL")
        .take(10)
        .collect();
    let high_vulns: Vec<_> = all_vulns
        .iter()
        .filter(|v| v.severity == "HIGH")
        .take(10)
        .collect();

    let jira_count = critical_vulns.len() + high_vulns.len();
    if jira_count > 0 {
        let jira_dir = opts.output_dir.join("jira-tickets");
        std::fs::create_dir_all(&jira_dir)?;

        for vuln in critical_vulns.iter().chain(high_vulns.iter()) {
            generate_jira_ticket_files(&jira_dir, vuln, &results.image_name)?;
        }

        println!(
            "   NOTE Jira tickets: {} files in {}",
            jira_count * 2,
            jira_dir.display().to_string().dimmed()
        );
    }

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
            println!("   WARN  No baseline found. Run with --baseline first to create one.");
        }
    }

    // Handle image comparison
    if let Some(ref compare_image) = opts.compare_image {
        println!();
        println!("SCAN {} Scanning comparison image...", "Step 8/8:".bold());

        let compare_output_dir = opts.output_dir.join("comparison");
        std::fs::create_dir_all(&compare_output_dir)?;
        std::fs::create_dir_all(compare_output_dir.join("sbom"))?;
        std::fs::create_dir_all(compare_output_dir.join("findings"))?;

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
            with_reachability: false,
            skip_pull: opts.skip_pull,
            allow_unsigned: opts.allow_unsigned,
            offline: opts.offline,
        })
        .await?;

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
            with_reachability: false,
            skip_pull: opts.skip_pull,
            allow_unsigned: opts.allow_unsigned,
            offline: opts.offline,
        })
        .await?;

        let compare_results =
            analyze_layer_attribution(compare_image, &compare_sbom, &compare_vuln).await?;

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
        println!(
            "   OK Report saved to: {}",
            report_file.bright_white().bold()
        );
    }

    // Handle interactive TUI
    if opts.interactive {
        println!();
        println!(
            "FAST {} Launching interactive explorer...",
            "Press any key".dimmed()
        );
        println!(
            "   {} Use arrow keys to navigate, 'q' to quit",
            "Tip:".dimmed()
        );
        std::thread::sleep(std::time::Duration::from_secs(2));
        launch_container_tui(&results)?;
    }

    // Display scan warnings summary if any
    if !scan_warnings.is_empty() {
        println!();
        println!(
            "{}",
            "╔═══════════════════════════════════════════════════════════════════╗".yellow()
        );
        println!(
            "{} {:^67} {}",
            "║".yellow(),
            format!(
                "WARN  SCAN COMPLETED WITH {} WARNING(S)",
                scan_warnings.len()
            ),
            "║".yellow()
        );
        println!(
            "{}",
            "╚═══════════════════════════════════════════════════════════════════╝".yellow()
        );
        for (i, warning) in scan_warnings.iter().enumerate() {
            println!("   {}. {}", i + 1, warning.dimmed());
        }
    }

    println!();

    Ok(())
}

#[derive(Debug)]
struct PreflightStatus {
    docker_available: bool,
    missing_tools: Vec<(String, String)>,
}

/// Check if required tools are installed
fn check_tools(opts: &ContainerScanOptions) -> Result<PreflightStatus> {
    // Docker availability (binary + daemon)
    let docker_available = Command::new("docker")
        .args(["version", "--format", "{{.Server.Version}}"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    // Check for Syft
    let syft_check = Command::new("syft").arg("version").output();
    if syft_check.is_err() {
        anyhow::bail!(
            "Syft not found. Install with: brew install syft\n   \
             Or visit: https://github.com/anchore/syft#installation"
        );
    }

    // Check for Trivy
    let trivy_check = Command::new("trivy").arg("--version").output();
    if trivy_check.is_err() {
        anyhow::bail!(
            "Trivy not found. Install with: brew install trivy\n   \
             Or visit: https://trivy.dev/latest/getting-started/installation/"
        );
    }

    // Use orchestrator tool list to flag other optional tools early
    let orchestrator =
        ToolOrchestrator::with_config(&opts.output_dir, OrchestratorConfig::default());
    let missing_tools = orchestrator.check_tools();

    Ok(PreflightStatus {
        docker_available,
        missing_tools,
    })
}

/// Verify container image signature using cosign
async fn verify_container_signature(image: &str) -> Result<SignatureStatus> {
    // Check if cosign is available
    let cosign_check = Command::new("cosign").arg("version").output();
    if cosign_check.is_err() {
        return Ok(SignatureStatus::ToolNotAvailable);
    }

    // Try to verify signature
    let output = Command::new("cosign")
        .args(["verify", image, "--output", "text"])
        .output()
        .context("failed to run cosign")?;

    if output.status.success() {
        Ok(SignatureStatus::Verified)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no matching signatures") || stderr.contains("not found") {
            Ok(SignatureStatus::NotSigned)
        } else {
            Ok(SignatureStatus::Invalid(stderr.to_string()))
        }
    }
}

/// Verify SLSA provenance attestation using cosign
async fn verify_slsa_provenance(image: &str) -> Result<ProvenanceStatus> {
    // Check if cosign is available
    let cosign_check = Command::new("cosign").arg("version").output();
    if cosign_check.is_err() {
        return Ok(ProvenanceStatus::ToolNotAvailable);
    }

    // Try to verify SLSA provenance attestation
    let output = Command::new("cosign")
        .args(["verify-attestation", "--type", "slsaprovenance", image])
        .output()
        .context("failed to run cosign verify-attestation")?;

    if output.status.success() {
        Ok(ProvenanceStatus::Verified)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("no matching attestations") || stderr.contains("not found") {
            Ok(ProvenanceStatus::NotAvailable)
        } else {
            Ok(ProvenanceStatus::Invalid(stderr.to_string()))
        }
    }
}

/// Generate SBOM using Syft (both SPDX/CycloneDX and native JSON for layer metadata)
async fn generate_sbom(opts: &ContainerScanOptions) -> Result<PathBuf> {
    // Determine output format and filename based on opts.format
    let (sbom_format, sbom_filename) = match opts.format.as_str() {
        "cyclonedx" => ("cyclonedx-json", "cyclonedx.json"),
        _ => ("spdx-json", "spdx.json"), // default to SPDX
    };
    let sbom_path = opts.output_dir.join("sbom").join(sbom_filename);
    let native_path = opts.output_dir.join("sbom").join("syft-native.json");

    // Generate SBOM in requested format
    let output = Command::new("syft")
        .arg(&opts.image_name)
        .arg("-o")
        .arg(format!("{}={}", sbom_format, sbom_path.display()))
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

/// Get layer metadata from Docker
fn get_docker_layer_info(image_name: &str) -> Result<Vec<DockerLayerMetadata>> {
    // Get layer digests from docker inspect
    let inspect_output = Command::new("docker")
        .arg("inspect")
        .arg(image_name)
        .output()
        .context("Failed to run docker inspect")?;

    if !inspect_output.status.success() {
        let stderr = String::from_utf8_lossy(&inspect_output.stderr);
        anyhow::bail!("docker inspect failed for '{}': {}", image_name, stderr);
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
    // Docker history is newest-first, but RootFS.Layers is oldest-first
    // So we need to reverse the history lines
    let history_lines: Vec<&str> = history.lines().collect();
    let mut layer_idx = 0;
    for line in history_lines.iter().rev() {
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

/// Detect base image from Docker inspect output
fn detect_base_image(image_name: &str) -> Option<String> {
    let inspect_output = Command::new("docker")
        .args(["inspect", "--format", "{{json .Config.Labels}}", image_name])
        .output()
        .ok()?;

    if !inspect_output.status.success() {
        return None;
    }

    let labels_json: serde_json::Value = serde_json::from_slice(&inspect_output.stdout).ok()?;

    // Check common base image labels
    let label_keys = [
        "org.opencontainers.image.base.name",
        "io.buildah.version",
        "maintainer",
    ];

    for key in &label_keys {
        if let Some(value) = labels_json.get(*key).and_then(|v| v.as_str()) {
            if key == &"org.opencontainers.image.base.name" {
                return Some(value.to_string());
            }
        }
    }

    // Try to detect from history - first layer often indicates base
    let history_output = Command::new("docker")
        .args([
            "history",
            "--no-trunc",
            "--format",
            "{{.CreatedBy}}",
            image_name,
        ])
        .output()
        .ok()?;

    let history = String::from_utf8_lossy(&history_output.stdout);
    let lines: Vec<&str> = history.lines().collect();

    // Check all layers for base image indicators (not just the last one)
    for line in &lines {
        // Common base image patterns
        if line.contains("alpine") || line.contains("apk add") {
            return Some("alpine".to_string());
        } else if line.contains("debian") || line.contains("apt-get") {
            return Some("debian".to_string());
        } else if line.contains("ubuntu") {
            return Some("ubuntu".to_string());
        } else if line.contains("centos") || line.contains("yum") {
            return Some("centos/rhel".to_string());
        } else if line.contains("fedora") || line.contains("dnf") {
            return Some("fedora".to_string());
        } else if line.contains("busybox") {
            return Some("busybox".to_string());
        }
    }

    None
}

/// Extract layer-to-package mapping from Syft native JSON
fn extract_layer_package_mapping(output_dir: &Path) -> Result<HashMap<String, Vec<String>>> {
    // Try orchestrator path first (syft-sbom.json), then standalone path (syft-native.json)
    let native_path = output_dir.join("syft-sbom.json");
    let content = if native_path.exists() {
        std::fs::read_to_string(&native_path)?
    } else {
        // Fallback for comparison scans which use generate_sbom
        let alt_path = output_dir.join("syft-native.json");
        std::fs::read_to_string(&alt_path).with_context(|| {
            format!(
                "Could not find syft-sbom.json or syft-native.json in {:?}",
                output_dir
            )
        })?
    };
    let doc: serde_json::Value = serde_json::from_str(&content)?;

    let mut layer_packages: HashMap<String, Vec<String>> = HashMap::new();

    if let Some(artifacts) = doc["artifacts"].as_array() {
        for artifact in artifacts {
            let package_name = artifact["name"].as_str().unwrap_or("unknown").to_string();

            if let Some(locations) = artifact["locations"].as_array() {
                for location in locations {
                    if let Some(layer_id) = location["layerID"].as_str() {
                        let packages = layer_packages.entry(layer_id.to_string()).or_default();
                        // Avoid duplicates within the same layer
                        if !packages.contains(&package_name) {
                            packages.push(package_name.clone());
                        }
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

    // Build dependency graph for transitive dependency analysis
    let dep_graph = DependencyGraph::new(&sbom);

    // Load Syft native JSON to get distro info for faster OSV lookups
    let syft_path = sbom_path
        .parent()
        .map(|p| p.join("syft-sbom.json"))
        .unwrap_or_default();
    let os_hint: Option<String> = if syft_path.exists() {
        let syft_content = std::fs::read_to_string(&syft_path).ok();
        syft_content
            .and_then(|content| serde_json::from_str::<serde_json::Value>(&content).ok())
            .and_then(|doc| doc["distro"]["id"].as_str().map(String::from))
    } else {
        None
    };

    // Load vulnerabilities
    let vuln_content = std::fs::read_to_string(vuln_path)?;
    let vuln_doc: serde_json::Value = serde_json::from_str(&vuln_content)?;

    // Extract vulnerability info
    let mut all_vulnerabilities = Vec::new();
    let mut seen_vulns: std::collections::HashSet<String> = std::collections::HashSet::new();

    if let Some(results) = vuln_doc["Results"].as_array() {
        for result in results {
            let target = result["Target"].as_str().unwrap_or("unknown");

            if let Some(vulns) = result["Vulnerabilities"].as_array() {
                for vuln in vulns {
                    // Deduplicate by CVE+package combination
                    let cve_id = vuln["VulnerabilityID"]
                        .as_str()
                        .unwrap_or("UNKNOWN")
                        .to_string();
                    let package_name = vuln["PkgName"].as_str().unwrap_or("unknown");
                    let dedup_key = format!("{}|{}", cve_id, package_name);

                    if seen_vulns.contains(&dedup_key) {
                        continue;
                    }
                    seen_vulns.insert(dedup_key);

                    let severity = vuln["Severity"]
                        .as_str()
                        .unwrap_or("UNKNOWN")
                        .to_uppercase();
                    let published_date = vuln["PublishedDate"].as_str().map(String::from);
                    let cvss_score = vuln["CVSS"]
                        .as_object()
                        .and_then(|cvss| cvss.get("nvd"))
                        .and_then(|nvd| nvd.get("V3Score"))
                        .and_then(|score| score.as_f64());

                    // Build references list
                    let mut references =
                        vec![format!("https://nvd.nist.gov/vuln/detail/{}", cve_id)];
                    if let Some(refs) = vuln["References"].as_array() {
                        for r in refs.iter().take(2) {
                            if let Some(url) = r.as_str() {
                                references.push(url.to_string());
                            }
                        }
                    }

                    // Detect breaking changes with framework-specific analysis
                    let package_name = vuln["PkgName"].as_str().unwrap_or("unknown");
                    let installed = vuln["InstalledVersion"].as_str().unwrap_or("unknown");
                    let fixed = vuln["FixedVersion"].as_str();
                    let (breaking_change, upgrade_path) = if let Some(fix_ver) = fixed {
                        analyze_upgrade_impact(package_name, installed, fix_ver)
                    } else {
                        (None, None)
                    };

                    all_vulnerabilities.push(VulnerabilityInfo {
                        cve_id: cve_id.clone(),
                        package_name: package_name.to_string(),
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
                        is_reachable: false, // Will be analyzed if --with-reachability is enabled
                        difficulty_score: None, // Will be calculated later
                        call_chain: None,    // Will be populated if reachable
                        dependency_path: dep_graph.find_path(package_name), // Populated from SBOM graph
                    });
                }
            }
        }
    }

    // Enrich vulnerabilities with EPSS and KEV data (using OS hint for faster OSV lookups)
    enrich_vulnerabilities_with_os(&mut all_vulnerabilities, os_hint.as_deref()).await?;

    // Merge Grype results for cross-validation (additional vulnerabilities only)
    let grype_path = sbom_path
        .parent()
        .map(|p| p.join("grype-results.json"))
        .unwrap_or_else(|| PathBuf::from("grype-results.json"));
    if grype_path.exists() {
        if let Ok(grype_content) = std::fs::read_to_string(&grype_path) {
            if let Ok(grype_doc) = serde_json::from_str::<serde_json::Value>(&grype_content) {
                let mut grype_added = 0;
                if let Some(matches) = grype_doc["matches"].as_array() {
                    for m in matches {
                        let vuln = &m["vulnerability"];
                        let artifact = &m["artifact"];

                        let cve_id = vuln["id"].as_str().unwrap_or("").to_string();
                        let package_name = artifact["name"].as_str().unwrap_or("").to_string();
                        let dedup_key = format!("{}|{}", cve_id, package_name);

                        // Only add if not already present from Trivy
                        if !seen_vulns.contains(&dedup_key) && !cve_id.is_empty() {
                            seen_vulns.insert(dedup_key);

                            let severity = vuln["severity"]
                                .as_str()
                                .unwrap_or("UNKNOWN")
                                .to_uppercase();
                            let fixed_version = vuln["fix"]["versions"]
                                .as_array()
                                .and_then(|v| v.first())
                                .and_then(|v| v.as_str())
                                .map(String::from);
                            let cvss_score = vuln["cvss"]
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|c| c["metrics"]["baseScore"].as_f64());

                            all_vulnerabilities.push(VulnerabilityInfo {
                                cve_id: cve_id.clone(),
                                package_name: package_name.clone(),
                                installed_version: artifact["version"]
                                    .as_str()
                                    .unwrap_or("")
                                    .to_string(),
                                fixed_version,
                                severity,
                                title: format!("{} in {}", cve_id, package_name),
                                description: vuln["description"].as_str().unwrap_or("").to_string(),
                                layer_digest: "grype-scan".to_string(),
                                published_date: None,
                                epss_score: None,
                                epss_percentile: None,
                                is_kev: false,
                                kev_due_date: None,
                                cvss_score,
                                priority: None,
                                references: vuln["urls"]
                                    .as_array()
                                    .map(|urls| {
                                        urls.iter()
                                            .filter_map(|u| u.as_str().map(String::from))
                                            .collect()
                                    })
                                    .unwrap_or_default(),
                                breaking_change: None,
                                upgrade_path: None,
                                is_reachable: false,
                                difficulty_score: None,
                                call_chain: None,
                                dependency_path: None,
                            });
                            grype_added += 1;
                        }
                    }
                }
                if grype_added > 0 {
                    tracing::info!(
                        "Merged {} additional vulnerabilities from Grype",
                        grype_added
                    );
                    // Re-enrich the new vulnerabilities
                    enrich_vulnerabilities_with_os(&mut all_vulnerabilities, os_hint.as_deref())
                        .await?;
                }
            }
        }
    }

    // Get Docker layer metadata
    let docker_layers = get_docker_layer_info(image_name)?;

    // Get output directory from sbom_path
    let output_dir = sbom_path.parent().context("Invalid SBOM path")?;

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
        let artifact_name = vuln
            .package_name
            .split(':')
            .next_back()
            .unwrap_or(&vuln.package_name);
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
        let layer_desc = if docker_layer.command.contains("COPY")
            || docker_layer.command.contains("ADD")
        {
            "Application files".to_string()
        } else if docker_layer.command.contains("RUN") && docker_layer.command.contains("java") {
            "Java runtime".to_string()
        } else if docker_layer.command.contains("RUN") {
            "Base OS packages".to_string()
        } else {
            "Configuration".to_string()
        };

        layers.push(LayerInfo {
            digest: format!(
                "{} ({})",
                &docker_layer.digest[..20.min(docker_layer.digest.len())],
                layer_desc
            ),
            size_mb: docker_layer.size_bytes as f64 / 1_000_000.0,
            packages: packages.clone(),
            vulnerabilities: layer_vulns,
        });
    }

    // Collect all vulnerabilities that were assigned to layers
    let mut assigned_cve_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for layer in &layers {
        for vuln in &layer.vulnerabilities {
            assigned_cve_ids.insert(vuln.cve_id.clone());
        }
    }

    // Find orphaned vulnerabilities (not assigned to any layer)
    // This happens when Trivy finds JAR vulnerabilities but Syft didn't detect the packages
    let orphaned_vulns: Vec<VulnerabilityInfo> = all_vulnerabilities
        .iter()
        .filter(|v| !assigned_cve_ids.contains(&v.cve_id))
        .cloned()
        .collect();

    // If there are orphaned vulnerabilities, create a synthetic "Java Dependencies" layer
    if !orphaned_vulns.is_empty() {
        tracing::info!(
            "Found {} orphaned vulnerabilities (likely Java dependencies not in Syft SBOM)",
            orphaned_vulns.len()
        );

        // Extract unique package names from orphaned vulnerabilities
        let mut orphaned_packages: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for vuln in &orphaned_vulns {
            orphaned_packages.insert(vuln.package_name.clone());
        }
        let orphaned_packages_vec: Vec<String> = orphaned_packages.into_iter().collect();

        layers.push(LayerInfo {
            digest: "Java Dependencies (from JAR analysis)".to_string(),
            size_mb: 0.0, // Unknown size
            packages: orphaned_packages_vec,
            vulnerabilities: orphaned_vulns,
        });
    }

    // Syft uses "artifacts" not "packages"
    let total_packages = sbom["artifacts"]
        .as_array()
        .map(|arr| arr.len())
        .unwrap_or(0);

    // Recalculate counts from actual layer vulnerabilities (after deduplication)
    let mut final_critical = 0;
    let mut final_high = 0;
    let mut final_medium = 0;
    let mut final_low = 0;
    let mut final_total = 0;

    for layer in &layers {
        for vuln in &layer.vulnerabilities {
            final_total += 1;
            match vuln.severity.as_str() {
                "CRITICAL" => final_critical += 1,
                "HIGH" => final_high += 1,
                "MEDIUM" => final_medium += 1,
                "LOW" => final_low += 1,
                _ => {}
            }
        }
    }

    // Detect base image
    let base_image = detect_base_image(image_name);

    Ok(ContainerScanResults {
        image_name: image_name.to_string(),
        total_packages,
        total_vulnerabilities: final_total,
        layers,
        base_image,
        critical_count: final_critical,
        high_count: final_high,
        medium_count: final_medium,
        low_count: final_low,
        upgrade_recommendations: vec![],
        reachability_summary: None,
        compliance_results: None,
    })
}

/// Extract container filesystem for reachability analysis
async fn extract_container_filesystem(image_name: &str, output_dir: &Path) -> Result<PathBuf> {
    let extract_dir = output_dir.join("filesystem");
    std::fs::create_dir_all(&extract_dir)?;

    // Generate unique container name to avoid collisions
    let container_name = format!("bazbom-{}", std::process::id());

    // Try to export the container filesystem using docker/podman
    let docker_export = Command::new("docker")
        .args(["create", "--name", &container_name, image_name])
        .output();

    let (container_id, use_podman) = if let Ok(output) = docker_export {
        if output.status.success() {
            (
                String::from_utf8_lossy(&output.stdout).trim().to_string(),
                false,
            )
        } else {
            // Try podman as fallback
            let podman_create = Command::new("podman")
                .args(["create", "--name", &container_name, image_name])
                .output()
                .context("Failed to create container with docker or podman")?;

            if !podman_create.status.success() {
                anyhow::bail!(
                    "Failed to create container for filesystem extraction: {}",
                    String::from_utf8_lossy(&podman_create.stderr)
                );
            }
            (
                String::from_utf8_lossy(&podman_create.stdout)
                    .trim()
                    .to_string(),
                true,
            )
        }
    } else {
        anyhow::bail!("Docker/Podman not available for filesystem extraction");
    };

    // Export the filesystem using the same tool that created the container
    let export_output = if use_podman {
        Command::new("podman")
            .args(["export", &container_id, "-o"])
            .arg(extract_dir.join("filesystem.tar"))
            .output()
    } else {
        Command::new("docker")
            .args(["export", &container_id, "-o"])
            .arg(extract_dir.join("filesystem.tar"))
            .output()
    };

    // Clean up the container using the correct tool
    if use_podman {
        let _ = Command::new("podman")
            .args(["rm", "-f", &container_id])
            .output();
    } else {
        let _ = Command::new("docker")
            .args(["rm", "-f", &container_id])
            .output();
    }

    // Check export result
    if export_output.as_ref().map_or(true, |o| !o.status.success()) {
        let stderr = export_output
            .as_ref()
            .map(|o| String::from_utf8_lossy(&o.stderr).to_string())
            .unwrap_or_else(|e| e.to_string());
        anyhow::bail!("Failed to export container filesystem: {}", stderr);
    }

    // Extract the tar
    let tar_path = extract_dir.join("filesystem.tar");
    let status = Command::new("tar")
        .args(["-xf"])
        .arg(&tar_path)
        .arg("-C")
        .arg(&extract_dir)
        .status()
        .context("Failed to extract filesystem tar")?;

    if !status.success() {
        anyhow::bail!("Failed to extract container filesystem");
    }

    // Clean up tar file
    let _ = std::fs::remove_file(tar_path);

    Ok(extract_dir)
}

/// Scan for JAR files in extracted container filesystem
///
/// This function uses BazBOM's polyglot scanner to detect and scan Java/Maven/Gradle
/// artifacts in the container filesystem. Results are merged with existing container
/// scan results with proper layer attribution.
#[allow(dead_code)]
async fn scan_container_jars(
    filesystem_dir: &Path,
    results: &mut ContainerScanResults,
) -> Result<usize> {
    tracing::info!(
        "Scanning for JAR files in container filesystem: {:?}",
        filesystem_dir
    );

    // Configure parallel orchestrator for Java ecosystem scanning
    let orchestrator_config = PolyglotOrchestratorConfig {
        max_concurrent: num_cpus::get(),
        show_progress: false,       // Don't show nested progress bars
        enable_reachability: false, // Reachability done separately
        enable_vulnerabilities: true,
    };

    let orchestrator = ParallelOrchestrator::with_config(orchestrator_config);

    // Scan the filesystem directory for all ecosystems
    let workspace_path = filesystem_dir
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("invalid filesystem path"))?;

    let polyglot_results = match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| {
            handle.block_on(orchestrator.scan_directory(workspace_path))
        })?,
        Err(_) => {
            // Create new runtime if not in async context
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(orchestrator.scan_directory(workspace_path))?
        }
    };

    // Filter to only Java/Maven/Gradle ecosystems
    let java_results: Vec<&EcosystemScanResult> = polyglot_results
        .iter()
        .filter(|r| {
            r.ecosystem == "Maven"
                || r.ecosystem == "Gradle"
                || r.ecosystem == "Maven (Bazel)"
                || r.ecosystem.contains("Java")
        })
        .collect();

    if java_results.is_empty() {
        tracing::info!("No Java artifacts found in container");
        return Ok(0);
    }

    let mut total_java_vulns = 0;

    // Merge Java vulnerabilities into container results
    for java_result in java_results {
        tracing::info!(
            "Found {} packages in {} with {} vulnerabilities",
            java_result.packages.len(),
            java_result.ecosystem,
            java_result.vulnerabilities.len()
        );

        for vuln in &java_result.vulnerabilities {
            // Convert scanner vulnerability to VulnerabilityInfo
            let vuln_info = VulnerabilityInfo {
                cve_id: vuln.id.clone(),
                package_name: vuln.package_name.clone(),
                installed_version: vuln.package_version.clone(),
                fixed_version: vuln.fixed_version.clone(),
                severity: vuln.severity.clone(),
                title: vuln.title.clone(),
                description: vuln.description.clone(),
                layer_digest: "Java Artifacts".to_string(), // Will be updated below
                published_date: vuln.published_date.clone(),
                epss_score: None, // Will be enriched later
                epss_percentile: None,
                is_kev: false, // Will be enriched later
                kev_due_date: None,
                cvss_score: vuln.cvss_score,
                priority: Some("P2".to_string()), // Default priority
                references: vuln.references.clone(),
                breaking_change: None,
                upgrade_path: None,
                is_reachable: false, // Will be analyzed later
                difficulty_score: None,
                call_chain: None,
                dependency_path: None,
            };

            // Determine which layer this JAR belongs to
            // For now, add to a special "Java Artifacts" layer
            // TODO: Map to actual Docker layer based on file location
            let layer_name = format!("Java Artifacts ({})", java_result.ecosystem);

            // Find or create the Java artifacts layer
            let layer_idx = results
                .layers
                .iter()
                .position(|l| l.digest.starts_with(&layer_name));

            if let Some(idx) = layer_idx {
                // Add to existing Java layer
                results.layers[idx].vulnerabilities.push(vuln_info);
            } else {
                // Create new layer for Java artifacts
                results.layers.push(LayerInfo {
                    digest: layer_name.clone(),
                    size_mb: 0.0, // Size calculated from package list
                    packages: java_result
                        .packages
                        .iter()
                        .map(|p| format!("{}:{}", p.name, p.version))
                        .collect(),
                    vulnerabilities: vec![vuln_info],
                });
            }

            total_java_vulns += 1;
        }
    }

    // Re-calculate vulnerability counts
    results.critical_count = 0;
    results.high_count = 0;
    results.medium_count = 0;
    results.low_count = 0;

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            match vuln.severity.to_uppercase().as_str() {
                "CRITICAL" => results.critical_count += 1,
                "HIGH" => results.high_count += 1,
                "MEDIUM" => results.medium_count += 1,
                "LOW" => results.low_count += 1,
                _ => {}
            }
        }
    }

    tracing::info!(
        "Added {} Java vulnerabilities to container scan results",
        total_java_vulns
    );
    Ok(total_java_vulns)
}

/// Reachability result with call chain information
struct ReachabilityResult {
    /// Whether the package is reachable
    reachable: bool,
    /// Call chain from entrypoint to vulnerable function
    call_chain: Option<Vec<String>>,
}

/// Perform reachability analysis on container vulnerabilities
async fn analyze_container_reachability(
    results: &mut ContainerScanResults,
    filesystem_dir: &Path,
) -> Result<()> {
    // Collect unique packages with vulnerabilities
    let mut packages_to_analyze: HashMap<String, Vec<String>> = HashMap::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            packages_to_analyze
                .entry(vuln.package_name.clone())
                .or_default()
                .push(vuln.cve_id.clone());
        }
    }

    if packages_to_analyze.is_empty() {
        return Ok(());
    }

    // Run polyglot reachability analysis on the extracted filesystem
    let reachability_results =
        run_polyglot_reachability(filesystem_dir, &packages_to_analyze).await?;

    // Update vulnerability reachability status and call chains
    for layer in &mut results.layers {
        for vuln in &mut layer.vulnerabilities {
            if let Some(result) = reachability_results.get(&vuln.package_name) {
                vuln.is_reachable = result.reachable;
                vuln.call_chain = result.call_chain.clone();
            }
        }
    }

    Ok(())
}

/// Run polyglot reachability analysis with full call graph analysis
async fn run_polyglot_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    let mut results: HashMap<String, ReachabilityResult> = HashMap::new();

    // Detect what languages are present in the container
    let ecosystems = bazbom_scanner::detect_ecosystems(project_path.to_str().unwrap_or("."))?;

    // Deduplicate by ecosystem type - we only need to run analysis once per language
    let mut seen_types = std::collections::HashSet::new();
    let unique_ecosystems: Vec<_> = ecosystems
        .into_iter()
        .filter(|e| seen_types.insert(std::mem::discriminant(&e.ecosystem_type)))
        .collect();

    // Run language-specific call graph analysis for each detected ecosystem
    for ecosystem in &unique_ecosystems {
        let ecosystem_results = match ecosystem.ecosystem_type {
            bazbom_scanner::EcosystemType::Npm => {
                analyze_npm_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Python => {
                analyze_python_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Go => {
                analyze_go_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Rust => {
                analyze_rust_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Ruby => {
                analyze_ruby_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Php => {
                analyze_php_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Maven | bazbom_scanner::EcosystemType::Gradle => {
                analyze_java_reachability(project_path, packages).await
            }
            bazbom_scanner::EcosystemType::Bazel => {
                // Bazel is polyglot - analyze based on detected languages in the workspace
                analyze_bazel_reachability(project_path, packages).await
            }
        };

        // Merge results from each ecosystem
        match ecosystem_results {
            Ok(ecosystem_map) => {
                for (pkg, result) in ecosystem_map {
                    // If already reachable from another ecosystem, keep that result
                    // Otherwise use this one
                    if !results.contains_key(&pkg) || result.reachable {
                        results.insert(pkg, result);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Call graph analysis failed for {}: {}", ecosystem.name, e);
                eprintln!(
                    "   WARN  Call graph analysis failed for {}: {}",
                    ecosystem.name, e
                );
            }
        }
    }

    Ok(results)
}

/// Analyze NPM package reachability using call graph
async fn analyze_npm_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::js::JsReachabilityAnalyzer;

    let mut analyzer = JsReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_path)?;
    let mut results = HashMap::new();

    // Check each package against the reachability report
    for package in packages.keys() {
        // Check if any functions from this package are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(&format!("node_modules/{}", package)));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Use the analyzer's call graph to find the call chain
            if let Some(vuln_result) = analyzer.check_vulnerability_reachability(
                package,
                func_id.split(':').last().unwrap_or(func_id),
            ) {
                (vuln_result.reachable, vuln_result.call_chain)
            } else {
                // Package is reachable but no specific function found
                // Build a simple call chain from entrypoints
                let mut chain = report.entrypoints.clone();
                if !chain.contains(func_id) {
                    chain.push(func_id.clone());
                }
                (true, if chain.is_empty() { None } else { Some(chain) })
            }
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Python package reachability using call graph
async fn analyze_python_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::python::PythonReachabilityAnalyzer;

    let mut analyzer = PythonReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_path)?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Python packages use underscores in imports
        let import_name = package.replace("-", "_");

        // Check if any functions from this package are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(&import_name));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Build call chain from entrypoints to the matched function
            let mut chain = report.entrypoints.clone();
            if !chain.contains(func_id) {
                chain.push(func_id.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Go package reachability using call graph
async fn analyze_go_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::go::analyze_go_project;

    let report = analyze_go_project(project_path)?;
    let mut results = HashMap::new();

    // Extract package names from reachable function IDs
    // Go function IDs are formatted as "package/path.FunctionName"
    let reachable_packages: std::collections::HashSet<String> = report
        .reachable_functions
        .iter()
        .filter_map(|func_id| {
            // Find the last dot to separate package from function name
            func_id.rfind('.').map(|idx| func_id[..idx].to_string())
        })
        .collect();

    // Check each vulnerable package against reachable packages
    for package in packages.keys() {
        let mut is_reachable = false;
        let mut matching_func: Option<String> = None;

        // Direct match
        if reachable_packages.contains(package) {
            is_reachable = true;
            matching_func = report
                .reachable_functions
                .iter()
                .find(|f| f.starts_with(package))
                .cloned();
        }
        // Check if package is a prefix of any reachable package (sub-packages)
        else if let Some(rp) = reachable_packages.iter().find(|rp| rp.starts_with(package)) {
            is_reachable = true;
            matching_func = report
                .reachable_functions
                .iter()
                .find(|f| f.starts_with(rp))
                .cloned();
        }
        // Check if any reachable package is a prefix (parent packages)
        else if let Some(rp) = reachable_packages
            .iter()
            .find(|rp| package.starts_with(*rp))
        {
            is_reachable = true;
            matching_func = report
                .reachable_functions
                .iter()
                .find(|f| f.starts_with(rp))
                .cloned();
        }

        // For Go, we build a simple call chain from entrypoints to the matched function
        let call_chain = if is_reachable {
            let mut chain = Vec::new();
            for ep in &report.entrypoints {
                chain.push(ep.clone());
            }
            if let Some(func) = matching_func {
                if !chain.contains(&func) {
                    chain.push(func);
                }
            }
            if chain.is_empty() {
                None
            } else {
                Some(chain)
            }
        } else {
            None
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Rust package reachability using call graph
async fn analyze_rust_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::rust::RustReachabilityAnalyzer;

    let mut analyzer = RustReachabilityAnalyzer::new(project_path.to_path_buf());
    let report = analyzer.analyze()?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Rust crates use underscores in module names
        let crate_name = package.replace("-", "_");

        // Check if any functions from this crate are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(&crate_name));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Build call chain from entrypoints to the matched function
            let mut chain = report.entrypoints.clone();
            if !chain.contains(func_id) {
                chain.push(func_id.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Ruby package reachability using call graph
async fn analyze_ruby_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::ruby::RubyReachabilityAnalyzer;

    let mut analyzer = RubyReachabilityAnalyzer::new(project_path.to_path_buf());
    let report = analyzer.analyze()?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Check if any functions from this gem are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(package));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Build call chain from entrypoints to the matched function
            let mut chain = report.entrypoints.clone();
            if !chain.contains(func_id) {
                chain.push(func_id.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze PHP package reachability using call graph
async fn analyze_php_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::php::PhpReachabilityAnalyzer;

    let mut analyzer = PhpReachabilityAnalyzer::new(project_path.to_path_buf());
    let report = analyzer.analyze()?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Check if any functions from this package are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(package));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Build call chain from entrypoints to the matched function
            let mut chain = report.entrypoints.clone();
            if !chain.contains(func_id) {
                chain.push(func_id.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Java/Maven/Gradle package reachability using call graph
async fn analyze_java_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::java::JavaReachabilityAnalyzer;

    let mut analyzer = JavaReachabilityAnalyzer::new();
    let report = analyzer.analyze(project_path)?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Java packages are in format "groupId:artifactId"
        // Convert to path format for matching
        let group_path = if let Some((group_id, _artifact_id)) = package.split_once(':') {
            group_id.replace('.', "/")
        } else {
            package.replace('.', "/")
        };

        // Check if any functions from this package are reachable
        let matching_func = report
            .reachable_functions
            .iter()
            .find(|func_id| func_id.contains(&group_path));

        let (is_reachable, call_chain) = if let Some(func_id) = matching_func {
            // Build call chain from entrypoints to the matched function
            let mut chain = report.entrypoints.clone();
            if !chain.contains(func_id) {
                chain.push(func_id.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Analyze Bazel workspace reachability (polyglot - analyzes all detected languages)
async fn analyze_bazel_reachability(
    project_path: &Path,
    packages: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ReachabilityResult>> {
    use bazbom_reachability::bazel::analyze_bazel_project;

    let report = analyze_bazel_project(project_path)?;
    let mut results = HashMap::new();

    for package in packages.keys() {
        // Bazel packages can be from any language
        // Check if any targets containing this package are reachable
        let matching_target = report.reachable_targets.iter().find(|target| {
            // Try various matching strategies for polyglot support
            // Bazel targets look like //path/to:target or @repo//path:target
            target.contains(package)
                || target.contains(&package.replace("-", "_"))
                || target.contains(&package.replace('.', "/"))
        });

        let (is_reachable, call_chain) = if let Some(target) = matching_target {
            // Build call chain from entrypoints to the matched target
            let mut chain = report.entrypoints.clone();
            if !chain.contains(target) {
                chain.push(target.clone());
            }
            (true, if chain.is_empty() { None } else { Some(chain) })
        } else {
            (false, None)
        };

        results.insert(
            package.clone(),
            ReachabilityResult {
                reachable: is_reachable,
                call_chain,
            },
        );
    }

    Ok(results)
}

/// Security score and grade information
struct SecurityScore {
    score: u32,
    grade: &'static str,
    grade_color: &'static str,
    grade_desc: &'static str,
}

/// Calculate security score from vulnerability counts
fn calculate_security_score(results: &ContainerScanResults) -> SecurityScore {
    let mut score = 100i32;
    score -= (results.critical_count as i32 * 10).min(100);
    score -= (results.high_count as i32 * 2).min(40);
    score -= (results.medium_count as i32).min(50);
    let score = score.max(0) as u32;

    let (grade, grade_color, grade_desc) = match score {
        90..=100 => ("A", "#27ae60", "Excellent - minimal risk"),
        80..=89 => ("B", "#2ecc71", "Good - low risk"),
        70..=79 => ("C", "#f39c12", "Fair - moderate risk"),
        60..=69 => ("D", "#e67e22", "Poor - high risk"),
        _ => ("F", "#e74c3c", "Critical - immediate action required"),
    };

    SecurityScore {
        score,
        grade,
        grade_color,
        grade_desc,
    }
}

/// Generate HTML for layer attribution section
fn generate_layers_html(results: &ContainerScanResults) -> String {
    let mut layers_html = String::new();
    for (i, layer) in results.layers.iter().enumerate() {
        let vuln_count = layer.vulnerabilities.len();
        let critical = layer
            .vulnerabilities
            .iter()
            .filter(|v| v.severity == "CRITICAL")
            .count();
        let high = layer
            .vulnerabilities
            .iter()
            .filter(|v| v.severity == "HIGH")
            .count();
        let medium = layer
            .vulnerabilities
            .iter()
            .filter(|v| v.severity == "MEDIUM")
            .count();
        let low = layer
            .vulnerabilities
            .iter()
            .filter(|v| v.severity == "LOW")
            .count();

        let status_class = if critical > 0 {
            "critical"
        } else if high > 0 {
            "high"
        } else if vuln_count > 0 {
            "medium"
        } else {
            "clean"
        };

        layers_html.push_str(&format!(
            r#"<div class="layer {status_class}">
                <div class="layer-header">
                    <span class="layer-num">Layer {}</span>
                    <span class="layer-digest">{}</span>
                </div>
                <div class="layer-stats">
                    <span>Size: {:.1} MB</span>
                    <span>Packages: {}</span>
                    <span class="vuln-badge">{} vulns</span>
                </div>
                {}</div>"#,
            i + 1,
            &layer.digest[..16.min(layer.digest.len())],
            layer.size_mb,
            layer.packages.len(),
            vuln_count,
            if vuln_count > 0 {
                format!(
                    "<div class='layer-vulns'>{}C / {}H / {}M / {}L</div>",
                    critical, high, medium, low
                )
            } else {
                "<div class='layer-clean'>OK Clean</div>".to_string()
            }
        ));
    }
    layers_html
}

/// Generate HTML for quick wins section
fn generate_quick_wins_html(all_vulns: &[&VulnerabilityInfo]) -> String {
    let mut quick_wins_html = String::new();
    let mut seen_packages: std::collections::HashSet<String> = std::collections::HashSet::new();

    for vuln in all_vulns
        .iter()
        .filter(|v| v.fixed_version.is_some() && v.breaking_change != Some(true))
    {
        if seen_packages.contains(&vuln.package_name) {
            continue;
        }
        seen_packages.insert(vuln.package_name.clone());

        // Count how many vulns this fix addresses
        let fixes_count = all_vulns
            .iter()
            .filter(|v| v.package_name == vuln.package_name && v.fixed_version.is_some())
            .count();

        if seen_packages.len() <= 8 {
            quick_wins_html.push_str(&format!(
                r#"<div class="quick-win">
                    <div class="qw-package">{}</div>
                    <div class="qw-version">{} → {}</div>
                    <div class="qw-impact">Fixes {} vulnerabilities</div>
                </div>"#,
                vuln.package_name,
                vuln.installed_version,
                vuln.fixed_version.as_deref().unwrap_or("unknown"),
                fixes_count
            ));
        }
    }
    quick_wins_html
}

/// Generate HTML for vulnerabilities list
fn generate_vulnerabilities_html(all_vulns: &[&VulnerabilityInfo]) -> String {
    let mut vulns_html = String::new();

    // Sort by priority
    let mut all_vulns_sorted: Vec<&VulnerabilityInfo> = all_vulns.to_vec();
    all_vulns_sorted.sort_by(|a, b| {
        let priority_order = |p: &Option<String>| match p.as_deref() {
            Some("P0") => 0,
            Some("P1") => 1,
            Some("P2") => 2,
            Some("P3") => 3,
            Some("P4") => 4,
            _ => 5,
        };
        priority_order(&a.priority).cmp(&priority_order(&b.priority))
    });

    // Group by severity for collapsible sections
    let critical: Vec<_> = all_vulns_sorted
        .iter()
        .filter(|v| v.severity == "CRITICAL")
        .collect();
    let high: Vec<_> = all_vulns_sorted
        .iter()
        .filter(|v| v.severity == "HIGH")
        .collect();
    let medium: Vec<_> = all_vulns_sorted
        .iter()
        .filter(|v| v.severity == "MEDIUM")
        .collect();
    let low: Vec<_> = all_vulns_sorted
        .iter()
        .filter(|v| v.severity == "LOW")
        .collect();

    // Add summary table
    vulns_html.push_str(r#"
        <div class="vuln-summary-table">
            <table style="width: 100%; border-collapse: collapse; margin-bottom: 20px;">
                <tr style="background: #f8f9fa;">
                    <th style="padding: 10px; text-align: left; border-bottom: 2px solid #dee2e6;">Severity</th>
                    <th style="padding: 10px; text-align: center; border-bottom: 2px solid #dee2e6;">Count</th>
                    <th style="padding: 10px; text-align: center; border-bottom: 2px solid #dee2e6;">KEV</th>
                    <th style="padding: 10px; text-align: center; border-bottom: 2px solid #dee2e6;">Reachable</th>
                    <th style="padding: 10px; text-align: center; border-bottom: 2px solid #dee2e6;">Fixable</th>
                </tr>
    "#);

    for (name, vulns, color) in [
        ("CRITICAL", &critical, "#e74c3c"),
        ("HIGH", &high, "#f39c12"),
        ("MEDIUM", &medium, "#3498db"),
        ("LOW", &low, "#95a5a6"),
    ] {
        let kev_count = vulns.iter().filter(|v| v.is_kev).count();
        let reachable_count = vulns.iter().filter(|v| v.is_reachable).count();
        let fixable_count = vulns.iter().filter(|v| v.fixed_version.is_some()).count();
        vulns_html.push_str(&format!(
            r#"<tr>
                <td style="padding: 10px; border-bottom: 1px solid #dee2e6; color: {}; font-weight: bold;">{}</td>
                <td style="padding: 10px; text-align: center; border-bottom: 1px solid #dee2e6;">{}</td>
                <td style="padding: 10px; text-align: center; border-bottom: 1px solid #dee2e6;">{}</td>
                <td style="padding: 10px; text-align: center; border-bottom: 1px solid #dee2e6;">{}</td>
                <td style="padding: 10px; text-align: center; border-bottom: 1px solid #dee2e6;">{}</td>
            </tr>"#,
            color, name, vulns.len(), kev_count, reachable_count, fixable_count
        ));
    }
    vulns_html.push_str("</table></div>");

    // Generate collapsible sections - show only top items by default
    let max_shown = 20; // Show top 20 in each severity

    for (severity_name, vulns, severity_class) in [
        ("Critical", critical, "critical"),
        ("High", high, "high"),
        ("Medium", medium, "medium"),
        ("Low", low, "low"),
    ] {
        if vulns.is_empty() {
            continue;
        }

        let total = vulns.len();
        let collapsed_class = if severity_class == "critical" || severity_class == "high" {
            ""
        } else {
            "collapsed"
        };

        vulns_html.push_str(&format!(
            r#"<details class="severity-section {}" {}><summary style="cursor: pointer; padding: 15px; background: #f8f9fa; border-radius: 8px; margin-bottom: 10px; font-weight: bold;">
                <span class="{}">{} ({} vulnerabilities)</span>
            </summary><div class="severity-vulns">"#,
            severity_class,
            if collapsed_class.is_empty() { "open" } else { "" },
            severity_class,
            severity_name,
            total
        ));

        // Show limited vulns with "show more" capability
        for (i, vuln) in vulns.iter().enumerate() {
            let hidden = if i >= max_shown {
                r#"style="display:none" class="hidden-vuln""#
            } else {
                ""
            };
            vulns_html.push_str(&format_single_vuln_html(vuln, hidden));
        }

        // Add "show more" button if needed
        if total > max_shown {
            vulns_html.push_str(&format!(
                r#"<button onclick="this.parentElement.querySelectorAll('.hidden-vuln').forEach(e => e.style.display='block'); this.style.display='none';"
                    style="width: 100%; padding: 10px; background: #f8f9fa; border: 1px solid #dee2e6; border-radius: 8px; cursor: pointer; margin-top: 10px;">
                    Show {} more {} vulnerabilities
                </button>"#,
                total - max_shown, severity_name.to_lowercase()
            ));
        }

        vulns_html.push_str("</div></details>");
    }

    // Add Jira ticket templates section for Critical and High vulns
    let actionable_vulns: Vec<_> = all_vulns_sorted
        .iter()
        .filter(|v| v.severity == "CRITICAL" || v.severity == "HIGH")
        .collect();

    if !actionable_vulns.is_empty() {
        vulns_html.push_str(&generate_jira_templates_html(&actionable_vulns));
    }

    vulns_html
}

/// Generate Jira ticket templates for Critical and High vulnerabilities
fn generate_jira_templates_html(vulns: &[&&VulnerabilityInfo]) -> String {
    let mut html = String::new();

    html.push_str(r#"
        <details class="jira-templates-section" style="margin-top: 30px;">
            <summary style="cursor: pointer; padding: 15px; background: #2c3e50; color: white; border-radius: 8px; margin-bottom: 10px; font-weight: bold;">
                NOTE Jira Ticket Templates (Copy & Paste)
            </summary>
            <div style="padding: 10px;">
    "#);

    for vuln in vulns {
        let priority = match vuln.severity.as_str() {
            "CRITICAL" => "P1 - Critical",
            "HIGH" => "P2 - High",
            _ => "P3 - Major",
        };

        let epss_note = vuln
            .epss_score
            .map(|s| {
                if s >= 0.2 {
                    format!("EPSS: {:.1}% (HIGH)", s * 100.0)
                } else {
                    format!("EPSS: {:.1}%", s * 100.0)
                }
            })
            .unwrap_or_else(|| "No EPSS data".to_string());

        let kev_note = if vuln.is_kev {
            format!(
                "WARN CISA KEV - Due: {}",
                vuln.kev_due_date.as_ref().unwrap_or(&"TBD".to_string())
            )
        } else {
            "Not in KEV".to_string()
        };

        // Triage Ticket
        html.push_str(&format!(r#"
            <div style="background: #fff3cd; border-left: 4px solid #f39c12; padding: 15px; margin-bottom: 15px; border-radius: 4px;">
                <h4 style="margin: 0 0 10px 0; color: #856404;">SCAN Triage Ticket: {}</h4>
                <pre style="background: white; padding: 10px; border-radius: 4px; overflow-x: auto; font-size: 11px; white-space: pre-wrap;">
<strong>Title:</strong> [TRIAGE] {} - {}
<strong>Type:</strong> Task
<strong>Team:</strong> Security
<strong>Labels:</strong> security-vulnerabilities

<strong>Description:</strong>
Vulnerable Component:
- Package: {} ({})
- CVE: {}
- Severity: {}

Vulnerability Details:
- {}: {}
- KEV Status: {}
- {}
- Fixed Version: {}

Evidence / Reported Data:
- Scan Source: BazBOM Container Scanner
- Reachability: {}

Mitigation / Remediation Notes:
- Upgrade {} to {}
- Review call path if reachable
                </pre>
                <button onclick="navigator.clipboard.writeText(this.previousElementSibling.innerText)"
                    style="background: #f39c12; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer;">
                    Copy Triage Ticket
                </button>
            </div>
        "#,
            vuln.cve_id,
            vuln.cve_id,
            vuln.package_name,
            vuln.package_name,
            vuln.installed_version,
            vuln.cve_id,
            vuln.severity,
            vuln.severity,
            &vuln.description.chars().take(100).collect::<String>(),
            kev_note,
            epss_note,
            vuln.fixed_version.as_ref().unwrap_or(&"No fix available".to_string()),
            if vuln.is_reachable { "REACHABLE - Exploitable path exists" } else { "Unreachable - No call path detected" },
            vuln.package_name,
            vuln.fixed_version.as_ref().unwrap_or(&"latest stable".to_string())
        ));

        // Remediation Ticket
        html.push_str(&format!(r#"
            <div style="background: #d1ecf1; border-left: 4px solid #17a2b8; padding: 15px; margin-bottom: 25px; border-radius: 4px;">
                <h4 style="margin: 0 0 10px 0; color: #0c5460;">TOOL Remediation Ticket: {}</h4>
                <pre style="background: white; padding: 10px; border-radius: 4px; overflow-x: auto; font-size: 11px; white-space: pre-wrap;">
<strong>Title:</strong> {} - {} upgrade required
<strong>Type:</strong> Bug
<strong>Priority:</strong> {}
<strong>Labels:</strong> security-vulnerabilities

<strong>Description:</strong>
Summary:
- Address {} in {} package

Risk Context:
- Severity: {}
- Priority: {}
- CVSS: {}
- {}
- {}

Required Actions:
1. Upgrade {} from {} to {}
2. Run security scan to verify fix
3. Update container image and redeploy

SLA Reminder:
- Follow remediation timeline for {} vulnerabilities
- {} vulnerabilities require immediate attention
                </pre>
                <button onclick="navigator.clipboard.writeText(this.previousElementSibling.innerText)"
                    style="background: #17a2b8; color: white; border: none; padding: 5px 10px; border-radius: 4px; cursor: pointer;">
                    Copy Remediation Ticket
                </button>
            </div>
        "#,
            vuln.cve_id,
            vuln.cve_id,
            vuln.package_name,
            priority,
            vuln.cve_id,
            vuln.package_name,
            vuln.severity,
            priority,
            vuln.cvss_score.map(|s| format!("{:.1}", s)).unwrap_or_else(|| "N/A".to_string()),
            epss_note,
            kev_note,
            vuln.package_name,
            vuln.installed_version,
            vuln.fixed_version.as_ref().unwrap_or(&"latest stable".to_string()),
            vuln.severity,
            if vuln.is_kev { "KEV-listed" } else { "High-severity" }
        ));
    }

    html.push_str("</div></details>");
    html
}

/// Generate Jira ticket markdown files for a vulnerability
fn generate_jira_ticket_files(
    output_dir: &std::path::Path,
    vuln: &VulnerabilityInfo,
    image_name: &str,
) -> Result<()> {
    let safe_cve = vuln.cve_id.replace(':', "_").replace('/', "_");

    let priority = match vuln.severity.as_str() {
        "CRITICAL" => "P1 - Critical",
        "HIGH" => "P2 - High",
        _ => "P3 - Major",
    };

    let epss_note = vuln
        .epss_score
        .map(|s| {
            if s >= 0.2 {
                format!("EPSS: {:.1}% (HIGH)", s * 100.0)
            } else {
                format!("EPSS: {:.1}%", s * 100.0)
            }
        })
        .unwrap_or_else(|| "No EPSS data".to_string());

    let kev_note = if vuln.is_kev {
        format!(
            "WARN CISA KEV - Due: {}",
            vuln.kev_due_date.as_ref().unwrap_or(&"TBD".to_string())
        )
    } else {
        "Not in KEV".to_string()
    };

    // Triage ticket
    let triage_content = format!(
        r#"# [TRIAGE] {} - {}

**Type:** Task
**Team:** Security
**Labels:** security-vulnerabilities

## Description

### Vulnerable Component
- **Image:** {}
- **Package:** {} ({})
- **CVE:** {}
- **Severity:** {}

### Vulnerability Details
- **{}:** {}
- **KEV Status:** {}
- **{}**
- **Fixed Version:** {}

### Evidence / Reported Data
- Scan Source: BazBOM Container Scanner
- Reachability: {}

### Mitigation / Remediation Notes
- Upgrade {} to {}
- Review call path if reachable
"#,
        vuln.cve_id,
        vuln.package_name,
        image_name,
        vuln.package_name,
        vuln.installed_version,
        vuln.cve_id,
        vuln.severity,
        vuln.severity,
        &vuln.description.chars().take(200).collect::<String>(),
        kev_note,
        epss_note,
        vuln.fixed_version
            .as_ref()
            .unwrap_or(&"No fix available".to_string()),
        if vuln.is_reachable {
            "REACHABLE - Exploitable path exists"
        } else {
            "Unreachable - No call path detected"
        },
        vuln.package_name,
        vuln.fixed_version
            .as_ref()
            .unwrap_or(&"latest stable".to_string())
    );

    let triage_path = output_dir.join(format!("{}-TRIAGE.md", safe_cve));
    std::fs::write(&triage_path, triage_content)?;

    // Remediation ticket
    let remediation_content = format!(
        r#"# {} - {} upgrade required

**Type:** Bug
**Priority:** {}
**Labels:** security-vulnerabilities

## Summary
Address {} in {} package

## Risk Context
- **Severity:** {}
- **Priority:** {}
- **CVSS:** {}
- **{}**
- **{}**

## Required Actions
1. Upgrade {} from {} to {}
2. Run security scan to verify fix
3. Update container image and redeploy

## SLA Reminder
- Follow remediation timeline for {} vulnerabilities
- {} vulnerabilities require immediate attention
"#,
        vuln.cve_id,
        vuln.package_name,
        priority,
        vuln.cve_id,
        vuln.package_name,
        vuln.severity,
        priority,
        vuln.cvss_score
            .map(|s| format!("{:.1}", s))
            .unwrap_or_else(|| "N/A".to_string()),
        epss_note,
        kev_note,
        vuln.package_name,
        vuln.installed_version,
        vuln.fixed_version
            .as_ref()
            .unwrap_or(&"latest stable".to_string()),
        vuln.severity,
        if vuln.is_kev {
            "KEV-listed"
        } else {
            "High-severity"
        }
    );

    let remediation_path = output_dir.join(format!("{}-REMEDIATION.md", safe_cve));
    std::fs::write(&remediation_path, remediation_content)?;

    Ok(())
}

/// Format a single vulnerability as HTML
fn format_single_vuln_html(vuln: &VulnerabilityInfo, extra_attrs: &str) -> String {
    let severity_class = vuln.severity.to_lowercase();
    let nvd_link = format!("https://nvd.nist.gov/vuln/detail/{}", vuln.cve_id);
    let epss_info = vuln
        .epss_score
        .map(|s| format!("EPSS: {:.1}%", s * 100.0))
        .unwrap_or_default();
    let reachable_badge = if vuln.is_reachable {
        r#"<span class="reachable-badge">TARGET REACHABLE</span>"#
    } else {
        r#"<span class="unreachable-badge">SHIELD Unreachable</span>"#
    };

    let mut html = format!(
        r#"<div class="vuln-item {severity_class}" {extra_attrs}>
            <div class="vuln-header">
                <a href="{}" target="_blank" class="cve-id">{}</a>
                <span class="priority-badge">{}</span>
                <span class="severity-badge {severity_class}">{}</span>
                {}
            </div>
            <div class="vuln-package"><strong>Package:</strong> {} ({}) → {}</div>
            <div class="vuln-desc">{}</div>
            <div class="vuln-meta">
                {}
                {}
                {}
            </div>
            {}"#,
        nvd_link,
        vuln.cve_id,
        vuln.priority.as_ref().unwrap_or(&"P2".to_string()),
        vuln.severity,
        reachable_badge,
        vuln.package_name,
        vuln.installed_version,
        vuln.fixed_version.as_ref().unwrap_or(&"No fix".to_string()),
        vuln.description.chars().take(200).collect::<String>(),
        if !epss_info.is_empty() {
            format!("<span class='epss-score'>{}</span>", epss_info)
        } else {
            String::new()
        },
        vuln.cvss_score
            .map(|s| format!("<span class='cvss-score'>CVSS: {:.1}</span>", s))
            .unwrap_or_default(),
        if vuln.is_kev {
            format!("<a href='https://www.cisa.gov/known-exploited-vulnerabilities-catalog' target='_blank' class='kev-badge'>🚨 CISA KEV{}</a>",
                vuln.kev_due_date.as_ref().map(|d| format!(" (due: {})", d)).unwrap_or_default())
        } else {
            String::new()
        },
        vuln.difficulty_score
            .map(|d| {
                let label = if d <= 20 {
                    "Trivial"
                } else if d <= 40 {
                    "Easy"
                } else if d <= 60 {
                    "Moderate"
                } else {
                    "Hard"
                };
                format!(
                    "<div class='difficulty'>Difficulty: {}/100 ({})</div>",
                    d, label
                )
            })
            .unwrap_or_default()
    );

    // Add call chain if reachable (limit to 5 items for readability)
    if vuln.is_reachable {
        if let Some(chain) = &vuln.call_chain {
            if !chain.is_empty() {
                let limited_chain: Vec<_> = chain.iter().take(5).collect();
                let chain_html = limited_chain
                    .iter()
                    .map(|f| {
                        // Shorten long paths
                        let short = f.split('/').last().unwrap_or(f);
                        format!("<code>{}</code>", short)
                    })
                    .collect::<Vec<_>>()
                    .join(" → ");
                let more_indicator = if chain.len() > 5 {
                    format!(
                        " <span style='color:#7f8c8d'>+{} more</span>",
                        chain.len() - 5
                    )
                } else {
                    String::new()
                };
                html.push_str(&format!(
                    r#"<div class="call-chain"><strong>Call Path:</strong> {}{}</div>"#,
                    chain_html, more_indicator
                ));
            }
        }
    }

    html.push_str("</div>");
    html
}

/// Generate HTML for upgrade recommendations
fn generate_upgrades_html(results: &ContainerScanResults) -> String {
    if results.upgrade_recommendations.is_empty() {
        return String::new();
    }

    let mut upgrades_html = String::new();

    upgrades_html.push_str(r#"
        <h2 style="color: #2c3e50; margin-top: 40px; border-bottom: 2px solid #3498db; padding-bottom: 10px;">
            TARGET Upgrade Intelligence
        </h2>
        <p style="color: #7f8c8d; margin-bottom: 20px;">
            AI-powered analysis showing upgrade effort, breaking changes, and transitive impact for each vulnerable package
        </p>
    "#);

    for rec in &results.upgrade_recommendations {
        let _risk_class = match rec.risk_level.as_str() {
            "LOW" => "low-risk",
            "MEDIUM" => "medium-risk",
            "HIGH" => "high-risk",
            "CRITICAL" => "critical-risk",
            _ => "medium-risk",
        };

        let risk_color = match rec.risk_level.as_str() {
            "LOW" => "#27ae60",
            "MEDIUM" => "#f39c12",
            "HIGH" => "#e67e22",
            "CRITICAL" => "#e74c3c",
            _ => "#95a5a6",
        };

        let effort_display = rec
            .effort_hours
            .map(|h| {
                if h < 1.0 {
                    format!("{} min", (h * 60.0) as u32)
                } else if h == 1.0 {
                    "1 hour".to_string()
                } else {
                    format!("{:.1} hours", h)
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let breaking_info = if let Some(count) = rec.breaking_changes_count {
            if count > 0 {
                format!(
                    r#"<div style="color: #e74c3c; font-weight: bold; margin-top: 8px;">
                    WARN {} breaking change{}</div>"#,
                    count,
                    if count == 1 { "" } else { "s" }
                )
            } else {
                r#"<div style="color: #27ae60; margin-top: 8px;">OK No breaking changes</div>"#
                    .to_string()
            }
        } else {
            String::new()
        };

        let transitive_info = if let Some(count) = rec.transitive_upgrades_count {
            if count > 0 {
                format!(
                    r#"<div style="color: #f39c12; margin-top: 5px;">
                    LINK Requires {} transitive upgrade{}</div>"#,
                    count,
                    if count == 1 { "" } else { "s" }
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let migration_guide = if let Some(ref url) = rec.migration_guide_url {
            format!(
                r#"<div style="margin-top: 8px;">
                <a href="{}" target="_blank" style="color: #3498db; text-decoration: none;">
                    📖 Migration Guide
                </a>
            </div>"#,
                url
            )
        } else {
            String::new()
        };

        let success_rate = if let Some(rate) = rec.success_rate {
            format!(
                r#"<div style="margin-top: 5px; color: #7f8c8d; font-size: 12px;">
                📊 {:.0}% success rate in community
            </div>"#,
                rate * 100.0
            )
        } else {
            String::new()
        };

        upgrades_html.push_str(&format!(
            r#"<div style="background: white; border-left: 4px solid {risk_color}; padding: 15px; margin-bottom: 15px; border-radius: 4px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
                <div style="font-weight: bold; font-size: 16px; color: #2c3e50; margin-bottom: 8px;">
                    {package}
                </div>
                <div style="color: #7f8c8d; font-size: 14px; margin-bottom: 8px;">
                    {installed} → {recommended}
                </div>
                <div style="display: flex; gap: 20px; flex-wrap: wrap; margin-top: 12px;">
                    <div>
                        <span style="color: #7f8c8d; font-size: 12px;">EFFORT:</span>
                        <span style="color: #2c3e50; font-weight: bold; margin-left: 5px;">{effort}</span>
                    </div>
                    <div>
                        <span style="color: #7f8c8d; font-size: 12px;">RISK:</span>
                        <span style="color: {risk_color}; font-weight: bold; margin-left: 5px;">{risk}</span>
                    </div>
                    <div>
                        <span style="color: #7f8c8d; font-size: 12px;">FIXES:</span>
                        <span style="color: #2c3e50; margin-left: 5px;">{cve_count} CVE{plural}</span>
                    </div>
                </div>
                {breaking}
                {transitive}
                {migration}
                {success}
            </div>"#,
            risk_color = risk_color,
            package = rec.package,
            installed = rec.installed_version,
            recommended = rec.recommended_version.as_ref().unwrap_or(&"N/A".to_string()),
            effort = effort_display,
            risk = rec.risk_level,
            cve_count = rec.fixes_cves.len(),
            plural = if rec.fixes_cves.len() == 1 { "" } else { "s" },
            breaking = breaking_info,
            transitive = transitive_info,
            migration = migration_guide,
            success = success_rate
        ));
    }

    upgrades_html
}

/// Generate executive report
/// Generate HTML for CISA KEV alert banner
fn generate_kev_alert_html(all_vulns: &[&VulnerabilityInfo]) -> String {
    let kev_vulns: Vec<&VulnerabilityInfo> =
        all_vulns.iter().filter(|v| v.is_kev).copied().collect();

    if kev_vulns.is_empty() {
        return String::new();
    }

    let mut html = String::from(
        r#"
    <div style="background: linear-gradient(135deg, #c0392b 0%, #e74c3c 100%); color: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 4px 6px rgba(231, 76, 60, 0.3);">
        <div style="display: flex; align-items: center; margin-bottom: 15px;">
            <div style="font-size: 48px; margin-right: 20px;">🚨</div>
            <div>
                <h2 style="margin: 0; font-size: 24px;">CISA KEV Alert</h2>
                <p style="margin: 5px 0 0 0; opacity: 0.95; font-size: 14px;">Known Exploited Vulnerabilities - Immediate Action Required</p>
            </div>
        </div>
    "#,
    );

    // Deduplicate by CVE ID
    let mut seen_cves = std::collections::HashSet::new();
    for vuln in &kev_vulns {
        if seen_cves.contains(&vuln.cve_id) {
            continue;
        }
        seen_cves.insert(&vuln.cve_id);

        html.push_str(&format!(
            r#"
        <div style="background: rgba(255,255,255,0.15); padding: 15px; border-radius: 6px; margin-bottom: 10px;">
            <div style="display: flex; justify-content: space-between; align-items: start;">
                <div style="flex: 1;">
                    <div style="font-size: 18px; font-weight: bold; margin-bottom: 8px;">
                        <a href="https://nvd.nist.gov/vuln/detail/{cve}" target="_blank" style="color: white; text-decoration: none;">{cve}</a>
                        <a href="https://www.cisa.gov/known-exploited-vulnerabilities-catalog" target="_blank" style="margin-left: 10px; font-size: 12px; opacity: 0.9; color: #ffe6e6;">→ CISA Catalog</a>
                    </div>
                    <div style="font-size: 14px; margin-bottom: 8px;"><strong>Package:</strong> {package}</div>
                    <div style="font-size: 13px; opacity: 0.95;">{description}</div>
                </div>
                <div style="text-align: right; margin-left: 20px;">
                    <div style="background: rgba(255,255,255,0.25); padding: 8px 12px; border-radius: 4px; font-size: 13px; font-weight: bold; margin-bottom: 8px;">
                        Due: {due_date}
                    </div>
                    <div style="font-size: 12px; opacity: 0.9;">Severity: {severity}</div>
                </div>
            </div>
            {fixed_version}
        </div>
        "#,
            cve = vuln.cve_id,
            package = vuln.package_name,
            description = vuln.description.chars().take(200).collect::<String>(),
            due_date = vuln.kev_due_date.as_deref().unwrap_or("Unknown"),
            severity = vuln.severity,
            fixed_version = if let Some(ref fix) = vuln.fixed_version {
                format!(r#"<div style="margin-top: 10px; padding: 8px 12px; background: rgba(255,255,255,0.2); border-radius: 4px; font-size: 13px;">
                    <strong>Fix Available:</strong> Upgrade to version {}</div>"#, fix)
            } else {
                String::new()
            }
        ));
    }

    html.push_str("    </div>");
    html
}

fn generate_executive_report(results: &ContainerScanResults, report_file: &str) -> Result<()> {
    // Calculate security metrics
    let security = calculate_security_score(results);

    // Collect all vulnerabilities
    let all_vulns: Vec<&VulnerabilityInfo> = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .collect();

    // Count reachable vs unreachable
    let reachable_count = all_vulns.iter().filter(|v| v.is_reachable).count();
    let unreachable_count = all_vulns.len() - reachable_count;

    // Generate HTML sections
    let kev_alert_html = generate_kev_alert_html(&all_vulns);
    let layers_html = generate_layers_html(results);
    let quick_wins_html = generate_quick_wins_html(&all_vulns);
    let _vulns_html = generate_vulnerabilities_html(&all_vulns);
    let _upgrades_html = generate_upgrades_html(results);

    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Container Security Report - {image_name}</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 1400px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
            background: #f5f6fa;
            color: #2c3e50;
        }}
        .header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 15px 20px;
            border-radius: 8px;
            margin-bottom: 20px;
        }}
        .header h1 {{ margin: 0 0 5px 0; font-size: 20px; }}
        .header p {{ margin: 3px 0; opacity: 0.9; font-size: 13px; }}

        .score-section {{
            display: grid;
            grid-template-columns: 200px 1fr;
            gap: 30px;
            margin-bottom: 30px;
        }}
        .score-card {{
            background: white;
            border-radius: 12px;
            padding: 30px;
            text-align: center;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .score-value {{
            font-size: 72px;
            font-weight: bold;
            color: {grade_color};
        }}
        .score-grade {{
            font-size: 24px;
            font-weight: bold;
            color: {grade_color};
            margin-top: 5px;
        }}
        .score-desc {{
            font-size: 12px;
            color: #7f8c8d;
            margin-top: 8px;
        }}
        .reachable-badge {{
            background: #e74c3c;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
        }}
        .unreachable-badge {{
            background: #27ae60;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 11px;
        }}
        .vuln-meta {{
            display: flex;
            gap: 15px;
            margin-top: 10px;
            font-size: 12px;
        }}
        .epss-score, .cvss-score {{
            background: #f8f9fa;
            padding: 2px 8px;
            border-radius: 4px;
        }}
        .difficulty {{
            margin-top: 8px;
            font-size: 12px;
            color: #7f8c8d;
        }}
        .call-chain {{
            margin-top: 8px;
            padding: 8px 12px;
            background: #e8f4fd;
            border-left: 3px solid #3498db;
            font-size: 11px;
            border-radius: 4px;
            overflow-x: auto;
        }}
        .call-chain code {{
            background: #fff;
            padding: 2px 6px;
            border-radius: 3px;
            font-family: 'Courier New', monospace;
            white-space: nowrap;
        }}
        .cve-id {{
            color: inherit;
            text-decoration: none;
        }}
        .cve-id:hover {{
            text-decoration: underline;
        }}
        .reachability-summary {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 20px;
            margin-bottom: 20px;
        }}
        .reach-card {{
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        .reach-card.reachable {{
            background: #fdf2f2;
            border: 2px solid #e74c3c;
        }}
        .reach-card.unreachable {{
            background: #eafaf1;
            border: 2px solid #27ae60;
        }}
        .reach-value {{
            font-size: 36px;
            font-weight: bold;
        }}
        .compliance-grid {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
        }}
        .compliance-card {{
            padding: 15px;
            border-radius: 8px;
            background: #f8f9fa;
            text-align: center;
        }}
        .compliance-card.pass {{
            background: #eafaf1;
            border-left: 4px solid #27ae60;
        }}
        .compliance-card.warn {{
            background: #fef9e7;
            border-left: 4px solid #f39c12;
        }}

        .metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 15px;
            background: white;
            border-radius: 12px;
            padding: 20px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .metric {{
            text-align: center;
            padding: 15px;
        }}
        .metric-value {{
            font-size: 32px;
            font-weight: bold;
        }}
        .metric-label {{
            font-size: 12px;
            color: #7f8c8d;
            text-transform: uppercase;
        }}
        .critical {{ color: #e74c3c; }}
        .high {{ color: #f39c12; }}
        .medium {{ color: #3498db; }}
        .low {{ color: #95a5a6; }}

        .section {{
            background: white;
            border-radius: 12px;
            padding: 25px;
            margin-bottom: 20px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .section h2 {{
            margin: 0 0 20px 0;
            padding-bottom: 10px;
            border-bottom: 2px solid #ecf0f1;
        }}

        .layers-grid {{
            display: grid;
            gap: 10px;
        }}
        .layer {{
            padding: 15px;
            border-radius: 8px;
            border-left: 4px solid #bdc3c7;
            background: #f8f9fa;
        }}
        .layer.critical {{ border-left-color: #e74c3c; background: #fdf2f2; }}
        .layer.high {{ border-left-color: #f39c12; background: #fef9e7; }}
        .layer.medium {{ border-left-color: #3498db; background: #ebf5fb; }}
        .layer.clean {{ border-left-color: #27ae60; background: #eafaf1; }}
        .layer-header {{
            display: flex;
            justify-content: space-between;
            margin-bottom: 8px;
        }}
        .layer-num {{ font-weight: bold; }}
        .layer-digest {{ font-family: monospace; font-size: 12px; color: #7f8c8d; }}
        .layer-stats {{
            display: flex;
            gap: 20px;
            font-size: 14px;
            color: #7f8c8d;
        }}
        .layer-vulns {{ font-weight: bold; margin-top: 8px; }}
        .layer-clean {{ color: #27ae60; font-weight: bold; margin-top: 8px; }}

        .quick-wins-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
        }}
        .quick-win {{
            padding: 15px;
            background: #eafaf1;
            border-radius: 8px;
            border-left: 4px solid #27ae60;
        }}
        .qw-package {{ font-weight: bold; color: #27ae60; }}
        .qw-version {{ font-family: monospace; font-size: 14px; margin: 5px 0; }}
        .qw-impact {{ font-size: 12px; color: #7f8c8d; }}

        .vuln-item {{
            padding: 15px;
            margin-bottom: 15px;
            border-radius: 8px;
            border-left: 4px solid #e74c3c;
            background: #fdf2f2;
        }}
        .vuln-item.high {{ border-left-color: #f39c12; background: #fef9e7; }}
        .vuln-item.medium {{ border-left-color: #3498db; background: #ebf5fb; }}
        .vuln-header {{
            display: flex;
            gap: 10px;
            align-items: center;
            margin-bottom: 10px;
        }}
        .cve-id {{ font-weight: bold; font-size: 16px; }}
        .priority-badge {{
            background: #e74c3c;
            color: white;
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
        }}
        .severity-badge {{
            padding: 2px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: bold;
        }}
        .severity-badge.critical {{ background: #e74c3c; color: white; }}
        .severity-badge.high {{ background: #f39c12; color: white; }}
        .vuln-package {{ margin-bottom: 8px; }}
        .vuln-desc {{ font-size: 14px; color: #555; margin-bottom: 8px; }}
        .vuln-fix {{ font-size: 14px; }}
        .kev-badge {{
            display: inline-block;
            background: #e74c3c;
            color: white;
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            margin-top: 8px;
        }}

        .upgrade-rec {{
            padding: 15px;
            margin-bottom: 10px;
            border-radius: 8px;
            border-left: 4px solid #3498db;
            background: #ebf5fb;
        }}
        .upgrade-rec.low-risk {{ border-left-color: #27ae60; background: #eafaf1; }}
        .upgrade-rec.medium-risk {{ border-left-color: #f39c12; background: #fef9e7; }}
        .upgrade-rec.high-risk {{ border-left-color: #e74c3c; background: #fdf2f2; }}
        .upgrade-package {{ font-weight: bold; }}
        .upgrade-version {{ font-family: monospace; margin: 5px 0; }}
        .upgrade-fixes {{ font-size: 12px; color: #7f8c8d; }}
        .upgrade-risk {{ font-size: 12px; font-weight: bold; margin-top: 5px; }}

        .footer {{
            text-align: center;
            padding: 20px;
            color: #7f8c8d;
            font-size: 14px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>CONTAINER Container Security Report</h1>
        <p><strong>Image:</strong> {image_name}</p>
        <p><strong>Generated:</strong> {timestamp}</p>
    </div>

    <div class="score-section">
        <div class="score-card">
            <div class="score-value">{score}</div>
            <div class="score-grade">Grade: {grade}</div>
            <div class="score-desc">{grade_desc}</div>
        </div>
        <div class="metrics">
            <div class="metric">
                <div class="metric-value">{packages}</div>
                <div class="metric-label">Packages</div>
            </div>
            <div class="metric">
                <div class="metric-value">{total_vulns}</div>
                <div class="metric-label">Vulnerabilities</div>
            </div>
            <div class="metric">
                <div class="metric-value critical">{critical}</div>
                <div class="metric-label">Critical</div>
            </div>
            <div class="metric">
                <div class="metric-value high">{high}</div>
                <div class="metric-label">High</div>
            </div>
            <div class="metric">
                <div class="metric-value medium">{medium}</div>
                <div class="metric-label">Medium</div>
            </div>
            <div class="metric">
                <div class="metric-value low">{low_count}</div>
                <div class="metric-label">Low</div>
            </div>
        </div>
    </div>

    {kev_alert_html}

    <div class="section">
        <h2>TARGET Reachability Analysis</h2>
        <p style="color: #7f8c8d; margin-bottom: 15px;">Vulnerabilities analyzed for actual exploitability in your code</p>
        <div class="reachability-summary">
            <div class="reach-card reachable">
                <div class="reach-value critical">{reachable_count}</div>
                <div>TARGET Reachable</div>
                <div style="font-size: 12px; color: #7f8c8d;">Prioritize these - actually exploitable</div>
            </div>
            <div class="reach-card unreachable">
                <div class="reach-value" style="color: #27ae60;">{unreachable_count}</div>
                <div>SHIELD Unreachable</div>
                <div style="font-size: 12px; color: #7f8c8d;">Lower priority - not in execution path</div>
            </div>
        </div>
    </div>

    <div class="section">
        <h2>PKG Layer Attribution</h2>
        <div class="layers-grid">
            {layers_html}
        </div>
    </div>

    <div class="section">
        <h2>⚡ Quick Wins & Upgrade Intelligence</h2>
        <p style="color: #7f8c8d; margin-bottom: 15px;">Prioritized upgrades with effort estimates and breaking change analysis</p>
        <div class="quick-wins-grid">
            {quick_wins}
        </div>
    </div>

    <div class="section">
        <h2>NOTE Compliance Status</h2>
        <div class="compliance-grid">
            <div class="compliance-card {pci_class}">
                <strong>PCI-DSS</strong>
                <div>{pci_status}</div>
                {pci_issues}
            </div>
            <div class="compliance-card {hipaa_class}">
                <strong>HIPAA</strong>
                <div>{hipaa_status}</div>
                {hipaa_issues}
            </div>
            <div class="compliance-card {soc2_class}">
                <strong>SOC 2</strong>
                <div>{soc2_status}</div>
                {soc2_issues}
            </div>
        </div>
    </div>

    <div class="footer">
        Generated by BazBOM Container Scanner • {timestamp}
    </div>
</body>
</html>"#,
        image_name = results.image_name,
        timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        score = security.score,
        grade = security.grade,
        grade_color = security.grade_color,
        grade_desc = security.grade_desc,
        packages = results.total_packages,
        total_vulns = all_vulns.len(),
        critical = results.critical_count,
        high = results.high_count,
        medium = results.medium_count,
        low_count = results.low_count,
        reachable_count = reachable_count,
        unreachable_count = unreachable_count,
        kev_alert_html = kev_alert_html,
        layers_html = layers_html,
        quick_wins = quick_wins_html,
        pci_class = if results.compliance_results.as_ref().map(|c| c.pci_dss.status == "Pass").unwrap_or(false) { "pass" } else { "warn" },
        pci_status = if results.compliance_results.as_ref().map(|c| c.pci_dss.status == "Pass").unwrap_or(false) { "OK Pass" } else { "WARN Fail" },
        pci_issues = results.compliance_results.as_ref().map(|c| {
            if c.pci_dss.issues.is_empty() {
                String::new()
            } else {
                format!("<ul style='font-size: 11px; margin: 8px 0 0 0; padding-left: 16px; text-align: left; color: #7f8c8d;'>{}</ul>",
                    c.pci_dss.issues.iter().map(|i| format!("<li>{}</li>", i)).collect::<String>())
            }
        }).unwrap_or_default(),
        hipaa_class = if results.compliance_results.as_ref().map(|c| c.hipaa.status == "Pass").unwrap_or(false) { "pass" } else { "warn" },
        hipaa_status = if results.compliance_results.as_ref().map(|c| c.hipaa.status == "Pass").unwrap_or(false) { "OK Pass" } else { "WARN Fail" },
        hipaa_issues = results.compliance_results.as_ref().map(|c| {
            if c.hipaa.issues.is_empty() {
                String::new()
            } else {
                format!("<ul style='font-size: 11px; margin: 8px 0 0 0; padding-left: 16px; text-align: left; color: #7f8c8d;'>{}</ul>",
                    c.hipaa.issues.iter().map(|i| format!("<li>{}</li>", i)).collect::<String>())
            }
        }).unwrap_or_default(),
        soc2_class = if results.compliance_results.as_ref().map(|c| c.soc2.status == "Pass").unwrap_or(false) { "pass" } else { "warn" },
        soc2_status = if results.compliance_results.as_ref().map(|c| c.soc2.status == "Pass").unwrap_or(false) { "OK Pass" } else { "WARN Fail" },
        soc2_issues = results.compliance_results.as_ref().map(|c| {
            if c.soc2.issues.is_empty() {
                String::new()
            } else {
                format!("<ul style='font-size: 11px; margin: 8px 0 0 0; padding-left: 16px; text-align: left; color: #7f8c8d;'>{}</ul>",
                    c.soc2.issues.iter().map(|i| format!("<li>{}</li>", i)).collect::<String>())
            }
        }).unwrap_or_default()
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
            if let Some(dep) = dependencies
                .iter_mut()
                .find(|d: &&mut Dependency| d.name == vuln.package_name)
            {
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

// =============================================================================
// NEW: Orchestrator-based scanning (uses all tools in parallel)
// =============================================================================

/// Run a comprehensive scan using the new tool orchestrator
/// This runs Trivy, Grype, Syft, Dockle, Dive, and TruffleHog in parallel
pub async fn run_orchestrated_scan(opts: &ContainerScanOptions) -> Result<AggregatedResults> {
    use std::fs;

    // Ensure output directories exist
    fs::create_dir_all(&opts.output_dir)?;
    fs::create_dir_all(opts.output_dir.join("findings"))?;
    fs::create_dir_all(opts.output_dir.join("sbom"))?;

    // Configure orchestrator
    let config = OrchestratorConfig::default();
    let orchestrator = ToolOrchestrator::with_config(&opts.output_dir, config);

    // Check for missing tools
    let missing = orchestrator.check_tools();
    if !missing.is_empty() {
        println!("WARN  {} Missing tools:", "Warning:".yellow());
        for (name, hint) in &missing {
            println!("   • {} - {}", name.red(), hint.dimmed());
        }
        println!();
    }

    // Run parallel scan
    let results = orchestrator.scan(&opts.image_name).await?;

    Ok(results)
}

/// Convert ContainerScanResults to VulnerabilityFindings for report generation
fn convert_to_vuln_findings(results: &ContainerScanResults) -> VulnerabilityFindings {
    let mut critical = Vec::new();
    let mut high = Vec::new();
    let mut medium = Vec::new();
    let mut low = Vec::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            let detail = VulnerabilityDetail {
                cve: vuln.cve_id.clone(),
                package_name: vuln.package_name.clone(),
                package_version: vuln.installed_version.clone(),
                severity: vuln.severity.clone(),
                cvss_score: vuln.cvss_score.unwrap_or(0.0),
                description: vuln.description.chars().take(200).collect(),
                fixed_version: vuln.fixed_version.clone(),
                is_reachable: vuln.is_reachable,
                is_kev: vuln.is_kev,
                epss_score: vuln.epss_score,
                call_chain: vuln.call_chain.clone(),
            };

            match vuln.severity.to_uppercase().as_str() {
                "CRITICAL" => critical.push(detail),
                "HIGH" => high.push(detail),
                "MEDIUM" => medium.push(detail),
                _ => low.push(detail),
            }
        }
    }

    VulnerabilityFindings {
        critical,
        high,
        medium,
        low,
    }
}

/// Generate SARIF report from container scan results
fn generate_sarif_report(results: &ContainerScanResults) -> SarifReport {
    let version = env!("CARGO_PKG_VERSION");
    let mut report = SarifReport::new("bazbom-container-scan", version);

    // Add rules and results for each vulnerability
    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            // Determine SARIF level from severity
            let level = match vuln.severity.to_uppercase().as_str() {
                "CRITICAL" => "error",
                "HIGH" => "error",
                "MEDIUM" => "warning",
                "LOW" => "note",
                _ => "note",
            };

            // Create rule for this CVE
            let rule = Rule::new(&vuln.cve_id, &vuln.title, level);
            report.add_rule(rule);

            // Create result with properties
            let mut message = format!(
                "{} in {} {} (layer: {})",
                vuln.cve_id, vuln.package_name, vuln.installed_version, layer.digest
            );
            if let Some(fix) = &vuln.fixed_version {
                message.push_str(&format!(" - Fix available: {}", fix));
            }

            let mut result =
                SarifResult::new(&vuln.cve_id, level, message).with_location(&results.image_name);

            // Add properties with enrichment data
            let mut properties = serde_json::json!({
                "package": vuln.package_name,
                "installedVersion": vuln.installed_version,
                "severity": vuln.severity,
                "layer": layer.digest,
                "isReachable": vuln.is_reachable,
            });

            if let Some(fix) = &vuln.fixed_version {
                properties["fixedVersion"] = serde_json::json!(fix);
            }
            if let Some(cvss) = vuln.cvss_score {
                properties["cvssScore"] = serde_json::json!(cvss);
            }
            if let Some(epss) = vuln.epss_score {
                properties["epssScore"] = serde_json::json!(epss);
            }
            if vuln.is_kev {
                properties["isKev"] = serde_json::json!(true);
                if let Some(due) = &vuln.kev_due_date {
                    properties["kevDueDate"] = serde_json::json!(due);
                }
            }
            if let Some(priority) = &vuln.priority {
                properties["priority"] = serde_json::json!(priority);
            }
            if let Some(breaking) = vuln.breaking_change {
                properties["breakingChange"] = serde_json::json!(breaking);
            }

            result = result.with_properties(properties);
            report.add_result(result);
        }
    }

    report
}

/// Merge native OS package scan vulnerabilities into container scan results
///
/// This provides supplementary vulnerability data from BazBOM's native OS package
/// scanners (Alpine, Debian, Red Hat) when external tools may have missed packages.
fn merge_native_vulnerabilities(
    results: &mut ContainerScanResults,
    os_results: &os_packages::OsScanResult,
) {
    use std::collections::HashSet;

    // Collect existing CVE+package pairs to avoid duplicates
    let mut existing: HashSet<(String, String)> = HashSet::new();
    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            existing.insert((vuln.cve_id.clone(), vuln.package_name.clone()));
        }
    }

    // Add new vulnerabilities from native scan
    let mut new_vulns = Vec::new();
    for vuln in &os_results.vulnerabilities {
        let key = (vuln.cve_id.clone(), vuln.package.clone());
        if !existing.contains(&key) {
            new_vulns.push(VulnerabilityInfo {
                cve_id: vuln.cve_id.clone(),
                package_name: vuln.package.clone(),
                installed_version: vuln.installed_version.clone(),
                fixed_version: vuln.fixed_version.clone(),
                severity: vuln.severity.clone(),
                cvss_score: None,
                title: format!("{} in {}", vuln.cve_id, vuln.package),
                description: format!(
                    "Vulnerability found by native {} scanner",
                    match vuln.os_type {
                        os_packages::OsType::Alpine => "Alpine",
                        os_packages::OsType::Debian => "Debian",
                        os_packages::OsType::Ubuntu => "Ubuntu",
                        os_packages::OsType::Rhel
                        | os_packages::OsType::CentOS
                        | os_packages::OsType::Fedora => "Red Hat",
                        os_packages::OsType::Unknown(_) => "OS",
                    }
                ),
                layer_digest: "native-scan".to_string(),
                published_date: None,
                epss_score: None,
                epss_percentile: None,
                is_kev: false,
                kev_due_date: None,
                priority: None,
                references: vec![],
                breaking_change: None,
                upgrade_path: None,
                is_reachable: false, // Will be analyzed by reachability pass
                difficulty_score: None,
                call_chain: None,      // Will be populated if reachable
                dependency_path: None, // Populated for transitive deps
            });
            existing.insert(key);
        }
    }

    // Add to first layer (or create one if none exist)
    if !new_vulns.is_empty() {
        if results.layers.is_empty() {
            results.layers.push(LayerInfo {
                digest: "native-scan".to_string(),
                size_mb: 0.0,
                packages: os_results.packages.iter().map(|p| p.name.clone()).collect(),
                vulnerabilities: new_vulns.clone(),
            });
        } else {
            results.layers[0].vulnerabilities.extend(new_vulns.clone());
        }

        // Update counts
        results.total_vulnerabilities += new_vulns.len();
        for vuln in &new_vulns {
            match vuln.severity.to_uppercase().as_str() {
                "CRITICAL" => results.critical_count += 1,
                "HIGH" => results.high_count += 1,
                "MEDIUM" => results.medium_count += 1,
                "LOW" => results.low_count += 1,
                _ => {}
            }
        }

        tracing::info!(
            "Merged {} new vulnerabilities from native OS scan",
            new_vulns.len()
        );
    }
}
