//! Native dependency analysis for non-deps.dev ecosystems
//!
//! Uses bazbom-scanner to parse lockfiles and build dependency trees
//! for ecosystems not supported by deps.dev (Packagist, Hex, Pub).

use anyhow::{Context, Result};
use bazbom_depsdev::{DependencyEdge, DependencyGraph, DependencyNode, Relation, System, VersionKey};
use bazbom_scanner::cache::LicenseCache;
use bazbom_scanner::scanner::ScanContext;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, warn};

/// Get dependency graph for a non-deps.dev ecosystem by parsing lockfiles
///
/// Returns a DependencyGraph compatible with the deps.dev format
pub async fn get_native_dependencies(
    system: System,
    package: &str,
    version: &str,
    project_root: Option<&Path>,
) -> Result<DependencyGraph> {
    // If we have a project root, scan it directly
    if let Some(root) = project_root {
        return scan_project_dependencies(system, root).await;
    }

    // Otherwise, we need to fetch the package and resolve its dependencies
    // This requires downloading/analyzing the package which is more complex
    // For now, return an empty graph with just the target package
    warn!(
        "No project root provided for {:?} - returning minimal dependency graph",
        system
    );

    Ok(DependencyGraph {
        nodes: vec![DependencyNode {
            version_key: VersionKey {
                system,
                name: package.to_string(),
                version: version.to_string(),
            },
            relation: Relation::SelfRelation,
            errors: vec![],
        }],
        edges: vec![],
    })
}

/// Scan a project directory and return its dependency graph
async fn scan_project_dependencies(system: System, root: &Path) -> Result<DependencyGraph> {
    let cache = Arc::new(LicenseCache::new());

    let (lockfile, manifest) = find_lockfile_and_manifest(system, root)?;

    let mut ctx = ScanContext::new(root.to_path_buf(), cache);
    if let Some(lf) = lockfile {
        ctx = ctx.with_lockfile(lf);
    }
    if let Some(mf) = manifest {
        ctx = ctx.with_manifest(mf);
    }

    // Get the appropriate scanner
    let result = match system {
        System::Packagist => {
            let scanner = bazbom_scanner::ecosystems::php::PhpScanner::new();
            bazbom_scanner::scanner::Scanner::scan(&scanner, &ctx).await?
        }
        System::Hex => {
            // TODO: Add Elixir scanner to bazbom-scanner
            warn!("Hex scanner not yet implemented in bazbom-scanner");
            return Ok(empty_graph());
        }
        System::Pub => {
            // TODO: Add Dart scanner to bazbom-scanner
            warn!("Pub scanner not yet implemented in bazbom-scanner");
            return Ok(empty_graph());
        }
        _ => {
            // For deps.dev supported ecosystems, this shouldn't be called
            warn!("Native scanning not needed for {:?}", system);
            return Ok(empty_graph());
        }
    };

    // Convert scanner result to DependencyGraph
    convert_to_dependency_graph(system, &result.packages)
}

/// Find lockfile and manifest for an ecosystem
fn find_lockfile_and_manifest(
    system: System,
    root: &Path,
) -> Result<(Option<PathBuf>, Option<PathBuf>)> {
    match system {
        System::Packagist => {
            let lockfile = root.join("composer.lock");
            let manifest = root.join("composer.json");
            Ok((
                if lockfile.exists() {
                    Some(lockfile)
                } else {
                    None
                },
                if manifest.exists() {
                    Some(manifest)
                } else {
                    None
                },
            ))
        }
        System::Hex => {
            let lockfile = root.join("mix.lock");
            let manifest = root.join("mix.exs");
            Ok((
                if lockfile.exists() {
                    Some(lockfile)
                } else {
                    None
                },
                if manifest.exists() {
                    Some(manifest)
                } else {
                    None
                },
            ))
        }
        System::Pub => {
            let lockfile = root.join("pubspec.lock");
            let manifest = root.join("pubspec.yaml");
            Ok((
                if lockfile.exists() {
                    Some(lockfile)
                } else {
                    None
                },
                if manifest.exists() {
                    Some(manifest)
                } else {
                    None
                },
            ))
        }
        _ => Ok((None, None)),
    }
}

/// Convert scanner packages to DependencyGraph format
fn convert_to_dependency_graph(
    system: System,
    packages: &[bazbom_scanner::types::Package],
) -> Result<DependencyGraph> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Create a map of package name to index for edge creation
    let mut pkg_to_idx: HashMap<String, usize> = HashMap::new();

    // First pass: create nodes
    for (idx, pkg) in packages.iter().enumerate() {
        pkg_to_idx.insert(pkg.name.clone(), idx);

        nodes.push(DependencyNode {
            version_key: VersionKey {
                system,
                name: pkg.name.clone(),
                version: pkg.version.clone(),
            },
            relation: if idx == 0 {
                Relation::SelfRelation
            } else {
                Relation::Direct
            },
            errors: vec![],
        });
    }

    // Second pass: create edges from dependencies
    for (from_idx, pkg) in packages.iter().enumerate() {
        for dep_name in &pkg.dependencies {
            if let Some(&to_idx) = pkg_to_idx.get(dep_name) {
                edges.push(DependencyEdge {
                    from_node: from_idx,
                    to_node: to_idx,
                    requirement: String::new(), // We don't have version requirements in Package
                });
            }
        }
    }

    Ok(DependencyGraph { nodes, edges })
}

/// Create an empty dependency graph
fn empty_graph() -> DependencyGraph {
    DependencyGraph {
        nodes: vec![],
        edges: vec![],
    }
}

/// Resolve what dependencies would change after upgrading a package
///
/// This is the key function for transitive upgrade analysis:
/// 1. Get current dependency tree from lockfile
/// 2. Fetch new version's requirements from registry
/// 3. Simulate resolution to find changed transitive deps
pub async fn resolve_upgrade_changes(
    system: System,
    package: &str,
    _from_version: &str,
    to_version: &str,
    project_root: Option<&Path>,
) -> Result<Vec<DependencyChange>> {
    let mut changes = Vec::new();

    match system {
        System::Packagist => {
            // Get current dependencies from lockfile
            let current_deps = if let Some(root) = project_root {
                get_composer_dependencies(root)?
            } else {
                HashMap::new()
            };

            // Get new version's requirements from Packagist
            let parts: Vec<&str> = package.split('/').collect();
            if parts.len() == 2 {
                match bazbom_packagist::get_package_info(parts[0], parts[1]) {
                    Ok(info) => {
                        // Find the target version in the versions HashMap
                        let version_info = info
                            .versions
                            .get(to_version)
                            .or_else(|| info.versions.get(&format!("v{}", to_version)));

                        if let Some(version_info) = version_info {
                            // Compare requirements
                            if let Some(require) = &version_info.require {
                                for (dep_name, constraint) in require {
                                    if dep_name == "php" || dep_name.starts_with("ext-") {
                                        continue;
                                    }

                                    let current_version = current_deps.get(dep_name);

                                    // Check if this is a new or changed dependency
                                    if current_version.is_none() {
                                        changes.push(DependencyChange {
                                            package: dep_name.clone(),
                                            from_version: None,
                                            to_version: Some(constraint.clone()),
                                            change_type: ChangeType::Added,
                                        });
                                    }
                                    // Note: Detecting version changes requires constraint resolution
                                    // which is complex - for now we just detect additions
                                }
                            }

                            // Check for removed dependencies
                            // Would need to compare with from_version's requirements
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch Packagist info: {}", e);
                    }
                }
            }
        }
        System::Hex => {
            // Similar logic for Hex.pm
            // Would need to parse mix.lock and compare with new version's deps
        }
        System::Pub => {
            // Similar logic for pub.dev
            // Would need to parse pubspec.lock and compare with new version's deps
        }
        _ => {}
    }

    Ok(changes)
}

/// A dependency that changed during upgrade
#[derive(Debug, Clone)]
pub struct DependencyChange {
    pub package: String,
    pub from_version: Option<String>,
    pub to_version: Option<String>,
    pub change_type: ChangeType,
}

/// Type of dependency change
#[derive(Debug, Clone)]
pub enum ChangeType {
    Added,
    Removed,
    Updated,
}

/// Get dependencies from composer.lock
fn get_composer_dependencies(root: &Path) -> Result<HashMap<String, String>> {
    let lockfile = root.join("composer.lock");
    if !lockfile.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&lockfile)
        .context("Failed to read composer.lock")?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .context("Failed to parse composer.lock")?;

    let mut deps = HashMap::new();

    if let Some(packages) = json.get("packages").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let (Some(name), Some(version)) = (
                pkg.get("name").and_then(|n| n.as_str()),
                pkg.get("version").and_then(|v| v.as_str()),
            ) {
                deps.insert(
                    name.to_string(),
                    version.trim_start_matches('v').to_string(),
                );
            }
        }
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_dependency_graph() {
        let packages = vec![
            bazbom_scanner::types::Package {
                name: "symfony/console".to_string(),
                version: "5.4.0".to_string(),
                ecosystem: "PHP".to_string(),
                namespace: None,
                dependencies: vec!["symfony/polyfill-php80".to_string()],
                license: None,
                description: None,
                homepage: None,
                repository: None,
            },
            bazbom_scanner::types::Package {
                name: "symfony/polyfill-php80".to_string(),
                version: "1.25.0".to_string(),
                ecosystem: "PHP".to_string(),
                namespace: None,
                dependencies: vec![],
                license: None,
                description: None,
                homepage: None,
                repository: None,
            },
        ];

        let graph = convert_to_dependency_graph(System::Packagist, &packages).unwrap();

        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].from_node, 0usize);
        assert_eq!(graph.edges[0].to_node, 1usize);
    }
}
