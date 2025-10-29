use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Serialize, Deserialize)]
pub struct BazelComponent {
    pub name: String,
    pub group: String,
    pub version: String,
    pub purl: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub scope: String,
    pub sha256: String,
    pub repository: String,
    pub coordinates: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BazelEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type")]
    pub edge_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BazelMetadata {
    pub build_system: String,
    pub workspace: String,
    pub maven_install_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BazelDependencyGraph {
    pub components: Vec<BazelComponent>,
    pub edges: Vec<BazelEdge>,
    pub metadata: BazelMetadata,
}

/// Extract dependencies from Bazel project using maven_install.json
pub fn extract_bazel_dependencies(
    workspace_path: &Path,
    output_path: &Path,
) -> Result<BazelDependencyGraph> {
    // Find the bazbom_extract_bazel_deps.py script
    // It should be in tools/supplychain/ relative to the workspace
    let script_path = workspace_path.join("tools/supplychain/bazbom_extract_bazel_deps.py");
    
    if !script_path.exists() {
        anyhow::bail!(
            "Bazel extraction script not found at {:?}. This workspace may not support BazBOM.",
            script_path
        );
    }

    // Look for maven_install.json in the workspace
    let maven_install_json = workspace_path.join("maven_install.json");
    if !maven_install_json.exists() {
        anyhow::bail!(
            "maven_install.json not found at {:?}. Run 'bazel run @maven//:pin' to generate it.",
            maven_install_json
        );
    }

    println!(
        "[bazbom] extracting Bazel dependencies from {:?}",
        maven_install_json
    );

    // Run the extraction script
    let status = Command::new("python3")
        .arg(&script_path)
        .arg("--workspace")
        .arg(workspace_path)
        .arg("--maven-install-json")
        .arg(&maven_install_json)
        .arg("--output")
        .arg(output_path)
        .status()
        .context("failed to execute bazbom_extract_bazel_deps.py")?;

    if !status.success() {
        anyhow::bail!("Bazel dependency extraction failed");
    }

    // Parse the output JSON
    let json_content =
        std::fs::read_to_string(output_path).context("failed to read extraction output")?;

    let graph: BazelDependencyGraph =
        serde_json::from_str(&json_content).context("failed to parse dependency graph")?;

    println!(
        "[bazbom] extracted {} components and {} edges",
        graph.components.len(),
        graph.edges.len()
    );

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_deserialization() {
        let json = r#"{
            "name": "guava",
            "group": "com.google.guava",
            "version": "31.1-jre",
            "purl": "pkg:maven/com/google/guava/guava@31.1-jre",
            "type": "maven",
            "scope": "compile",
            "sha256": "a42edc9cab792e39fe39bb94f3fca655ed157ff87a8af78e1d6ba5b07c4a00ab",
            "repository": "https://repo1.maven.org/maven2/",
            "coordinates": "com.google.guava:guava:31.1-jre"
        }"#;

        let component: BazelComponent = serde_json::from_str(json).unwrap();
        assert_eq!(component.name, "guava");
        assert_eq!(component.group, "com.google.guava");
        assert_eq!(component.version, "31.1-jre");
    }

    #[test]
    fn test_graph_deserialization() {
        let json = r#"{
            "components": [],
            "edges": [],
            "metadata": {
                "build_system": "bazel",
                "workspace": "/path/to/workspace",
                "maven_install_version": "2"
            }
        }"#;

        let graph: BazelDependencyGraph = serde_json::from_str(json).unwrap();
        assert_eq!(graph.metadata.build_system, "bazel");
        assert_eq!(graph.components.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }
}
