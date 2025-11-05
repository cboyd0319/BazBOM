//! Notification integrations for threat alerts
//!
//! Supports Slack, email (SMTP), Microsoft Teams, and GitHub Issues

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NotificationChannel {
    /// Slack webhook
    Slack {
        webhook_url: String,
        #[serde(default)]
        channel: Option<String>,
        #[serde(default)]
        username: Option<String>,
    },
    /// Email via SMTP
    Email {
        smtp_server: String,
        smtp_port: u16,
        from_address: String,
        to_addresses: Vec<String>,
        #[serde(default)]
        username: Option<String>,
        #[serde(default)]
        password: Option<String>,
    },
    /// Microsoft Teams webhook
    Teams { webhook_url: String },
    /// GitHub Issues
    GithubIssue {
        token: String,
        owner: String,
        repo: String,
        #[serde(default)]
        labels: Vec<String>,
    },
}

/// Notification message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Message title
    pub title: String,
    /// Message body
    pub message: String,
    /// Severity level (critical, high, medium, low)
    pub severity: String,
    /// Additional metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl Notification {
    /// Create a new notification
    pub fn new(
        title: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            severity: severity.into(),
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get color based on severity
    fn get_color(&self) -> &str {
        match self.severity.to_lowercase().as_str() {
            "critical" => "#FF0000",
            "high" => "#FF6600",
            "medium" => "#FFCC00",
            "low" => "#00CC00",
            _ => "#808080",
        }
    }

    /// Get emoji for severity
    fn get_emoji(&self) -> &str {
        match self.severity.to_lowercase().as_str() {
            "critical" => "🚨",
            "high" => "⚠️",
            "medium" => "⚡",
            "low" => "ℹ️",
            _ => "📢",
        }
    }
}

/// Notification service
pub struct Notifier {
    channels: Vec<NotificationChannel>,
}

impl Notifier {
    /// Create a new notifier with channels
    pub fn new(channels: Vec<NotificationChannel>) -> Self {
        Self { channels }
    }

    /// Send notification to all channels
    pub fn send(&self, notification: &Notification) -> Result<Vec<Result<()>>> {
        let mut results = Vec::new();

        for channel in &self.channels {
            let result = match channel {
                NotificationChannel::Slack {
                    webhook_url,
                    channel,
                    username,
                } => self.send_slack(
                    webhook_url,
                    notification,
                    channel.as_deref(),
                    username.as_deref(),
                ),
                NotificationChannel::Email {
                    smtp_server,
                    smtp_port,
                    from_address,
                    to_addresses,
                    username,
                    password,
                } => self.send_email(
                    smtp_server,
                    *smtp_port,
                    from_address,
                    to_addresses,
                    notification,
                    username.as_deref(),
                    password.as_deref(),
                ),
                NotificationChannel::Teams { webhook_url } => {
                    self.send_teams(webhook_url, notification)
                }
                NotificationChannel::GithubIssue {
                    token,
                    owner,
                    repo,
                    labels,
                } => self.send_github_issue(token, owner, repo, labels, notification),
            };

            results.push(result);
        }

        Ok(results)
    }

    /// Send Slack notification
    fn send_slack(
        &self,
        webhook_url: &str,
        notification: &Notification,
        channel: Option<&str>,
        username: Option<&str>,
    ) -> Result<()> {
        let emoji = notification.get_emoji();
        let color = notification.get_color();

        let mut payload = json!({
            "attachments": [{
                "color": color,
                "title": format!("{} {}", emoji, notification.title),
                "text": notification.message,
                "fields": [
                    {
                        "title": "Severity",
                        "value": notification.severity,
                        "short": true
                    }
                ]
            }]
        });

        if let Some(ch) = channel {
            payload["channel"] = json!(ch);
        }

        if let Some(user) = username {
            payload["username"] = json!(user);
        }

        // Send HTTP POST to Slack webhook
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to send Slack notification: {}", e))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Slack webhook returned error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }

        log::info!("✓ Slack notification sent successfully");
        Ok(())
    }

    /// Send email notification (SMTP)
    #[allow(clippy::too_many_arguments)]
    fn send_email(
        &self,
        smtp_server: &str,
        smtp_port: u16,
        from: &str,
        to: &[String],
        notification: &Notification,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<()> {
        use lettre::message::header::ContentType;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{Message, SmtpTransport, Transport};

        // Build email body with severity indicator
        let emoji = notification.get_emoji();
        let email_body = format!(
            "{} {}\n\nSeverity: {}\n\n{}\n\n---\nSent by BazBOM",
            emoji, notification.title, notification.severity, notification.message
        );

        // Build the email message
        let mut email_builder = Message::builder()
            .from(from.parse()?)
            .subject(format!("{} {}", emoji, notification.title))
            .header(ContentType::TEXT_PLAIN);

        // Add all recipients
        for recipient in to {
            email_builder = email_builder.to(recipient.parse()?);
        }

        let email = email_builder.body(email_body)?;

        // Build SMTP client
        let mailer = if let (Some(user), Some(pass)) = (username, password) {
            // Authenticated SMTP
            let creds = Credentials::new(user.to_string(), pass.to_string());
            SmtpTransport::relay(smtp_server)?
                .port(smtp_port)
                .credentials(creds)
                .build()
        } else {
            // Unauthenticated SMTP
            SmtpTransport::builder_dangerous(smtp_server)
                .port(smtp_port)
                .build()
        };

        // Send email
        mailer
            .send(&email)
            .map_err(|e| anyhow::anyhow!("Failed to send email: {}", e))?;

        log::info!("✓ Email notification sent successfully to {:?}", to);
        Ok(())
    }

    /// Send Microsoft Teams notification
    fn send_teams(&self, webhook_url: &str, notification: &Notification) -> Result<()> {
        let color = notification.get_color();

        let payload = json!({
            "@type": "MessageCard",
            "@context": "https://schema.org/extensions",
            "themeColor": color.trim_start_matches('#'),
            "title": notification.title,
            "text": notification.message,
            "sections": [{
                "facts": [
                    {
                        "name": "Severity:",
                        "value": notification.severity
                    }
                ]
            }]
        });

        // Send HTTP POST to Teams webhook
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(webhook_url)
            .json(&payload)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to send Teams notification: {}", e))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Teams webhook returned error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }

        log::info!("✓ Teams notification sent successfully");
        Ok(())
    }

    /// Create GitHub issue
    fn send_github_issue(
        &self,
        token: &str,
        owner: &str,
        repo: &str,
        labels: &[String],
        notification: &Notification,
    ) -> Result<()> {
        let emoji = notification.get_emoji();
        let issue_body = format!(
            "## {}\n\n{}\n\n**Severity:** {}\n\n---\n*Automatically created by BazBOM*",
            notification.message,
            notification
                .metadata
                .get("details")
                .unwrap_or(&"".to_string()),
            notification.severity
        );

        let mut issue_labels = labels.to_vec();
        issue_labels.push(format!("severity:{}", notification.severity.to_lowercase()));

        let issue_payload = json!({
            "title": format!("{} {}", emoji, notification.title),
            "body": issue_body,
            "labels": issue_labels
        });

        // Send HTTP POST to GitHub API
        let client = reqwest::blocking::Client::new();
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

        let response = client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "BazBOM")
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .json(&issue_payload)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to create GitHub issue: {}", e))?;

        if !response.status().is_success() {
            anyhow::bail!(
                "GitHub API returned error: {} - {}",
                response.status(),
                response.text().unwrap_or_default()
            );
        }

        let issue_response: serde_json::Value = response.json()?;
        let issue_number = issue_response["number"].as_u64().unwrap_or(0);

        log::info!("✓ GitHub issue #{} created successfully", issue_number);
        Ok(())
    }
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Enable notifications
    pub enabled: bool,
    /// Channels to send notifications to
    pub channels: Vec<NotificationChannel>,
    /// Minimum severity to trigger notification
    pub min_severity: String,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            channels: Vec::new(),
            min_severity: "high".to_string(),
        }
    }
}

impl NotificationConfig {
    /// Check if severity meets threshold
    pub fn should_notify(&self, severity: &str) -> bool {
        if !self.enabled {
            return false;
        }

        let severity_value = severity_to_value(severity);
        let min_value = severity_to_value(&self.min_severity);

        severity_value >= min_value
    }
}

/// Convert severity to numeric value for comparison
fn severity_to_value(severity: &str) -> u8 {
    match severity.to_lowercase().as_str() {
        "critical" => 4,
        "high" => 3,
        "medium" => 2,
        "low" => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_creation() {
        let notif = Notification::new("Test", "Test message", "high");
        assert_eq!(notif.title, "Test");
        assert_eq!(notif.message, "Test message");
        assert_eq!(notif.severity, "high");
    }

    #[test]
    fn test_notification_with_metadata() {
        let notif = Notification::new("Test", "Message", "critical")
            .with_metadata("cve", "CVE-2024-1234")
            .with_metadata("package", "log4j");

        assert_eq!(notif.metadata.get("cve").unwrap(), "CVE-2024-1234");
        assert_eq!(notif.metadata.get("package").unwrap(), "log4j");
    }

    #[test]
    fn test_notification_colors() {
        let critical = Notification::new("Test", "Msg", "critical");
        assert_eq!(critical.get_color(), "#FF0000");

        let high = Notification::new("Test", "Msg", "high");
        assert_eq!(high.get_color(), "#FF6600");

        let medium = Notification::new("Test", "Msg", "medium");
        assert_eq!(medium.get_color(), "#FFCC00");

        let low = Notification::new("Test", "Msg", "low");
        assert_eq!(low.get_color(), "#00CC00");
    }

    #[test]
    fn test_notification_emojis() {
        let critical = Notification::new("Test", "Msg", "critical");
        assert_eq!(critical.get_emoji(), "🚨");

        let high = Notification::new("Test", "Msg", "high");
        assert_eq!(high.get_emoji(), "⚠️");
    }

    #[test]
    fn test_notifier_creation() {
        let channels = vec![NotificationChannel::Slack {
            webhook_url: "https://hooks.slack.com/test".to_string(),
            channel: Some("#security".to_string()),
            username: Some("BazBOM".to_string()),
        }];

        let notifier = Notifier::new(channels);
        assert_eq!(notifier.channels.len(), 1);
    }

    #[test]
    fn test_severity_threshold() {
        let config = NotificationConfig {
            enabled: true,
            channels: vec![],
            min_severity: "high".to_string(),
        };

        assert!(config.should_notify("critical"));
        assert!(config.should_notify("high"));
        assert!(!config.should_notify("medium"));
        assert!(!config.should_notify("low"));
    }

    #[test]
    fn test_disabled_notifications() {
        let config = NotificationConfig {
            enabled: false,
            channels: vec![],
            min_severity: "low".to_string(),
        };

        assert!(!config.should_notify("critical"));
        assert!(!config.should_notify("high"));
    }

    #[test]
    fn test_severity_to_value() {
        assert_eq!(severity_to_value("critical"), 4);
        assert_eq!(severity_to_value("high"), 3);
        assert_eq!(severity_to_value("medium"), 2);
        assert_eq!(severity_to_value("low"), 1);
        assert_eq!(severity_to_value("unknown"), 0);
    }
}
