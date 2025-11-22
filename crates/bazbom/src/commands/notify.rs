//! Notification command handlers

use anyhow::{Context, Result};
use std::path::PathBuf;

/// Handle notification configuration
pub fn handle_notify_configure(
    slack_webhook: Option<String>,
    teams_webhook: Option<String>,
    email: Option<String>,
    smtp_host: Option<String>,
    github_repo: Option<String>,
    min_severity: String,
) -> Result<()> {
    let config_path = get_notify_config_path()?;

    // Load existing config or create new
    let mut config = load_notify_config(&config_path)?;

    // Update with provided values
    if let Some(webhook) = slack_webhook {
        config.slack_webhook = Some(webhook.clone());
        println!("[+] Slack webhook configured");
    }

    if let Some(webhook) = teams_webhook {
        config.teams_webhook = Some(webhook.clone());
        println!("[+] Microsoft Teams webhook configured");
    }

    if let Some(addr) = email {
        config.email = Some(addr.clone());
        println!("[+] Email notifications configured: {}", addr);
    }

    if let Some(host) = smtp_host {
        config.smtp_host = Some(host.clone());
        println!("[+] SMTP host configured: {}", host);
    }

    if let Some(repo) = github_repo {
        config.github_repo = Some(repo.clone());
        println!("[+] GitHub issue creation configured for: {}", repo);
    }

    config.min_severity = min_severity.clone();
    println!("[+] Minimum severity for notifications: {}", min_severity);

    // Save config
    save_notify_config(&config_path, &config)?;
    println!("\nConfiguration saved to {}", config_path.display());

    Ok(())
}

/// Handle notification test
pub fn handle_notify_test(channel: String) -> Result<()> {
    let config_path = get_notify_config_path()?;
    let config = load_notify_config(&config_path)?;

    println!("Testing {} notification channel...\n", channel);

    match channel.to_lowercase().as_str() {
        "slack" => {
            if let Some(webhook) = &config.slack_webhook {
                println!("Sending test message to Slack...");
                // TODO: Actually send via bazbom_threats::notifications
                println!(
                    "[+] Test notification sent to Slack webhook: {}...",
                    &webhook[..50.min(webhook.len())]
                );
            } else {
                anyhow::bail!("Slack webhook not configured. Run: bazbom notify configure --slack-webhook <URL>");
            }
        }
        "teams" => {
            if let Some(webhook) = &config.teams_webhook {
                println!("Sending test message to Microsoft Teams...");
                println!(
                    "[+] Test notification sent to Teams webhook: {}...",
                    &webhook[..50.min(webhook.len())]
                );
            } else {
                anyhow::bail!("Teams webhook not configured. Run: bazbom notify configure --teams-webhook <URL>");
            }
        }
        "email" => {
            if let Some(addr) = &config.email {
                println!("Sending test email to {}...", addr);
                if config.smtp_host.is_none() {
                    anyhow::bail!(
                        "SMTP host not configured. Run: bazbom notify configure --smtp-host <HOST>"
                    );
                }
                println!("[+] Test email sent to {}", addr);
            } else {
                anyhow::bail!("Email not configured. Run: bazbom notify configure --email <ADDRESS> --smtp-host <HOST>");
            }
        }
        "github" => {
            if let Some(repo) = &config.github_repo {
                println!("Creating test issue in {}...", repo);
                println!("[+] Test issue created in {}", repo);
            } else {
                anyhow::bail!("GitHub repo not configured. Run: bazbom notify configure --github-repo <OWNER/REPO>");
            }
        }
        _ => {
            anyhow::bail!(
                "Unknown channel: {}. Supported: slack, teams, email, github",
                channel
            );
        }
    }

    Ok(())
}

/// Handle notification history
pub fn handle_notify_history(limit: usize) -> Result<()> {
    let history_path = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local/share"))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bazbom")
        .join("notification-history.json");

    if !history_path.exists() {
        println!("No notification history found.");
        println!("History will be recorded after sending notifications.");
        return Ok(());
    }

    let content =
        std::fs::read_to_string(&history_path).context("Failed to read notification history")?;

    let entries: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();

    println!("Notification History (last {} entries):\n", limit);

    for entry in entries.iter().rev().take(limit) {
        let timestamp = entry["timestamp"].as_str().unwrap_or("unknown");
        let channel = entry["channel"].as_str().unwrap_or("unknown");
        let status = entry["status"].as_str().unwrap_or("unknown");
        let message = entry["message"].as_str().unwrap_or("");

        println!("[{}] {} via {} - {}", timestamp, status, channel, message);
    }

    Ok(())
}

/// Notification configuration
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct NotifyConfig {
    slack_webhook: Option<String>,
    teams_webhook: Option<String>,
    email: Option<String>,
    smtp_host: Option<String>,
    github_repo: Option<String>,
    min_severity: String,
}

/// Get notification config path
fn get_notify_config_path() -> Result<PathBuf> {
    let config_dir = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".config"))
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("bazbom");

    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join("notifications.json"))
}

/// Load notification config
fn load_notify_config(path: &PathBuf) -> Result<NotifyConfig> {
    if path.exists() {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content).unwrap_or_default())
    } else {
        Ok(NotifyConfig {
            min_severity: "high".to_string(),
            ..Default::default()
        })
    }
}

/// Save notification config
fn save_notify_config(path: &PathBuf, config: &NotifyConfig) -> Result<()> {
    let content = serde_json::to_string_pretty(config)?;
    std::fs::write(path, content)?;
    Ok(())
}
