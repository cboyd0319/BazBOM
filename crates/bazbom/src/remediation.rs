// Remediation automation for BazBOM
// Provides "suggest" and "apply" modes for fixing vulnerabilities

use anyhow::{Context, Result};
use bazbom_advisories::Vulnerability;
use bazbom_core::BuildSystem;
use chrono;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::backup::{choose_backup_strategy, BackupHandle};
use crate::enrich::depsdev::{BreakingChanges, DepsDevClient};
use crate::test_runner::{has_tests, run_tests};

/// Shared utility for parsing semantic version strings
/// Returns (major, minor, patch) tuple
pub(crate) fn parse_semantic_version(version: &str) -> Option<(u32, u32, u32)> {
    let clean_version = version.split('-').next()?;
    let parts: Vec<&str> = clean_version.split('.').collect();
    
    if parts.len() < 3 {
        return None;
    }
    
    let parse_part = |s: &str| -> Option<u32> {
        if s.chars().all(|c| c.is_ascii_digit()) {
            s.parse().ok()
        } else {
            None
        }
    };
    
    let major = parse_part(parts[0])?;
    let minor = parse_part(parts[1])?;
    let patch = parse_part(parts[2])?;
    
    Some((major, minor, patch))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemediationSuggestion {
    pub vulnerability_id: String,
    pub affected_package: String,
    pub current_version: String,
    pub fixed_version: Option<String>,
    pub severity: String,
    pub priority: String,
    pub why_fix: String,
    pub how_to_fix: String,
    pub breaking_changes: Option<String>,
    pub references: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemediationReport {
    pub summary: RemediationSummary,
    pub suggestions: Vec<RemediationSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemediationSummary {
    pub total_vulnerabilities: usize,
    pub fixable: usize,
    pub unfixable: usize,
    pub estimated_effort: String,
}

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
            let current_version = "unknown"; // We don't have version info in AffectedPackage

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

    let estimated_effort = if fixable == 0 {
        "No immediate fixes available".to_string()
    } else if fixable <= 5 {
        "Low (< 1 hour)".to_string()
    } else if fixable <= 20 {
        "Medium (1-4 hours)".to_string()
    } else {
        "High (> 4 hours)".to_string()
    };

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

/// Enrich remediation suggestions with deps.dev breaking changes information
pub fn enrich_with_depsdev(
    mut report: RemediationReport,
    depsdev_client: &DepsDevClient,
) -> RemediationReport {
    for suggestion in &mut report.suggestions {
        // Only enrich if we have a fixed version
        if let Some(ref fixed_version) = suggestion.fixed_version {
            // Try to construct a PURL for the package
            // This is a simplified approach - in production, you'd want to extract
            // the actual PURL from the SBOM or vulnerability data
            if let Some(purl) = construct_purl(&suggestion.affected_package, fixed_version) {
                if let Ok(package_info) = depsdev_client.get_package_info(&purl) {
                    if let Some(breaking_changes) = package_info.breaking_changes {
                        suggestion.breaking_changes =
                            Some(format_breaking_changes(&breaking_changes));
                        
                        // Add changelog URL to references if available
                        if let Some(url) = breaking_changes.changelog_url {
                            if !suggestion.references.contains(&url) {
                                suggestion.references.push(url);
                            }
                        }
                        
                        // Add migration guide URL to references if available
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
    // Try to detect package ecosystem from name patterns
    if package_name.contains('/') || package_name.contains('.') {
        // Likely Maven: groupId/artifactId or groupId.artifactId
        let maven_name = if package_name.contains('/') {
            package_name.to_string()
        } else {
            // Convert groupId.artifactId to groupId/artifactId
            let parts: Vec<&str> = package_name.rsplitn(2, '.').collect();
            if parts.len() == 2 {
                format!("{}/{}", parts[1], parts[0])
            } else {
                package_name.to_string()
            }
        };
        Some(format!("pkg:maven/{}@{}", maven_name, version))
    } else if package_name.starts_with('@') {
        // Likely scoped npm package
        Some(format!("pkg:npm/{}@{}", package_name, version))
    } else if package_name.chars().all(|c| c.is_ascii_lowercase() || c == '-' || c == '_') {
        // Likely npm or PyPI package
        // Default to npm for now
        Some(format!("pkg:npm/{}@{}", package_name, version))
    } else {
        None
    }
}

/// Generate "why fix this?" explanation
fn generate_why_fix(vuln: &Vulnerability) -> String {
    let mut reasons = Vec::new();

    // Severity
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

    // KEV presence
    if vuln.kev.is_some() {
        reasons.push(
            "Listed in CISA KEV (Known Exploited Vulnerabilities) - actively exploited in the wild"
                .to_string(),
        );
    }

    // EPSS
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

    // CVSS score
    if let Some(severity) = &vuln.severity {
        // Use CVSS v3 if available, otherwise v4
        let cvss = severity.cvss_v3.or(severity.cvss_v4);
        if let Some(cvss) = cvss {
            if cvss >= 9.0 {
                reasons.push(format!("Very high CVSS score: {}", cvss));
            } else if cvss >= 7.0 {
                reasons.push(format!("High CVSS score: {}", cvss));
            }
        }
    }

    // Summary
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

    // Parse semantic versions to detect major version changes
    // Use shared utility to parse versions
    let (current_major, current_minor, _) = match parse_semantic_version(current_version) {
        Some(v) => v,
        None => {
            return Some(format!(
                "⚠️  Version change ({} → {})\n\n\
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
                "⚠️  Version change ({} → {})\n\n\
                 Cannot parse target version number. Please review the changelog manually.",
                current_version, fixed
            ));
        }
    };

    if fixed_major > current_major {
        // Major version bump detected
        let mut warning = format!(
            "⚠️  MAJOR VERSION UPGRADE ({} → {})\n\n\
             This is a major version upgrade which may include breaking changes:\n\n\
             - API changes: Methods may be removed, renamed, or have different signatures\n\
             - Deprecated features: Previously deprecated APIs may be removed\n\
             - Behavioral changes: Existing functionality may behave differently\n\
             - Configuration changes: Configuration file formats or options may change\n\
             - Dependency changes: Transitive dependencies may change significantly\n\n",
            current_version, fixed
        );

        // Add package-specific warnings for well-known libraries
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
        } else if package_lower.contains("hibernate")
            || package_lower.contains("jakarta.persistence")
        {
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

        Some(warning)
    } else if fixed_major == current_major {
        // Minor or patch version upgrade
        if fixed_minor > current_minor {
            // Minor version bump
            Some(format!(
                "ℹ️  Minor version upgrade ({} → {})\n\n\
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
            ))
        } else {
            // Patch version bump
            Some(format!(
                "✅ Patch version upgrade ({} → {})\n\n\
                 This is a patch version upgrade which should be fully backward compatible.\n\
                 It typically includes:\n\
                 - Bug fixes\n\
                 - Security patches\n\
                 - Performance improvements\n\n\
                 This upgrade should be safe, but it's still recommended to:\n\
                 1. Run your test suite\n\
                 2. Review the changelog for the specific fixes included",
                current_version, fixed
            ))
        }
    } else {
        // Downgrade or unusual version scheme
        Some(format!(
            "⚠️  Version change ({} → {})\n\n\
             This version change doesn't follow typical semantic versioning.\n\
             Please review the library's changelog carefully before upgrading.",
            current_version, fixed
        ))
    }
}

/// Apply fixes automatically (Phase 4 - simplified version)
pub fn apply_fixes(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
) -> Result<ApplyResult> {
    let mut applied = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for suggestion in suggestions {
        if suggestion.fixed_version.is_none() {
            skipped += 1;
            continue;
        }

        let result = match build_system {
            BuildSystem::Maven => apply_maven_fix(suggestion, project_root),
            BuildSystem::Gradle => apply_gradle_fix(suggestion, project_root),
            BuildSystem::Bazel => apply_bazel_fix(suggestion, project_root),
            _ => {
                eprintln!("[bazbom] unsupported build system for auto-fix");
                skipped += 1;
                continue;
            }
        };

        match result {
            Ok(_) => applied += 1,
            Err(e) => {
                eprintln!(
                    "[bazbom] failed to apply fix for {}: {}",
                    suggestion.affected_package, e
                );
                failed += 1;
            }
        }
    }

    Ok(ApplyResult {
        applied,
        failed,
        skipped,
    })
}

#[derive(Debug, Serialize)]
pub struct ApplyResult {
    pub applied: usize,
    pub failed: usize,
    pub skipped: usize,
}

fn apply_maven_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let pom_path = project_root.join("pom.xml");
    if !pom_path.exists() {
        anyhow::bail!("pom.xml not found in project root");
    }

    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    // Read the pom.xml
    let content = fs::read_to_string(&pom_path)?;

    // Extract artifact name
    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    // Simple approach: find <version> tags that follow <artifactId> tags containing our artifact
    // and replace the version if it matches the current version
    let mut updated = content.clone();
    let mut match_found = false;

    let lines: Vec<&str> = content.lines().collect();
    for i in 0..lines.len() {
        let line = lines[i];

        // If this line has the artifactId we're looking for
        if line.contains("<artifactId>")
            && line.contains(artifact)
            && line.contains("</artifactId>")
        {
            // Look ahead for a <version> tag
            for j in (i + 1).min(lines.len())..((i + 5).min(lines.len())) {
                let version_line = lines[j];
                if version_line.contains("<version>")
                    && version_line.contains(&suggestion.current_version)
                {
                    // Found it! Replace this line
                    let new_line = version_line.replace(&suggestion.current_version, fixed_version);
                    updated = updated.replace(version_line, &new_line);
                    match_found = true;
                    println!(
                        "  ✓ Updated {}: {} → {}",
                        artifact, suggestion.current_version, fixed_version
                    );
                    break;
                }
            }
            if match_found {
                break;
            }
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in pom.xml",
            artifact,
            suggestion.current_version
        );
    }

    // Write updated content back to file
    fs::write(&pom_path, updated)?;
    Ok(())
}

fn apply_gradle_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    // Try both build.gradle and build.gradle.kts
    let build_gradle = project_root.join("build.gradle");
    let build_gradle_kts = project_root.join("build.gradle.kts");

    let gradle_file = if build_gradle.exists() {
        build_gradle
    } else if build_gradle_kts.exists() {
        build_gradle_kts
    } else {
        anyhow::bail!("No build.gradle or build.gradle.kts found in project root");
    };

    let content = fs::read_to_string(&gradle_file)?;

    // Extract artifact name (last part after ':')
    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    // Simple string-based replacement
    // Look for patterns like: 'group:artifact:version' or "group:artifact:version"
    let mut updated = content.clone();
    let mut match_found = false;

    // Try to find the artifact in the file
    for line in content.lines() {
        if line.contains(artifact) && line.contains(&suggestion.current_version) {
            // Simple replacement: replace old version with new version on lines containing the artifact
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  ✓ Updated {}: {} → {}",
                artifact, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in {}",
            artifact,
            suggestion.current_version,
            gradle_file.display()
        );
    }

    // Write updated content back
    fs::write(&gradle_file, updated)?;
    Ok(())
}

fn apply_bazel_fix(suggestion: &RemediationSuggestion, project_root: &Path) -> Result<()> {
    let fixed_version = suggestion
        .fixed_version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No fixed version available"))?;

    // Try MODULE.bazel first, then WORKSPACE
    let module_bazel = project_root.join("MODULE.bazel");
    let workspace = project_root.join("WORKSPACE");

    let bazel_file = if module_bazel.exists() {
        module_bazel
    } else if workspace.exists() {
        workspace
    } else {
        anyhow::bail!("No MODULE.bazel or WORKSPACE found in project root");
    };

    let content = fs::read_to_string(&bazel_file)?;

    // Extract artifact coordinates
    let artifact = suggestion
        .affected_package
        .rsplit(':')
        .next()
        .unwrap_or(&suggestion.affected_package);

    // Simple string-based replacement
    let mut updated = content.clone();
    let mut match_found = false;

    // Look for the artifact with current version in maven coordinates
    for line in content.lines() {
        if line.contains(artifact) && line.contains(&suggestion.current_version) {
            let new_line = line.replace(&suggestion.current_version, fixed_version);
            updated = updated.replace(line, &new_line);
            match_found = true;
            println!(
                "  ✓ Updated {}: {} → {}",
                artifact, suggestion.current_version, fixed_version
            );
            break;
        }
    }

    if !match_found {
        anyhow::bail!(
            "Dependency {} with version {} not found in {}",
            artifact,
            suggestion.current_version,
            bazel_file.display()
        );
    }

    // Write updated content back
    fs::write(&bazel_file, updated)?;

    // Note: In a production system, we would also:
    // 1. Update maven_install.json if it exists
    // 2. Run `bazel run @maven//:pin` to regenerate lock files
    // For now, just update the BUILD/WORKSPACE file
    println!("  ⚠️  Remember to run: bazel run @maven//:pin");

    Ok(())
}

/// Apply fixes with testing and automatic rollback on failure
/// This is the safe way to apply fixes - wraps apply_fixes with test execution
pub fn apply_fixes_with_testing(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
    skip_tests: bool,
) -> Result<ApplyResultWithTests> {
    // Step 1: Create backup
    println!("\n[bazbom] Creating backup before applying fixes...");
    let strategy = choose_backup_strategy(project_root);
    let backup = BackupHandle::create(project_root, strategy)?;

    // Step 2: Apply fixes
    println!("\n[bazbom] Applying fixes...");
    let apply_result = apply_fixes(suggestions, build_system, project_root)?;

    if apply_result.applied == 0 {
        println!("\n[bazbom] No fixes were applied");
        backup.cleanup()?;
        return Ok(ApplyResultWithTests {
            apply_result,
            tests_passed: true,
            tests_run: false,
            test_output: None,
        });
    }

    // Step 3: Run tests if not skipped and tests exist
    let should_run_tests = !skip_tests && has_tests(build_system, project_root);

    if !should_run_tests {
        println!("\n[bazbom] Skipping tests");
        backup.cleanup()?;
        return Ok(ApplyResultWithTests {
            apply_result,
            tests_passed: true,
            tests_run: false,
            test_output: None,
        });
    }

    println!("\n[bazbom] Running tests to verify fixes...");
    let test_result = run_tests(build_system, project_root)?;

    if test_result.success {
        println!("\n✅ Tests passed! Fixes applied successfully.");
        println!("   Duration: {:.2}s", test_result.duration.as_secs_f64());

        // Clean up backup
        backup.cleanup()?;

        Ok(ApplyResultWithTests {
            apply_result,
            tests_passed: true,
            tests_run: true,
            test_output: Some(test_result.output),
        })
    } else {
        println!("\n❌ Tests failed! Rolling back changes...");
        println!("   Exit code: {}", test_result.exit_code);

        // Restore from backup
        backup.restore()?;

        println!("\n[bazbom] Changes rolled back successfully.");
        println!("\nTest output:\n{}", test_result.output);

        anyhow::bail!(
            "Fixes were rolled back because tests failed. \
             Review the test output above to understand the issue."
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ApplyResultWithTests {
    pub apply_result: ApplyResult,
    pub tests_passed: bool,
    pub tests_run: bool,
    pub test_output: Option<String>,
}

/// Configuration for PR generation
#[derive(Debug)]
pub struct PrConfig {
    pub github_token: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub base_branch: String,
}

impl PrConfig {
    /// Load PR configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let github_token = std::env::var("GITHUB_TOKEN")
            .or_else(|_| std::env::var("GH_TOKEN"))
            .context("GITHUB_TOKEN or GH_TOKEN environment variable not set")?;

        let repository = std::env::var("GITHUB_REPOSITORY")
            .context("GITHUB_REPOSITORY environment variable not set (format: owner/repo)")?;

        let parts: Vec<&str> = repository.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("GITHUB_REPOSITORY must be in format: owner/repo");
        }

        let base_branch = std::env::var("GITHUB_BASE_REF")
            .or_else(|_| std::env::var("GITHUB_REF_NAME"))
            .unwrap_or_else(|_| "main".to_string());

        Ok(Self {
            github_token,
            repo_owner: parts[0].to_string(),
            repo_name: parts[1].to_string(),
            base_branch,
        })
    }
}

/// Generate a PR with fixes applied
///
/// This function:
/// 1. Creates a new branch
/// 2. Applies fixes with testing
/// 3. Commits changes
/// 4. Pushes to remote
/// 5. Opens a PR via GitHub API
pub fn generate_pr(
    suggestions: &[RemediationSuggestion],
    build_system: BuildSystem,
    project_root: &Path,
) -> Result<String> {
    println!("\n[bazbom] Generating PR for vulnerability fixes...");

    // Load configuration
    let config = PrConfig::from_env()?;

    // Generate branch name with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let branch_name = format!("bazbom/fix-vulnerabilities-{}", timestamp);

    println!("[bazbom] Creating branch: {}", branch_name);

    // Create new branch
    let status = Command::new("git")
        .args(&["checkout", "-b", &branch_name])
        .current_dir(project_root)
        .status()
        .context("Failed to create git branch")?;

    if !status.success() {
        anyhow::bail!("Failed to create branch {}", branch_name);
    }

    // Apply fixes with testing
    println!("\n[bazbom] Applying fixes...");
    let apply_result = apply_fixes_with_testing(suggestions, build_system, project_root, false)?;

    if apply_result.apply_result.applied == 0 {
        println!("[bazbom] No fixes were applied, skipping PR creation");
        // Checkout back to original branch
        let _ = Command::new("git")
            .args(&["checkout", "-"])
            .current_dir(project_root)
            .status();
        return Ok("No fixes applied, PR not created".to_string());
    }

    // Stage all changes
    println!("\n[bazbom] Staging changes...");
    let status = Command::new("git")
        .args(&["add", "-A"])
        .current_dir(project_root)
        .status()
        .context("Failed to stage changes")?;

    if !status.success() {
        anyhow::bail!("Failed to stage changes");
    }

    // Generate commit message
    let commit_message = generate_commit_message(suggestions, &apply_result);

    println!("[bazbom] Committing changes...");
    let status = Command::new("git")
        .args(&["commit", "-m", &commit_message])
        .current_dir(project_root)
        .status()
        .context("Failed to commit changes")?;

    if !status.success() {
        anyhow::bail!("Failed to commit changes");
    }

    // Push branch
    println!("[bazbom] Pushing branch to remote...");
    let status = Command::new("git")
        .args(&["push", "-u", "origin", &branch_name])
        .current_dir(project_root)
        .status()
        .context("Failed to push branch")?;

    if !status.success() {
        anyhow::bail!("Failed to push branch {}", branch_name);
    }

    // Generate PR body
    let pr_body = generate_pr_body(suggestions, &apply_result);
    let pr_title = generate_pr_title(suggestions, &apply_result.apply_result);

    // Create PR via GitHub API
    println!("\n[bazbom] Creating pull request...");
    let pr_url = create_github_pr(&config, &pr_title, &pr_body, &branch_name)?;

    println!("\n✅ Pull request created successfully!");
    println!("   URL: {}", pr_url);

    Ok(pr_url)
}

/// Generate commit message
fn generate_commit_message(
    suggestions: &[RemediationSuggestion],
    apply_result: &ApplyResultWithTests,
) -> String {
    let cve_count = apply_result.apply_result.applied;

    // Collect unique CVE IDs from applied suggestions
    let cves: Vec<String> = suggestions
        .iter()
        .filter(|s| s.fixed_version.is_some())
        .take(apply_result.apply_result.applied)
        .map(|s| s.vulnerability_id.clone())
        .collect();

    let cve_list = if cves.len() <= 3 {
        cves.join(", ")
    } else {
        format!("{} and {} more", cves[..3].join(", "), cves.len() - 3)
    };

    // Test status message
    let test_status = if apply_result.tests_run {
        if apply_result.tests_passed {
            "All tests passed after applying fixes."
        } else {
            "Tests failed (changes were rolled back)."
        }
    } else {
        "Tests were not run or not found."
    };

    format!(
        "fix: upgrade {} dependencies to fix vulnerabilities\n\n\
         Fixes: {}\n\n\
         Applied {} dependency upgrades to address security vulnerabilities.\n\
         {}\n\n\
         🤖 Generated by BazBOM\n\
         Co-Authored-By: BazBOM <noreply@bazbom.io>",
        cve_count, cve_list, apply_result.apply_result.applied, test_status
    )
}

/// Generate PR title
fn generate_pr_title(_suggestions: &[RemediationSuggestion], apply_result: &ApplyResult) -> String {
    let count = apply_result.applied;
    if count == 1 {
        "🔒 Fix 1 security vulnerability".to_string()
    } else {
        format!("🔒 Fix {} security vulnerabilities", count)
    }
}

/// Generate PR body with detailed information
fn generate_pr_body(
    suggestions: &[RemediationSuggestion],
    apply_result: &ApplyResultWithTests,
) -> String {
    let mut body = String::from("## 🔒 Security Fixes\n\n");
    body.push_str(
        "This PR automatically upgrades vulnerable dependencies identified by BazBOM.\n\n",
    );

    // Summary section
    body.push_str("### Summary\n\n");
    body.push_str(&format!(
        "- ✅ **{}** vulnerabilities fixed\n",
        apply_result.apply_result.applied
    ));
    if apply_result.apply_result.failed > 0 {
        body.push_str(&format!(
            "- ❌ **{}** fixes failed\n",
            apply_result.apply_result.failed
        ));
    }
    if apply_result.apply_result.skipped > 0 {
        body.push_str(&format!(
            "- ⏭️  **{}** vulnerabilities skipped (no fix available)\n",
            apply_result.apply_result.skipped
        ));
    }
    body.push_str("\n");

    // Details section
    body.push_str("### Vulnerabilities Fixed\n\n");
    body.push_str("| Package | Current | Fixed | Severity | CVE |\n");
    body.push_str("|---------|---------|-------|----------|-----|\n");

    for suggestion in suggestions {
        if let Some(fixed) = &suggestion.fixed_version {
            body.push_str(&format!(
                "| {} | {} | {} | {} | {} |\n",
                suggestion.affected_package,
                suggestion.current_version,
                fixed,
                suggestion.severity,
                suggestion.vulnerability_id
            ));
        }
    }
    body.push_str("\n");

    // Test results
    body.push_str("### Test Results\n\n");
    if apply_result.tests_run {
        if apply_result.tests_passed {
            body.push_str("✅ All tests passed after applying fixes.\n\n");
        } else {
            body.push_str("❌ Tests failed (changes were rolled back).\n\n");
        }
    } else {
        body.push_str("⏭️  Tests were skipped or not found.\n\n");
    }

    // Review instructions
    body.push_str("### How to Review\n\n");
    body.push_str("1. Review the diff to ensure only dependency versions were changed\n");
    body.push_str("2. Check the CVE details in the table above\n");
    body.push_str("3. Verify that tests pass in CI\n");
    body.push_str("4. Merge if changes look correct\n\n");

    // Footer
    body.push_str("---\n");
    body.push_str("🤖 Generated with [BazBOM](https://github.com/cboyd0319/BazBOM)\n");

    body
}

/// Create a pull request via GitHub API
fn create_github_pr(
    config: &PrConfig,
    title: &str,
    body: &str,
    head_branch: &str,
) -> Result<String> {
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/pulls",
        config.repo_owner, config.repo_name
    );

    let pr_data = json!({
        "title": title,
        "body": body,
        "head": head_branch,
        "base": config.base_branch,
    });

    let response = ureq::post(&api_url)
        .set("Authorization", &format!("token {}", config.github_token))
        .set("Accept", "application/vnd.github.v3+json")
        .set("User-Agent", "BazBOM/0.2.1")
        .send_json(pr_data)
        .context("Failed to create pull request via GitHub API")?;

    let status = response.status();
    if status != 201 {
        let error_body = response
            .into_string()
            .unwrap_or_else(|_| "Unknown error".to_string());
        anyhow::bail!("GitHub API returned status {}: {}", status, error_body);
    }

    let pr_response: serde_json::Value = response
        .into_json()
        .context("Failed to parse GitHub API response")?;

    let pr_url = pr_response["html_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("No html_url in GitHub API response"))?
        .to_string();

    Ok(pr_url)
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
