//! Ecosystem Detection
//!
//! Scans a directory tree to detect which programming language ecosystems are present
//! based on manifest files and lockfiles.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Supported ecosystem types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EcosystemType {
    /// Node.js/npm/yarn ecosystem
    Npm,
    /// Python pip/poetry/pipenv ecosystem
    Python,
    /// Go modules ecosystem
    Go,
    /// Rust Cargo ecosystem
    Rust,
    /// Ruby Bundler ecosystem
    Ruby,
    /// PHP Composer ecosystem
    Php,
    /// Maven (Java) ecosystem
    Maven,
    /// Gradle (Java) ecosystem
    Gradle,
}

impl EcosystemType {
    pub fn name(&self) -> &'static str {
        match self {
            EcosystemType::Npm => "Node.js/npm",
            EcosystemType::Python => "Python",
            EcosystemType::Go => "Go",
            EcosystemType::Rust => "Rust",
            EcosystemType::Ruby => "Ruby",
            EcosystemType::Php => "PHP",
            EcosystemType::Maven => "Maven",
            EcosystemType::Gradle => "Gradle",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            EcosystemType::Npm => "📦",
            EcosystemType::Python => "🐍",
            EcosystemType::Go => "🐹",
            EcosystemType::Rust => "🦀",
            EcosystemType::Ruby => "💎",
            EcosystemType::Php => "🐘",
            EcosystemType::Maven => "☕",
            EcosystemType::Gradle => "🐘",
        }
    }
}

/// Detected ecosystem with location and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ecosystem {
    pub ecosystem_type: EcosystemType,
    pub name: String,
    pub root_path: PathBuf,
    pub manifest_file: Option<PathBuf>,
    pub lockfile: Option<PathBuf>,
}

impl Ecosystem {
    pub fn new(
        ecosystem_type: EcosystemType,
        root_path: PathBuf,
        manifest_file: Option<PathBuf>,
        lockfile: Option<PathBuf>,
    ) -> Self {
        Self {
            name: ecosystem_type.name().to_string(),
            ecosystem_type,
            root_path,
            manifest_file,
            lockfile,
        }
    }
}

/// Detect all ecosystems in a directory tree
pub fn detect_ecosystems<P: AsRef<Path>>(path: P) -> Result<Vec<Ecosystem>> {
    let path = path.as_ref();
    let mut ecosystems = Vec::new();

    // Walk directory tree looking for manifest files
    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip common directories that shouldn't be scanned
            let file_name = e.file_name().to_str();
            if let Some(name) = file_name {
                // Skip these directories entirely (don't descend into them)
                !(name == "node_modules"
                    || name == ".git"
                    || name == "target"
                    || name == "dist"
                    || name == "build"
                    || name == "__pycache__"
                    || name == ".venv"
                    || name == "venv")
            } else {
                true
            }
        })
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        let file_name = file_path.file_name().and_then(|n| n.to_str());

        if !entry.file_type().is_file() {
            continue;
        }

        // Detect ecosystem based on manifest files
        if let Some(name) = file_name {
            let dir_path = file_path.parent().unwrap().to_path_buf();

            match name {
                // Node.js/npm
                "package.json" => {
                    let lockfile = find_lockfile(&dir_path, &["package-lock.json", "yarn.lock", "pnpm-lock.yaml"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Npm,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // Python
                "requirements.txt" | "pyproject.toml" | "Pipfile" | "setup.py" => {
                    let lockfile = find_lockfile(&dir_path, &["poetry.lock", "Pipfile.lock", "requirements-lock.txt"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Python,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // Go
                "go.mod" => {
                    let lockfile = find_lockfile(&dir_path, &["go.sum"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Go,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // Rust
                "Cargo.toml" => {
                    let lockfile = find_lockfile(&dir_path, &["Cargo.lock"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Rust,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // Ruby
                "Gemfile" => {
                    let lockfile = find_lockfile(&dir_path, &["Gemfile.lock"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Ruby,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // PHP
                "composer.json" => {
                    let lockfile = find_lockfile(&dir_path, &["composer.lock"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Php,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                // Maven
                "pom.xml" => {
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Maven,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        None,  // Maven doesn't use a traditional lockfile
                    ));
                }

                // Gradle
                "build.gradle" | "build.gradle.kts" => {
                    let lockfile = find_lockfile(&dir_path, &["gradle.lockfile"]);
                    ecosystems.push(Ecosystem::new(
                        EcosystemType::Gradle,
                        dir_path.clone(),
                        Some(file_path.to_path_buf()),
                        lockfile,
                    ));
                }

                _ => {}
            }
        }
    }

    // Deduplicate ecosystems (in case there are multiple manifest files in same directory)
    deduplicate_ecosystems(&mut ecosystems);

    Ok(ecosystems)
}

/// Find lockfile in the same directory
fn find_lockfile(dir: &Path, lockfile_names: &[&str]) -> Option<PathBuf> {
    for name in lockfile_names {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Deduplicate ecosystems by directory (keep the one with lockfile if available)
fn deduplicate_ecosystems(ecosystems: &mut Vec<Ecosystem>) {
    let mut seen = std::collections::HashMap::new();

    ecosystems.retain(|ecosystem| {
        let key = (ecosystem.ecosystem_type, ecosystem.root_path.clone());

        if let Some(existing) = seen.get(&key) {
            // Keep the one with lockfile
            let existing_has_lockfile: &bool = existing;
            let current_has_lockfile = ecosystem.lockfile.is_some();

            if current_has_lockfile && !existing_has_lockfile {
                seen.insert(key, current_has_lockfile);
                return true;
            }
            return false;
        }

        seen.insert(key, ecosystem.lockfile.is_some());
        true
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_detect_npm() {
        let temp = tempfile::tempdir().unwrap();
        let package_json = temp.path().join("package.json");
        fs::write(&package_json, r#"{"name": "test"}"#).unwrap();

        let ecosystems = detect_ecosystems(temp.path()).unwrap();
        assert_eq!(ecosystems.len(), 1);
        assert_eq!(ecosystems[0].ecosystem_type, EcosystemType::Npm);
    }

    #[test]
    fn test_detect_python() {
        let temp = tempfile::tempdir().unwrap();
        let requirements = temp.path().join("requirements.txt");
        fs::write(&requirements, "requests==2.28.0\n").unwrap();

        let ecosystems = detect_ecosystems(temp.path()).unwrap();
        assert_eq!(ecosystems.len(), 1);
        assert_eq!(ecosystems[0].ecosystem_type, EcosystemType::Python);
    }

    #[test]
    fn test_detect_multiple() {
        let temp = tempfile::tempdir().unwrap();

        // Create Node.js project
        fs::write(temp.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

        // Create Python project in subdir
        let python_dir = temp.path().join("api");
        fs::create_dir(&python_dir).unwrap();
        fs::write(python_dir.join("requirements.txt"), "flask==2.0.0\n").unwrap();

        let ecosystems = detect_ecosystems(temp.path()).unwrap();
        assert_eq!(ecosystems.len(), 2);

        let types: Vec<_> = ecosystems.iter().map(|e| e.ecosystem_type).collect();
        assert!(types.contains(&EcosystemType::Npm));
        assert!(types.contains(&EcosystemType::Python));
    }

    #[test]
    fn test_skip_node_modules() {
        let temp = tempfile::tempdir().unwrap();

        // Root package.json
        fs::write(temp.path().join("package.json"), r#"{"name": "root"}"#).unwrap();

        // node_modules package.json (should be skipped)
        let node_modules = temp.path().join("node_modules").join("some-lib");
        fs::create_dir_all(&node_modules).unwrap();
        fs::write(node_modules.join("package.json"), r#"{"name": "lib"}"#).unwrap();

        let ecosystems = detect_ecosystems(temp.path()).unwrap();
        assert_eq!(ecosystems.len(), 1); // Only root package.json
    }
}
