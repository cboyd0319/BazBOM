use crate::error::{GitHubError, Result};
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

/// Webhook server for receiving GitHub events
pub struct WebhookServer {
    port: u16,
    #[allow(dead_code)]
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
        let port = self.port;

        let app = Router::new()
            .route("/webhooks/github", post(handle_webhook))
            .with_state(Arc::new(self));

        let addr = format!("0.0.0.0:{}", port)
            .parse()
            .map_err(|e| GitHubError::Config(format!("Invalid address: {}", e)))?;

        info!("Starting GitHub webhook server on {}", addr);

        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await
            .map_err(|e| GitHubError::Config(format!("Server error: {}", e)))?;

        Ok(())
    }
}

/// Webhook handler trait
#[async_trait::async_trait]
pub trait WebhookHandler {
    async fn handle_pull_request(&self, event: PullRequestEvent) -> Result<()>;
    async fn handle_check_run(&self, event: CheckRunEvent) -> Result<()>;
}

/// Pull request event
#[derive(Debug, Clone, Deserialize)]
pub struct PullRequestEvent {
    pub action: String,
    pub number: u64,
    pub pull_request: serde_json::Value,
}

/// Check run event
#[derive(Debug, Clone, Deserialize)]
pub struct CheckRunEvent {
    pub action: String,
    pub check_run: serde_json::Value,
}

/// Webhook event
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "event")]
pub enum WebhookEvent {
    #[serde(rename = "pull_request")]
    PullRequest(PullRequestEvent),

    #[serde(rename = "check_run")]
    CheckRun(CheckRunEvent),
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
        WebhookEvent::PullRequest(event) => {
            info!("Received PR event: {} #{}", event.action, event.number);
            server
                .handler
                .handle_pull_request(event)
                .await
                .map_err(|e| {
                    error!("Failed to handle PR event: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        WebhookEvent::CheckRun(event) => {
            info!("Received check run event: {}", event.action);
            server.handler.handle_check_run(event).await.map_err(|e| {
                error!("Failed to handle check run event: {}", e);
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
            async fn handle_pull_request(&self, _event: PullRequestEvent) -> Result<()> {
                Ok(())
            }

            async fn handle_check_run(&self, _event: CheckRunEvent) -> Result<()> {
                Ok(())
            }
        }

        let handler = Arc::new(TestHandler);
        let server = WebhookServer::new(8081, "secret".to_string(), handler);
        assert_eq!(server.port, 8081);
    }
}
