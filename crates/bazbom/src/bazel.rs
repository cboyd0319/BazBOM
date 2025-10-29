use anyhow::{Context, Result};
use bazbom_formats::spdx::{Package, Relationship, SpdxDocument};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

impl BazelDependencyGraph {
    /// Convert Bazel dependency graph to SPDX document
    pub fn to_spdx(&self, project_name: &str) -> SpdxDocument {
        let namespace = format!(
            "https://github.com/cboyd0319/BazBOM/sbom/bazel/{}",
            project_name
        );
        let mut doc = SpdxDocument::new(format!("{}-sbom", project_name), namespace);

        // Create a map from coordinates to SPDX IDs
        let mut coord_to_id: HashMap<String, String> = HashMap::new();

        // Add packages
        for (idx, component) in self.components.iter().enumerate() {
            let spdx_id = format!("Package-{}", idx);
            coord_to_id.insert(component.coordinates.clone(), spdx_id.clone());

            let mut package = Package::new(&spdx_id, &component.name)
                .with_version(&component.version)
                .with_purl(&component.purl);

            // Set download location if repository is available
            if !component.repository.is_empty() {
                package.download_location = component.repository.clone();
            }

            doc.add_package(package);

            // Add relationship from document to package
            doc.add_relationship(Relationship {
                spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
                relationship_type: "DESCRIBES".to_string(),
                related_spdx_element: format!("SPDXRef-{}", spdx_id),
            });
        }

        // Add dependency relationships
        for edge in &self.edges {
            if let (Some(from_id), Some(to_id)) = (
                coord_to_id.get(&edge.from),
                coord_to_id.get(&edge.to),
            ) {
                doc.add_relationship(Relationship {
                    spdx_element_id: format!("SPDXRef-{}", from_id),
                    relationship_type: "DEPENDS_ON".to_string(),
                    related_spdx_element: format!("SPDXRef-{}", to_id),
                });
            }
        }

        doc
    }
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

/// Query Bazel targets using the bazel_query.py script
pub fn query_bazel_targets(
    workspace_path: &Path,
    query_expr: Option<&str>,
    kind: Option<&str>,
    affected_by_files: Option<&[String]>,
    universe: &str,
) -> Result<Vec<String>> {
    let script_path = workspace_path.join("tools/supplychain/bazel_query.py");
    
    if !script_path.exists() {
        anyhow::bail!(
            "Bazel query script not found at {:?}. This workspace may not support BazBOM.",
            script_path
        );
    }

    let mut cmd = Command::new("python3");
    cmd.arg(&script_path)
        .arg("--workspace")
        .arg(workspace_path)
        .arg("--universe")
        .arg(universe)
        .arg("--format")
        .arg("json");

    // Add query type based on what was provided
    if let Some(query) = query_expr {
        cmd.arg("--query").arg(query);
    } else if let Some(target_kind) = kind {
        cmd.arg("--kind").arg(target_kind);
    } else if let Some(files) = affected_by_files {
        cmd.arg("--affected-by-files");
        for file in files {
            cmd.arg(file);
        }
    } else {
        anyhow::bail!("Must provide either query, kind, or affected_by_files");
    }

    println!("[bazbom] executing Bazel query...");
    let output = cmd
        .output()
        .context("failed to execute bazel_query.py")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bazel query failed: {}", stderr);
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let result: serde_json::Value = serde_json::from_str(&stdout)
        .context("failed to parse query results")?;
    
    let targets = result["targets"]
        .as_array()
        .context("query result missing 'targets' array")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    Ok(targets)
}

/// Extract dependencies for specific Bazel targets
pub fn extract_bazel_dependencies_for_targets(
    workspace_path: &Path,
    targets: &[String],
    output_path: &Path,
) -> Result<BazelDependencyGraph> {
    // For now, we still use maven_install.json for all targets
    // In the future, we could filter the graph to only include
    // dependencies actually used by the specified targets
    
    println!(
        "[bazbom] extracting dependencies for {} targets",
        targets.len()
    );
    for target in targets {
        println!("  - {}", target);
    }
    
    // Use the existing extraction function
    extract_bazel_dependencies(workspace_path, output_path)
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

    #[test]
    fn test_to_spdx() {
        let graph = BazelDependencyGraph {
            components: vec![
                BazelComponent {
                    name: "guava".to_string(),
                    group: "com.google.guava".to_string(),
                    version: "31.1-jre".to_string(),
                    purl: "pkg:maven/com/google/guava/guava@31.1-jre".to_string(),
                    component_type: "maven".to_string(),
                    scope: "compile".to_string(),
                    sha256: "abc123".to_string(),
                    repository: "https://repo1.maven.org/maven2/".to_string(),
                    coordinates: "com.google.guava:guava:31.1-jre".to_string(),
                },
                BazelComponent {
                    name: "jsr305".to_string(),
                    group: "com.google.code.findbugs".to_string(),
                    version: "3.0.2".to_string(),
                    purl: "pkg:maven/com/google/code/findbugs/jsr305@3.0.2".to_string(),
                    component_type: "maven".to_string(),
                    scope: "compile".to_string(),
                    sha256: "def456".to_string(),
                    repository: "https://repo1.maven.org/maven2/".to_string(),
                    coordinates: "com.google.code.findbugs:jsr305:3.0.2".to_string(),
                },
            ],
            edges: vec![BazelEdge {
                from: "com.google.guava:guava:31.1-jre".to_string(),
                to: "com.google.code.findbugs:jsr305:3.0.2".to_string(),
                edge_type: "depends_on".to_string(),
            }],
            metadata: BazelMetadata {
                build_system: "bazel".to_string(),
                workspace: "/test".to_string(),
                maven_install_version: "2".to_string(),
            },
        };

        let spdx = graph.to_spdx("test-project");
        
        // Verify document structure
        assert_eq!(spdx.name, "test-project-sbom");
        assert!(spdx.document_namespace.contains("test-project"));
        
        // Verify packages
        assert_eq!(spdx.packages.len(), 2);
        assert_eq!(spdx.packages[0].name, "guava");
        assert_eq!(spdx.packages[0].version_info, Some("31.1-jre".to_string()));
        assert_eq!(spdx.packages[1].name, "jsr305");
        
        // Verify relationships (DESCRIBES + DEPENDS_ON)
        assert_eq!(spdx.relationships.len(), 3); // 2 DESCRIBES + 1 DEPENDS_ON
        
        // Check for DEPENDS_ON relationship
        let depends_on = spdx
            .relationships
            .iter()
            .find(|r| r.relationship_type == "DEPENDS_ON");
        assert!(depends_on.is_some());
    }
}
