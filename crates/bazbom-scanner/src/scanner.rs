//! Scanner trait for unified ecosystem scanning
//!
//! This module defines the core `Scanner` trait that all ecosystem scanners implement.
//! It provides a consistent interface for detecting, scanning, and analyzing packages
//! across different programming languages and build systems.

use async_trait::async_trait;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::cache::LicenseCache;
use crate::types::EcosystemScanResult;

/// Context provided to scanners during scanning
#[derive(Debug, Clone)]
pub struct ScanContext {
    /// Root directory of the project
    pub root: PathBuf,
    /// Path to manifest file (if applicable)
    pub manifest: Option<PathBuf>,
    /// Path to lockfile (if applicable)
    pub lockfile: Option<PathBuf>,
    /// Shared license cache for performance
    pub cache: Arc<LicenseCache>,
}

impl ScanContext {
    /// Create a new scan context
    pub fn new(root: PathBuf, cache: Arc<LicenseCache>) -> Self {
        Self {
            root,
            manifest: None,
            lockfile: None,
            cache,
        }
    }

    /// Set the manifest path
    pub fn with_manifest(mut self, manifest: PathBuf) -> Self {
        self.manifest = Some(manifest);
        self
    }

    /// Set the lockfile path
    pub fn with_lockfile(mut self, lockfile: PathBuf) -> Self {
        self.lockfile = Some(lockfile);
        self
    }
}

/// License information for a package
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum License {
    /// SPDX license identifier
    Spdx(String),
    /// Custom license text or identifier
    Custom(String),
    /// Unknown license
    Unknown,
}

impl License {
    /// Convert to SPDX format string
    pub fn as_spdx(&self) -> String {
        match self {
            Self::Spdx(id) => id.clone(),
            Self::Custom(text) => text.clone(),
            Self::Unknown => "NOASSERTION".to_string(),
        }
    }
}

/// Context for fetching license information
pub struct LicenseContext<'a> {
    /// Project root directory
    pub root: &'a Path,
    /// Package name
    pub package: &'a str,
    /// Package version
    pub version: &'a str,
    /// Shared license cache
    pub cache: &'a LicenseCache,
}

/// Main scanner trait that all ecosystem scanners implement
#[async_trait]
pub trait Scanner: Send + Sync {
    /// Get the name of this scanner (e.g., "npm", "python", "maven")
    fn name(&self) -> &str;

    /// Detect if this scanner applies to the given directory
    ///
    /// This should be a fast check, typically just looking for specific files
    /// (e.g., package.json for npm, Cargo.toml for Rust)
    fn detect(&self, root: &Path) -> bool;

    /// Perform a full scan of the ecosystem
    ///
    /// This parses lockfiles, extracts dependencies, and builds the package list.
    async fn scan(&self, ctx: &ScanContext) -> anyhow::Result<EcosystemScanResult>;

    /// Fetch license information for a package
    ///
    /// This method includes caching by default. Override `fetch_license_uncached`
    /// to provide ecosystem-specific license fetching logic.
    fn fetch_license(&self, ctx: &LicenseContext) -> License {
        let key = format!("{}:{}:{}", self.name(), ctx.package, ctx.version);
        ctx.cache.get_or_insert_with(key, || {
            self.fetch_license_uncached(ctx)
        })
    }

    /// Fetch license without caching (override in implementations)
    ///
    /// Default implementation returns Unknown. Ecosystem-specific scanners
    /// should override this to read licenses from node_modules, site-packages, etc.
    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        License::Unknown
    }
}
