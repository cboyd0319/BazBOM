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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Commands::Scan {
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
        benchmark: false,
        ml_risk: false,
    });

    match command {
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
            benchmark,
            ml_risk,
        } => handle_scan(
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
            benchmark,
            ml_risk,
        ),
        Commands::Policy { action } => handle_policy(action),
        Commands::Fix {
            suggest,
            apply,
            pr,
            interactive,
            ml_prioritize,
            llm,
            llm_provider,
            llm_model,
        } => handle_fix(
            suggest,
            apply,
            pr,
            interactive,
            ml_prioritize,
            llm,
            llm_provider,
            llm_model,
        ),
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
