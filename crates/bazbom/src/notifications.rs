//! Notification system for team coordination
//!
//! This module implements notification channels for alerting teams about:
//! - New vulnerabilities discovered
//! - Vulnerability assignments
//! - Security policy violations
//! - Remediation completions

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub title: String,
    pub message: String,
    pub severity: NotificationSeverity,
    pub details: Option<HashMap<String, String>>,
}

/// Notification severity level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationSeverity {
    Info,
    Warning,
    Critical,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Slack { webhook_url: String },
    Email { smtp_url: String },
    MicrosoftTeams { webhook_url: String },
    // Future: GitHub Issues, PagerDuty, etc.
}

/// Notifier for sending notifications to configured channels
pub struct Notifier {
    channels: Vec<NotificationChannel>,
}

impl Notifier {
    /// Create a new notifier with channels
    pub fn new(channels: Vec<NotificationChannel>) -> Self {
        Self { channels }
    }

    /// Send a notification to all configured channels
    pub fn send(&self, notification: &Notification) -> Result<()> {
        let mut errors = Vec::new();

        for channel in &self.channels {
            if let Err(e) = self.send_to_channel(channel, notification) {
                eprintln!(
                    "[bazbom] Warning: Failed to send notification via {:?}: {}",
                    channel, e
                );
                errors.push(e);
            }
        }

        if !errors.is_empty() && errors.len() == self.channels.len() {
            anyhow::bail!("Failed to send notification to all channels");
        }

        Ok(())
    }

    /// Send notification to a specific channel
    fn send_to_channel(
        &self,
        channel: &NotificationChannel,
        notification: &Notification,
    ) -> Result<()> {
        match channel {
            NotificationChannel::Slack { webhook_url } => {
                self.send_slack(webhook_url, notification)
            }
            NotificationChannel::Email { smtp_url } => self.send_email(smtp_url, notification),
            NotificationChannel::MicrosoftTeams { webhook_url } => {
                self.send_teams(webhook_url, notification)
            }
        }
    }

    /// Send notification to Slack
    fn send_slack(&self, webhook_url: &str, notification: &Notification) -> Result<()> {
        let color = match notification.severity {
            NotificationSeverity::Info => "#36a64f",     // Green
            NotificationSeverity::Warning => "#ff9900",  // Orange
            NotificationSeverity::Critical => "#ff0000", // Red
        };

        let mut fields = Vec::new();
        if let Some(details) = &notification.details {
            for (key, value) in details {
                fields.push(serde_json::json!({
                    "title": key,
                    "value": value,
                    "short": true
                }));
            }
        }

        let payload = serde_json::json!({
            "attachments": [{
                "color": color,
                "title": notification.title,
                "text": notification.message,
                "fields": fields,
                "footer": "BazBOM Security Scanner",
                "footer_icon": "https://github.com/cboyd0319/BazBOM/raw/main/static/logo.png"
            }]
        });

        let response = ureq::post(webhook_url)
            .set("Content-Type", "application/json")
            .send_json(&payload)
            .context("Failed to send Slack notification")?;

        if response.status() != 200 {
            anyhow::bail!("Slack webhook returned status: {}", response.status());
        }

        Ok(())
    }

    /// Send notification via email (SMTP)
    fn send_email(&self, _smtp_url: &str, notification: &Notification) -> Result<()> {
        // TODO: Implement SMTP email sending
        // For now, just log that we would send an email
        println!(
            "[bazbom] Email notification (SMTP not yet implemented): {}",
            notification.title
        );
        Ok(())
    }

    /// Send notification to Microsoft Teams
    fn send_teams(&self, webhook_url: &str, notification: &Notification) -> Result<()> {
        let color = match notification.severity {
            NotificationSeverity::Info => "0078D4",     // Blue
            NotificationSeverity::Warning => "FFB900",  // Yellow
            NotificationSeverity::Critical => "D13438", // Red
        };

        let mut facts = Vec::new();
        if let Some(details) = &notification.details {
            for (key, value) in details {
                facts.push(serde_json::json!({
                    "name": key,
                    "value": value
                }));
            }
        }

        let payload = serde_json::json!({
            "@type": "MessageCard",
            "@context": "https://schema.org/extensions",
            "summary": notification.title,
            "themeColor": color,
            "title": notification.title,
            "text": notification.message,
            "sections": [{
                "facts": facts
            }]
        });

        let response = ureq::post(webhook_url)
            .set("Content-Type", "application/json")
            .send_json(&payload)
            .context("Failed to send Microsoft Teams notification")?;

        if response.status() != 200 {
            anyhow::bail!(
                "Microsoft Teams webhook returned status: {}",
                response.status()
            );
        }

        Ok(())
    }
}

/// Configuration for notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub channels: Vec<NotificationChannel>,
}

impl NotificationConfig {
    /// Load notification configuration from file
    pub fn load(path: &str) -> Result<Self> {
        let content =
            std::fs::read_to_string(path).context("Failed to read notification configuration")?;
        let config: NotificationConfig =
            serde_json::from_str(&content).context("Failed to parse notification configuration")?;
        Ok(config)
    }

    /// Save notification configuration to file
    pub fn save(&self, path: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

/// Helper function to send vulnerability notification
pub fn notify_vulnerability_discovered(
    notifier: &Notifier,
    cve: &str,
    severity: &str,
    package: &str,
    version: &str,
) -> Result<()> {
    let mut details = HashMap::new();
    details.insert("CVE".to_string(), cve.to_string());
    details.insert("Package".to_string(), format!("{}:{}", package, version));
    details.insert("Severity".to_string(), severity.to_string());

    let notification = Notification {
        title: format!("🚨 {} Vulnerability Discovered", severity),
        message: format!(
            "New {} severity vulnerability {} found in {}:{}",
            severity, cve, package, version
        ),
        severity: match severity {
            "CRITICAL" => NotificationSeverity::Critical,
            "HIGH" => NotificationSeverity::Warning,
            _ => NotificationSeverity::Info,
        },
        details: Some(details),
    };

    notifier.send(&notification)
}

/// Helper function to send assignment notification
pub fn notify_vulnerability_assigned(
    notifier: &Notifier,
    cve: &str,
    assignee: &str,
    assigner: &str,
) -> Result<()> {
    let mut details = HashMap::new();
    details.insert("CVE".to_string(), cve.to_string());
    details.insert("Assigned To".to_string(), assignee.to_string());
    details.insert("Assigned By".to_string(), assigner.to_string());

    let notification = Notification {
        title: "[*] Vulnerability Assigned".to_string(),
        message: format!("{} has been assigned to {}", cve, assignee),
        severity: NotificationSeverity::Info,
        details: Some(details),
    };

    notifier.send(&notification)
}

/// Helper function to send remediation notification
pub fn notify_vulnerability_fixed(
    notifier: &Notifier,
    cve: &str,
    package: &str,
    old_version: &str,
    new_version: &str,
    fixed_by: &str,
) -> Result<()> {
    let mut details = HashMap::new();
    details.insert("CVE".to_string(), cve.to_string());
    details.insert("Package".to_string(), package.to_string());
    details.insert(
        "Upgraded".to_string(),
        format!("{} → {}", old_version, new_version),
    );
    details.insert("Fixed By".to_string(), fixed_by.to_string());

    let notification = Notification {
        title: "[+] Vulnerability Fixed".to_string(),
        message: format!(
            "{} has been remediated by upgrading {} from {} to {}",
            cve, package, old_version, new_version
        ),
        severity: NotificationSeverity::Info,
        details: Some(details),
    };

    notifier.send(&notification)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_serialization() {
        let mut details = HashMap::new();
        details.insert("CVE".to_string(), "CVE-2021-44228".to_string());
        details.insert("Severity".to_string(), "CRITICAL".to_string());

        let notification = Notification {
            title: "Test Notification".to_string(),
            message: "This is a test".to_string(),
            severity: NotificationSeverity::Critical,
            details: Some(details),
        };

        let json = serde_json::to_string(&notification).unwrap();
        let parsed: Notification = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.title, "Test Notification");
        assert_eq!(parsed.message, "This is a test");
    }

    #[test]
    fn test_notification_config() {
        let config = NotificationConfig {
            channels: vec![
                NotificationChannel::Slack {
                    webhook_url: "https://hooks.slack.com/test".to_string(),
                },
                NotificationChannel::MicrosoftTeams {
                    webhook_url: "https://outlook.office.com/webhook/test".to_string(),
                },
            ],
        };

        assert_eq!(config.channels.len(), 2);
    }

    #[test]
    fn test_notifier_creation() {
        let channels = vec![NotificationChannel::Slack {
            webhook_url: "https://hooks.slack.com/test".to_string(),
        }];

        let notifier = Notifier::new(channels);
        assert_eq!(notifier.channels.len(), 1);
    }
}
