use crate::error::{DepsDevError, Result};
use crate::models::*;
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};

/// Client for the deps.dev API
///
/// ## Example
///
/// ```no_run
/// use bazbom_depsdev::{DepsDevClient, System};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let client = DepsDevClient::new();
///
///     let version_info = client.get_version(
///         System::Maven,
///         "org.apache.logging.log4j:log4j-core",
///         "2.20.0"
///     ).await?;
///
///     println!("Published: {}", version_info.published_at);
///     println!("Licenses: {:?}", version_info.licenses);
///
///     Ok(())
/// }
/// ```
pub struct DepsDevClient {
    client: Client,
    base_url: String,
}

impl DepsDevClient {
    /// Create a new deps.dev API client
    pub fn new() -> Self {
        Self::with_base_url("https://api.deps.dev/v3")
    }

    /// Create a client with a custom base URL (for testing)
    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("BazBOM/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url: base_url.into(),
        }
    }

    /// Get version information for a specific package version
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use bazbom_depsdev::{DepsDevClient, System};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = DepsDevClient::new();
    /// let info = client.get_version(
    ///     System::Maven,
    ///     "com.google.guava:guava",
    ///     "32.0.0-jre"
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_version(
        &self,
        system: System,
        package: &str,
        version: &str,
    ) -> Result<VersionInfo> {
        let url = format!(
            "{}/systems/{}/packages/{}/versions/{}",
            self.base_url,
            system.as_str(),
            urlencoding::encode(package),
            urlencoding::encode(version)
        );

        debug!("Fetching version info: {}", url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let version_info = response.json::<VersionInfo>().await?;
            Ok(version_info)
        } else if response.status() == 404 {
            Err(DepsDevError::PackageNotFound {
                system: system.as_str().to_string(),
                name: package.to_string(),
                version: version.to_string(),
            })
        } else if response.status() == 429 {
            Err(DepsDevError::RateLimited)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DepsDevError::ApiError(format!("HTTP {}: {}", status, body)))
        }
    }

    /// Get the resolved dependency graph for a package version
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use bazbom_depsdev::{DepsDevClient, System};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = DepsDevClient::new();
    /// let deps = client.get_dependencies(
    ///     System::Maven,
    ///     "org.springframework.boot:spring-boot-starter-web",
    ///     "3.2.0"
    /// ).await?;
    ///
    /// println!("Total dependencies: {}", deps.nodes.len());
    /// println!("Direct dependencies: {}", deps.direct_dependencies().len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_dependencies(
        &self,
        system: System,
        package: &str,
        version: &str,
    ) -> Result<DependencyGraph> {
        let url = format!(
            "{}/systems/{}/packages/{}/versions/{}:dependencies",
            self.base_url,
            system.as_str(),
            urlencoding::encode(package),
            urlencoding::encode(version)
        );

        debug!("Fetching dependency graph: {}", url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let graph = response.json::<DependencyGraph>().await?;
            Ok(graph)
        } else if response.status() == 404 {
            warn!(
                "Dependency graph not available for {}/{}@{}",
                system.as_str(),
                package,
                version
            );
            // Return empty graph instead of error (some packages don't have dep graphs)
            Ok(DependencyGraph {
                nodes: vec![],
                edges: vec![],
            })
        } else if response.status() == 429 {
            Err(DepsDevError::RateLimited)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DepsDevError::ApiError(format!("HTTP {}: {}", status, body)))
        }
    }

    /// Get package metadata including all available versions
    ///
    /// ## Example
    ///
    /// ```no_run
    /// # use bazbom_depsdev::{DepsDevClient, System};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let client = DepsDevClient::new();
    /// let package = client.get_package(
    ///     System::Maven,
    ///     "org.apache.logging.log4j:log4j-core"
    /// ).await?;
    ///
    /// println!("Available versions: {:?}", package.versions);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_package(&self, system: System, package: &str) -> Result<Package> {
        let url = format!(
            "{}/systems/{}/packages/{}",
            self.base_url,
            system.as_str(),
            urlencoding::encode(package)
        );

        debug!("Fetching package info: {}", url);

        let response = self.client.get(&url).send().await?;

        if response.status().is_success() {
            let package = response.json::<Package>().await?;
            Ok(package)
        } else if response.status() == 404 {
            Err(DepsDevError::PackageNotFound {
                system: system.as_str().to_string(),
                name: package.to_string(),
                version: "N/A".to_string(),
            })
        } else if response.status() == 429 {
            Err(DepsDevError::RateLimited)
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(DepsDevError::ApiError(format!("HTTP {}: {}", status, body)))
        }
    }

    /// Find the GitHub repository URL for a package version
    ///
    /// Returns None if no GitHub repository is found.
    pub async fn find_github_repo(
        &self,
        system: System,
        package: &str,
        version: &str,
    ) -> Result<Option<String>> {
        let version_info = self.get_version(system, package, version).await?;

        // Check repository link first
        if let Some(repo) = version_info.links.repository {
            if repo.contains("github.com") {
                return Ok(Some(repo));
            }
        }

        // Check homepage as fallback
        if let Some(homepage) = version_info.links.homepage {
            if homepage.contains("github.com") {
                return Ok(Some(homepage));
            }
        }

        Ok(None)
    }
}

impl Default for DepsDevClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_version() {
        let client = DepsDevClient::new();
        let result = client
            .get_version(System::Maven, "com.google.guava:guava", "32.0.0-jre")
            .await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert!(info.licenses.contains(&"Apache-2.0".to_string()));
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_get_dependencies() {
        let client = DepsDevClient::new();
        let result = client
            .get_dependencies(
                System::Maven,
                "org.springframework.boot:spring-boot-starter-web",
                "3.2.0",
            )
            .await;

        assert!(result.is_ok());
        let graph = result.unwrap();
        assert!(!graph.nodes.is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_find_github_repo() {
        let client = DepsDevClient::new();
        let result = client
            .find_github_repo(
                System::Maven,
                "org.apache.logging.log4j:log4j-core",
                "2.20.0",
            )
            .await;

        assert!(result.is_ok());
        let repo = result.unwrap();
        assert!(repo.is_some());
        assert!(repo.unwrap().contains("github.com"));
    }
}
