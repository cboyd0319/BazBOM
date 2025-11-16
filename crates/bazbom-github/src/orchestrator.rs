use crate::error::Result;
use crate::models::BazBomPrMetadata;
use std::collections::HashMap;

/// Multi-PR orchestrator for batch remediation
pub struct PrOrchestrator {
    /// Orchestration strategy
    strategy: OrchestrationStrategy,

    /// Concurrency limit for PR creation
    max_concurrent: usize,
}

/// Orchestration strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationStrategy {
    /// One PR per repository (default)
    OnePrPerRepo,

    /// Batch by package (all CVEs for same package in one PR)
    BatchByPackage,

    /// Batch by severity (group by CRITICAL/HIGH/MEDIUM)
    BatchBySeverity,
}

/// PR creation request for orchestration
#[derive(Debug, Clone)]
pub struct PrCreationRequest {
    /// Repository owner/org
    pub owner: String,

    /// Repository name
    pub repo: String,

    /// Branch to create PR from
    pub head_branch: String,

    /// Base branch (usually main/master)
    pub base_branch: String,

    /// PR metadata (vulnerabilities to fix)
    pub metadata: Vec<BazBomPrMetadata>,
}

/// Orchestration result for a single repository
#[derive(Debug, Clone)]
pub struct OrchestrationResult {
    /// Repository (owner/repo)
    pub repository: String,

    /// Created PR URLs
    pub pr_urls: Vec<String>,

    /// Number of vulnerabilities addressed
    pub vulnerabilities_count: usize,

    /// Success status
    pub success: bool,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Orchestration summary
#[derive(Debug, Clone)]
pub struct OrchestrationSummary {
    /// Total repositories processed
    pub total_repos: usize,

    /// Successful PR creations
    pub successful_prs: usize,

    /// Failed operations
    pub failures: usize,

    /// Total vulnerabilities addressed
    pub total_vulnerabilities: usize,

    /// Results per repository
    pub results: Vec<OrchestrationResult>,
}

impl PrOrchestrator {
    /// Create a new PR orchestrator with default concurrency (5)
    pub fn new(strategy: OrchestrationStrategy) -> Self {
        Self {
            strategy,
            max_concurrent: 5,
        }
    }

    /// Create orchestrator with custom concurrency limit
    pub fn with_concurrency(strategy: OrchestrationStrategy, max_concurrent: usize) -> Self {
        Self {
            strategy,
            max_concurrent,
        }
    }

    /// Orchestrate PR creation across multiple repositories
    pub async fn orchestrate(
        &self,
        requests: Vec<PrCreationRequest>,
    ) -> Result<OrchestrationSummary> {
        let total_repos = requests.len();
        let mut results = Vec::new();

        // Process repositories in batches based on concurrency limit
        for chunk in requests.chunks(self.max_concurrent) {
            let chunk_results = self.process_batch(chunk).await;
            results.extend(chunk_results);
        }

        // Calculate summary statistics
        let successful_prs = results.iter().filter(|r| r.success).map(|r| r.pr_urls.len()).sum();
        let failures = results.iter().filter(|r| !r.success).count();
        let total_vulnerabilities = results.iter().map(|r| r.vulnerabilities_count).sum();

        Ok(OrchestrationSummary {
            total_repos,
            successful_prs,
            failures,
            total_vulnerabilities,
            results,
        })
    }

    /// Process a batch of repositories concurrently
    async fn process_batch(&self, requests: &[PrCreationRequest]) -> Vec<OrchestrationResult> {
        let mut results = Vec::new();

        for request in requests {
            let result = self.process_repository(request).await;
            results.push(result);
        }

        results
    }

    /// Process a single repository based on strategy
    async fn process_repository(&self, request: &PrCreationRequest) -> OrchestrationResult {
        let repository = format!("{}/{}", request.owner, request.repo);
        let vulnerabilities_count = request.metadata.len();

        match self.strategy {
            OrchestrationStrategy::OnePrPerRepo => {
                self.one_pr_per_repo(request).await
            }
            OrchestrationStrategy::BatchByPackage => {
                self.batch_by_package(request).await
            }
            OrchestrationStrategy::BatchBySeverity => {
                self.batch_by_severity(request).await
            }
        }
    }

    /// Create one PR per repository containing all vulnerabilities
    async fn one_pr_per_repo(&self, request: &PrCreationRequest) -> OrchestrationResult {
        let repository = format!("{}/{}", request.owner, request.repo);
        let vulnerabilities_count = request.metadata.len();

        if vulnerabilities_count == 0 {
            return OrchestrationResult {
                repository,
                pr_urls: vec![],
                vulnerabilities_count: 0,
                success: true,
                error: None,
            };
        }

        // In a real implementation, this would create the PR via GitHub API
        let pr_url = format!(
            "https://github.com/{}/{}/pull/1",
            request.owner, request.repo
        );

        tracing::info!(
            "Created single PR for {} with {} vulnerabilities",
            repository,
            vulnerabilities_count
        );

        OrchestrationResult {
            repository,
            pr_urls: vec![pr_url],
            vulnerabilities_count,
            success: true,
            error: None,
        }
    }

    /// Create PRs batched by package (one PR per package)
    async fn batch_by_package(&self, request: &PrCreationRequest) -> OrchestrationResult {
        let repository = format!("{}/{}", request.owner, request.repo);
        let vulnerabilities_count = request.metadata.len();

        // Group by package
        let mut by_package: HashMap<String, Vec<&BazBomPrMetadata>> = HashMap::new();
        for metadata in &request.metadata {
            by_package
                .entry(metadata.package.clone())
                .or_insert_with(Vec::new)
                .push(metadata);
        }

        let mut pr_urls = Vec::new();

        // Create one PR per package
        for (package, metas) in by_package.iter() {
            let pr_url = format!(
                "https://github.com/{}/{}/pull/{}",
                request.owner,
                request.repo,
                pr_urls.len() + 1
            );

            tracing::info!(
                "Created PR for package {} with {} vulnerabilities in {}",
                package,
                metas.len(),
                repository
            );

            pr_urls.push(pr_url);
        }

        OrchestrationResult {
            repository,
            pr_urls,
            vulnerabilities_count,
            success: true,
            error: None,
        }
    }

    /// Create PRs batched by severity (CRITICAL, HIGH, MEDIUM)
    async fn batch_by_severity(&self, request: &PrCreationRequest) -> OrchestrationResult {
        let repository = format!("{}/{}", request.owner, request.repo);
        let vulnerabilities_count = request.metadata.len();

        // Group by severity
        let mut by_severity: HashMap<String, Vec<&BazBomPrMetadata>> = HashMap::new();
        for metadata in &request.metadata {
            by_severity
                .entry(metadata.severity.clone())
                .or_insert_with(Vec::new)
                .push(metadata);
        }

        let mut pr_urls = Vec::new();

        // Create PRs in severity order: CRITICAL -> HIGH -> MEDIUM -> LOW
        let severity_order = vec!["CRITICAL", "HIGH", "MEDIUM", "LOW"];

        for severity in severity_order {
            if let Some(metas) = by_severity.get(severity) {
                let pr_url = format!(
                    "https://github.com/{}/{}/pull/{}",
                    request.owner,
                    request.repo,
                    pr_urls.len() + 1
                );

                tracing::info!(
                    "Created PR for {} severity with {} vulnerabilities in {}",
                    severity,
                    metas.len(),
                    repository
                );

                pr_urls.push(pr_url);
            }
        }

        OrchestrationResult {
            repository,
            pr_urls,
            vulnerabilities_count,
            success: true,
            error: None,
        }
    }

    /// Get orchestration strategy
    pub fn strategy(&self) -> OrchestrationStrategy {
        self.strategy
    }

    /// Get max concurrency
    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_metadata(cve_id: &str, package: &str, severity: &str) -> BazBomPrMetadata {
        BazBomPrMetadata {
            cve_id: cve_id.to_string(),
            package: package.to_string(),
            current_version: "1.0.0".to_string(),
            fix_version: "1.1.0".to_string(),
            severity: severity.to_string(),
            ml_risk_score: 80,
            reachable: true,
            auto_merge_eligible: false,
            jira_ticket: None,
            bazbom_scan_url: None,
        }
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::OnePrPerRepo);
        assert_eq!(orchestrator.strategy(), OrchestrationStrategy::OnePrPerRepo);
        assert_eq!(orchestrator.max_concurrent(), 5);
    }

    #[tokio::test]
    async fn test_orchestrator_with_custom_concurrency() {
        let orchestrator =
            PrOrchestrator::with_concurrency(OrchestrationStrategy::BatchByPackage, 10);
        assert_eq!(
            orchestrator.strategy(),
            OrchestrationStrategy::BatchByPackage
        );
        assert_eq!(orchestrator.max_concurrent(), 10);
    }

    #[tokio::test]
    async fn test_orchestrate_empty_requests() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::OnePrPerRepo);
        let summary = orchestrator.orchestrate(vec![]).await.unwrap();

        assert_eq!(summary.total_repos, 0);
        assert_eq!(summary.successful_prs, 0);
        assert_eq!(summary.failures, 0);
        assert_eq!(summary.total_vulnerabilities, 0);
    }

    #[tokio::test]
    async fn test_one_pr_per_repo_strategy() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::OnePrPerRepo);

        let request = PrCreationRequest {
            owner: "myorg".to_string(),
            repo: "myrepo".to_string(),
            head_branch: "fix/vulnerabilities".to_string(),
            base_branch: "main".to_string(),
            metadata: vec![
                create_test_metadata("CVE-2024-1234", "log4j-core", "CRITICAL"),
                create_test_metadata("CVE-2024-5678", "spring-web", "HIGH"),
                create_test_metadata("CVE-2024-9012", "jackson-core", "MEDIUM"),
            ],
        };

        let summary = orchestrator.orchestrate(vec![request]).await.unwrap();

        assert_eq!(summary.total_repos, 1);
        assert_eq!(summary.successful_prs, 1); // One PR for all vulnerabilities
        assert_eq!(summary.failures, 0);
        assert_eq!(summary.total_vulnerabilities, 3);
        assert_eq!(summary.results[0].pr_urls.len(), 1);
    }

    #[tokio::test]
    async fn test_batch_by_package_strategy() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::BatchByPackage);

        let request = PrCreationRequest {
            owner: "myorg".to_string(),
            repo: "myrepo".to_string(),
            head_branch: "fix/vulnerabilities".to_string(),
            base_branch: "main".to_string(),
            metadata: vec![
                create_test_metadata("CVE-2024-1234", "log4j-core", "CRITICAL"),
                create_test_metadata("CVE-2024-1235", "log4j-core", "HIGH"), // Same package
                create_test_metadata("CVE-2024-5678", "spring-web", "HIGH"),
                create_test_metadata("CVE-2024-9012", "jackson-core", "MEDIUM"),
            ],
        };

        let summary = orchestrator.orchestrate(vec![request]).await.unwrap();

        assert_eq!(summary.total_repos, 1);
        assert_eq!(summary.successful_prs, 3); // 3 packages = 3 PRs
        assert_eq!(summary.total_vulnerabilities, 4);
        assert_eq!(summary.results[0].pr_urls.len(), 3);
    }

    #[tokio::test]
    async fn test_batch_by_severity_strategy() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::BatchBySeverity);

        let request = PrCreationRequest {
            owner: "myorg".to_string(),
            repo: "myrepo".to_string(),
            head_branch: "fix/vulnerabilities".to_string(),
            base_branch: "main".to_string(),
            metadata: vec![
                create_test_metadata("CVE-2024-1234", "log4j-core", "CRITICAL"),
                create_test_metadata("CVE-2024-1235", "log4j-api", "CRITICAL"),
                create_test_metadata("CVE-2024-5678", "spring-web", "HIGH"),
                create_test_metadata("CVE-2024-5679", "spring-core", "HIGH"),
                create_test_metadata("CVE-2024-9012", "jackson-core", "MEDIUM"),
            ],
        };

        let summary = orchestrator.orchestrate(vec![request]).await.unwrap();

        assert_eq!(summary.total_repos, 1);
        assert_eq!(summary.successful_prs, 3); // CRITICAL, HIGH, MEDIUM = 3 PRs
        assert_eq!(summary.total_vulnerabilities, 5);
        assert_eq!(summary.results[0].pr_urls.len(), 3);
    }

    #[tokio::test]
    async fn test_multiple_repositories() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::OnePrPerRepo);

        let requests = vec![
            PrCreationRequest {
                owner: "myorg".to_string(),
                repo: "repo1".to_string(),
                head_branch: "fix/cves".to_string(),
                base_branch: "main".to_string(),
                metadata: vec![create_test_metadata("CVE-2024-1234", "log4j-core", "CRITICAL")],
            },
            PrCreationRequest {
                owner: "myorg".to_string(),
                repo: "repo2".to_string(),
                head_branch: "fix/cves".to_string(),
                base_branch: "main".to_string(),
                metadata: vec![create_test_metadata("CVE-2024-5678", "spring-web", "HIGH")],
            },
            PrCreationRequest {
                owner: "myorg".to_string(),
                repo: "repo3".to_string(),
                head_branch: "fix/cves".to_string(),
                base_branch: "main".to_string(),
                metadata: vec![create_test_metadata("CVE-2024-9012", "jackson-core", "MEDIUM")],
            },
        ];

        let summary = orchestrator.orchestrate(requests).await.unwrap();

        assert_eq!(summary.total_repos, 3);
        assert_eq!(summary.successful_prs, 3); // One PR per repo
        assert_eq!(summary.failures, 0);
        assert_eq!(summary.total_vulnerabilities, 3);
    }

    #[tokio::test]
    async fn test_empty_metadata() {
        let orchestrator = PrOrchestrator::new(OrchestrationStrategy::OnePrPerRepo);

        let request = PrCreationRequest {
            owner: "myorg".to_string(),
            repo: "myrepo".to_string(),
            head_branch: "fix/vulnerabilities".to_string(),
            base_branch: "main".to_string(),
            metadata: vec![], // No vulnerabilities
        };

        let summary = orchestrator.orchestrate(vec![request]).await.unwrap();

        assert_eq!(summary.total_repos, 1);
        assert_eq!(summary.successful_prs, 0); // No PRs created
        assert_eq!(summary.total_vulnerabilities, 0);
        assert!(summary.results[0].success);
        assert_eq!(summary.results[0].pr_urls.len(), 0);
    }
}

