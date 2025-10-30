use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const DEPSDEV_API_BASE: &str = "https://api.deps.dev/v3alpha";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    pub purl: String,
    pub name: String,
    pub version: String,
    pub licenses: Vec<String>,
    pub description: Option<String>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub popularity_score: Option<f64>,
    pub latest_version: Option<String>,
    pub versions: Vec<VersionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub published_at: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DepsDevResponse {
    #[serde(rename = "packageKey")]
    package_key: PackageKey,
    version: Option<VersionDetails>,
    versions: Option<Vec<DepsDevVersion>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PackageKey {
    name: String,
    system: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct VersionDetails {
    #[serde(rename = "versionKey")]
    version_key: VersionKey,
    licenses: Option<Vec<String>>,
    links: Option<Vec<Link>>,
}

#[derive(Debug, Deserialize)]
struct VersionKey {
    version: String,
}

#[derive(Debug, Deserialize)]
struct Link {
    label: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct DepsDevVersion {
    #[serde(rename = "versionKey")]
    version_key: VersionKey,
    #[serde(rename = "publishedAt")]
    published_at: Option<String>,
    #[serde(rename = "isDefault")]
    is_default: Option<bool>,
}

pub struct DepsDevClient {
    offline: bool,
}

impl DepsDevClient {
    pub fn new(offline: bool) -> Self {
        Self { offline }
    }

    /// Query deps.dev for package information by PURL
    /// Example PURL: pkg:maven/org.apache.commons/commons-io@2.11.0
    pub fn get_package_info(&self, purl: &str) -> Result<PackageInfo> {
        if self.offline {
            return Err(anyhow::anyhow!("offline mode enabled"));
        }

        // Parse PURL to extract system, name, and version
        let (system, name, version) = Self::parse_purl(purl)?;

        // Query the deps.dev API
        let url = format!(
            "{}/systems/{}/packages/{}/versions/{}",
            DEPSDEV_API_BASE,
            Self::system_to_depsdev(&system),
            urlencoding::encode(&name),
            urlencoding::encode(&version)
        );

        let response = ureq::get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .call()
            .context("deps.dev API request failed")?;

        let deps_dev_resp: DepsDevResponse = response
            .into_json()
            .context("failed to parse deps.dev response")?;

        // Extract information
        let licenses = deps_dev_resp
            .version
            .as_ref()
            .and_then(|v| v.licenses.clone())
            .unwrap_or_default();

        let mut homepage = None;
        let mut repository = None;
        let description = None;

        if let Some(version_details) = &deps_dev_resp.version {
            if let Some(links) = &version_details.links {
                for link in links {
                    match link.label.as_str() {
                        "HOMEPAGE" => homepage = Some(link.url.clone()),
                        "SOURCE_REPO" => repository = Some(link.url.clone()),
                        _ => {}
                    }
                }
            }
        }

        // Get version list
        let versions: Vec<VersionInfo> = deps_dev_resp
            .versions
            .unwrap_or_default()
            .iter()
            .map(|v| VersionInfo {
                version: v.version_key.version.clone(),
                published_at: v.published_at.clone(),
                is_default: v.is_default.unwrap_or(false),
            })
            .collect();

        // Find latest version (the one marked as default)
        let latest_version = versions
            .iter()
            .find(|v| v.is_default)
            .map(|v| v.version.clone());

        Ok(PackageInfo {
            purl: purl.to_string(),
            name: name.clone(),
            version: version.clone(),
            licenses,
            description,
            homepage,
            repository,
            popularity_score: None, // Would need additional API call
            latest_version,
            versions,
        })
    }

    fn parse_purl(purl: &str) -> Result<(String, String, String)> {
        // Simple PURL parser: pkg:system/namespace/name@version or pkg:system/name@version
        if !purl.starts_with("pkg:") {
            anyhow::bail!("invalid PURL: must start with 'pkg:'");
        }

        let parts: Vec<&str> = purl[4..].split('@').collect();
        if parts.len() != 2 {
            anyhow::bail!("invalid PURL: must contain version after '@'");
        }

        let version = parts[1];
        let before_version = parts[0];

        let mut components: Vec<&str> = before_version.split('/').collect();
        if components.is_empty() {
            anyhow::bail!("invalid PURL: missing system");
        }

        let system = components.remove(0);
        let name = components.join("/"); // Join remaining parts for namespace/name

        Ok((system.to_string(), name, version.to_string()))
    }

    fn system_to_depsdev(system: &str) -> &str {
        match system {
            "maven" => "maven",
            "npm" => "npm",
            "pypi" => "pypi",
            "cargo" => "cargo",
            "go" => "go",
            _ => system,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_purl_maven() {
        let result = DepsDevClient::parse_purl("pkg:maven/org.apache.commons/commons-io@2.11.0");
        assert!(result.is_ok());
        let (system, name, version) = result.unwrap();
        assert_eq!(system, "maven");
        assert_eq!(name, "org.apache.commons/commons-io");
        assert_eq!(version, "2.11.0");
    }

    #[test]
    fn test_parse_purl_npm() {
        let result = DepsDevClient::parse_purl("pkg:npm/lodash@4.17.21");
        assert!(result.is_ok());
        let (system, name, version) = result.unwrap();
        assert_eq!(system, "npm");
        assert_eq!(name, "lodash");
        assert_eq!(version, "4.17.21");
    }

    #[test]
    fn test_parse_purl_invalid() {
        let result = DepsDevClient::parse_purl("invalid-purl");
        assert!(result.is_err());
    }

    #[test]
    fn test_client_offline() {
        let client = DepsDevClient::new(true);
        let result = client.get_package_info("pkg:maven/test/test@1.0.0");
        assert!(result.is_err());
    }
}
