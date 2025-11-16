use crate::error::{JiraError, Result};
use crate::models::*;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Webhook server for receiving Jira events
pub struct WebhookServer {
    port: u16,
    secret: String,
    handler: Arc<dyn WebhookHandler + Send + Sync>,
}

impl WebhookServer {
    /// Create a new webhook server
    pub fn new(port: u16, secret: String, handler: Arc<dyn WebhookHandler + Send + Sync>) -> Self {
        Self {
            port,
            secret,
            handler,
        }
    }

    /// Start the webhook server
    pub async fn start(self) -> Result<()> {
        let app = Router::new()
            .route("/webhooks/jira", post(handle_webhook))
            .with_state(Arc::new(self));

        let addr = format!("0.0.0.0:{}", self.port)
            .parse()
            .map_err(|e| JiraError::Config(format!("Invalid address: {}", e)))?;

        info!("Starting Jira webhook server on {}", addr);

        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| JiraError::Config(format!("Server error: {}", e)))?;

        Ok(())
    }
}

/// Webhook handler trait
#[async_trait::async_trait]
pub trait WebhookHandler {
    async fn handle_issue_updated(&self, event: IssueUpdatedEvent) -> Result<()>;
    async fn handle_comment_created(&self, event: CommentCreatedEvent) -> Result<()>;
}

/// Webhook event payload
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "webhookEvent")]
pub enum WebhookEvent {
    #[serde(rename = "jira:issue_updated")]
    IssueUpdated(IssueUpdatedEvent),

    #[serde(rename = "comment_created")]
    CommentCreated(CommentCreatedEvent),
}

/// Issue updated event
#[derive(Debug, Clone, Deserialize)]
pub struct IssueUpdatedEvent {
    pub issue: JiraIssue,
    pub changelog: Option<Changelog>,
}

/// Comment created event
#[derive(Debug, Clone, Deserialize)]
pub struct CommentCreatedEvent {
    pub issue: JiraIssue,
    pub comment: Comment,
}

/// Changelog
#[derive(Debug, Clone, Deserialize)]
pub struct Changelog {
    pub items: Vec<ChangelogItem>,
}

/// Changelog item
#[derive(Debug, Clone, Deserialize)]
pub struct ChangelogItem {
    pub field: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

/// Comment
#[derive(Debug, Clone, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub author: JiraUser,
}

/// Webhook handler
async fn handle_webhook(
    State(server): State<Arc<WebhookServer>>,
    _headers: axum::http::HeaderMap,
    Json(event): Json<WebhookEvent>,
) -> std::result::Result<StatusCode, StatusCode> {
    // TODO: Verify webhook signature

    // Process event
    match event {
        WebhookEvent::IssueUpdated(event) => {
            info!("Received issue updated event: {}", event.issue.key);
            server
                .handler
                .handle_issue_updated(event)
                .await
                .map_err(|e| {
                    error!("Failed to handle issue updated event: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        WebhookEvent::CommentCreated(event) => {
            info!("Received comment created event on issue: {}", event.issue.key);
            server
                .handler
                .handle_comment_created(event)
                .await
                .map_err(|e| {
                    error!("Failed to handle comment created event: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_server_creation() {
        struct TestHandler;

        #[async_trait::async_trait]
        impl WebhookHandler for TestHandler {
            async fn handle_issue_updated(&self, _event: IssueUpdatedEvent) -> Result<()> {
                Ok(())
            }

            async fn handle_comment_created(&self, _event: CommentCreatedEvent) -> Result<()> {
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler);
        let server = WebhookServer::new(8080, "secret".to_string(), handler);
        assert_eq!(server.port, 8080);
    }
}
