// Dependency updaters for different package ecosystems

pub mod bazel;
pub mod go;
pub mod gradle;
pub mod maven;
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
        System::Maven => Box::new(maven::MavenUpdater),
        System::NuGet => Box::new(maven::MavenUpdater), // NuGet uses similar XML format
        System::Packagist => Box::new(php::PhpUpdater),
        System::Hex => Box::new(ruby::RubyUpdater),     // Elixir uses mix.exs (similar to Ruby)
        System::Pub => Box::new(npm::NpmUpdater),       // Dart pubspec.yaml similar to package.json
        // OS packages don't have updaters - use system package manager
        System::Alpine | System::Debian | System::Rpm => Box::new(maven::MavenUpdater), // Placeholder
    }
}

/// Get the Gradle updater
pub fn get_gradle_updater() -> Box<dyn DependencyUpdater> {
    Box::new(gradle::GradleUpdater)
}

/// Get the Bazel updater
pub fn get_bazel_updater() -> Box<dyn DependencyUpdater> {
    Box::new(bazel::BazelUpdater)
}
