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
                None,              // target - TODO: auto-detect main module
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
