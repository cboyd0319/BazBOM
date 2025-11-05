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
            if let (Some(from_id), Some(to_id)) =
                (coord_to_id.get(&edge.from), coord_to_id.get(&edge.to))
            {
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

    // Parse maven_install.json directly (more reliable than external script)
    let graph = parse_maven_install_json(workspace_path, &maven_install_json)?;

    // Write the graph to output
    let json_content =
        serde_json::to_string_pretty(&graph).context("failed to serialize dependency graph")?;

    // Ensure parent directory exists
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create directory {:?}", parent))?;
    }

    std::fs::write(output_path, json_content)
        .with_context(|| format!("failed to write dependency graph to {:?}", output_path))?;

    println!(
        "[bazbom] extracted {} components and {} edges",
        graph.components.len(),
        graph.edges.len()
    );

    Ok(graph)
}

/// Parse maven_install.json directly to extract dependencies
fn parse_maven_install_json(
    workspace_path: &Path,
    maven_install_json: &Path,
) -> Result<BazelDependencyGraph> {
    use serde_json::Value;

    let content =
        std::fs::read_to_string(maven_install_json).context("failed to read maven_install.json")?;
    let data: Value =
        serde_json::from_str(&content).context("failed to parse maven_install.json")?;

    let mut components = Vec::new();
    let mut edges = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Extract artifacts
    let artifacts = data["artifacts"]
        .as_object()
        .context("maven_install.json missing 'artifacts' object")?;

    let empty_map = serde_json::Map::new();
    let dependencies = data
        .get("dependencies")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);

    let empty_repos = serde_json::Map::new();
    let repositories = data
        .get("repositories")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_repos);

    for (coord, artifact_info) in artifacts {
        if seen.contains(coord) {
            continue;
        }
        seen.insert(coord.clone());

        // Parse coordinate (format: "group:artifact")
        let parts: Vec<&str> = coord.split(':').collect();
        if parts.len() < 2 {
            continue;
        }

        let group = parts[0];
        let artifact = parts[1];

        // Extract version from artifact info
        let version = artifact_info["version"].as_str().unwrap_or("").to_string();

        if version.is_empty() {
            continue;
        }

        // Extract SHA256 from shasums
        let sha256 = artifact_info
            .get("shasums")
            .and_then(|s| s.get("jar"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Create PURL
        let purl = format!(
            "pkg:maven/{}/{}@{}",
            group.replace('.', "/"),
            artifact,
            version
        );

        // Find repository
        let mut repo_url = String::new();
        for (repo, artifacts_list) in repositories {
            if let Some(list) = artifacts_list.as_array() {
                for item in list {
                    if item.as_str() == Some(coord) {
                        repo_url = repo.clone();
                        break;
                    }
                }
            }
            if !repo_url.is_empty() {
                break;
            }
        }

        // Full Maven coordinate
        let full_coord = format!("{}:{}:{}", group, artifact, version);

        let component = BazelComponent {
            name: artifact.to_string(),
            group: group.to_string(),
            version,
            purl,
            component_type: "maven".to_string(),
            scope: "compile".to_string(),
            sha256,
            repository: repo_url,
            coordinates: full_coord.clone(),
        };
        components.push(component);

        // Process dependencies
        let deps = dependencies
            .get(&full_coord)
            .or_else(|| dependencies.get(coord))
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        for dep_coord in deps {
            // dep_coord is in format "group:artifact", need to find the full coordinate
            if let Some(dep_info) = artifacts.get(dep_coord) {
                if let Some(dep_version) = dep_info["version"].as_str() {
                    let full_dep_coord = format!("{}:{}", dep_coord, dep_version);
                    edges.push(BazelEdge {
                        from: full_coord.clone(),
                        to: full_dep_coord,
                        edge_type: "depends_on".to_string(),
                    });
                }
            }
        }
    }

    let metadata = BazelMetadata {
        build_system: "bazel".to_string(),
        workspace: workspace_path.to_string_lossy().to_string(),
        maven_install_version: data["version"].as_str().unwrap_or("unknown").to_string(),
    };

    Ok(BazelDependencyGraph {
        components,
        edges,
        metadata,
    })
}

/// Query Bazel targets directly using bazel query command (Rust-native, no Python)
pub fn query_bazel_targets(
    workspace_path: &Path,
    query_expr: Option<&str>,
    kind: Option<&str>,
    affected_by_files: Option<&[String]>,
    universe: &str,
) -> Result<Vec<String>> {
    // Build the query expression based on input parameters
    let query = if let Some(q) = query_expr {
        // Use provided query directly
        q.to_string()
    } else if let Some(target_kind) = kind {
        // Generate kind query
        format!("kind({}, {})", target_kind, universe)
    } else if let Some(files) = affected_by_files {
        // Generate rdeps query for affected files
        if files.is_empty() {
            anyhow::bail!("affected_by_files cannot be empty");
        }
        // Escape file paths to prevent shell injection
        // Validate against comprehensive list of shell metacharacters
        let is_safe_path = |path: &str| -> bool {
            let dangerous_chars = [
                '\'', '"', '$', '`', ';', '&', '|', '>', '<', '(', ')', '{', '}', '[', ']', '*',
                '?', '~', '\n', '\r',
            ];
            !path.chars().any(|c| dangerous_chars.contains(&c))
        };

        let file_set = files
            .iter()
            .filter_map(|f| {
                // Validate file path doesn't contain dangerous characters
                if is_safe_path(f) {
                    // Escape for Bazel query (quotes are sufficient as we validated above)
                    Some(format!("\"{}\"", f))
                } else {
                    eprintln!(
                        "[bazbom] warning: skipping file with shell metacharacters: {}",
                        f
                    );
                    None
                }
            })
            .collect::<Vec<_>>()
            .join(", ");

        if file_set.is_empty() {
            anyhow::bail!("No valid files after filtering shell metacharacters");
        }
        format!("rdeps({}, set({}))", universe, file_set)
    } else {
        anyhow::bail!("Must provide either query, kind, or affected_by_files");
    };

    println!("[bazbom] executing Bazel query: {}", query);

    let mut cmd = Command::new("bazel");
    cmd.arg("query")
        .arg(&query)
        .arg("--output=label")
        .current_dir(workspace_path);

    let output = cmd.output().context("failed to execute bazel query")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Bazel query failed: {}", stderr);
    }

    // Parse output - one target per line
    let stdout = String::from_utf8_lossy(&output.stdout);
    let targets: Vec<String> = stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect();

    println!("[bazbom] found {} matching targets", targets.len());
    Ok(targets)
}

/// Extract dependencies for specific Bazel targets
pub fn extract_bazel_dependencies_for_targets(
    workspace_path: &Path,
    targets: &[String],
    output_path: &Path,
) -> Result<BazelDependencyGraph> {
    println!(
        "[bazbom] extracting dependencies for {} targets",
        targets.len()
    );
    for target in targets {
        println!("  - {}", target);
    }

    // First get all dependencies from maven_install.json
    let full_graph = extract_bazel_dependencies(workspace_path, output_path)?;

    // If no specific targets, return full graph
    if targets.is_empty() {
        return Ok(full_graph);
    }

    // Try to filter graph based on targets using Bazel query
    // This requires running `bazel query --output=proto` for each target
    // For now, return the full graph as filtering requires more complex query logic
    // TODO: Implement target-specific filtering using bazel cquery

    println!(
        "[bazbom] note: returning full dependency graph (target-specific filtering not yet implemented)"
    );

    Ok(full_graph)
}

/// Optimized Bazel query execution with caching and batching
#[allow(dead_code)]
pub struct BazelQueryOptimizer {
    workspace_path: std::path::PathBuf,
    cache: std::collections::HashMap<String, Vec<String>>,
    metrics: BazelPerformanceMetrics,
}

#[allow(dead_code)]
impl BazelQueryOptimizer {
    /// Create a new Bazel query optimizer
    pub fn new(workspace_path: std::path::PathBuf) -> Self {
        Self {
            workspace_path,
            cache: std::collections::HashMap::new(),
            metrics: BazelPerformanceMetrics::new(),
        }
    }

    /// Execute an optimized query with caching
    pub fn query(&mut self, query_expr: &str) -> Result<Vec<String>> {
        use std::time::Instant;
        let start = Instant::now();

        self.metrics.query_count += 1;

        // Check cache first
        if let Some(cached) = self.cache.get(query_expr) {
            println!("[bazbom] using cached query result");
            self.metrics.cache_hits += 1;
            return Ok(cached.clone());
        }

        self.metrics.cache_misses += 1;

        // Execute query
        let result =
            query_bazel_targets(&self.workspace_path, Some(query_expr), None, None, "//...")?;

        // Update metrics
        self.metrics.query_time_ms += start.elapsed().as_millis() as u64;
        self.metrics.total_targets += result.len();

        // Cache the result
        self.cache.insert(query_expr.to_string(), result.clone());

        Ok(result)
    }

    /// Batch multiple queries for efficiency
    pub fn batch_query(&mut self, queries: &[String]) -> Result<Vec<Vec<String>>> {
        let mut results = Vec::new();

        for query in queries {
            let result = self.query(query)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Query dependencies with optimized filtering
    pub fn query_deps(&mut self, target: &str, max_depth: Option<usize>) -> Result<Vec<String>> {
        let depth_suffix = if let Some(d) = max_depth {
            format!(", {}", d)
        } else {
            String::new()
        };

        let query = format!("deps({}{})", target, depth_suffix);
        self.query(&query)
    }

    /// Query reverse dependencies efficiently
    pub fn query_rdeps(&mut self, target: &str, universe: &str) -> Result<Vec<String>> {
        let query = format!("rdeps({}, {})", universe, target);
        self.query(&query)
    }

    /// Clear query cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.metrics = BazelPerformanceMetrics::new();
    }

    /// Get current performance metrics
    pub fn metrics(&self) -> &BazelPerformanceMetrics {
        &self.metrics
    }

    /// Print performance summary
    pub fn print_metrics(&self) {
        println!("\n[bazbom] Bazel Query Performance Metrics:");
        println!("  Total queries: {}", self.metrics.query_count);
        println!(
            "  Cache hits: {} ({:.1}% hit rate)",
            self.metrics.cache_hits,
            self.metrics.cache_hit_rate()
        );
        println!("  Cache misses: {}", self.metrics.cache_misses);
        println!("  Total targets: {}", self.metrics.total_targets);
        println!("  Total query time: {}ms", self.metrics.query_time_ms);
        if self.metrics.query_count > 0 {
            println!(
                "  Avg query time: {}ms",
                self.metrics.query_time_ms / self.metrics.query_count as u64
            );
        }
    }
}

/// Performance metrics for Bazel operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BazelPerformanceMetrics {
    pub query_count: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
    pub total_targets: usize,
    pub query_time_ms: u64,
}

impl BazelPerformanceMetrics {
    pub fn new() -> Self {
        Self {
            query_count: 0,
            cache_hits: 0,
            cache_misses: 0,
            total_targets: 0,
            query_time_ms: 0,
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        if self.query_count == 0 {
            0.0
        } else {
            (self.cache_hits as f64) / (self.query_count as f64) * 100.0
        }
    }
}

impl Default for BazelPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a Bazel rule is a JVM-related rule (Java, Kotlin, or Scala)
#[allow(dead_code)]
pub fn is_jvm_rule(rule_kind: &str) -> bool {
    let jvm_rules = [
        // Java rules
        "java_library",
        "java_binary",
        "java_test",
        "java_plugin",
        "java_import",
        // Kotlin rules (rules_kotlin)
        "kotlin_library",
        "kotlin_jvm_library",
        "kt_jvm_library",
        "kt_jvm_binary",
        "kt_jvm_test",
        "kt_jvm_import",
        // Scala rules (rules_scala)
        "scala_library",
        "scala_binary",
        "scala_test",
        "scala_import",
        "scala_macro_library",
    ];

    jvm_rules.contains(&rule_kind)
}

/// Get a Bazel query expression for all JVM rules in the workspace
#[allow(dead_code)]
pub fn get_jvm_rule_query(universe: &str) -> String {
    let rule_kinds = [
        "java_library",
        "java_binary",
        "java_test",
        "kotlin_library",
        "kt_jvm_library",
        "kt_jvm_binary",
        "kt_jvm_test",
        "scala_library",
        "scala_binary",
        "scala_test",
    ];

    let kind_queries: Vec<String> = rule_kinds
        .iter()
        .map(|kind| format!("kind({}, {})", kind, universe))
        .collect();

    // Combine all queries with union operator
    kind_queries.join(" + ")
}

/// Query all JVM targets in the workspace (Java, Kotlin, Scala)
#[allow(dead_code)]
pub fn query_all_jvm_targets(workspace_path: &Path) -> Result<Vec<String>> {
    let query = get_jvm_rule_query("//...");
    println!("[bazbom] querying all JVM targets (Java, Kotlin, Scala)");

    query_bazel_targets(workspace_path, Some(&query), None, None, "//...")
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

    #[test]
    fn test_parse_maven_install_json_structure() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        // Create a minimal maven_install.json
        let json_content = r#"{
            "version": "2",
            "artifacts": {
                "com.google.guava:guava": {
                    "version": "31.1-jre",
                    "shasums": {
                        "jar": "abc123"
                    }
                }
            },
            "dependencies": {},
            "repositories": {
                "https://repo1.maven.org/maven2": ["com.google.guava:guava"]
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let workspace = std::env::temp_dir();
        let result = super::parse_maven_install_json(&workspace, temp_file.path());

        assert!(result.is_ok());
        let graph = result.unwrap();

        assert_eq!(graph.components.len(), 1);
        assert_eq!(graph.components[0].name, "guava");
        assert_eq!(graph.components[0].group, "com.google.guava");
        assert_eq!(graph.components[0].version, "31.1-jre");
        assert_eq!(graph.components[0].sha256, "abc123");
    }

    #[test]
    fn test_parse_maven_install_json_with_dependencies() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let json_content = r#"{
            "version": "2",
            "artifacts": {
                "com.google.guava:guava": {
                    "version": "31.1-jre",
                    "shasums": {"jar": "abc123"}
                },
                "com.google.code.findbugs:jsr305": {
                    "version": "3.0.2",
                    "shasums": {"jar": "def456"}
                }
            },
            "dependencies": {
                "com.google.guava:guava:31.1-jre": ["com.google.code.findbugs:jsr305"]
            },
            "repositories": {}
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(json_content.as_bytes()).unwrap();
        temp_file.flush().unwrap();

        let workspace = std::env::temp_dir();
        let result = super::parse_maven_install_json(&workspace, temp_file.path());

        assert!(result.is_ok());
        let graph = result.unwrap();

        assert_eq!(graph.components.len(), 2);
        assert_eq!(graph.edges.len(), 1);

        let edge = &graph.edges[0];
        assert_eq!(edge.from, "com.google.guava:guava:31.1-jre");
        assert_eq!(edge.to, "com.google.code.findbugs:jsr305:3.0.2");
        assert_eq!(edge.edge_type, "depends_on");
    }

    #[test]
    fn test_bazel_performance_metrics_new() {
        let metrics = BazelPerformanceMetrics::new();
        assert_eq!(metrics.query_count, 0);
        assert_eq!(metrics.cache_hits, 0);
        assert_eq!(metrics.cache_misses, 0);
        assert_eq!(metrics.total_targets, 0);
        assert_eq!(metrics.query_time_ms, 0);
    }

    #[test]
    fn test_bazel_performance_metrics_cache_hit_rate() {
        let mut metrics = BazelPerformanceMetrics::new();

        // No queries yet
        assert_eq!(metrics.cache_hit_rate(), 0.0);

        // 50% hit rate
        metrics.query_count = 10;
        metrics.cache_hits = 5;
        metrics.cache_misses = 5;
        assert_eq!(metrics.cache_hit_rate(), 50.0);

        // 100% hit rate
        metrics.cache_hits = 10;
        metrics.cache_misses = 0;
        assert_eq!(metrics.cache_hit_rate(), 100.0);

        // 0% hit rate
        metrics.cache_hits = 0;
        metrics.cache_misses = 10;
        assert_eq!(metrics.cache_hit_rate(), 0.0);
    }

    #[test]
    fn test_bazel_query_optimizer_creation() {
        let workspace = std::path::PathBuf::from("/tmp/test");
        let optimizer = BazelQueryOptimizer::new(workspace.clone());

        assert_eq!(optimizer.workspace_path, workspace);
        assert_eq!(optimizer.cache.len(), 0);
        assert_eq!(optimizer.metrics.query_count, 0);
    }

    #[test]
    fn test_bazel_query_optimizer_clear_cache() {
        let workspace = std::path::PathBuf::from("/tmp/test");
        let mut optimizer = BazelQueryOptimizer::new(workspace);

        // Manually add some cache entries to simulate usage
        optimizer
            .cache
            .insert("test_query".to_string(), vec!["//target1".to_string()]);
        optimizer.metrics.query_count = 5;
        optimizer.metrics.cache_hits = 3;

        // Clear cache
        optimizer.clear_cache();

        // Verify everything is reset
        assert_eq!(optimizer.cache.len(), 0);
        assert_eq!(optimizer.metrics.query_count, 0);
        assert_eq!(optimizer.metrics.cache_hits, 0);
    }

    #[test]
    fn test_bazel_query_optimizer_metrics_access() {
        let workspace = std::path::PathBuf::from("/tmp/test");
        let optimizer = BazelQueryOptimizer::new(workspace);

        let metrics = optimizer.metrics();
        assert_eq!(metrics.query_count, 0);
        assert_eq!(metrics.cache_hits, 0);
    }

    #[test]
    fn test_jvm_rule_detection() {
        assert!(is_jvm_rule("java_library"));
        assert!(is_jvm_rule("java_binary"));
        assert!(is_jvm_rule("kotlin_library"));
        assert!(is_jvm_rule("kt_jvm_library"));
        assert!(is_jvm_rule("scala_library"));
        assert!(is_jvm_rule("scala_binary"));
        assert!(!is_jvm_rule("py_binary"));
        assert!(!is_jvm_rule("go_library"));
    }

    #[test]
    fn test_get_jvm_rule_query() {
        let query = get_jvm_rule_query("//...");
        assert!(query.contains("java_"));
        assert!(query.contains("kotlin_"));
        assert!(query.contains("scala_"));
    }
}
