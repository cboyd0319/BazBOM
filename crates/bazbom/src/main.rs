use anyhow::Result;
use clap::Parser;

mod advisory;
mod bazel;
mod ci_templates;
mod commands;
mod errors;
mod output;
mod policy_integration;
mod reachability;
mod reachability_cache;
mod scan;
mod shading;
mod smart_defaults;
mod suggestions;

use bazbom::cli::{Cli, Commands};
use commands::*;

/// Auto-detect the main module/subproject in a workspace
///
/// Looks for common project indicators and returns the most likely main module:
/// - Maven: pom.xml (prefers root or packaging=jar/war)
/// - Gradle: build.gradle (prefers root)
/// - JavaScript: package.json (prefers root with dependencies)
/// - Rust: Cargo.toml (prefers workspace root or binary crate)
/// - Go: go.mod (prefers root)
/// - Python: setup.py, pyproject.toml (prefers root)
fn auto_detect_main_module(base_path: &str) -> Option<String> {
    use std::fs;
    use std::path::Path;

    let base = Path::new(base_path);

    // Try Maven (pom.xml)
    if let Ok(entries) = fs::read_dir(base) {
        let pom_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name() == "pom.xml")
            .collect();

        if !pom_files.is_empty() {
            // Prefer root pom.xml
            if base.join("pom.xml").exists() {
                return Some(".".to_string());
            }
        }
    }

    // Try Gradle (build.gradle or build.gradle.kts)
    if base.join("build.gradle").exists() || base.join("build.gradle.kts").exists() {
        return Some(".".to_string());
    }

    // Try JavaScript/Node (package.json)
    if base.join("package.json").exists() {
        return Some(".".to_string());
    }

    // Try Rust (Cargo.toml)
    if base.join("Cargo.toml").exists() {
        return Some(".".to_string());
    }

    // Try Go (go.mod)
    if base.join("go.mod").exists() {
        return Some(".".to_string());
    }

    // Try Python (setup.py, pyproject.toml, or requirements.txt)
    if base.join("setup.py").exists() || base.join("pyproject.toml").exists() {
        return Some(".".to_string());
    }

    // Look for subdirectories with build files (common in monorepos)
    if let Ok(entries) = fs::read_dir(base) {
        let mut candidates = Vec::new();

        for entry in entries.filter_map(|e| e.ok()) {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }

            let dir_name = entry.file_name().to_string_lossy().to_string();

            // Skip common non-project directories
            if dir_name.starts_with('.') || dir_name == "node_modules" || dir_name == "target" || dir_name == "build" {
                continue;
            }

            let dir_path = entry.path();

            // Check for project indicators
            if dir_path.join("pom.xml").exists()
                || dir_path.join("build.gradle").exists()
                || dir_path.join("package.json").exists()
                || dir_path.join("Cargo.toml").exists()
                || dir_path.join("go.mod").exists()
            {
                candidates.push(dir_name);
            }
        }

        // Prefer directories named "app", "main", "core", "server", or "api"
        for preferred in &["app", "main", "core", "server", "api"] {
            if candidates.contains(&preferred.to_string()) {
                return Some(preferred.to_string());
            }
        }

        // Return first candidate if any
        if !candidates.is_empty() {
            return Some(candidates[0].clone());
        }
    }

    None
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Scan {
        path: ".".into(),
        profile: None,
        reachability: false,
        fast: false,
        format: "spdx".into(),
        out_dir: ".".into(),
        json: false,
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
        diff: false,
        baseline: None,
        benchmark: false,
        ml_risk: false,
    });

    match command {
        Commands::Scan {
            path,
            profile,
            reachability,
            fast,
            format,
            out_dir,
            json,
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
            diff,
            baseline,
            benchmark,
            ml_risk,
        } => handle_scan(
            path,
            profile,
            reachability,
            fast,
            format,
            out_dir,
            json,
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
            diff,
            baseline,
            benchmark,
            ml_risk,
        ),

        // ========== QUICK COMMAND HANDLERS ==========
        Commands::Check { path } => {
            println!("🚀 Running quick local check (fast mode)...\n");

            // Auto-detect main module if scanning current directory
            let scan_path = if path == "." {
                match auto_detect_main_module(&path) {
                    Some(detected) => {
                        println!("📍 Auto-detected main module: {}\n", detected);
                        detected
                    }
                    None => {
                        println!("ℹ️  Scanning entire workspace (no main module detected)\n");
                        path
                    }
                }
            } else {
                path
            };

            handle_scan(
                scan_path,
                None,              // profile
                false,             // reachability
                true,              // fast
                "spdx".into(),     // format
                ".".into(),        // out_dir
                false,             // json
                None,              // bazel_targets_query
                None,              // bazel_targets
                None,              // bazel_affected_by_files
                "//...".into(),    // bazel_universe
                false,             // cyclonedx
                false,             // with_semgrep
                None,              // with_codeql
                None,              // autofix
                None,              // containers
                true,              // no_upload
                None,              // target
                false,             // incremental
                "main".into(),     // base
                false,             // diff
                None,              // baseline
                false,             // benchmark
                false,             // ml_risk
            )
        },

        Commands::Ci { path, out_dir } => {
            println!("🤖 Running CI-optimized scan (JSON + SARIF)...\n");
            handle_scan(
                path,
                None,              // profile
                false,             // reachability (too slow for CI)
                true,              // fast
                "sarif".into(),    // format
                out_dir,           // out_dir
                true,              // json
                None,              // bazel_targets_query
                None,              // bazel_targets
                None,              // bazel_affected_by_files
                "//...".into(),    // bazel_universe
                false,             // cyclonedx
                false,             // with_semgrep
                None,              // with_codeql
                None,              // autofix
                None,              // containers
                true,              // no_upload
                None,              // target
                false,             // incremental
                "main".into(),     // base
                false,             // diff
                None,              // baseline
                false,             // benchmark
                false,             // ml_risk
            )
        },

        Commands::Pr { path, base, baseline } => {
            println!("📋 Running PR-optimized scan (incremental + diff)...\n");
            handle_scan(
                path,
                None,              // profile
                false,             // reachability
                false,             // fast
                "sarif".into(),    // format
                ".".into(),        // out_dir
                true,              // json
                None,              // bazel_targets_query
                None,              // bazel_targets
                None,              // bazel_affected_by_files
                "//...".into(),    // bazel_universe
                false,             // cyclonedx
                false,             // with_semgrep
                None,              // with_codeql
                None,              // autofix
                None,              // containers
                false,             // no_upload
                None,              // target
                true,              // incremental
                base,              // base
                true,              // diff
                baseline,          // baseline
                false,             // benchmark
                false,             // ml_risk
            )
        },

        Commands::Full { path, out_dir } => {
            println!("💪 Running FULL scan with ALL features enabled...\n");
            handle_scan(
                path,
                None,              // profile
                true,              // reachability
                false,             // fast
                "spdx".into(),     // format
                out_dir,           // out_dir
                false,             // json
                None,              // bazel_targets_query
                None,              // bazel_targets
                None,              // bazel_affected_by_files
                "//...".into(),    // bazel_universe
                true,              // cyclonedx
                false,             // with_semgrep
                None,              // with_codeql
                None,              // autofix
                None,              // containers
                false,             // no_upload
                None,              // target
                false,             // incremental
                "main".into(),     // base
                false,             // diff
                None,              // baseline
                true,              // benchmark
                true,              // ml_risk
            )
        },

        Commands::Quick { path } => {
            println!("⚡ Running super-fast smoke test (< 5 seconds)...\n");
            handle_scan(
                path,
                None,              // profile
                false,             // reachability
                true,              // fast
                "spdx".into(),     // format
                ".".into(),        // out_dir
                false,             // json
                None,              // bazel_targets_query
                None,              // bazel_targets
                None,              // bazel_affected_by_files
                "//...".into(),    // bazel_universe
                false,             // cyclonedx
                false,             // with_semgrep
                None,              // with_codeql
                None,              // autofix
                None,              // containers
                true,              // no_upload
                auto_detect_main_module("."),  // target - auto-detected
                false,             // incremental
                "main".into(),     // base
                false,             // diff
                None,              // baseline
                false,             // benchmark
                false,             // ml_risk
            )
        },

        Commands::ContainerScan {
            image,
            output,
            format,
            baseline,
            compare_baseline,
            compare,
            create_issues,
            interactive,
            report,
            show,
            with_reachability,
        } => {
            use commands::container_scan::ContainerScanOptions;
            use std::path::PathBuf;

            let opts = ContainerScanOptions {
                image_name: image,
                output_dir: PathBuf::from(output),
                format,
                baseline,
                compare_baseline,
                compare_image: compare,
                create_issues_repo: create_issues,
                interactive,
                report_file: report,
                filter: show,
                with_reachability,
            };

            commands::container_scan::handle_container_scan(opts).await?;
            Ok(())
        },
        Commands::Policy { action } => handle_policy(action),
        Commands::Fix {
            package,
            suggest,
            apply,
            pr,
            interactive,
            explain,
            ml_prioritize,
            llm,
            llm_provider,
            llm_model,
        } => {
            handle_fix(
                package,
                suggest,
                apply,
                pr,
                interactive,
                explain,
                ml_prioritize,
                llm,
                llm_provider,
                llm_model,
            ).await?;
            Ok(())
        },
        Commands::License { action } => handle_license(action),
        Commands::Db { action } => handle_db(action),
        Commands::InstallHooks { policy, fast } => handle_install_hooks(policy, fast),
        Commands::Install { provider, list } => {
            if list {
                ci_templates::list_templates();
                Ok(())
            } else if let Some(provider) = provider {
                ci_templates::install_ci_template(&provider)
            } else {
                println!("❌ Error: Specify a provider or use --list to see options\n");
                ci_templates::list_templates();
                Ok(())
            }
        },
        Commands::Init { path } => handle_init(&path),
        Commands::Explore { sbom, findings } => handle_explore(sbom, findings),
        Commands::Dashboard { port, open, export } => handle_dashboard(port, open, export),
        Commands::Explain { cve_id, findings, verbose } => handle_explain(cve_id, findings, verbose),
        Commands::Status { verbose, findings } => handle_status(verbose, findings),
        Commands::Compare { base, target, verbose } => handle_compare(base, target, verbose),
        Commands::Watch { path, interval, critical_only } => handle_watch(path, interval, critical_only),
        Commands::Team { action } => handle_team(action),
        Commands::Report { action } => handle_report(action),
    }
}
