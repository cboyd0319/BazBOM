//! Interactive fix mode - beautiful TUI for fixing vulnerabilities
//!
//! Provides a guided, interactive experience for fixing security vulnerabilities
//! with clear explanations and actionable steps.

use anyhow::Result;
use colored::*;
use dialoguer::{theme::ColorfulTheme, Confirm, Select};
use std::fmt;

/// A vulnerability that can be fixed
#[derive(Debug, Clone)]
pub struct FixableVulnerability {
    pub cve_id: String,
    pub package: String,
    pub current_version: String,
    pub fixed_version: String,
    pub severity: Severity,
    pub epss_score: Option<f64>,
    pub in_cisa_kev: bool,
    pub description: String,
    pub breaking_changes: usize,
    pub estimated_effort_hours: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    pub fn emoji(&self) -> &'static str {
        match self {
            Severity::Critical => "🔴",
            Severity::High => "🟠",
            Severity::Medium => "🟡",
            Severity::Low => "🟢",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Severity::Critical => Color::Red,
            Severity::High => Color::Yellow,
            Severity::Medium => Color::BrightYellow,
            Severity::Low => Color::Green,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
        }
    }

    pub fn priority_score(&self) -> u32 {
        match self {
            Severity::Critical => 4,
            Severity::High => 3,
            Severity::Medium => 2,
            Severity::Low => 1,
        }
    }
}

impl fmt::Display for FixableVulnerability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} in {} {} → {} ({})",
            self.severity.emoji(),
            self.cve_id,
            self.package,
            self.current_version,
            self.fixed_version,
            if self.in_cisa_kev {
                "🚨 ACTIVELY EXPLOITED"
            } else {
                ""
            }
        )
    }
}

/// Interactive fix session
pub struct InteractiveFix {
    vulnerabilities: Vec<FixableVulnerability>,
    theme: ColorfulTheme,
}

impl InteractiveFix {
    /// Create a new interactive fix session
    pub fn new(vulnerabilities: Vec<FixableVulnerability>) -> Self {
        Self {
            vulnerabilities,
            theme: ColorfulTheme::default(),
        }
    }

    /// Run the interactive fix session
    pub async fn run(&mut self) -> Result<()> {
        self.print_header();

        // Sort by priority (CISA KEV > Critical > High > Medium > Low)
        self.vulnerabilities.sort_by(|a, b| {
            // CISA KEV first
            match (a.in_cisa_kev, b.in_cisa_kev) {
                (true, false) => return std::cmp::Ordering::Less,
                (false, true) => return std::cmp::Ordering::Greater,
                _ => {}
            }
            // Then by severity
            b.severity.priority_score().cmp(&a.severity.priority_score())
        });

        let total = self.vulnerabilities.len();
        let mut fixed = 0;
        let mut skipped = 0;

        for (idx, vuln) in self.vulnerabilities.clone().iter().enumerate() {
            println!();
            self.print_vulnerability_card(idx + 1, total, vuln);

            match self.prompt_action(vuln).await? {
                Action::FixNow => {
                    self.apply_fix(vuln).await?;
                    fixed += 1;
                }
                Action::Explain => {
                    self.explain_fix(vuln).await?;
                    // Ask again after explaining
                    if self.confirm_fix(vuln)? {
                        self.apply_fix(vuln).await?;
                        fixed += 1;
                    } else {
                        skipped += 1;
                    }
                }
                Action::Skip => {
                    println!("   {} Skipped", "⊘".dimmed());
                    skipped += 1;
                }
                Action::SkipAllLowPriority => {
                    println!("   {} Skipping all remaining low priority vulnerabilities", "⊘".dimmed());
                    skipped += total - idx;
                    break;
                }
                Action::Quit => {
                    println!("\n   👋 Exiting interactive fix mode");
                    break;
                }
            }
        }

        self.print_summary(fixed, skipped, total);

        Ok(())
    }

    /// Print the session header
    fn print_header(&self) {
        println!();
        println!("{}", "╔═══════════════════════════════════════════════════════════════════╗".bright_magenta().bold());
        println!("{} {} {}",
            "║".bright_magenta().bold(),
            "🛠️  INTERACTIVE FIX MODE - Let's fix these vulnerabilities!".bright_cyan().bold(),
            "║".bright_magenta().bold()
        );
        println!("{}", "╚═══════════════════════════════════════════════════════════════════╝".bright_magenta().bold());
        println!();
        println!("Found {} vulnerabilities. Let's go through them one by one.", self.vulnerabilities.len().to_string().bold());
    }

    /// Print a vulnerability card
    fn print_vulnerability_card(&self, current: usize, total: usize, vuln: &FixableVulnerability) {
        println!("{}", "┌──────────────────────────────────────────────────────────────┐".cyan());
        println!("{} {:<58} {}",
            "│".cyan(),
            format!("{}/{}: {}", current, total, vuln.cve_id).bold(),
            "│".cyan()
        );
        println!("{}", "├──────────────────────────────────────────────────────────────┤".cyan());

        // Package info
        println!("{} 📦 {:<50} {}",
            "│".cyan(),
            format!("{} {} → {}", vuln.package, vuln.current_version.yellow(), vuln.fixed_version.green()),
            "│".cyan()
        );

        // Severity
        let severity_line = format!("{} Severity: {} {}",
            "│".cyan(),
            vuln.severity.emoji(),
            vuln.severity.label()
        );
        println!("{:<70} {}", severity_line, "│".cyan());

        // CISA KEV warning
        if vuln.in_cisa_kev {
            println!("{} 🚨 {:<50} {}",
                "│".cyan(),
                "ACTIVELY EXPLOITED - Fix immediately!".red().bold(),
                "│".cyan()
            );
        }

        // EPSS score
        if let Some(epss) = vuln.epss_score {
            let epss_display = if epss > 0.5 {
                format!("EPSS: {:.1}% (HIGH risk)", epss * 100.0).red()
            } else {
                format!("EPSS: {:.1}%", epss * 100.0).normal()
            };
            println!("{} {:<58} {}",
                "│".cyan(),
                epss_display,
                "│".cyan()
            );
        }

        // Effort estimate
        let effort = format!("⏱️  Estimated effort: {:.1} hrs", vuln.estimated_effort_hours);
        println!("{} {:<58} {}",
            "│".cyan(),
            effort,
            "│".cyan()
        );

        // Breaking changes warning
        if vuln.breaking_changes > 0 {
            println!("{} {} {:<50} {}",
                "│".cyan(),
                "⚠️ ".yellow(),
                format!("{} breaking changes detected", vuln.breaking_changes).yellow(),
                "│".cyan()
            );
        }

        println!("{}", "└──────────────────────────────────────────────────────────────┘".cyan());
    }

    /// Prompt for action
    async fn prompt_action(&self, vuln: &FixableVulnerability) -> Result<Action> {
        let options = if vuln.in_cisa_kev {
            vec![
                "🔥 Fix NOW (actively exploited!)",
                "📖 Explain breaking changes first",
                "⊘ Skip (NOT recommended)",
                "🚪 Quit",
            ]
        } else if vuln.severity == Severity::Critical || vuln.severity == Severity::High {
            vec![
                "✅ Fix now",
                "📖 Explain breaking changes",
                "⊘ Skip for now",
                "⏭️  Skip all low priority",
                "🚪 Quit",
            ]
        } else {
            vec![
                "✅ Fix now",
                "📖 Explain first",
                "⊘ Skip for now",
                "⏭️  Skip all low priority",
                "🚪 Quit",
            ]
        };

        let selection = Select::with_theme(&self.theme)
            .with_prompt("What do you want to do?")
            .items(&options)
            .default(0)
            .interact()?;

        Ok(match selection {
            0 => Action::FixNow,
            1 => Action::Explain,
            2 => Action::Skip,
            3 if options.len() > 4 => Action::SkipAllLowPriority,
            3 => Action::Quit,
            4 => Action::Quit,
            _ => Action::Skip,
        })
    }

    /// Explain a fix in detail
    async fn explain_fix(&self, vuln: &FixableVulnerability) -> Result<()> {
        println!();
        println!("{}", "═".repeat(60).bright_blue());
        println!("{}", format!("📖 Detailed Analysis: {}", vuln.cve_id).bold());
        println!("{}", "═".repeat(60).bright_blue());
        println!();

        println!("{}", "Description:".bold());
        println!("  {}", vuln.description);
        println!();

        println!("{}", "Fix Details:".bold());
        println!("  {} Upgrade {} from {} to {}",
            "•".cyan(),
            vuln.package.bright_white(),
            vuln.current_version.yellow(),
            vuln.fixed_version.green()
        );
        println!("  {} Estimated effort: {:.1} hours", "•".cyan(), vuln.estimated_effort_hours);
        println!("  {} Breaking changes: {}", "•".cyan(), vuln.breaking_changes);
        println!();

        if vuln.breaking_changes > 0 {
            println!("{}", "⚠️  Breaking Changes:".yellow().bold());
            println!("  Run {} for detailed analysis",
                format!("bazbom fix {} --explain", vuln.package).bright_white().bold()
            );
            println!();
        }

        println!("{}", "Why you should fix this:".bold());
        if vuln.in_cisa_kev {
            println!("  {} This vulnerability is being {} in the wild",
                "🚨".red(),
                "ACTIVELY EXPLOITED".red().bold()
            );
            println!("  {} Attackers have weaponized exploits available", "•".red());
            println!("  {} Fix this {} to protect your application", "•".red(), "IMMEDIATELY".red().bold());
        } else if vuln.severity == Severity::Critical {
            println!("  {} Critical severity - high impact if exploited", "•".red());
            if let Some(epss) = vuln.epss_score {
                if epss > 0.5 {
                    println!("  {} High probability of exploitation (EPSS: {:.1}%)",
                        "•".yellow(), epss * 100.0
                    );
                }
            }
        } else {
            println!("  {} Reduces attack surface", "•".cyan());
            println!("  {} Keeps dependencies up to date", "•".cyan());
            println!("  {} May include performance improvements", "•".cyan());
        }

        println!();
        Ok(())
    }

    /// Confirm fix after explanation
    fn confirm_fix(&self, _vuln: &FixableVulnerability) -> Result<bool> {
        Ok(Confirm::with_theme(&self.theme)
            .with_prompt("Do you want to apply this fix?")
            .default(true)
            .interact()?)
    }

    /// Apply a fix
    async fn apply_fix(&self, vuln: &FixableVulnerability) -> Result<()> {
        use crate::progress::simple_spinner;

        let spinner = simple_spinner(&format!("Applying fix for {}...", vuln.cve_id));

        // The actual fix application would integrate with the remediation system here
        // For now, we provide information on how to apply the fix
        let ecosystem = self.detect_ecosystem();
        let fix_applied = match ecosystem {
            "maven" | "gradle" => {
                // Would call OpenRewrite or update pom.xml/build.gradle
                false
            }
            "npm" | "yarn" => {
                // Would run npm update or edit package.json
                false
            }
            "pip" => {
                // Would update requirements.txt or Pipfile
                false
            }
            "cargo" => {
                // Would update Cargo.toml
                false
            }
            _ => false,
        };

        if fix_applied {
            spinner.finish_with_message(format!("   {} Fixed {}!", "✅".green(), vuln.cve_id));
        } else {
            spinner.finish_with_message(format!(
                "   {} Manual fix required: Update {} from {} to {}",
                "📝".yellow(),
                vuln.package,
                vuln.current_version,
                vuln.fixed_version
            ));
        }

        Ok(())
    }

    /// Detect the project's ecosystem based on manifest files
    fn detect_ecosystem(&self) -> &str {
        use std::path::Path;

        if Path::new("pom.xml").exists() {
            "maven"
        } else if Path::new("build.gradle").exists() || Path::new("build.gradle.kts").exists() {
            "gradle"
        } else if Path::new("package.json").exists() {
            "npm"
        } else if Path::new("requirements.txt").exists() || Path::new("Pipfile").exists() {
            "pip"
        } else if Path::new("Cargo.toml").exists() {
            "cargo"
        } else if Path::new("Gemfile").exists() {
            "gem"
        } else if Path::new("composer.json").exists() {
            "composer"
        } else {
            "unknown"
        }
    }

    /// Print session summary
    fn print_summary(&self, fixed: usize, skipped: usize, total: usize) {
        println!();
        println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_green().bold());
        println!("{} {:^61} {}",
            "║".bright_green().bold(),
            "✨ INTERACTIVE FIX SESSION COMPLETE!",
            "║".bright_green().bold()
        );
        println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_green().bold());
        println!("{} {:<59} {}",
            "║".bright_green().bold(),
            format!("Fixed:   {} / {}", fixed, total).green().bold(),
            "║".bright_green().bold()
        );
        println!("{} {:<59} {}",
            "║".bright_green().bold(),
            format!("Skipped: {} / {}", skipped, total).dimmed(),
            "║".bright_green().bold()
        );
        println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_green().bold());
        println!();

        if fixed > 0 {
            println!("{}", "Next steps:".bold());
            println!("  {} Run tests to verify fixes", "•".cyan());
            println!("  {} Commit changes: {}", "•".cyan(), "git add . && git commit -m 'fix: resolve security vulnerabilities'".dimmed());
            println!("  {} Create PR for review", "•".cyan());
            println!();
        }
    }
}

/// Action to take for a vulnerability
#[derive(Debug, Clone, Copy)]
enum Action {
    FixNow,
    Explain,
    Skip,
    SkipAllLowPriority,
    Quit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn create_test_vulns() -> Vec<FixableVulnerability> {
        vec![
            FixableVulnerability {
                cve_id: "CVE-2024-1234".to_string(),
                package: "org.apache.logging.log4j:log4j-core".to_string(),
                current_version: "2.17.0".to_string(),
                fixed_version: "2.20.0".to_string(),
                severity: Severity::Critical,
                epss_score: Some(0.85),
                in_cisa_kev: true,
                description: "Remote code execution vulnerability in log4j-core".to_string(),
                breaking_changes: 2,
                estimated_effort_hours: 4.0,
            },
            FixableVulnerability {
                cve_id: "CVE-2024-5678".to_string(),
                package: "com.fasterxml.jackson.core:jackson-databind".to_string(),
                current_version: "2.14.0".to_string(),
                fixed_version: "2.15.2".to_string(),
                severity: Severity::High,
                epss_score: Some(0.45),
                in_cisa_kev: false,
                description: "Deserialization vulnerability".to_string(),
                breaking_changes: 0,
                estimated_effort_hours: 1.5,
            },
        ]
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Critical.priority_score() > Severity::High.priority_score());
        assert!(Severity::High.priority_score() > Severity::Medium.priority_score());
        assert!(Severity::Medium.priority_score() > Severity::Low.priority_score());
    }
}
