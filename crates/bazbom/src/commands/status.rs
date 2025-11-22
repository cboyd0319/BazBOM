use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

/// Handle the `bazbom status` command
pub fn handle_status(verbose: bool, findings: Option<String>) -> Result<()> {
    // Find latest scan results
    let findings_path = if let Some(path) = findings {
        PathBuf::from(path)
    } else {
        find_latest_scan_results()?
    };

    if !findings_path.exists() {
        println!("{}", "WARN  No scan results found".yellow().bold());
        println!();
        println!("Run a scan first:");
        println!("  {}", "bazbom scan".green());
        println!("  {} Quick scan", "bazbom check".green());
        return Ok(());
    }

    // Parse scan results
    let content = fs::read_to_string(&findings_path).context("Failed to read scan results")?;
    let sarif: serde_json::Value =
        serde_json::from_str(&content).context("Failed to parse SARIF")?;

    // Extract vulnerability stats
    let stats = extract_vulnerability_stats(&sarif);

    // Calculate security score
    let security_score = calculate_security_score(&stats);

    // Get last scan time
    let last_scan = get_last_scan_time(&findings_path)?;

    // Display status
    display_status(security_score, &stats, &last_scan, verbose);

    Ok(())
}

/// Find the most recent scan results file
fn find_latest_scan_results() -> Result<PathBuf> {
    let candidates = vec![
        PathBuf::from("./sca_findings.sarif"),
        PathBuf::from("./scan-results/sca_findings.sarif"),
        PathBuf::from("../sca_findings.sarif"),
    ];

    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!("No scan results found. Run 'bazbom scan' first.")
}

#[derive(Debug)]
struct VulnerabilityStats {
    total: usize,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
    reachable: usize,
}

/// Extract vulnerability statistics from SARIF
fn extract_vulnerability_stats(sarif: &serde_json::Value) -> VulnerabilityStats {
    let mut stats = VulnerabilityStats {
        total: 0,
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
        reachable: 0,
    };

    if let Some(runs) = sarif.get("runs").and_then(|r| r.as_array()) {
        for run in runs {
            if let Some(results) = run.get("results").and_then(|r| r.as_array()) {
                stats.total = results.len();

                for result in results {
                    // Check severity
                    if let Some(level) = result.get("level").and_then(|l| l.as_str()) {
                        match level {
                            "error" => stats.critical += 1,
                            "warning" => stats.high += 1,
                            "note" => stats.medium += 1,
                            _ => stats.low += 1,
                        }
                    }

                    // Check if reachable (example - adjust based on your SARIF format)
                    if let Some(properties) = result.get("properties") {
                        if let Some(reachable) =
                            properties.get("reachable").and_then(|r| r.as_bool())
                        {
                            if reachable {
                                stats.reachable += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    stats
}

/// Calculate security score (0-100) based on vulnerabilities
fn calculate_security_score(stats: &VulnerabilityStats) -> u8 {
    if stats.total == 0 {
        return 100;
    }

    let mut score: u8 = 100;

    // Deduct points based on severity
    score = score.saturating_sub((stats.critical * 15) as u8);
    score = score.saturating_sub((stats.high * 8) as u8);
    score = score.saturating_sub((stats.medium * 3) as u8);
    score = score.saturating_sub(stats.low as u8);

    // Extra penalty for reachable vulns
    score = score.saturating_sub((stats.reachable * 5) as u8);

    score
}

/// Get last scan timestamp from file metadata
fn get_last_scan_time(path: &Path) -> Result<String> {
    let metadata = fs::metadata(path)?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(std::time::UNIX_EPOCH)?;
    let seconds = duration.as_secs();

    // Convert to human-readable relative time
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let diff = now.saturating_sub(seconds);

    if diff < 60 {
        Ok(format!("{} seconds ago", diff))
    } else if diff < 3600 {
        Ok(format!("{} minutes ago", diff / 60))
    } else if diff < 86400 {
        Ok(format!("{} hours ago", diff / 3600))
    } else {
        Ok(format!("{} days ago", diff / 86400))
    }
}

/// Display security status with beautiful formatting
fn display_status(score: u8, stats: &VulnerabilityStats, last_scan: &str, verbose: bool) {
    println!();
    println!(
        "{}",
        "┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓".bright_blue()
    );
    println!(
        "{} {} {}",
        "┃".bright_blue(),
        "SHIELD  SECURITY STATUS".bold().bright_cyan(),
        "                        ┃".bright_blue()
    );
    println!(
        "{}",
        "┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫".bright_blue()
    );

    // Security score
    let score_color = if score >= 80 {
        "green"
    } else if score >= 60 {
        "yellow"
    } else {
        "red"
    };

    let score_emoji = if score >= 80 {
        "OK"
    } else if score >= 60 {
        "WARN"
    } else {
        "🚨"
    };

    println!(
        "┃  {} Security Score: {:<28} ┃",
        score_emoji,
        format!("{}/100", score).color(score_color).bold()
    );
    println!("┃                                              ┃");

    // Vulnerability counts
    if stats.total == 0 {
        println!(
            "┃  {}  {}                   ┃",
            "✨".green(),
            "NO VULNERABILITIES!".green().bold()
        );
    } else {
        println!(
            "┃  Total Vulnerabilities: {:<19} ┃",
            stats.total.to_string().bold()
        );

        if stats.critical > 0 {
            println!(
                "┃    🚨 Critical:  {:<27} ┃",
                stats.critical.to_string().red().bold()
            );
        }
        if stats.high > 0 {
            println!(
                "┃    WARN  High:      {:<27} ┃",
                stats.high.to_string().yellow().bold()
            );
        }
        if stats.medium > 0 {
            println!(
                "┃    ⚡ Medium:    {:<27} ┃",
                stats.medium.to_string().cyan()
            );
        }
        if stats.low > 0 {
            println!(
                "┃    INFO  Low:       {:<27} ┃",
                stats.low.to_string().white()
            );
        }

        println!("┃                                              ┃");

        if stats.reachable > 0 {
            println!(
                "┃  {} Reachable: {:<25} ┃",
                "TARGET".red(),
                format!(
                    "{} ({}%)",
                    stats.reachable,
                    (stats.reachable * 100) / stats.total.max(1)
                )
                .red()
                .bold()
            );
        }
    }

    println!("┃                                              ┃");
    println!("┃  Last Scan: {:<32} ┃", last_scan.dimmed());
    println!(
        "{}",
        "┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛".bright_blue()
    );
    println!();

    // Recommendations
    if stats.critical > 0 {
        println!(
            "HINT {} Immediate Action Required:",
            "CRITICAL:".red().bold()
        );
        println!("   Fix {} critical vulnerabilities ASAP", stats.critical);
        println!("   Run: {}", "bazbom fix --suggest".green());
        println!();
    } else if stats.high > 0 {
        println!("HINT {} High Priority Issues:", "WARNING:".yellow().bold());
        println!("   Address {} high-severity vulnerabilities", stats.high);
        println!("   Run: {}", "bazbom fix --suggest".green());
        println!();
    } else if stats.total == 0 {
        println!("HINT {} Your project is secure!", "SUCCESS:".green().bold());
        println!("   Run periodic scans to stay protected");
        println!("   Next: {}", "bazbom watch".green());
        println!();
    }

    // Verbose mode - show more details
    if verbose && stats.total > 0 {
        println!("{}", "📊 Detailed Breakdown:".cyan().bold());
        println!();
        println!("  Severity Distribution:");
        print_bar("  Critical", stats.critical, stats.total, "red");
        print_bar("  High", stats.high, stats.total, "yellow");
        print_bar("  Medium", stats.medium, stats.total, "cyan");
        print_bar("  Low", stats.low, stats.total, "white");
        println!();
        println!(
            "  Reachability: {}% of vulnerabilities are reachable",
            (stats.reachable * 100) / stats.total.max(1)
        );
        println!();
    }
}

/// Print a simple horizontal bar chart
fn print_bar(label: &str, count: usize, total: usize, color: &str) {
    let percentage = if total > 0 { (count * 100) / total } else { 0 };
    let bar_length = (percentage / 2).min(40); // Max 40 chars
    let bar = "█".repeat(bar_length);

    println!(
        "    {:<10} {:>3} | {} {}%",
        label,
        count,
        bar.color(color),
        percentage
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_score_calculation() {
        let stats = VulnerabilityStats {
            total: 10,
            critical: 0,
            high: 0,
            medium: 0,
            low: 10,
            reachable: 0,
        };
        assert!(calculate_security_score(&stats) >= 85);

        let critical_stats = VulnerabilityStats {
            total: 5,
            critical: 5,
            high: 0,
            medium: 0,
            low: 0,
            reachable: 3,
        };
        assert!(calculate_security_score(&critical_stats) < 50);
    }
}
