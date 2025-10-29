use anyhow::{Context, Result};
use bazbom_core::{detect_build_system, write_stub_sbom};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod advisory;
mod policy_integration;
mod reachability;

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
    }) {
        Commands::Scan {
            path,
            reachability,
            format,
            out_dir,
        } => {
            let root = PathBuf::from(&path);
            let system = detect_build_system(&root);
            println!(
                "[bazbom] scan path={} reachability={} format={} system={:?}",
                path, reachability, format, system
            );
            let out = PathBuf::from(&out_dir);
            let sbom_path = write_stub_sbom(&out, &format, system)
                .with_context(|| format!("failed writing stub SBOM to {:?}", out))?;
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

            // Create findings file with vulnerability data
            let findings_path = out.join("sca_findings.json");
            let findings_data = serde_json::json!({
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
            fs::write(&findings_path, serde_json::to_vec_pretty(&findings_data).unwrap())
                .with_context(|| format!("failed writing {:?}", findings_path))?;
            println!("[bazbom] wrote {:?} ({} vulnerabilities)", findings_path, vulnerabilities.len());

            // Create SARIF report with vulnerability results
            let sarif_path = out.join("sca_findings.sarif");
            let mut sarif = bazbom_formats::sarif::SarifReport::new("bazbom", bazbom_core::VERSION);
            
            // Add vulnerability results to SARIF
            for vuln in &vulnerabilities {
                let level = match vuln.severity.as_ref().map(|s| s.level) {
                    Some(bazbom_advisories::SeverityLevel::Critical) => "error",
                    Some(bazbom_advisories::SeverityLevel::High) => "error",
                    Some(bazbom_advisories::SeverityLevel::Medium) => "warning",
                    Some(bazbom_advisories::SeverityLevel::Low) => "note",
                    _ => "note",
                };
                
                let message = vuln.summary.clone()
                    .or_else(|| vuln.details.clone())
                    .unwrap_or_else(|| format!("Vulnerability {}", vuln.id));
                
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
                
                let policy_result = policy_integration::check_policy(&vulnerabilities, &policy);
                
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

            if reachability {
                // Attempt to run reachability analysis if configured
                if let Ok(jar) = std::env::var("BAZBOM_REACHABILITY_JAR") {
                    let jar_path = PathBuf::from(&jar);
                    if !jar_path.exists() {
                        eprintln!("[bazbom] BAZBOM_REACHABILITY_JAR points to non-existent file: {:?}", jar_path);
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
                        
                        match reachability::analyze_reachability(&jar_path, &classpath, "", &out_file) {
                            Ok(result) => {
                                println!("[bazbom] reachability analysis complete");
                                if result.reachable_classes.is_empty() {
                                    println!("[bazbom] no reachable classes found (classpath may be empty)");
                                }
                            }
                            Err(e) => {
                                eprintln!("[bazbom] reachability analysis failed: {}", e);
                            }
                        }
                    }
                } else {
                    eprintln!("[bazbom] --reachability set but BAZBOM_REACHABILITY_JAR not configured");
                    eprintln!("[bazbom] set BAZBOM_REACHABILITY_JAR to the path of bazbom-reachability.jar");
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
