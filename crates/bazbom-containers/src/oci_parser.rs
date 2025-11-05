//! OCI image format parser
//!
//! Parses OCI/Docker image tarballs and extracts metadata, layers, and configuration

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Archive;

/// OCI Image manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub config: OciDescriptor,
    pub layers: Vec<OciDescriptor>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

/// OCI descriptor (references content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciDescriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: String,
    pub size: i64,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub annotations: HashMap<String, String>,
}

/// OCI Image configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciImageConfig {
    pub architecture: String,
    pub os: String,
    #[serde(default)]
    pub config: OciContainerConfig,
    #[serde(default)]
    pub rootfs: OciRootFs,
    #[serde(default)]
    pub history: Vec<OciHistory>,
}

/// Container configuration in OCI image
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OciContainerConfig {
    #[serde(default)]
    #[serde(rename = "Env")]
    pub env: Vec<String>,
    #[serde(default)]
    #[serde(rename = "Cmd")]
    pub cmd: Vec<String>,
    #[serde(default)]
    #[serde(rename = "Entrypoint")]
    pub entrypoint: Vec<String>,
    #[serde(default)]
    #[serde(rename = "WorkingDir")]
    pub working_dir: String,
    #[serde(default)]
    #[serde(rename = "Labels")]
    pub labels: HashMap<String, String>,
}

/// Root filesystem information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OciRootFs {
    #[serde(rename = "type")]
    pub fs_type: String,
    #[serde(default)]
    pub diff_ids: Vec<String>,
}

/// History entry in OCI image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciHistory {
    pub created: Option<String>,
    #[serde(default)]
    pub created_by: String,
    #[serde(default)]
    pub empty_layer: bool,
    #[serde(default)]
    pub comment: String,
}

/// OCI image parser
pub struct OciImageParser {
    /// Path to the image tarball
    image_path: PathBuf,
}

impl OciImageParser {
    /// Create a new OCI image parser
    pub fn new(image_path: impl AsRef<Path>) -> Self {
        Self {
            image_path: image_path.as_ref().to_path_buf(),
        }
    }

    /// Parse the manifest.json from the image tarball
    pub fn parse_manifest(&self) -> Result<OciManifest> {
        let file = File::open(&self.image_path)
            .with_context(|| format!("Failed to open image file: {}", self.image_path.display()))?;

        let mut archive = Archive::new(file);

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;

            if path.file_name() == Some(std::ffi::OsStr::new("manifest.json")) {
                let mut contents = String::new();
                entry.read_to_string(&mut contents)?;

                // Docker image format uses an array of manifests
                let manifests: Vec<DockerManifestEntry> =
                    serde_json::from_str(&contents).context("Failed to parse manifest.json")?;

                if let Some(manifest) = manifests.first() {
                    return self.convert_docker_manifest(manifest);
                }
            }
        }

        anyhow::bail!("No manifest.json found in image tarball")
    }

    /// Parse the image configuration (auto-detect from manifest)
    pub fn parse_config(&self) -> Result<OciImageConfig> {
        // First get the manifest to find the config digest
        let manifest = self.parse_manifest()?;
        let config_digest = &manifest.config.digest;

        self.parse_config_with_digest(config_digest)
    }

    /// Parse the image configuration with explicit digest
    pub fn parse_config_with_digest(&self, config_digest: &str) -> Result<OciImageConfig> {
        let file = File::open(&self.image_path)
            .with_context(|| format!("Failed to open image file: {}", self.image_path.display()))?;

        let mut archive = Archive::new(file);

        // Docker uses blobs/sha256/<hash>.json or just <hash>.json
        let config_filename = config_digest.replace("sha256:", "");

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;
            let path_str = path.to_string_lossy();

            // Match both <hash>.json and blobs/sha256/<hash>.json
            if path_str.contains(&config_filename) && path_str.ends_with(".json") {
                let mut contents = String::new();
                entry.read_to_string(&mut contents)?;

                let config: OciImageConfig =
                    serde_json::from_str(&contents).context("Failed to parse image config")?;

                return Ok(config);
            }
        }

        anyhow::bail!("Config file not found: {}", config_digest)
    }

    /// Extract all layers from the image
    pub fn extract_layers(&self, output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
        let file = File::open(&self.image_path)
            .with_context(|| format!("Failed to open image file: {}", self.image_path.display()))?;

        let mut archive = Archive::new(file);
        let mut extracted_layers = Vec::new();

        std::fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;

            // Layer files are typically named like <hash>/layer.tar
            if path.extension() == Some(std::ffi::OsStr::new("tar"))
                && !path.to_string_lossy().contains("manifest")
            {
                let layer_path = output_dir.as_ref().join(path.file_name().unwrap());
                let mut output = File::create(&layer_path).with_context(|| {
                    format!("Failed to create layer file: {}", layer_path.display())
                })?;

                std::io::copy(&mut entry, &mut output)?;
                extracted_layers.push(layer_path);
            }
        }

        Ok(extracted_layers)
    }

    /// Get layer digests from the image
    pub fn get_layer_digests(&self) -> Result<Vec<String>> {
        let manifest = self.parse_manifest()?;
        Ok(manifest.layers.iter().map(|l| l.digest.clone()).collect())
    }

    /// Convert Docker manifest format to OCI format
    fn convert_docker_manifest(
        &self,
        docker_manifest: &DockerManifestEntry,
    ) -> Result<OciManifest> {
        let config = OciDescriptor {
            media_type: "application/vnd.docker.container.image.v1+json".to_string(),
            digest: docker_manifest.config.clone(),
            size: 0, // Not available in Docker manifest
            urls: vec![],
            annotations: HashMap::new(),
        };

        let layers = docker_manifest
            .layers
            .iter()
            .map(|layer| OciDescriptor {
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
                digest: layer.clone(),
                size: 0,
                urls: vec![],
                annotations: HashMap::new(),
            })
            .collect();

        Ok(OciManifest {
            schema_version: 2,
            media_type: "application/vnd.docker.distribution.manifest.v2+json".to_string(),
            config,
            layers,
            annotations: HashMap::new(),
        })
    }

    /// Scan a layer for Java artifacts
    pub fn scan_layer_for_artifacts(
        &self,
        layer_path: impl AsRef<Path>,
    ) -> Result<Vec<JavaArtifactCandidate>> {
        let file = File::open(layer_path.as_ref())
            .with_context(|| format!("Failed to open layer: {}", layer_path.as_ref().display()))?;

        let mut archive = Archive::new(file);
        let mut artifacts = Vec::new();

        for entry_result in archive.entries()? {
            let entry = entry_result?;
            let path = entry.path()?;

            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy();
                if matches!(ext_str.as_ref(), "jar" | "war" | "ear" | "class") {
                    artifacts.push(JavaArtifactCandidate {
                        path: path.to_string_lossy().to_string(),
                        size: entry.size(),
                        artifact_type: match ext_str.as_ref() {
                            "jar" => ArtifactType::Jar,
                            "war" => ArtifactType::War,
                            "ear" => ArtifactType::Ear,
                            "class" => ArtifactType::Class,
                            _ => ArtifactType::Unknown,
                        },
                    });
                }
            }
        }

        Ok(artifacts)
    }

    /// Extract Maven metadata from a JAR file in a layer
    pub fn extract_maven_metadata(
        &self,
        layer_path: impl AsRef<Path>,
        jar_path: &str,
    ) -> Result<Option<MavenMetadata>> {
        let file = File::open(layer_path.as_ref())
            .with_context(|| format!("Failed to open layer: {}", layer_path.as_ref().display()))?;

        let mut archive = Archive::new(file);

        // First, extract the JAR file from the layer
        for entry_result in archive.entries()? {
            let mut entry = entry_result?;
            let path = entry.path()?;

            if path.to_string_lossy() == jar_path {
                // Read JAR contents into memory
                let mut jar_contents = Vec::new();
                entry.read_to_end(&mut jar_contents)?;

                // Parse JAR as a ZIP archive to find pom.properties
                return self.parse_jar_for_maven_metadata(&jar_contents);
            }
        }

        Ok(None)
    }

    /// Parse JAR file (as ZIP) to extract Maven metadata
    fn parse_jar_for_maven_metadata(&self, jar_contents: &[u8]) -> Result<Option<MavenMetadata>> {
        use std::io::Cursor;

        let cursor = Cursor::new(jar_contents);
        let mut archive =
            zip::ZipArchive::new(cursor).context("Failed to read JAR as ZIP archive")?;

        // Look for META-INF/maven/*/*/pom.properties
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if name.starts_with("META-INF/maven/") && name.ends_with("pom.properties") {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                return Ok(Some(Self::parse_pom_properties(&contents)?));
            }
        }

        Ok(None)
    }

    /// Parse pom.properties file format
    fn parse_pom_properties(contents: &str) -> Result<MavenMetadata> {
        let mut group_id = None;
        let mut artifact_id = None;
        let mut version = None;

        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "groupId" => group_id = Some(value.trim().to_string()),
                    "artifactId" => artifact_id = Some(value.trim().to_string()),
                    "version" => version = Some(value.trim().to_string()),
                    _ => {}
                }
            }
        }

        match (group_id, artifact_id, version) {
            (Some(g), Some(a), Some(v)) => Ok(MavenMetadata {
                group_id: g,
                artifact_id: a,
                version: v,
            }),
            _ => anyhow::bail!("Incomplete Maven metadata in pom.properties"),
        }
    }
}

/// Docker manifest entry (Docker image format)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DockerManifestEntry {
    #[serde(rename = "Config")]
    pub config: String,
    #[serde(rename = "RepoTags")]
    pub repo_tags: Vec<String>,
    #[serde(rename = "Layers")]
    pub layers: Vec<String>,
}

/// Type of Java artifact
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactType {
    Jar,
    War,
    Ear,
    Class,
    Unknown,
}

/// Java artifact candidate found in layer
#[derive(Debug, Clone)]
pub struct JavaArtifactCandidate {
    pub path: String,
    pub size: u64,
    pub artifact_type: ArtifactType,
}

/// Maven metadata extracted from pom.properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MavenMetadata {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_descriptor_creation() {
        let descriptor = OciDescriptor {
            media_type: "application/vnd.oci.image.layer.v1.tar".to_string(),
            digest: "sha256:abcd1234".to_string(),
            size: 1024,
            urls: vec![],
            annotations: HashMap::new(),
        };

        assert_eq!(descriptor.digest, "sha256:abcd1234");
        assert_eq!(descriptor.size, 1024);
    }

    #[test]
    fn test_oci_image_config_creation() {
        let config = OciImageConfig {
            architecture: "amd64".to_string(),
            os: "linux".to_string(),
            config: OciContainerConfig::default(),
            rootfs: OciRootFs::default(),
            history: vec![],
        };

        assert_eq!(config.architecture, "amd64");
        assert_eq!(config.os, "linux");
    }

    #[test]
    fn test_artifact_type_detection() {
        let artifact = JavaArtifactCandidate {
            path: "/app/lib/myapp.jar".to_string(),
            size: 1024,
            artifact_type: ArtifactType::Jar,
        };

        assert_eq!(artifact.artifact_type, ArtifactType::Jar);
        assert_eq!(artifact.size, 1024);
    }

    #[test]
    fn test_oci_manifest_serialization() {
        let manifest = OciManifest {
            schema_version: 2,
            media_type: "application/vnd.oci.image.manifest.v1+json".to_string(),
            config: OciDescriptor {
                media_type: "application/vnd.oci.image.config.v1+json".to_string(),
                digest: "sha256:config".to_string(),
                size: 512,
                urls: vec![],
                annotations: HashMap::new(),
            },
            layers: vec![],
            annotations: HashMap::new(),
        };

        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("schemaVersion"));
        assert!(json.contains("mediaType"));
    }
}
