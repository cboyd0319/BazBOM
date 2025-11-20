use crate::community_data::CommunityDatabase;
use crate::ecosystem_detection::detect_ecosystem_from_package;
use crate::github::GitHubAnalyzer;
use crate::models::*;
use crate::native_deps;
use crate::semver::analyze_semver_risk;
use anyhow::{Context, Result};
use bazbom_depsdev::{DependencyGraph, DependencyNode, DepsDevClient, Relation, System};
use futures::future::join_all;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Main upgrade analyzer with recursive transitive analysis
pub struct UpgradeAnalyzer {
    deps_dev: DepsDevClient,
    github: GitHubAnalyzer,
    community_db: CommunityDatabase,
    /// Cache for already-analyzed packages to avoid duplicate work
    analysis_cache: HashMap<String, SinglePackageAnalysis>,
    /// Project root for lockfile-based analysis (non-deps.dev ecosystems)
    project_root: Option<PathBuf>,
}

/// Analysis result for a single package (cached)
#[derive(Debug, Clone)]
struct SinglePackageAnalysis {
    breaking_changes: Vec<BreakingChange>,
    risk_level: RiskLevel,
    github_repo: Option<String>,
}

impl UpgradeAnalyzer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            deps_dev: DepsDevClient::new(),
            github: GitHubAnalyzer::new()?,
            community_db: CommunityDatabase::new()?,
            analysis_cache: HashMap::new(),
            project_root: None,
        })
    }

    /// Create analyzer with a project root for lockfile-based analysis
    pub fn with_project_root(mut self, root: PathBuf) -> Self {
        self.project_root = Some(root);
        self
    }

    /// Recursively analyze an upgrade including ALL transitive dependency changes
    ///
    /// This is the main entry point that does the deep analysis.
    pub async fn analyze_upgrade(
        &mut self,
        package: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<UpgradeAnalysis> {
        // Auto-detect ecosystem from package name format
        let system = detect_ecosystem_from_package(package);

        info!(
            "Starting recursive upgrade analysis: {} {} -> {} (ecosystem: {:?})",
            package, from_version, to_version, system
        );

        // 1. Analyze the target package itself
        let direct_analysis = self
            .analyze_single_package(system, package, from_version, to_version)
            .await?;

        // 2. Get dependency graphs for both versions
        let required_upgrades = if system.is_depsdev_supported() {
            // Use deps.dev API for supported ecosystems
            let from_deps = self
                .deps_dev
                .get_dependencies(system, package, from_version)
                .await
                .context("Failed to fetch from_version dependencies")?;

            let to_deps = self
                .deps_dev
                .get_dependencies(system, package, to_version)
                .await
                .context("Failed to fetch to_version dependencies")?;

            // 3. Recursively analyze ALL required dependency upgrades
            self.analyze_dependency_changes(&from_deps, &to_deps, package)
                .await?
        } else if self.project_root.is_some() {
            // Use native lockfile-based analysis for non-deps.dev ecosystems
            info!(
                "Using native lockfile analysis for {:?} ecosystem",
                system
            );

            let _from_deps = native_deps::get_native_dependencies(
                system,
                package,
                from_version,
                self.project_root.as_deref(),
            )
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to get native dependencies: {}", e);
                DependencyGraph {
                    nodes: vec![],
                    edges: vec![],
                }
            });

            // For the "to" version, we need to resolve what would change
            // This is complex - for now, use the change detection
            let changes = native_deps::resolve_upgrade_changes(
                system,
                package,
                from_version,
                to_version,
                self.project_root.as_deref(),
            )
            .await
            .unwrap_or_default();

            // Convert changes to RequiredUpgrade format
            changes
                .into_iter()
                .map(|change| RequiredUpgrade {
                    package: change.package,
                    from_version: change.from_version.unwrap_or_else(|| "none".to_string()),
                    to_version: change.to_version.unwrap_or_else(|| "unknown".to_string()),
                    reason: match change.change_type {
                        native_deps::ChangeType::Added => UpgradeReason::NewDependency,
                        native_deps::ChangeType::Removed => UpgradeReason::Removed,
                        native_deps::ChangeType::Updated => UpgradeReason::VersionAlignment {
                            required_by: package.to_string(),
                        },
                    },
                    breaking_changes: vec![],
                    risk_level: RiskLevel::Low,
                    optional: false,
                })
                .collect()
        } else {
            // No project root and not deps.dev supported - basic analysis only
            debug!(
                "Transitive dependency analysis not available for {:?} - using semver-only",
                system
            );
            vec![]
        };

        // 4. Calculate overall risk
        let overall_risk = self.calculate_overall_risk(
            direct_analysis.risk_level,
            &direct_analysis.breaking_changes,
            &required_upgrades,
        );

        // 5. Generate compatibility notes
        let compatibility_notes = self.generate_compatibility_notes(
            package,
            from_version,
            to_version,
            &required_upgrades,
        );

        // 6. Estimate effort
        let estimated_effort_hours = self.estimate_effort(
            direct_analysis.risk_level,
            direct_analysis.breaking_changes.len(),
            &required_upgrades,
        );

        // 7. Try to find migration guide
        let migration_guide_url = if let Some(ref repo) = direct_analysis.github_repo {
            self.github
                .find_migration_guide(repo, to_version)
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // Query community database for success rate
        let success_rate = self
            .community_db
            .get_success_rate(package, from_version, to_version);

        Ok(UpgradeAnalysis {
            target_package: package.to_string(),
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            direct_breaking_changes: direct_analysis.breaking_changes,
            required_upgrades,
            overall_risk,
            estimated_effort_hours,
            github_repo: direct_analysis.github_repo,
            migration_guide_url,
            compatibility_notes,
            success_rate,
        })
    }

    /// Analyze a single package (non-recursive, may use cache)
    async fn analyze_single_package(
        &mut self,
        system: System,
        package: &str,
        from_version: &str,
        to_version: &str,
    ) -> Result<SinglePackageAnalysis> {
        let cache_key = format!("{}:{}:{}", package, from_version, to_version);

        // Check cache first
        if let Some(cached) = self.analysis_cache.get(&cache_key) {
            debug!("Using cached analysis for {}", cache_key);
            return Ok(cached.clone());
        }

        debug!(
            "Analyzing package: {} {} -> {}",
            package, from_version, to_version
        );

        // 1. Get semver-based risk
        let semver_risk = analyze_semver_risk(from_version, to_version);

        // 2. Try to find GitHub repo - use deps.dev for supported ecosystems,
        //    or native crates for others
        let github_repo = if system.is_depsdev_supported() {
            self.deps_dev
                .find_github_repo(system, package, to_version)
                .await
                .ok()
                .flatten()
        } else {
            // For non-deps.dev ecosystems, try to get repo from native crates
            self.find_github_repo_native(system, package).await
        };

        // 3. If we have a GitHub repo, fetch breaking changes from release notes
        let breaking_changes = if let Some(ref repo) = github_repo {
            match self
                .github
                .analyze_upgrade(repo, from_version, to_version)
                .await
            {
                Ok(changes) => changes,
                Err(e) => {
                    warn!("Failed to fetch GitHub release notes: {}", e);
                    vec![]
                }
            }
        } else {
            vec![]
        };

        // 4. Determine risk level
        let risk_level = if !breaking_changes.is_empty() {
            // If we found breaking changes, risk is at least Medium
            semver_risk.max(RiskLevel::Medium)
        } else {
            semver_risk
        };

        let analysis = SinglePackageAnalysis {
            breaking_changes,
            risk_level,
            github_repo,
        };

        // Cache the result
        self.analysis_cache.insert(cache_key, analysis.clone());

        Ok(analysis)
    }

    /// Recursively analyze ALL dependency changes
    async fn analyze_dependency_changes(
        &mut self,
        from_deps: &DependencyGraph,
        to_deps: &DependencyGraph,
        parent_package: &str,
    ) -> Result<Vec<RequiredUpgrade>> {
        let mut required_upgrades = Vec::new();

        // Create maps for easier lookup
        let from_map: HashMap<String, &DependencyNode> = from_deps
            .nodes
            .iter()
            .filter(|n| n.relation != Relation::SelfRelation)
            .map(|n| (n.version_key.name.clone(), n))
            .collect();

        let to_map: HashMap<String, &DependencyNode> = to_deps
            .nodes
            .iter()
            .filter(|n| n.relation != Relation::SelfRelation)
            .map(|n| (n.version_key.name.clone(), n))
            .collect();

        // Find all changed or new dependencies
        let mut analysis_futures = vec![];

        for (dep_name, to_node) in &to_map {
            match from_map.get(dep_name) {
                Some(from_node) if from_node.version_key.version != to_node.version_key.version => {
                    // Dependency version changed - analyze it!
                    let from_ver = from_node.version_key.version.clone();
                    let to_ver = to_node.version_key.version.clone();
                    let pkg = dep_name.clone();
                    let parent = parent_package.to_string();

                    analysis_futures.push(async move {
                        (
                            pkg,
                            from_ver,
                            to_ver,
                            UpgradeReason::VersionAlignment {
                                required_by: parent,
                            },
                        )
                    });
                }
                None => {
                    // New dependency added
                    let to_ver = to_node.version_key.version.clone();
                    required_upgrades.push(RequiredUpgrade {
                        package: dep_name.clone(),
                        from_version: "none".to_string(),
                        to_version: to_ver,
                        reason: UpgradeReason::NewDependency,
                        breaking_changes: vec![],
                        risk_level: RiskLevel::Low,
                        optional: false,
                    });
                }
                _ => {
                    // Version unchanged, skip
                }
            }
        }

        // Check for removed dependencies
        for (dep_name, from_node) in &from_map {
            if !to_map.contains_key(dep_name) {
                required_upgrades.push(RequiredUpgrade {
                    package: dep_name.clone(),
                    from_version: from_node.version_key.version.clone(),
                    to_version: "removed".to_string(),
                    reason: UpgradeReason::Removed,
                    breaking_changes: vec![BreakingChange {
                        description: format!("Dependency {} was removed", dep_name),
                        version: "N/A".to_string(),
                        auto_fixable: false,
                        affected_apis: vec![],
                        migration_hint: None,
                    }],
                    risk_level: RiskLevel::High,
                    optional: false,
                });
            }
        }

        // Execute all analyses in parallel
        let futures_to_analyze = analysis_futures
            .into_iter()
            .map(|future| async move {
                let (pkg, from_ver, to_ver, reason) = future.await;
                (pkg, from_ver, to_ver, reason)
            })
            .collect::<Vec<_>>();

        let results = join_all(futures_to_analyze).await;

        // Now analyze each changed dependency
        for (pkg, from_ver, to_ver, reason) in results {
            // Detect ecosystem for each dependency (don't assume Maven!)
            let dep_system = detect_ecosystem_from_package(&pkg);

            match self
                .analyze_single_package(dep_system, &pkg, &from_ver, &to_ver)
                .await
            {
                Ok(analysis) => {
                    required_upgrades.push(RequiredUpgrade {
                        package: pkg,
                        from_version: from_ver,
                        to_version: to_ver,
                        reason,
                        breaking_changes: analysis.breaking_changes,
                        risk_level: analysis.risk_level,
                        optional: false,
                    });
                }
                Err(e) => {
                    warn!("Failed to analyze dependency {}: {}", pkg, e);
                }
            }
        }

        Ok(required_upgrades)
    }

    /// Calculate overall risk considering direct + transitive changes
    fn calculate_overall_risk(
        &self,
        direct_risk: RiskLevel,
        direct_changes: &[BreakingChange],
        required_upgrades: &[RequiredUpgrade],
    ) -> RiskLevel {
        let mut max_risk = direct_risk;

        // Factor in direct breaking changes
        if !direct_changes.is_empty() && max_risk < RiskLevel::High {
            max_risk = RiskLevel::High;
        }

        // Factor in transitive breaking changes
        for upgrade in required_upgrades {
            if !upgrade.breaking_changes.is_empty() {
                max_risk = max_risk.max(RiskLevel::Medium);
            }
            max_risk = max_risk.max(upgrade.risk_level);
        }

        // Check for removed dependencies (critical)
        if required_upgrades
            .iter()
            .any(|u| matches!(u.reason, UpgradeReason::Removed))
        {
            max_risk = max_risk.max(RiskLevel::Critical);
        }

        max_risk
    }

    /// Generate compatibility notes
    fn generate_compatibility_notes(
        &self,
        _package: &str,
        _from_version: &str,
        _to_version: &str,
        required_upgrades: &[RequiredUpgrade],
    ) -> Vec<String> {
        let mut notes = Vec::new();

        // Count breaking changes
        let total_breaking = required_upgrades
            .iter()
            .map(|u| u.breaking_changes.len())
            .sum::<usize>();

        if total_breaking > 0 {
            notes.push(format!(
                "{} transitive breaking changes detected across {} dependencies",
                total_breaking,
                required_upgrades.len()
            ));
        }

        // Check for removed dependencies
        let removed_count = required_upgrades
            .iter()
            .filter(|u| matches!(u.reason, UpgradeReason::Removed))
            .count();

        if removed_count > 0 {
            notes.push(format!(
                "⚠️  {} dependencies will be removed",
                removed_count
            ));
        }

        // Check for new dependencies
        let new_count = required_upgrades
            .iter()
            .filter(|u| matches!(u.reason, UpgradeReason::NewDependency))
            .count();

        if new_count > 0 {
            notes.push(format!("{} new dependencies will be added", new_count));
        }

        notes
    }

    /// Estimate effort in hours
    fn estimate_effort(
        &self,
        direct_risk: RiskLevel,
        direct_breaking_count: usize,
        required_upgrades: &[RequiredUpgrade],
    ) -> f32 {
        let mut hours = 0.5; // Base overhead

        // Factor in direct risk
        hours += match direct_risk {
            RiskLevel::Low => 0.25,
            RiskLevel::Medium => 1.0,
            RiskLevel::High => 4.0,
            RiskLevel::Critical => 8.0,
        };

        // Factor in direct breaking changes (30 min per change)
        hours += (direct_breaking_count as f32) * 0.5;

        // Factor in transitive upgrades
        for upgrade in required_upgrades {
            hours += match upgrade.risk_level {
                RiskLevel::Low => 0.1,
                RiskLevel::Medium => 0.5,
                RiskLevel::High => 2.0,
                RiskLevel::Critical => 4.0,
            };

            // Add time for each breaking change
            hours += (upgrade.breaking_changes.len() as f32) * 0.5;
        }

        // Round to nearest 0.25 hours
        (hours * 4.0).round() / 4.0
    }

    /// Find GitHub repository URL for non-deps.dev ecosystems
    async fn find_github_repo_native(&self, system: System, package: &str) -> Option<String> {
        match system {
            System::Packagist => {
                // Parse vendor/package format
                let parts: Vec<&str> = package.split('/').collect();
                if parts.len() == 2 {
                    match bazbom_packagist::get_package_info(parts[0], parts[1]) {
                        Ok(info) => {
                            // Look for GitHub URL in repository field
                            if let Some(repo) = info.repository {
                                if repo.contains("github.com") {
                                    return extract_github_repo(&repo);
                                }
                            }
                        }
                        Err(e) => {
                            debug!("Failed to fetch Packagist info for {}: {}", package, e);
                        }
                    }
                }
                None
            }
            System::Hex => {
                match bazbom_hex::get_package_info(package) {
                    Ok(info) => {
                        // Check links for GitHub
                        if let Some(github) = info.meta.links.get("GitHub") {
                            return extract_github_repo(github);
                        }
                        if let Some(github) = info.meta.links.get("github") {
                            return extract_github_repo(github);
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch Hex info for {}: {}", package, e);
                    }
                }
                None
            }
            System::Pub => {
                match bazbom_pub::get_package_info(package) {
                    Ok(info) => {
                        // Check repository in pubspec
                        if let Some(repo) = info.latest.pubspec.repository {
                            if repo.contains("github.com") {
                                return extract_github_repo(&repo);
                            }
                        }
                        // Check homepage as fallback
                        if let Some(home) = info.latest.pubspec.homepage {
                            if home.contains("github.com") {
                                return extract_github_repo(&home);
                            }
                        }
                    }
                    Err(e) => {
                        debug!("Failed to fetch pub.dev info for {}: {}", package, e);
                    }
                }
                None
            }
            // OS packages don't typically have GitHub repos
            System::Alpine | System::Debian | System::Rpm => None,
            // deps.dev supported - shouldn't reach here
            _ => None,
        }
    }
}

/// Extract GitHub owner/repo from URL
fn extract_github_repo(url: &str) -> Option<String> {
    // Handle various GitHub URL formats
    let url = url.trim_end_matches('/').trim_end_matches(".git");

    if let Some(rest) = url.strip_prefix("https://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    if let Some(rest) = url.strip_prefix("git://github.com/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return Some(format!("{}/{}", parts[0], parts[1]));
        }
    }

    None
}

impl Default for UpgradeAnalyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create UpgradeAnalyzer")
    }
}
