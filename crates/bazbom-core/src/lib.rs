//! Core types and utilities for BazBOM
//!
//! This crate provides fundamental types and functions used across all BazBOM crates:
//! - Build system detection (Maven, Gradle, Bazel, Ant, Buildr, Sbt)
//! - Version information
//! - Common type definitions

use std::fs;
use std::path::{Path, PathBuf};

/// BazBOM version string
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Get the BazBOM version
pub fn version() -> &'static str {
    VERSION
}

/// Get BazBOM's shared cache directory
///
/// Returns `~/.cache/bazbom` on Linux/macOS or equivalent on Windows.
/// Creates the directory if it doesn't exist.
pub fn cache_dir() -> PathBuf {
    let dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("bazbom");

    // Ensure the directory exists
    let _ = std::fs::create_dir_all(&dir);

    dir
}

/// Get a subdirectory within BazBOM's cache
///
/// Example: `cache_subdir("advisories")` returns `~/.cache/bazbom/advisories`
pub fn cache_subdir(name: &str) -> PathBuf {
    let dir = cache_dir().join(name);
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Supported JVM build systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    Maven,
    Gradle,
    Bazel,
    Ant,
    Buildr,
    Sbt,
    Unknown,
}

pub fn detect_build_system<P: AsRef<Path>>(root: P) -> BuildSystem {
    let root = root.as_ref();
    let exists = |p: &str| root.join(p).exists();

    // Check in priority order (most specific first)

    // Maven: pom.xml
    if exists("pom.xml") {
        return BuildSystem::Maven;
    }

    // Gradle: build.gradle, build.gradle.kts, settings.gradle, settings.gradle.kts
    if exists("build.gradle")
        || exists("build.gradle.kts")
        || exists("settings.gradle")
        || exists("settings.gradle.kts")
    {
        return BuildSystem::Gradle;
    }

    // Bazel: MODULE.bazel, WORKSPACE, WORKSPACE.bazel
    if exists("MODULE.bazel") || exists("WORKSPACE") || exists("WORKSPACE.bazel") {
        return BuildSystem::Bazel;
    }

    // sbt (Scala Build Tool): build.sbt or project/build.properties
    if exists("build.sbt") || exists("project/build.properties") {
        return BuildSystem::Sbt;
    }

    // Ant: build.xml
    if exists("build.xml") {
        return BuildSystem::Ant;
    }

    // Buildr: buildfile (lowercase) or Rakefile with Buildr
    if exists("buildfile") {
        return BuildSystem::Buildr;
    }

    // Check for Rakefile with Buildr content
    let rakefile_path = root.join("Rakefile");
    if rakefile_path.exists() {
        if let Ok(content) = fs::read_to_string(&rakefile_path) {
            // Check if Rakefile contains Buildr-specific content
            if content.contains("require 'buildr'")
                || content.contains("require \"buildr\"")
                || content.contains("Buildr.application")
            {
                return BuildSystem::Buildr;
            }
        }
    }

    BuildSystem::Unknown
}

pub fn write_stub_sbom<P: AsRef<Path>>(
    dir: P,
    format: &str,
    system: BuildSystem,
) -> std::io::Result<PathBuf> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    match format {
        "cyclonedx" => {
            let path = dir.join("sbom.cyclonedx.json");
            let bom = bazbom_formats::cyclonedx::CycloneDxBom::new("bazbom", VERSION);
            let content = serde_json::to_vec_pretty(&bom).map_err(std::io::Error::other)?;
            fs::write(&path, content)?;
            Ok(path)
        }
        _ => {
            let path = dir.join("sbom.spdx.json");
            let doc = bazbom_formats::spdx::SpdxDocument::new(
                "bazbom-stub",
                format!("https://github.com/cboyd0319/BazBOM/sbom/{:?}", system),
            );
            let content = serde_json::to_vec_pretty(&doc).map_err(std::io::Error::other)?;
            fs::write(&path, content)?;

            // Also write sca_findings.json as expected by tests
            let findings_path = dir.join("sca_findings.json");
            let empty_findings = serde_json::json!({
                "findings": [],
                "metadata": {
                    "version": VERSION,
                    "build_system": format!("{:?}", system)
                }
            });
            fs::write(
                &findings_path,
                serde_json::to_vec_pretty(&empty_findings).map_err(std::io::Error::other)?,
            )?;

            Ok(path)
        }
    }
}
