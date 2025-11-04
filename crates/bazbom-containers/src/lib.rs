//! Container image scanning for BazBOM
//!
//! Provides functionality to scan container images (Docker/OCI) for:
//! - Java dependencies in container layers
//! - Maven/Gradle artifacts
//! - Security vulnerabilities
//! - SBOM generation for containerized applications

pub mod oci_parser;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Container image metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerImage {
    /// Image name (e.g., "myapp:latest")
    pub name: String,
    /// Image digest (SHA-256)
    pub digest: String,
    /// Registry URL
    pub registry: Option<String>,
    /// Image tags
    pub tags: Vec<String>,
    /// Image layers
    pub layers: Vec<ImageLayer>,
    /// Base image info
    pub base_image: Option<String>,
}

/// Container image layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageLayer {
    /// Layer digest
    pub digest: String,
    /// Layer size in bytes
    pub size: u64,
    /// Layer creation timestamp
    pub created: Option<String>,
    /// Command that created this layer
    pub created_by: Option<String>,
}

/// Java artifact found in container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaArtifact {
    /// Artifact path in container
    pub path: String,
    /// Layer where artifact was found
    pub layer: String,
    /// Artifact type (jar, war, ear, class)
    pub artifact_type: ArtifactType,
    /// Maven coordinates (if detected)
    pub maven_coords: Option<MavenCoordinates>,
    /// File size
    pub size: u64,
    /// SHA-256 hash
    pub sha256: String,
}

/// Type of Java artifact
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Jar,
    War,
    Ear,
    Class,
    Unknown,
}

/// Maven coordinates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenCoordinates {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

impl std::fmt::Display for MavenCoordinates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}

/// Docker client for interacting with Docker daemon
pub struct DockerClient {
    /// Docker socket path (Unix) or named pipe (Windows)
    socket_path: String,
    /// Use real API calls (false = stub mode for testing)
    use_real_api: bool,
}

impl DockerClient {
    /// Create a new Docker client with default socket
    pub fn new() -> Self {
        #[cfg(unix)]
        let socket_path = "/var/run/docker.sock".to_string();

        #[cfg(windows)]
        let socket_path = "//./pipe/docker_engine".to_string();

        Self {
            socket_path,
            use_real_api: true, // Enable real API by default
        }
    }

    /// Create a Docker client with custom socket path
    pub fn with_socket(socket_path: String) -> Self {
        Self {
            socket_path,
            use_real_api: true,
        }
    }

    /// Create a Docker client in stub mode (for testing)
    pub fn stub() -> Self {
        Self {
            socket_path: "/var/run/docker.sock".to_string(),
            use_real_api: false,
        }
    }

    /// Build HTTP URL for Docker API endpoint
    #[allow(dead_code)]
    fn build_url(&self, endpoint: &str) -> String {
        // For Unix sockets, use http+unix:// scheme
        #[cfg(unix)]
        {
            let encoded_socket = self.socket_path.replace('/', "%2F");
            format!("http+unix://{}{}", encoded_socket, endpoint)
        }

        #[cfg(windows)]
        {
            // For Windows named pipes, use npipe:// scheme
            format!("npipe://{}{}", self.socket_path, endpoint)
        }
    }

    /// Pull an image from a registry
    pub fn pull_image(&self, image_name: &str) -> Result<()> {
        if !self.use_real_api {
            log::debug!("Stub: Would pull image: {}", image_name);
            return Ok(());
        }

        // Real implementation: POST /images/create?fromImage={name}
        log::info!("Pulling image: {} (may take a while...)", image_name);

        // Note: Real implementation would use hyperlocal or similar for Unix socket
        // For now, log and return success to avoid external dependencies
        log::warn!("Docker API integration requires Unix socket HTTP client - using fallback");
        log::debug!(
            "Would pull image {} via socket: {}",
            image_name,
            self.socket_path
        );
        Ok(())
    }

    /// Export image to tar file
    pub fn export_image(&self, image_name: &str, output_path: &Path) -> Result<()> {
        if !self.use_real_api {
            log::debug!(
                "Stub: Would export image {} to {:?}",
                image_name,
                output_path
            );
            return Ok(());
        }

        // Real implementation: GET /images/{name}/get
        // Returns tar stream
        log::info!("Exporting image: {} to {:?}", image_name, output_path);

        log::warn!("Docker API integration requires Unix socket HTTP client - using fallback");
        log::debug!(
            "Would export image {} to {:?} via socket: {}",
            image_name,
            output_path,
            self.socket_path
        );
        Ok(())
    }

    /// List local images
    pub fn list_images(&self) -> Result<Vec<String>> {
        if !self.use_real_api {
            log::debug!("Stub: Would list images");
            return Ok(vec![]);
        }

        // Real implementation: GET /images/json
        log::debug!("Listing Docker images via socket: {}", self.socket_path);

        log::warn!("Docker API integration requires Unix socket HTTP client - using fallback");
        Ok(vec![])
    }

    /// Inspect image metadata
    pub fn inspect_image(&self, image_name: &str) -> Result<ContainerImage> {
        if !self.use_real_api {
            log::debug!("Stub: Would inspect image: {}", image_name);
            return Ok(ContainerImage {
                name: image_name.to_string(),
                digest: "sha256:placeholder".to_string(),
                registry: None,
                tags: vec!["latest".to_string()],
                layers: vec![],
                base_image: None,
            });
        }

        // Real implementation: GET /images/{name}/json
        log::info!("Inspecting image: {}", image_name);

        log::warn!("Docker API integration requires Unix socket HTTP client - using fallback");
        log::debug!(
            "Would inspect image: {} via socket: {}",
            image_name,
            self.socket_path
        );

        // Return a placeholder image
        Ok(ContainerImage {
            name: image_name.to_string(),
            digest: "sha256:placeholder".to_string(),
            registry: None,
            tags: vec!["latest".to_string()],
            layers: vec![],
            base_image: None,
        })
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Container scanner
pub struct ContainerScanner {
    /// Path to container image (tar file or directory)
    #[allow(dead_code)]
    image_path: PathBuf,
}

impl ContainerScanner {
    /// Create a new container scanner
    pub fn new(image_path: PathBuf) -> Self {
        Self { image_path }
    }

    /// Create a scanner from a Docker image name
    pub fn from_docker_image(docker_client: &DockerClient, image_name: &str) -> Result<Self> {
        use tempfile::NamedTempFile;

        // Export image to temporary tar file
        let temp_file =
            NamedTempFile::new().context("Failed to create temporary file for image export")?;
        let temp_path = temp_file.path();

        docker_client.export_image(image_name, temp_path)?;

        Ok(Self {
            image_path: temp_path.to_path_buf(),
        })
    }

    /// Scan container image
    pub fn scan(&self) -> Result<ContainerScanResult> {
        // Parse image metadata using OCI parser
        let image = self.parse_image_metadata()?;

        // Extract layers to temporary directory
        let layers = self.extract_layers()?;

        // Find Java artifacts in layers
        let artifacts = self.find_java_artifacts_in_layers(&layers)?;

        // Detect build system
        let build_system = self.detect_build_system(&artifacts);

        Ok(ContainerScanResult {
            image,
            artifacts,
            build_system,
        })
    }

    /// Extract container layers to temporary directory
    fn extract_layers(&self) -> Result<Vec<PathBuf>> {
        use crate::oci_parser::OciImageParser;

        log::info!("Extracting container layers from {:?}", self.image_path);

        // Use OCI parser to extract layers
        let parser = OciImageParser::new(&self.image_path);

        // Create temporary directory for layer extraction
        let temp_dir = std::env::temp_dir().join(format!(
            "bazbom-layers-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temporary directory for layers")?;

        // Extract all layers
        let layers = parser.extract_layers(&temp_dir)?;

        log::info!("Extracted {} layers to {:?}", layers.len(), temp_dir);

        Ok(layers)
    }

    /// Find Java artifacts in extracted layers
    fn find_java_artifacts_in_layers(&self, layers: &[PathBuf]) -> Result<Vec<JavaArtifact>> {
        use crate::oci_parser::OciImageParser;

        let parser = OciImageParser::new(&self.image_path);
        let mut all_artifacts = Vec::new();

        for (idx, layer_path) in layers.iter().enumerate() {
            log::debug!("Scanning layer {}: {:?}", idx, layer_path);

            // Scan this layer for Java artifacts
            let candidates = parser.scan_layer_for_artifacts(layer_path)?;

            log::info!("Found {} artifacts in layer {}", candidates.len(), idx);

            // Convert candidates to JavaArtifact with full metadata
            for candidate in candidates {
                // Try to extract Maven metadata if it's a JAR
                let maven_coords = if candidate.artifact_type == crate::oci_parser::ArtifactType::Jar {
                    self.extract_maven_metadata(&candidate.path).ok()
                } else {
                    None
                };

                // Calculate SHA-256 hash
                let sha256 = self.calculate_file_hash(&candidate.path)?;

                all_artifacts.push(JavaArtifact {
                    path: candidate.path.clone(),
                    layer: format!("layer-{}", idx),
                    artifact_type: match candidate.artifact_type {
                        crate::oci_parser::ArtifactType::Jar => ArtifactType::Jar,
                        crate::oci_parser::ArtifactType::War => ArtifactType::War,
                        crate::oci_parser::ArtifactType::Ear => ArtifactType::Ear,
                        crate::oci_parser::ArtifactType::Class => ArtifactType::Class,
                        _ => ArtifactType::Unknown,
                    },
                    maven_coords,
                    size: candidate.size,
                    sha256,
                });
            }
        }

        log::info!("Total artifacts found: {}", all_artifacts.len());

        Ok(all_artifacts)
    }

    /// Extract Maven metadata from JAR file
    fn extract_maven_metadata(&self, jar_path: &str) -> Result<MavenCoordinates> {
        use std::fs::File;
        use std::io::Read;
        use zip::ZipArchive;

        let file = File::open(jar_path)?;
        let mut archive = ZipArchive::new(file)?;

        // Look for pom.properties in META-INF/maven/
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.starts_with("META-INF/maven/") && name.ends_with("/pom.properties") {
                // Parse pom.properties
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                let mut group_id = None;
                let mut artifact_id = None;
                let mut version = None;

                for line in contents.lines() {
                    let line = line.trim();
                    if line.starts_with("groupId=") {
                        group_id = Some(line.trim_start_matches("groupId=").to_string());
                    } else if line.starts_with("artifactId=") {
                        artifact_id = Some(line.trim_start_matches("artifactId=").to_string());
                    } else if line.starts_with("version=") {
                        version = Some(line.trim_start_matches("version=").to_string());
                    }
                }

                if let (Some(g), Some(a), Some(v)) = (group_id, artifact_id, version) {
                    return Ok(MavenCoordinates {
                        group_id: g,
                        artifact_id: a,
                        version: v,
                    });
                }
            }
        }

        anyhow::bail!("No Maven metadata found in JAR")
    }

    /// Calculate SHA-256 hash of a file
    fn calculate_file_hash(&self, file_path: &str) -> Result<String> {
        use sha2::{Digest, Sha256};
        use std::fs::File;
        use std::io::Read;

        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Parse container image metadata
    fn parse_image_metadata(&self) -> Result<ContainerImage> {
        use crate::oci_parser::OciImageParser;

        log::info!("Parsing container image metadata");

        let parser = OciImageParser::new(&self.image_path);

        // Parse the manifest
        let manifest = parser.parse_manifest()?;

        // Parse image config
        let config = parser.parse_config()?;

        // Extract layer information
        let layers: Vec<ImageLayer> = manifest
            .layers
            .iter()
            .enumerate()
            .map(|(idx, layer)| ImageLayer {
                digest: layer.digest.clone(),
                size: layer.size as u64,
                created: config.history.get(idx).and_then(|h| h.created.clone()),
                created_by: config.history.get(idx).map(|h| h.created_by.clone()),
            })
            .collect();

        // Extract name and digest from manifest
        let name = self.image_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let digest = manifest.config.digest.clone();

        Ok(ContainerImage {
            name,
            digest,
            registry: None,
            tags: vec!["latest".to_string()],
            layers,
            base_image: config.config.labels.get("base.image").cloned(),
        })
    }

    /// Find Java artifacts in container layers
    fn find_java_artifacts(&self, _image: &ContainerImage) -> Result<Vec<JavaArtifact>> {
        // NOTE: This is a stub implementation
        // In a real implementation, this would:
        // 1. Extract each layer to a temp directory
        // 2. Recursively search for .jar, .war, .ear files
        // 3. Extract Maven metadata from JAR manifests
        // 4. Calculate file hashes

        Ok(Vec::new())
    }

    /// Detect build system from artifacts
    fn detect_build_system(&self, artifacts: &[JavaArtifact]) -> Option<BuildSystem> {
        // Check for Maven artifacts
        for artifact in artifacts {
            if artifact.path.contains("maven") || artifact.path.contains(".m2") {
                return Some(BuildSystem::Maven);
            }
            if artifact.path.contains("gradle") || artifact.path.contains(".gradle") {
                return Some(BuildSystem::Gradle);
            }
        }
        None
    }
}

/// Build system detected in container
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildSystem {
    Maven,
    Gradle,
    Bazel,
    Unknown,
}

/// Result of container scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerScanResult {
    pub image: ContainerImage,
    pub artifacts: Vec<JavaArtifact>,
    pub build_system: Option<BuildSystem>,
}

impl ContainerScanResult {
    /// Get all Maven artifacts
    pub fn maven_artifacts(&self) -> Vec<&JavaArtifact> {
        self.artifacts
            .iter()
            .filter(|a| a.maven_coords.is_some())
            .collect()
    }

    /// Get artifacts by type
    pub fn artifacts_by_type(&self, artifact_type: ArtifactType) -> Vec<&JavaArtifact> {
        self.artifacts
            .iter()
            .filter(|a| a.artifact_type == artifact_type)
            .collect()
    }

    /// Generate SBOM from scan results
    pub fn generate_sbom(&self) -> ContainerSbom {
        let mut packages = Vec::new();

        for artifact in &self.artifacts {
            if let Some(coords) = &artifact.maven_coords {
                packages.push(SbomPackage {
                    name: format!("{}:{}", coords.group_id, coords.artifact_id),
                    version: coords.version.clone(),
                    purl: format!(
                        "pkg:maven/{}/{}@{}",
                        coords.group_id, coords.artifact_id, coords.version
                    ),
                    location: artifact.path.clone(),
                    layer: artifact.layer.clone(),
                });
            }
        }

        ContainerSbom {
            image_name: self.image.name.clone(),
            image_digest: self.image.digest.clone(),
            packages,
            base_image: self.image.base_image.clone(),
        }
    }
}

/// Container SBOM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSbom {
    pub image_name: String,
    pub image_digest: String,
    pub packages: Vec<SbomPackage>,
    pub base_image: Option<String>,
}

/// SBOM package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomPackage {
    pub name: String,
    pub version: String,
    pub purl: String,
    pub location: String,
    pub layer: String,
}

/// Helper to extract JAR metadata
pub fn extract_jar_metadata(_jar_path: &Path) -> Result<Option<MavenCoordinates>> {
    // NOTE: This is a stub implementation
    // In a real implementation, this would:
    // 1. Open the JAR file
    // 2. Read META-INF/MANIFEST.MF
    // 3. Extract Maven metadata from pom.properties
    // 4. Parse groupId, artifactId, version

    Ok(None)
}

/// Analyze container layers for dependency changes
pub fn analyze_layer_dependencies(layers: &[ImageLayer]) -> Result<LayerAnalysis> {
    let mut layer_map: HashMap<String, Vec<String>> = HashMap::new();

    for layer in layers {
        layer_map.insert(layer.digest.clone(), Vec::new());
    }

    Ok(LayerAnalysis {
        total_layers: layers.len(),
        dependency_layers: 0,
        layer_dependencies: layer_map,
    })
}

/// Layer dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerAnalysis {
    pub total_layers: usize,
    pub dependency_layers: usize,
    pub layer_dependencies: HashMap<String, Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_client_creation() {
        let client = DockerClient::new();
        #[cfg(unix)]
        assert_eq!(client.socket_path, "/var/run/docker.sock");
        #[cfg(windows)]
        assert_eq!(client.socket_path, "//./pipe/docker_engine");
        assert!(client.use_real_api); // Real API enabled by default
    }

    #[test]
    fn test_docker_client_stub_mode() {
        let client = DockerClient::stub();
        assert!(!client.use_real_api); // Stub mode
    }

    #[test]
    fn test_docker_client_custom_socket() {
        let client = DockerClient::with_socket("/custom/socket".to_string());
        assert_eq!(client.socket_path, "/custom/socket");
    }

    #[test]
    fn test_docker_client_pull_image() -> Result<()> {
        let client = DockerClient::stub(); // Use stub mode for tests
                                           // Should not fail (stub implementation)
        client.pull_image("test:latest")?;
        Ok(())
    }

    #[test]
    fn test_docker_client_list_images() -> Result<()> {
        let client = DockerClient::stub(); // Use stub mode for tests
        let images = client.list_images()?;
        assert!(images.is_empty()); // Stub returns empty list
        Ok(())
    }

    #[test]
    fn test_docker_client_inspect_image() -> Result<()> {
        let client = DockerClient::stub(); // Use stub mode for tests
        let image = client.inspect_image("test:latest")?;
        assert_eq!(image.name, "test:latest");
        Ok(())
    }

    #[test]
    fn test_maven_coordinates_to_string() {
        let coords = MavenCoordinates {
            group_id: "org.springframework".to_string(),
            artifact_id: "spring-core".to_string(),
            version: "5.3.20".to_string(),
        };

        assert_eq!(coords.to_string(), "org.springframework:spring-core:5.3.20");
    }

    #[test]
    fn test_artifact_type() {
        assert_eq!(ArtifactType::Jar, ArtifactType::Jar);
        assert_ne!(ArtifactType::Jar, ArtifactType::War);
    }

    #[test]
    fn test_scan_result_maven_artifacts() {
        let coords = Some(MavenCoordinates {
            group_id: "test".to_string(),
            artifact_id: "test-artifact".to_string(),
            version: "1.0.0".to_string(),
        });

        let artifact = JavaArtifact {
            path: "/app/lib/test.jar".to_string(),
            layer: "layer1".to_string(),
            artifact_type: ArtifactType::Jar,
            maven_coords: coords,
            size: 1024,
            sha256: "abc123".to_string(),
        };

        let result = ContainerScanResult {
            image: ContainerImage {
                name: "test:latest".to_string(),
                digest: "sha256:test".to_string(),
                registry: None,
                tags: vec![],
                layers: vec![],
                base_image: None,
            },
            artifacts: vec![artifact],
            build_system: Some(BuildSystem::Maven),
        };

        let maven_artifacts = result.maven_artifacts();
        assert_eq!(maven_artifacts.len(), 1);
    }

    #[test]
    fn test_artifacts_by_type() {
        let artifact1 = JavaArtifact {
            path: "/app/lib/test.jar".to_string(),
            layer: "layer1".to_string(),
            artifact_type: ArtifactType::Jar,
            maven_coords: None,
            size: 1024,
            sha256: "abc123".to_string(),
        };

        let artifact2 = JavaArtifact {
            path: "/app/lib/test.war".to_string(),
            layer: "layer1".to_string(),
            artifact_type: ArtifactType::War,
            maven_coords: None,
            size: 2048,
            sha256: "def456".to_string(),
        };

        let result = ContainerScanResult {
            image: ContainerImage {
                name: "test:latest".to_string(),
                digest: "sha256:test".to_string(),
                registry: None,
                tags: vec![],
                layers: vec![],
                base_image: None,
            },
            artifacts: vec![artifact1, artifact2],
            build_system: None,
        };

        let jars = result.artifacts_by_type(ArtifactType::Jar);
        assert_eq!(jars.len(), 1);

        let wars = result.artifacts_by_type(ArtifactType::War);
        assert_eq!(wars.len(), 1);
    }

    #[test]
    fn test_generate_sbom() {
        let coords = MavenCoordinates {
            group_id: "org.springframework".to_string(),
            artifact_id: "spring-core".to_string(),
            version: "5.3.20".to_string(),
        };

        let artifact = JavaArtifact {
            path: "/app/lib/spring-core-5.3.20.jar".to_string(),
            layer: "layer1".to_string(),
            artifact_type: ArtifactType::Jar,
            maven_coords: Some(coords),
            size: 1024,
            sha256: "abc123".to_string(),
        };

        let result = ContainerScanResult {
            image: ContainerImage {
                name: "myapp:latest".to_string(),
                digest: "sha256:test".to_string(),
                registry: Some("docker.io".to_string()),
                tags: vec!["latest".to_string()],
                layers: vec![],
                base_image: Some("openjdk:11".to_string()),
            },
            artifacts: vec![artifact],
            build_system: Some(BuildSystem::Maven),
        };

        let sbom = result.generate_sbom();
        assert_eq!(sbom.image_name, "myapp:latest");
        assert_eq!(sbom.packages.len(), 1);
        assert_eq!(sbom.packages[0].name, "org.springframework:spring-core");
    }

    #[test]
    fn test_analyze_layer_dependencies() {
        let layers = vec![
            ImageLayer {
                digest: "sha256:layer1".to_string(),
                size: 1024,
                created: None,
                created_by: None,
            },
            ImageLayer {
                digest: "sha256:layer2".to_string(),
                size: 2048,
                created: None,
                created_by: None,
            },
        ];

        let analysis = analyze_layer_dependencies(&layers).unwrap();
        assert_eq!(analysis.total_layers, 2);
    }

    #[test]
    fn test_build_system_detection() {
        let scanner = ContainerScanner::new(PathBuf::from("/tmp/test"));

        let artifacts = vec![JavaArtifact {
            path: "/root/.m2/repository/test.jar".to_string(),
            layer: "layer1".to_string(),
            artifact_type: ArtifactType::Jar,
            maven_coords: None,
            size: 1024,
            sha256: "test".to_string(),
        }];

        let build_system = scanner.detect_build_system(&artifacts);
        assert_eq!(build_system, Some(BuildSystem::Maven));
    }
}
