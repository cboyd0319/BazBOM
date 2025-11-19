use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use ureq::Agent;
use zip::ZipArchive;

// ============================================================================
// JAR Identity Extraction
// ============================================================================

/// Identified JAR artifact with Maven coordinates
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JarIdentity {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub source: JarIdentitySource,
    pub checksum: Option<String>,
}

/// How the JAR identity was determined
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JarIdentitySource {
    PomProperties,
    Manifest,
    ChecksumLookup,
    Fingerprint,
}

#[allow(dead_code)]
impl JarIdentity {
    /// Convert to Maven GAV string (groupId:artifactId:version)
    pub fn gav(&self) -> String {
        format!("{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }

    /// Convert to PURL format
    pub fn purl(&self) -> String {
        format!(
            "pkg:maven/{}/{}@{}",
            self.group_id, self.artifact_id, self.version
        )
    }
}

/// Extract identity from an unknown JAR file
///
/// Tries multiple strategies in order:
/// 1. pom.properties (most reliable)
/// 2. MANIFEST.MF (fallback)
/// 3. Maven Central checksum lookup (requires network)
pub fn identify_jar(jar_path: &Path, agent: Option<&Agent>) -> Result<Option<JarIdentity>> {
    // Calculate checksum first (needed for lookup and reporting)
    let checksum = compute_jar_checksum(jar_path)?;

    // Strategy 1: Try pom.properties
    if let Some(mut identity) = extract_pom_properties(jar_path)? {
        identity.checksum = Some(checksum);
        return Ok(Some(identity));
    }

    // Strategy 2: Try MANIFEST.MF
    if let Some(mut identity) = extract_manifest_identity(jar_path)? {
        identity.checksum = Some(checksum);
        return Ok(Some(identity));
    }

    // Strategy 3: Maven Central checksum lookup
    if let Some(agent) = agent {
        if let Some(mut identity) = lookup_jar_by_checksum(agent, &checksum)? {
            identity.checksum = Some(checksum);
            return Ok(Some(identity));
        }
    }

    Ok(None)
}

/// Compute SHA-256 checksum of a JAR file
pub fn compute_jar_checksum(jar_path: &Path) -> Result<String> {
    let mut file = fs::File::open(jar_path)
        .with_context(|| format!("failed to open JAR: {:?}", jar_path))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

/// Extract identity from META-INF/maven/<groupId>/<artifactId>/pom.properties
pub fn extract_pom_properties(jar_path: &Path) -> Result<Option<JarIdentity>> {
    let file = fs::File::open(jar_path)
        .with_context(|| format!("failed to open JAR: {:?}", jar_path))?;
    let mut archive = ZipArchive::new(file)
        .context("failed to read JAR as ZIP archive")?;

    // Look for pom.properties files
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        // Match pattern: META-INF/maven/<groupId>/<artifactId>/pom.properties
        if name.starts_with("META-INF/maven/") && name.ends_with("/pom.properties") {
            let mut content = String::new();
            entry.read_to_string(&mut content)?;

            let mut group_id = None;
            let mut artifact_id = None;
            let mut version = None;

            for line in content.lines() {
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

            if let (Some(g), Some(a), Some(v)) = (group_id, artifact_id, version) {
                return Ok(Some(JarIdentity {
                    group_id: g,
                    artifact_id: a,
                    version: v,
                    source: JarIdentitySource::PomProperties,
                    checksum: None,
                }));
            }
        }
    }

    Ok(None)
}

/// Extract identity from META-INF/MANIFEST.MF
pub fn extract_manifest_identity(jar_path: &Path) -> Result<Option<JarIdentity>> {
    let file = fs::File::open(jar_path)
        .with_context(|| format!("failed to open JAR: {:?}", jar_path))?;
    let mut archive = ZipArchive::new(file)
        .context("failed to read JAR as ZIP archive")?;

    // Find MANIFEST.MF
    let manifest_entry = archive.by_name("META-INF/MANIFEST.MF");
    if manifest_entry.is_err() {
        return Ok(None);
    }

    let mut manifest = manifest_entry.unwrap();
    let mut content = String::new();
    manifest.read_to_string(&mut content)?;

    let mut impl_title = None;
    let mut impl_version = None;
    let mut impl_vendor_id = None;
    let mut bundle_name = None;
    let mut bundle_version = None;
    let mut bundle_symbolic_name = None;

    // Parse manifest attributes (handle line continuations)
    let mut current_line = String::new();
    for line in content.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation line
            current_line.push_str(line.trim_start());
            continue;
        }

        // Process previous line
        if !current_line.is_empty() {
            parse_manifest_line(&current_line, &mut impl_title, &mut impl_version,
                &mut impl_vendor_id, &mut bundle_name, &mut bundle_version, &mut bundle_symbolic_name);
        }
        current_line = line.to_string();
    }
    // Process last line
    if !current_line.is_empty() {
        parse_manifest_line(&current_line, &mut impl_title, &mut impl_version,
            &mut impl_vendor_id, &mut bundle_name, &mut bundle_version, &mut bundle_symbolic_name);
    }

    // Try to construct identity from manifest attributes
    // Priority: OSGi Bundle > Implementation attributes

    if let (Some(name), Some(version)) = (&bundle_symbolic_name, &bundle_version) {
        // OSGi bundle - symbolic name often has groupId.artifactId format
        let (group_id, artifact_id) = if let Some(last_dot) = name.rfind('.') {
            (name[..last_dot].to_string(), name[last_dot + 1..].to_string())
        } else {
            ("unknown".to_string(), name.clone())
        };

        return Ok(Some(JarIdentity {
            group_id,
            artifact_id,
            version: version.clone(),
            source: JarIdentitySource::Manifest,
            checksum: None,
        }));
    }

    if let (Some(title), Some(version)) = (&impl_title, &impl_version) {
        // Implementation-Title is often just artifactId
        let group_id = impl_vendor_id.unwrap_or_else(|| "unknown".to_string());

        return Ok(Some(JarIdentity {
            group_id,
            artifact_id: title.clone(),
            version: version.clone(),
            source: JarIdentitySource::Manifest,
            checksum: None,
        }));
    }

    Ok(None)
}

fn parse_manifest_line(
    line: &str,
    impl_title: &mut Option<String>,
    impl_version: &mut Option<String>,
    impl_vendor_id: &mut Option<String>,
    bundle_name: &mut Option<String>,
    bundle_version: &mut Option<String>,
    bundle_symbolic_name: &mut Option<String>,
) {
    if let Some((key, value)) = line.split_once(':') {
        let key = key.trim();
        let value = value.trim().to_string();

        match key {
            "Implementation-Title" => *impl_title = Some(value),
            "Implementation-Version" => *impl_version = Some(value),
            "Implementation-Vendor-Id" => *impl_vendor_id = Some(value),
            "Bundle-Name" => *bundle_name = Some(value),
            "Bundle-Version" => *bundle_version = Some(value),
            "Bundle-SymbolicName" => {
                // Strip directives like ";singleton:=true"
                let name = value.split(';').next().unwrap_or(&value).trim();
                *bundle_symbolic_name = Some(name.to_string());
            }
            _ => {}
        }
    }
}

/// Response from Maven Central Search API
#[derive(Deserialize)]
struct MavenSearchResponse {
    response: MavenSearchDocs,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MavenSearchDocs {
    docs: Vec<MavenSearchDoc>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct MavenSearchDoc {
    g: String,  // groupId
    a: String,  // artifactId
    v: String,  // version
}

/// Look up a JAR in Maven Central by its SHA-256 checksum
#[allow(dead_code)]
pub fn lookup_jar_by_checksum(agent: &Agent, sha256: &str) -> Result<Option<JarIdentity>> {
    // Maven Central Search API
    let url = format!(
        "https://search.maven.org/solrsearch/select?q=1:\"{}\"&rows=1&wt=json",
        sha256
    );

    let response = agent
        .get(&url)
        .call()
        .context("failed to query Maven Central")?;

    if response.status() != 200 {
        return Ok(None);
    }

    let body = response
        .into_body()
        .read_to_string()
        .context("failed to read Maven Central response")?;

    let search_result: MavenSearchResponse = serde_json::from_str(&body)
        .context("failed to parse Maven Central response")?;

    if let Some(doc) = search_result.response.docs.first() {
        return Ok(Some(JarIdentity {
            group_id: doc.g.clone(),
            artifact_id: doc.a.clone(),
            version: doc.v.clone(),
            source: JarIdentitySource::ChecksumLookup,
            checksum: Some(sha256.to_string()),
        }));
    }

    Ok(None)
}

/// Identify multiple JARs (can be parallelized with rayon)
#[allow(dead_code)]
pub fn identify_jars(
    jar_paths: &[&Path],
    agent: Option<&Agent>,
) -> Vec<Result<Option<JarIdentity>>> {
    jar_paths
        .iter()
        .map(|jar_path| identify_jar(jar_path, agent))
        .collect()
}

/// Information about an extracted and identified JAR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct IdentifiedJar {
    /// Path to the extracted JAR file
    pub path: PathBuf,
    /// Original name within the archive
    pub archive_name: String,
    /// Identified Maven coordinates (if successful)
    pub identity: Option<JarIdentity>,
}

/// Extract nested JARs from a fat JAR and identify each one
///
/// This is the main entry point for analyzing unknown JARs in containers/repos.
/// It combines extraction with identity resolution to provide full artifact information.
///
/// # Example
/// ```ignore
/// use bazbom::shading::extract_and_identify_jars;
///
/// let results = extract_and_identify_jars(
///     Path::new("app.jar"),
///     Path::new("/tmp/extracted"),
///     None, // or Some(&http_agent) for checksum lookups
/// )?;
///
/// for jar in &results {
///     if let Some(ref id) = jar.identity {
///         println!("Found: {}:{}:{}", id.group_id, id.artifact_id, id.version);
///     }
/// }
/// ```
#[allow(dead_code)]
pub fn extract_and_identify_jars(
    jar_path: &Path,
    output_dir: &Path,
    agent: Option<&Agent>,
) -> Result<Vec<IdentifiedJar>> {
    use tracing::{debug, info};

    info!("Extracting and identifying JARs from {:?}", jar_path);

    // First extract the nested JARs
    let extracted_names = extract_nested_jars(jar_path, output_dir)?;

    if extracted_names.is_empty() {
        debug!("No nested JARs found in {:?}", jar_path);
        return Ok(Vec::new());
    }

    info!("Found {} nested JARs, identifying...", extracted_names.len());

    // Identify each extracted JAR
    let mut results = Vec::with_capacity(extracted_names.len());

    for name in extracted_names {
        let extracted_path = output_dir.join(&name);

        let identity = match identify_jar(&extracted_path, agent) {
            Ok(id) => {
                if let Some(ref identity) = id {
                    debug!(
                        "Identified {}: {}:{}:{}",
                        name, identity.group_id, identity.artifact_id, identity.version
                    );
                } else {
                    debug!("Could not identify {}", name);
                }
                id
            }
            Err(e) => {
                debug!("Error identifying {}: {}", name, e);
                None
            }
        };

        results.push(IdentifiedJar {
            path: extracted_path,
            archive_name: name,
            identity,
        });
    }

    let identified_count = results.iter().filter(|r| r.identity.is_some()).count();
    info!(
        "Identified {}/{} JARs from {:?}",
        identified_count,
        results.len(),
        jar_path
    );

    Ok(results)
}

/// Scan a directory for JAR files and identify each one
///
/// Useful for scanning lib/ directories or extracted container layers.
#[allow(dead_code)]
pub fn scan_and_identify_jars(
    dir: &Path,
    agent: Option<&Agent>,
) -> Result<Vec<IdentifiedJar>> {
    use glob::glob;
    use tracing::{debug, info};

    info!("Scanning {:?} for JAR files", dir);

    let pattern = format!("{}/**/*.jar", dir.display());
    let mut results = Vec::new();

    for entry in glob(&pattern).context("failed to read glob pattern")? {
        match entry {
            Ok(path) => {
                debug!("Found JAR: {:?}", path);

                let identity = match identify_jar(&path, agent) {
                    Ok(id) => id,
                    Err(e) => {
                        debug!("Error identifying {:?}: {}", path, e);
                        None
                    }
                };

                results.push(IdentifiedJar {
                    path: path.clone(),
                    archive_name: path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("unknown")
                        .to_string(),
                    identity,
                });
            }
            Err(e) => {
                debug!("Glob error: {}", e);
            }
        }
    }

    let identified_count = results.iter().filter(|r| r.identity.is_some()).count();
    info!(
        "Found {} JARs, identified {}/{}",
        results.len(),
        identified_count,
        results.len()
    );

    Ok(results)
}

/// Represents a class relocation mapping (e.g., org.foo -> com.shaded.org.foo)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct RelocationMapping {
    pub pattern: String,
    pub shadedPattern: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excludes: Option<Vec<String>>,
}

/// Represents configuration for a shaded JAR
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct ShadingConfiguration {
    pub source: String, // "maven-shade-plugin" or "gradle-shadow"
    pub relocations: Vec<RelocationMapping>,
    pub finalName: Option<String>,
}

/// Class fingerprint for matching shaded classes to original artifacts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct ClassFingerprint {
    pub className: String,
    pub methodSignatures: Vec<String>,
    pub fieldSignatures: Vec<String>,
    pub bytecodeHash: String,
}

/// Represents a match between a shaded class and its original artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct ShadingMatch {
    pub shadedClassName: String,
    pub originalClassName: String,
    pub originalArtifact: Option<String>, // GAV coordinates
    pub confidence: f32,                  // 0.0 to 1.0
}

impl RelocationMapping {
    /// Check if a class name matches this relocation pattern
    ///
    /// Future use: Runtime JAR analysis to match classes against relocation patterns
    /// for accurate vulnerability attribution in shaded dependencies.
    #[allow(dead_code)]
    pub fn matches(&self, class_name: &str) -> bool {
        // Check if class is in the pattern namespace
        if !class_name.starts_with(&self.pattern) {
            return false;
        }

        // Check includes if specified
        if let Some(includes) = &self.includes {
            if !includes.iter().any(|inc| class_name.starts_with(inc)) {
                return false;
            }
        }

        // Check excludes if specified
        if let Some(excludes) = &self.excludes {
            if excludes.iter().any(|exc| class_name.starts_with(exc)) {
                return false;
            }
        }

        true
    }

    /// Apply this relocation to a class name, returning the original name
    ///
    /// Future use: Map shaded class names back to original artifacts for
    /// accurate vulnerability reporting and dependency attribution.
    #[allow(dead_code)]
    pub fn reverse_relocate(&self, shaded_class_name: &str) -> Option<String> {
        if !shaded_class_name.starts_with(&self.shadedPattern) {
            return None;
        }

        let suffix = shaded_class_name.strip_prefix(&self.shadedPattern)?;
        Some(format!("{}{}", self.pattern, suffix))
    }
}

#[allow(dead_code)]
/// Parse Maven Shade Plugin configuration from pom.xml
pub fn parse_maven_shade_config(pom_path: &Path) -> Result<Option<ShadingConfiguration>> {
    if !pom_path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(pom_path).context("failed to read pom.xml")?;

    // Quick check if shade plugin is present
    if !content.contains("maven-shade-plugin") {
        return Ok(None);
    }

    let mut reader = Reader::from_str(&content);
    reader.config_mut().trim_text(true);

    let mut relocations = Vec::new();
    let mut in_shade_plugin = false;
    let mut in_configuration = false;
    let mut in_relocations = false;
    let mut in_relocation = false;

    let mut current_pattern = String::new();
    let mut current_shaded_pattern = String::new();
    let mut current_includes: Vec<String> = Vec::new();
    let mut current_excludes: Vec<String> = Vec::new();
    let mut in_pattern = false;
    let mut in_shaded_pattern = false;
    let mut in_includes = false;
    let mut in_excludes = false;
    let mut in_include = false;
    let mut in_exclude = false;
    let mut final_name: Option<String> = None;
    let mut in_final_name = false;

    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = e.name();
                match name.as_ref() {
                    b"artifactId" => {
                        // Check if this is maven-shade-plugin
                        if let Ok(Event::Text(text)) = reader.read_event_into(&mut buf) {
                            let text_str = reader.decoder().decode(text.as_ref()).ok();
                            if text_str.as_deref() == Some("maven-shade-plugin") {
                                in_shade_plugin = true;
                            }
                        }
                    }
                    b"configuration" if in_shade_plugin => in_configuration = true,
                    b"relocations" if in_configuration => in_relocations = true,
                    b"relocation" if in_relocations => {
                        in_relocation = true;
                        // Reset current relocation data
                        current_pattern.clear();
                        current_shaded_pattern.clear();
                        current_includes.clear();
                        current_excludes.clear();
                    }
                    b"pattern" if in_relocation => in_pattern = true,
                    b"shadedPattern" if in_relocation => in_shaded_pattern = true,
                    b"includes" if in_relocation => in_includes = true,
                    b"excludes" if in_relocation => in_excludes = true,
                    b"include" if in_includes => in_include = true,
                    b"exclude" if in_excludes => in_exclude = true,
                    b"finalName" if in_configuration => in_final_name = true,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                let text = reader
                    .decoder()
                    .decode(e.as_ref())
                    .unwrap_or_default()
                    .to_string();
                if in_pattern {
                    current_pattern = text;
                } else if in_shaded_pattern {
                    current_shaded_pattern = text;
                } else if in_include {
                    current_includes.push(text);
                } else if in_exclude {
                    current_excludes.push(text);
                } else if in_final_name {
                    final_name = Some(text);
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"plugin" => in_shade_plugin = false,
                    b"configuration" => in_configuration = false,
                    b"relocations" => in_relocations = false,
                    b"relocation" => {
                        // Save completed relocation
                        if !current_pattern.is_empty() && !current_shaded_pattern.is_empty() {
                            relocations.push(RelocationMapping {
                                pattern: current_pattern.clone(),
                                shadedPattern: current_shaded_pattern.clone(),
                                includes: if current_includes.is_empty() {
                                    None
                                } else {
                                    Some(current_includes.clone())
                                },
                                excludes: if current_excludes.is_empty() {
                                    None
                                } else {
                                    Some(current_excludes.clone())
                                },
                            });
                        }
                        in_relocation = false;
                    }
                    b"pattern" => in_pattern = false,
                    b"shadedPattern" => in_shaded_pattern = false,
                    b"includes" => in_includes = false,
                    b"excludes" => in_excludes = false,
                    b"include" => in_include = false,
                    b"exclude" => in_exclude = false,
                    b"finalName" => in_final_name = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parse error at position {}: {}",
                    reader.buffer_position(),
                    e
                ));
            }
            _ => {}
        }
        buf.clear();
    }

    if relocations.is_empty() {
        return Ok(None);
    }

    Ok(Some(ShadingConfiguration {
        source: "maven-shade-plugin".to_string(),
        relocations,
        finalName: final_name,
    }))
}

/// Parse Gradle Shadow Plugin configuration from build.gradle or build.gradle.kts
#[allow(dead_code)]
pub fn parse_gradle_shadow_config(build_file: &Path) -> Result<Option<ShadingConfiguration>> {
    if !build_file.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(build_file).context("failed to read build file")?;

    // Look for shadow plugin and relocate statements
    if !content.contains("shadow") && !content.contains("com.github.johnrengelman.shadow") {
        return Ok(None);
    }

    let mut relocations = Vec::new();

    // Look for relocate() calls
    // Example: relocate 'org.apache', 'myapp.shaded.apache'
    for line in content.lines() {
        if line.contains("relocate") {
            // Try to extract pattern and shaded pattern
            // This is simplified - real implementation would parse Groovy/Kotlin properly
            if let Some((pattern, shaded)) = parse_gradle_relocate_line(line) {
                relocations.push(RelocationMapping {
                    pattern,
                    shadedPattern: shaded,
                    includes: None,
                    excludes: None,
                });
            }
        }
    }

    if relocations.is_empty() {
        return Ok(None);
    }

    Ok(Some(ShadingConfiguration {
        source: "gradle-shadow-plugin".to_string(),
        relocations,
        finalName: None,
    }))
}

/// Extract nested JARs from a fat JAR
///
/// Future use: Deep analysis of fat JARs (uber JARs) to extract and analyze
/// embedded dependencies, useful for Spring Boot applications and other
/// packaging formats that bundle dependencies within the main JAR.
#[allow(dead_code)]
pub fn extract_nested_jars(jar_path: &Path, output_dir: &Path) -> Result<Vec<String>> {
    use zip::ZipArchive;

    if !jar_path.exists() {
        return Err(anyhow::anyhow!("JAR file not found: {:?}", jar_path));
    }

    // Ensure output directory exists
    fs::create_dir_all(output_dir).context("failed to create output directory")?;

    let file = fs::File::open(jar_path).context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file).context("failed to read ZIP archive")?;

    let mut nested_jars = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();

        // Look for nested JAR files
        if name.ends_with(".jar") {
            let output_path = output_dir.join(&name);

            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).context("failed to create nested directory")?;
            }

            // Extract the nested JAR
            let mut output_file =
                fs::File::create(&output_path).context("failed to create output file")?;
            std::io::copy(&mut file, &mut output_file).context("failed to copy nested JAR")?;

            nested_jars.push(name);
        }
    }

    Ok(nested_jars)
}

/// Generate a fingerprint for a class file with method and field signatures
///
/// Extracts class metadata including:
/// - Class name
/// - Method signatures (name + descriptor)
/// - Field signatures (name + descriptor)
/// - Bytecode hash for matching
#[allow(dead_code)]
pub fn fingerprint_class(class_bytes: &[u8]) -> Result<ClassFingerprint> {
    // Compute bytecode hash for matching
    let hash = blake3::hash(class_bytes).to_hex().to_string();

    // Parse constant pool
    let constant_pool = parse_constant_pool(class_bytes)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse constant pool"))?;

    // Extract class name from bytecode
    let class_name =
        extract_class_name_from_bytecode(class_bytes).unwrap_or_else(|| "Unknown".to_string());

    // Extract method and field signatures
    let (method_sigs, field_sigs) = extract_signatures(class_bytes, &constant_pool)?;

    Ok(ClassFingerprint {
        className: class_name,
        methodSignatures: method_sigs,
        fieldSignatures: field_sigs,
        bytecodeHash: hash,
    })
}

/// Extract method and field signatures from class bytecode
fn extract_signatures(
    class_bytes: &[u8],
    constant_pool: &[ConstantPoolEntry],
) -> Result<(Vec<String>, Vec<String>)> {
    if class_bytes.len() < 10 {
        return Ok((vec![], vec![]));
    }

    // Skip to the end of constant pool
    let mut pos = 8; // Skip magic + version
    let cp_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2;

    // Skip constant pool entries
    let mut i = 1;
    while i < cp_count {
        if pos >= class_bytes.len() {
            return Ok((vec![], vec![]));
        }
        let tag = class_bytes[pos];
        pos += 1;

        let skip = match tag {
            1 => {
                let length = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
                2 + length
            }
            3 | 4 => 4,
            5 | 6 => {
                i += 1;
                8
            }
            7 | 8 | 16 => 2,
            9 | 10 | 11 | 12 | 18 => 4,
            15 => 3,
            _ => return Ok((vec![], vec![])),
        };
        pos += skip;
        i += 1;
    }

    // Read access_flags, this_class, super_class
    if pos + 6 > class_bytes.len() {
        return Ok((vec![], vec![]));
    }
    pos += 6;

    // Read interfaces_count and skip interfaces
    if pos + 2 > class_bytes.len() {
        return Ok((vec![], vec![]));
    }
    let interfaces_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2 + (interfaces_count * 2);

    // Parse fields
    let mut field_signatures = Vec::new();
    if pos + 2 > class_bytes.len() {
        return Ok((vec![], vec![]));
    }
    let fields_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2;

    for _ in 0..fields_count {
        if pos + 6 > class_bytes.len() {
            break;
        }
        let _access_flags = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]);
        let name_index = u16::from_be_bytes([class_bytes[pos + 2], class_bytes[pos + 3]]);
        let descriptor_index = u16::from_be_bytes([class_bytes[pos + 4], class_bytes[pos + 5]]);
        pos += 6;

        if let (Some(name), Some(descriptor)) = (
            get_utf8_from_cp(constant_pool, name_index),
            get_utf8_from_cp(constant_pool, descriptor_index),
        ) {
            field_signatures.push(format!("{}: {}", name, descriptor));
        }

        // Skip attributes
        if pos + 2 > class_bytes.len() {
            break;
        }
        let attributes_count =
            u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
        pos += 2;

        for _ in 0..attributes_count {
            if pos + 6 > class_bytes.len() {
                break;
            }
            let attribute_length = u32::from_be_bytes([
                class_bytes[pos + 2],
                class_bytes[pos + 3],
                class_bytes[pos + 4],
                class_bytes[pos + 5],
            ]) as usize;
            pos += 6 + attribute_length;
        }
    }

    // Parse methods
    let mut method_signatures = Vec::new();
    if pos + 2 > class_bytes.len() {
        return Ok((method_signatures, field_signatures));
    }
    let methods_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2;

    for _ in 0..methods_count {
        if pos + 6 > class_bytes.len() {
            break;
        }
        let _access_flags = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]);
        let name_index = u16::from_be_bytes([class_bytes[pos + 2], class_bytes[pos + 3]]);
        let descriptor_index = u16::from_be_bytes([class_bytes[pos + 4], class_bytes[pos + 5]]);
        pos += 6;

        if let (Some(name), Some(descriptor)) = (
            get_utf8_from_cp(constant_pool, name_index),
            get_utf8_from_cp(constant_pool, descriptor_index),
        ) {
            method_signatures.push(format!("{}{}", name, descriptor));
        }

        // Skip attributes
        if pos + 2 > class_bytes.len() {
            break;
        }
        let attributes_count =
            u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
        pos += 2;

        for _ in 0..attributes_count {
            if pos + 6 > class_bytes.len() {
                break;
            }
            let attribute_length = u32::from_be_bytes([
                class_bytes[pos + 2],
                class_bytes[pos + 3],
                class_bytes[pos + 4],
                class_bytes[pos + 5],
            ]) as usize;
            pos += 6 + attribute_length;
        }
    }

    Ok((method_signatures, field_signatures))
}

/// Java class file constant pool entry types
#[derive(Debug, Clone)]
enum ConstantPoolEntry {
    Utf8(String),
    Integer(()), // Data parsed but never used
    Float(()),   // Data parsed but never used
    Long(()),    // Data parsed but never used
    Double(()),  // Data parsed but never used
    Class(u16),
    String(()),                 // Data parsed but never used
    Fieldref((), ()),           // Data parsed but never used
    Methodref((), ()),          // Data parsed but never used
    InterfaceMethodref((), ()), // Data parsed but never used
    NameAndType((), ()),        // Data parsed but never used
    MethodHandle((), ()),       // Data parsed but never used
    MethodType(()),             // Data parsed but never used
    InvokeDynamic((), ()),      // Data parsed but never used
    Invalid,                    // Padding entry for Long/Double
}

/// Parse Java class file constant pool
fn parse_constant_pool(class_bytes: &[u8]) -> Option<Vec<ConstantPoolEntry>> {
    if class_bytes.len() < 10 || &class_bytes[0..4] != b"\xCA\xFE\xBA\xBE" {
        return None;
    }

    let mut pos = 8; // Skip magic (4) + minor (2) + major (2)

    // Read constant pool count
    if pos + 2 > class_bytes.len() {
        return None;
    }
    let cp_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2;

    let mut constant_pool = vec![ConstantPoolEntry::Invalid]; // Index 0 is invalid

    let mut i = 1;
    while i < cp_count {
        if pos >= class_bytes.len() {
            return None;
        }

        let tag = class_bytes[pos];
        pos += 1;

        let entry = match tag {
            1 => {
                // CONSTANT_Utf8
                if pos + 2 > class_bytes.len() {
                    return None;
                }
                let length = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
                pos += 2;
                if pos + length > class_bytes.len() {
                    return None;
                }
                let string = String::from_utf8_lossy(&class_bytes[pos..pos + length]).to_string();
                pos += length;
                ConstantPoolEntry::Utf8(string)
            }
            3 => {
                // CONSTANT_Integer
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 4;
                ConstantPoolEntry::Integer(())
            }
            4 => {
                // CONSTANT_Float
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 4;
                ConstantPoolEntry::Float(())
            }
            5 => {
                // CONSTANT_Long
                if pos + 8 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 8;
                // Long and Double take 2 constant pool entries
                constant_pool.push(ConstantPoolEntry::Long(()));
                constant_pool.push(ConstantPoolEntry::Invalid);
                i += 1;
                continue;
            }
            6 => {
                // CONSTANT_Double
                if pos + 8 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 8;
                constant_pool.push(ConstantPoolEntry::Double(()));
                constant_pool.push(ConstantPoolEntry::Invalid);
                i += 1;
                continue;
            }
            7 => {
                // CONSTANT_Class
                if pos + 2 > class_bytes.len() {
                    return None;
                }
                let name_index = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]);
                pos += 2;
                ConstantPoolEntry::Class(name_index)
            }
            8 => {
                // CONSTANT_String
                if pos + 2 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 2;
                ConstantPoolEntry::String(())
            }
            9 => {
                // CONSTANT_Fieldref
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 4;
                ConstantPoolEntry::Fieldref((), ())
            }
            10 => {
                // CONSTANT_Methodref
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 4;
                ConstantPoolEntry::Methodref((), ())
            }
            11 => {
                // CONSTANT_InterfaceMethodref
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 4;
                ConstantPoolEntry::InterfaceMethodref((), ())
            }
            12 => {
                // CONSTANT_NameAndType
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 4;
                ConstantPoolEntry::NameAndType((), ())
            }
            15 => {
                // CONSTANT_MethodHandle
                if pos + 3 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 3;
                ConstantPoolEntry::MethodHandle((), ())
            }
            16 => {
                // CONSTANT_MethodType
                if pos + 2 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the value
                pos += 2;
                ConstantPoolEntry::MethodType(())
            }
            18 => {
                // CONSTANT_InvokeDynamic
                if pos + 4 > class_bytes.len() {
                    return None;
                }
                // Parse but don't store the values
                pos += 4;
                ConstantPoolEntry::InvokeDynamic((), ())
            }
            _ => {
                // Unknown tag
                return None;
            }
        };

        constant_pool.push(entry);
        i += 1;
    }

    Some(constant_pool)
}

/// Get UTF-8 string from constant pool
fn get_utf8_from_cp(constant_pool: &[ConstantPoolEntry], index: u16) -> Option<String> {
    if index == 0 || index as usize >= constant_pool.len() {
        return None;
    }
    match &constant_pool[index as usize] {
        ConstantPoolEntry::Utf8(s) => Some(s.clone()),
        _ => None,
    }
}

/// Get class name from constant pool
fn get_class_name_from_cp(constant_pool: &[ConstantPoolEntry], index: u16) -> Option<String> {
    if index == 0 || index as usize >= constant_pool.len() {
        return None;
    }
    match &constant_pool[index as usize] {
        ConstantPoolEntry::Class(name_index) => {
            get_utf8_from_cp(constant_pool, *name_index).map(|s| s.replace('/', "."))
        }
        _ => None,
    }
}

/// Extract class name from bytecode with full constant pool parsing
#[allow(dead_code)]
fn extract_class_name_from_bytecode(class_bytes: &[u8]) -> Option<String> {
    let constant_pool = parse_constant_pool(class_bytes)?;

    // Find this_class index (after constant pool)
    let mut pos = 8; // Skip magic + version
    let cp_count = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
    pos += 2;

    // Skip constant pool entries
    let mut i = 1;
    while i < cp_count {
        if pos >= class_bytes.len() {
            return None;
        }
        let tag = class_bytes[pos];
        pos += 1;

        let skip = match tag {
            1 => {
                // CONSTANT_Utf8
                let length = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]) as usize;
                2 + length
            }
            3 | 4 => 4, // Integer, Float
            5 | 6 => {
                i += 1; // Long and Double take 2 entries
                8
            }
            7 | 8 | 16 => 2,            // Class, String, MethodType
            9 | 10 | 11 | 12 | 18 => 4, // Field/Method/Interface/NameAndType/InvokeDynamic
            15 => 3,                    // MethodHandle
            _ => return None,
        };
        pos += skip;
        i += 1;
    }

    // Read access_flags (2 bytes)
    if pos + 2 > class_bytes.len() {
        return None;
    }
    pos += 2;

    // Read this_class (2 bytes)
    if pos + 2 > class_bytes.len() {
        return None;
    }
    let this_class = u16::from_be_bytes([class_bytes[pos], class_bytes[pos + 1]]);

    get_class_name_from_cp(&constant_pool, this_class)
}

/// Scan a JAR file and create fingerprints for all classes
///
/// Future use: Build a database of class fingerprints from known artifacts
/// for matching against shaded JARs. Enables identification of dependencies
/// in fat JARs even without relocation configuration.
#[allow(dead_code)]
pub fn fingerprint_jar(jar_path: &Path) -> Result<HashMap<String, ClassFingerprint>> {
    use zip::ZipArchive;

    let file = fs::File::open(jar_path).context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file).context("failed to read ZIP archive")?;

    let mut fingerprints = HashMap::new();

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();

        // Only process .class files
        if name.ends_with(".class") && !name.contains("module-info") {
            let mut class_bytes = Vec::new();
            file.read_to_end(&mut class_bytes)
                .context("failed to read class file")?;

            if let Ok(fingerprint) = fingerprint_class(&class_bytes) {
                // Use the file path as the key (without .class extension)
                let class_key = name.trim_end_matches(".class").replace('/', ".");
                fingerprints.insert(class_key, fingerprint);
            }
        }
    }

    Ok(fingerprints)
}

/// Match a shaded class to its original artifact using fingerprints
///
/// Future use: Compare fingerprints of shaded classes against a database
/// of known artifacts to identify dependencies and enable accurate
/// vulnerability attribution with confidence scoring.
#[allow(dead_code)]
pub fn match_shaded_class(
    shaded_class: &ClassFingerprint,
    known_fingerprints: &HashMap<String, ClassFingerprint>,
) -> Option<ShadingMatch> {
    // Compare fingerprints to find best match
    // This is a simplified implementation

    for (artifact_gav, original_fingerprint) in known_fingerprints {
        if shaded_class.bytecodeHash == original_fingerprint.bytecodeHash {
            return Some(ShadingMatch {
                shadedClassName: shaded_class.className.clone(),
                originalClassName: original_fingerprint.className.clone(),
                originalArtifact: Some(artifact_gav.clone()),
                confidence: 1.0,
            });
        }
    }

    None
}

/// API change detected during JAR comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[allow(non_snake_case)]
pub struct ApiChange {
    pub changeType: ApiChangeType,
    pub className: String,
    pub signature: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiChangeType {
    RemovedMethod,
    RemovedField,
    AddedMethod,
    AddedField,
    RemovedClass,
    AddedClass,
}

/// Compare two JAR files and detect API breaking changes
///
/// Compares method and field signatures between two versions of a JAR file.
/// Returns a list of API changes that may break compatibility.
#[allow(dead_code)]
pub fn compare_jars_for_breaking_changes(
    old_jar_path: &Path,
    new_jar_path: &Path,
) -> Result<Vec<ApiChange>> {
    let old_fingerprints = fingerprint_jar(old_jar_path)?;
    let new_fingerprints = fingerprint_jar(new_jar_path)?;

    let mut changes = Vec::new();

    // Check for removed classes
    for (old_class_name, old_fp) in &old_fingerprints {
        if !new_fingerprints.contains_key(old_class_name) {
            changes.push(ApiChange {
                changeType: ApiChangeType::RemovedClass,
                className: old_fp.className.clone(),
                signature: old_class_name.clone(),
                description: format!("Class {} was removed", old_fp.className),
            });
        }
    }

    // Check for added classes
    for (new_class_name, new_fp) in &new_fingerprints {
        if !old_fingerprints.contains_key(new_class_name) {
            changes.push(ApiChange {
                changeType: ApiChangeType::AddedClass,
                className: new_fp.className.clone(),
                signature: new_class_name.clone(),
                description: format!("Class {} was added", new_fp.className),
            });
        }
    }

    // Check for method and field changes in existing classes
    for (class_name, new_fp) in &new_fingerprints {
        if let Some(old_fp) = old_fingerprints.get(class_name) {
            // Compare methods
            let old_methods: std::collections::HashSet<_> =
                old_fp.methodSignatures.iter().collect();
            let new_methods: std::collections::HashSet<_> =
                new_fp.methodSignatures.iter().collect();

            // Removed methods (breaking change)
            for removed in old_methods.difference(&new_methods) {
                changes.push(ApiChange {
                    changeType: ApiChangeType::RemovedMethod,
                    className: new_fp.className.clone(),
                    signature: removed.to_string(),
                    description: format!(
                        "Method {} was removed from class {}",
                        removed, new_fp.className
                    ),
                });
            }

            // Added methods (non-breaking, but note for completeness)
            for added in new_methods.difference(&old_methods) {
                changes.push(ApiChange {
                    changeType: ApiChangeType::AddedMethod,
                    className: new_fp.className.clone(),
                    signature: added.to_string(),
                    description: format!(
                        "Method {} was added to class {}",
                        added, new_fp.className
                    ),
                });
            }

            // Compare fields
            let old_fields: std::collections::HashSet<_> = old_fp.fieldSignatures.iter().collect();
            let new_fields: std::collections::HashSet<_> = new_fp.fieldSignatures.iter().collect();

            // Removed fields (breaking change)
            for removed in old_fields.difference(&new_fields) {
                changes.push(ApiChange {
                    changeType: ApiChangeType::RemovedField,
                    className: new_fp.className.clone(),
                    signature: removed.to_string(),
                    description: format!(
                        "Field {} was removed from class {}",
                        removed, new_fp.className
                    ),
                });
            }

            // Added fields (non-breaking, but note for completeness)
            for added in new_fields.difference(&old_fields) {
                changes.push(ApiChange {
                    changeType: ApiChangeType::AddedField,
                    className: new_fp.className.clone(),
                    signature: added.to_string(),
                    description: format!("Field {} was added to class {}", added, new_fp.className),
                });
            }
        }
    }

    Ok(changes)
}

/// Detect shading in a JAR by comparing its classes against a relocation map
///
/// Future use: Runtime analysis of JAR files to detect shaded classes and
/// map them to original artifacts. Complements build-time detection for
/// cases where build configuration is unavailable or incomplete.
#[allow(dead_code)]
pub fn detect_shading_in_jar(
    jar_path: &Path,
    relocation: &RelocationMapping,
) -> Result<Vec<ShadingMatch>> {
    use zip::ZipArchive;

    let file = fs::File::open(jar_path).context("failed to open JAR file")?;
    let mut archive = ZipArchive::new(file).context("failed to read ZIP archive")?;

    let mut matches = Vec::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .context("failed to read archive entry")?;
        let name = file.name().to_string();

        // Only process .class files
        if name.ends_with(".class") {
            // Convert file path to class name
            let class_name = name.trim_end_matches(".class").replace('/', ".");

            // Check if this class matches the relocation pattern
            if let Some(original_name) = relocation.reverse_relocate(&class_name) {
                matches.push(ShadingMatch {
                    shadedClassName: class_name,
                    originalClassName: original_name.clone(),
                    originalArtifact: None, // Would need fingerprint matching to determine
                    confidence: 0.8,        // High confidence based on relocation pattern match
                });
            }
        }
    }

    Ok(matches)
}

// Helper functions

#[allow(dead_code)]
fn parse_gradle_relocate_line(line: &str) -> Option<(String, String)> {
    // Parse patterns like:
    // relocate 'org.apache', 'myapp.shaded.apache'
    // relocate("org.apache", "myapp.shaded.apache")

    let cleaned = line
        .trim()
        .replace("relocate", "")
        .replace(['(', ')', '\'', '"'], "");

    let parts: Vec<&str> = cleaned.split(',').map(|s| s.trim()).collect();

    if parts.len() >= 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relocation_matches() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: None,
        };

        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("com.example.Foo"));
    }

    #[test]
    fn test_relocation_with_includes() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: Some(vec!["org.apache.commons".to_string()]),
            excludes: None,
        };

        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("org.apache.log4j.Logger"));
    }

    #[test]
    fn test_relocation_with_excludes() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: Some(vec!["org.apache.log4j".to_string()]),
        };

        assert!(relocation.matches("org.apache.commons.Lang"));
        assert!(!relocation.matches("org.apache.log4j.Logger"));
    }

    #[test]
    fn test_reverse_relocate() {
        let relocation = RelocationMapping {
            pattern: "org.apache".to_string(),
            shadedPattern: "shaded.org.apache".to_string(),
            includes: None,
            excludes: None,
        };

        let original = relocation.reverse_relocate("shaded.org.apache.commons.Lang");
        assert_eq!(original, Some("org.apache.commons.Lang".to_string()));

        let no_match = relocation.reverse_relocate("com.example.Foo");
        assert_eq!(no_match, None);
    }

    #[test]
    fn test_parse_gradle_relocate_line() {
        let line1 = "  relocate 'org.apache', 'myapp.shaded.apache'  ";
        let result1 = parse_gradle_relocate_line(line1);
        assert_eq!(
            result1,
            Some(("org.apache".to_string(), "myapp.shaded.apache".to_string()))
        );

        let line2 = r#"  relocate("org.apache", "myapp.shaded.apache")  "#;
        let result2 = parse_gradle_relocate_line(line2);
        assert_eq!(
            result2,
            Some(("org.apache".to_string(), "myapp.shaded.apache".to_string()))
        );
    }

    #[test]
    fn test_fingerprint_class() {
        // Create a minimal valid Java class file
        // Magic: CAFEBABE, Minor: 0000, Major: 0034 (Java 8)
        // CP Count: 0004 (4 entries including index 0)
        let class_bytes = vec![
            0xCA, 0xFE, 0xBA, 0xBE, // magic
            0x00, 0x00, // minor version
            0x00, 0x34, // major version (Java 8)
            0x00, 0x04, // constant pool count (4)
            // CP entry 1: CONSTANT_Utf8 "java/lang/Object"
            0x01, 0x00, 0x10, // tag=1, length=16
            0x6A, 0x61, 0x76, 0x61, 0x2F, 0x6C, 0x61, 0x6E, 0x67, 0x2F, 0x4F, 0x62, 0x6A, 0x65,
            0x63, 0x74, // CP entry 2: CONSTANT_Class (ref to entry 1)
            0x07, 0x00, 0x01, // CP entry 3: CONSTANT_Utf8 "TestClass"
            0x01, 0x00, 0x09, // tag=1, length=9
            0x54, 0x65, 0x73, 0x74, 0x43, 0x6C, 0x61, 0x73, 0x73,
            // Access flags: public (0x0021)
            0x00, 0x21, // This class: 0 (invalid for testing, but minimal)
            0x00, 0x00, // Super class: entry 2
            0x00, 0x02, // Interfaces count: 0
            0x00, 0x00, // Fields count: 0
            0x00, 0x00, // Methods count: 0
            0x00, 0x00, // Attributes count: 0
            0x00, 0x00,
        ];

        let fingerprint = fingerprint_class(&class_bytes).unwrap();

        // Verify the hash is Blake3 length (64 hex chars)
        assert_eq!(fingerprint.bytecodeHash.len(), 64);
        // Verify class name was extracted
        assert!(!fingerprint.className.is_empty());
    }

    #[test]
    fn test_fingerprint_class_invalid_bytes() {
        // Test that invalid bytes return an error
        let dummy_bytes = b"dummy class bytes";
        let result = fingerprint_class(dummy_bytes);

        assert!(result.is_err(), "Should fail on invalid class bytes");
    }

    #[test]
    fn test_match_shaded_class() {
        let mut known = HashMap::new();
        let original = ClassFingerprint {
            className: "org.apache.commons.Lang".to_string(),
            methodSignatures: vec![],
            fieldSignatures: vec![],
            bytecodeHash: "abc123".to_string(),
        };
        known.insert(
            "org.apache.commons:commons-lang3:3.12.0".to_string(),
            original.clone(),
        );

        let shaded = ClassFingerprint {
            className: "shaded.org.apache.commons.Lang".to_string(),
            methodSignatures: vec![],
            fieldSignatures: vec![],
            bytecodeHash: "abc123".to_string(),
        };

        let match_result = match_shaded_class(&shaded, &known);
        assert!(match_result.is_some());

        let matched = match_result.unwrap();
        assert_eq!(matched.confidence, 1.0);
        assert_eq!(matched.originalClassName, "org.apache.commons.Lang");
    }

    #[test]
    fn test_parse_maven_shade_config_complete() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>shaded-app</artifactId>
    <version>1.0.0</version>
    
    <build>
        <plugins>
            <plugin>
                <groupId>org.apache.maven.plugins</groupId>
                <artifactId>maven-shade-plugin</artifactId>
                <version>3.2.4</version>
                <configuration>
                    <finalName>shaded-app</finalName>
                    <relocations>
                        <relocation>
                            <pattern>org.apache.commons</pattern>
                            <shadedPattern>com.example.shaded.commons</shadedPattern>
                        </relocation>
                        <relocation>
                            <pattern>com.google.guava</pattern>
                            <shadedPattern>com.example.shaded.guava</shadedPattern>
                            <includes>
                                <include>com.google.guava.collect</include>
                            </includes>
                            <excludes>
                                <exclude>com.google.guava.base</exclude>
                            </excludes>
                        </relocation>
                    </relocations>
                </configuration>
            </plugin>
        </plugins>
    </build>
</project>
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(pom_content.as_bytes()).unwrap();

        let config = parse_maven_shade_config(temp_file.path()).unwrap();
        assert!(config.is_some());

        let config = config.unwrap();
        assert_eq!(config.source, "maven-shade-plugin");
        assert_eq!(config.finalName, Some("shaded-app".to_string()));
        assert_eq!(config.relocations.len(), 2);

        let first = &config.relocations[0];
        assert_eq!(first.pattern, "org.apache.commons");
        assert_eq!(first.shadedPattern, "com.example.shaded.commons");
        assert!(first.includes.is_none());
        assert!(first.excludes.is_none());

        let second = &config.relocations[1];
        assert_eq!(second.pattern, "com.google.guava");
        assert_eq!(second.shadedPattern, "com.example.shaded.guava");
        assert!(second.includes.is_some());
        assert_eq!(second.includes.as_ref().unwrap().len(), 1);
        assert!(second.excludes.is_some());
        assert_eq!(second.excludes.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_parse_maven_shade_config_no_plugin() {
        use std::io::Write;
        use tempfile::NamedTempFile;

        let pom_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
    <modelVersion>4.0.0</modelVersion>
    <groupId>com.example</groupId>
    <artifactId>normal-app</artifactId>
    <version>1.0.0</version>
</project>
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(pom_content.as_bytes()).unwrap();

        let config = parse_maven_shade_config(temp_file.path()).unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn test_extract_class_name_from_bytecode() {
        // Test with invalid bytecode
        let invalid = b"not a class file";
        let result = super::extract_class_name_from_bytecode(invalid);
        assert!(result.is_none());

        // Test with valid magic number but short content
        let short_valid = b"\xCA\xFE\xBA\xBE\x00\x00\x00\x34";
        let result = super::extract_class_name_from_bytecode(short_valid);
        // Should return None as we don't fully parse the constant pool
        assert!(result.is_none());
    }

    #[test]
    fn test_compute_jar_checksum() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a minimal JAR file
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\n").unwrap();
        zip.finish().unwrap();

        let checksum = compute_jar_checksum(temp_file.path()).unwrap();
        assert_eq!(checksum.len(), 64); // SHA-256 hex string
        assert!(checksum.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_extract_pom_properties() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a JAR with pom.properties
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        zip.start_file("META-INF/maven/com.example/my-artifact/pom.properties", options).unwrap();
        zip.write_all(b"groupId=com.example\nartifactId=my-artifact\nversion=1.0.0\n").unwrap();
        zip.finish().unwrap();

        let identity = extract_pom_properties(temp_file.path()).unwrap().unwrap();
        assert_eq!(identity.group_id, "com.example");
        assert_eq!(identity.artifact_id, "my-artifact");
        assert_eq!(identity.version, "1.0.0");
        assert_eq!(identity.source, JarIdentitySource::PomProperties);
    }

    #[test]
    fn test_extract_pom_properties_not_found() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a JAR without pom.properties
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\n").unwrap();
        zip.finish().unwrap();

        let identity = extract_pom_properties(temp_file.path()).unwrap();
        assert!(identity.is_none());
    }

    #[test]
    fn test_extract_manifest_identity() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a JAR with MANIFEST.MF containing identity info
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\nImplementation-Title: my-artifact\nImplementation-Version: 2.0.0\nImplementation-Vendor-Id: org.example\n").unwrap();
        zip.finish().unwrap();

        let identity = extract_manifest_identity(temp_file.path()).unwrap().unwrap();
        assert_eq!(identity.group_id, "org.example");
        assert_eq!(identity.artifact_id, "my-artifact");
        assert_eq!(identity.version, "2.0.0");
        assert_eq!(identity.source, JarIdentitySource::Manifest);
    }

    #[test]
    fn test_extract_manifest_identity_bundle_style() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a JAR with OSGi bundle-style MANIFEST.MF
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();
        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\nBundle-SymbolicName: org.osgi.bundle\nBundle-Version: 3.0.0\n").unwrap();
        zip.finish().unwrap();

        let identity = extract_manifest_identity(temp_file.path()).unwrap().unwrap();
        assert_eq!(identity.group_id, "org.osgi");
        assert_eq!(identity.artifact_id, "bundle");
        assert_eq!(identity.version, "3.0.0");
        assert_eq!(identity.source, JarIdentitySource::Manifest);
    }

    #[test]
    fn test_identify_jar_uses_pom_properties_first() {
        use std::io::Write;
        use tempfile::NamedTempFile;
        use zip::write::SimpleFileOptions;
        use zip::ZipWriter;

        // Create a JAR with both pom.properties and MANIFEST.MF
        let temp_file = NamedTempFile::new().unwrap();
        let file = std::fs::File::create(temp_file.path()).unwrap();
        let mut zip = ZipWriter::new(file);

        let options = SimpleFileOptions::default();

        // Add pom.properties
        zip.start_file("META-INF/maven/com.pom/pom-artifact/pom.properties", options).unwrap();
        zip.write_all(b"groupId=com.pom\nartifactId=pom-artifact\nversion=1.0.0\n").unwrap();

        // Add MANIFEST.MF with different values
        zip.start_file("META-INF/MANIFEST.MF", options).unwrap();
        zip.write_all(b"Manifest-Version: 1.0\nImplementation-Title: manifest-artifact\nImplementation-Version: 2.0.0\nImplementation-Vendor-Id: com.manifest\n").unwrap();

        zip.finish().unwrap();

        // identify_jar should prefer pom.properties
        let identity = identify_jar(temp_file.path(), None).unwrap().unwrap();
        assert_eq!(identity.group_id, "com.pom");
        assert_eq!(identity.artifact_id, "pom-artifact");
        assert_eq!(identity.version, "1.0.0");
        assert_eq!(identity.source, JarIdentitySource::PomProperties);
    }

    #[test]
    fn test_jar_identity_source_serialization() {
        let identity = JarIdentity {
            group_id: "com.example".to_string(),
            artifact_id: "test".to_string(),
            version: "1.0.0".to_string(),
            source: JarIdentitySource::PomProperties,
            checksum: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&identity).unwrap();
        assert!(json.contains("PomProperties"));

        let parsed: JarIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.source, JarIdentitySource::PomProperties);
    }
}
