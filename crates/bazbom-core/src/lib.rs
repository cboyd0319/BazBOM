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
    Unknown,
}

pub fn detect_build_system<P: AsRef<Path>>(root: P) -> BuildSystem {
    let root = root.as_ref();
    let exists = |p: &str| root.join(p).exists();

    if exists("pom.xml") {
        return BuildSystem::Maven;
    }
    if exists("build.gradle") || exists("build.gradle.kts") || exists("settings.gradle") || exists("settings.gradle.kts") {
        return BuildSystem::Gradle;
    }
    if exists("MODULE.bazel") || exists("WORKSPACE") {
        return BuildSystem::Bazel;
    }
    BuildSystem::Unknown
}

pub fn write_stub_sbom<P: AsRef<Path>>(dir: P, format: &str, system: BuildSystem) -> std::io::Result<PathBuf> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;
    match format {
        "cyclonedx" => {
            let path = dir.join("sbom.cyclonedx.json");
            let content = serde_json::json!({
                "bomFormat": "CycloneDX",
                "specVersion": "1.5",
                "version": 1,
                "metadata": {
                    "tools": [{"name": "bazbom", "version": VERSION}],
                    "properties": [{"name": "buildSystem", "value": format!("{:?}", system)}]
                },
                "components": []
            });
            fs::write(&path, serde_json::to_vec_pretty(&content).unwrap())?;
            Ok(path)
        }
        _ => {
            let path = dir.join("sbom.spdx.json");
            let content = serde_json::json!({
                "spdxVersion": "SPDX-2.3",
                "dataLicense": "CC0-1.0",
                "SPDXID": "SPDXRef-DOCUMENT",
                "name": "bazbom-stub",
                "creationInfo": {
                    "creators": [format!("Tool: bazbom/{}", VERSION)],
                },
                "documentDescribes": [],
                "packages": [],
                "relationships": [],
                "annotations": [
                    {"SPDXID": "SPDXRef-DOCUMENT", "annotationType": "OTHER", "annotator": "Tool: bazbom", "comment": format!("buildSystem={:?}", system)}
                ]
            });
            fs::write(&path, serde_json::to_vec_pretty(&content).unwrap())?;
            Ok(path)
        }
    }
}
