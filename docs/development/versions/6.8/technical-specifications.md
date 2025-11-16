# BazBOM v6.8 - Jira Integration Technical Specifications

**Version:** 6.8
**Last Updated:** 2025-11-16
**Status:** Planning

## Table of Contents

1. [Architecture](#architecture)
2. [Data Models](#data-models)
3. [API Specifications](#api-specifications)
4. [Webhook Protocol](#webhook-protocol)
5. [Configuration Schema](#configuration-schema)
6. [Database Schema](#database-schema)
7. [Error Handling](#error-handling)
8. [Performance Requirements](#performance-requirements)

---

## Architecture

### Component Design

```
crates/
├── bazbom-jira/                    # New crate for Jira integration
│   ├── src/
│   │   ├── lib.rs                  # Public API
│   │   ├── client.rs               # Jira REST API client
│   │   ├── models.rs               # Jira data models
│   │   ├── webhook.rs              # Webhook server and processor
│   │   ├── config.rs               # Configuration management
│   │   ├── templates.rs            # Ticket templates
│   │   ├── sync.rs                 # Bidirectional sync engine
│   │   ├── routing.rs              # Team/component routing
│   │   └── error.rs                # Error types
│   ├── tests/
│   │   ├── integration_tests.rs
│   │   └── fixtures/
│   └── Cargo.toml
│
├── bazbom-core/                    # Enhanced for Jira
│   └── src/
│       ├── jira.rs                 # Jira config models (new)
│       └── vulnerability.rs        # Add Jira metadata fields
│
└── bazbom/                         # CLI enhancements
    └── src/
        └── commands/
            └── jira.rs             # New Jira CLI commands
```

### Crate Dependencies

**`bazbom-jira/Cargo.toml`:**
```toml
[package]
name = "bazbom-jira"
version = "6.8.0"
edition = "2021"
license = "MIT"
repository = "https://github.com/cboyd0319/BazBOM"

[dependencies]
# Core
bazbom-core = { path = "../bazbom-core" }
anyhow = "1"
thiserror = "1"
tracing = "0.1"

# HTTP client
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
reqwest-middleware = "0.4"
reqwest-retry = "0.7"

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Async runtime
tokio = { version = "1", features = ["full"] }
futures = "0.3"

# Rate limiting
governor = "0.7"
tower = "0.5"

# Security
hmac = "0.13"
sha2 = "0.10"
hex = "0.4"
subtle = "2.6"

# Webhook server
axum = "0.8"
axum-server = { version = "0.7", features = ["tls-rustls"] }

# Utilities
chrono = { version = "0.4", features = ["serde"] }
url = "2"
urlencoding = "2"

[dev-dependencies]
wiremock = "0.6"
tokio-test = "0.4"
assert-json-diff = "2"
```

---

## Data Models

### Core Models

**`bazbom-jira/src/models.rs`:**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Jira issue representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssue {
    /// Issue key (e.g., "SEC-123")
    pub key: String,

    /// Issue ID (internal Jira ID)
    pub id: String,

    /// Fields
    pub fields: JiraFields,
}

/// Jira issue fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraFields {
    /// Project
    pub project: JiraProject,

    /// Issue type
    #[serde(rename = "issuetype")]
    pub issue_type: JiraIssueType,

    /// Summary (title)
    pub summary: String,

    /// Description (body)
    pub description: JiraDescription,

    /// Priority
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<JiraPriority>,

    /// Assignee
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<JiraUser>,

    /// Status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<JiraStatus>,

    /// Labels
    #[serde(default)]
    pub labels: Vec<String>,

    /// Components
    #[serde(default)]
    pub components: Vec<JiraComponent>,

    /// Custom fields
    #[serde(flatten)]
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Jira project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraProject {
    /// Project key (e.g., "SEC")
    pub key: String,

    /// Project ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira issue type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraIssueType {
    /// Type name (e.g., "Bug", "Task")
    pub name: String,

    /// Type ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira description (Atlassian Document Format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraDescription {
    #[serde(rename = "type")]
    pub doc_type: String,  // "doc"

    pub version: i32,  // 1

    pub content: Vec<JiraContent>,
}

/// Jira content block
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum JiraContent {
    #[serde(rename = "paragraph")]
    Paragraph {
        content: Vec<JiraTextNode>,
    },

    #[serde(rename = "heading")]
    Heading {
        attrs: HeadingAttrs,
        content: Vec<JiraTextNode>,
    },

    #[serde(rename = "codeBlock")]
    CodeBlock {
        attrs: CodeBlockAttrs,
        content: Vec<JiraTextNode>,
    },

    #[serde(rename = "bulletList")]
    BulletList {
        content: Vec<ListItem>,
    },
}

/// Jira text node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTextNode {
    #[serde(rename = "type")]
    pub node_type: String,  // "text"

    pub text: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<TextMark>>,
}

/// Text mark (bold, italic, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMark {
    #[serde(rename = "type")]
    pub mark_type: String,  // "strong", "em", "code", "link"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, serde_json::Value>>,
}

/// Heading attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadingAttrs {
    pub level: i32,  // 1-6
}

/// Code block attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlockAttrs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,  // "bash", "rust", etc.
}

/// List item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    #[serde(rename = "type")]
    pub item_type: String,  // "listItem"

    pub content: Vec<JiraContent>,
}

/// Jira priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraPriority {
    pub name: String,  // "Highest", "High", "Medium", "Low", "Lowest"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "accountId")]
    pub account_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

/// Jira status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraStatus {
    pub name: String,  // "To Do", "In Progress", "Done"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Jira component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComponent {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// BazBOM-specific Jira metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BazBomJiraMetadata {
    /// CVE ID
    pub cve_id: String,

    /// CVSS score
    pub cvss_score: f32,

    /// EPSS score
    pub epss_score: Option<f32>,

    /// CISA KEV status
    pub kev_status: bool,

    /// Reachability
    pub reachability: Reachability,

    /// Package PURL
    pub package_purl: String,

    /// Current version
    pub current_version: String,

    /// Fix version
    pub fix_version: Option<String>,

    /// Remediation effort estimate
    pub remediation_effort: RemediationEffort,

    /// BazBOM scan URL
    pub bazbom_link: Option<String>,
}

/// Reachability status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Reachability {
    Reachable,
    Unreachable,
    Unknown,
}

/// Remediation effort estimate
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RemediationEffort {
    #[serde(rename = "<1h")]
    LessThanOneHour,

    #[serde(rename = "1-4h")]
    OneToFourHours,

    #[serde(rename = "1d")]
    OneDay,

    #[serde(rename = "1w")]
    OneWeek,

    #[serde(rename = ">1w")]
    MoreThanOneWeek,
}

/// Issue creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub fields: JiraFields,
}

/// Issue creation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueResponse {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_url: String,
}

/// Issue update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<HashMap<String, serde_json::Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<HashMap<String, Vec<UpdateOperation>>>,
}

/// Update operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateOperation {
    pub operation: String,  // "add", "set", "remove"

    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// Transition request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRequest {
    pub transition: Transition,
}

/// Transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub id: String,
}

/// Comment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddCommentRequest {
    pub body: JiraDescription,
}

/// Search request (JQL)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub jql: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<String>>,

    #[serde(rename = "startAt", skip_serializing_if = "Option::is_none")]
    pub start_at: Option<i32>,

    #[serde(rename = "maxResults", skip_serializing_if = "Option::is_none")]
    pub max_results: Option<i32>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total: i32,

    #[serde(rename = "startAt")]
    pub start_at: i32,

    #[serde(rename = "maxResults")]
    pub max_results: i32,

    pub issues: Vec<JiraIssue>,
}
```

---

## API Specifications

### Jira REST API Client

**`bazbom-jira/src/client.rs`:**

```rust
use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use governor::{Quota, RateLimiter, state::InMemoryState, clock::DefaultClock};
use std::num::NonZeroU32;
use std::sync::Arc;
use crate::models::*;
use crate::error::JiraError;

/// Jira API client
pub struct JiraClient {
    /// Base URL (e.g., "https://example.atlassian.net")
    base_url: String,

    /// HTTP client with retry middleware
    client: ClientWithMiddleware,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,

    /// Authentication token
    auth_token: String,

    /// Username (for Basic auth)
    username: Option<String>,
}

impl JiraClient {
    /// Create new Jira client
    pub fn new(base_url: &str, auth_token: &str) -> Self {
        Self::with_username(base_url, auth_token, None)
    }

    /// Create new Jira client with username (for Basic auth)
    pub fn with_username(
        base_url: &str,
        auth_token: &str,
        username: Option<String>,
    ) -> Self {
        // Configure retry policy: 3 retries with exponential backoff
        let retry_policy = ExponentialBackoff::builder()
            .build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Rate limiter: 5 requests/second (Jira Cloud limit)
        let quota = Quota::per_second(NonZeroU32::new(5).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            rate_limiter,
            auth_token: auth_token.to_string(),
            username,
        }
    }

    /// Create issue
    pub async fn create_issue(
        &self,
        request: CreateIssueRequest,
    ) -> Result<CreateIssueResponse, JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue", self.base_url);

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let result: CreateIssueResponse = response.json().await?;
                Ok(result)
            }
            StatusCode::BAD_REQUEST => {
                let error_body = response.text().await?;
                Err(JiraError::BadRequest(error_body))
            }
            StatusCode::UNAUTHORIZED => {
                Err(JiraError::Unauthorized)
            }
            StatusCode::FORBIDDEN => {
                Err(JiraError::Forbidden)
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Get issue by key
    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue, JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        let response = self.client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let issue: JiraIssue = response.json().await?;
                Ok(issue)
            }
            StatusCode::NOT_FOUND => {
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            StatusCode::UNAUTHORIZED => {
                Err(JiraError::Unauthorized)
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Update issue
    pub async fn update_issue(
        &self,
        issue_key: &str,
        request: UpdateIssueRequest,
    ) -> Result<(), JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        let response = self.client
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::BAD_REQUEST => {
                let error_body = response.text().await?;
                Err(JiraError::BadRequest(error_body))
            }
            StatusCode::NOT_FOUND => {
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Add comment to issue
    pub async fn add_comment(
        &self,
        issue_key: &str,
        comment: AddCommentRequest,
    ) -> Result<(), JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}/comment", self.base_url, issue_key);

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&comment)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => Ok(()),
            StatusCode::NOT_FOUND => {
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Transition issue
    pub async fn transition_issue(
        &self,
        issue_key: &str,
        transition_id: &str,
    ) -> Result<(), JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, issue_key);

        let request = TransitionRequest {
            transition: Transition {
                id: transition_id.to_string(),
            },
        };

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => {
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Search issues using JQL
    pub async fn search(
        &self,
        jql: &str,
        start_at: i32,
        max_results: i32,
    ) -> Result<SearchResponse, JiraError> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/search", self.base_url);

        let request = SearchRequest {
            jql: jql.to_string(),
            fields: None,
            start_at: Some(start_at),
            max_results: Some(max_results),
        };

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let result: SearchResponse = response.json().await?;
                Ok(result)
            }
            StatusCode::BAD_REQUEST => {
                let error_body = response.text().await?;
                Err(JiraError::BadRequest(error_body))
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Bulk create issues (up to 50)
    pub async fn bulk_create_issues(
        &self,
        requests: Vec<CreateIssueRequest>,
    ) -> Result<Vec<Result<CreateIssueResponse, JiraError>>, JiraError> {
        if requests.len() > 50 {
            return Err(JiraError::BulkLimitExceeded(requests.len()));
        }

        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/bulk", self.base_url);

        #[derive(Serialize)]
        struct BulkRequest {
            #[serde(rename = "issueUpdates")]
            issue_updates: Vec<CreateIssueRequest>,
        }

        let bulk_request = BulkRequest {
            issue_updates: requests,
        };

        let response = self.client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .json(&bulk_request)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                #[derive(Deserialize)]
                struct BulkResponse {
                    issues: Vec<CreateIssueResponse>,
                    errors: Vec<serde_json::Value>,
                }

                let result: BulkResponse = response.json().await?;
                let responses = result.issues.into_iter()
                    .map(Ok)
                    .collect();
                Ok(responses)
            }
            status => {
                let error_body = response.text().await?;
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Get authentication header
    fn auth_header(&self) -> String {
        if let Some(username) = &self.username {
            // Basic auth: base64(username:token)
            let credentials = format!("{}:{}", username, self.auth_token);
            let encoded = base64::encode(&credentials);
            format!("Basic {}", encoded)
        } else {
            // Bearer token (OAuth or PAT)
            format!("Bearer {}", self.auth_token)
        }
    }
}
```

---

## Webhook Protocol

### Webhook Server

**`bazbom-jira/src/webhook.rs`:**

```rust
use axum::{
    Router,
    routing::post,
    extract::{State, Json},
    http::StatusCode,
};
use axum_server::tls_rustls::RustlsConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, warn, error};
use hmac::{Hmac, Mac};
use sha2::Sha256;

/// Webhook server
pub struct WebhookServer {
    port: u16,
    secret: String,
    handler: Arc<dyn WebhookHandler + Send + Sync>,
}

impl WebhookServer {
    pub fn new(port: u16, secret: String, handler: Arc<dyn WebhookHandler + Send + Sync>) -> Self {
        Self { port, secret, handler }
    }

    /// Start webhook server
    pub async fn start(self) -> Result<(), anyhow::Error> {
        let app = Router::new()
            .route("/webhooks/jira", post(handle_webhook))
            .with_state(Arc::new(self));

        let addr = format!("0.0.0.0:{}", self.port).parse()?;
        info!("Starting Jira webhook server on {}", addr);

        axum_server::bind(addr)
            .serve(app.into_make_service())
            .await?;

        Ok(())
    }
}

/// Webhook handler trait
#[async_trait::async_trait]
pub trait WebhookHandler {
    async fn handle_issue_updated(&self, event: IssueUpdatedEvent) -> Result<(), anyhow::Error>;
    async fn handle_comment_created(&self, event: CommentCreatedEvent) -> Result<(), anyhow::Error>;
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
    headers: axum::http::HeaderMap,
    Json(event): Json<WebhookEvent>,
) -> Result<StatusCode, StatusCode> {
    // Verify webhook signature
    if let Some(signature) = headers.get("X-Hub-Signature-256") {
        // TODO: Implement signature verification
    }

    // Process event
    match event {
        WebhookEvent::IssueUpdated(event) => {
            info!("Received issue updated event: {}", event.issue.key);
            server.handler.handle_issue_updated(event).await
                .map_err(|e| {
                    error!("Failed to handle issue updated event: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }

        WebhookEvent::CommentCreated(event) => {
            info!("Received comment created event on issue: {}", event.issue.key);
            server.handler.handle_comment_created(event).await
                .map_err(|e| {
                    error!("Failed to handle comment created event: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    Ok(StatusCode::OK)
}
```

---

## Configuration Schema

**`.bazbom/jira.yml` JSON Schema:**

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "BazBOM Jira Configuration",
  "type": "object",
  "required": ["jira"],
  "properties": {
    "jira": {
      "type": "object",
      "required": ["url", "auth", "project"],
      "properties": {
        "url": {
          "type": "string",
          "format": "uri",
          "description": "Jira instance URL"
        },
        "auth": {
          "type": "object",
          "required": ["type"],
          "properties": {
            "type": {
              "type": "string",
              "enum": ["api-token", "pat", "oauth2"]
            },
            "token_env": {
              "type": "string",
              "description": "Environment variable containing auth token"
            },
            "username_env": {
              "type": "string",
              "description": "Environment variable containing username (for Basic auth)"
            }
          }
        },
        "project": {
          "type": "string",
          "pattern": "^[A-Z]+$",
          "description": "Default Jira project key"
        },
        "issue_type": {
          "type": "string",
          "default": "Bug"
        },
        "auto_create": {
          "type": "object",
          "properties": {
            "enabled": {
              "type": "boolean",
              "default": false
            },
            "min_priority": {
              "type": "string",
              "enum": ["P0", "P1", "P2", "P3", "P4"]
            },
            "only_reachable": {
              "type": "boolean",
              "default": true
            }
          }
        },
        "custom_fields": {
          "type": "object",
          "additionalProperties": {
            "type": "string",
            "pattern": "^customfield_\\d+$"
          }
        },
        "routing": {
          "type": "array",
          "items": {
            "type": "object",
            "required": ["pattern"],
            "properties": {
              "pattern": {
                "type": "string",
                "description": "Regex pattern for package matching"
              },
              "project": {"type": "string"},
              "component": {"type": "string"},
              "assignee": {"type": "string"},
              "labels": {
                "type": "array",
                "items": {"type": "string"}
              },
              "priority": {
                "type": "string",
                "enum": ["Highest", "High", "Medium", "Low", "Lowest"]
              }
            }
          }
        },
        "sla": {
          "type": "object",
          "properties": {
            "P0": {"type": "string"},
            "P1": {"type": "string"},
            "P2": {"type": "string"},
            "P3": {"type": "string"},
            "P4": {"type": "string"}
          }
        },
        "templates": {
          "type": "object",
          "properties": {
            "title": {"type": "string"},
            "description": {"type": "string"}
          }
        },
        "sync": {
          "type": "object",
          "properties": {
            "bidirectional": {"type": "boolean"},
            "auto_close_on_fix": {"type": "boolean"},
            "update_on_rescan": {"type": "boolean"}
          }
        },
        "webhook": {
          "type": "object",
          "properties": {
            "enabled": {"type": "boolean"},
            "port": {"type": "integer"},
            "secret_env": {"type": "string"}
          }
        }
      }
    }
  }
}
```

---

## Database Schema

**Jira tracking table (SQLite):**

```sql
CREATE TABLE jira_issues (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Jira issue info
    issue_key TEXT NOT NULL UNIQUE,
    issue_id TEXT NOT NULL,
    project_key TEXT NOT NULL,

    -- CVE info
    cve_id TEXT NOT NULL,
    package_purl TEXT NOT NULL,

    -- Status
    status TEXT NOT NULL,  -- "open", "in_progress", "done", "rejected"

    -- Timestamps
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    closed_at TEXT,

    -- Metadata
    metadata TEXT,  -- JSON

    INDEX idx_cve_id (cve_id),
    INDEX idx_issue_key (issue_key),
    INDEX idx_status (status)
);

CREATE TABLE jira_sync_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- Event info
    event_type TEXT NOT NULL,  -- "created", "updated", "closed"
    issue_key TEXT NOT NULL,

    -- Details
    details TEXT,  -- JSON

    -- Timestamp
    timestamp TEXT NOT NULL,

    INDEX idx_issue_key (issue_key),
    INDEX idx_timestamp (timestamp)
);
```

---

## Error Handling

**`bazbom-jira/src/error.rs`:**

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum JiraError {
    #[error("Unauthorized: Invalid credentials")]
    Unauthorized,

    #[error("Forbidden: Insufficient permissions")]
    Forbidden,

    #[error("Issue not found: {0}")]
    IssueNotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unexpected HTTP status {0}: {1}")]
    UnexpectedStatus(u16, String),

    #[error("Bulk operation limit exceeded: {0} issues (max 50)")]
    BulkLimitExceeded(usize),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    Config(String),
}
```

---

## Performance Requirements

### Latency Targets

| Operation | Target (p95) | Target (p99) |
|-----------|--------------|--------------|
| Create single issue | <2s | <5s |
| Bulk create 50 issues | <10s | <20s |
| Update issue | <1s | <2s |
| Search (JQL) | <3s | <5s |
| Webhook processing | <500ms | <1s |

### Throughput Targets

- **Ticket Creation:** 100 tickets/minute sustained
- **Webhook Events:** 1000 events/hour
- **JQL Queries:** 50 queries/minute

### Resource Limits

- **Memory:** <100 MB for webhook server
- **CPU:** <10% for background sync
- **Disk:** <10 MB for local tracking database

---

## Appendices

### A. Jira API Version Compatibility

| Jira Version | REST API | Support Status |
|--------------|----------|----------------|
| Cloud | v3 | Primary |
| Server 9.x | v2 | Supported |
| Data Center 9.x | v2 | Supported |
| Server <8.x | v2 | Best Effort |

### B. Custom Field IDs

Custom field IDs vary per Jira instance. Users must configure mappings:

```yaml
custom_fields:
  cve_id: customfield_10001      # Text
  cvss_score: customfield_10002   # Number
  epss_score: customfield_10003   # Number
  kev_status: customfield_10004   # Checkbox
  reachability: customfield_10005 # Select
  # ... etc
```

### C. Jira Transition IDs

Transition IDs are workflow-specific. Common mappings:

```yaml
transitions:
  to_do: "11"          # Open → To Do
  in_progress: "21"    # To Do → In Progress
  done: "31"           # In Progress → Done
  rejected: "41"       # * → Rejected
```

---

**Next:** See [Implementation Roadmap](implementation-roadmap.md)
