use crate::models::BreakingChange;
use anyhow::Result;
use octocrab::Octocrab;
use regex::Regex;
use tracing::{debug, warn};

/// GitHub release notes analyzer
pub struct GitHubAnalyzer {
    client: Octocrab,
}

impl GitHubAnalyzer {
    pub fn new() -> Result<Self> {
        let client = Octocrab::builder().build()?;
        Ok(Self { client })
    }

    /// Parse owner and repo from GitHub URL
    fn parse_repo_url(&self, url: &str) -> Option<(String, String)> {
        // Match: https://github.com/owner/repo or git@github.com:owner/repo.git
        let re = Regex::new(r"github\.com[:/]([^/]+)/([^/\.]+)").ok()?;
        let caps = re.captures(url)?;
        Some((caps[1].to_string(), caps[2].to_string()))
    }

    /// Find breaking changes between two versions by analyzing release notes
    pub async fn analyze_upgrade(
        &self,
        repo_url: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<Vec<BreakingChange>> {
        let (owner, repo) = match self.parse_repo_url(repo_url) {
            Some(parsed) => parsed,
            None => {
                warn!("Could not parse GitHub URL: {}", repo_url);
                return Ok(vec![]);
            }
        };

        debug!(
            "Analyzing GitHub releases for {}/{}: {} -> {}",
            owner, repo, from_version, to_version
        );

        // Fetch releases
        let releases = match self
            .client
            .repos(&owner, &repo)
            .releases()
            .list()
            .per_page(100)
            .send()
            .await
        {
            Ok(releases) => releases,
            Err(e) => {
                warn!("Failed to fetch releases: {}", e);
                return Ok(vec![]);
            }
        };

        // Find releases between from_version and to_version
        let mut breaking_changes = Vec::new();
        let mut in_range = false;

        for release in releases {
            let tag = release.tag_name.trim_start_matches('v');

            // Check if this is the to_version
            if tag == to_version {
                in_range = true;
            }

            // Extract breaking changes from this release
            if in_range && tag != from_version {
                if let Some(body) = &release.body {
                    breaking_changes.extend(self.extract_breaking_changes(tag, body));
                }
            }

            // Stop when we reach from_version
            if tag == from_version {
                break;
            }
        }

        Ok(breaking_changes)
    }

    /// Extract breaking changes from release notes
    fn extract_breaking_changes(&self, version: &str, body: &str) -> Vec<BreakingChange> {
        let mut changes = Vec::new();

        // Patterns for common breaking change markers
        let patterns = vec![
            Regex::new(r"(?im)^#+\s*breaking\s+change[s]?:?\s*$").unwrap(),
            Regex::new(r"(?im)^#+\s*breaking\s*$").unwrap(),
            Regex::new(r"(?i)breaking change[s]?:").unwrap(),
            Regex::new(r"(?i)\*\*breaking\*\*:?\s*(.+)").unwrap(),
            Regex::new(r"(?i)[⚠️💥]\s*(.+)").unwrap(),
        };

        // Split body into lines
        let lines: Vec<&str> = body.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Check if this line starts a breaking changes section
            if patterns[0].is_match(line) || patterns[1].is_match(line) {
                // Next lines until next header are breaking changes
                i += 1;
                while i < lines.len() {
                    let content = lines[i].trim();

                    // Stop at next markdown header
                    if content.starts_with('#') {
                        break;
                    }

                    // Extract bullet points or numbered items
                    if content.starts_with('-') || content.starts_with('*') || content.starts_with(char::is_numeric) {
                        let description = content
                            .trim_start_matches('-')
                            .trim_start_matches('*')
                            .trim_start_matches(char::is_numeric)
                            .trim_start_matches('.')
                            .trim()
                            .to_string();

                        if !description.is_empty() {
                            changes.push(BreakingChange {
                                description,
                                version: version.to_string(),
                                auto_fixable: false,
                                affected_apis: vec![],
                                migration_hint: None,
                            });
                        }
                    }

                    i += 1;
                }
            } else {
                // Check for inline breaking change markers
                for pattern in &patterns[2..] {
                    if let Some(caps) = pattern.captures(line) {
                        if caps.len() > 1 {
                            let description = caps[1].trim().to_string();
                            if !description.is_empty() {
                                changes.push(BreakingChange {
                                    description,
                                    version: version.to_string(),
                                    auto_fixable: false,
                                    affected_apis: vec![],
                                    migration_hint: None,
                                });
                            }
                        }
                    }
                }
                i += 1;
            }
        }

        changes
    }

    /// Find migration guide URL
    pub async fn find_migration_guide(
        &self,
        repo_url: &str,
        version: &str,
    ) -> Result<Option<String>> {
        let (owner, repo) = match self.parse_repo_url(repo_url) {
            Some(parsed) => parsed,
            None => return Ok(None),
        };

        // Common migration guide paths
        let paths = vec![
            "MIGRATION.md",
            "UPGRADING.md",
            "docs/migration.md",
            "docs/upgrading.md",
            &format!("docs/migration/{}.md", version),
        ];

        for path in paths {
            match self
                .client
                .repos(&owner, &repo)
                .get_content()
                .path(path)
                .send()
                .await
            {
                Ok(_) => {
                    return Ok(Some(format!(
                        "https://github.com/{}/{}/blob/main/{}",
                        owner, repo, path
                    )));
                }
                Err(_) => continue,
            }
        }

        Ok(None)
    }
}

impl Default for GitHubAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHub analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_repo_url() {
        let analyzer = GitHubAnalyzer::default();

        assert_eq!(
            analyzer.parse_repo_url("https://github.com/apache/logging-log4j2"),
            Some(("apache".to_string(), "logging-log4j2".to_string()))
        );

        assert_eq!(
            analyzer.parse_repo_url("git@github.com:apache/logging-log4j2.git"),
            Some(("apache".to_string(), "logging-log4j2".to_string()))
        );
    }

    #[test]
    fn test_extract_breaking_changes() {
        let analyzer = GitHubAnalyzer::default();

        let body = r#"
# Breaking Changes

- Removed deprecated API X
- Changed signature of method Y

# Bug Fixes

- Fixed issue Z
        "#;

        let changes = analyzer.extract_breaking_changes("2.0.0", body);
        assert_eq!(changes.len(), 2);
        assert!(changes[0].description.contains("Removed deprecated API X"));
        assert!(changes[1].description.contains("Changed signature of method Y"));
    }
}
