//! Dependency resolution for Rust projects
//!
//! Parses Cargo.lock to find all dependencies and locates their source code

use super::error::{Result, RustReachabilityError};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub source_path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: String,
    version: String,
    #[serde(default)]
    source: Option<String>,
}

pub struct DependencyResolver {
    project_root: PathBuf,
    cargo_home: PathBuf,
}

impl DependencyResolver {
    pub fn new(project_root: PathBuf) -> Self {
        let cargo_home = std::env::var("CARGO_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::home_dir()
                    .expect("Could not find home directory")
                    .join(".cargo")
            });

        Self {
            project_root,
            cargo_home,
        }
    }

    /// Resolve all dependencies from Cargo.lock
    pub fn resolve_dependencies(&self) -> Result<Vec<Dependency>> {
        let cargo_lock_path = self.project_root.join("Cargo.lock");

        if !cargo_lock_path.exists() {
            tracing::warn!(
                "Cargo.lock not found at {:?}, cannot resolve dependencies",
                cargo_lock_path
            );
            return Ok(Vec::new());
        }

        let cargo_lock_content = fs::read_to_string(&cargo_lock_path)
            .map_err(|e| RustReachabilityError::IoError(e))?;

        let cargo_lock: CargoLock = toml::from_str(&cargo_lock_content)
            .map_err(|e| RustReachabilityError::ParseError(e.to_string()))?;

        let mut dependencies = Vec::new();

        for package in cargo_lock.package {
            // Skip if it's a path dependency (local crate)
            if let Some(ref source) = package.source {
                if source.starts_with("registry+") {
                    // This is a crates.io dependency
                    let source_path = self.locate_crate_source(&package.name, &package.version);

                    dependencies.push(Dependency {
                        name: package.name.clone(),
                        version: package.version.clone(),
                        source_path,
                    });
                }
            }
        }

        tracing::info!(
            "Resolved {} dependencies from Cargo.lock",
            dependencies.len()
        );

        Ok(dependencies)
    }

    /// Locate the source code for a specific crate version
    fn locate_crate_source(&self, name: &str, version: &str) -> Option<PathBuf> {
        // Try 1: Check for vendored dependencies first (project_root/vendor/)
        let vendor_dir = self.project_root.join("vendor");
        if vendor_dir.exists() {
            // In vendor/, crates are stored as "crate-name/" (version comes from Cargo.toml)
            let crate_path = vendor_dir.join(name);
            if crate_path.exists() && crate_path.is_dir() {
                tracing::debug!("Found vendored source for {}@{} at {:?}", name, version, crate_path);
                return Some(crate_path);
            }
        }

        // Try 2: Check Cargo registry (~/.cargo/registry/src/*/crate-name-version/)
        let registry_src = self.cargo_home.join("registry").join("src");

        if registry_src.exists() {
            // Search in all registry source directories
            for entry in fs::read_dir(registry_src).ok()? {
                let registry_dir = entry.ok()?.path();

                if registry_dir.is_dir() {
                    // Look for crate-name-version directory
                    let crate_dir_name = format!("{}-{}", name, version);
                    let crate_path = registry_dir.join(&crate_dir_name);

                    if crate_path.exists() && crate_path.is_dir() {
                        tracing::debug!("Found source for {}@{} at {:?}", name, version, crate_path);
                        return Some(crate_path);
                    }
                }
            }
        }

        tracing::warn!(
            "Could not locate source for {}@{} (checked vendor/ and cargo registry)",
            name,
            version
        );
        None
    }

    /// Check if a crate is a vulnerable package we care about
    pub fn is_vulnerable_package(&self, name: &str, vulnerable_packages: &[String]) -> bool {
        vulnerable_packages
            .iter()
            .any(|vuln_pkg| vuln_pkg == name || vuln_pkg.starts_with(&format!("{}@", name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_resolver_creation() {
        let resolver = DependencyResolver::new(PathBuf::from("."));
        assert!(resolver.cargo_home.to_string_lossy().contains(".cargo"));
    }
}
