//! Display functions for container scan results

use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

use super::{
    detect_ecosystem, format_difficulty_label, ActionItem, ContainerScanOptions,
    ContainerScanResults, PackageEcosystem, QuickWin, VulnerabilityInfo,
};

/// Apply filter to scan results
pub(crate) fn apply_filter(
    results: &ContainerScanResults,
    filter: &str,
) -> Result<ContainerScanResults> {
    let mut filtered = results.clone();

    for layer in &mut filtered.layers {
        layer.vulnerabilities = layer
            .vulnerabilities
            .iter()
            .filter(|v| {
                match filter.to_lowercase().as_str() {
                    "p0" => v.priority.as_ref().map(|p| p == "P0").unwrap_or(false),
                    "p1" => v.priority.as_ref().map(|p| p == "P1").unwrap_or(false),
                    "p2" => v.priority.as_ref().map(|p| p == "P2").unwrap_or(false),
                    "fixable" => v.fixed_version.is_some(),
                    "quick-wins" => v.fixed_version.is_some() && v.breaking_change != Some(true),
                    "critical" => v.severity == "CRITICAL",
                    "high" => v.severity == "HIGH",
                    "medium" => v.severity == "MEDIUM",
                    "low" => v.severity == "LOW",
                    "kev" => v.is_kev,
                    _ => true, // Unknown filter, show all
                }
            })
            .cloned()
            .collect();
    }

    // Recalculate counts
    filtered.total_vulnerabilities = 0;
    filtered.critical_count = 0;
    filtered.high_count = 0;
    filtered.medium_count = 0;
    filtered.low_count = 0;

    for layer in &filtered.layers {
        filtered.total_vulnerabilities += layer.vulnerabilities.len();
        for vuln in &layer.vulnerabilities {
            match vuln.severity.as_str() {
                "CRITICAL" => filtered.critical_count += 1,
                "HIGH" => filtered.high_count += 1,
                "MEDIUM" => filtered.medium_count += 1,
                "LOW" => filtered.low_count += 1,
                _ => {}
            }
        }
    }

    Ok(filtered)
}

/// Display results with beautiful UX
pub(crate) fn display_results(
    results: &ContainerScanResults,
    opts: &ContainerScanOptions,
) -> Result<()> {
    use bazbom::container_ux::ContainerSummary;
    use std::time::Duration;

    // Apply filter if specified
    let filtered_results = if let Some(ref filter) = opts.filter {
        apply_filter(results, filter)?
    } else {
        results.clone()
    };

    // Show filter status if active
    if let Some(ref filter) = opts.filter {
        println!();
        println!("{}", format!("Filter: {}", filter).bright_yellow().bold());
        println!(
            "   Showing {} of {} total vulnerabilities",
            filtered_results.total_vulnerabilities, results.total_vulnerabilities
        );
    }

    println!("{}", "━".repeat(67).bright_cyan());
    println!("{}", "SECURITY ANALYSIS RESULTS".bright_cyan().bold());
    println!("{}", "━".repeat(67).bright_cyan());
    println!();

    // Show layer breakdown with detailed info
    println!("{}", "Layer Attribution:".bold());
    println!();
    for (idx, layer) in filtered_results.layers.iter().enumerate() {
        let layer_vulns = layer.vulnerabilities.len();
        let pkg_count = layer.packages.len();

        // Count severity breakdown for this layer
        let mut critical = 0;
        let mut high = 0;
        let mut medium = 0;
        let mut low = 0;
        for vuln in &layer.vulnerabilities {
            match vuln.severity.as_str() {
                "CRITICAL" => critical += 1,
                "HIGH" => high += 1,
                "MEDIUM" => medium += 1,
                "LOW" => low += 1,
                _ => {}
            }
        }

        let status = if layer_vulns == 0 {
            "clean".green()
        } else if critical > 0 {
            format!(
                "{} vulns ({}C/{}H/{}M/{}L)",
                layer_vulns, critical, high, medium, low
            )
            .red()
            .bold()
        } else if high > 0 {
            format!("{} vulns ({}H/{}M/{}L)", layer_vulns, high, medium, low)
                .yellow()
                .bold()
        } else {
            format!("{} vulns ({}M/{}L)", layer_vulns, medium, low).yellow()
        };

        let tree_char = if idx == filtered_results.layers.len() - 1 {
            "'-"
        } else {
            "|-"
        };

        println!(
            "  {} Layer {}: {}",
            tree_char.bright_cyan(),
            idx + 1,
            layer.digest.bright_white()
        );
        println!(
            "     Size: {:.1} MB | Packages: {} | {}",
            layer.size_mb.to_string().bright_white(),
            pkg_count.to_string().bright_white().bold(),
            status
        );

        // Show sample packages (first 3)
        if !layer.packages.is_empty() {
            let sample_count = 3.min(layer.packages.len());
            let samples: Vec<String> = layer.packages.iter().take(sample_count).cloned().collect();
            println!("     Packages: {}", samples.join(", ").dimmed());
            if layer.packages.len() > sample_count {
                println!(
                    "        {} and {} more...",
                    "".dimmed(),
                    (layer.packages.len() - sample_count).to_string().dimmed()
                );
            }
        }

        // Show top vulnerabilities in this layer
        if !layer.vulnerabilities.is_empty() {
            let mut vulns_by_severity = layer.vulnerabilities.clone();
            vulns_by_severity.sort_by(|a, b| {
                let severity_order = |s: &str| match s {
                    "CRITICAL" => 0,
                    "HIGH" => 1,
                    "MEDIUM" => 2,
                    "LOW" => 3,
                    _ => 4,
                };
                severity_order(&a.severity).cmp(&severity_order(&b.severity))
            });

            let show_count = 3.min(vulns_by_severity.len());
            println!("     Top vulnerabilities:");
            for vuln in vulns_by_severity.iter().take(show_count) {
                let severity_icon = match vuln.severity.as_str() {
                    "CRITICAL" => "CRIT",
                    "HIGH" => "HIGH",
                    "MEDIUM" => "MED",
                    "LOW" => "LOW",
                    _ => "UNK",
                };

                // Priority badge
                let priority_badge = if let Some(ref priority) = vuln.priority {
                    match priority.as_str() {
                        "P0" => " [P0]".red().bold(),
                        "P1" => " [P1]".yellow().bold(),
                        _ => "".normal(),
                    }
                } else {
                    "".normal()
                };

                // KEV indicator
                let kev_indicator = if vuln.is_kev {
                    format!(
                        " KEV (due: {})",
                        vuln.kev_due_date.as_ref().unwrap_or(&"unknown".to_string())
                    )
                    .red()
                    .bold()
                } else {
                    "".normal()
                };

                // Reachability indicator
                let reachability_indicator = if opts.with_reachability {
                    if vuln.is_reachable {
                        " REACHABLE".red().bold()
                    } else {
                        " unreachable".dimmed()
                    }
                } else {
                    "".normal()
                };

                // Fix status with breaking change warning
                let fix_status = if let Some(ref fix) = vuln.fixed_version {
                    let mut status = format!("-> {}", fix).green();
                    if vuln.breaking_change == Some(true) {
                        status = format!("{} breaking", status).yellow();
                    }
                    status
                } else {
                    "no fix available".dimmed()
                };

                println!(
                    "        {} {}{}{}{}",
                    severity_icon,
                    vuln.cve_id.bright_white().bold(),
                    priority_badge,
                    kev_indicator,
                    reachability_indicator
                );
                println!(
                    "           in {} {} {}",
                    vuln.package_name.bright_cyan(),
                    fix_status,
                    if let Some(epss) = vuln.epss_score {
                        format!("| EPSS: {:.1}%", epss * 100.0).dimmed()
                    } else {
                        "".normal()
                    }
                );

                // Show CVSS score
                if let Some(cvss) = vuln.cvss_score {
                    println!(
                        "           CVSS: {:.1} | {}",
                        cvss.to_string().bright_white(),
                        if let Some(refs) = vuln.references.first() {
                            refs.dimmed()
                        } else {
                            "".normal()
                        }
                    );
                }

                // Show upgrade intelligence
                if let Some(ref upgrade_path) = vuln.upgrade_path {
                    println!("           {}", upgrade_path.dimmed());
                }

                // Show difficulty score
                if let Some(difficulty) = vuln.difficulty_score {
                    let difficulty_label = format_difficulty_label(difficulty);
                    println!("           {}", difficulty_label);
                }

                // Show dependency path for transitive vulnerabilities
                if let Some(ref dep_path) = vuln.dependency_path {
                    if dep_path.len() > 1 {
                        let path_str = dep_path.join(" → ");
                        println!("           Dep path: {}", path_str.dimmed());
                    }
                }
            }
            if vulns_by_severity.len() > show_count {
                println!(
                    "        {} and {} more vulnerabilities...",
                    "".dimmed(),
                    (vulns_by_severity.len() - show_count).to_string().dimmed()
                );
            }
        }
        println!();
    }

    // Show vulnerability breakdown by severity
    if filtered_results.total_vulnerabilities > 0 {
        println!("{}", "Vulnerabilities by Severity:".bold());
        println!();
        if filtered_results.critical_count > 0 {
            println!(
                "  CRITICAL: {} (fix immediately!)",
                filtered_results.critical_count.to_string().red().bold()
            );
        }
        if filtered_results.high_count > 0 {
            println!(
                "  HIGH:     {}",
                filtered_results.high_count.to_string().yellow().bold()
            );
        }
        if filtered_results.medium_count > 0 {
            println!(
                "  MEDIUM:   {}",
                filtered_results.medium_count.to_string().yellow()
            );
        }
        if filtered_results.low_count > 0 {
            println!(
                "  LOW:      {}",
                filtered_results.low_count.to_string().green()
            );
        }
        println!();
    }

    // Extract Java artifacts count from SBOM if available
    let java_artifacts = if let Ok(sbom_content) =
        std::fs::read_to_string(opts.output_dir.join("sbom").join("spdx.json"))
    {
        if let Ok(sbom) = serde_json::from_str::<serde_json::Value>(&sbom_content) {
            sbom.get("components")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter(|comp| {
                            comp.get("type").and_then(|t| t.as_str()) == Some("library")
                                && comp
                                    .get("group")
                                    .and_then(|g| g.as_str())
                                    .map(|g| g.contains('.') || g.contains('/'))
                                    .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        }
    } else {
        0
    };

    // Show container summary
    let summary = ContainerSummary {
        image_name: filtered_results.image_name.clone(),
        image_digest: "sha256:...".to_string(),
        base_image: filtered_results.base_image.clone(),
        total_layers: filtered_results.layers.len(),
        total_size_mb: filtered_results.layers.iter().map(|l| l.size_mb).sum(),
        java_artifacts,
        vulnerabilities: filtered_results.total_vulnerabilities,
        critical_vulns: filtered_results.critical_count,
        high_vulns: filtered_results.high_count,
        medium_vulns: filtered_results.medium_count,
        low_vulns: filtered_results.low_count,
        scan_duration: Duration::from_secs(0),
    };

    summary.print();

    // Analyze and display intelligence
    display_top_fixes(&filtered_results)?;
    display_quick_wins(&filtered_results)?;
    display_action_plan(&filtered_results)?;
    display_remediation_commands(&filtered_results)?;
    display_effort_analysis(&filtered_results)?;
    display_security_score(&filtered_results)?;
    display_priority_scoring(&filtered_results)?;

    println!();

    Ok(())
}

/// Display top 5 fixes by impact - aggregated by package showing total CVEs fixed
#[allow(clippy::type_complexity)]
pub(crate) fn display_top_fixes(results: &ContainerScanResults) -> Result<()> {
    // Aggregate all fixes by package
    let mut package_fixes: HashMap<
        String,
        (String, String, Vec<String>, usize, usize, usize, usize),
    > = HashMap::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                let entry = package_fixes
                    .entry(vuln.package_name.clone())
                    .or_insert_with(|| {
                        (
                            vuln.installed_version.clone(),
                            fixed.clone(),
                            Vec::new(),
                            0,
                            0,
                            0,
                            0,
                        )
                    });
                entry.2.push(vuln.cve_id.clone());
                match vuln.severity.as_str() {
                    "CRITICAL" => entry.3 += 1,
                    "HIGH" => entry.4 += 1,
                    "MEDIUM" => entry.5 += 1,
                    "LOW" => entry.6 += 1,
                    _ => {}
                }
            }
        }
    }

    if package_fixes.is_empty() {
        return Ok(());
    }

    // Convert to vec and sort by total impact (weighted: critical=10, high=5, medium=2, low=1)
    let mut fixes: Vec<_> = package_fixes.into_iter().collect();
    fixes.sort_by(|a, b| {
        let score_a = a.1 .3 * 10 + a.1 .4 * 5 + a.1 .5 * 2 + a.1 .6;
        let score_b = b.1 .3 * 10 + b.1 .4 * 5 + b.1 .5 * 2 + b.1 .6;
        score_b.cmp(&score_a)
    });

    println!();
    println!("{}", "━".repeat(67).bright_white());
    println!("{}", "TOP 5 FIXES BY IMPACT".bright_white().bold());
    println!("{}", "━".repeat(67).bright_white());
    println!();

    for (idx, (package, (current, fixed, cves, crit, high, med, low))) in
        fixes.iter().take(5).enumerate()
    {
        let total = cves.len();
        let severity_breakdown = format!("{}C/{}H/{}M/{}L", crit, high, med, low);

        println!(
            "  {}. {} {} → {}",
            idx + 1,
            package.bright_cyan().bold(),
            current.dimmed(),
            fixed.green().bold()
        );
        println!(
            "     Fixes {} vulnerabilities ({})",
            total.to_string().bright_white().bold(),
            severity_breakdown
        );

        // Show first 3 CVEs
        let show_cves: Vec<_> = cves.iter().take(3).collect();
        if !show_cves.is_empty() {
            let cve_str = show_cves
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            if cves.len() > 3 {
                println!("     CVEs: {} +{} more", cve_str.dimmed(), cves.len() - 3);
            } else {
                println!("     CVEs: {}", cve_str.dimmed());
            }
        }
        println!();
    }

    if fixes.len() > 5 {
        let remaining: usize = fixes
            .iter()
            .skip(5)
            .map(|(_, (_, _, cves, _, _, _, _))| cves.len())
            .sum();
        println!(
            "  {} {} more fixes address {} additional vulnerabilities",
            "...".dimmed(),
            (fixes.len() - 5).to_string().dimmed(),
            remaining.to_string().dimmed()
        );
        println!();
    }

    Ok(())
}

/// Display quick wins - easy fixes with high impact
pub(crate) fn display_quick_wins(results: &ContainerScanResults) -> Result<()> {
    let mut quick_wins = Vec::new();

    // Collect all fixable vulnerabilities that are NOT breaking changes
    for layer in &results.layers {
        let mut package_fixes: HashMap<String, QuickWin> = HashMap::new();

        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                if vuln.breaking_change != Some(true) {
                    let entry = package_fixes
                        .entry(vuln.package_name.clone())
                        .or_insert_with(|| QuickWin {
                            package: vuln.package_name.clone(),
                            current_version: vuln.installed_version.clone(),
                            fixed_version: fixed.clone(),
                            vulns_fixed: Vec::new(),
                            severity: vuln.severity.clone(),
                            estimated_minutes: 5,
                        });
                    entry.vulns_fixed.push(vuln.cve_id.clone());
                }
            }
        }

        quick_wins.extend(package_fixes.into_values());
    }

    if quick_wins.is_empty() {
        return Ok(());
    }

    // Sort by severity and number of vulns fixed
    quick_wins.sort_by(|a, b| {
        let severity_order = |s: &str| match s {
            "CRITICAL" => 0,
            "HIGH" => 1,
            "MEDIUM" => 2,
            "LOW" => 3,
            _ => 4,
        };
        severity_order(&a.severity)
            .cmp(&severity_order(&b.severity))
            .then(b.vulns_fixed.len().cmp(&a.vulns_fixed.len()))
    });

    let total_time: u32 = quick_wins.iter().map(|qw| qw.estimated_minutes).sum();
    let total_vulns: usize = quick_wins.iter().map(|qw| qw.vulns_fixed.len()).sum();

    println!();
    println!("{}", "━".repeat(67).bright_green());
    println!(
        "{}",
        format!(
            "QUICK WINS ({} {}, {} vulns fixed!)",
            total_time,
            if total_time == 1 { "minute" } else { "minutes" },
            total_vulns
        )
        .bright_green()
        .bold()
    );
    println!("{}", "━".repeat(67).bright_green());
    println!();

    for (idx, qw) in quick_wins.iter().take(5).enumerate() {
        println!(
            "  {}. Update {}: {} -> {}",
            idx + 1,
            qw.package.bright_cyan().bold(),
            qw.current_version.dimmed(),
            qw.fixed_version.green().bold()
        );
        println!(
            "     Fixes: {} ({} vulns)",
            qw.vulns_fixed.join(", ").bright_white(),
            qw.vulns_fixed.len()
        );
        println!("     Risk: LOW (patch update)");
        println!("     Time: ~{} minutes", qw.estimated_minutes);
        println!();
    }

    if quick_wins.len() > 5 {
        println!(
            "  {} and {} more quick wins available...",
            "".dimmed(),
            (quick_wins.len() - 5).to_string().dimmed()
        );
        println!();
    }

    Ok(())
}

/// Display prioritized action plan
pub(crate) fn display_action_plan(results: &ContainerScanResults) -> Result<()> {
    // Group vulnerabilities by (package, fixed_version) for consolidation
    let mut grouped_actions: HashMap<(String, String), Vec<&VulnerabilityInfo>> = HashMap::new();

    // Collect all actionable vulnerabilities and group by fix
    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                let key = (vuln.package_name.clone(), fixed.clone());
                grouped_actions.entry(key).or_default().push(vuln);
            }
        }
    }

    if grouped_actions.is_empty() {
        return Ok(());
    }

    // Convert grouped vulns into consolidated ActionItems
    let mut actions = Vec::new();
    for ((package, fixed_version), vulns) in grouped_actions {
        // Take highest priority from group
        let priority_index = vulns
            .iter()
            .map(|v| {
                let p = v.priority.as_deref().unwrap_or("P4");
                match p {
                    "P0" => 0,
                    "P1" => 1,
                    "P2" => 2,
                    "P3" => 3,
                    _ => 4,
                }
            })
            .min()
            .unwrap_or(4);

        let priority = match priority_index {
            0 => "P0",
            1 => "P1",
            2 => "P2",
            3 => "P3",
            _ => "P4",
        }
        .to_string();

        // Collect all CVE IDs
        let cves_fixed: Vec<String> = vulns.iter().map(|v| v.cve_id.clone()).collect();

        // Check if any CVE is KEV or breaking
        let has_kev = vulns.iter().any(|v| v.is_kev);
        let is_breaking = vulns.iter().any(|v| v.breaking_change == Some(true));

        // Max EPSS score
        let max_epss = vulns
            .iter()
            .filter_map(|v| v.epss_score)
            .fold(0.0, f64::max);

        // Estimated hours based on severity
        let estimated_hours = if is_breaking {
            2.0 // Breaking changes take longer
        } else if has_kev {
            1.0 // KEV requires immediate attention
        } else {
            0.25 // Quick patches
        };

        actions.push(ActionItem {
            priority,
            cve_id: cves_fixed[0].clone(), // Primary CVE for display
            cves_fixed,
            package,
            fixed_version,
            description: vulns[0].title.clone(),
            estimated_hours,
            breaking: is_breaking,
            kev: has_kev,
            epss: max_epss,
        });
    }

    // Sort by priority
    actions.sort_by(|a, b| a.priority.cmp(&b.priority));

    println!();
    println!("{}", "━".repeat(67).bright_cyan());
    println!("{}", "RECOMMENDED ACTION PLAN".bright_cyan().bold());
    println!("{}", "━".repeat(67).bright_cyan());
    println!();

    // P0 - Urgent
    let p0_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P0").collect();
    if !p0_actions.is_empty() {
        println!("{}", "URGENT (Do TODAY):".red().bold());
        for (idx, action) in p0_actions.iter().take(3).enumerate() {
            println!(
                "  {}. {} Update {} -> {}",
                idx + 1,
                if action.kev {
                    "[P0/KEV]".red().bold()
                } else {
                    "[P0]".red().bold()
                },
                action.package.bright_cyan().bold(),
                action.fixed_version.green().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                action.cves_fixed.join(", ").bright_white(),
                action.cves_fixed.len(),
                if action.cves_fixed.len() == 1 {
                    "vuln"
                } else {
                    "vulns"
                }
            );
            println!("     Est: {}", format_time(action.estimated_hours));
            if action.breaking {
                println!("     Breaking change - review migration guide");
            }
            if action.epss > 0.5 {
                println!(
                    "     EPSS: {:.0}% (high exploitation risk)",
                    action.epss * 100.0
                );
            }
            println!();
        }
    }

    // P1 - High Priority
    let p1_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P1").collect();
    if !p1_actions.is_empty() {
        println!("{}", "HIGH PRIORITY (This week):".yellow().bold());
        for (idx, action) in p1_actions.iter().take(3).enumerate() {
            println!(
                "  {}. [P1] Update {} -> {}",
                p0_actions.len() + idx + 1,
                action.package.bright_cyan().bold(),
                action.fixed_version.green().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                action.cves_fixed.join(", ").bright_white(),
                action.cves_fixed.len(),
                if action.cves_fixed.len() == 1 {
                    "vuln"
                } else {
                    "vulns"
                }
            );
            println!("     Est: {}", format_time(action.estimated_hours));
            println!();
        }
    }

    // P2 - Medium Priority
    let p2_actions: Vec<&ActionItem> = actions.iter().filter(|a| a.priority == "P2").collect();
    if !p2_actions.is_empty() && !p2_actions.is_empty() {
        println!("{}", "MEDIUM PRIORITY (This sprint):".yellow());
        println!("  {} vulnerabilities requiring attention", p2_actions.len());
        println!(
            "  Estimated total: {}",
            format_time(p2_actions.iter().map(|a| a.estimated_hours).sum())
        );
        println!();
    }

    Ok(())
}

/// Display copy-paste remediation commands
pub(crate) fn display_remediation_commands(results: &ContainerScanResults) -> Result<()> {
    // Group fixes by (package, fixed_version) to deduplicate
    let mut java_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut python_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut js_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut go_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut rust_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut ruby_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();
    let mut php_fixes: HashMap<(String, String), (String, Vec<String>)> = HashMap::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref fixed) = vuln.fixed_version {
                let ecosystem = detect_ecosystem(&vuln.package_name);
                let key = (vuln.package_name.clone(), fixed.clone());

                let fixes_map = match ecosystem {
                    PackageEcosystem::Java => &mut java_fixes,
                    PackageEcosystem::Python => &mut python_fixes,
                    PackageEcosystem::JavaScript => &mut js_fixes,
                    PackageEcosystem::Go => &mut go_fixes,
                    PackageEcosystem::Rust => &mut rust_fixes,
                    PackageEcosystem::Ruby => &mut ruby_fixes,
                    PackageEcosystem::Php => &mut php_fixes,
                    PackageEcosystem::Other => continue,
                };

                fixes_map
                    .entry(key)
                    .or_insert_with(|| (vuln.installed_version.clone(), Vec::new()))
                    .1
                    .push(vuln.cve_id.clone());
            }
        }
    }

    // If no fixes available, skip
    if java_fixes.is_empty()
        && python_fixes.is_empty()
        && js_fixes.is_empty()
        && go_fixes.is_empty()
        && rust_fixes.is_empty()
        && ruby_fixes.is_empty()
        && php_fixes.is_empty()
    {
        return Ok(());
    }

    println!();
    println!("{}", "━".repeat(67).bright_magenta());
    println!("{}", "COPY-PASTE FIXES".bright_magenta().bold());
    println!("{}", "━".repeat(67).bright_magenta());
    println!();

    // Display Java fixes
    if !java_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = java_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            let parts: Vec<&str> = package.split(':').collect();
            let (group_id, artifact_id) = if parts.len() >= 2 {
                (parts[0], parts[1])
            } else {
                (package.as_str(), package.as_str())
            };

            println!(
                "  {} Package: {}",
                "Java".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "Maven (pom.xml):".bright_white().bold());
            println!("  {}", "```xml".dimmed());
            println!("  <dependency>");
            println!("    <groupId>{}</groupId>", group_id.bright_white());
            println!(
                "    <artifactId>{}</artifactId>",
                artifact_id.bright_white()
            );
            println!("    <version>{}</version>", fixed.green().bold());
            println!("  </dependency>");
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Gradle (build.gradle):".bright_white().bold());
            println!("  {}", "```groovy".dimmed());
            println!(
                "  implementation '{}:{}:{}'",
                group_id.bright_white(),
                artifact_id.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display Python fixes
    if !python_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = python_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "Python".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "requirements.txt:".bright_white().bold());
            println!("  {}", "```".dimmed());
            println!("  {}=={}", package.bright_white(), fixed.green().bold());
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "pyproject.toml (Poetry):".bright_white().bold());
            println!("  {}", "```toml".dimmed());
            println!(
                "  {} = \"^{}\"",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Pipfile:".bright_white().bold());
            println!("  {}", "```toml".dimmed());
            println!(
                "  {} = \"=={}\"",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display JavaScript/Node fixes
    if !js_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = js_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "JS".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "package.json:".bright_white().bold());
            println!("  {}", "```json".dimmed());
            println!("  \"dependencies\": {{");
            println!(
                "    \"{}\": \"^{}\"",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  }}");
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "npm:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!(
                "  npm install {}@{}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "yarn:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!(
                "  yarn add {}@{}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display Go fixes
    if !go_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = go_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "Go".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "go.mod:".bright_white().bold());
            println!("  {}", "```".dimmed());
            println!(
                "  require {} {}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Command:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!(
                "  go get {}@{}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display Rust fixes
    if !rust_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = rust_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "Rust".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "Cargo.toml:".bright_white().bold());
            println!("  {}", "```toml".dimmed());
            println!("  [dependencies]");
            println!(
                "  {} = \"{}\"",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Command:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!(
                "  cargo add {}@{}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display Ruby fixes
    if !ruby_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = ruby_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "Ruby".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "Gemfile:".bright_white().bold());
            println!("  {}", "```ruby".dimmed());
            println!(
                "  gem '{}', '{}'",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Command:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!("  bundle update {}", package.bright_white());
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    // Display PHP fixes
    if !php_fixes.is_empty() {
        let mut fixes_vec: Vec<_> = php_fixes.into_iter().collect();
        fixes_vec.truncate(2);
        for ((package, fixed), (_current, cves)) in fixes_vec {
            println!(
                "  {} Package: {}",
                "PHP".bright_yellow(),
                package.bright_cyan().bold()
            );
            println!(
                "     Fixes: {} ({} {})",
                cves.join(", ").bright_white(),
                cves.len(),
                if cves.len() == 1 { "vuln" } else { "vulns" }
            );
            println!();
            println!("  {}", "composer.json:".bright_white().bold());
            println!("  {}", "```json".dimmed());
            println!("  \"require\": {{");
            println!(
                "    \"{}\": \"^{}\"",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  }}");
            println!("  {}", "```".dimmed());
            println!();
            println!("  {}", "Command:".bright_white().bold());
            println!("  {}", "```bash".dimmed());
            println!(
                "  composer require {}:{}",
                package.bright_white(),
                fixed.green().bold()
            );
            println!("  {}", "```".dimmed());
            println!();
        }
    }

    Ok(())
}

/// Display effort analysis
pub(crate) fn display_effort_analysis(results: &ContainerScanResults) -> Result<()> {
    let mut p0_time = 0.0;
    let mut p1_time = 0.0;
    let mut p2_time = 0.0;
    let mut p0_count = 0;
    let mut p1_count = 0;
    let mut p2_count = 0;

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if vuln.fixed_version.is_none() {
                continue;
            }

            let time = if vuln.breaking_change == Some(true) {
                2.0
            } else if vuln.is_kev {
                1.0
            } else {
                0.25
            };

            match vuln.priority.as_deref() {
                Some("P0") => {
                    p0_time += time;
                    p0_count += 1;
                }
                Some("P1") => {
                    p1_time += time;
                    p1_count += 1;
                }
                Some("P2") => {
                    p2_time += time;
                    p2_count += 1;
                }
                _ => {}
            }
        }
    }

    if p0_count == 0 && p1_count == 0 && p2_count == 0 {
        return Ok(());
    }

    println!();
    println!("{}", "━".repeat(67).bright_blue());
    println!("{}", "REMEDIATION EFFORT SUMMARY".bright_blue().bold());
    println!("{}", "━".repeat(67).bright_blue());
    println!();

    if p0_count > 0 {
        println!(
            "  P0 Fixes: ~{} ({} {})",
            format_time(p0_time).red().bold(),
            p0_count,
            if p0_count == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
    }
    if p1_count > 0 {
        println!(
            "  P1 Fixes: ~{} ({} {})",
            format_time(p1_time).yellow().bold(),
            p1_count,
            if p1_count == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
    }
    if p2_count > 0 {
        println!(
            "  P2 Fixes: ~{} ({} {})",
            format_time(p2_time).yellow(),
            p2_count,
            if p2_count == 1 {
                "vulnerability"
            } else {
                "vulnerabilities"
            }
        );
    }

    let total_time = p0_time + p1_time + p2_time;
    println!();
    println!(
        "  Total estimated time: {}",
        format_time(total_time).bright_white().bold()
    );
    println!(
        "  Risk reduction: {} -> {}",
        if results.critical_count > 0 {
            "CRITICAL".red().bold()
        } else {
            "HIGH".yellow().bold()
        },
        "LOW".green().bold()
    );
    println!();

    Ok(())
}

/// Display security score
pub(crate) fn display_security_score(results: &ContainerScanResults) -> Result<()> {
    // Calculate security score (0-100)
    let mut score: i32 = 100;

    // Deduct points for vulnerabilities
    score -= (results.critical_count.min(10) * 10) as i32;
    score -= (results.high_count.min(20) * 2) as i32;
    score -= results.medium_count.min(50) as i32;

    // Extra penalty for KEV
    let kev_count = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.is_kev)
        .count();
    score -= (kev_count.min(5) * 5) as i32;

    let score = score.max(0) as usize;

    let rating = if score >= 90 {
        ("Excellent", "A+".green())
    } else if score >= 75 {
        ("Good", "B".green())
    } else if score >= 60 {
        ("Acceptable", "C".yellow())
    } else if score >= 40 {
        ("Needs Work", "D".yellow())
    } else {
        ("Critical", "F".red())
    };

    println!();
    println!("{}", "━".repeat(67).bright_yellow());
    println!("{}", "SECURITY SCORE".bright_yellow().bold());
    println!("{}", "━".repeat(67).bright_yellow());
    println!();

    let score_color = if score >= 75 {
        score.to_string().green().bold()
    } else if score >= 60 {
        score.to_string().yellow().bold()
    } else {
        score.to_string().red().bold()
    };

    println!(
        "  Score: {}{} - {} {}",
        score_color,
        "/100".dimmed(),
        rating.1,
        rating.0.bright_white().bold()
    );
    println!();

    // Show what would improve the score
    if score < 90 {
        println!("{}", "  To improve:".bright_white().bold());
        if kev_count > 0 {
            println!(
                "    * Fix {} KEV {}: +{} points",
                kev_count,
                if kev_count == 1 {
                    "vulnerability"
                } else {
                    "vulnerabilities"
                },
                kev_count * 5
            );
        }
        if results.critical_count > 0 {
            println!(
                "    * Fix {} CRITICAL {}: +{} points",
                results.critical_count.min(3),
                if results.critical_count == 1 {
                    "vulnerability"
                } else {
                    "vulnerabilities"
                },
                results.critical_count.min(3) * 10
            );
        }
        if results.high_count > 0 {
            println!(
                "    * Fix {} HIGH {}: +{} points",
                results.high_count.min(5),
                if results.high_count == 1 {
                    "vulnerability"
                } else {
                    "vulnerabilities"
                },
                results.high_count.min(5) * 2
            );
        }
        println!();
    }

    println!("  {} Industry average: 65/100", "Info:".dimmed());
    let target_score = ((score + 15).min(95) / 5) * 5;
    println!("  {} Target: {}/100", "Goal:".dimmed(), target_score);
    println!();

    Ok(())
}

/// Display priority scoring breakdown
pub(crate) fn display_priority_scoring(results: &ContainerScanResults) -> Result<()> {
    // Calculate component scores based on vulnerability analysis

    // Impact Score (max 42): Data sensitivity, operational disruption, reputation, production, sentiment
    let mut impact_score = 0;
    let mut impact_rationales = Vec::new();

    // Critical vulns = high operational disruption
    if results.critical_count > 0 {
        impact_score += 10;
        impact_rationales.push("Critical vulnerabilities can disrupt production (10pts)");
    } else if results.high_count >= 3 {
        impact_score += 6;
        impact_rationales.push("Multiple HIGH vulns may degrade service (6pts)");
    } else if results.medium_count >= 10 {
        impact_score += 3;
        impact_rationales.push("Moderate vuln volume implies minimal disruption (3pts)");
    }

    // KEV or secrets = reputation damage
    let kev_count = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.is_kev)
        .count();
    if kev_count > 0 {
        impact_score += 9;
        impact_rationales.push("Active exploitation damages trust (9pts)");
    } else {
        impact_score += 5;
        impact_rationales.push("Potential reputation impact from disclosed vulns (5pts)");
    }

    // Public sentiment (KEV = high attention)
    if kev_count > 0 {
        impact_score += 3;
        impact_rationales.push("Listed on CISA KEV (High public sentiment) (3pts)");
    }

    // Assume production environment for containers
    impact_score += 10;
    impact_rationales.push("Container image assumed production-facing (10pts)");

    // Data sensitivity - assume customer data exposure risk
    impact_score += 6;
    impact_rationales.push("Default to internal/customer data exposure (6pts)");

    // Probability Score (max 31): Exposure, active exploitation, exploit availability
    let mut probability_score = 0;
    let mut probability_rationales = Vec::new();

    // Assume internet-facing
    probability_score += 10;
    probability_rationales.push("Internet-facing attack surface (10pts)");

    // Active exploitation (KEV or high EPSS)
    let max_epss = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter_map(|v| v.epss_score)
        .fold(0.0, f64::max);

    if kev_count > 0 {
        probability_score += 10;
        probability_rationales.push("Confirmed exploited vulnerability (CISA KEV) (10pts)");
    } else if max_epss >= 0.5 {
        probability_score += 6;
        probability_rationales.push("High EPSS probability (>=0.5) (6pts)");
    } else if max_epss >= 0.2 {
        probability_score += 3;
        probability_rationales.push("Observed exploit interest (EPSS >=0.2) (3pts)");
    }

    // Exploit availability
    if kev_count > 0 || max_epss >= 0.9 {
        probability_score += 9;
        probability_rationales.push("Public exploit / automated tooling available (9pts)");
    } else if results.critical_count + results.high_count > 0 {
        probability_score += 6;
        probability_rationales.push("Detailed advisories exist for high severity vulns (6pts)");
    } else if results.medium_count > 0 {
        probability_score += 3;
        probability_rationales.push("Vendor guidance only (3pts)");
    }

    // Complexity Score (max 13): User interaction, authentication
    let mut complexity_score = 0;
    let mut complexity_rationales = Vec::new();

    // No user interaction for critical vulns
    if results.critical_count > 0 {
        complexity_score += 7;
        complexity_rationales.push("No user interaction required (7pts)");
    } else if results.high_count > 0 {
        complexity_score += 4;
        complexity_rationales.push("Minimal interaction expected (4pts)");
    }

    // Assume no authentication for container services
    complexity_score += 6;
    complexity_rationales.push("No authentication enforced (6pts)");

    // Exploitability Score (max 14): Code path reachability
    let mut exploitability_score = 0;
    let mut exploitability_rationales = Vec::new();

    // Reachability analysis
    let reachable_count = results
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .filter(|v| v.is_reachable)
        .count();

    if kev_count > 0 || results.critical_count > 0 {
        exploitability_score += 14;
        exploitability_rationales.push("Vulnerable path actively exploited/critical (14pts)");
    } else if results.high_count >= 3 {
        exploitability_score += 9;
        exploitability_rationales.push("Vulnerable path reachable under common workflows (9pts)");
    } else if results.medium_count > 0 || reachable_count > 0 {
        exploitability_score += 5;
        exploitability_rationales.push("Path reachable under constrained conditions (5pts)");
    }

    // Calculate total and determine priority
    let total_score = impact_score + probability_score + complexity_score + exploitability_score;

    let (priority_level, ticket_type) = if total_score >= 80 {
        ("P1 - Show Stopper", "Incident")
    } else if total_score >= 60 {
        ("P2 - Critical", "Bug")
    } else if total_score >= 40 {
        ("P3 - Major", "Bug")
    } else if total_score >= 20 {
        ("P4 - Normal", "Bug")
    } else {
        ("P5 - Minor", "Bug")
    };

    // Display the scoring
    println!();
    println!("{}", "━".repeat(67).bright_magenta());
    println!("{}", "PRIORITY SCORING BREAKDOWN".bright_magenta().bold());
    println!("{}", "━".repeat(67).bright_magenta());
    println!();

    // Overall priority
    let priority_color = if total_score >= 80 {
        priority_level.red().bold()
    } else if total_score >= 60 {
        priority_level.yellow().bold()
    } else if total_score >= 40 {
        priority_level.yellow()
    } else {
        priority_level.green()
    };

    println!(
        "  Overall Priority: {} ({} pts)",
        priority_color, total_score
    );
    println!("  Ticket Type: {}", ticket_type.dimmed());
    println!();

    // Component breakdown
    println!("  Component Scores:");
    println!(
        "    Impact:        {}/42 {}",
        impact_score.to_string().bright_white().bold(),
        "(data sensitivity, disruption, reputation)"
            .to_string()
            .dimmed()
    );
    println!(
        "    Probability:   {}/31 {}",
        probability_score.to_string().bright_white().bold(),
        "(exposure, exploitation, availability)"
            .to_string()
            .dimmed()
    );
    println!(
        "    Complexity:    {}/13 {}",
        complexity_score.to_string().bright_white().bold(),
        "(user interaction, authentication)".to_string().dimmed()
    );
    println!(
        "    Exploitability:{}/14 {}",
        exploitability_score.to_string().bright_white().bold(),
        "(code path reachability)".to_string().dimmed()
    );
    println!();

    // Top rationales
    println!("  Key Factors:");

    // Show most impactful rationales
    let mut all_rationales: Vec<&str> = Vec::new();
    all_rationales.extend(
        impact_rationales
            .iter()
            .filter(|r| r.contains("10pts") || r.contains("9pts"))
            .copied(),
    );
    all_rationales.extend(
        probability_rationales
            .iter()
            .filter(|r| r.contains("10pts") || r.contains("9pts"))
            .copied(),
    );
    all_rationales.extend(
        exploitability_rationales
            .iter()
            .filter(|r| r.contains("14pts") || r.contains("9pts"))
            .copied(),
    );

    for rationale in all_rationales.iter().take(5) {
        println!("    • {}", rationale.dimmed());
    }

    println!();

    // Priority thresholds reference
    println!("  Priority Thresholds:");
    println!("    P1 (Incident): ≥80 | P2 (Critical): ≥60 | P3 (Major): ≥40 | P4 (Normal): ≥20");
    println!();

    Ok(())
}

/// Format time in human-readable form
pub(crate) fn format_time(hours: f32) -> String {
    if hours < 1.0 {
        format!("{} minutes", (hours * 60.0) as u32)
    } else if hours == 1.0 {
        "1 hour".to_string()
    } else {
        format!("{:.1} hours", hours)
    }
}

/// Save scan results as baseline
pub(crate) fn save_baseline(results: &ContainerScanResults, image_name: &str) -> Result<()> {
    let baseline_dir = PathBuf::from(".bazbom/baselines");
    std::fs::create_dir_all(&baseline_dir)?;

    let filename = image_name.replace([':', '/'], "_");
    let baseline_path = baseline_dir.join(format!("{}.json", filename));

    let json = serde_json::to_string_pretty(results)?;
    std::fs::write(&baseline_path, json)?;

    Ok(())
}

/// Load baseline scan results
pub(crate) fn load_baseline(image_name: &str) -> Result<ContainerScanResults> {
    let filename = image_name.replace([':', '/'], "_");
    let baseline_path = PathBuf::from(format!(".bazbom/baselines/{}.json", filename));

    let content = std::fs::read_to_string(&baseline_path)?;
    let results: ContainerScanResults = serde_json::from_str(&content)?;

    Ok(results)
}

/// Display baseline comparison
pub(crate) fn display_baseline_comparison(
    baseline: &ContainerScanResults,
    current: &ContainerScanResults,
) -> Result<()> {
    println!();
    println!("{}", "━".repeat(67).bright_blue());
    println!("{}", "BASELINE COMPARISON".bright_blue().bold());
    println!("{}", "━".repeat(67).bright_blue());
    println!();

    let crit_diff = current.critical_count as i32 - baseline.critical_count as i32;
    let high_diff = current.high_count as i32 - baseline.high_count as i32;
    let total_diff = current.total_vulnerabilities as i32 - baseline.total_vulnerabilities as i32;

    println!(
        "  Baseline vulnerabilities: {}",
        baseline.total_vulnerabilities
    );
    println!(
        "  Current vulnerabilities:  {}",
        current.total_vulnerabilities
    );
    println!();

    let change_icon = if total_diff < 0 {
        "IMPROVED".green()
    } else if total_diff > 0 {
        "WORSE".red()
    } else {
        "SAME".normal()
    };

    println!(
        "  {} Total change: {}{}",
        change_icon,
        if total_diff > 0 { "+" } else { "" },
        total_diff.to_string().bright_white().bold()
    );

    if crit_diff != 0 {
        println!(
            "     CRITICAL: {}{}",
            if crit_diff > 0 { "+" } else { "" },
            if crit_diff > 0 {
                crit_diff.to_string().red().bold()
            } else {
                crit_diff.to_string().green().bold()
            }
        );
    }

    if high_diff != 0 {
        println!(
            "     HIGH:     {}{}",
            if high_diff > 0 { "+" } else { "" },
            if high_diff > 0 {
                high_diff.to_string().yellow().bold()
            } else {
                high_diff.to_string().green().bold()
            }
        );
    }

    // Show new CVEs
    let baseline_cves: std::collections::HashSet<String> = baseline
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .map(|v| v.cve_id.clone())
        .collect();

    let current_cves: std::collections::HashSet<String> = current
        .layers
        .iter()
        .flat_map(|l| &l.vulnerabilities)
        .map(|v| v.cve_id.clone())
        .collect();

    let new_cves: Vec<_> = current_cves.difference(&baseline_cves).collect();
    let fixed_cves: Vec<_> = baseline_cves.difference(&current_cves).collect();

    if !new_cves.is_empty() {
        println!();
        println!("  New vulnerabilities:");
        for cve in new_cves.iter().take(5) {
            println!("     * {}", cve.red());
        }
        if new_cves.len() > 5 {
            println!(
                "     {} and {} more...",
                "".dimmed(),
                (new_cves.len() - 5).to_string().dimmed()
            );
        }
    }

    if !fixed_cves.is_empty() {
        println!();
        println!("  Fixed vulnerabilities:");
        for cve in fixed_cves.iter().take(5) {
            println!("     * {}", cve.green());
        }
        if fixed_cves.len() > 5 {
            println!(
                "     {} and {} more...",
                "".dimmed(),
                (fixed_cves.len() - 5).to_string().dimmed()
            );
        }
    }

    println!();

    Ok(())
}

/// Display image comparison
pub(crate) fn display_image_comparison(
    image1: &ContainerScanResults,
    image2: &ContainerScanResults,
) -> Result<()> {
    println!();
    println!("{}", "━".repeat(67).bright_magenta());
    println!("{}", "IMAGE COMPARISON".bright_magenta().bold());
    println!("{}", "━".repeat(67).bright_magenta());
    println!();

    println!("  Image 1: {}", image1.image_name.bright_cyan().bold());
    println!("  Image 2: {}", image2.image_name.bright_cyan().bold());
    println!();

    println!(
        "  {:<30} {:>15} {:>15}",
        "Metric".bold(),
        "Image 1".bold(),
        "Image 2".bold()
    );
    println!("  {}", "-".repeat(67).dimmed());
    println!(
        "  {:<30} {:>15} {:>15}",
        "Total Packages", image1.total_packages, image2.total_packages
    );
    println!(
        "  {:<30} {:>15} {:>15}",
        "Total Vulnerabilities", image1.total_vulnerabilities, image2.total_vulnerabilities
    );
    println!(
        "  {:<30} {:>15} {:>15}",
        "CRITICAL", image1.critical_count, image2.critical_count
    );
    println!(
        "  {:<30} {:>15} {:>15}",
        "HIGH", image1.high_count, image2.high_count
    );
    println!(
        "  {:<30} {:>15} {:>15}",
        "MEDIUM", image1.medium_count, image2.medium_count
    );
    println!(
        "  {:<30} {:>15} {:>15}",
        "LOW", image1.low_count, image2.low_count
    );
    println!();

    // Recommendation
    let total1 = image1.total_vulnerabilities;
    let total2 = image2.total_vulnerabilities;
    let crit1 = image1.critical_count;
    let crit2 = image2.critical_count;

    if total1 < total2 || (total1 == total2 && crit1 < crit2) {
        println!("  Recommendation: Use {}", image1.image_name.green().bold());
        println!("     Fewer vulnerabilities and lower severity");
    } else if total2 < total1 || (total1 == total2 && crit2 < crit1) {
        println!("  Recommendation: Use {}", image2.image_name.green().bold());
        println!("     Fewer vulnerabilities and lower severity");
    } else {
        println!("  Both images have similar security profiles");
    }

    println!();

    Ok(())
}

/// Create GitHub issues for vulnerabilities
pub(crate) fn create_github_issues(results: &ContainerScanResults, repo: &str) -> Result<()> {
    let gh_check = Command::new("gh").arg("--version").output();

    if gh_check.is_err() {
        anyhow::bail!("GitHub CLI (gh) not found. Install from: https://cli.github.com/");
    }

    let mut high_priority_vulns = Vec::new();

    for layer in &results.layers {
        for vuln in &layer.vulnerabilities {
            if let Some(ref priority) = vuln.priority {
                if priority == "P0" || priority == "P1" {
                    high_priority_vulns.push(vuln.clone());
                }
            }
        }
    }

    if high_priority_vulns.is_empty() {
        println!("   No P0/P1 vulnerabilities found. Nothing to create.");
        return Ok(());
    }

    let mut seen_cves = std::collections::HashSet::new();
    let mut unique_vulns = Vec::new();
    for vuln in high_priority_vulns {
        if seen_cves.insert(vuln.cve_id.clone()) {
            unique_vulns.push(vuln);
        }
    }

    println!(
        "   Creating {} issues in {}...",
        unique_vulns.len(),
        repo.bright_cyan()
    );

    for vuln in unique_vulns.iter().take(10) {
        let title = format!(
            "[Security] {} in {} ({})",
            vuln.cve_id,
            vuln.package_name,
            vuln.priority.as_ref().unwrap_or(&"P2".to_string())
        );

        let body = format!(
            "## Vulnerability Details\n\n\
             **CVE:** {}\n\
             **Package:** {} ({})\n\
             **Severity:** {}\n\
             **Priority:** {}\n\n\
             ## Description\n\n{}\n\n\
             ## Remediation\n\n{}\n\n\
             ## References\n\n{}\n\n\
             ---\n\
             *Automatically generated by BazBOM container-scan*",
            vuln.cve_id,
            vuln.package_name,
            vuln.installed_version,
            vuln.severity,
            vuln.priority.as_ref().unwrap_or(&"P2".to_string()),
            vuln.description,
            if let Some(ref fixed) = vuln.fixed_version {
                format!("Upgrade to version {}", fixed)
            } else {
                "No fix available yet. Monitor for updates.".to_string()
            },
            vuln.references.join("\n")
        );

        let output = Command::new("gh")
            .args([
                "issue", "create", "--repo", repo, "--title", &title, "--body", &body, "--label",
                "security",
            ])
            .output()?;

        if output.status.success() {
            let url = String::from_utf8_lossy(&output.stdout);
            println!("   Created: {}", url.trim().bright_green());
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            println!(
                "   Failed to create issue for {}: {}",
                vuln.cve_id,
                err.red()
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::container_scan::{ContainerScanResults, LayerInfo, VulnerabilityInfo};

    fn make_test_vuln(
        cve: &str,
        severity: &str,
        priority: Option<&str>,
        fixed: Option<&str>,
        kev: bool,
        breaking: Option<bool>,
    ) -> VulnerabilityInfo {
        VulnerabilityInfo {
            cve_id: cve.to_string(),
            package_name: "test-pkg".to_string(),
            installed_version: "1.0.0".to_string(),
            fixed_version: fixed.map(String::from),
            severity: severity.to_string(),
            cvss_score: Some(7.5),
            title: "Test vulnerability".to_string(),
            description: "Test description".to_string(),
            layer_digest: "sha256:abc123".to_string(),
            published_date: None,
            epss_score: None,
            epss_percentile: None,
            is_kev: kev,
            kev_due_date: None,
            priority: priority.map(String::from),
            references: vec![],
            breaking_change: breaking,
            upgrade_path: None,
            is_reachable: false,
            difficulty_score: None,
            call_chain: None,
            dependency_path: None,
        }
    }

    fn make_test_results(vulns: Vec<VulnerabilityInfo>) -> ContainerScanResults {
        let critical = vulns.iter().filter(|v| v.severity == "CRITICAL").count();
        let high = vulns.iter().filter(|v| v.severity == "HIGH").count();
        let medium = vulns.iter().filter(|v| v.severity == "MEDIUM").count();
        let low = vulns.iter().filter(|v| v.severity == "LOW").count();

        ContainerScanResults {
            image_name: "test:latest".to_string(),
            base_image: None,
            layers: vec![LayerInfo {
                digest: "sha256:abc123".to_string(),
                size_mb: 10.0,
                packages: vec![],
                vulnerabilities: vulns.clone(),
            }],
            total_packages: 1,
            total_vulnerabilities: vulns.len(),
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
            upgrade_recommendations: vec![],
            reachability_summary: None,
            compliance_results: None,
        }
    }

    #[test]
    fn test_format_time_minutes() {
        assert_eq!(format_time(0.5), "30 minutes");
        assert_eq!(format_time(0.25), "15 minutes");
    }

    #[test]
    fn test_format_time_one_hour() {
        assert_eq!(format_time(1.0), "1 hour");
    }

    #[test]
    fn test_format_time_hours() {
        assert_eq!(format_time(2.5), "2.5 hours");
        assert_eq!(format_time(8.0), "8.0 hours");
    }

    #[test]
    fn test_apply_filter_severity_critical() {
        let vulns = vec![
            make_test_vuln("CVE-1", "CRITICAL", None, None, false, None),
            make_test_vuln("CVE-2", "HIGH", None, None, false, None),
            make_test_vuln("CVE-3", "MEDIUM", None, None, false, None),
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "critical").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 1);
        assert_eq!(filtered.critical_count, 1);
        assert_eq!(filtered.high_count, 0);
    }

    #[test]
    fn test_apply_filter_severity_high() {
        let vulns = vec![
            make_test_vuln("CVE-1", "HIGH", None, None, false, None),
            make_test_vuln("CVE-2", "HIGH", None, None, false, None),
            make_test_vuln("CVE-3", "LOW", None, None, false, None),
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "high").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 2);
        assert_eq!(filtered.high_count, 2);
    }

    #[test]
    fn test_apply_filter_priority_p0() {
        let vulns = vec![
            make_test_vuln("CVE-1", "CRITICAL", Some("P0"), None, false, None),
            make_test_vuln("CVE-2", "HIGH", Some("P1"), None, false, None),
            make_test_vuln("CVE-3", "MEDIUM", Some("P2"), None, false, None),
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "p0").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 1);
    }

    #[test]
    fn test_apply_filter_fixable() {
        let vulns = vec![
            make_test_vuln("CVE-1", "HIGH", None, Some("2.0.0"), false, None),
            make_test_vuln("CVE-2", "HIGH", None, None, false, None),
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "fixable").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 1);
    }

    #[test]
    fn test_apply_filter_quick_wins() {
        let vulns = vec![
            make_test_vuln("CVE-1", "HIGH", None, Some("2.0.0"), false, Some(false)), // Quick win
            make_test_vuln("CVE-2", "HIGH", None, Some("2.0.0"), false, Some(true)),  // Breaking
            make_test_vuln("CVE-3", "HIGH", None, None, false, None),                 // No fix
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "quick-wins").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 1);
    }

    #[test]
    fn test_apply_filter_kev() {
        let vulns = vec![
            make_test_vuln("CVE-1", "CRITICAL", None, None, true, None), // KEV
            make_test_vuln("CVE-2", "HIGH", None, None, false, None),    // Not KEV
        ];
        let results = make_test_results(vulns);

        let filtered = apply_filter(&results, "kev").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 1);
    }

    #[test]
    fn test_apply_filter_unknown() {
        let vulns = vec![
            make_test_vuln("CVE-1", "HIGH", None, None, false, None),
            make_test_vuln("CVE-2", "LOW", None, None, false, None),
        ];
        let results = make_test_results(vulns);

        // Unknown filter should return all
        let filtered = apply_filter(&results, "unknown-filter").unwrap();
        assert_eq!(filtered.total_vulnerabilities, 2);
    }
}
