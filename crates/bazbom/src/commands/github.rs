//! GitHub integration commands
//!
//! This module provides CLI commands for interacting with GitHub:
//! - `bazbom github init` - Interactive setup wizard
//! - `bazbom github pr create` - Create a pull request with fixes
//! - `bazbom github pr get` - Fetch PR details
//! - `bazbom github pr list` - List repository PRs

use anyhow::{Context, Result};
use bazbom_github::client::GitHubClient;
use bazbom_github::config::GitHubConfig;
use bazbom_github::models::{BazBomPrMetadata, CreatePullRequestRequest};
use bazbom_github::pr_template::PrTemplateEngine;
use colored::Colorize;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Handle the `bazbom github` subcommand
pub async fn handle_github(cmd: GitHubCommand) -> Result<()> {
    match cmd {
        GitHubCommand::Init => handle_github_init().await,
        GitHubCommand::PrCreate {
            owner,
            repo,
            base,
            head,
            title,
            cve,
            package,
        } => handle_github_pr_create(owner, repo, base, head, title, cve, package).await,
        GitHubCommand::PrGet { owner, repo, number } => {
            handle_github_pr_get(owner, repo, number).await
        }
        GitHubCommand::PrList { owner, repo, state } => {
            handle_github_pr_list(owner, repo, state).await
        }
    }
}

/// GitHub subcommands
#[derive(Debug, Clone)]
pub enum GitHubCommand {
    /// Initialize GitHub integration (interactive setup)
    Init,
    /// Create a pull request
    PrCreate {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// Base branch (default: main)
        base: Option<String>,
        /// Head branch with fixes
        head: String,
        /// PR title (optional)
        title: Option<String>,
        /// CVE ID (for metadata)
        cve: Option<String>,
        /// Package name (for metadata)
        package: Option<String>,
    },
    /// Get PR details
    PrGet {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// PR number
        number: u64,
    },
    /// List PRs
    PrList {
        /// Repository owner
        owner: String,
        /// Repository name
        repo: String,
        /// PR state (open, closed, all)
        state: Option<String>,
    },
}

/// Interactive setup wizard for GitHub integration
async fn handle_github_init() -> Result<()> {
    println!("{}", "🎯 BazBOM GitHub Integration Setup".bold().cyan());
    println!();

    // Get default owner/org
    print!("Default GitHub owner/org (e.g., mycompany): ");
    io::stdout().flush()?;
    let mut owner = String::new();
    io::stdin().read_line(&mut owner)?;
    let owner = owner.trim().to_string();

    // Get default repository
    print!("Default repository (optional): ");
    io::stdout().flush()?;
    let mut repo = String::new();
    io::stdin().read_line(&mut repo)?;
    let repo = if repo.trim().is_empty() {
        None
    } else {
        Some(repo.trim().to_string())
    };

    // Get base branch
    print!("Default base branch (default: main): ");
    io::stdout().flush()?;
    let mut base = String::new();
    io::stdin().read_line(&mut base)?;
    let base = if base.trim().is_empty() {
        "main".to_string()
    } else {
        base.trim().to_string()
    };

    // Auto-merge configuration
    println!();
    println!("Auto-merge settings:");
    print!("  Enable auto-merge for low-risk PRs? (y/N): ");
    io::stdout().flush()?;
    let mut auto_merge_input = String::new();
    io::stdin().read_line(&mut auto_merge_input)?;
    let auto_merge = auto_merge_input.trim().eq_ignore_ascii_case("y");

    // Create configuration
    let config = GitHubConfig {
        owner,
        repo,
        base_branch: base,
        auto_merge,
        auto_merge_min_confidence: 80,
        require_tests_pass: true,
        require_approvals: if auto_merge { 1 } else { 0 },
    };

    // Create .bazbom directory if it doesn't exist
    let config_dir = PathBuf::from(".bazbom");
    fs::create_dir_all(&config_dir).context("Failed to create .bazbom directory")?;

    // Write configuration file
    let config_path = config_dir.join("github.yml");
    let yaml = serde_yaml::to_string(&config).context("Failed to serialize config")?;
    fs::write(&config_path, yaml).context("Failed to write config file")?;

    println!();
    println!("{}", "✅ Configuration saved!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Set environment variable:");
    println!("     export GITHUB_TOKEN=<your-pat>");
    println!("     (Create token at: https://github.com/settings/tokens)");
    println!();
    println!("  2. Test the connection:");
    println!("     bazbom github pr list {} <repo>", config.owner);
    println!();
    println!("  3. Create your first automated PR:");
    println!("     bazbom scan --github-pr");
    println!();

    Ok(())
}

/// Create a GitHub pull request
async fn handle_github_pr_create(
    owner: String,
    repo: String,
    base: Option<String>,
    head: String,
    title: Option<String>,
    cve: Option<String>,
    package: Option<String>,
) -> Result<()> {
    // Get authentication token
    let token = std::env::var("GITHUB_TOKEN")
        .context("GITHUB_TOKEN environment variable not set")?;

    // Create client
    let client = GitHubClient::new(&token);

    // Load config for defaults
    let config = load_github_config().await.ok();
    let base_branch = base
        .or_else(|| config.as_ref().map(|c| c.base_branch.clone()))
        .unwrap_or_else(|| "main".to_string());

    // Build metadata for PR description
    let metadata = if let (Some(cve), Some(package)) = (cve.clone(), package.clone()) {
        BazBomPrMetadata {
            cve_id: cve,
            package,
            current_version: "unknown".to_string(),
            fix_version: "latest".to_string(),
            severity: "MEDIUM".to_string(),
            ml_risk_score: 50,
            reachable: false,
            confidence: 70,
            auto_merge_eligible: false,
            jira_ticket: None,
            scan_url: None,
        }
    } else {
        anyhow::bail!("--cve and --package are required for PR creation");
    };

    // Create template engine and render description
    let template = PrTemplateEngine::new();
    let description = template.render(&metadata)?;

    let pr_title = title.unwrap_or_else(|| {
        format!("fix: Update {} to address {}", metadata.package, metadata.cve_id)
    });

    println!("Creating GitHub PR...");
    println!("  Repository: {}/{}", owner, repo);
    println!("  Base: {} ← Head: {}", base_branch, head);
    println!("  Title: {}", pr_title);

    // Build create request
    let request = CreatePullRequestRequest {
        title: pr_title,
        body: description,
        head: head.clone(),
        base: base_branch,
        draft: false,
        maintainer_can_modify: true,
    };

    // Create PR
    let pr = client.create_pull_request(&owner, &repo, request).await?;

    println!();
    println!("{}", "✅ Pull request created!".green().bold());
    println!("  Number: #{}", pr.number);
    println!("  URL: {}", pr.html_url);
    println!("  Status: {}", pr.state);
    println!();

    Ok(())
}

/// Get GitHub PR details
async fn handle_github_pr_get(owner: String, repo: String, number: u64) -> Result<()> {
    let token = std::env::var("GITHUB_TOKEN")
        .context("GITHUB_TOKEN environment variable not set")?;

    let client = GitHubClient::new(&token);

    println!("Fetching PR #{} from {}/{}...", number, owner, repo);

    let pr = client.get_pull_request(&owner, &repo, number).await?;

    println!();
    println!("{}", format!("🔀 PR #{}: {}", pr.number, pr.title).bold());
    println!("  State: {}", pr.state.cyan());
    println!("  Author: {}", pr.user.login);
    println!("  Base: {} ← Head: {}", pr.base.ref_name, pr.head.ref_name);
    println!("  URL: {}", pr.html_url);
    if pr.draft {
        println!("  Draft: {}", "yes".yellow());
    }
    if pr.merged {
        println!("  Merged: {}", "yes".green());
    }
    println!();

    Ok(())
}

/// List GitHub PRs
async fn handle_github_pr_list(
    owner: String,
    repo: String,
    state: Option<String>,
) -> Result<()> {
    let token = std::env::var("GITHUB_TOKEN")
        .context("GITHUB_TOKEN environment variable not set")?;

    let client = GitHubClient::new(&token);

    let state_filter = state.as_deref().unwrap_or("open");

    println!("Listing {} PRs for {}/{}...", state_filter, owner, repo);
    println!();

    // TODO: Implement list_pull_requests in GitHubClient
    println!("{}", "⚠️  PR listing not yet implemented".yellow());
    println!("This feature will be available soon.");
    println!();

    Ok(())
}

/// Load GitHub configuration from .bazbom/github.yml
async fn load_github_config() -> Result<GitHubConfig> {
    let config_path = PathBuf::from(".bazbom/github.yml");

    if !config_path.exists() {
        anyhow::bail!(
            "GitHub configuration not found. Run 'bazbom github init' first."
        );
    }

    let yaml = fs::read_to_string(&config_path)
        .context("Failed to read GitHub configuration")?;

    let config: GitHubConfig = serde_yaml::from_str(&yaml)
        .context("Failed to parse GitHub configuration")?;

    Ok(config)
}
