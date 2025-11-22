//! Vulnerability enrichment with KEV, EPSS, and priority scoring
//!
//! This module leverages bazbom-vulnerabilities for:
//! - EPSS score loading (full database)
//! - CISA KEV catalog loading
//! - Priority level calculation (P0-P4)
//!
//! Container-specific features:
//! - Remediation difficulty scoring
//! - Upgrade impact analysis
//! - OSV severity fallback for UNKNOWN

use super::types::{detect_ecosystem, PackageEcosystem, VulnerabilityInfo};
use anyhow::Result;
use colored::*;
use std::collections::HashMap;

// Use existing BazBOM shared infrastructure
use bazbom_core::cache_dir;
use bazbom_vulnerabilities::{
    calculate_priority, db_sync, fetch_osv_severities_with_hint, load_epss_scores,
    load_kev_catalog, EpssScore, KevEntry, Severity, SeverityLevel,
};

/// Enrich vulnerabilities with EPSS, KEV data, and OSV severity fallback
pub(crate) async fn enrich_vulnerabilities(vulns: &mut [VulnerabilityInfo]) -> Result<()> {
    enrich_vulnerabilities_with_os(vulns, None).await
}

/// Enrich vulnerabilities with optional OS hint for faster OSV lookups
pub(crate) async fn enrich_vulnerabilities_with_os(
    vulns: &mut [VulnerabilityInfo],
    os_hint: Option<&str>,
) -> Result<()> {
    // Use BazBOM's shared cache directory
    let cache = cache_dir();

    // Sync EPSS and KEV databases (downloads if needed)
    tracing::info!("Syncing vulnerability databases...");
    if let Err(e) = db_sync(&cache, false) {
        tracing::warn!("Failed to sync databases: {}", e);
    }

    // Load EPSS scores from cached file
    let epss_path = cache.join("advisories/epss.csv");
    let epss_map = match load_epss_scores(&epss_path) {
        Ok(map) => {
            tracing::info!("Loaded EPSS data for {} CVEs", map.len());
            map
        }
        Err(e) => {
            tracing::warn!("Failed to load EPSS data: {}", e);
            HashMap::new()
        }
    };

    // Load CISA KEV catalog from cached file
    let kev_path = cache.join("advisories/kev.json");
    let kev_map = match load_kev_catalog(&kev_path) {
        Ok(map) => {
            tracing::info!("Loaded CISA KEV data for {} CVEs", map.len());
            map
        }
        Err(e) => {
            tracing::warn!("Failed to load KEV data: {}", e);
            HashMap::new()
        }
    };

    // Track CVEs with unknown severity for OSV lookup
    let mut unknown_severity_cves: Vec<String> = Vec::new();

    for vuln in vulns.iter_mut() {
        // Enrich with EPSS
        if let Some(epss_score) = epss_map.get(&vuln.cve_id) {
            vuln.epss_score = Some(epss_score.score);
            vuln.epss_percentile = Some(epss_score.percentile);
        }

        // Enrich with KEV
        if let Some(kev_entry) = kev_map.get(&vuln.cve_id) {
            vuln.is_kev = true;
            vuln.kev_due_date = Some(kev_entry.due_date.clone());
        }

        // Track UNKNOWN severity for OSV lookup (only actual CVE IDs, not DLA/DSA/USN advisories)
        if vuln.severity == "UNKNOWN" && vuln.cve_id.starts_with("CVE-") {
            unknown_severity_cves.push(vuln.cve_id.clone());
        }
    }

    // Query OSV for severity of UNKNOWN vulns (using shared function)
    if !unknown_severity_cves.is_empty() {
        tracing::info!(
            "Looking up severity for {} UNKNOWN vulnerabilities via OSV...",
            unknown_severity_cves.len()
        );
        let osv_severities = fetch_osv_severities_with_hint(&unknown_severity_cves, os_hint);

        for vuln in vulns.iter_mut() {
            if vuln.severity == "UNKNOWN" {
                if let Some(severity) = osv_severities.get(&vuln.cve_id) {
                    vuln.severity = severity.clone();
                }
            }
        }
    }

    // Calculate priority and difficulty for all vulns
    for vuln in vulns.iter_mut() {
        // Use shared priority calculation from bazbom-vulnerabilities
        let severity = Some(string_to_severity(&vuln.severity, vuln.cvss_score));
        let kev: Option<KevEntry> = if vuln.is_kev {
            // Create a minimal KevEntry if we have KEV status
            Some(KevEntry {
                cve_id: vuln.cve_id.clone(),
                vendor_project: String::new(),
                product: String::new(),
                vulnerability_name: String::new(),
                date_added: vuln.kev_due_date.clone().unwrap_or_default(),
                required_action: String::new(),
                due_date: vuln.kev_due_date.clone().unwrap_or_default(),
            })
        } else {
            None
        };
        let epss: Option<EpssScore> = vuln.epss_score.map(|score| EpssScore {
            score,
            percentile: vuln.epss_percentile.unwrap_or(0.0),
        });

        let priority = calculate_priority(&severity, &kev, &epss);
        vuln.priority = Some(priority.to_string());

        // Calculate remediation difficulty score (0-100)
        vuln.difficulty_score = Some(calculate_difficulty_score(vuln));
    }

    Ok(())
}

/// Convert severity string to Severity struct for priority calculation
fn string_to_severity(severity_str: &str, cvss: Option<f64>) -> Severity {
    let level = match severity_str.to_uppercase().as_str() {
        "CRITICAL" => SeverityLevel::Critical,
        "HIGH" => SeverityLevel::High,
        "MEDIUM" => SeverityLevel::Medium,
        "LOW" => SeverityLevel::Low,
        _ => SeverityLevel::Unknown,
    };
    Severity {
        cvss_v3: cvss,
        cvss_v4: None,
        level,
    }
}

/// Calculate remediation difficulty score (0-100)
///
/// Factors:
/// - No fix available: 100 (impossible)
/// - Breaking changes (major version): +40
/// - Major version jumps: +15 per jump
/// - Framework migrations: +25
/// - Base: 10 (simple patch)
pub(crate) fn calculate_difficulty_score(vuln: &VulnerabilityInfo) -> u8 {
    // No fix = impossible
    if vuln.fixed_version.is_none() {
        return 100;
    }

    let mut score = 10u8; // Base difficulty for any upgrade

    // Breaking changes add significant complexity
    if vuln.breaking_change == Some(true) {
        score = score.saturating_add(40);
    }

    // Calculate version jumps (e.g., 1.0 -> 4.0 = 3 major jumps)
    if let Some(ref fixed) = vuln.fixed_version {
        let major_jumps = count_major_version_jumps(&vuln.installed_version, fixed);
        score = score.saturating_add(major_jumps.saturating_mul(15));
    }

    // Framework-specific complexity (requires migration guides)
    if is_framework_package(&vuln.package_name) {
        score = score.saturating_add(25);
    }

    score.min(95) // Cap at 95 (100 reserved for "no fix")
}

/// Count major version jumps (e.g., 1.2.3 -> 4.5.6 = 3 jumps)
fn count_major_version_jumps(from: &str, to: &str) -> u8 {
    let parse_major = |v: &str| -> Option<u32> { v.split('.').next()?.parse().ok() };

    match (parse_major(from), parse_major(to)) {
        (Some(from_major), Some(to_major)) if to_major > from_major => {
            (to_major - from_major).min(6) as u8 // Cap at 6 jumps
        }
        _ => 0,
    }
}

/// Check if package is a major framework requiring migration guides
fn is_framework_package(name: &str) -> bool {
    let frameworks = [
        "spring-boot",
        "spring-core",
        "spring-framework",
        "django",
        "flask",
        "rails",
        "railties",
        "react",
        "react-dom",
        "vue",
        "vuejs",
        "angular",
        "@angular/core",
        "express",
        "laravel",
        "symfony",
    ];

    frameworks.iter().any(|f| name.contains(f))
}

/// Format difficulty score as a colored label
pub(crate) fn format_difficulty_label(score: u8) -> ColoredString {
    let (emoji, label, color_fn): (&str, &str, fn(&str) -> ColoredString) = match score {
        0..=20 => ("🟢", "Trivial", |s| s.green()),
        21..=40 => ("🟡", "Easy", |s| s.yellow()),
        41..=60 => ("🟠", "Moderate", |s| s.bright_yellow()),
        61..=80 => ("🔴", "Hard", |s| s.red()),
        81..=99 => ("WARN", "Very Hard", |s| s.bright_red().bold()),
        100 => ("NO", "No Fix Available", |s| s.bright_red().bold()),
        _ => ("❓", "Unknown", |s| s.dimmed()),
    };

    color_fn(&format!("Difficulty: {} {} ({}/100)", emoji, label, score))
}

/// Get framework-specific migration guide URL
pub(crate) fn get_framework_migration_guide(
    package: &str,
    current_major: u32,
    fixed_major: u32,
) -> Option<String> {
    // Spring Boot major version migrations
    if package.starts_with("org.springframework.boot") || package.contains("spring-boot") {
        return match (current_major, fixed_major) {
            (2, 3) => Some("Spring Boot 2→3 requires Java 17+. Migration guide: https://github.com/spring-projects/spring-boot/wiki/Spring-Boot-3.0-Migration-Guide".to_string()),
            (1, 2) => Some("Spring Boot 1→2 has significant changes. Guide: https://github.com/spring-projects/spring-boot/wiki/Spring-Boot-2.0-Migration-Guide".to_string()),
            _ => None,
        };
    }

    // Django major versions
    if package == "django" || package.starts_with("Django") {
        return match (current_major, fixed_major) {
            (4, 5) => Some("Django 4→5 removes deprecated features. Guide: https://docs.djangoproject.com/en/5.0/howto/upgrade-version/".to_string()),
            (3, 4) => Some("Django 3→4 removes deprecated features. Guide: https://docs.djangoproject.com/en/4.0/howto/upgrade-version/".to_string()),
            (2, 3) => Some("Django 2→3 drops Python 2 support. Requires Python 3.6+".to_string()),
            _ => None,
        };
    }

    // Rails
    if package == "rails" || package.starts_with("rails") {
        return match (current_major, fixed_major) {
            (6, 7) => Some("Rails 6→7 requires Ruby 2.7+. Guide: https://guides.rubyonrails.org/upgrading_ruby_on_rails.html".to_string()),
            (5, 6) => Some("Rails 5→6 requires Ruby 2.5+. Guide: https://guides.rubyonrails.org/upgrading_ruby_on_rails.html".to_string()),
            _ => None,
        };
    }

    // React
    if package == "react" {
        return match (current_major, fixed_major) {
            (17, 18) => Some(
                "React 17→18 introduces concurrent features. May need createRoot() migration"
                    .to_string(),
            ),
            (16, 17) => Some(
                "React 16→17 is a stepping stone release. Minimal breaking changes".to_string(),
            ),
            _ => None,
        };
    }

    // Vue
    if package == "vue" || package.starts_with("@vue/") {
        return match (current_major, fixed_major) {
            (2, 3) => Some(
                "Vue 2→3 has major API changes. Migration guide: https://v3-migration.vuejs.org/"
                    .to_string(),
            ),
            _ => None,
        };
    }

    // Angular
    if package.starts_with("@angular/") {
        return match (current_major, fixed_major) {
            (v1, v2) if v2 > v1 => Some(format!(
                "Angular {}→{} migration guide: https://update.angular.io/",
                v1, v2
            )),
            _ => None,
        };
    }

    // Express
    if package == "express" {
        return match (current_major, fixed_major) {
            (4, 5) => {
                Some("Express 4→5 has breaking changes in middleware and routing".to_string())
            }
            _ => None,
        };
    }

    // Go modules
    if (package.starts_with("github.com/") || package.starts_with("golang.org/"))
        && fixed_major >= 2
        && fixed_major > current_major
    {
        return Some(format!(
            "Go module major version {}→{}. Update import paths to include /v{}",
            current_major, fixed_major, fixed_major
        ));
    }

    None
}

/// Get ecosystem-specific version semantics explanation
pub(crate) fn get_ecosystem_version_semantics(package: &str) -> Option<&'static str> {
    let ecosystem = detect_ecosystem(package);

    match ecosystem {
        PackageEcosystem::Python => {
            Some("Python packages don't always follow semver strictly. Check changelog carefully.")
        }
        PackageEcosystem::JavaScript => {
            Some("npm packages follow semver. Major = breaking changes.")
        }
        PackageEcosystem::Go => {
            Some("Go modules use v2+ for breaking changes (import path changes).")
        }
        PackageEcosystem::Rust => {
            Some("Rust crates follow semver. Pre-1.0 allows breaking changes in minor versions.")
        }
        PackageEcosystem::Ruby => Some("Ruby gems generally follow semver for version 1.0+."),
        PackageEcosystem::Php => Some("PHP/Composer packages typically follow semver."),
        PackageEcosystem::Java => {
            Some("Java packages typically follow semver. Check for API deprecations.")
        }
        PackageEcosystem::Other => None,
    }
}

/// Enhanced upgrade impact analysis with framework-specific knowledge
pub(crate) fn analyze_upgrade_impact(
    package: &str,
    current: &str,
    fixed: &str,
) -> (Option<bool>, Option<String>) {
    let current_parts: Vec<&str> = current.split('.').collect();
    let fixed_parts: Vec<&str> = fixed.split('.').collect();

    if current_parts.is_empty() || fixed_parts.is_empty() {
        return (None, None);
    }

    let current_major = current_parts[0].parse::<u32>().ok();
    let fixed_major = fixed_parts[0].parse::<u32>().ok();
    let ecosystem = detect_ecosystem(package);

    if let (Some(cur), Some(fix)) = (current_major, fixed_major) {
        if fix > cur {
            // Check for framework-specific migration guides
            if let Some(guide) = get_framework_migration_guide(package, cur, fix) {
                return (Some(true), Some(guide));
            }

            // Special case: Pre-1.0 Rust crates
            if ecosystem == PackageEcosystem::Rust && cur == 0 {
                return (
                    Some(true),
                    Some(format!(
                        "Pre-1.0 Rust crate: {}→{} may have breaking changes in minor versions",
                        current, fixed
                    )),
                );
            }

            // Special case: Go v2+ module versioning
            if ecosystem == PackageEcosystem::Go && fix >= 2 {
                return (
                    Some(true),
                    Some(format!(
                        "Go module major version {}→{}. Update import paths to /v{}",
                        cur, fix, fix
                    )),
                );
            }

            // Generic major version upgrade with ecosystem context
            let mut msg = format!(
                "Major version upgrade {}→{} may require code changes",
                cur, fix
            );
            if let Some(semantics) = get_ecosystem_version_semantics(package) {
                msg.push_str(&format!(". {}", semantics));
            }
            return (Some(true), Some(msg));
        } else if fix == cur && fixed_parts.len() > 1 && current_parts.len() > 1 {
            // Minor version change
            let current_minor = current_parts[1].parse::<u32>().ok();
            let fixed_minor = fixed_parts[1].parse::<u32>().ok();
            if let (Some(cur_min), Some(fix_min)) = (current_minor, fixed_minor) {
                // Special handling for pre-1.0 Rust crates where minor = breaking
                if ecosystem == PackageEcosystem::Rust && cur == 0 && fix_min > cur_min {
                    return (
                        Some(true),
                        Some(format!(
                            "Pre-1.0 Rust: 0.{}→0.{} may contain breaking changes",
                            cur_min, fix_min
                        )),
                    );
                }

                if fix_min > cur_min + 5 {
                    return (
                        Some(false),
                        Some(format!(
                            "Minor version jump {}.{}→{}.{} - review changelog",
                            cur, cur_min, fix, fix_min
                        )),
                    );
                }
            }
            return (Some(false), Some("Patch update - low risk".to_string()));
        }
    }

    (None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_vuln(
        package: &str,
        installed: &str,
        fixed: Option<&str>,
        breaking: Option<bool>,
    ) -> VulnerabilityInfo {
        VulnerabilityInfo {
            cve_id: "CVE-2024-1234".to_string(),
            package_name: package.to_string(),
            installed_version: installed.to_string(),
            fixed_version: fixed.map(String::from),
            severity: "HIGH".to_string(),
            cvss_score: Some(7.5),
            title: "Test vulnerability".to_string(),
            description: "Test description".to_string(),
            layer_digest: "sha256:abc123".to_string(),
            published_date: None,
            epss_score: None,
            epss_percentile: None,
            is_kev: false,
            kev_due_date: None,
            priority: None,
            references: vec![],
            breaking_change: breaking,
            upgrade_path: None,
            is_reachable: false,
            difficulty_score: None,
            call_chain: None,
            dependency_path: None,
        }
    }

    #[test]
    fn test_calculate_difficulty_score_no_fix() {
        let vuln = make_test_vuln("pkg", "1.0.0", None, None);
        assert_eq!(calculate_difficulty_score(&vuln), 100);
    }

    #[test]
    fn test_calculate_difficulty_score_patch() {
        let vuln = make_test_vuln("pkg", "1.0.0", Some("1.0.1"), None);
        assert_eq!(calculate_difficulty_score(&vuln), 10); // Base only
    }

    #[test]
    fn test_calculate_difficulty_score_major_jump() {
        let vuln = make_test_vuln("pkg", "1.0.0", Some("2.0.0"), None);
        assert_eq!(calculate_difficulty_score(&vuln), 25); // 10 + 15
    }

    #[test]
    fn test_calculate_difficulty_score_breaking_change() {
        let vuln = make_test_vuln("pkg", "1.0.0", Some("1.0.1"), Some(true));
        assert_eq!(calculate_difficulty_score(&vuln), 50); // 10 + 40
    }

    #[test]
    fn test_calculate_difficulty_score_framework() {
        let vuln = make_test_vuln("spring-boot-starter", "1.0.0", Some("1.0.1"), None);
        assert_eq!(calculate_difficulty_score(&vuln), 35); // 10 + 25 (framework)
    }

    #[test]
    fn test_calculate_difficulty_score_capped() {
        // Multiple major jumps + breaking + framework should cap at 95
        let vuln = make_test_vuln("spring-boot", "1.0.0", Some("5.0.0"), Some(true));
        let score = calculate_difficulty_score(&vuln);
        assert!(score <= 95);
    }

    #[test]
    fn test_get_framework_migration_guide_spring() {
        let guide = get_framework_migration_guide("spring-boot-starter", 2, 3);
        assert!(guide.is_some());
        assert!(guide.unwrap().contains("Spring Boot 2→3"));
    }

    #[test]
    fn test_get_framework_migration_guide_django() {
        let guide = get_framework_migration_guide("django", 3, 4);
        assert!(guide.is_some());
        assert!(guide.unwrap().contains("Django 3→4"));
    }

    #[test]
    fn test_get_framework_migration_guide_none() {
        let guide = get_framework_migration_guide("random-package", 1, 2);
        assert!(guide.is_none());
    }

    #[test]
    fn test_get_ecosystem_version_semantics() {
        assert!(get_ecosystem_version_semantics("requests").is_some()); // Python
        assert!(get_ecosystem_version_semantics("express").is_some()); // JavaScript
        assert!(get_ecosystem_version_semantics("github.com/foo/bar").is_some()); // Go
        assert!(get_ecosystem_version_semantics("serde").is_some()); // Rust
    }

    #[test]
    fn test_analyze_upgrade_impact_major() {
        let (breaking, note) = analyze_upgrade_impact("pkg", "1.2.3", "2.0.0");
        assert_eq!(breaking, Some(true));
        assert!(note.is_some());
    }

    #[test]
    fn test_analyze_upgrade_impact_patch() {
        let (breaking, note) = analyze_upgrade_impact("pkg", "1.2.3", "1.2.4");
        assert_eq!(breaking, Some(false));
        assert!(note.unwrap().contains("Patch update"));
    }

    #[test]
    fn test_analyze_upgrade_impact_go_module() {
        let (breaking, note) = analyze_upgrade_impact("github.com/foo/bar", "1.0.0", "2.0.0");
        assert_eq!(breaking, Some(true));
        assert!(note.unwrap().contains("import paths"));
    }

    #[test]
    fn test_analyze_upgrade_impact_rust_pre_1() {
        let (breaking, note) = analyze_upgrade_impact("serde", "0.8.0", "0.9.0");
        assert_eq!(breaking, Some(true));
        assert!(note.unwrap().contains("Pre-1.0 Rust"));
    }

    #[test]
    fn test_format_difficulty_label_trivial() {
        let label = format_difficulty_label(10);
        let s = label.to_string();
        assert!(s.contains("Trivial"));
    }

    #[test]
    fn test_format_difficulty_label_no_fix() {
        let label = format_difficulty_label(100);
        let s = label.to_string();
        assert!(s.contains("No Fix"));
    }
}
