use anyhow::Result;
use std::path::Path;
use serde_yaml;
use crate::ecosystems::{Package, EcosystemScanResult};

pub fn detect_github_actions(workspace: &Path) -> Result<EcosystemScanResult> {
    let workflows_dir = workspace.join(".github/workflows");
    if !workflows_dir.exists() {
        return Ok(EcosystemScanResult {
            ecosystem: "GitHub Actions".to_string(),
            root_path: workspace.to_string_lossy().to_string(),
            packages: vec![],
            vulnerabilities: vec![],
            total_packages: 0,
            total_vulnerabilities: 0,
            reachability: None,
        });
    }

    let mut packages = Vec::new();

    // Find all .yml and .yaml files in workflows directory
    for entry in std::fs::read_dir(&workflows_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("yml")
            || path.extension().and_then(|s| s.to_str()) == Some("yaml") {

            let content = std::fs::read_to_string(&path)?;
            let workflow: serde_yaml::Value = serde_yaml::from_str(&content)?;

            // Extract 'uses:' references from jobs
            if let Some(jobs) = workflow.get("jobs").and_then(|j| j.as_mapping()) {
                for (_job_name, job) in jobs {
                    if let Some(steps) = job.get("steps").and_then(|s| s.as_sequence()) {
                        for step in steps {
                            if let Some(uses) = step.get("uses").and_then(|u| u.as_str()) {
                                // Parse: "actions/checkout@v5" -> name: actions/checkout, version: v5
                                if let Some((name, version)) = parse_github_action(uses) {
                                    packages.push(Package {
                                        name: name.to_string(),
                                        version: version.to_string(),
                                        ecosystem: "GitHub Actions".to_string(),
                                        namespace: None,
                                        dependencies: vec![],
                                        license: None,
                                        description: Some(format!("GitHub Action: {}", uses)),
                                        homepage: Some(format!("https://github.com/{}", name)),
                                        repository: Some(format!("https://github.com/{}", name)),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let total_packages = packages.len();
    Ok(EcosystemScanResult {
        ecosystem: "GitHub Actions".to_string(),
        root_path: workspace.to_string_lossy().to_string(),
        packages,
        total_packages,
        vulnerabilities: vec![],
        total_vulnerabilities: 0,
        reachability: None,
    })
}

fn parse_github_action(uses: &str) -> Option<(&str, &str)> {
    // Format: "owner/repo@version" or "owner/repo/path@version"
    // Examples:
    //   "actions/checkout@v5" -> ("actions/checkout", "v5")
    //   "docker://alpine:3.7" -> skip (Docker image, not GitHub Action)
    //   "./local/action" -> skip (local action)

    if uses.starts_with("docker://") || uses.starts_with("./") {
        return None;
    }

    // Split on '@' to get name and version
    let parts: Vec<&str> = uses.split('@').collect();
    if parts.len() != 2 {
        return None;
    }

    let name = parts[0];
    let version = parts[1];

    // Validate it's a GitHub action (contains '/')
    if !name.contains('/') {
        return None;
    }

    Some((name, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_action() {
        assert_eq!(
            parse_github_action("actions/checkout@v5"),
            Some(("actions/checkout", "v5"))
        );

        assert_eq!(
            parse_github_action("EmbarkStudios/cargo-deny-action@v1"),
            Some(("EmbarkStudios/cargo-deny-action", "v1"))
        );

        assert_eq!(parse_github_action("docker://alpine:3.7"), None);
        assert_eq!(parse_github_action("./local/action"), None);
    }
}
