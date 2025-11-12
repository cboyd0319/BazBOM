//! Beautiful terminal output formatting
//!
//! Makes BazBOM output scannable and delightful with boxes, colors, and visual hierarchy.

use colored::*;

/// Print a fancy box with vulnerability details
pub fn print_vulnerability_box(
    cve_id: &str,
    severity: &str,
    cvss: f64,
    package: &str,
    version: &str,
    reachable: bool,
) {
    let severity_color = match severity.to_uppercase().as_str() {
        "CRITICAL" => "red",
        "HIGH" => "yellow",
        "MEDIUM" => "cyan",
        "LOW" => "white",
        _ => "white",
    };

    let emoji = match severity.to_uppercase().as_str() {
        "CRITICAL" => "🚨",
        "HIGH" => "⚠️",
        "MEDIUM" => "⚡",
        "LOW" => "ℹ️",
        _ => "📋",
    };

    println!("┌─────────────────────────────────────────────┐");
    println!(
        "│ {} {:<40} │",
        emoji,
        format!("{}: {}", severity.to_uppercase(), cve_id)
            .color(severity_color)
            .bold()
    );
    println!("├─────────────────────────────────────────────┤");
    println!(
        "│ Package:  {:<34} │",
        format!("{} {}", package, version)
    );
    println!(
        "│ Severity: {:<34} │",
        format!("{} (CVSS {:.1})", severity.to_uppercase(), cvss)
    );

    if reachable {
        println!(
            "│ Status:   {:<34} │",
            "REACHABLE ⚠️ (actively used!)".red().to_string()
        );
    } else {
        println!(
            "│ Status:   {:<34} │",
            "UNREACHABLE ✅ (dead code)".green().to_string()
        );
    }

    println!("├─────────────────────────────────────────────┤");
    println!("│ Quick Fix:                                  │");
    println!(
        "│ $ bazbom fix {} --apply             │",
        package.green()
    );
    println!("│                                             │");
    println!("│ Learn more:                                 │");
    println!("│ $ bazbom explain {}              │", cve_id.cyan());
    println!("└─────────────────────────────────────────────┘");
    println!();
}

/// Print scan summary box
pub fn print_scan_summary(
    total_vulns: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    reachable: usize,
    scan_time: f64,
) {
    println!();
    println!("┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓");
    println!("┃           📊 SCAN SUMMARY                    ┃");
    println!("┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫");

    if total_vulns == 0 {
        println!("┃                                              ┃");
        println!(
            "┃  {}  {}                  ┃",
            "✨".green().bold(),
            "NO VULNERABILITIES FOUND!".green().bold()
        );
        println!("┃  {}                         ┃", "Your code is clean! 🎉".green());
        println!("┃                                              ┃");
    } else {
        println!("┃  Total Vulnerabilities: {:<19} ┃", total_vulns.to_string().bold());
        println!("┃                                              ┃");

        if critical > 0 {
            println!(
                "┃  {} Critical:  {:<28} ┃",
                "🚨".red(),
                critical.to_string().red().bold()
            );
        }
        if high > 0 {
            println!(
                "┃  {} High:      {:<28} ┃",
                "⚠️",
                high.to_string().yellow().bold()
            );
        }
        if medium > 0 {
            println!(
                "┃  {} Medium:    {:<28} ┃",
                "⚡",
                medium.to_string().cyan()
            );
        }
        if low > 0 {
            println!(
                "┃  {} Low:       {:<28} ┃",
                "ℹ️",
                low.to_string().white()
            );
        }

        println!("┃                                              ┃");
        println!(
            "┃  {} Reachable: {:<25} ┃",
            "🎯".red(),
            format!("{} ({}%)", reachable, (reachable * 100) / total_vulns.max(1))
                .red()
                .bold()
        );
    }

    println!("┃                                              ┃");
    println!("┃  Scan Time: {:<30} ┃", format!("{:.2}s", scan_time));
    println!("┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛");
    println!();
}

/// Print smart suggestions after scan
pub fn print_suggestions(suggestions: Vec<String>) {
    if suggestions.is_empty() {
        return;
    }

    println!("💡 {} Suggestions:", "Smart".cyan().bold());
    for suggestion in suggestions {
        println!("  • {}", suggestion);
    }
    println!();
}

/// Print a success message with a box
pub fn print_success(message: &str) {
    println!();
    println!("┌─────────────────────────────────────────────┐");
    println!("│ {} {:<38} │", "✅".green(), message.green().bold());
    println!("└─────────────────────────────────────────────┘");
    println!();
}

/// Print an error message with a box
pub fn print_error(title: &str, message: &str, quick_fix: Option<&str>, docs_link: Option<&str>) {
    println!();
    println!("┌─────────────────────────────────────────────┐");
    println!("│ {} {:<38} │", "❌".red(), title.red().bold());
    println!("├─────────────────────────────────────────────┤");

    // Wrap message text to fit in box
    for line in message.lines() {
        println!("│ {:<43} │", line);
    }

    if let Some(fix) = quick_fix {
        println!("├─────────────────────────────────────────────┤");
        println!("│ {} Quick Fix:                                │", "💡".yellow());
        for line in fix.lines() {
            println!("│   {:<41} │", line.green());
        }
    }

    if let Some(link) = docs_link {
        println!("├─────────────────────────────────────────────┤");
        println!("│ {} Documentation:                            │", "📚".cyan());
        println!("│   {:<41} │", link.cyan());
    }

    println!("└─────────────────────────────────────────────┘");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vulnerability_box() {
        // Just make sure it doesn't panic
        print_vulnerability_box(
            "CVE-2024-1234",
            "CRITICAL",
            9.8,
            "log4j-core",
            "2.17.0",
            true,
        );
    }

    #[test]
    fn test_scan_summary() {
        print_scan_summary(47, 2, 10, 20, 15, 12, 23.45);
    }

    #[test]
    fn test_scan_summary_clean() {
        print_scan_summary(0, 0, 0, 0, 0, 0, 5.23);
    }
}
