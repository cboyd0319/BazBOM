use crate::error::{GitHubError, Result};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::post,
    Router,
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;
use tracing::{error, info, warn};

type HmacSha256 = Hmac<Sha256>;

/// Webhook server for receiving GitHub events
pub struct WebhookServer {
    port: u16,
    secret: String,
    handler: Arc<dyn WebhookHandler + Send + Sync>,
}

/// Verify GitHub webhook signature using HMAC-SHA256
fn verify_signature(secret: &str, signature_header: Option<&str>, body: &[u8]) -> bool {
    let signature = match signature_header {
        Some(sig) => sig,
        None => {
            warn!("Missing X-Hub-Signature-256 header");
            return false;
        }
    };

    // GitHub sends signature as "sha256=<hex>"
    let expected_signature = match signature.strip_prefix("sha256=") {
        Some(sig) => sig,
        None => {
            warn!("Invalid signature format: {}", signature);
            return false;
        }
    };

    // Compute HMAC-SHA256
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(mac) => mac,
        Err(e) => {
            error!("Failed to create HMAC: {}", e);
            return false;
        }
    };
    mac.update(body);
    let computed = mac.finalize().into_bytes();
    let computed_hex = hex::encode(computed);

    // Constant-time comparison to prevent timing attacks
    if computed_hex.len() != expected_signature.len() {
        return false;
    }

    computed_hex
        .bytes()
        .zip(expected_signature.bytes())
        .fold(0, |acc, (a, b)| acc | (a ^ b))
        == 0
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
    headers: HeaderMap,
    body: Bytes,
) -> std::result::Result<StatusCode, StatusCode> {
    // Verify webhook signature
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok());

    if !verify_signature(&server.secret, signature, &body) {
        warn!("Invalid webhook signature - rejecting request");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Parse the event from the body
    let event: WebhookEvent = serde_json::from_slice(&body).map_err(|e| {
        error!("Failed to parse webhook event: {}", e);
        StatusCode::BAD_REQUEST
    })?;

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

    #[test]
    fn test_verify_signature_valid() {
        let secret = "test-secret";
        let body = b"test body";

        // Compute expected signature
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature = format!("sha256={}", expected);

        assert!(verify_signature(secret, Some(&signature), body));
    }

    #[test]
    fn test_verify_signature_invalid() {
        let secret = "test-secret";
        let body = b"test body";
        let wrong_signature = "sha256=0000000000000000000000000000000000000000000000000000000000000000";

        assert!(!verify_signature(secret, Some(wrong_signature), body));
    }

    #[test]
    fn test_verify_signature_missing_header() {
        let secret = "test-secret";
        let body = b"test body";

        assert!(!verify_signature(secret, None, body));
    }

    #[test]
    fn test_verify_signature_invalid_format() {
        let secret = "test-secret";
        let body = b"test body";

        // Missing sha256= prefix
        assert!(!verify_signature(secret, Some("invalid-format"), body));
    }

    #[test]
    fn test_verify_signature_tampered_body() {
        let secret = "test-secret";
        let original_body = b"original body";
        let tampered_body = b"tampered body";

        // Compute signature for original body
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(original_body);
        let expected = hex::encode(mac.finalize().into_bytes());
        let signature = format!("sha256={}", expected);

        // Verify with tampered body should fail
        assert!(!verify_signature(secret, Some(&signature), tampered_body));
    }
}
