//! Jira integration commands
//!
//! This module provides CLI commands for interacting with Jira:
//! - `bazbom jira init` - Interactive setup wizard
//! - `bazbom jira create` - Create a Jira ticket
//! - `bazbom jira get` - Fetch ticket details
//! - `bazbom jira update` - Update a ticket
//! - `bazbom jira sync` - Manual synchronization

use anyhow::{Context, Result};
use bazbom_jira::client::JiraClient;
use bazbom_jira::config::JiraConfig;
use bazbom_jira::models::{CreateIssueRequest, IssueFields, UpdateIssueRequest};
use bazbom_jira::templates::TemplateEngine;
use colored::Colorize;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

/// Handle the `bazbom jira` subcommand
pub async fn handle_jira(cmd: JiraCommand) -> Result<()> {
    match cmd {
        JiraCommand::Init => handle_jira_init().await,
        JiraCommand::Create { file, cve, package, severity } => {
            handle_jira_create(file, cve, package, severity).await
        }
        JiraCommand::Get { key } => handle_jira_get(key).await,
        JiraCommand::Update { key, status, assignee } => {
            handle_jira_update(key, status, assignee).await
        }
        JiraCommand::Sync => handle_jira_sync().await,
    }
}

/// Jira subcommands
#[derive(Debug, Clone)]
pub enum JiraCommand {
    /// Initialize Jira integration (interactive setup)
    Init,
    /// Create a Jira ticket
    Create {
        /// Path to vulnerability findings file (JSON)
        file: Option<String>,
        /// CVE ID (for manual creation)
        cve: Option<String>,
        /// Package name (for manual creation)
        package: Option<String>,
        /// Severity (for manual creation)
        severity: Option<String>,
    },
    /// Get Jira ticket details
    Get {
        /// Jira issue key (e.g., SEC-123)
        key: String,
    },
    /// Update a Jira ticket
    Update {
        /// Jira issue key
        key: String,
        /// New status
        status: Option<String>,
        /// New assignee
        assignee: Option<String>,
    },
    /// Synchronize Jira tickets with BazBOM
    Sync,
}

/// Interactive setup wizard for Jira integration
async fn handle_jira_init() -> Result<()> {
    println!("{}", "🎯 BazBOM Jira Integration Setup".bold().cyan());
    println!();

    // Get Jira URL
    print!("Jira URL (e.g., https://example.atlassian.net): ");
    io::stdout().flush()?;
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim().to_string();

    // Get authentication type
    println!();
    println!("Authentication method:");
    println!("  1. API Token (recommended for Cloud)");
    println!("  2. Personal Access Token (PAT)");
    print!("Select (1-2): ");
    io::stdout().flush()?;
    let mut auth_choice = String::new();
    io::stdin().read_line(&mut auth_choice)?;

    let auth_type = match auth_choice.trim() {
        "1" => "api-token",
        "2" => "pat",
        _ => "api-token",
    };

    // Get username (for API token)
    let username = if auth_type == "api-token" {
        print!("\nJira username/email: ");
        io::stdout().flush()?;
        let mut user = String::new();
        io::stdin().read_line(&mut user)?;
        Some(user.trim().to_string())
    } else {
        None
    };

    // Get project key
    print!("\nJira project key (e.g., SEC): ");
    io::stdout().flush()?;
    let mut project = String::new();
    io::stdin().read_line(&mut project)?;
    let project = project.trim().to_string();

    // Get issue type
    print!("Issue type (default: Bug): ");
    io::stdout().flush()?;
    let mut issue_type = String::new();
    io::stdin().read_line(&mut issue_type)?;
    let issue_type = if issue_type.trim().is_empty() {
        "Bug".to_string()
    } else {
        issue_type.trim().to_string()
    };

    // Create configuration
    let config = JiraConfig {
        url,
        auth: bazbom_jira::config::AuthConfig {
            auth_type: auth_type.to_string(),
            token_env: Some("JIRA_API_TOKEN".to_string()),
            username_env: username.as_ref().map(|_| "JIRA_USERNAME".to_string()),
        },
        project,
        issue_type,
        auto_create: bazbom_jira::config::AutoCreateConfig {
            enabled: false,
            min_priority: Some("P2".to_string()),
            only_reachable: true,
        },
        custom_fields: Default::default(),
        routing: Vec::new(),
        sla: Default::default(),
        sync: Default::default(),
        webhook: Default::default(),
    };

    // Create .bazbom directory if it doesn't exist
    let config_dir = PathBuf::from(".bazbom");
    fs::create_dir_all(&config_dir).context("Failed to create .bazbom directory")?;

    // Write configuration file
    let config_path = config_dir.join("jira.yml");
    let yaml = serde_yaml::to_string(&config).context("Failed to serialize config")?;
    fs::write(&config_path, yaml).context("Failed to write config file")?;

    println!();
    println!("{}", "✅ Configuration saved!".green().bold());
    println!();
    println!("Next steps:");
    println!("  1. Set environment variables:");
    println!("     export JIRA_API_TOKEN=<your-token>");
    if username.is_some() {
        println!("     export JIRA_USERNAME={}", username.as_ref().unwrap());
    }
    println!();
    println!("  2. Test the connection:");
    println!("     bazbom jira get {}-1", config.project);
    println!();
    println!("  3. Enable auto-creation in {}", config_path.display());
    println!("     Set auto_create.enabled: true");
    println!();

    Ok(())
}

/// Create a Jira ticket from vulnerability data
async fn handle_jira_create(
    file: Option<String>,
    cve: Option<String>,
    package: Option<String>,
    severity: Option<String>,
) -> Result<()> {
    // Load configuration
    let config = load_jira_config().await?;

    // Get authentication token
    let token = std::env::var("JIRA_API_TOKEN")
        .context("JIRA_API_TOKEN environment variable not set")?;

    // Get username from environment if needed
    let username = if let Some(username_env) = &config.auth.username_env {
        std::env::var(username_env).ok()
    } else {
        None
    };

    // Create client
    let client = if let Some(username) = username {
        JiraClient::with_username(&config.url, &token, Some(username))
    } else {
        JiraClient::new(&config.url, &token)
    };

    // Build variables for template
    let mut variables = HashMap::new();

    if let (Some(cve), Some(package), Some(severity)) = (cve, package, severity) {
        // Manual creation from command line arguments
        variables.insert("cve_id".to_string(), cve.clone());
        variables.insert("package".to_string(), package);
        variables.insert("severity".to_string(), severity);
        variables.insert("summary".to_string(), format!("Security: {} in dependency", cve));
    } else if let Some(file_path) = file {
        // Creation from findings file
        println!("Loading findings from: {}", file_path);
        // TODO: Parse findings file and create multiple tickets
        anyhow::bail!("File-based creation not yet implemented");
    } else {
        anyhow::bail!("Either provide --cve, --package, --severity OR --file");
    }

    // Create template engine
    let template = TemplateEngine::new();

    // Render title and description
    let title = template.render_title(&variables)?;
    let description = template.render_description(&variables)?;

    println!("Creating Jira ticket...");
    println!("  Title: {}", title);

    // Build create request
    let request = CreateIssueRequest {
        fields: IssueFields {
            project: bazbom_jira::models::ProjectRef {
                key: config.project.clone(),
            },
            summary: title,
            description: Some(description),
            issuetype: bazbom_jira::models::IssueTypeRef {
                name: config.issue_type.clone(),
            },
            labels: Some(vec!["security".to_string(), "bazbom".to_string()]),
            priority: None,
            assignee: None,
            custom_fields: HashMap::new(),
        },
    };

    // Create issue
    let response = client.create_issue(request).await?;

    println!();
    println!("{}", "✅ Jira ticket created!".green().bold());
    println!("  Key: {}", response.key.bold());
    println!("  URL: {}/browse/{}", config.url, response.key);
    println!();

    Ok(())
}

/// Get Jira ticket details
async fn handle_jira_get(key: String) -> Result<()> {
    let config = load_jira_config().await?;
    let token = std::env::var("JIRA_API_TOKEN")
        .context("JIRA_API_TOKEN environment variable not set")?;

    // Get username from environment if needed
    let username = if let Some(username_env) = &config.auth.username_env {
        std::env::var(username_env).ok()
    } else {
        None
    };

    let client = if let Some(username) = username {
        JiraClient::with_username(&config.url, &token, Some(username))
    } else {
        JiraClient::new(&config.url, &token)
    };

    println!("Fetching Jira ticket: {}", key);

    let issue = client.get_issue(&key).await?;

    println!();
    println!("{}", format!("📋 {}", issue.key).bold());
    println!("  Summary: {}", issue.fields.summary);
    if let Some(status) = &issue.fields.status {
        println!("  Status: {}", status.name.cyan());
    }
    println!("  Type: {}", issue.fields.issue_type.name);
    if let Some(assignee) = &issue.fields.assignee {
        if let Some(display_name) = &assignee.display_name {
            println!("  Assignee: {}", display_name);
        }
    }
    if !issue.fields.labels.is_empty() {
        println!("  Labels: {}", issue.fields.labels.join(", "));
    }
    println!();

    Ok(())
}

/// Update a Jira ticket
async fn handle_jira_update(
    key: String,
    status: Option<String>,
    assignee: Option<String>,
) -> Result<()> {
    let config = load_jira_config().await?;
    let token = std::env::var("JIRA_API_TOKEN")
        .context("JIRA_API_TOKEN environment variable not set")?;

    // Get username from environment if needed
    let username = if let Some(username_env) = &config.auth.username_env {
        std::env::var(username_env).ok()
    } else {
        None
    };

    let client = if let Some(username) = username {
        JiraClient::with_username(&config.url, &token, Some(username))
    } else {
        JiraClient::new(&config.url, &token)
    };

    println!("Updating Jira ticket: {}", key);

    let mut fields = HashMap::new();

    if let Some(status) = status {
        println!("  Setting status to: {}", status);
        // TODO: Implement status transitions
    }

    if let Some(assignee) = assignee {
        println!("  Setting assignee to: {}", assignee);
        fields.insert("assignee".to_string(), serde_json::json!({ "name": assignee }));
    }

    let request = UpdateIssueRequest {
        fields: if fields.is_empty() { None } else { Some(fields) },
        update: None,
    };

    client.update_issue(&key, request).await?;

    println!();
    println!("{}", "✅ Ticket updated!".green().bold());
    println!();

    Ok(())
}

/// Synchronize Jira tickets with BazBOM
async fn handle_jira_sync() -> Result<()> {
    println!("{}", "🔄 Synchronizing Jira tickets...".bold().cyan());
    println!();

    // TODO: Implement synchronization logic
    // - Query Jira for security tickets
    // - Update local database
    // - Report changes

    println!("{}", "⚠️  Sync not yet implemented".yellow());
    println!("This feature will be available in Phase 3.");
    println!();

    Ok(())
}

/// Load Jira configuration from .bazbom/jira.yml
async fn load_jira_config() -> Result<JiraConfig> {
    let config_path = PathBuf::from(".bazbom/jira.yml");

    if !config_path.exists() {
        anyhow::bail!(
            "Jira configuration not found. Run 'bazbom jira init' first."
        );
    }

    let yaml = fs::read_to_string(&config_path)
        .context("Failed to read Jira configuration")?;

    let config: JiraConfig = serde_yaml::from_str(&yaml)
        .context("Failed to parse Jira configuration")?;

    Ok(config)
}
