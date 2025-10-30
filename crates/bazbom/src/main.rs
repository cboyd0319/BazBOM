use anyhow::{Context, Result};
use bazbom_core::{detect_build_system, write_stub_sbom};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod advisory;
mod bazel;
mod policy_integration;
mod reachability;
mod reachability_cache;
mod shading;

#[derive(Parser, Debug)]
#[command(name = "bazbom", version, about = "JVM SBOM, SCA, and dependency graph tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Scan a project and generate SBOM + findings
    Scan {
        /// Path to project (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
        /// Enable reachability analysis (OPAL)
        #[arg(long)]
        reachability: bool,
        /// Output format (spdx|cyclonedx)
        #[arg(long, default_value = "spdx")]
        format: String,
        /// Output directory (defaults to current directory)
        #[arg(long, value_name = "DIR", default_value = ".")]
        out_dir: String,
        /// Bazel-specific: query expression to select targets (e.g., 'kind(java_binary, //...)')
        #[arg(long, value_name = "QUERY")]
        bazel_targets_query: Option<String>,
        /// Bazel-specific: explicit list of targets to scan
        #[arg(long, value_name = "TARGET", num_args = 1..)]
        bazel_targets: Option<Vec<String>>,
        /// Bazel-specific: scan only targets affected by these files (incremental mode)
        #[arg(long, value_name = "FILE", num_args = 1..)]
        bazel_affected_by_files: Option<Vec<String>>,
        /// Bazel-specific: universe pattern for rdeps queries (default: //...)
        #[arg(long, value_name = "PATTERN", default_value = "//...")]
        bazel_universe: String,
    },
    /// Apply policy checks and output SARIF/JSON verdicts
    Policy {
        #[command(subcommand)]
        action: PolicyCmd,
    },
    /// Show remediation suggestions or apply fixes
    Fix {
        /// Suggest fixes without applying changes
        #[arg(long)]
        suggest: bool,
        /// Apply fixes and open PRs
        #[arg(long)]
        apply: bool,
    },
    /// Advisory database operations (offline sync)
    Db {
        #[command(subcommand)]
        action: DbCmd,
    },
}

#[derive(Subcommand, Debug)]
enum PolicyCmd {
    /// Run policy checks
    Check {},
}

#[derive(Subcommand, Debug)]
enum DbCmd {
    /// Sync local advisory mirrors for offline use
    Sync {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command.unwrap_or(Commands::Scan {
        path: ".".into(),
        reachability: false,
        format: "spdx".into(),
        out_dir: ".".into(),
        bazel_targets_query: None,
        bazel_targets: None,
        bazel_affected_by_files: None,
        bazel_universe: "//...".into(),
    }) {
        Commands::Scan {
            path,
            reachability,
            format,
            out_dir,
            bazel_targets_query,
            bazel_targets,
            bazel_affected_by_files,
            bazel_universe,
        } => {
            let root = PathBuf::from(&path);
            let system = detect_build_system(&root);
            
            // Handle Bazel-specific target selection
            let bazel_targets_to_scan = if system == bazbom_core::BuildSystem::Bazel {
                if let Some(query) = &bazel_targets_query {
                    println!("[bazbom] using Bazel query: {}", query);
                    match bazel::query_bazel_targets(&root, Some(query), None, None, &bazel_universe) {
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
                    match bazel::query_bazel_targets(&root, None, None, Some(files), &bazel_universe) {
                        Ok(targets) => Some(targets),
                        Err(e) => {
                            eprintln!("[bazbom] warning: failed to find affected targets: {}", e);
                            None
                        }
                    }
                } else {
                    None  // Scan all targets
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
            
            println!(
                "[bazbom] scan path={} reachability={} format={} system={:?}",
                path, reachability, format, system
            );
            let out = PathBuf::from(&out_dir);
            
            // For Bazel projects, extract dependencies and generate SBOM
            let sbom_path = if system == bazbom_core::BuildSystem::Bazel {
                let deps_json_path = out.join("bazel_deps.json");
                
                // If we have specific targets, extract dependencies for those
                let extraction_result = if let Some(targets) = &bazel_targets_to_scan {
                    if !targets.is_empty() {
                        bazel::extract_bazel_dependencies_for_targets(&root, targets, &deps_json_path)
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
                        eprintln!("[bazbom] warning: failed to extract Bazel dependencies: {}", e);
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

            // Run reachability analysis if requested
            let reachability_result = if reachability {
                // Attempt to run reachability analysis if configured
                if let Ok(jar) = std::env::var("BAZBOM_REACHABILITY_JAR") {
                    let jar_path = PathBuf::from(&jar);
                    if !jar_path.exists() {
                        eprintln!("[bazbom] BAZBOM_REACHABILITY_JAR points to non-existent file: {:?}", jar_path);
                        None
                    } else {
                        let out_file = out.join("reachability.json");
                        
                        // Extract classpath based on build system
                        let classpath = match system {
                            bazbom_core::BuildSystem::Maven => {
                                reachability::extract_maven_classpath(&root)
                                    .unwrap_or_else(|e| {
                                        eprintln!("[bazbom] failed to extract Maven classpath: {}", e);
                                        String::new()
                                    })
                            }
                            bazbom_core::BuildSystem::Gradle => {
                                reachability::extract_gradle_classpath(&root)
                                    .unwrap_or_else(|e| {
                                        eprintln!("[bazbom] failed to extract Gradle classpath: {}", e);
                                        String::new()
                                    })
                            }
                            bazbom_core::BuildSystem::Bazel => {
                                reachability::extract_bazel_classpath(&root, "")
                                    .unwrap_or_else(|e| {
                                        eprintln!("[bazbom] failed to extract Bazel classpath: {}", e);
                                        String::new()
                                    })
                            }
                            _ => String::new(),
                        };
                        
                        let entrypoints = "";
                        let cache_dir = reachability_cache::get_cache_dir();
                        
                        // Check cache first
                        let result = if let Ok(Some(cached)) = reachability_cache::load_cached_result(
                            &cache_dir,
                            &classpath,
                            entrypoints,
                        ) {
                            println!("[bazbom] using cached reachability result");
                            Some(cached)
                        } else {
                            // Run analysis and cache result
                            match reachability::analyze_reachability(&jar_path, &classpath, entrypoints, &out_file) {
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
                    eprintln!("[bazbom] --reachability set but BAZBOM_REACHABILITY_JAR not configured");
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
                                println!("[bazbom] detected Maven Shade plugin with {} relocations", 
                                         config.relocations.len());
                                Some(config)
                            }
                            Ok(None) => None,
                            Err(e) => {
                                eprintln!("[bazbom] warning: failed to parse Maven Shade config: {}", e);
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
                                println!("[bazbom] detected Gradle Shadow plugin with {} relocations", 
                                         config.relocations.len());
                                Some(config)
                            }
                            Ok(None) => None,
                            Err(e) => {
                                eprintln!("[bazbom] warning: failed to parse Gradle Shadow config: {}", e);
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
            
            fs::write(&findings_path, serde_json::to_vec_pretty(&findings_data).unwrap())
                .with_context(|| format!("failed writing {:?}", findings_path))?;
            println!("[bazbom] wrote {:?} ({} vulnerabilities)", findings_path, vulnerabilities.len());

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
                let info_result = bazbom_formats::sarif::Result::new(
                    "shading/detected",
                    "note",
                    &shading_note
                );
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
                
                let mut message = vuln.summary.clone()
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
                    println!("[bazbom] ⚠ policy violations detected ({} violations)", policy_result.violations.len());
                    for violation in &policy_result.violations {
                        println!("  - {}: {}", violation.rule, violation.message);
                    }
                    
                    // Write policy violations to separate file
                    let policy_violations_path = out.join("policy_violations.json");
                    fs::write(&policy_violations_path, serde_json::to_vec_pretty(&policy_result).unwrap())
                        .with_context(|| format!("failed writing {:?}", policy_violations_path))?;
                    println!("[bazbom] wrote {:?}", policy_violations_path);
                } else {
                    println!("[bazbom] ✓ all policy checks passed");
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
                println!("[bazbom] loaded policy config (threshold={:?})", policy.severity_threshold);
                
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
                let mut sarif = bazbom_formats::sarif::SarifReport::new("bazbom-policy", bazbom_core::VERSION);
                
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
                    let result_item = bazbom_formats::sarif::Result::new(&rule_id, level, &violation.message);
                    sarif.add_result(result_item);
                }
                
                fs::write(&sarif_path, serde_json::to_vec_pretty(&sarif).unwrap())
                    .with_context(|| format!("failed writing {:?}", sarif_path))?;
                println!("[bazbom] wrote {:?} ({} violations)", sarif_path, result.violations.len());
                
                // Print summary
                if result.passed {
                    println!("[bazbom] ✓ policy check passed (no violations)");
                } else {
                    println!("[bazbom] ✗ policy check failed ({} violations)", result.violations.len());
                    for violation in &result.violations {
                        println!("  - {}: {}", violation.rule, violation.message);
                    }
                    std::process::exit(1);
                }
            }
        },
        Commands::Fix { suggest, apply } => {
            println!("[bazbom] fix suggest={} apply={}", suggest, apply);
        }
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
    }
    Ok(())
}
