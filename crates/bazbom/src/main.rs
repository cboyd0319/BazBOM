use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use bazbom_core::{detect_build_system, write_stub_sbom, BuildSystem};

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
    match cli.command.unwrap_or(Commands::Scan { path: ".".into(), reachability: false, format: "spdx".into(), out_dir: ".".into() }) {
        Commands::Scan { path, reachability, format, out_dir } => {
            let root = PathBuf::from(&path);
            let system = detect_build_system(&root);
            println!("[bazbom] scan path={} reachability={} format={} system={:?}", path, reachability, format, system);
            let out = PathBuf::from(&out_dir);
            let sbom_path = write_stub_sbom(&out, &format, system)
                .with_context(|| format!("failed writing stub SBOM to {:?}", out))?;
            println!("[bazbom] wrote {:?}", sbom_path);
            // Create placeholder findings file
            let findings_path = out.join("sca_findings.json");
            if !findings_path.exists() {
                fs::write(&findings_path, b"{\n  \"vulnerabilities\": []\n}\n")
                    .with_context(|| format!("failed writing {:?}", findings_path))?;
            }
            // Create minimal SARIF stub
            let sarif_path = out.join("sca_findings.sarif");
            if !sarif_path.exists() {
                let sarif = serde_json::json!({
                    "version": "2.1.0",
                    "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
                    "runs": [
                        {"tool": {"driver": {"name": "bazbom", "version": bazbom_core::VERSION}}}
                    ]
                });
                fs::write(&sarif_path, serde_json::to_vec_pretty(&sarif).unwrap())
                    .with_context(|| format!("failed writing {:?}", sarif_path))?;
            }

            if reachability {
                // Attempt to run reachability skeleton if configured
                if let Ok(jar) = std::env::var("BAZBOM_REACHABILITY_JAR") {
                    let out_file = out.join("reachability.json");
                    let status = std::process::Command::new("java")
                        .arg("-cp")
                        .arg(&jar)
                        .arg("io.bazbom.reachability.Main")
                        .arg("--classpath").arg("")
                        .arg("--entrypoints").arg("")
                        .arg("--output").arg(&out_file)
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            println!("[bazbom] reachability wrote {:?}", out_file);
                            if let Ok(bytes) = fs::read(&out_file) {
                                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                                    println!("[bazbom] reachability summary: keys={}", val.as_object().map(|m| m.len()).unwrap_or(0));
                                }
                            }
                        }
                        Ok(s) => eprintln!("[bazbom] reachability failed with status {:?}", s),
                        Err(e) => eprintln!("[bazbom] failed to invoke java: {}", e),
                    }
                } else {
                    eprintln!("[bazbom] --reachability set but BAZBOM_REACHABILITY_JAR not configured; skipping");
                }
            }
        }
        Commands::Policy { action } => match action {
            PolicyCmd::Check {} => {
                println!("[bazbom] policy check");
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
                println!("[bazbom] advisories cached at {:?} ({} files)", cache_dir, manifest.files.len());
            }
        },
    }
    Ok(())
}
