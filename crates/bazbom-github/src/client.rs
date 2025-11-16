use crate::error::{GitHubError, Result};
use crate::models::*;
use governor::{clock::DefaultClock, state::{InMemoryState, NotKeyed}, Quota, RateLimiter};
use reqwest::{Client, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use std::num::NonZeroU32;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// GitHub API client
pub struct GitHubClient {
    /// HTTP client with retry middleware
    client: ClientWithMiddleware,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,

    /// Authentication token (PAT)
    token: String,
}

impl GitHubClient {
    /// Create new GitHub client
    pub fn new(token: &str) -> Self {
        // Configure retry policy: 3 retries with exponential backoff
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let client = ClientBuilder::new(Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        // Rate limiter: 60 requests/minute (GitHub unauthenticated limit)
        // Authenticated limit is 5000/hour, but we'll use conservative 60/minute
        let quota = Quota::per_minute(NonZeroU32::new(60).unwrap());
        let rate_limiter = Arc::new(RateLimiter::direct(quota));

        info!("Initialized GitHub client");

        Self {
            client,
            rate_limiter,
            token: token.to_string(),
        }
    }

    /// Create a pull request
    pub async fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        request: CreatePullRequestRequest,
    ) -> Result<PullRequest> {
        self.rate_limiter.until_ready().await;

        let url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo);

        debug!("Creating GitHub PR: {} -> {}", request.head, request.base);

        let body = serde_json::to_vec(&request)?;

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "BazBOM/6.8.0")
            .body(body)
            .send()
            .await?;

        match response.status() {
            StatusCode::CREATED => {
                let pr: PullRequest = response.json().await?;
                info!("Created GitHub PR #{}: {}", pr.number, pr.title);
                Ok(pr)
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error_body = response.text().await?;
                warn!("Validation error creating PR: {}", error_body);
                Err(GitHubError::BadRequest(error_body))
            }
            StatusCode::UNAUTHORIZED => {
                warn!("Unauthorized when creating PR");
                Err(GitHubError::Unauthorized)
            }
            StatusCode::FORBIDDEN => {
                warn!("Forbidden when creating PR");
                Err(GitHubError::Forbidden)
            }
            StatusCode::NOT_FOUND => {
                warn!("Repository not found: {}/{}", owner, repo);
                Err(GitHubError::RepositoryNotFound(format!("{}/{}", owner, repo)))
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} creating PR: {}", status, error_body);
                Err(GitHubError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Get a pull request
    pub async fn get_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
    ) -> Result<PullRequest> {
        self.rate_limiter.until_ready().await;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        debug!("Getting GitHub PR #{}", pr_number);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "BazBOM/6.8.0")
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let pr: PullRequest = response.json().await?;
                debug!("Retrieved GitHub PR #{}", pr.number);
                Ok(pr)
            }
            StatusCode::NOT_FOUND => {
                warn!("PR #{} not found", pr_number);
                Err(GitHubError::PullRequestNotFound(pr_number))
            }
            StatusCode::UNAUTHORIZED => {
                warn!("Unauthorized when getting PR");
                Err(GitHubError::Unauthorized)
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} getting PR: {}", status, error_body);
                Err(GitHubError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }

    /// Update a pull request
    pub async fn update_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr_number: u64,
        request: UpdatePullRequestRequest,
    ) -> Result<PullRequest> {
        self.rate_limiter.until_ready().await;

        let url = format!(
            "https://api.github.com/repos/{}/{}/pulls/{}",
            owner, repo, pr_number
        );

        debug!("Updating GitHub PR #{}", pr_number);

        let body = serde_json::to_vec(&request)?;

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("X-GitHub-Api-Version", "2022-11-28")
            .header("User-Agent", "BazBOM/6.8.0")
            .body(body)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let pr: PullRequest = response.json().await?;
                info!("Updated GitHub PR #{}", pr.number);
                Ok(pr)
            }
            StatusCode::NOT_FOUND => {
                warn!("PR #{} not found", pr_number);
                Err(GitHubError::PullRequestNotFound(pr_number))
            }
            StatusCode::UNPROCESSABLE_ENTITY => {
                let error_body = response.text().await?;
                warn!("Validation error updating PR: {}", error_body);
                Err(GitHubError::BadRequest(error_body))
            }
            status => {
                let error_body = response.text().await?;
                warn!("Unexpected status {} updating PR: {}", status, error_body);
                Err(GitHubError::UnexpectedStatus(status.as_u16(), error_body))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new("test-token");
        assert_eq!(client.token, "test-token");
    }
}
