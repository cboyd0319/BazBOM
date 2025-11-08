// Suggestion generation for remediation

use bazbom_advisories::Vulnerability;
use bazbom_core::BuildSystem;

use super::types::{RemediationReport, RemediationSuggestion, RemediationSummary};
use super::version::parse_semantic_version;
use crate::enrich::depsdev::{BreakingChanges, DepsDevClient};

/// Generate remediation suggestions from vulnerabilities
pub fn generate_suggestions(
    vulnerabilities: &[Vulnerability],
    build_system: BuildSystem,
) -> RemediationReport {
    let mut suggestions = Vec::new();
    let mut fixable = 0;
    let mut unfixable = 0;

    for vuln in vulnerabilities {
        for affected in &vuln.affected {
            let package_name = &affected.package;
            let current_version = "unknown";

            // Determine fixed version from ranges
            let fixed_version = affected.ranges.iter().find_map(|r| {
                r.events.iter().find_map(|e| match e {
                    bazbom_advisories::VersionEvent::Fixed { fixed } => Some(fixed.clone()),
                    _ => None,
                })
            });

            let is_fixable = fixed_version.is_some();
            if is_fixable {
                fixable += 1;
            } else {
                unfixable += 1;
            }

            let severity = vuln
                .severity
                .as_ref()
                .map(|s| format!("{:?}", s.level))
                .unwrap_or_else(|| "UNKNOWN".to_string());

            let priority = vuln
                .priority
                .as_ref()
                .map(|p| format!("{:?}", p))
                .unwrap_or_else(|| "P4".to_string());

            let why_fix = generate_why_fix(vuln);
            let how_to_fix =
                generate_how_to_fix(package_name, current_version, &fixed_version, build_system);
            let breaking_changes =
                generate_breaking_changes_warning(package_name, current_version, &fixed_version);

            let references = vuln.references.iter().map(|r| r.url.clone()).collect();

            suggestions.push(RemediationSuggestion {
                vulnerability_id: vuln.id.clone(),
                affected_package: package_name.clone(),
                current_version: current_version.to_string(),
                fixed_version,
                severity,
                priority,
                why_fix,
                how_to_fix,
                breaking_changes,
                references,
            });
        }
    }

    let estimated_effort = estimate_effort(fixable);

    RemediationReport {
        summary: RemediationSummary {
            total_vulnerabilities: vulnerabilities.len(),
            fixable,
            unfixable,
            estimated_effort,
        },
        suggestions,
    }
}

fn estimate_effort(fixable: usize) -> String {
    if fixable == 0 {
        "No immediate fixes available".to_string()
    } else if fixable <= 5 {
        "Low (< 1 hour)".to_string()
    } else if fixable <= 20 {
        "Medium (1-4 hours)".to_string()
    } else {
        "High (> 4 hours)".to_string()
    }
}

/// Enrich remediation suggestions with deps.dev breaking changes information
pub fn enrich_with_depsdev(
    mut report: RemediationReport,
    depsdev_client: &DepsDevClient,
) -> RemediationReport {
    for suggestion in &mut report.suggestions {
        if let Some(ref fixed_version) = suggestion.fixed_version {
            if let Some(purl) = construct_purl(&suggestion.affected_package, fixed_version) {
                if let Ok(package_info) = depsdev_client.get_package_info(&purl) {
                    if let Some(breaking_changes) = package_info.breaking_changes {
                        suggestion.breaking_changes =
                            Some(format_breaking_changes(&breaking_changes));

                        if let Some(url) = breaking_changes.changelog_url {
                            if !suggestion.references.contains(&url) {
                                suggestion.references.push(url);
                            }
                        }

                        if let Some(url) = breaking_changes.migration_guide_url {
                            if !suggestion.references.contains(&url) {
                                suggestion.references.push(url);
                            }
                        }
                    }
                }
            }
        }
    }
    report
}

/// Format breaking changes for display in remediation suggestions
fn format_breaking_changes(breaking_changes: &BreakingChanges) -> String {
    let mut output = String::new();

    if let Some(ref summary) = breaking_changes.summary {
        output.push_str(summary);
        output.push_str("\n\n");
    }

    if !breaking_changes.details.is_empty() {
        output.push_str("Breaking changes details:\n");
        for detail in &breaking_changes.details {
            output.push_str("  - ");
            output.push_str(detail);
            output.push('\n');
        }
        output.push('\n');
    }

    if let Some(ref url) = breaking_changes.changelog_url {
        output.push_str(&format!("Changelog: {}\n", url));
    }

    if let Some(ref url) = breaking_changes.migration_guide_url {
        output.push_str(&format!("Migration guide: {}\n", url));
    }

    output
}

/// Construct a PURL from package name and version
/// This is a heuristic approach - ideally PURLs would come from the SBOM
fn construct_purl(package_name: &str, version: &str) -> Option<String> {
    if package_name.contains('/') || package_name.contains('.') {
        let maven_name = if package_name.contains('/') {
            package_name.to_string()
        } else {
            let parts: Vec<&str> = package_name.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                format!("{}/{}", parts[1], parts[0])
            } else {
                package_name.to_string()
            }
        };
        Some(format!("pkg:maven/{}@{}", maven_name, version))
    } else if package_name.starts_with('@')
        || package_name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '-' || c == '_')
    {
        Some(format!("pkg:npm/{}@{}", package_name, version))
    } else {
        None
    }
}

/// Generate "why fix this?" explanation
fn generate_why_fix(vuln: &Vulnerability) -> String {
    let mut reasons = Vec::new();

    if let Some(severity) = &vuln.severity {
        let severity_reason = match severity.level {
            bazbom_advisories::SeverityLevel::Critical => {
                "CRITICAL severity - immediate action required"
            }
            bazbom_advisories::SeverityLevel::High => "HIGH severity - fix as soon as possible",
            bazbom_advisories::SeverityLevel::Medium => {
                "MEDIUM severity - schedule fix in near term"
            }
            bazbom_advisories::SeverityLevel::Low => "LOW severity - fix when convenient",
            _ => "Unknown severity",
        };
        reasons.push(severity_reason.to_string());
    }

    if vuln.kev.is_some() {
        reasons.push(
            "Listed in CISA KEV (Known Exploited Vulnerabilities) - actively exploited in the wild"
                .to_string(),
        );
    }

    if let Some(epss) = &vuln.epss {
        if epss.score >= 0.9 {
            reasons.push(format!(
                "Very high exploit probability (EPSS: {:.1}%)",
                epss.score * 100.0
            ));
        } else if epss.score >= 0.5 {
            reasons.push(format!(
                "High exploit probability (EPSS: {:.1}%)",
                epss.score * 100.0
            ));
        } else if epss.score >= 0.1 {
            reasons.push(format!(
                "Moderate exploit probability (EPSS: {:.1}%)",
                epss.score * 100.0
            ));
        }
    }

    if let Some(severity) = &vuln.severity {
        let cvss = severity.cvss_v3.or(severity.cvss_v4);
        if let Some(cvss) = cvss {
            if cvss >= 9.0 {
                reasons.push(format!("Very high CVSS score: {}", cvss));
            } else if cvss >= 7.0 {
                reasons.push(format!("High CVSS score: {}", cvss));
            }
        }
    }

    if let Some(summary) = &vuln.summary {
        reasons.push(format!("Impact: {}", summary));
    }

    if reasons.is_empty() {
        "This vulnerability should be addressed to reduce security risk".to_string()
    } else {
        reasons.join(". ")
    }
}

/// Generate "how to fix" instructions
fn generate_how_to_fix(
    package: &str,
    current_version: &str,
    fixed_version: &Option<String>,
    build_system: BuildSystem,
) -> String {
    match fixed_version {
        Some(fixed) => {
            let upgrade_instruction = match build_system {
                BuildSystem::Maven => {
                    format!(
                        "Update pom.xml:\n\
                         <dependency>\n  \
                           <groupId>{}</groupId>\n  \
                           <artifactId>...</artifactId>\n  \
                           <version>{}</version>\n\
                         </dependency>\n\
                         Then run: mvn clean install",
                        package.split(':').next().unwrap_or(package),
                        fixed
                    )
                }
                BuildSystem::Gradle => {
                    format!(
                        "Update build.gradle or build.gradle.kts:\n\
                         implementation('{}:{}')\n\
                         Then run: gradle build",
                        package, fixed
                    )
                }
                BuildSystem::Bazel => {
                    format!(
                        "Update maven_install in WORKSPACE or MODULE.bazel:\n\
                         \"{}:{}\"\n\
                         Then run: bazel run @maven//:pin",
                        package, fixed
                    )
                }
                _ => {
                    format!("Upgrade {} from {} to {}", package, current_version, fixed)
                }
            };

            format!("Upgrade to version {}.\n\n{}", fixed, upgrade_instruction)
        }
        None => "No fixed version available yet. Consider:\n\
             1. Check for updates from the package maintainer\n\
             2. Apply temporary workarounds if available\n\
             3. Consider alternative packages\n\
             4. Monitor security advisories for updates"
            .to_string(),
    }
}

/// Generate breaking changes warning
fn generate_breaking_changes_warning(
    package: &str,
    current_version: &str,
    fixed_version: &Option<String>,
) -> Option<String> {
    let fixed = match fixed_version {
        Some(v) => v,
        None => return None,
    };

    let (current_major, current_minor, _) = match parse_semantic_version(current_version) {
        Some(v) => v,
        None => {
            return Some(format!(
                "[!] Version change ({} → {})\n\n\
                 Cannot parse semantic version numbers. Please review the changelog manually.\n\
                 Version formats that don't follow semantic versioning (X.Y.Z) require careful review:\n\
                 1. Check the library's release notes\n\
                 2. Review breaking changes documentation\n\
                 3. Test thoroughly in a staging environment",
                current_version, fixed
            ));
        }
    };

    let (fixed_major, fixed_minor, _) = match parse_semantic_version(fixed) {
        Some(v) => v,
        None => {
            return Some(format!(
                "[!] Version change ({} → {})\n\n\
                 Cannot parse target version number. Please review the changelog manually.",
                current_version, fixed
            ));
        }
    };

    if fixed_major > current_major {
        Some(generate_major_version_warning(
            package,
            current_version,
            fixed,
        ))
    } else if fixed_major == current_major {
        if fixed_minor > current_minor {
            Some(generate_minor_version_warning(current_version, fixed))
        } else {
            Some(generate_patch_version_warning(current_version, fixed))
        }
    } else {
        Some(format!(
            "[!] Version change ({} → {})\n\n\
             This version change doesn't follow typical semantic versioning.\n\
             Please review the library's changelog carefully before upgrading.",
            current_version, fixed
        ))
    }
}

fn generate_major_version_warning(package: &str, current_version: &str, fixed: &str) -> String {
    let mut warning = format!(
        "[!] MAJOR VERSION UPGRADE ({} → {})\n\n\
         This is a major version upgrade which may include breaking changes:\n\n\
         - API changes: Methods may be removed, renamed, or have different signatures\n\
         - Deprecated features: Previously deprecated APIs may be removed\n\
         - Behavioral changes: Existing functionality may behave differently\n\
         - Configuration changes: Configuration file formats or options may change\n\
         - Dependency changes: Transitive dependencies may change significantly\n\n",
        current_version, fixed
    );

    let package_lower = package.to_lowercase();
    if package_lower.contains("spring") {
        warning.push_str(
            "Spring Framework specific considerations:\n\
             - Check for configuration property changes\n\
             - Review deprecated @Bean definitions\n\
             - Update Spring Boot parent version if applicable\n\
             - Test all integration points thoroughly\n\n",
        );
    } else if package_lower.contains("jackson") {
        warning.push_str(
            "Jackson specific considerations:\n\
             - Verify JSON serialization/deserialization behavior\n\
             - Check for ObjectMapper configuration changes\n\
             - Test custom serializers and deserializers\n\
             - Review annotation processing changes\n\n",
        );
    } else if package_lower.contains("log4j") {
        warning.push_str(
            "Log4j specific considerations:\n\
             - Update log4j2.xml configuration if needed\n\
             - Review appender and filter configurations\n\
             - Check for plugin compatibility\n\
             - Verify logging output format\n\n",
        );
    } else if package_lower.contains("junit") {
        warning.push_str(
            "JUnit specific considerations:\n\
             - Update test annotations (@Test, @Before, @After)\n\
             - Review assertion methods (may have changed)\n\
             - Check for runner compatibility\n\
             - Verify test lifecycle hooks\n\n",
        );
    } else if package_lower.contains("hibernate") || package_lower.contains("jakarta.persistence") {
        warning.push_str(
            "Hibernate/JPA specific considerations:\n\
             - Review entity mapping annotations\n\
             - Check for query language changes (HQL/JPQL)\n\
             - Verify transaction management behavior\n\
             - Test database migrations carefully\n\n",
        );
    }

    warning.push_str(
        "Recommended actions before upgrading:\n\
         1. Review the library's changelog and migration guide\n\
         2. Run all unit and integration tests\n\
         3. Test in a staging environment first\n\
         4. Have a rollback plan ready\n\
         5. Update any dependent libraries if needed\n\
         6. Document any code changes required for the upgrade",
    );

    warning
}

fn generate_minor_version_warning(current_version: &str, fixed: &str) -> String {
    format!(
        "[i] Minor version upgrade ({} → {})\n\n\
         This is a minor version upgrade which should be backward compatible but may include:\n\
         - New features and APIs\n\
         - Deprecation warnings for future removal\n\
         - Performance improvements\n\
         - Bug fixes\n\n\
         Recommended actions:\n\
         1. Review release notes for new deprecations\n\
         2. Run full test suite to verify compatibility\n\
         3. Check for any new security recommendations",
        current_version, fixed
    )
}

fn generate_patch_version_warning(current_version: &str, fixed: &str) -> String {
    format!(
        "[+] Patch version upgrade ({} → {})\n\n\
         This is a patch version upgrade which should be fully backward compatible.\n\
         It typically includes:\n\
         - Bug fixes\n\
         - Security patches\n\
         - Performance improvements\n\n\
         This upgrade should be safe, but it's still recommended to:\n\
         1. Run your test suite\n\
         2. Review the changelog for the specific fixes included",
        current_version, fixed
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breaking_changes_major_version() {
        let warning = generate_breaking_changes_warning(
            "org.springframework:spring-core",
            "5.3.0",
            &Some("6.0.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("MAJOR VERSION UPGRADE"));
        assert!(text.contains("5.3.0 → 6.0.0"));
        assert!(text.contains("Spring Framework specific considerations"));
    }

    #[test]
    fn test_breaking_changes_minor_version() {
        let warning = generate_breaking_changes_warning(
            "com.fasterxml.jackson.core:jackson-databind",
            "2.13.0",
            &Some("2.14.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Minor version upgrade"));
        assert!(text.contains("2.13.0 → 2.14.0"));
        assert!(text.contains("backward compatible"));
    }

    #[test]
    fn test_breaking_changes_patch_version() {
        let warning = generate_breaking_changes_warning(
            "org.apache.logging.log4j:log4j-core",
            "2.17.0",
            &Some("2.17.1".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Patch version upgrade"));
        assert!(text.contains("2.17.0 → 2.17.1"));
        assert!(text.contains("fully backward compatible"));
    }

    #[test]
    fn test_breaking_changes_no_fixed_version() {
        let warning = generate_breaking_changes_warning("test-package", "1.0.0", &None);
        assert!(warning.is_none());
    }

    #[test]
    fn test_breaking_changes_log4j_specific() {
        let warning = generate_breaking_changes_warning(
            "org.apache.logging.log4j:log4j-core",
            "2.14.0",
            &Some("3.0.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Log4j specific considerations"));
        assert!(text.contains("log4j2.xml configuration"));
    }

    #[test]
    fn test_breaking_changes_jackson_specific() {
        let warning = generate_breaking_changes_warning(
            "com.fasterxml.jackson.core:jackson-databind",
            "2.0.0",
            &Some("3.0.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Jackson specific considerations"));
        assert!(text.contains("JSON serialization"));
    }

    #[test]
    fn test_breaking_changes_junit_specific() {
        let warning = generate_breaking_changes_warning(
            "org.junit:junit",
            "4.13.0",
            &Some("5.0.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("JUnit specific considerations"));
        assert!(text.contains("test annotations"));
    }

    #[test]
    fn test_breaking_changes_hibernate_specific() {
        let warning = generate_breaking_changes_warning(
            "org.hibernate:hibernate-core",
            "5.6.0",
            &Some("6.0.0".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Hibernate/JPA specific considerations"));
        assert!(text.contains("entity mapping"));
    }

    #[test]
    fn test_breaking_changes_snapshot_version() {
        let warning = generate_breaking_changes_warning(
            "org.springframework:spring-core",
            "5.3.0-SNAPSHOT",
            &Some("6.0.0-RELEASE".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("MAJOR VERSION UPGRADE"));
        assert!(text.contains("Spring Framework specific considerations"));
    }

    #[test]
    fn test_breaking_changes_invalid_version() {
        let warning =
            generate_breaking_changes_warning("test-package", "alpha", &Some("beta".to_string()));
        assert!(warning.is_some());
        let text = warning.unwrap();
        assert!(text.contains("Cannot parse semantic version"));
        assert!(text.contains("review the changelog manually"));
    }

    #[test]
    fn test_breaking_changes_complex_version() {
        let warning = generate_breaking_changes_warning(
            "org.apache.logging.log4j:log4j-core",
            "2.17.1-rc1",
            &Some("2.18.0-beta".to_string()),
        );
        assert!(warning.is_some());
        let text = warning.unwrap();
        // Should strip suffixes and detect minor version bump
        assert!(text.contains("Minor version upgrade") || text.contains("Version change"));
    }
}
