//! Beautiful summary dashboards for scan results
//!
//! Provides rich, visual summaries of security scans with actionable next steps.

use colored::*;
use std::time::Duration;

/// Summary statistics from a security scan
#[derive(Debug, Default)]
pub struct ScanSummary {
    pub dependencies_scanned: usize,
    pub vulnerabilities_found: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub license_issues: usize,
    pub policy_violations: usize,
    pub scan_duration: Duration,
    pub reports_dir: String,
    pub uploaded_to_github: bool,
    pub cache_hit: bool,
    pub files_scanned: Option<usize>,
    pub targets_analyzed: Option<usize>,
}

impl ScanSummary {
    /// Create a new summary with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of findings
    pub fn total_findings(&self) -> usize {
        self.vulnerabilities_found + self.license_issues + self.policy_violations
    }

    /// Get severity emoji for a count
    #[allow(dead_code)]
    fn severity_emoji(count: usize) -> &'static str {
        if count > 0 { "⚠️ " } else { "✓" }
    }

    /// Get color for severity level
    #[allow(dead_code)]
    fn severity_color(severity: &str) -> ColoredString {
        match severity {
            "CRITICAL" => "CRITICAL".red().bold(),
            "HIGH" => "HIGH".red(),
            "MEDIUM" => "MEDIUM".yellow(),
            "LOW" => "LOW".green(),
            _ => severity.normal(),
        }
    }

    /// Print the beautiful summary dashboard
    pub fn print(&self) {
        println!();
        println!("{}", "╔═══════════════════════════════════════════════════════════════╗".bright_cyan().bold());
        println!("{} {:^61} {}",
            "║".bright_cyan().bold(),
            "📊 SCAN SUMMARY",
            "║".bright_cyan().bold()
        );
        println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Cache status
        if self.cache_hit {
            println!("{} {} {}                                                {}",
                "║".bright_cyan().bold(),
                "⚡".yellow(),
                "Cache Hit - Using Cached Results".dimmed(),
                "║".bright_cyan().bold()
            );
            println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan().bold());
        }

        // Dependencies
        if let Some(targets) = self.targets_analyzed {
            self.print_row("Bazel Targets Analyzed:", &targets.to_string().bright_white().bold().to_string());
        }
        self.print_row("Dependencies Scanned:", &self.dependencies_scanned.to_string().bright_white().bold().to_string());

        if let Some(files) = self.files_scanned {
            self.print_row("Files Scanned:", &files.to_string().bright_white().bold().to_string());
        }

        // Separator
        println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan().bold());

        // Vulnerabilities - color coded by severity
        let vuln_display = if self.vulnerabilities_found == 0 {
            format!("{} {}", "✅", self.vulnerabilities_found.to_string().green().bold())
        } else if self.critical_count > 0 || self.high_count > 5 {
            format!("{} {}", "🔴", self.vulnerabilities_found.to_string().red().bold())
        } else if self.high_count > 0 {
            format!("{} {}", "🟠", self.vulnerabilities_found.to_string().yellow().bold())
        } else {
            format!("{} {}", "🟡", self.vulnerabilities_found.to_string().yellow())
        };

        self.print_row("Vulnerabilities Found:", &vuln_display);

        if self.vulnerabilities_found > 0 {
            // Breakdown by severity
            if self.critical_count > 0 {
                self.print_row(
                    "  ├─ Critical:",
                    &format!("{:>3}  {}", self.critical_count, "🔴")
                );
            }
            if self.high_count > 0 {
                self.print_row(
                    "  ├─ High:",
                    &format!("{:>3}  {}", self.high_count, "🟠")
                );
            }
            if self.medium_count > 0 {
                self.print_row(
                    "  ├─ Medium:",
                    &format!("{:>3}  {}", self.medium_count, "🟡")
                );
            }
            if self.low_count > 0 {
                self.print_row(
                    "  └─ Low:",
                    &format!("{:>3}  {}", self.low_count, "🟢")
                );
            }
        }

        // Other issues
        if self.license_issues > 0 || self.policy_violations > 0 {
            println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan().bold());
        }

        if self.license_issues > 0 {
            self.print_row(
                "License Issues:",
                &format!("{} {}", "⚠️ ", self.license_issues.to_string().yellow().bold())
            );
        }

        if self.policy_violations > 0 {
            self.print_row(
                "Policy Violations:",
                &format!("{} {}", "❌", self.policy_violations.to_string().red().bold())
            );
        }

        // Performance metrics
        println!("{}", "╠═══════════════════════════════════════════════════════════════╣".bright_cyan().bold());
        self.print_row("⏱️  Scan Duration:", &format_duration(self.scan_duration).bright_white().bold().to_string());
        self.print_row("📁 Reports:", &self.reports_dir.bright_blue().underline().to_string());

        if self.uploaded_to_github {
            self.print_row("📤 GitHub Upload:", &"✅ Complete".green().bold().to_string());
        }

        println!("{}", "╚═══════════════════════════════════════════════════════════════╝".bright_cyan().bold());
        println!();

        // Next steps
        self.print_next_steps();
    }

    /// Print a row in the summary table
    fn print_row(&self, label: &str, value: &str) {
        // Manually strip ANSI codes for length calculation
        let value_display_len = console::strip_ansi_codes(value).len();
        let padding = 40usize.saturating_sub(value_display_len);

        println!("{} {:<23} {:>width$} {}",
            "║".bright_cyan().bold(),
            label,
            value,
            "║".bright_cyan().bold(),
            width = padding
        );
    }

    /// Print actionable next steps
    fn print_next_steps(&self) {
        println!("{}", "Next steps:".bold().bright_white());

        if self.vulnerabilities_found > 0 {
            if self.critical_count > 0 || self.high_count > 0 {
                println!("  {} Run {} to fix critical vulnerabilities",
                    "🔥".red(),
                    "'bazbom fix --interactive'".bright_white().bold()
                );
            } else {
                println!("  {} Run {} to fix vulnerabilities",
                    "•".cyan(),
                    "'bazbom fix --interactive'".bright_white().bold()
                );
            }
        }

        if self.total_findings() > 0 {
            println!("  {} View detailed report: {}",
                "•".cyan(),
                "'bazbom explore'".bright_white().bold()
            );
        }

        if !self.uploaded_to_github {
            println!("  {} Upload to GitHub: {}",
                "•".cyan(),
                "Configure GitHub Code Scanning".dimmed()
            );
        }

        if self.total_findings() == 0 {
            println!("  {}  All clear! No action needed.",
                "✨".green()
            );
        }

        println!();
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();

    if total_secs < 60 {
        format!("{}s", total_secs)
    } else if total_secs < 3600 {
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        format!("{}m {}s", mins, secs)
    } else {
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        format!("{}h {}m", hours, mins)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_display() {
        let summary = ScanSummary {
            dependencies_scanned: 1245,
            vulnerabilities_found: 15,
            critical_count: 2,
            high_count: 5,
            medium_count: 6,
            low_count: 2,
            license_issues: 3,
            policy_violations: 1,
            scan_duration: Duration::from_secs(135), // 2m 15s
            reports_dir: "./bazbom-findings".to_string(),
            uploaded_to_github: true,
            cache_hit: false,
            files_scanned: Some(456),
            targets_analyzed: Some(42),
        };

        summary.print();
    }

    #[test]
    fn test_clean_scan() {
        let summary = ScanSummary {
            dependencies_scanned: 500,
            vulnerabilities_found: 0,
            scan_duration: Duration::from_secs(45),
            reports_dir: "./bazbom-findings".to_string(),
            uploaded_to_github: false,
            ..Default::default()
        };

        summary.print();
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(45)), "45s");
        assert_eq!(format_duration(Duration::from_secs(135)), "2m 15s");
        assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
    }
}
