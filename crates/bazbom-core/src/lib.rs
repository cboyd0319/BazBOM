use std::fs;
use std::path::{Path, PathBuf};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn version() -> &'static str {
    VERSION
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSystem {
    Maven,
    Gradle,
    Bazel,
    Ant,
    Buildr,
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
                || content.contains("Buildr.application") {
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
            let content = serde_json::to_vec_pretty(&bom).unwrap();
            fs::write(&path, content)?;
            Ok(path)
        }
        _ => {
            let path = dir.join("sbom.spdx.json");
            let doc = bazbom_formats::spdx::SpdxDocument::new(
                "bazbom-stub",
                format!("https://github.com/cboyd0319/BazBOM/sbom/{:?}", system),
            );
            let content = serde_json::to_vec_pretty(&doc).unwrap();
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
                serde_json::to_vec_pretty(&empty_findings).unwrap(),
            )?;

            Ok(path)
        }
    }
}
