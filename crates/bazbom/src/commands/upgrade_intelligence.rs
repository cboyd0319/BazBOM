use anyhow::{Context, Result};
use bazbom_upgrade_analyzer::UpgradeAnalyzer;
use colored::*;

/// Show detailed upgrade impact analysis for a package
pub async fn explain_upgrade(package: &str) -> Result<()> {
    println!("\n{}", "━".repeat(60).bright_black());
    println!("{}", format!("Upgrade Intelligence: {}", package).bold());
    println!("{}\n", "━".repeat(60).bright_black());

    // Parse package and versions from findings or pom.xml
    // For now, we'll use example data
    let (current_version, target_version) = find_upgrade_versions(package)?;

    println!("{}", "📊 Analyzing upgrade impact...".cyan());
    println!("   {} {} → {}\n", package, current_version, target_version);

    // Create analyzer and run analysis
    let mut analyzer = UpgradeAnalyzer::new()?;
    let analysis = analyzer
        .analyze_upgrade(package, &current_version, &target_version)
        .await?;

    // Print results
    print_upgrade_analysis(&analysis);

    Ok(())
}

fn print_upgrade_analysis(analysis: &bazbom_upgrade_analyzer::UpgradeAnalysis) {
    use bazbom_upgrade_analyzer::RiskLevel;

    println!("{}", "━".repeat(60).bright_black());
    println!("{}", format!("Upgrade Analysis: {} {} → {}",
        analysis.target_package,
        analysis.from_version,
        analysis.to_version
    ).bold());
    println!("{}", "━".repeat(60).bright_black());
    println!();

    // Overall risk
    println!("{} Overall Risk: {}",
        "🔍".bold(),
        format_risk_level(analysis.overall_risk)
    );
    println!();

    // Direct changes
    println!("{} Direct Changes ({} itself):",
        "📦".bold(),
        analysis.target_package
    );

    if analysis.direct_breaking_changes.is_empty() {
        println!("   {} Breaking changes: 0", "✅");
        println!("   {} API compatibility: 100%", "✅");
        println!("   {} Risk: {}", "✅", format_risk_level(RiskLevel::Low));
    } else {
        println!("   {} Breaking changes: {}", "⚠️ ", analysis.direct_breaking_changes.len());
        println!();
        for change in &analysis.direct_breaking_changes {
            println!("   {} {}", "•".yellow(), change.description);
            if let Some(hint) = &change.migration_hint {
                println!("     {} {}", "💡".bright_blue(), hint.dimmed());
            }
        }
    }
    println!();

    // Required dependency upgrades
    if !analysis.required_upgrades.is_empty() {
        println!("{} Required Dependency Upgrades: {}",
            "⚠️ ".bold(),
            analysis.required_upgrades.len()
        );
        println!();

        for upgrade in &analysis.required_upgrades {
            let status_icon = if upgrade.breaking_changes.is_empty() {
                "✅"
            } else {
                upgrade.risk_level.emoji()
            };

            println!("   {} {}: {} → {}",
                status_icon,
                upgrade.package,
                upgrade.from_version,
                upgrade.to_version
            );
            println!("      Reason: {}", upgrade.reason.to_string().dimmed());

            if !upgrade.breaking_changes.is_empty() {
                println!("      {} {} breaking changes:",
                    "⚠️ ",
                    upgrade.breaking_changes.len()
                );

                for change in &upgrade.breaking_changes {
                    println!("      {} {}", "•".yellow(), change.description);
                }
            }
            println!();
        }
    }

    // Compatibility notes
    if !analysis.compatibility_notes.is_empty() {
        println!("{}", "━".repeat(60).bright_black());
        println!();
        println!("{} Compatibility Notes:", "ℹ️ ".bold());
        for note in &analysis.compatibility_notes {
            println!("   {} {}", "•".cyan(), note);
        }
        println!();
    }

    // Migration guide
    if let Some(ref guide_url) = analysis.migration_guide_url {
        println!("{} Migration Guide:", "📄".bold());
        println!("   {}", guide_url.bright_blue().underline());
        println!();
    }

    // GitHub repo
    if let Some(ref repo_url) = analysis.github_repo {
        println!("{} Repository:", "🔗".bold());
        println!("   {}", repo_url.bright_blue().underline());
        println!();
    }

    // Summary
    println!("{}", "━".repeat(60).bright_black());
    println!();
    println!("{} Impact Summary:", "📊".bold());
    println!("   {} Direct breaking changes: {}", "├─", analysis.direct_breaking_changes.len());
    println!("   {} Transitive breaking changes: {}",
        "├─",
        analysis.required_upgrades.iter().map(|u| u.breaking_changes.len()).sum::<usize>()
    );
    println!("   {} Total packages to upgrade: {}", "├─", analysis.total_packages_affected());
    println!("   {} Overall risk: {}", "└─", format_risk_level(analysis.overall_risk));
    println!();

    // Effort estimate
    println!("{} Estimated Effort: {} hours",
        "⏱️ ".bold(),
        analysis.estimated_effort_hours
    );

    let (effort_desc, effort_breakdown) = match analysis.estimated_effort_hours {
        h if h < 1.0 => ("Quick fix", "Update dependency and run tests"),
        h if h < 4.0 => ("Moderate effort", "Update dependencies, fix breaking changes, test thoroughly"),
        h if h < 8.0 => ("Significant effort", "Plan migration, update deps, fix code, extensive testing"),
        _ => ("Major migration", "Dedicate sprint or more, careful planning required"),
    };

    println!("   {} {}", "├─", effort_desc.yellow());
    println!("   └─ {}", effort_breakdown.dimmed());
    println!();

    // Recommendation
    println!("{}", "━".repeat(60).bright_black());
    println!();
    print_recommendation(analysis);
}

fn format_risk_level(risk: bazbom_upgrade_analyzer::RiskLevel) -> ColoredString {
    use bazbom_upgrade_analyzer::RiskLevel;

    match risk {
        RiskLevel::Low => format!("{} {}", risk.emoji(), risk.label()).green().bold(),
        RiskLevel::Medium => format!("{} {}", risk.emoji(), risk.label()).yellow().bold(),
        RiskLevel::High => format!("{} {}", risk.emoji(), risk.label()).red().bold(),
        RiskLevel::Critical => format!("{} {}", risk.emoji(), risk.label()).red().bold().on_bright_white(),
    }
}

fn print_recommendation(analysis: &bazbom_upgrade_analyzer::UpgradeAnalysis) {
    use bazbom_upgrade_analyzer::RiskLevel;

    println!("{} Recommendation:", "🎯".bold());

    if analysis.is_safe() {
        println!("   {} Apply upgrade", "✅".green().bold());
        println!("   This is a low-risk upgrade with no breaking changes.");
        println!("   The main benefit is addressing vulnerabilities.");
        println!();
        println!("   {} What to do:", "💡".bright_blue());
        println!("      1. Run: bazbom fix {} --apply", analysis.target_package);
        println!("      2. Run tests to verify");
        println!("      3. Commit and deploy");
    } else {
        match analysis.overall_risk {
            RiskLevel::Low | RiskLevel::Medium => {
                println!("   {} Review before applying", "⚠️ ".yellow().bold());
                println!("   This upgrade has some breaking changes but is manageable.");
                println!();
                println!("   {} What to do:", "💡".bright_blue());
                println!("      1. Review breaking changes above");
                println!("      2. Create feature branch for testing");
                println!("      3. Run: bazbom fix {} --apply --test", analysis.target_package);
                println!("      4. Fix any compilation/test errors");
                println!("      5. Test thoroughly in staging");
                println!("      6. Merge to production");
            }
            RiskLevel::High | RiskLevel::Critical => {
                println!("   {} DO NOT APPLY IMMEDIATELY", "🚨".red().bold());
                println!("   This is a major upgrade with significant breaking changes.");
                println!();
                println!("   {} Recommended approach:", "💡".bright_blue());
                println!("      1. Schedule dedicated time ({:.1} hours)", analysis.estimated_effort_hours);
                println!("      2. Read migration guide: {}",
                    analysis.migration_guide_url.as_deref().unwrap_or("Search docs"));
                println!("      3. Create migration branch");
                println!("      4. Apply changes incrementally");
                println!("      5. Extensive testing at each step");
                println!("      6. Deploy to staging first");
                println!("      7. Monitor carefully after production deploy");
            }
        }
    }

    println!();
    println!("{}",  "━".repeat(60).bright_black());
}

/// Find current and target versions for a package
fn find_upgrade_versions(package: &str) -> Result<(String, String)> {
    // TODO: Parse from actual findings or pom.xml
    // For now, return example data based on package name
    let (current, target) = if package.contains("log4j-core") {
        ("2.17.0", "2.20.0")
    } else if package.contains("spring-boot") {
        ("2.7.0", "3.2.0")
    } else {
        // Try to read from findings
        ("1.0.0", "2.0.0") // Fallback
    };

    Ok((current.to_string(), target.to_string()))
}
