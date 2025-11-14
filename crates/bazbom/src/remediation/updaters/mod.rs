// Dependency updaters for different package ecosystems

pub mod go;
pub mod npm;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;

use anyhow::Result;
use bazbom_depsdev::System;
use std::path::Path;

/// Trait for ecosystem-specific dependency updaters
pub trait DependencyUpdater {
    /// Update dependency version in manifest file
    fn update_version(&self, file_path: &Path, package: &str, new_version: &str) -> Result<()>;

    /// Run package manager install command
    fn install(&self, project_root: &Path) -> Result<()>;

    /// Get lockfile path (if applicable)
    fn lockfile_path(&self, project_root: &Path) -> Option<std::path::PathBuf>;

    /// Get the name of the manifest file for this ecosystem
    fn manifest_name(&self) -> &str;
}

/// Get the appropriate updater for an ecosystem
pub fn get_updater(system: System) -> Box<dyn DependencyUpdater> {
    match system {
        System::Npm => Box::new(npm::NpmUpdater),
        System::PyPI => Box::new(python::PythonUpdater),
        System::Go => Box::new(go::GoUpdater),
        System::Cargo => Box::new(rust::RustUpdater),
        System::RubyGems => Box::new(ruby::RubyUpdater),
        // For PHP, we'd use a separate System variant, but for now use NuGet as placeholder
        System::Maven | System::NuGet => panic!("Maven/Gradle updaters are in build_systems.rs"),
    }
}
