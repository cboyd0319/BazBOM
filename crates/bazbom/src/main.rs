use anyhow::Result;
use clap::Parser;

mod advisory;
mod bazel;
mod commands;
mod policy_integration;
mod reachability;
mod reachability_cache;
mod scan;
mod shading;

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
            benchmark,
            ml_risk,
        } => handle_scan(
            path,
            profile,
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
            benchmark,
            ml_risk,
        ),
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
        Commands::Init { path } => handle_init(&path),
        Commands::Explore { sbom, findings } => handle_explore(sbom, findings),
        Commands::Dashboard { port, open, export } => handle_dashboard(port, open, export),
        Commands::Team { action } => handle_team(action),
        Commands::Report { action } => handle_report(action),
    }
}
