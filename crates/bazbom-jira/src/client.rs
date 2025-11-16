use crate::error::{JiraError, Result};
use crate::models::*;
use governor::{clock::DefaultClock, state::{InMemoryState, NotKeyed}, Quota, RateLimiter};
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, info, warn};

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
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Rate limiter: 5 requests/second (Jira Cloud limit)
        let quota = Quota::per_second(NonZeroU32::new(5).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        info!("Initialized Jira client for {}", base_url);

        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client,
            rate_limiter,
            auth_token: auth_token.to_string(),
            username,
        }
    }

    /// Create issue
    pub async fn create_issue(&self, request: CreateIssueRequest) -> Result<CreateIssueResponse> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue", self.base_url);

        debug!("Creating Jira issue");

        let body = serde_json::to_vec(&request)?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let result: CreateIssueResponse = response.json().await?;
                info!("Created Jira issue: {}", result.key);
                Ok(result)
            }
            StatusCode::BAD_REQUEST => {
                let error_body = response.text().await?;
                warn!("Bad request creating Jira issue: {}", error_body);
                Err(JiraError::BadRequest(error_body))
            }
            StatusCode::UNAUTHORIZED => {
                warn!("Unauthorized when creating Jira issue");
                Err(JiraError::Unauthorized)
            }
            StatusCode::FORBIDDEN => {
                warn!("Forbidden when creating Jira issue");
                Err(JiraError::Forbidden)
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} creating Jira issue: {}", status, error_body);
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Get issue by key
    pub async fn get_issue(&self, issue_key: &str) -> Result<JiraIssue> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        debug!("Getting Jira issue: {}", issue_key);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let issue: JiraIssue = response.json().await?;
                debug!("Retrieved Jira issue: {}", issue.key);
                Ok(issue)
            }
            StatusCode::NOT_FOUND => {
                warn!("Jira issue not found: {}", issue_key);
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            StatusCode::UNAUTHORIZED => {
                warn!("Unauthorized when getting Jira issue");
                Err(JiraError::Unauthorized)
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} getting Jira issue: {}", status, error_body);
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Update issue
    pub async fn update_issue(&self, issue_key: &str, request: UpdateIssueRequest) -> Result<()> {
        self.rate_limiter.until_ready().await;

        let url = format!("{}/rest/api/3/issue/{}", self.base_url, issue_key);

        debug!("Updating Jira issue: {}", issue_key);

        let body = serde_json::to_vec(&request)?;

        let response = self
            .client
            .put(&url)
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => {
                info!("Updated Jira issue: {}", issue_key);
                Ok(())
            }
            StatusCode::BAD_REQUEST => {
                let error_body = response.text().await?;
                warn!("Bad request updating Jira issue: {}", error_body);
                Err(JiraError::BadRequest(error_body))
            }
            StatusCode::NOT_FOUND => {
                warn!("Jira issue not found: {}", issue_key);
                Err(JiraError::IssueNotFound(issue_key.to_string()))
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} updating Jira issue: {}", status, error_body);
                Err(JiraError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Get authentication header
    fn auth_header(&self) -> String {
        if let Some(username) = &self.username {
            // Basic auth: base64(username:token)
            let credentials = format!("{}:{}", username, self.auth_token);
            let encoded = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                credentials.as_bytes(),
            );
            format!("Basic {}", encoded)
        } else {
            // Bearer token (OAuth or PAT)
            format!("Bearer {}", self.auth_token)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = JiraClient::new("https://example.atlassian.net", "test-token");
        assert_eq!(client.base_url, "https://example.atlassian.net");
    }

    #[test]
    fn test_client_creation_with_trailing_slash() {
        let client = JiraClient::new("https://example.atlassian.net/", "test-token");
        assert_eq!(client.base_url, "https://example.atlassian.net");
    }
}
