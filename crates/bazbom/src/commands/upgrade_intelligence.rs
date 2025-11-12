use anyhow::Result;
use bazbom_upgrade_analyzer::UpgradeAnalyzer;
use colored::*;

/// Show detailed upgrade impact analysis for a package
pub async fn explain_upgrade(package: &str) -> Result<()> {
    // Header with style
    println!();
    println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".bright_blue().bold());
    println!("{} {} {}",
        "║".bright_blue().bold(),
        format!("🔮 UPGRADE INTELLIGENCE: {}", package).bright_cyan().bold(),
        "║".bright_blue().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".bright_blue().bold());
    println!();

    // Parse package and versions from findings or pom.xml
    // For now, we'll use example data
    let (current_version, target_version) = find_upgrade_versions(package)?;

    // Analyzing indicator with animation feel
    println!("  {}", "┌─────────────────────────────────────────────────────┐".bright_black());
    println!("  │ 📊 {}                                    │",
        "Analyzing upgrade impact...".cyan().bold()
    );
    println!("  │                                                       │");
    println!("  │   {} {} {} {}              │",
        package.bright_white().bold(),
        current_version.yellow(),
        "→".bright_black(),
        target_version.green().bold()
    );
    println!("  {}", "└─────────────────────────────────────────────────────┘".bright_black());
    println!();

    // Create analyzer with progress tracking
    use bazbom::progress::MultiStepProgress;

    let steps = vec![
        "Fetching package metadata from deps.dev".to_string(),
        "Analyzing dependency graph".to_string(),
        "Checking GitHub for breaking changes".to_string(),
        "Calculating risk and effort estimates".to_string(),
    ];

    let mut progress = MultiStepProgress::new(steps);

    let mut analyzer = UpgradeAnalyzer::new()?;

    // Step 1: Fetch metadata
    progress.next_step();
    let analysis = analyzer
        .analyze_upgrade(package, &current_version, &target_version)
        .await?;

    progress.finish();
    println!();

    // Print results
    print_upgrade_analysis(&analysis);

    Ok(())
}

fn print_upgrade_analysis(analysis: &bazbom_upgrade_analyzer::UpgradeAnalysis) {
    use bazbom_upgrade_analyzer::RiskLevel;

    // Title section with gradient box
    println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".cyan().bold());
    println!("{} {:^67} {}",
        "║".cyan().bold(),
        format!("📊 ANALYSIS RESULTS"),
        "║".cyan().bold()
    );
    println!("{}", "╠═══════════════════════════════════════════════════════════════════╣".cyan().bold());
    println!("{} {:<65} {}",
        "║".cyan().bold(),
        format!("{} {} {} {}",
            analysis.target_package.bright_white().bold(),
            analysis.from_version.yellow(),
            "→".bright_black(),
            analysis.to_version.green().bold()
        ),
        "║".cyan().bold()
    );
    println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".cyan().bold());
    println!();

    // Overall risk - big and prominent
    println!("  {}", "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_yellow().bold());
    println!("  ┃  {} {:43} ┃",
        "🔍 OVERALL RISK:".bold(),
        format_risk_level(analysis.overall_risk).to_string(),
    );
    println!("  {}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛".bright_yellow().bold());
    println!();

    // Direct changes section
    println!("  📦 {}",
        format!("Direct Changes: {}", analysis.target_package).bright_white().bold()
    );
    println!("  {}", "─".repeat(65).bright_black());

    if analysis.direct_breaking_changes.is_empty() {
        println!("  {}  ✅ Breaking changes: {}", "│".bright_black(), "0".green().bold());
        println!("  {}  ✅ API compatibility: {}", "│".bright_black(), "100%".green().bold());
        println!("  {}  ✅ Risk level: {}", "│".bright_black(), format_risk_level(RiskLevel::Low));
    } else {
        println!("  {}  ⚠️  Breaking changes: {}", "│".bright_black(), analysis.direct_breaking_changes.len().to_string().red().bold());
        println!("  {}", "│".bright_black());
        for (i, change) in analysis.direct_breaking_changes.iter().enumerate() {
            let prefix = if i == analysis.direct_breaking_changes.len() - 1 { "└─" } else { "├─" };
            println!("  {}  {} {}", "│".bright_black(), prefix.yellow(), change.description);
            if let Some(hint) = &change.migration_hint {
                println!("  {}     {} {}", "│".bright_black(), "💡".bright_blue(), hint.dimmed());
            }
        }
    }
    println!();

    // Required dependency upgrades
    if !analysis.required_upgrades.is_empty() {
        println!("  ⚙️  {}",
            format!("Transitive Dependencies: {} upgrades required", analysis.required_upgrades.len()).bright_white().bold()
        );
        println!("  {}", "─".repeat(65).bright_black());

        for (idx, upgrade) in analysis.required_upgrades.iter().enumerate() {
            let is_last = idx == analysis.required_upgrades.len() - 1;
            let tree_char = if is_last { "└─" } else { "├─" };
            let tree_ext = if is_last { " " } else { "│" };

            let status_icon = if upgrade.breaking_changes.is_empty() {
                "✅"
            } else {
                upgrade.risk_level.emoji()
            };

            println!("  {} {} {} {} {} {} {}",
                "│".bright_black(),
                tree_char.cyan(),
                status_icon,
                upgrade.package.bright_white().bold(),
                upgrade.from_version.yellow(),
                "→".bright_black(),
                upgrade.to_version.green().bold()
            );

            println!("  {} {}   {} {}",
                "│".bright_black(),
                tree_ext.cyan(),
                "↳".dimmed(),
                upgrade.reason.to_string().dimmed()
            );

            if !upgrade.breaking_changes.is_empty() {
                println!("  {} {}   {} {} breaking changes:",
                    "│".bright_black(),
                    tree_ext.cyan(),
                    "⚠️ ".red(),
                    upgrade.breaking_changes.len()
                );

                for (i, change) in upgrade.breaking_changes.iter().enumerate() {
                    let change_prefix = if i == upgrade.breaking_changes.len() - 1 { "└─" } else { "├─" };
                    println!("  {} {}     {} {}",
                        "│".bright_black(),
                        tree_ext.cyan(),
                        change_prefix.yellow(),
                        change.description.dimmed()
                    );
                }
            }

            if !is_last {
                println!("  {}", "│".bright_black());
            }
        }
        println!();
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

    // Summary section with fancy box
    println!("  {}", "╔═══════════════════════════════════════════════════════════════╗".bright_magenta().bold());
    println!("  {} {:^61} {}",
        "║".bright_magenta().bold(),
        "📊 IMPACT SUMMARY",
        "║".bright_magenta().bold()
    );
    println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_magenta().bold());

    let transitive_breaking = analysis.required_upgrades.iter()
        .map(|u| u.breaking_changes.len())
        .sum::<usize>();

    println!("  {} {} Direct breaking changes:       {:>25} {}",
        "║".bright_magenta().bold(),
        "├─".cyan(),
        analysis.direct_breaking_changes.len().to_string().bright_white().bold(),
        "║".bright_magenta().bold()
    );
    println!("  {} {} Transitive breaking changes:   {:>25} {}",
        "║".bright_magenta().bold(),
        "├─".cyan(),
        transitive_breaking.to_string().bright_white().bold(),
        "║".bright_magenta().bold()
    );
    println!("  {} {} Total packages to upgrade:     {:>25} {}",
        "║".bright_magenta().bold(),
        "├─".cyan(),
        analysis.total_packages_affected().to_string().bright_white().bold(),
        "║".bright_magenta().bold()
    );
    println!("  {} {} Overall risk:                  {:>25} {}",
        "║".bright_magenta().bold(),
        "└─".cyan(),
        format_risk_badge(analysis.overall_risk),
        "║".bright_magenta().bold()
    );
    println!("  {}", "╚═══════════════════════════════════════════════════════════════╝".bright_magenta().bold());
    println!();

    // Effort estimate with visual appeal
    let (effort_desc, effort_breakdown, effort_color) = match analysis.estimated_effort_hours {
        h if h < 1.0 => ("Quick fix", "Update dependency and run tests", "green"),
        h if h < 4.0 => ("Moderate effort", "Update dependencies, fix breaking changes, test thoroughly", "yellow"),
        h if h < 8.0 => ("Significant effort", "Plan migration, update deps, fix code, extensive testing", "red"),
        _ => ("Major migration", "Dedicate sprint or more, careful planning required", "bright_red"),
    };

    let effort_hours_display = match effort_color {
        "green" => format!("{:.1} hrs", analysis.estimated_effort_hours).green().bold(),
        "yellow" => format!("{:.1} hrs", analysis.estimated_effort_hours).yellow().bold(),
        "red" => format!("{:.1} hrs", analysis.estimated_effort_hours).red().bold(),
        _ => format!("{:.1} hrs", analysis.estimated_effort_hours).bright_red().bold(),
    };

    println!("  {}", "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_green().bold());
    println!("  ┃  {} {}                                ┃",
        "⏱️  ESTIMATED EFFORT:".bold(),
        effort_hours_display
    );
    println!("  ┃  {} {}                                           ┃",
        "├─".cyan(),
        effort_desc.bright_white()
    );
    println!("  ┃  {} {}   ┃",
        "└─".cyan(),
        effort_breakdown.dimmed()
    );
    println!("  {}", "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛".bright_green().bold());
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

fn format_risk_badge(risk: bazbom_upgrade_analyzer::RiskLevel) -> ColoredString {
    use bazbom_upgrade_analyzer::RiskLevel;

    match risk {
        RiskLevel::Low => "[ LOW ]".green().bold(),
        RiskLevel::Medium => "[ MEDIUM ]".yellow().bold(),
        RiskLevel::High => "[ HIGH ]".red().bold(),
        RiskLevel::Critical => "[ CRITICAL ]".red().bold().on_bright_white(),
    }
}

fn print_recommendation(analysis: &bazbom_upgrade_analyzer::UpgradeAnalysis) {
    use bazbom_upgrade_analyzer::RiskLevel;

    if analysis.is_safe() {
        // Safe upgrade - green box
        println!("  {}", "╔═══════════════════════════════════════════════════════════════╗".bright_green().bold());
        println!("  {} {:^61} {}",
            "║".bright_green().bold(),
            "🎯 RECOMMENDATION: SAFE TO APPLY",
            "║".bright_green().bold()
        );
        println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_green().bold());
        println!("  {} ✅ This is a low-risk upgrade with no breaking changes.                                                   {}",
            "║".bright_green().bold(),
            "║".bright_green().bold()
        );
        println!("  {}    The main benefit is addressing vulnerabilities.                                                   {}",
            "║".bright_green().bold(),
            "║".bright_green().bold()
        );
        println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_green().bold());
        println!("  {} 💡 {}                                              {}",
            "║".bright_green().bold(),
            "NEXT STEPS:".bold(),
            "║".bright_green().bold()
        );
        println!("  {} {}                                        {}",
            "║".bright_green().bold(),
            format!("   1. Run: bazbom fix {} --apply", analysis.target_package).bright_white(),
            "║".bright_green().bold()
        );
        println!("  {} {}                                                   {}",
            "║".bright_green().bold(),
            "   2. Run tests to verify".bright_white(),
            "║".bright_green().bold()
        );
        println!("  {} {}                                                   {}",
            "║".bright_green().bold(),
            "   3. Commit and deploy".bright_white(),
            "║".bright_green().bold()
        );
        println!("  {}", "╚═══════════════════════════════════════════════════════════════╝".bright_green().bold());
    } else {
        match analysis.overall_risk {
            RiskLevel::Low | RiskLevel::Medium => {
                // Medium risk - yellow box
                println!("  {}", "╔═══════════════════════════════════════════════════════════════╗".bright_yellow().bold());
                println!("  {} {:^61} {}",
                    "║".bright_yellow().bold(),
                    "🎯 RECOMMENDATION: REVIEW BEFORE APPLYING",
                    "║".bright_yellow().bold()
                );
                println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_yellow().bold());
                println!("  {} ⚠️  This upgrade has some breaking changes but is manageable.                                                   {}",
                    "║".bright_yellow().bold(),
                    "║".bright_yellow().bold()
                );
                println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_yellow().bold());
                println!("  {} 💡 {}                                              {}",
                    "║".bright_yellow().bold(),
                    "RECOMMENDED APPROACH:".bold(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    "   1. Review breaking changes above".bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    "   2. Create feature branch for testing".bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    format!("   3. Run: bazbom fix {} --apply --test", analysis.target_package).bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    "   4. Fix any compilation/test errors".bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    "   5. Test thoroughly in staging".bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_yellow().bold(),
                    "   6. Merge to production".bright_white(),
                    "║".bright_yellow().bold()
                );
                println!("  {}", "╚═══════════════════════════════════════════════════════════════╝".bright_yellow().bold());
            }
            RiskLevel::High | RiskLevel::Critical => {
                // High risk - red box
                println!("  {}", "╔═══════════════════════════════════════════════════════════════╗".bright_red().bold());
                println!("  {} {:^61} {}",
                    "║".bright_red().bold(),
                    "🚨 WARNING: DO NOT APPLY IMMEDIATELY",
                    "║".bright_red().bold()
                );
                println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_red().bold());
                println!("  {}    This is a major upgrade with significant breaking changes.                                                   {}",
                    "║".bright_red().bold(),
                    "║".bright_red().bold()
                );
                println!("  {}", "╠═══════════════════════════════════════════════════════════════╣".bright_red().bold());
                println!("  {} 💡 {}                                              {}",
                    "║".bright_red().bold(),
                    "RECOMMENDED APPROACH:".bold(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    format!("   1. Schedule dedicated time ({:.1} hours)", analysis.estimated_effort_hours).bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    format!("   2. Read migration guide: {}",
                        analysis.migration_guide_url.as_deref().unwrap_or("Search docs")).bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    "   3. Create migration branch".bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    "   4. Apply changes incrementally".bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    "   5. Extensive testing at each step".bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    "   6. Deploy to staging first".bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {} {}                                                   {}",
                    "║".bright_red().bold(),
                    "   7. Monitor carefully after production deploy".bright_white(),
                    "║".bright_red().bold()
                );
                println!("  {}", "╚═══════════════════════════════════════════════════════════════╝".bright_red().bold());
            }
        }
    }

    println!();
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
