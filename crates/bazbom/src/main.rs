use anyhow::Result;
use clap::Parser;

mod advisory;
mod bazel;
mod ci_templates;
mod commands;
mod policy_integration;
mod reachability;
mod reachability_cache;
mod scan;
mod shading;
mod smart_defaults;

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
            if dir_name.starts_with('.')
                || dir_name == "node_modules"
                || dir_name == "target"
                || dir_name == "build"
            {
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
    // Initialize Rustls crypto provider for HTTPS connections
    let _ = rustls::crypto::ring::default_provider().install_default();

    // Initialize tracing/logging with environment variable support
    // Set RUST_LOG=debug for verbose output, or RUST_LOG=info for normal output
    // Example: RUST_LOG=debug bazbom full
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

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
        bazel_exclude_targets: None,
        bazel_workspace_path: None,
        include_path: None,
        languages: None,
        bazel_rc_path: None,
        bazel_flags: None,
        bazel_show_internal_targets: false,
        bazel_vendor_manifest_path: None,
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
        jira_create: false,
        jira_dry_run: false,
        github_pr: false,
        github_pr_dry_run: false,
        auto_remediate: false,
        remediate_min_severity: None,
        remediate_reachable_only: false,
        limit: None,
        include_cicd: false,
        fetch_checksums: false,
        sign_sbom: false,
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
            bazel_exclude_targets,
            bazel_workspace_path,
            include_path,
            languages,
            bazel_rc_path,
            bazel_flags,
            bazel_show_internal_targets,
            bazel_vendor_manifest_path,
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
            jira_create,
            jira_dry_run,
            github_pr,
            github_pr_dry_run,
            auto_remediate,
            remediate_min_severity,
            remediate_reachable_only,
            limit,
            include_cicd,
            fetch_checksums,
            sign_sbom,
        } => {
            handle_scan(
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
                bazel_exclude_targets,
                bazel_workspace_path,
                include_path,
                languages,
                bazel_rc_path,
                bazel_flags,
                bazel_show_internal_targets,
                bazel_vendor_manifest_path,
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
                jira_create,
                jira_dry_run,
                github_pr,
                github_pr_dry_run,
                auto_remediate,
                remediate_min_severity,
                remediate_reachable_only,
                limit,
                include_cicd,
                fetch_checksums,
                sign_sbom,
            )
            .await
        }

        // ========== QUICK COMMAND HANDLERS ==========
        Commands::Check { path } => {
            println!("FAST Running quick local check (fast mode)...\n");

            // Auto-detect main module if scanning current directory
            let scan_path = if path == "." {
                match auto_detect_main_module(&path) {
                    Some(detected) => {
                        println!("📍 Auto-detected main module: {}\n", detected);
                        detected
                    }
                    None => {
                        println!("INFO  Scanning entire workspace (no main module detected)\n");
                        path
                    }
                }
            } else {
                path
            };

            handle_scan(
                scan_path,
                None,           // profile
                false,          // reachability
                true,           // fast
                "spdx".into(),  // format
                ".".into(),     // out_dir
                false,          // json
                None,           // bazel_targets_query
                None,           // bazel_targets
                None,           // bazel_affected_by_files
                "//...".into(), // bazel_universe
                None,           // bazel_exclude_targets
                None,           // bazel_workspace_path
                None,           // include_path
                None,           // languages
                None,           // bazel_rc_path
                None,           // bazel_flags
                false,          // bazel_show_internal_targets
                None,           // bazel_vendor_manifest_path
                false,          // cyclonedx
                false,          // with_semgrep
                None,           // with_codeql
                None,           // autofix
                None,           // containers
                true,           // no_upload
                None,           // target
                false,          // incremental
                "main".into(),  // base
                false,          // diff
                None,           // baseline
                false,          // benchmark
                false,          // ml_risk
                false,          // jira_create
                false,          // jira_dry_run
                false,          // github_pr
                false,          // github_pr_dry_run
                false,          // auto_remediate
                None,           // remediate_min_severity
                false,          // remediate_reachable_only
                None,           // limit
                false,          // include_cicd
                false,          // fetch_checksums
                false,          // sign_sbom
            )
            .await
        }

        Commands::Ci { path, out_dir } => {
            println!("🤖 Running CI-optimized scan (JSON + SARIF)...\n");
            handle_scan(
                path,
                None,           // profile
                false,          // reachability (too slow for CI)
                true,           // fast
                "sarif".into(), // format
                out_dir,        // out_dir
                true,           // json
                None,           // bazel_targets_query
                None,           // bazel_targets
                None,           // bazel_affected_by_files
                "//...".into(), // bazel_universe
                None,           // bazel_exclude_targets
                None,           // bazel_workspace_path
                None,           // include_path
                None,           // languages
                None,           // bazel_rc_path
                None,           // bazel_flags
                false,          // bazel_show_internal_targets
                None,           // bazel_vendor_manifest_path
                false,          // cyclonedx
                false,          // with_semgrep
                None,           // with_codeql
                None,           // autofix
                None,           // containers
                true,           // no_upload
                None,           // target
                false,          // incremental
                "main".into(),  // base
                false,          // diff
                None,           // baseline
                false,          // benchmark
                false,          // ml_risk
                false,          // jira_create
                false,          // jira_dry_run
                false,          // github_pr
                false,          // github_pr_dry_run
                false,          // auto_remediate
                None,           // remediate_min_severity
                false,          // remediate_reachable_only
                None,           // limit
                false,          // include_cicd
                false,          // fetch_checksums
                false,          // sign_sbom
            )
            .await
        }

        Commands::Pr {
            path,
            base,
            baseline,
        } => {
            println!("NOTE Running PR-optimized scan (incremental + diff)...\n");
            handle_scan(
                path,
                None,           // profile
                false,          // reachability
                false,          // fast
                "sarif".into(), // format
                ".".into(),     // out_dir
                true,           // json
                None,           // bazel_targets_query
                None,           // bazel_targets
                None,           // bazel_affected_by_files
                "//...".into(), // bazel_universe
                None,           // bazel_exclude_targets
                None,           // bazel_workspace_path
                None,           // include_path
                None,           // languages
                None,           // bazel_rc_path
                None,           // bazel_flags
                false,          // bazel_show_internal_targets
                None,           // bazel_vendor_manifest_path
                false,          // cyclonedx
                false,          // with_semgrep
                None,           // with_codeql
                None,           // autofix
                None,           // containers
                false,          // no_upload
                None,           // target
                true,           // incremental
                base,           // base
                true,           // diff
                baseline,       // baseline
                false,          // benchmark
                false,          // ml_risk
                false,          // jira_create
                false,          // jira_dry_run
                false,          // github_pr
                false,          // github_pr_dry_run
                false,          // auto_remediate
                None,           // remediate_min_severity
                false,          // remediate_reachable_only
                None,           // limit
                false,          // include_cicd
                false,          // fetch_checksums
                false,          // sign_sbom
            )
            .await
        }

        Commands::Full {
            path,
            out_dir,
            limit,
        } => {
            println!("💪 Running FULL scan with ALL features enabled...\n");
            if let Some(n) = limit {
                println!("   INFO  Limiting scan to {} packages/targets\n", n);
            }
            handle_scan(
                path,
                None,           // profile
                true,           // reachability
                false,          // fast
                "spdx".into(),  // format
                out_dir,        // out_dir
                false,          // json
                None,           // bazel_targets_query
                None,           // bazel_targets
                None,           // bazel_affected_by_files
                "//...".into(), // bazel_universe
                None,           // bazel_exclude_targets
                None,           // bazel_workspace_path
                None,           // include_path
                None,           // languages
                None,           // bazel_rc_path
                None,           // bazel_flags
                false,          // bazel_show_internal_targets
                None,           // bazel_vendor_manifest_path
                true,           // cyclonedx
                false,          // with_semgrep
                None,           // with_codeql
                None,           // autofix
                None,           // containers
                false,          // no_upload
                None,           // target
                false,          // incremental
                "main".into(),  // base
                false,          // diff
                None,           // baseline
                true,           // benchmark
                true,           // ml_risk
                false,          // jira_create
                false,          // jira_dry_run
                false,          // github_pr
                false,          // github_pr_dry_run
                false,          // auto_remediate
                None,           // remediate_min_severity
                false,          // remediate_reachable_only
                limit,          // limit
                false,          // include_cicd
                false,          // fetch_checksums
                false,          // sign_sbom
            )
            .await
        }

        Commands::Quick { path } => {
            println!("⚡ Running super-fast smoke test (< 5 seconds)...\n");
            handle_scan(
                path,
                None,                         // profile
                false,                        // reachability
                true,                         // fast
                "spdx".into(),                // format
                ".".into(),                   // out_dir
                false,                        // json
                None,                         // bazel_targets_query
                None,                         // bazel_targets
                None,                         // bazel_affected_by_files
                "//...".into(),               // bazel_universe
                None,                         // bazel_exclude_targets
                None,                         // bazel_workspace_path
                None,                         // include_path
                None,                         // languages
                None,                         // bazel_rc_path
                None,                         // bazel_flags
                false,                        // bazel_show_internal_targets
                None,                         // bazel_vendor_manifest_path
                false,                        // cyclonedx
                false,                        // with_semgrep
                None,                         // with_codeql
                None,                         // autofix
                None,                         // containers
                true,                         // no_upload
                auto_detect_main_module("."), // target - auto-detected
                false,                        // incremental
                "main".into(),                // base
                false,                        // diff
                None,                         // baseline
                false,                        // benchmark
                false,                        // ml_risk
                false,                        // jira_create
                false,                        // jira_dry_run
                false,                        // github_pr
                false,                        // github_pr_dry_run
                false,                        // auto_remediate
                None,                         // remediate_min_severity
                false,                        // remediate_reachable_only
                None,                         // limit
                false,                        // include_cicd
                false,                        // fetch_checksums
                false,                        // sign_sbom
            )
            .await
        }

        Commands::ContainerScan {
            image,
            preset,
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
            skip_pull,
            allow_unsigned,
            offline,
        } => {
            use commands::container_scan::ContainerScanOptions;
            use std::path::PathBuf;

            // Smart output directory default: ~/Documents/container-scans/<image-name>
            let output_dir = output.unwrap_or_else(|| {
                let safe_name = image.replace(':', "_").replace('/', "_").replace('@', "_");
                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                format!("{}/Documents/container-scans/{}", home, safe_name)
            });

            // Apply preset settings (default = full capabilities)
            let (enable_reachability, show_preset_info) = match preset.as_deref() {
                Some("quick") => {
                    println!("FAST Quick scan mode: fast CI check, no reachability");
                    (false, true)
                }
                Some("standard") => {
                    println!("NOTE Standard scan mode: vulnerability analysis");
                    (false, true)
                }
                Some("full") | None => {
                    // Default: full capabilities
                    if preset.is_none() {
                        println!("SCAN Full scan mode: all capabilities enabled (reachability, compliance, Jira tickets)");
                    }
                    (true, preset.is_some())
                }
                Some("compliance") => {
                    println!("DOC Compliance scan mode: focus on compliance reports");
                    (true, true)
                }
                Some(other) => {
                    eprintln!("WARN  Unknown preset '{}', using full scan", other);
                    (true, false)
                }
            };

            if show_preset_info || preset.is_none() {
                println!("OUTPUT Output: {}\n", output_dir);
            }

            let opts = ContainerScanOptions {
                image_name: image,
                output_dir: PathBuf::from(output_dir),
                format,
                baseline,
                compare_baseline,
                compare_image: compare,
                create_issues_repo: create_issues,
                interactive,
                report_file: report,
                filter: show,
                with_reachability: with_reachability || enable_reachability,
                skip_pull,
                allow_unsigned,
                offline,
            };

            commands::container_scan::handle_container_scan(opts).await?;
            Ok(())
        }
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
            )
            .await?;
            Ok(())
        }
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
                println!("FAIL Error: Specify a provider or use --list to see options\n");
                ci_templates::list_templates();
                Ok(())
            }
        }
        Commands::Init { path } => handle_init(&path),
        Commands::Explore { sbom, findings } => handle_explore(sbom, findings),
        Commands::Dashboard { port, open, export } => handle_dashboard(port, open, export),
        Commands::Explain {
            cve_id,
            findings,
            verbose,
        } => handle_explain(cve_id, findings, verbose),
        Commands::Status { verbose, findings } => handle_status(verbose, findings),
        Commands::Compare {
            base,
            target,
            verbose,
        } => handle_compare(base, target, verbose),
        Commands::Watch {
            path,
            interval,
            critical_only,
        } => handle_watch(path, interval, critical_only),
        Commands::Team { action } => handle_team(action),
        Commands::Report { action } => handle_report(action),
        Commands::Jira { action } => {
            use bazbom::cli::JiraCmd;
            use commands::jira::JiraCommand;

            let cmd = match action {
                JiraCmd::Init => JiraCommand::Init,
                JiraCmd::Create {
                    file,
                    cve,
                    package,
                    severity,
                } => JiraCommand::Create {
                    file,
                    cve,
                    package,
                    severity,
                },
                JiraCmd::Get { key } => JiraCommand::Get { key },
                JiraCmd::Update {
                    key,
                    status,
                    assignee,
                } => JiraCommand::Update {
                    key,
                    status,
                    assignee,
                },
                JiraCmd::Sync => JiraCommand::Sync,
            };

            handle_jira(cmd).await?;
            Ok(())
        }
        Commands::GitHub { action } => {
            use bazbom::cli::{GitHubCmd, GitHubPrCmd};
            use commands::github::GitHubCommand;

            let cmd = match action {
                GitHubCmd::Init => GitHubCommand::Init,
                GitHubCmd::Pr(pr_cmd) => match pr_cmd {
                    GitHubPrCmd::Create {
                        owner,
                        repo,
                        head,
                        base,
                        title,
                        cve,
                        package,
                    } => GitHubCommand::PrCreate {
                        owner,
                        repo,
                        base,
                        head,
                        title,
                        cve,
                        package,
                    },
                    GitHubPrCmd::Get {
                        owner,
                        repo,
                        number,
                    } => GitHubCommand::PrGet {
                        owner,
                        repo,
                        number,
                    },
                    GitHubPrCmd::List { owner, repo, state } => {
                        GitHubCommand::PrList { owner, repo, state }
                    }
                },
            };

            handle_github(cmd).await?;
            Ok(())
        }

        Commands::Vex { action } => {
            use bazbom::cli::VexCmd;
            use commands::vex::{handle_vex_apply, handle_vex_create, handle_vex_list};

            match action {
                VexCmd::Create {
                    cve,
                    status,
                    justification,
                    impact,
                    package,
                    author,
                    output,
                } => {
                    handle_vex_create(cve, status, justification, impact, package, author, output)?;
                }
                VexCmd::Apply {
                    vex_dir,
                    findings,
                    output,
                } => {
                    handle_vex_apply(vex_dir, findings, output)?;
                }
                VexCmd::List { vex_dir } => {
                    handle_vex_list(vex_dir)?;
                }
            }
            Ok(())
        }

        Commands::Threats { action } => {
            use bazbom::cli::ThreatsCmd;
            use commands::threats::{handle_threats_configure, handle_threats_scan};

            match action {
                ThreatsCmd::Scan {
                    path,
                    typosquatting,
                    dep_confusion,
                    maintainer_takeover,
                    scorecard,
                    json,
                    output,
                    min_level,
                } => {
                    handle_threats_scan(
                        path,
                        typosquatting,
                        dep_confusion,
                        maintainer_takeover,
                        scorecard,
                        json,
                        output,
                        min_level,
                    )?;
                }
                ThreatsCmd::Configure {
                    add_feed,
                    remove_feed,
                    list,
                } => {
                    handle_threats_configure(add_feed, remove_feed, list)?;
                }
            }
            Ok(())
        }

        Commands::Notify { action } => {
            use bazbom::cli::NotifyCmd;
            use commands::notify::{
                handle_notify_configure, handle_notify_history, handle_notify_test,
            };

            match action {
                NotifyCmd::Configure {
                    slack_webhook,
                    teams_webhook,
                    email,
                    smtp_host,
                    github_repo,
                    min_severity,
                } => {
                    handle_notify_configure(
                        slack_webhook,
                        teams_webhook,
                        email,
                        smtp_host,
                        github_repo,
                        min_severity,
                    )?;
                }
                NotifyCmd::Test { channel } => {
                    handle_notify_test(channel)?;
                }
                NotifyCmd::History { limit } => {
                    handle_notify_history(limit)?;
                }
            }
            Ok(())
        }

        Commands::Anomaly { action } => {
            use bazbom::cli::AnomalyCmd;
            use commands::anomaly::{
                handle_anomaly_report, handle_anomaly_scan, handle_anomaly_train,
            };

            match action {
                AnomalyCmd::Scan { path, json, output } => {
                    handle_anomaly_scan(path, json, output)?;
                }
                AnomalyCmd::Train { from_dir, output } => {
                    handle_anomaly_train(from_dir, output)?;
                }
                AnomalyCmd::Report { path, output } => {
                    handle_anomaly_report(path, output)?;
                }
            }
            Ok(())
        }

        Commands::Lsp { stdio: _ } => {
            use commands::lsp::handle_lsp;
            handle_lsp()?;
            Ok(())
        }

        Commands::Auth { action } => {
            use bazbom::cli::AuthCmd;
            use commands::auth::{
                handle_auth_audit_log, handle_auth_init, handle_auth_token, handle_auth_user,
            };

            match action {
                AuthCmd::Init {} => {
                    handle_auth_init()?;
                }
                AuthCmd::User(user_cmd) => {
                    handle_auth_user(user_cmd)?;
                }
                AuthCmd::Token(token_cmd) => {
                    handle_auth_token(token_cmd)?;
                }
                AuthCmd::AuditLog { limit, event_type } => {
                    handle_auth_audit_log(limit, event_type)?;
                }
            }
            Ok(())
        }
    }
}
