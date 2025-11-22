//! Go modules scanner
//!
//! Parses go.mod and go.sum files

use crate::scanner::{License, LicenseContext, ScanContext, Scanner};
use crate::types::{EcosystemScanResult, Package, ReachabilityData};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Go ecosystem scanner
pub struct GoScanner;

impl Default for GoScanner {
    fn default() -> Self {
        Self
    }
}

impl GoScanner {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Scanner for GoScanner {
    fn name(&self) -> &str {
        "go"
    }

    fn detect(&self, root: &Path) -> bool {
        root.join("go.mod").exists()
    }

    async fn scan(&self, ctx: &ScanContext) -> Result<EcosystemScanResult> {
        let mut result = EcosystemScanResult::new("Go".to_string(), ctx.root.display().to_string());

        // Parse go.mod if available
        if let Some(ref manifest_path) = ctx.manifest {
            parse_go_mod(manifest_path, &mut result)?;
        }

        // go.sum is optional but provides checksums - we can use it to verify versions
        // For now, go.mod is sufficient for dependency tracking

        // Run reachability analysis
        if let Err(e) = analyze_reachability(&ctx.root, &mut result) {
            eprintln!("Warning: Go reachability analysis failed: {}", e);
        }

        Ok(result)
    }

    fn fetch_license_uncached(&self, _ctx: &LicenseContext) -> License {
        // Go doesn't have a standard location for license files in dependencies
        // License info would need to be fetched from go.mod or upstream repo
        License::Unknown
    }
}

/// Analyze reachability for Go project
fn analyze_reachability(root: &Path, result: &mut EcosystemScanResult) -> Result<()> {
    use bazbom_reachability::go::analyze_go_project;

    let report = analyze_go_project(root)?;
    let mut vulnerable_packages_reachable = HashMap::new();

    for package in &result.packages {
        let key = format!("{}@{}", package.name, package.version);
        vulnerable_packages_reachable.insert(key, !report.reachable_functions.is_empty());
    }

    result.reachability = Some(ReachabilityData {
        analyzed: true,
        total_functions: report.all_functions.len(),
        reachable_functions: report.reachable_functions.len(),
        unreachable_functions: report.unreachable_functions.len(),
        vulnerable_packages_reachable,
    });
    Ok(())
}

/// Parse go.mod file
/// Format:
///   module github.com/example/project
///
///   go 1.19
///
///   require (
///       github.com/gin-gonic/gin v1.7.0
///       github.com/lib/pq v1.10.0
///   )
///
///   require github.com/gorilla/mux v1.8.0 // indirect
///
///   replace github.com/old/module => github.com/new/module v1.2.3
fn parse_go_mod(file_path: &Path, result: &mut EcosystemScanResult) -> Result<()> {
    let content = fs::read_to_string(file_path).context("Failed to read go.mod")?;

    let mut replacements: HashMap<String, (String, String)> = HashMap::new();
    let mut in_require_block = false;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with("//") {
            continue;
        }

        // Handle require block
        if line.starts_with("require (") {
            in_require_block = true;
            continue;
        }

        if in_require_block {
            if line == ")" {
                in_require_block = false;
                continue;
            }

            // Parse requirement in block
            if let Some((module, version)) = parse_require_line(line) {
                result.add_package(create_go_package(module, version));
            }
            continue;
        }

        // Handle single-line require
        if line.starts_with("require ") {
            let line = line.strip_prefix("require ").unwrap().trim();
            if let Some((module, version)) = parse_require_line(line) {
                result.add_package(create_go_package(module, version));
            }
            continue;
        }

        // Handle replace directives
        if line.starts_with("replace ") {
            if let Some((old_module, new_module, new_version)) = parse_replace_line(line) {
                replacements.insert(
                    old_module.to_string(),
                    (new_module.to_string(), new_version.to_string()),
                );
            }
            continue;
        }
    }

    // Apply replacements to packages
    apply_replacements(result, &replacements);

    Ok(())
}

/// Parse a require line
/// Examples:
///   github.com/gin-gonic/gin v1.7.0
///   github.com/lib/pq v1.10.0 // indirect
///   golang.org/x/sys v0.0.0-20210630005230-0f9fa26af87c
fn parse_require_line(line: &str) -> Option<(&str, &str)> {
    // Remove comments
    let line = line.split("//").next()?.trim();

    // Split by whitespace
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() >= 2 {
        let module = parts[0];
        let version = parts[1].trim_start_matches('v'); // Remove 'v' prefix

        if !module.is_empty() && !version.is_empty() {
            return Some((module, version));
        }
    }

    None
}

/// Parse a replace directive
/// Examples:
///   replace github.com/old/module => github.com/new/module v1.2.3
///   replace github.com/old => ./local/path
fn parse_replace_line(line: &str) -> Option<(&str, &str, &str)> {
    let line = line.strip_prefix("replace ")?.trim();

    // Split by =>
    let parts: Vec<&str> = line.split("=>").collect();
    if parts.len() != 2 {
        return None;
    }

    let old_module = parts[0].trim();
    let replacement = parts[1].trim();

    // Parse replacement (module + version)
    let replacement_parts: Vec<&str> = replacement.split_whitespace().collect();

    if replacement_parts.len() >= 2 {
        let new_module = replacement_parts[0];
        let new_version = replacement_parts[1].trim_start_matches('v');
        return Some((old_module, new_module, new_version));
    }

    None
}

/// Create a Go package
fn create_go_package(module: &str, version: &str) -> Package {
    // For Go, the full module path IS the package identifier in OSV
    // e.g., "github.com/gin-gonic/gin" should be queried as-is, not split
    // OSV Go ecosystem uses full import paths as package names
    Package {
        name: module.to_string(),
        version: version.to_string(),
        ecosystem: "Go".to_string(),
        namespace: None,
        dependencies: Vec::new(),
        license: None,
        description: None,
        homepage: None,
        repository: None,
    }
}

/// Apply replacement directives to packages
fn apply_replacements(
    result: &mut EcosystemScanResult,
    replacements: &HashMap<String, (String, String)>,
) {
    // Rebuild module path from namespace + name
    for package in &mut result.packages {
        let module_path = if let Some(ref ns) = package.namespace {
            format!("{}/{}", ns, package.name)
        } else {
            package.name.clone()
        };

        // Check if this module has a replacement
        if let Some((new_module, new_version)) = replacements.get(&module_path) {
            // Update to replacement
            if let Some(last_slash) = new_module.rfind('/') {
                package.namespace = Some(new_module[..last_slash].to_string());
                package.name = new_module[last_slash + 1..].to_string();
            } else {
                package.namespace = None;
                package.name = new_module.clone();
            }
            package.version = new_version.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::LicenseCache;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_go_scanner_detect() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("go.mod"), "module test").unwrap();

        let scanner = GoScanner::new();
        assert!(scanner.detect(temp.path()));
    }

    #[test]
    fn test_parse_require_line() {
        assert_eq!(
            parse_require_line("github.com/gin-gonic/gin v1.7.0"),
            Some(("github.com/gin-gonic/gin", "1.7.0"))
        );
        assert_eq!(
            parse_require_line("github.com/lib/pq v1.10.0 // indirect"),
            Some(("github.com/lib/pq", "1.10.0"))
        );
        assert_eq!(
            parse_require_line("golang.org/x/sys v0.0.0-20210630005230-0f9fa26af87c"),
            Some(("golang.org/x/sys", "0.0.0-20210630005230-0f9fa26af87c"))
        );
    }

    #[test]
    fn test_parse_replace_line() {
        assert_eq!(
            parse_replace_line("replace github.com/old/module => github.com/new/module v1.2.3"),
            Some(("github.com/old/module", "github.com/new/module", "1.2.3"))
        );
    }

    #[tokio::test]
    async fn test_parse_go_mod() {
        let temp = TempDir::new().unwrap();
        let go_mod = temp.path().join("go.mod");

        fs::write(
            &go_mod,
            r#"
module github.com/example/project

go 1.19

require (
    github.com/gin-gonic/gin v1.7.0
    github.com/lib/pq v1.10.0
)

require github.com/gorilla/mux v1.8.0 // indirect
"#,
        )
        .unwrap();

        let scanner = GoScanner::new();
        let cache = Arc::new(LicenseCache::new());
        let ctx = ScanContext::new(temp.path().to_path_buf(), cache).with_manifest(go_mod);

        let result = scanner.scan(&ctx).await.unwrap();
        assert_eq!(result.total_packages, 3);

        // Check that packages are found (exact package name depends on how we parse them)
        assert!(result.packages.iter().any(|p| p.name.contains("gin")));
        assert!(result.packages.iter().any(|p| p.name.contains("pq")));
        assert!(result.packages.iter().any(|p| p.name.contains("mux")));
    }
}
