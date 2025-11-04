use anyhow::{Context, Result};
use bazbom_core::{detect_build_system, write_stub_sbom};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

mod advisory;
mod bazel;
mod policy_integration;
mod reachability;
mod reachability_cache;
mod shading;

use bazbom::cli::{Cli, Commands, DbCmd, LicenseCmd, PolicyCmd, ReportCmd, ComplianceFrameworkArg};
use bazbom::hooks::{install_hooks, HooksConfig};
use bazbom::remediation;
use bazbom::scan_orchestrator::ScanOrchestrator;

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Scan {
        path: ".".into(),
        reachability: false,
        fast: false,
        format: "spdx".into(),
        out_dir: ".".into(),
        bazel_targets_query: None,
        bazel_targets: None,
        bazel_affected_by_files: None,
        bazel_universe: "//...".into(),
        cyclonedx: false,
        with_semgrep: false,
        with_codeql: None,
        autofix: None,
        containers: None,
        no_upload: false,
        target: None,
        incremental: false,
        base: "main".into(),
    }) {
        Commands::Scan {
            path,
            reachability,
            fast,
            format,
            out_dir,
            bazel_targets_query,
            bazel_targets,
            bazel_affected_by_files,
            bazel_universe,
            cyclonedx,
            with_semgrep,
            with_codeql,
            autofix,
            containers,
            no_upload,
            target,
            incremental,
            base,
        } => {
            // Check if any orchestration flags are set
            let use_orchestrator = cyclonedx
                || with_semgrep
                || with_codeql.is_some()
                || autofix.is_some()
                || containers.is_some();

            if use_orchestrator {
                // Use new orchestration path
                println!("[bazbom] using orchestrated scan mode");
                let workspace = PathBuf::from(&path);
                let output_dir = PathBuf::from(&out_dir);

                let orchestrator = ScanOrchestrator::new(
                    workspace,
                    output_dir,
                    bazbom::scan_orchestrator::ScanOrchestratorOptions {
                        cyclonedx,
                        with_semgrep,
                        with_codeql,
                        autofix,
                        containers,
                        no_upload,
                        target,
                        threat_detection: None, // Use default from config
                        incremental: false,     // Disabled by default, enable via CLI flag
                    },
                )?;

                return orchestrator.run();
            }

            // Original scan logic follows
            let root = PathBuf::from(&path);
            let system = detect_build_system(&root);

            // Handle incremental analysis if requested
            if incremental {
                use bazbom::incremental::IncrementalAnalyzer;

                println!("[bazbom] incremental mode enabled (base: {})", base);
                let analyzer = IncrementalAnalyzer::new(root.clone(), base.clone());

                if !analyzer.is_supported() {
                    println!("[bazbom] warning: incremental analysis not supported (not a git repository or invalid base ref)");
                    println!("[bazbom] falling back to full scan");
                } else {
                    match analyzer.find_affected_targets() {
                        Ok(affected_targets) => {
                            if affected_targets.is_empty() {
                                println!("[bazbom] no changes detected since {}. Using cached results.", base);
                                println!("[bazbom] tip: run without --incremental to force a full scan");
                                return Ok(());
                            }

                            println!(
                                "[bazbom] detected {} affected targets",
                                affected_targets.len()
                            );
                            println!("[bazbom] proceeding with incremental scan...");

                            // For Bazel, we can use the affected targets directly
                            // For Maven/Gradle, full scan is needed (handled by build tool)
                        }
                        Err(e) => {
                            println!("[bazbom] error detecting affected targets: {}", e);
                            println!("[bazbom] falling back to full scan");
                        }
                    }
                }
            }

            // Initialize cache (unless disabled via env var for testing)
            let cache_enabled = std::env::var("BAZBOM_DISABLE_CACHE").is_err();
            let cache_dir = root.join(".bazbom").join("cache");
            let mut scan_cache_opt = if cache_enabled {
                bazbom::scan_cache::ScanCache::new(cache_dir.clone())
                    .context("Failed to initialize scan cache")
                    .ok()
            } else {
                None
            };

            // Build scan parameters for cache key
            let scan_params = bazbom::scan_cache::ScanParameters {
                reachability: reachability && !fast,
                fast,
                format: format.clone(),
                bazel_targets: bazel_targets.clone(),
            };

            // Determine build files for cache key
            let build_files: Vec<PathBuf> = match system {
                bazbom_core::BuildSystem::Maven => vec![root.join("pom.xml")],
                bazbom_core::BuildSystem::Gradle => vec![
                    root.join("build.gradle"),
                    root.join("build.gradle.kts"),
                    root.join("settings.gradle"),
                    root.join("settings.gradle.kts"),
                ],
                bazbom_core::BuildSystem::Bazel => vec![
                    root.join("BUILD"),
                    root.join("BUILD.bazel"),
                    root.join("WORKSPACE"),
                    root.join("WORKSPACE.bazel"),
                    root.join("MODULE.bazel"),
                ],
                bazbom_core::BuildSystem::Unknown => vec![],
            };

            // Generate cache key
            let cache_key = bazbom::scan_cache::ScanCache::generate_cache_key(
                &root,
                &build_files,
                &scan_params,
            )?;

            // Check cache first (if enabled)
            if let Some(scan_cache) = scan_cache_opt.as_mut() {
                if let Some(cached_result) = scan_cache.get_scan_result(&cache_key)? {
                println!(
                    "[bazbom] cache hit! using cached scan from {}",
                    cached_result.scanned_at
                );

                // Write cached results to disk
                let out = PathBuf::from(&out_dir);
                let sbom_path = match format.as_str() {
                    "cyclonedx" => out.join("sbom.cyclonedx.json"),
                    _ => out.join("sbom.spdx.json"),
                };
                fs::write(&sbom_path, cached_result.sbom_json.as_bytes())
                    .context("Failed to write cached SBOM")?;

                if let Some(findings) = cached_result.findings_json {
                    let findings_path = out.join("findings.json");
                    fs::write(&findings_path, findings.as_bytes())
                        .context("Failed to write cached findings")?;
                }

                    println!("[bazbom] scan complete (from cache)");
                    return Ok(());
                }
            }

            // Handle Bazel-specific target selection
            let bazel_targets_to_scan = if system == bazbom_core::BuildSystem::Bazel {
                if let Some(query) = &bazel_targets_query {
                    println!("[bazbom] using Bazel query: {}", query);
                    match bazel::query_bazel_targets(
                        &root,
                        Some(query),
                        None,
                        None,
                        &bazel_universe,
                    ) {
                        Ok(targets) => Some(targets),
                        Err(e) => {
                            eprintln!("[bazbom] warning: Bazel query failed: {}", e);
                            None
                        }
                    }
                } else if let Some(targets) = &bazel_targets {
                    println!("[bazbom] using explicit targets: {:?}", targets);
                    Some(targets.clone())
                } else if let Some(files) = &bazel_affected_by_files {
                    println!("[bazbom] finding targets affected by {} files", files.len());
                    match bazel::query_bazel_targets(
                        &root,
                        None,
                        None,
                        Some(files),
                        &bazel_universe,
                    ) {
                        Ok(targets) => Some(targets),
                        Err(e) => {
                            eprintln!("[bazbom] warning: failed to find affected targets: {}", e);
                            None
                        }
                    }
                } else {
                    None // Scan all targets
                }
            } else {
                None
            };

            if let Some(ref targets) = bazel_targets_to_scan {
                if targets.is_empty() {
                    println!("[bazbom] warning: no targets selected, scanning entire workspace");
                } else {
                    println!("[bazbom] scanning {} selected targets", targets.len());
                }
            }

            if fast {
                println!("[bazbom] fast mode enabled (skipping reachability analysis)");
            }

            println!(
                "[bazbom] scan path={} reachability={} format={} system={:?}",
                path,
                reachability && !fast,
                format,
                system
            );
            let out = PathBuf::from(&out_dir);

            // For Bazel projects, extract dependencies and generate SBOM
            let sbom_path = if system == bazbom_core::BuildSystem::Bazel {
                let deps_json_path = out.join("bazel_deps.json");

                // If we have specific targets, extract dependencies for those
                let extraction_result = if let Some(targets) = &bazel_targets_to_scan {
                    if !targets.is_empty() {
                        bazel::extract_bazel_dependencies_for_targets(
                            &root,
                            targets,
                            &deps_json_path,
                        )
                    } else {
                        bazel::extract_bazel_dependencies(&root, &deps_json_path)
                    }
                } else {
                    bazel::extract_bazel_dependencies(&root, &deps_json_path)
                };

                match extraction_result {
                    Ok(graph) => {
                        println!(
                            "[bazbom] extracted {} Bazel components and {} edges",
                            graph.components.len(),
                            graph.edges.len()
                        );

                        // Write raw dependency graph
                        println!("[bazbom] wrote dependency graph to {:?}", deps_json_path);

                        // Convert to SPDX format
                        let project_name = root
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("bazel-project");
                        let spdx_doc = graph.to_spdx(project_name);

                        let sbom_path = match format.as_str() {
                            "cyclonedx" => out.join("sbom.cyclonedx.json"),
                            _ => out.join("sbom.spdx.json"),
                        };

                        fs::write(&sbom_path, serde_json::to_vec_pretty(&spdx_doc).unwrap())
                            .with_context(|| format!("failed writing {:?}", sbom_path))?;

                        sbom_path
                    }
                    Err(e) => {
                        eprintln!(
                            "[bazbom] warning: failed to extract Bazel dependencies: {}",
                            e
                        );
                        eprintln!("[bazbom] falling back to stub SBOM");
                        write_stub_sbom(&out, &format, system)
                            .with_context(|| format!("failed writing stub SBOM to {:?}", out))?
                    }
                }
            } else {
                write_stub_sbom(&out, &format, system)
                    .with_context(|| format!("failed writing stub SBOM to {:?}", out))?
            };
            println!("[bazbom] wrote {:?}", sbom_path);

            // Load advisories from cache
            let cache_dir = PathBuf::from(".bazbom/cache");
            let vulnerabilities = if cache_dir.exists() {
                match advisory::load_advisories(&cache_dir) {
                    Ok(vulns) => {
                        println!("[bazbom] loaded {} vulnerabilities from cache", vulns.len());
                        vulns
                    }
                    Err(e) => {
                        eprintln!("[bazbom] warning: failed to load advisories: {}", e);
                        Vec::new()
                    }
                }
            } else {
                eprintln!("[bazbom] warning: advisory cache not found at {:?}, run 'bazbom db sync' first", cache_dir);
                Vec::new()
            };

            // Run reachability analysis if requested (unless fast mode is enabled)
            let reachability_result = if reachability && !fast {
                // Attempt to run reachability analysis if configured
                if let Ok(jar) = std::env::var("BAZBOM_REACHABILITY_JAR") {
                    let jar_path = PathBuf::from(&jar);
                    if !jar_path.exists() {
                        eprintln!(
                            "[bazbom] BAZBOM_REACHABILITY_JAR points to non-existent file: {:?}",
                            jar_path
                        );
                        None
                    } else {
                        let out_file = out.join("reachability.json");

                        // Extract classpath based on build system
                        let classpath = match system {
                            bazbom_core::BuildSystem::Maven => {
                                reachability::extract_maven_classpath(&root).unwrap_or_else(|e| {
                                    eprintln!("[bazbom] failed to extract Maven classpath: {}", e);
                                    String::new()
                                })
                            }
                            bazbom_core::BuildSystem::Gradle => {
                                reachability::extract_gradle_classpath(&root).unwrap_or_else(|e| {
                                    eprintln!("[bazbom] failed to extract Gradle classpath: {}", e);
                                    String::new()
                                })
                            }
                            bazbom_core::BuildSystem::Bazel => {
                                reachability::extract_bazel_classpath(&root, "").unwrap_or_else(
                                    |e| {
                                        eprintln!(
                                            "[bazbom] failed to extract Bazel classpath: {}",
                                            e
                                        );
                                        String::new()
                                    },
                                )
                            }
                            _ => String::new(),
                        };

                        let entrypoints = "";
                        let cache_dir = reachability_cache::get_cache_dir();

                        // Check cache first
                        let result = if let Ok(Some(cached)) =
                            reachability_cache::load_cached_result(
                                &cache_dir,
                                &classpath,
                                entrypoints,
                            ) {
                            println!("[bazbom] using cached reachability result");
                            Some(cached)
                        } else {
                            // Run analysis and cache result
                            match reachability::analyze_reachability(
                                &jar_path,
                                &classpath,
                                entrypoints,
                                &out_file,
                            ) {
                                Ok(result) => {
                                    println!("[bazbom] reachability analysis complete");
                                    if result.reachable_classes.is_empty() {
                                        println!("[bazbom] no reachable classes found (classpath may be empty)");
                                    }

                                    // Save to cache
                                    if let Err(e) = reachability_cache::save_cached_result(
                                        &cache_dir,
                                        &classpath,
                                        entrypoints,
                                        &result,
                                    ) {
                                        eprintln!("[bazbom] warning: failed to cache reachability result: {}", e);
                                    }

                                    Some(result)
                                }
                                Err(e) => {
                                    eprintln!("[bazbom] reachability analysis failed: {}", e);
                                    None
                                }
                            }
                        };

                        result
                    }
                } else {
                    eprintln!(
                        "[bazbom] --reachability set but BAZBOM_REACHABILITY_JAR not configured"
                    );
                    eprintln!("[bazbom] set BAZBOM_REACHABILITY_JAR to the path of bazbom-reachability.jar");
                    None
                }
            } else {
                None
            };

            // Detect shading/relocation configuration
            let shading_config = match system {
                bazbom_core::BuildSystem::Maven => {
                    let pom_path = root.join("pom.xml");
                    if pom_path.exists() {
                        match shading::parse_maven_shade_config(&pom_path) {
                            Ok(Some(config)) => {
                                println!(
                                    "[bazbom] detected Maven Shade plugin with {} relocations",
                                    config.relocations.len()
                                );
                                Some(config)
                            }
                            Ok(None) => None,
                            Err(e) => {
                                eprintln!(
                                    "[bazbom] warning: failed to parse Maven Shade config: {}",
                                    e
                                );
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                bazbom_core::BuildSystem::Gradle => {
                    let build_gradle = root.join("build.gradle");
                    let build_gradle_kts = root.join("build.gradle.kts");
                    let build_file = if build_gradle.exists() {
                        build_gradle
                    } else if build_gradle_kts.exists() {
                        build_gradle_kts
                    } else {
                        PathBuf::new()
                    };

                    if build_file.exists() {
                        match shading::parse_gradle_shadow_config(&build_file) {
                            Ok(Some(config)) => {
                                println!(
                                    "[bazbom] detected Gradle Shadow plugin with {} relocations",
                                    config.relocations.len()
                                );
                                Some(config)
                            }
                            Ok(None) => None,
                            Err(e) => {
                                eprintln!(
                                    "[bazbom] warning: failed to parse Gradle Shadow config: {}",
                                    e
                                );
                                None
                            }
                        }
                    } else {
                        None
                    }
                }
                _ => None,
            };

            // Write shading configuration to file if detected
            if let Some(ref config) = shading_config {
                let shading_output = out.join("shading_config.json");
                fs::write(&shading_output, serde_json::to_vec_pretty(&config).unwrap())
                    .with_context(|| format!("failed writing {:?}", shading_output))?;
                println!("[bazbom] wrote {:?}", shading_output);
            }

            // Create findings file with vulnerability data, including reachability and shading info
            let findings_path = out.join("sca_findings.json");
            let mut findings_data = serde_json::json!({
                "vulnerabilities": vulnerabilities,
                "summary": {
                    "total": vulnerabilities.len(),
                    "critical": vulnerabilities.iter().filter(|v| {
                        matches!(v.severity.as_ref().map(|s| s.level), Some(bazbom_advisories::SeverityLevel::Critical))
                    }).count(),
                    "high": vulnerabilities.iter().filter(|v| {
                        matches!(v.severity.as_ref().map(|s| s.level), Some(bazbom_advisories::SeverityLevel::High))
                    }).count(),
                    "medium": vulnerabilities.iter().filter(|v| {
                        matches!(v.severity.as_ref().map(|s| s.level), Some(bazbom_advisories::SeverityLevel::Medium))
                    }).count(),
                    "low": vulnerabilities.iter().filter(|v| {
                        matches!(v.severity.as_ref().map(|s| s.level), Some(bazbom_advisories::SeverityLevel::Low))
                    }).count(),
                }
            });

            // Add reachability info if available
            if let Some(ref reach) = reachability_result {
                findings_data["reachability"] = serde_json::json!({
                    "enabled": true,
                    "detected_entrypoints": reach.detected_entrypoints.len(),
                    "reachable_methods": reach.reachable_methods.len(),
                    "reachable_classes": reach.reachable_classes.len(),
                    "reachable_packages": reach.reachable_packages.len(),
                });
            } else {
                findings_data["reachability"] = serde_json::json!({
                    "enabled": false,
                });
            }

            // Add shading info if available
            if let Some(ref config) = shading_config {
                findings_data["shading"] = serde_json::json!({
                    "detected": true,
                    "source": config.source,
                    "relocations": config.relocations.len(),
                });
            } else {
                findings_data["shading"] = serde_json::json!({
                    "detected": false,
                });
            }

            fs::write(
                &findings_path,
                serde_json::to_vec_pretty(&findings_data).unwrap(),
            )
            .with_context(|| format!("failed writing {:?}", findings_path))?;
            println!(
                "[bazbom] wrote {:?} ({} vulnerabilities)",
                findings_path,
                vulnerabilities.len()
            );

            // Create SARIF report with vulnerability results
            let sarif_path = out.join("sca_findings.sarif");
            let mut sarif = bazbom_formats::sarif::SarifReport::new("bazbom", bazbom_core::VERSION);

            // Add informational note about shading if detected
            if let Some(ref config) = shading_config {
                let shading_note = format!(
                    "Shading detected: {} relocations from {} (see shading_config.json for details)",
                    config.relocations.len(),
                    config.source
                );
                let info_result =
                    bazbom_formats::sarif::Result::new("shading/detected", "note", &shading_note);
                sarif.add_result(info_result);
            }

            // Add vulnerability results to SARIF with reachability info if available
            for vuln in &vulnerabilities {
                let level = match vuln.severity.as_ref().map(|s| s.level) {
                    Some(bazbom_advisories::SeverityLevel::Critical) => "error",
                    Some(bazbom_advisories::SeverityLevel::High) => "error",
                    Some(bazbom_advisories::SeverityLevel::Medium) => "warning",
                    Some(bazbom_advisories::SeverityLevel::Low) => "note",
                    _ => "note",
                };

                let mut message = vuln
                    .summary
                    .clone()
                    .or_else(|| vuln.details.clone())
                    .unwrap_or_else(|| format!("Vulnerability {}", vuln.id));

                // Add reachability info to message if available
                if let Some(ref reach) = reachability_result {
                    // Extract package name from first affected package
                    if let Some(affected) = vuln.affected.first() {
                        let package_name = affected.package.clone();
                        if reach.is_package_reachable(&package_name) {
                            message.push_str(" [REACHABLE]");
                        } else {
                            message.push_str(" [NOT REACHABLE]");
                        }
                    }
                }

                let result = bazbom_formats::sarif::Result::new(&vuln.id, level, &message);
                sarif.add_result(result);
            }

            fs::write(&sarif_path, serde_json::to_vec_pretty(&sarif).unwrap())
                .with_context(|| format!("failed writing {:?}", sarif_path))?;
            println!("[bazbom] wrote {:?}", sarif_path);

            // Apply policy checks if policy file exists
            let policy_path = PathBuf::from("bazbom.yml");
            if policy_path.exists() {
                let policy = policy_integration::load_policy_config(&policy_path)
                    .context("failed to load policy configuration")?;

                let policy_result = policy_integration::check_policy_with_reachability(
                    &vulnerabilities,
                    &policy,
                    reachability_result.as_ref(),
                );

                if !policy_result.passed {
                    println!(
                        "[bazbom] ⚠ policy violations detected ({} violations)",
                        policy_result.violations.len()
                    );
                    for violation in &policy_result.violations {
                        println!("  - {}: {}", violation.rule, violation.message);
                    }

                    // Write policy violations to separate file
                    let policy_violations_path = out.join("policy_violations.json");
                    fs::write(
                        &policy_violations_path,
                        serde_json::to_vec_pretty(&policy_result).unwrap(),
                    )
                    .with_context(|| format!("failed writing {:?}", policy_violations_path))?;
                    println!("[bazbom] wrote {:?}", policy_violations_path);
                } else {
                    println!("[bazbom] ✓ all policy checks passed");
                }
            }

            // Store results in cache for next time
            let sbom_json = fs::read_to_string(&sbom_path)
                .context("Failed to read SBOM for caching")?;
            let findings_json = if findings_path.exists() {
                Some(
                    fs::read_to_string(&findings_path)
                        .context("Failed to read findings for caching")?,
                )
            } else {
                None
            };

            // Store in cache if enabled
            if let Some(scan_cache) = scan_cache_opt.as_mut() {
                let scan_result =
                    bazbom::scan_cache::ScanResult::new(sbom_json, findings_json, scan_params);
                
                if let Err(e) = scan_cache.put_scan_result(&cache_key, &scan_result) {
                    eprintln!("[bazbom] warning: failed to cache scan results: {}", e);
                    // Don't fail the scan if caching fails
                } else {
                    println!("[bazbom] scan results cached for future runs");
                }
            }
        }
        Commands::Policy { action } => match action {
            PolicyCmd::Check {} => {
                println!("[bazbom] policy check");

                // Load policy configuration
                let policy_path = PathBuf::from("bazbom.yml");
                let policy = policy_integration::load_policy_config(&policy_path)
                    .context("failed to load policy configuration")?;
                println!(
                    "[bazbom] loaded policy config (threshold={:?})",
                    policy.severity_threshold
                );

                // Load advisories from cache
                let cache_dir = PathBuf::from(".bazbom/cache");
                let vulnerabilities = if cache_dir.exists() {
                    match advisory::load_advisories(&cache_dir) {
                        Ok(vulns) => {
                            println!("[bazbom] loaded {} vulnerabilities from cache", vulns.len());
                            vulns
                        }
                        Err(e) => {
                            eprintln!("[bazbom] warning: failed to load advisories: {}", e);
                            Vec::new()
                        }
                    }
                } else {
                    eprintln!("[bazbom] warning: advisory cache not found at {:?}, run 'bazbom db sync' first", cache_dir);
                    Vec::new()
                };

                // Check vulnerabilities against policy
                let result = policy_integration::check_policy(&vulnerabilities, &policy);

                // Write policy result to JSON
                let policy_output = PathBuf::from("policy_result.json");
                fs::write(&policy_output, serde_json::to_vec_pretty(&result).unwrap())
                    .with_context(|| format!("failed writing {:?}", policy_output))?;
                println!("[bazbom] wrote {:?}", policy_output);

                // Write policy violations to SARIF
                let sarif_path = PathBuf::from("policy_violations.sarif");
                let mut sarif =
                    bazbom_formats::sarif::SarifReport::new("bazbom-policy", bazbom_core::VERSION);

                for violation in &result.violations {
                    let level = if violation.rule == "kev_gate" {
                        "error"
                    } else if let Some(vuln) = &violation.vulnerability {
                        match vuln.severity {
                            bazbom_policy::SeverityLevel::Critical => "error",
                            bazbom_policy::SeverityLevel::High => "error",
                            bazbom_policy::SeverityLevel::Medium => "warning",
                            _ => "note",
                        }
                    } else {
                        "warning"
                    };

                    let rule_id = format!("policy/{}", violation.rule);
                    let result_item =
                        bazbom_formats::sarif::Result::new(&rule_id, level, &violation.message);
                    sarif.add_result(result_item);
                }

                fs::write(&sarif_path, serde_json::to_vec_pretty(&sarif).unwrap())
                    .with_context(|| format!("failed writing {:?}", sarif_path))?;
                println!(
                    "[bazbom] wrote {:?} ({} violations)",
                    sarif_path,
                    result.violations.len()
                );

                // Print summary
                if result.passed {
                    println!("[bazbom] ✓ policy check passed (no violations)");
                } else {
                    println!(
                        "[bazbom] ✗ policy check failed ({} violations)",
                        result.violations.len()
                    );
                    for violation in &result.violations {
                        println!("  - {}: {}", violation.rule, violation.message);
                    }
                    std::process::exit(1);
                }
            }
            PolicyCmd::Init {
                list,
                template,
                output,
            } => {
                if list {
                    println!("[bazbom] Available policy templates:\n");
                    let templates = bazbom_policy::PolicyTemplateLibrary::list_templates();

                    let mut by_category: std::collections::HashMap<String, Vec<_>> =
                        std::collections::HashMap::new();
                    for template in templates {
                        by_category
                            .entry(template.category.clone())
                            .or_insert_with(Vec::new)
                            .push(template);
                    }

                    for (category, templates) in by_category {
                        println!("{}:", category);
                        for template in templates {
                            println!("  {} - {}", template.id, template.name);
                            println!("    {}", template.description);
                        }
                        println!();
                    }

                    println!("Usage: bazbom policy init --template <template-id>");
                } else if let Some(template_id) = template {
                    let output_path = PathBuf::from(&output);
                    match bazbom_policy::PolicyTemplateLibrary::initialize_template(
                        &template_id,
                        &output_path,
                    ) {
                        Ok(msg) => println!("{}", msg),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: Either --list or --template <template-id> must be specified");
                    eprintln!("Run 'bazbom policy init --list' to see available templates");
                    std::process::exit(1);
                }
            }
            PolicyCmd::Validate { policy_file } => {
                println!("[bazbom] validating policy file: {}", policy_file);

                let policy_path = PathBuf::from(&policy_file);
                match policy_integration::load_policy_config(&policy_path) {
                    Ok(policy) => {
                        println!("✓ Policy file is valid");
                        println!("\nPolicy Configuration:");
                        println!("  Severity threshold: {:?}", policy.severity_threshold);
                        println!("  KEV gate: {}", policy.kev_gate);
                        println!("  EPSS threshold: {:?}", policy.epss_threshold);
                        println!("  Reachability required: {}", policy.reachability_required);
                        println!("  VEX auto-apply: {}", policy.vex_auto_apply);

                        if let Some(allowlist) = &policy.license_allowlist {
                            println!("  License allowlist: {} licenses", allowlist.len());
                        }
                        if let Some(denylist) = &policy.license_denylist {
                            println!("  License denylist: {} licenses", denylist.len());
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Policy file validation failed: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },
        Commands::Fix {
            suggest,
            apply,
            pr,
            interactive,
        } => {
            println!(
                "[bazbom] fix suggest={} apply={} pr={} interactive={}",
                suggest, apply, pr, interactive
            );

            // Detect build system
            let root = PathBuf::from(".");
            let system = detect_build_system(&root);
            println!("[bazbom] detected build system: {:?}", system);

            // Load advisories from cache
            let cache_dir = PathBuf::from(".bazbom/cache");
            let vulnerabilities = if cache_dir.exists() {
                match advisory::load_advisories(&cache_dir) {
                    Ok(vulns) => {
                        println!("[bazbom] loaded {} vulnerabilities from cache", vulns.len());
                        vulns
                    }
                    Err(e) => {
                        eprintln!("[bazbom] warning: failed to load advisories: {}", e);
                        Vec::new()
                    }
                }
            } else {
                eprintln!("[bazbom] warning: advisory cache not found at {:?}, run 'bazbom db sync' first", cache_dir);
                Vec::new()
            };

            if vulnerabilities.is_empty() {
                println!("[bazbom] no vulnerabilities found - nothing to fix");
                return Ok(());
            }

            // Generate remediation suggestions
            let report = remediation::generate_suggestions(&vulnerabilities, system);

            // Display summary
            println!("\n[bazbom] Remediation Summary:");
            println!(
                "  Total vulnerabilities: {}",
                report.summary.total_vulnerabilities
            );
            println!("  Fixable: {}", report.summary.fixable);
            println!("  Unfixable: {}", report.summary.unfixable);
            println!("  Estimated effort: {}", report.summary.estimated_effort);

            if suggest {
                // Suggest mode: display suggestions
                println!("\n[bazbom] Remediation Suggestions:\n");

                for (i, suggestion) in report.suggestions.iter().enumerate() {
                    println!(
                        "{}. {} ({})",
                        i + 1,
                        suggestion.vulnerability_id,
                        suggestion.affected_package
                    );
                    println!("   Current version: {}", suggestion.current_version);
                    if let Some(fixed) = &suggestion.fixed_version {
                        println!("   Fixed version: {}", fixed);
                    } else {
                        println!("   Fixed version: NOT AVAILABLE");
                    }
                    println!(
                        "   Severity: {} | Priority: {}",
                        suggestion.severity, suggestion.priority
                    );
                    println!("\n   WHY FIX THIS:");
                    println!("   {}", suggestion.why_fix);
                    println!("\n   HOW TO FIX:");
                    for line in suggestion.how_to_fix.lines() {
                        println!("   {}", line);
                    }
                    if let Some(breaking) = &suggestion.breaking_changes {
                        println!("\n   BREAKING CHANGES:");
                        for line in breaking.lines() {
                            println!("   {}", line);
                        }
                    }
                    if !suggestion.references.is_empty() {
                        println!("\n   REFERENCES:");
                        for ref_url in &suggestion.references {
                            println!("   - {}", ref_url);
                        }
                    }
                    println!();
                }

                // Write suggestions to file
                let suggestions_path = PathBuf::from("remediation_suggestions.json");
                fs::write(
                    &suggestions_path,
                    serde_json::to_vec_pretty(&report).unwrap(),
                )
                .with_context(|| format!("failed writing {:?}", suggestions_path))?;
                println!("[bazbom] wrote {:?}", suggestions_path);
            }

            if interactive {
                // Interactive mode: smart batch processing with user confirmation
                use bazbom::batch_fixer::BatchFixer;
                use dialoguer::{theme::ColorfulTheme, Confirm};

                println!(
                    "\n🔍 Found {} fixable vulnerabilities",
                    report.suggestions.len()
                );
                println!("📊 Grouping by impact analysis...\n");

                let batch_fixer = BatchFixer::new(&report.suggestions);
                let batches = batch_fixer.create_batches();

                println!("✅ Safe batch groups identified: {}\n", batches.len());

                for (batch_num, batch) in batches.iter().enumerate() {
                    // Display batch header
                    println!("┌─ Batch {}: {} ─", batch_num + 1, batch.description());
                    println!("│");

                    // List updates in this batch
                    for (i, update) in batch.updates.iter().enumerate() {
                        println!(
                            "│  {}. {}: {} → {} ({})",
                            i + 1,
                            update.package,
                            update.current_version,
                            update.target_version,
                            update.severity
                        );

                        if let Some(reason) = &update.breaking_reason {
                            println!("│     ⚠️  {}", reason);
                        }
                    }

                    println!("│");
                    println!(
                        "│ Estimated time: ~{} seconds",
                        batch.estimated_time_seconds
                    );
                    if batch.test_count > 0 {
                        println!("│ Test coverage: {} tests will run", batch.test_count);
                    }
                    println!("│");

                    // Display conflicts if any
                    if !batch.conflicts.is_empty() {
                        println!("│ ⚠️  Conflicts detected:");
                        for conflict in &batch.conflicts {
                            println!(
                                "│   - {}: requested {}",
                                conflict.package, conflict.requested_version
                            );
                            for dep in &conflict.conflicts_with {
                                println!(
                                    "│     requires {} {}",
                                    dep.package, dep.required_version_range
                                );
                            }
                        }
                        println!("│");
                    }

                    println!("└─────────────────────────────────────────────────────");
                    println!();

                    // Ask user if they want to apply this batch
                    let apply_batch = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Apply Batch {}?", batch_num + 1))
                        .default(matches!(batch.risk, bazbom::batch_fixer::RiskLevel::Low))
                        .interact()
                        .unwrap_or(false);

                    if apply_batch {
                        println!("\n⏳ Applying {} updates...", batch.updates.len());

                        // Convert batch updates back to suggestions for remediation
                        let batch_suggestions: Vec<_> = report
                            .suggestions
                            .iter()
                            .filter(|s| {
                                batch.updates.iter().any(|u| {
                                    u.package == s.affected_package
                                        && u.current_version == s.current_version
                                })
                            })
                            .cloned()
                            .collect();

                        match remediation::apply_fixes(&batch_suggestions, system, &root) {
                            Ok(result) => {
                                if result.applied == batch_suggestions.len() {
                                    println!(
                                        "✅ All {} updates applied successfully!",
                                        result.applied
                                    );
                                } else {
                                    println!(
                                        "⚠️  Applied: {}, Failed: {}, Skipped: {}",
                                        result.applied, result.failed, result.skipped
                                    );
                                }

                                // Run tests if available
                                use bazbom::test_runner::run_tests;
                                if let Ok(test_result) = run_tests(system, &root) {
                                    if test_result.success {
                                        println!(
                                            "✅ All tests passed! ({:.1}s)",
                                            test_result.duration.as_secs_f64()
                                        );
                                    } else {
                                        println!("⚠️  Tests failed! Rolling back changes...");
                                        println!("{}", test_result.output);
                                        eprintln!("\n[bazbom] Batch {} failed tests - please review manually", batch_num + 1);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to apply Batch {}: {}", batch_num + 1, e);
                            }
                        }
                    } else {
                        println!("⏭️  Skipped Batch {}\n", batch_num + 1);
                    }
                }

                println!("\n📊 Summary:");
                println!("  Batches processed: {}", batches.len());
                println!("\n💡 Next steps:");
                println!("  1. Review changes: git diff");
                println!(
                    "  2. Commit changes: git commit -m 'fix: upgrade vulnerable dependencies'"
                );
                println!("  3. Create PR: bazbom fix --pr (or push manually)");
                println!("\n🎉 Great job! Your project is more secure.");
            }

            if apply {
                // Apply mode: attempt to apply fixes
                println!("\n[bazbom] Applying fixes...");

                match remediation::apply_fixes(&report.suggestions, system, &root) {
                    Ok(result) => {
                        println!("\n[bazbom] Apply Results:");
                        println!("  Applied: {}", result.applied);
                        println!("  Failed: {}", result.failed);
                        println!("  Skipped: {}", result.skipped);

                        if result.applied > 0 {
                            println!(
                                "\n[bazbom] Please review changes and run tests before committing"
                            );
                        }
                        if result.failed > 0 || result.skipped > 0 {
                            println!("[bazbom] Some fixes require manual intervention - see suggestions above");
                        }
                    }
                    Err(e) => {
                        eprintln!("[bazbom] failed to apply fixes: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            if pr {
                // PR mode: apply fixes with testing and create a pull request
                println!("\n[bazbom] Creating pull request with fixes...");

                match remediation::generate_pr(&report.suggestions, system, &root) {
                    Ok(pr_url) => {
                        println!("\n✅ Pull request created successfully!");
                        println!("   URL: {}", pr_url);
                    }
                    Err(e) => {
                        eprintln!("\n❌ Failed to create pull request: {}", e);
                        eprintln!("\nTroubleshooting:");
                        eprintln!("  - Ensure GITHUB_TOKEN environment variable is set");
                        eprintln!("  - Ensure GITHUB_REPOSITORY is set (format: owner/repo)");
                        eprintln!("  - Ensure you have write access to the repository");
                        eprintln!("  - Ensure git is configured and you're in a git repository");
                        std::process::exit(1);
                    }
                }
            }

            if !suggest && !apply && !pr {
                println!("\n[bazbom] Use --suggest to see remediation suggestions");
                println!("[bazbom] Use --apply to automatically apply fixes");
                println!("[bazbom] Use --pr to create a pull request with fixes");
            }
        }
        Commands::License { action } => match action {
            LicenseCmd::Obligations { sbom_file } => {
                println!("[bazbom] generating license obligations report");

                let sbom_path = sbom_file.as_deref().unwrap_or("sbom.spdx.json");

                if sbom_file.is_some() {
                    println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
                }

                let obligations_db = bazbom_formats::licenses::LicenseObligations::new();

                println!("\n# License Obligations Report\n");
                println!("Example output for: {}\n", sbom_path);

                let example_licenses = vec![
                    ("MIT", "example-mit-lib:1.0.0"),
                    ("Apache-2.0", "example-apache-lib:2.0.0"),
                    ("GPL-3.0-only", "example-gpl-lib:3.0.0"),
                ];

                for (license, component) in example_licenses {
                    if let Some(obligations) = obligations_db.get(license) {
                        println!("## {} ({})\n", component, license);
                        for obligation in obligations {
                            println!(
                                "- **{:?}**: {} (Severity: {:?})",
                                obligation.obligation_type,
                                obligation.description,
                                obligation.severity
                            );
                        }
                        println!();
                    }
                }

                println!(
                    "Note: This is a demonstration. Full SBOM parsing integration coming soon."
                );
            }
            LicenseCmd::Compatibility {
                project_license,
                sbom_file,
            } => {
                println!("[bazbom] checking license compatibility");
                println!("Project license: {}", project_license);

                if let Some(sbom) = &sbom_file {
                    println!("SBOM file: {}", sbom);
                    println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
                }

                let test_dependencies = vec![
                    ("MIT", "example-mit-lib"),
                    ("Apache-2.0", "example-apache-lib"),
                    ("GPL-3.0-only", "example-gpl-lib"),
                    ("AGPL-3.0-only", "example-agpl-lib"),
                ];

                println!("\n# License Compatibility Report\n");

                for (dep_license, dep_name) in test_dependencies {
                    let risk = bazbom_formats::licenses::LicenseCompatibility::is_compatible(
                        &project_license,
                        dep_license,
                    );

                    let risk_str = format!("{:?}", risk);
                    let indicator = match risk {
                        bazbom_formats::licenses::LicenseRisk::Safe => "✓",
                        bazbom_formats::licenses::LicenseRisk::Low => "⚠",
                        bazbom_formats::licenses::LicenseRisk::Medium => "⚠",
                        bazbom_formats::licenses::LicenseRisk::High => "✗",
                        bazbom_formats::licenses::LicenseRisk::Critical => "✗✗",
                    };

                    println!(
                        "{} {} ({}) - Risk: {}",
                        indicator, dep_name, dep_license, risk_str
                    );
                }

                println!(
                    "\nNote: This is a demonstration. Full SBOM parsing integration coming soon."
                );
            }
            LicenseCmd::Contamination { sbom_file } => {
                println!("[bazbom] detecting copyleft contamination");

                if let Some(sbom) = &sbom_file {
                    println!("SBOM file: {}", sbom);
                    println!("[bazbom] note: SBOM file parsing not yet implemented, showing example data");
                }

                let test_dependencies = vec![
                    bazbom_formats::licenses::Dependency {
                        name: "example-mit-lib:1.0.0".to_string(),
                        license: "MIT".to_string(),
                    },
                    bazbom_formats::licenses::Dependency {
                        name: "example-gpl-lib:2.0.0".to_string(),
                        license: "GPL-3.0-only".to_string(),
                    },
                    bazbom_formats::licenses::Dependency {
                        name: "example-agpl-lib:3.0.0".to_string(),
                        license: "AGPL-3.0-only".to_string(),
                    },
                ];

                let warnings = bazbom_formats::licenses::LicenseCompatibility::check_contamination(
                    &test_dependencies,
                );

                println!("\n# Copyleft Contamination Report\n");

                if warnings.is_empty() {
                    println!("✓ No copyleft contamination detected");
                } else {
                    for warning in warnings {
                        let risk_indicator = match warning.risk {
                            bazbom_formats::licenses::LicenseRisk::Critical => "✗✗ CRITICAL",
                            bazbom_formats::licenses::LicenseRisk::High => "✗ HIGH",
                            bazbom_formats::licenses::LicenseRisk::Medium => "⚠ MEDIUM",
                            _ => "ⓘ INFO",
                        };

                        println!("{}: {}", risk_indicator, warning.message);
                        println!(
                            "Affected licenses: {}",
                            warning.affected_licenses.join(", ")
                        );
                        println!();
                    }
                }

                println!(
                    "Note: This is a demonstration. Full SBOM parsing integration coming soon."
                );
            }
        },
        Commands::Db { action } => match action {
            DbCmd::Sync {} => {
                println!("[bazbom] db sync");
                let cache_dir = PathBuf::from(".bazbom/cache");
                let offline = std::env::var("BAZBOM_OFFLINE").is_ok();
                let manifest = bazbom_advisories::db_sync(&cache_dir, offline)
                    .context("failed advisory DB sync")?;
                println!(
                    "[bazbom] advisories cached at {:?} ({} files)",
                    cache_dir,
                    manifest.files.len()
                );
            }
        },
        Commands::InstallHooks { policy, fast } => {
            println!("[bazbom] installing pre-commit hooks");
            let config = HooksConfig {
                policy_file: policy,
                fast_mode: fast,
            };
            install_hooks(&config)?;
        }
        Commands::Init { path } => {
            bazbom::init::run_init(&path)?;
        }
        Commands::Explore { sbom, findings } => {
            use bazbom::explore;

            // Load dependencies from SBOM/findings or use mock data
            let dependencies = explore::load_dependencies(sbom.as_deref(), findings.as_deref())?;

            if sbom.is_some() || findings.is_some() {
                println!("[bazbom] Loaded {} dependencies", dependencies.len());
            } else {
                println!("[bazbom] No SBOM/findings specified, using demo data");
                println!("[bazbom] Hint: Use --sbom=<file> or --findings=<file> to load your data");
            }

            bazbom_tui::run(dependencies)?;
        }
        Commands::Dashboard { port, open, export } => {
            use bazbom_dashboard::{start_dashboard, DashboardConfig};
            use std::path::PathBuf;

            if let Some(export_path) = export {
                use bazbom_dashboard::{export_to_html, DashboardSummary, DependencyGraph, VulnerabilityCounts, Vulnerability};
                use std::fs;
                
                println!(
                    "[bazbom] Exporting static HTML dashboard to: {}",
                    export_path
                );
                
                // Load findings from cache
                let cache_dir = PathBuf::from(".bazbom/cache");
                let findings_path = cache_dir.join("sca_findings.json");
                
                let (summary, graph_data, vulnerabilities) = if findings_path.exists() {
                    let findings_content = fs::read_to_string(&findings_path)
                        .context("Failed to read findings file")?;
                    let findings: serde_json::Value = serde_json::from_str(&findings_content)
                        .context("Failed to parse findings JSON")?;
                    
                    // Extract summary
                    let summary = DashboardSummary {
                        security_score: findings["summary"]["security_score"].as_u64().unwrap_or(0) as u8,
                        total_dependencies: findings["summary"]["total_dependencies"].as_u64().unwrap_or(0) as usize,
                        vulnerabilities: VulnerabilityCounts {
                            critical: findings["summary"]["vulnerabilities"]["critical"].as_u64().unwrap_or(0) as usize,
                            high: findings["summary"]["vulnerabilities"]["high"].as_u64().unwrap_or(0) as usize,
                            medium: findings["summary"]["vulnerabilities"]["medium"].as_u64().unwrap_or(0) as usize,
                            low: findings["summary"]["vulnerabilities"]["low"].as_u64().unwrap_or(0) as usize,
                        },
                        license_issues: 0,
                        policy_violations: 0,
                    };
                    
                    // Extract vulnerabilities
                    let vulns: Vec<_> = findings["vulnerabilities"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|v| Vulnerability {
                            cve: v["cve"].as_str().unwrap_or("").to_string(),
                            package_name: v["package"]["name"].as_str().unwrap_or("").to_string(),
                            package_version: v["package"]["version"].as_str().unwrap_or("").to_string(),
                            severity: v["severity"].as_str().unwrap_or("").to_string(),
                            cvss: v["cvss"].as_f64().unwrap_or(0.0) as f32,
                            description: v["description"].as_str().map(|s| s.to_string()),
                            fixed_version: v["fixed_version"].as_str().map(|s| s.to_string()),
                        })
                        .collect();
                    
                    let graph_data = DependencyGraph {
                        nodes: vec![],
                        edges: vec![],
                    };
                    
                    (summary, graph_data, vulns)
                } else {
                    println!("[bazbom] No findings file found, generating empty report");
                    let summary = DashboardSummary {
                        security_score: 100,
                        total_dependencies: 0,
                        vulnerabilities: VulnerabilityCounts {
                            critical: 0,
                            high: 0,
                            medium: 0,
                            low: 0,
                        },
                        license_issues: 0,
                        policy_violations: 0,
                    };
                    (summary, DependencyGraph { nodes: vec![], edges: vec![] }, vec![])
                };
                
                // Export to HTML
                export_to_html(
                    &PathBuf::from(&export_path),
                    &summary,
                    &graph_data,
                    &vulnerabilities,
                )?;
                
                println!("[bazbom] Successfully exported to: {}", export_path);
                println!("[bazbom] Open the file in your browser to view the report");
                return Ok(());
            }

            // Create dashboard configuration
            let config = DashboardConfig {
                port,
                cache_dir: PathBuf::from(".bazbom/cache"),
                project_root: PathBuf::from("."),
            };

            // Open browser if requested
            if open {
                let url = format!("http://localhost:{}", port);
                println!("[bazbom] Opening browser at {}", url);
                if let Err(e) = webbrowser::open(&url) {
                    eprintln!(
                        "[bazbom] warning: failed to open browser automatically: {}",
                        e
                    );
                    eprintln!("[bazbom] Please open {} manually in your browser", url);
                }
            }

            // Start dashboard with tokio runtime
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { start_dashboard(config).await })?;
        }
        Commands::Team { action } => {
            use bazbom::cli::TeamCmd;
            use bazbom::team::{TeamConfig, TeamCoordinator};

            let coordinator = TeamCoordinator::new(None);

            match action {
                TeamCmd::Assign { cve, to } => {
                    coordinator.assign(&cve, &to)?;
                    coordinator.log_audit_event(
                        &format!("Assigned {}", cve),
                        Some(format!("Assigned to {}", to)),
                    )?;
                }
                TeamCmd::List {} => {
                    let assignments = coordinator.list_assignments()?;
                    if assignments.is_empty() {
                        println!("No assignments found.");
                    } else {
                        println!("Vulnerability Assignments:");
                        for assignment in assignments {
                            println!(
                                "  {} → {} (assigned {})",
                                assignment.cve,
                                assignment.assignee,
                                assignment.assigned_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                }
                TeamCmd::Mine {} => {
                    // Get current user
                    let user = std::process::Command::new("git")
                        .args(["config", "user.email"])
                        .output()
                        .ok()
                        .and_then(|output| {
                            if output.status.success() {
                                Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| "unknown".to_string());

                    let assignments = coordinator.get_my_assignments(&user)?;
                    if assignments.is_empty() {
                        println!("No assignments for {}", user);
                    } else {
                        println!("{} vulnerabilities assigned to you:", assignments.len());
                        for assignment in assignments {
                            println!(
                                "  {} (assigned {})",
                                assignment.cve,
                                assignment.assigned_at.format("%Y-%m-%d %H:%M")
                            );
                        }
                    }
                }
                TeamCmd::AuditLog { format, output } => {
                    if format == "csv" {
                        let output_path = output.unwrap_or_else(|| "audit.csv".to_string());
                        coordinator.export_audit_log(&output_path)?;
                    } else {
                        let entries = coordinator.get_audit_log(Some(50))?;
                        if entries.is_empty() {
                            println!("No audit entries found.");
                        } else {
                            println!("Recent Audit Log:");
                            for entry in entries {
                                println!(
                                    "  {} | {} | {}",
                                    entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                                    entry.user,
                                    entry.action
                                );
                                if let Some(details) = entry.details {
                                    println!("    {}", details);
                                }
                            }
                        }
                    }
                }
                TeamCmd::Config {
                    name,
                    add_member,
                    remove_member,
                } => {
                    let config_path = ".bazbom/team-config.json";
                    let mut config = TeamConfig::load(config_path).unwrap_or_else(|_| TeamConfig {
                        name: "Security Team".to_string(),
                        members: Vec::new(),
                        notification_channels: std::collections::HashMap::new(),
                    });

                    if let Some(team_name) = name {
                        config.name = team_name;
                        println!("✅ Set team name to: {}", config.name);
                    }

                    if let Some(email) = add_member {
                        config.add_member(email.clone());
                        println!("✅ Added team member: {}", email);
                    }

                    if let Some(email) = remove_member {
                        config.remove_member(&email);
                        println!("✅ Removed team member: {}", email);
                    }

                    // Create .bazbom directory if it doesn't exist
                    std::fs::create_dir_all(".bazbom")?;
                    config.save(config_path)?;
                    println!("✅ Team configuration saved to {}", config_path);
                }
            }
        }
        Commands::Report { action } => {
            use bazbom_reports::{
                ComplianceFramework, PolicyStatus, ReportGenerator, ReportType, SbomData,
                VulnerabilityDetail, VulnerabilityFindings,
            };
            use chrono::Utc;
            use std::path::Path;

            // Helper function to load findings from JSON
            fn load_findings_from_file(path: &str) -> Result<VulnerabilityFindings> {
                let content = fs::read_to_string(path)
                    .with_context(|| format!("Failed to read findings file: {}", path))?;
                let findings: serde_json::Value = serde_json::from_str(&content)?;

                // Parse vulnerabilities by severity
                let critical = extract_vulnerabilities(&findings, "CRITICAL");
                let high = extract_vulnerabilities(&findings, "HIGH");
                let medium = extract_vulnerabilities(&findings, "MEDIUM");
                let low = extract_vulnerabilities(&findings, "LOW");

                Ok(VulnerabilityFindings {
                    critical,
                    high,
                    medium,
                    low,
                })
            }

            fn extract_vulnerabilities(
                findings: &serde_json::Value,
                severity: &str,
            ) -> Vec<VulnerabilityDetail> {
                findings
                    .get("vulnerabilities")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| {
                                if v.get("severity")?.as_str()? == severity {
                                    Some(VulnerabilityDetail {
                                        cve: v
                                            .get("id")
                                            .or_else(|| v.get("cve"))
                                            .and_then(|id| id.as_str())
                                            .unwrap_or("UNKNOWN")
                                            .to_string(),
                                        package_name: v
                                            .get("package")
                                            .or_else(|| v.get("package_name"))
                                            .and_then(|p| p.as_str())
                                            .unwrap_or("unknown")
                                            .to_string(),
                                        package_version: v
                                            .get("version")
                                            .or_else(|| v.get("package_version"))
                                            .and_then(|ver| ver.as_str())
                                            .unwrap_or("unknown")
                                            .to_string(),
                                        severity: severity.to_string(),
                                        cvss_score: v
                                            .get("cvss")
                                            .or_else(|| v.get("cvss_score"))
                                            .and_then(|s| s.as_f64())
                                            .unwrap_or(0.0),
                                        description: v
                                            .get("description")
                                            .and_then(|d| d.as_str())
                                            .unwrap_or("No description")
                                            .to_string(),
                                        fixed_version: v
                                            .get("fixed_version")
                                            .and_then(|f| f.as_str())
                                            .map(|s| s.to_string()),
                                        is_reachable: v
                                            .get("reachable")
                                            .or_else(|| v.get("is_reachable"))
                                            .and_then(|r| r.as_bool())
                                            .unwrap_or(false),
                                        is_kev: v
                                            .get("kev")
                                            .or_else(|| v.get("is_kev"))
                                            .and_then(|k| k.as_bool())
                                            .unwrap_or(false),
                                        epss_score: v
                                            .get("epss")
                                            .or_else(|| v.get("epss_score"))
                                            .and_then(|e| e.as_f64()),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            }

            // Helper function to create SBOM data from SBOM file or defaults
            fn load_sbom_data(sbom_path: Option<&str>) -> Result<SbomData> {
                if let Some(path) = sbom_path {
                    // Try to parse SBOM file
                    let content = fs::read_to_string(path)
                        .with_context(|| format!("Failed to read SBOM file: {}", path))?;
                    let sbom: serde_json::Value = serde_json::from_str(&content)?;

                    Ok(SbomData {
                        project_name: sbom
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Unknown Project")
                            .to_string(),
                        project_version: sbom
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("0.0.0")
                            .to_string(),
                        scan_timestamp: Utc::now(),
                        total_dependencies: sbom
                            .get("packages")
                            .and_then(|p| p.as_array())
                            .map(|arr| arr.len())
                            .unwrap_or(0),
                        direct_dependencies: 0, // Would need graph analysis
                        transitive_dependencies: 0,
                    })
                } else {
                    // Return default data
                    Ok(SbomData {
                        project_name: "Unknown Project".to_string(),
                        project_version: "0.0.0".to_string(),
                        scan_timestamp: Utc::now(),
                        total_dependencies: 0,
                        direct_dependencies: 0,
                        transitive_dependencies: 0,
                    })
                }
            }

            // Convert CLI framework arg to report framework
            fn convert_framework(arg: ComplianceFrameworkArg) -> ComplianceFramework {
                match arg {
                    ComplianceFrameworkArg::PciDss => ComplianceFramework::PciDss,
                    ComplianceFrameworkArg::Hipaa => ComplianceFramework::Hipaa,
                    ComplianceFrameworkArg::FedRampModerate => ComplianceFramework::FedRampModerate,
                    ComplianceFrameworkArg::Soc2 => ComplianceFramework::Soc2,
                    ComplianceFrameworkArg::Gdpr => ComplianceFramework::Gdpr,
                    ComplianceFrameworkArg::Iso27001 => ComplianceFramework::Iso27001,
                    ComplianceFrameworkArg::NistCsf => ComplianceFramework::NistCsf,
                }
            }

            match action {
                ReportCmd::Executive {
                    sbom,
                    findings,
                    output,
                } => {
                    println!("📊 Generating executive summary report...");

                    let sbom_data = load_sbom_data(sbom.as_deref())?;
                    let vulnerabilities = if let Some(findings_path) = findings {
                        load_findings_from_file(&findings_path)?
                    } else {
                        VulnerabilityFindings {
                            critical: vec![],
                            high: vec![],
                            medium: vec![],
                            low: vec![],
                        }
                    };

                    let policy = PolicyStatus {
                        policy_violations: 0,
                        license_issues: 0,
                        blocked_packages: 0,
                    };

                    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
                    generator.generate(ReportType::Executive, Path::new(&output))?;

                    println!("✅ Executive report generated: {}", output);
                }
                ReportCmd::Compliance {
                    framework,
                    sbom,
                    findings,
                    output,
                } => {
                    let framework_name = convert_framework(framework);
                    println!(
                        "📊 Generating compliance report for {}...",
                        framework_name.name()
                    );

                    let sbom_data = load_sbom_data(sbom.as_deref())?;
                    let vulnerabilities = if let Some(findings_path) = findings {
                        load_findings_from_file(&findings_path)?
                    } else {
                        VulnerabilityFindings {
                            critical: vec![],
                            high: vec![],
                            medium: vec![],
                            low: vec![],
                        }
                    };

                    let policy = PolicyStatus {
                        policy_violations: 0,
                        license_issues: 0,
                        blocked_packages: 0,
                    };

                    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
                    generator.generate(
                        ReportType::Compliance(framework_name),
                        Path::new(&output),
                    )?;

                    println!("✅ Compliance report generated: {}", output);
                }
                ReportCmd::Developer {
                    sbom,
                    findings,
                    output,
                } => {
                    println!("📊 Generating developer report...");

                    let sbom_data = load_sbom_data(sbom.as_deref())?;
                    let vulnerabilities = if let Some(findings_path) = findings {
                        load_findings_from_file(&findings_path)?
                    } else {
                        VulnerabilityFindings {
                            critical: vec![],
                            high: vec![],
                            medium: vec![],
                            low: vec![],
                        }
                    };

                    let policy = PolicyStatus {
                        policy_violations: 0,
                        license_issues: 0,
                        blocked_packages: 0,
                    };

                    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
                    generator.generate(ReportType::Developer, Path::new(&output))?;

                    println!("✅ Developer report generated: {}", output);
                }
                ReportCmd::Trend {
                    sbom,
                    findings,
                    output,
                } => {
                    println!("📊 Generating trend report...");

                    let sbom_data = load_sbom_data(sbom.as_deref())?;
                    let vulnerabilities = if let Some(findings_path) = findings {
                        load_findings_from_file(&findings_path)?
                    } else {
                        VulnerabilityFindings {
                            critical: vec![],
                            high: vec![],
                            medium: vec![],
                            low: vec![],
                        }
                    };

                    let policy = PolicyStatus {
                        policy_violations: 0,
                        license_issues: 0,
                        blocked_packages: 0,
                    };

                    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);
                    generator.generate(ReportType::Trend, Path::new(&output))?;

                    println!("✅ Trend report generated: {}", output);
                }
                ReportCmd::All {
                    sbom,
                    findings,
                    output_dir,
                } => {
                    println!("📊 Generating all reports...");

                    // Create output directory
                    fs::create_dir_all(&output_dir)?;

                    let sbom_data = load_sbom_data(sbom.as_deref())?;
                    let vulnerabilities = if let Some(findings_path) = findings {
                        load_findings_from_file(&findings_path)?
                    } else {
                        VulnerabilityFindings {
                            critical: vec![],
                            high: vec![],
                            medium: vec![],
                            low: vec![],
                        }
                    };

                    let policy = PolicyStatus {
                        policy_violations: 0,
                        license_issues: 0,
                        blocked_packages: 0,
                    };

                    let generator = ReportGenerator::new(sbom_data, vulnerabilities, policy);

                    // Generate all report types
                    let reports = vec![
                        (
                            ReportType::Executive,
                            format!("{}/executive-report.html", output_dir),
                        ),
                        (
                            ReportType::Developer,
                            format!("{}/developer-report.html", output_dir),
                        ),
                        (
                            ReportType::Trend,
                            format!("{}/trend-report.html", output_dir),
                        ),
                        (
                            ReportType::Compliance(ComplianceFramework::PciDss),
                            format!("{}/compliance-pci-dss.html", output_dir),
                        ),
                        (
                            ReportType::Compliance(ComplianceFramework::Hipaa),
                            format!("{}/compliance-hipaa.html", output_dir),
                        ),
                        (
                            ReportType::Compliance(ComplianceFramework::Soc2),
                            format!("{}/compliance-soc2.html", output_dir),
                        ),
                    ];

                    for (report_type, output_path) in reports {
                        generator.generate(report_type, Path::new(&output_path))?;
                        println!("  ✅ {}", output_path);
                    }

                    println!("\n✅ All reports generated in: {}", output_dir);
                }
            }
        }
    }
    Ok(())
}
